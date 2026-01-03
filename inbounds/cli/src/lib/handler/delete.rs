use super::*;
use chrono::Local;

fn filter_non_deleted_tasks(tasks: &[Task]) -> Vec<&Task> {
    let mut non_deleted_tasks: Vec<&Task> = Vec::new();
    for task in tasks {
        if task.deleted_at.is_some() {
            let display_id = get_display_id(task);
            println!(
                "Task {} '{}' is not deletable.",
                display_id, task.description
            );
        } else {
            non_deleted_tasks.push(task);
        }
    }
    non_deleted_tasks
}

impl<TS: TaskService> Handler<TS> {
    pub fn delete(&self, raw_filters: &[String], args: &Modification) -> anyhow::Result<()> {
        let filter = parser::parse_en_passant_filter(raw_filters, &args.description);
        if filter.is_empty() {
            confirm_empty_filter()?;
        }

        let tasks = self.task_service.all(&filter)?;
        if !validate_tasks(&tasks) {
            return Ok(());
        }

        let now = Local::now().timestamp();
        let modification = TaskModification {
            description: None,
            completed_at: None,
            deleted_at: Some(Some(now)),
        };
        let action = Action::Delete;
        let non_deleted_tasks = filter_non_deleted_tasks(&tasks);
        if non_deleted_tasks.is_empty() {
            print_action_result(&action, 0);
            return Ok(());
        }

        let approved_ids =
            collect_approved_ids(&action, &non_deleted_tasks, &modification, tasks.len())?;
        if approved_ids.is_empty() {
            print_action_result(&action, 0);
            return Ok(());
        }

        self.task_service.modify(modification, &approved_ids)?;
        print_action_result(&action, approved_ids.len());
        Ok(())
    }
}
