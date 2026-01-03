use super::*;
use chrono::Local;

fn filter_pending_tasks(tasks: &[Task]) -> Vec<&Task> {
    let mut pending: Vec<&Task> = Vec::new();
    for task in tasks {
        if task.completed_at.is_some() || task.deleted_at.is_some() {
            let display_id = get_display_id(task);
            println!(
                "Task {} '{}' is neither pending nor waiting.",
                display_id, task.description
            );
        } else {
            pending.push(task);
        }
    }
    pending
}

impl<TS: TaskService> Handler<TS> {
    pub fn done(&self, raw_filters: &[String], args: &Modification) -> anyhow::Result<()> {
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
            completed_at: Some(Some(now)),
            deleted_at: None,
        };
        let action = Action::Complete;
        let pending_tasks = filter_pending_tasks(&tasks);
        if pending_tasks.is_empty() {
            print_action_result(&action, 0);
            return Ok(());
        }

        let approved_ids =
            collect_approved_ids(&action, &pending_tasks, &modification, tasks.len())?;
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
    fn filter_pending_tasks_returns_pending_only() {
        let tasks = vec![
            make_task("pending", Some(1), false, false),
            make_task("completed", Some(2), true, false),
            make_task("deleted", Some(3), false, true),
        ];
        let pending = filter_pending_tasks(&tasks);
        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0].description.to_string(), "pending");
    }

    #[test]
    fn filter_pending_tasks_returns_empty_when_all_completed() {
        let tasks = vec![
            make_task("completed1", Some(1), true, false),
            make_task("completed2", Some(2), true, false),
        ];
        let pending = filter_pending_tasks(&tasks);
        assert!(pending.is_empty());
    }

    #[test]
    fn filter_pending_tasks_excludes_deleted_tasks() {
        let tasks = vec![
            make_task("deleted1", Some(1), false, true),
            make_task("deleted2", Some(2), false, true),
        ];
        let pending = filter_pending_tasks(&tasks);
        assert!(pending.is_empty());
    }

    #[test]
    fn filter_pending_tasks_returns_all_when_all_pending() {
        let tasks = vec![
            make_task("pending1", Some(1), false, false),
            make_task("pending2", Some(2), false, false),
            make_task("pending3", Some(3), false, false),
        ];
        let pending = filter_pending_tasks(&tasks);
        assert_eq!(pending.len(), 3);
    }
}
