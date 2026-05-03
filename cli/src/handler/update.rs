use std::fmt::Display;

use super::*;

use crate::table::date_format::{DATE_FMT, format_absolute};
use colored::Colorize;
use inquire::{Confirm, Select};

// Threshold for requiring individual confirmation on bulk modify operations
const BULK_CONFIRM_THRESHOLD: usize = 3;

pub(crate) fn confirm_empty_filter() -> Result<(), CliError> {
    let confirmed = Confirm::new("This command has no filter, and will modify all (including completed and deleted) tasks. Are you sure?")
                .with_default(false)
                .prompt()
                .unwrap_or(false);
    if !confirmed {
        return Err(CliError::usage(anyhow::anyhow!(
            "Command prevented from running."
        )));
    }
    Ok(())
}

pub(crate) fn validate_tasks(tasks: &[Task]) -> Result<(), CliError> {
    if tasks.is_empty() {
        return Err(CliError::NoSpecified);
    }
    if tasks.len() > 1 {
        println!("This command will alter {} tasks.", tasks.len());
    }
    Ok(())
}

/// Action type for bulk operations: determines message wording
pub(crate) enum Action {
    Modify,
    Complete,
    Delete,
    Purge,
}

impl Action {
    fn verb_present(&self) -> &'static str {
        match self {
            Action::Modify => "Modify",
            Action::Complete => "Complete",
            Action::Delete => "Delete",
            Action::Purge => "Permanently remove",
        }
    }

    fn verb_past(&self) -> &'static str {
        match self {
            Self::Modify => "Modified",
            Self::Complete => "Completed",
            Self::Delete => "Deleted",
            Self::Purge => "Purged",
        }
    }

    fn verb_ing(&self) -> &'static str {
        match self {
            Action::Modify => "Modifying",
            Action::Complete => "Completed",
            Action::Delete => "Deleting",
            Action::Purge => unreachable!("purge does not use print_action"),
        }
    }

    pub(crate) fn not_done_msg(&self) -> &'static str {
        match self {
            Action::Modify => "Task not modified.",
            Action::Complete => "Task not completed.",
            Action::Delete => "Task not deleted.",
            Action::Purge => unreachable!("purge does not use process_confirmations"),
        }
    }
}

/// Print result message (e.g., "Modified 1 task.")
pub(crate) fn print_result(action: &Action, count: usize) {
    match count {
        1 => println!("{} 1 task.", action.verb_past()),
        _ => println!("{} {} tasks.", action.verb_past(), count),
    }
}

pub(crate) fn collect_approved_ids(
    action: &Action,
    candidates: &[&Task],
    modification: &TaskModification,
    original_count: usize,
) -> anyhow::Result<Vec<Uuid>> {
    let needs_confirm = original_count >= BULK_CONFIRM_THRESHOLD;
    process_confirmations(action, candidates, modification, |i, task| {
        if !needs_confirm {
            return Ok(ConfirmResult::Yes);
        }
        if i != 0 {
            println!();
        }
        print_diff(task, modification)?;
        confirm_bulk(&get_display_id(task), &task.description, action)
    })
}

pub(crate) fn process_confirmations<F>(
    action: &Action,
    candidates: &[&Task],
    modification: &TaskModification,
    mut confirm: F,
) -> anyhow::Result<Vec<Uuid>>
where
    F: FnMut(usize, &Task) -> anyhow::Result<ConfirmResult>,
{
    let mut approved: Vec<Uuid> = Vec::new();
    for (i, task) in candidates.iter().enumerate() {
        match confirm(i, task)? {
            ConfirmResult::Yes => {
                print_action(action, task, modification);
                approved.push(task.uuid);
            }
            ConfirmResult::No => println!("{}", action.not_done_msg()),
            ConfirmResult::All => {
                for remaining in &candidates[i..] {
                    print_action(action, remaining, modification);
                    approved.push(remaining.uuid);
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

pub(crate) fn get_display_id(task: &Task) -> String {
    match &task.index {
        Some(index) => index.to_string(),
        None => task.uuid.to_string(),
    }
}

// Print diff for a task before confirmation (3+ tasks mode)
fn print_diff(task: &Task, modification: &TaskModification) -> anyhow::Result<()> {
    if let Some(new_desc) = &modification.description {
        println!(
            "  - Description will be changed from '{}' to '{}'.",
            task.description, new_desc
        );
    }

    // "End will be set" message for complete action
    if let Some(Some(timestamp)) = &modification.completed
        && task.completed.is_none()
    {
        let date = format_absolute(timestamp, &Local, DATE_FMT)?;
        println!("  - End will be set to '{}'.", date);
    }

    let new_status = if matches!(&modification.deleted, Some(Some(_))) {
        Some("deleted")
    } else if matches!(&modification.completed, Some(Some(_))) {
        Some("completed")
    } else if modification.completed == Some(None) || modification.deleted == Some(None) {
        Some("pending")
    } else {
        None
    };
    if let Some(status) = new_status {
        let old_status = task.status().to_string().to_lowercase();
        println!(
            "  - Status will be changed from '{}' to '{}'.",
            old_status, status
        );
    }

    Ok(())
}

pub(crate) enum ConfirmResult {
    Yes,  // Modify this task
    No,   // Skip this task
    All,  // Modify all remaining tasks
    Quit, // Skip all remaining tasks
}

pub(crate) fn confirm_bulk(
    display_id: &impl Display,
    description: &Description,
    action: &Action,
) -> anyhow::Result<ConfirmResult> {
    let prompt = format!(
        "{} task {} '{}'?",
        action.verb_present(),
        display_id,
        description
    );
    let options = vec!["Yes", "No", "All", "Quit"];
    let selection = Select::new(&prompt, options).prompt()?;
    match selection {
        "Yes" => Ok(ConfirmResult::Yes),
        "No" => Ok(ConfirmResult::No),
        "All" => Ok(ConfirmResult::All),
        "Quit" => Ok(ConfirmResult::Quit),
        _ => unreachable!(),
    }
}

/// Print action message for a task (e.g., "Modifying task 1 'description'.")
pub(crate) fn print_action(action: &Action, task: &Task, modification: &TaskModification) {
    let display_id = get_display_id(task);
    let desc = match &modification.description {
        Some(d) => d,
        None => &task.description,
    };
    println!("{} task {} '{}'.", action.verb_ing(), display_id, desc);
}

pub(crate) fn print_not_pending_for_ids(tasks: &[Task], modified_ids: &[Uuid]) {
    tasks
        .iter()
        .filter(|t| modified_ids.contains(&t.uuid))
        .filter(|t| t.completed.is_some() || t.deleted.is_some())
        .for_each(|t| {
            let status = t.status();
            let msg = format!(
                "Note: Modified task {} is {}. \
                 You may wish to make this task pending with: \
                 task {} modify --status pending",
                t.uuid,
                status.to_string().to_lowercase(),
                t.uuid,
            )
            .yellow();
            eprintln!("{}", msg);
        });
}
