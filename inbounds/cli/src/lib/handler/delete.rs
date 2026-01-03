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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::make_task;

    #[test]
    fn filter_non_deleted_tasks_returns_non_deleted_only() {
        let tasks = vec![
            make_task("pending", Some(1), false, false),
            make_task("completed", Some(2), true, false),
            make_task("deleted", Some(3), false, true),
        ];
        let non_deleted = filter_non_deleted_tasks(&tasks);
        assert_eq!(non_deleted.len(), 2);
        assert_eq!(non_deleted[0].description.to_string(), "pending");
        assert_eq!(non_deleted[1].description.to_string(), "completed");
    }

    #[test]
    fn filter_non_deleted_tasks_returns_empty_when_all_deleted() {
        let tasks = vec![
            make_task("deleted1", Some(1), false, true),
            make_task("deleted2", Some(2), false, true),
        ];
        let non_deleted = filter_non_deleted_tasks(&tasks);
        assert!(non_deleted.is_empty());
    }

    #[test]
    fn filter_non_deleted_tasks_returns_all_when_none_deleted() {
        let tasks = vec![
            make_task("pending1", Some(1), false, false),
            make_task("pending2", Some(2), false, false),
            make_task("completed", Some(3), true, false),
        ];
        let non_deleted = filter_non_deleted_tasks(&tasks);
        assert_eq!(non_deleted.len(), 3);
    }
}
