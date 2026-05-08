use super::{
    update::{
        Action, collect_decisions, confirm_empty_filter, get_display_id, print_result,
        validate_tasks,
    },
    *,
};

impl<TS: TaskService> Handler<TS> {
    pub(crate) fn done(&self, raw_filters: &[String], mods: &[String]) -> Result<(), CliError> {
        // TODO: route trailing text to an annotation once annotations are supported (TW modAnnotate).
        let (filter, _) = filter::parse_mutation(raw_filters, mods);
        if filter.is_empty() {
            confirm_empty_filter()?;
        }

        let tasks = self.task_service.list(&filter)?;
        validate_tasks(&tasks)?;

        let action = Action::Complete;
        let candidates = filter_pending_tasks(&tasks);
        if candidates.is_empty() {
            print_result(&action, 0);
            return Err(CliError::Partial);
        }

        let now = Local::now().timestamp();
        let completed = Timestamp::new(now).map_err(CliError::usage)?;
        let modification = TaskModification {
            description: None,
            completed: Some(Some(completed)),
            deleted: None,
        };
        let approved =
            collect_decisions(&action, &candidates, &modification, tasks.len())?.approved;
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

fn filter_pending_tasks(tasks: &[Task]) -> Vec<&Task> {
    let mut pending = Vec::new();
    for task in tasks {
        if task.status() == Status::Pending {
            pending.push(task);
        } else {
            println!(
                "Task {} '{}' is neither pending nor waiting.",
                get_display_id(task),
                task.description
            );
        }
    }
    pending
}
