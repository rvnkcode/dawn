use super::{
    update::{
        Action, collect_decisions_with_prompt, confirm_empty_filter, get_display_id, print_result,
        validate_tasks,
    },
    *,
};

impl<TS: TaskService> Handler<TS> {
    pub(crate) fn delete(&self, raw_filters: &[String], mods: &[String]) -> Result<(), CliError> {
        // TODO: route trailing text to an annotation once annotations are supported (TW modAnnotate).
        let (filter, _) = filter::parse_mutation(raw_filters, mods);
        if filter.is_empty() {
            confirm_empty_filter()?;
        }

        let tasks = self.task_service.list(&filter)?;
        validate_tasks(&tasks)?;

        let action = Action::Delete;
        let candidates = filter_non_deleted_tasks(&tasks);
        if candidates.is_empty() {
            print_result(&action, 0);
            return Err(CliError::Partial);
        }

        let now = Local::now().timestamp();
        let deleted = Timestamp::new(now).map_err(CliError::usage)?;
        let modification = TaskModification {
            description: None,
            completed: None,
            deleted: Some(Some(deleted)),
        };
        let approved = collect_decisions_with_prompt(
            &action,
            &candidates,
            &modification,
            tasks.len(),
            |task| {
                format!(
                    "Delete task {} '{}'?",
                    get_display_id(task),
                    task.description
                )
            },
        )?
        .approved;
        if approved.is_empty() {
            print_result(&action, 0);
            return Err(CliError::Partial);
        }

        self.task_service.modify(&modification, &approved)?;
        print_result(&action, approved.len());
        if tasks.len() > approved.len() {
            return Err(CliError::Partial);
        }
        Ok(())
    }
}

fn filter_non_deleted_tasks(tasks: &[Task]) -> Vec<&Task> {
    let mut non_deleted = Vec::new();
    for task in tasks {
        if task.status() == Status::Deleted {
            println!(
                "Task {} '{}' is not deletable.",
                get_display_id(task),
                task.description
            );
        } else {
            non_deleted.push(task);
        }
    }
    non_deleted
}
