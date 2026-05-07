use colored::Colorize;
use inquire::Confirm;

use crate::handler::update::get_display_id;

use super::{
    update::{Action, ConfirmResult, confirm_bulk, confirm_empty_filter, print_result},
    *,
};

impl<TS: TaskService> Handler<TS> {
    pub(crate) fn purge(&self, pre: &[String], post: &[String]) -> Result<(), CliError> {
        let filter = filter::parse_report(pre, post);
        if filter.is_empty() {
            confirm_empty_filter()?;
        }

        let tasks = self.task_service.list(&filter)?;
        if tasks.is_empty() {
            return Err(CliError::NoSpecified);
        }

        let action = Action::Purge;
        // keep only deleted tasks
        let deleted: Vec<&Task> = tasks
            .iter()
            .filter(|task| task.status() == Status::Deleted)
            .collect();
        if deleted.is_empty() {
            print_result(&action, 0);
            eprintln!(
                "{}",
                "No deleted tasks specified. Maybe you forgot to delete tasks first?".yellow()
            );
            return Ok(());
        }

        let approved = collect_approved_ids_for_purge(&action, &deleted, tasks.len())?;
        if approved.is_empty() {
            print_result(&action, 0);
            return Ok(());
        }

        self.task_service.purge(&approved)?;
        print_result(&action, approved.len());
        Ok(())
    }
}

fn collect_approved_ids_for_purge(
    action: &Action,
    candidates: &[&Task],
    original_count: usize,
) -> anyhow::Result<Vec<Uuid>> {
    let is_single = original_count == 1;
    let mut approved_ids = Vec::new();
    for (i, task) in candidates.iter().enumerate() {
        let display_id = get_display_id(task);
        if is_single {
            let prompt = format!(
                "Permanently remove task {} '{}'?",
                display_id, task.description
            );
            if Confirm::new(&prompt).with_default(false).prompt()? {
                approved_ids.push(task.uuid);
            }
            continue;
        }
        if i != 0 {
            println!();
        }
        match confirm_bulk(&display_id, &task.description, action)? {
            ConfirmResult::Yes => approved_ids.push(task.uuid),
            ConfirmResult::No => continue,
            ConfirmResult::All => {
                for remaining in &candidates[i..] {
                    approved_ids.push(remaining.uuid);
                }
                break;
            }
            ConfirmResult::Quit => break,
        }
    }
    Ok(approved_ids)
}
