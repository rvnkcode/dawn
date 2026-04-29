use inquire::Confirm;

use super::*;

impl<TS: TaskService> Handler<TS> {
    pub(crate) fn delete(&self, raw_filters: &[String], mods: &[String]) -> Result<(), CliError> {
        // TODO: route trailing text to an annotation once annotations are supported (TW modAnnotate).
        let (filter, _) = filter::parse_from_mods(raw_filters, mods);
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
        let approved_ids = collect_approved_ids_for_delete(&action, &candidates, &modification)?;
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

fn collect_approved_ids_for_delete<'a>(
    action: &Action,
    candidates: &[&'a Task],
    modification: &TaskModification,
) -> anyhow::Result<Vec<&'a UniqueID>> {
    let mut approved: Vec<&UniqueID> = Vec::new();
    let is_single = candidates.len() == 1;

    for (i, task) in candidates.iter().enumerate() {
        let display_id = get_display_id(task);
        let prompt = format!("Delete task {} '{}'?", display_id, task.description);

        let result = if is_single {
            // yes/no confirmation
            if Confirm::new(&prompt).with_default(false).prompt()? {
                ConfirmResult::Yes
            } else {
                ConfirmResult::No
            }
        } else {
            if i != 0 {
                println!();
            }
            // yes/no/all/quit confirmation
            confirm_bulk(&display_id, &task.description, action)?
        };

        match result {
            ConfirmResult::Yes => {
                print_action(action, task, modification);
                approved.push(&task.uid);
            }
            ConfirmResult::No => {
                println!("{}", action.not_done_msg());
            }
            ConfirmResult::All => {
                for remaining in &candidates[i..] {
                    print_action(action, remaining, modification);
                    approved.push(&remaining.uid);
                }
                break;
            }
            ConfirmResult::Quit => {
                println!("{}", action.not_done_msg());
                break;
            }
        }
    }
    Ok(approved)
}
