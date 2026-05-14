use colored::Colorize;
use inquire::{Confirm, Select};

use super::*;
use crate::table::{
    date_format::{DATE_FMT, format_absolute},
    get_prefix,
};

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

pub(crate) struct UserDecisions<'a> {
    pub approved: Vec<Uuid>,
    pub attempted: Vec<&'a Task>,
}

pub(crate) fn collect_decisions<'a>(
    action: &Action,
    candidates: &[&'a Task],
    modification: &TaskModification,
    original_count: usize,
) -> anyhow::Result<UserDecisions<'a>> {
    let needs_confirm = original_count >= BULK_CONFIRM_THRESHOLD;
    process_confirmations(action, candidates, modification, |i, task| {
        if !needs_confirm {
            return Ok(ConfirmResult::Yes);
        }
        if i != 0 {
            println!();
        }
        print_diff(action, task, modification)?;
        confirm_bulk(task, action, modification)
    })
}

// Delete command has a different confirm threshold from modify/done
pub(crate) fn collect_decisions_with_prompt<'a>(
    action: &Action,
    candidates: &[&'a Task],
    modification: &TaskModification,
    original_count: usize,
    single_prompt: impl Fn(&Task) -> String,
) -> anyhow::Result<UserDecisions<'a>> {
    let is_single = original_count == 1;
    process_confirmations(action, candidates, modification, |i, task| {
        if is_single {
            let prompt = single_prompt(task);
            return Ok(if Confirm::new(&prompt).with_default(false).prompt()? {
                ConfirmResult::Yes
            } else {
                ConfirmResult::No
            });
        }
        if i != 0 {
            println!();
        }
        confirm_bulk(task, action, modification)
    })
}

// Print diff for a task before confirmation (3+ tasks mode)
fn print_diff(action: &Action, task: &Task, modification: &TaskModification) -> anyhow::Result<()> {
    if let Some(new_desc) = &modification.description
        && new_desc != &task.description
    {
        println!(
            "  - Description will be changed from '{}' to '{}'.",
            task.description, new_desc
        );
    }

    // "End will be set" only fires via `done` command
    if matches!(action, Action::Complete)
        && let Some(Some(timestamp)) = &modification.completed
        && task.completed.is_none()
    {
        let date = format_absolute(timestamp, &Local, DATE_FMT)?;
        println!("  - End will be set to '{}'.", date);
    }

    // if deleted timestamp set
    let new_status = if matches!(&modification.deleted, Some(Some(_))) {
        Some("deleted")
    // else if completed timestamp set
    } else if matches!(&modification.completed, Some(Some(_))) {
        Some("completed")
    } else if modification.completed == Some(None) || modification.deleted == Some(None) {
        Some("pending")
    } else {
        None
    };
    if let Some(status) = new_status {
        let old_status = task.status().to_string().to_lowercase();
        if old_status != status {
            println!(
                "  - Status will be changed from '{}' to '{}'.",
                old_status, status
            );
        }
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
    task: &Task,
    action: &Action,
    modification: &TaskModification,
) -> anyhow::Result<ConfirmResult> {
    let display_id = get_display_id(task);
    let desc = match &modification.description {
        Some(d) => d,
        None => &task.description,
    };
    let prompt = format!("{} task {} '{}'?", action.verb_present(), display_id, desc);
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

pub(crate) fn process_confirmations<'a, F>(
    action: &Action,
    candidates: &[&'a Task],
    modification: &TaskModification,
    mut confirm: F,
) -> anyhow::Result<UserDecisions<'a>>
where
    F: FnMut(usize, &Task) -> anyhow::Result<ConfirmResult>,
{
    let mut approved: Vec<Uuid> = Vec::new();
    let mut attempted: Vec<&'a Task> = Vec::new();
    for (i, &task) in candidates.iter().enumerate() {
        attempted.push(task);
        match confirm(i, task)? {
            ConfirmResult::Yes => {
                approved.push(task.uuid);
                print_action(action, task, modification);
            }
            ConfirmResult::No => println!("{}", action.not_done_msg()),
            ConfirmResult::All => {
                approved.push(task.uuid);
                print_action(action, task, modification);
                for &remaining in &candidates[i + 1..] {
                    attempted.push(remaining);
                    approved.push(remaining.uuid);
                    print_action(action, remaining, modification);
                }
                break;
            }
            ConfirmResult::Quit => {
                println!("{}", action.not_done_msg());
                break;
            }
        }
    }
    Ok(UserDecisions {
        approved,
        attempted,
    })
}

pub(crate) fn get_display_id(task: &Task) -> String {
    match &task.index {
        Some(index) => index.to_string(),
        None => get_prefix(&task.uuid),
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

pub(crate) fn print_not_pending(attempted: &[&Task], new_status: Option<&Status>) {
    attempted
        .iter()
        .filter(|t| {
            (t.completed.is_some() || t.deleted.is_some())
                && new_status.is_none_or(|s| s == &t.status())
        })
        .for_each(|t| {
            let status = t.status();
            let uuid_prefix = get_display_id(t);
            let msg = format!(
                "Note: Modified task {} is {}. \
                 You may wish to make this task pending with: \
                 task {} modify --status pending",
                uuid_prefix,
                status.to_string().to_lowercase(),
                uuid_prefix,
            )
            .yellow();
            eprintln!("{}", msg);
        });
}
