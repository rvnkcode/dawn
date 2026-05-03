use super::{
    update::{
        Action, collect_approved_ids, confirm_empty_filter, print_not_pending_for_ids,
        print_result, validate_tasks,
    },
    *,
};

impl<TS: TaskService> Handler<TS> {
    pub(crate) fn modify(&self, raw_filters: &[String], mods: &[String]) -> Result<(), CliError> {
        let (filter, new_description) = filter::parse_mutation(raw_filters, mods);
        if filter.is_empty() {
            confirm_empty_filter()?;
        }

        let tasks = self.task_service.list(&filter)?;
        validate_tasks(&tasks)?;
        let action = Action::Modify;
        let modification = TaskModification {
            description: new_description,
            completed: None,
            deleted: None,
        };
        if modification.is_empty() {
            print_result(&action, 0);
            return Ok(());
        }

        let candidates: Vec<&Task> = tasks
            .iter()
            .filter(|task| has_changes(task, &modification))
            .collect();
        if candidates.is_empty() {
            print_result(&action, 0);
            return Ok(());
        }

        let approved_ids = collect_approved_ids(&action, &candidates, &modification, tasks.len())?;
        if approved_ids.is_empty() {
            print_result(&action, 0);
            return Err(CliError::Partial);
        }

        self.task_service.modify(&modification, &approved_ids)?;
        print_result(&action, approved_ids.len());
        print_not_pending_for_ids(&tasks, &approved_ids);
        if candidates.len() > approved_ids.len() {
            return Err(CliError::Partial);
        }
        Ok(())
    }
}

fn has_changes(task: &Task, modification: &TaskModification) -> bool {
    if let Some(new_desc) = &modification.description
        && &task.description != new_desc
    {
        return true;
    }
    false
}
