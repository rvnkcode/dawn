use super::*;
use chrono::Local;

impl<TS: TaskService> Handler<TS> {
    pub fn done(&self, raw_filters: &[String], args: &Modification) -> anyhow::Result<()> {
        let filter = parser::parse_en_passant_filter(raw_filters, &args.description);
        if filter.is_empty() {
            confirm_empty_filter()?;
        }

        let tasks = self.task_service.all(&filter)?;
        if tasks.is_empty() {
            println!("{}", "No tasks specified.".yellow());
            return Ok(());
        }
        if tasks.len() > 1 {
            println!("This command will alter {} tasks.", tasks.len());
        }

        let now = Local::now().timestamp();
        let modification = TaskModification {
            description: None,
            completed_at: Some(Some(now)),
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
