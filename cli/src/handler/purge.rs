use super::*;

use colored::Colorize;
use inquire::Confirm;

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

        // filter not deleted tasks
        let deleted: Vec<&Task> = tasks
            .iter()
            .filter(|task| task.status() == Status::Deleted)
            .collect();
        if deleted.is_empty() {
            println!(
                "{}",
                "No deleted tasks specified. Maybe you forgot to delete tasks first?".yellow()
            );
            return Ok(());
        }

        let action = Action::Purge;
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

fn collect_approved_ids_for_purge<'a>(
    action: &Action,
    candidates: &[&'a Task],
    original_count: usize,
) -> anyhow::Result<Vec<&'a UniqueID>> {
    let is_single = original_count == 1;
    let mut approved_ids = Vec::new();
    for (i, task) in candidates.iter().enumerate() {
        if is_single {
            let prompt = format!(
                "Permanently remove task {} '{}'?",
                task.uid, task.description
            );
            if Confirm::new(&prompt).with_default(false).prompt()? {
                approved_ids.push(&task.uid);
            }
            continue;
        }
        if i != 0 {
            println!();
        }
        match confirm_bulk(&task.uid, &task.description, action)? {
            ConfirmResult::Yes => approved_ids.push(&task.uid),
            ConfirmResult::No => continue,
            ConfirmResult::All => {
                for remaining in &candidates[i..] {
                    approved_ids.push(&remaining.uid);
                }
                break;
            }
            ConfirmResult::Quit => break,
        }
    }
    Ok(approved_ids)
}
