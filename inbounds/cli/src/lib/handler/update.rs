use chrono::{Local, TimeZone};
use dawn::domain::task::{Description, Task, TaskModification, UniqueID};
use inquire::{Confirm, Select};

use crate::handler::Status;

/// Threshold for requiring individual confirmation on bulk modify operations
const BULK_CONFIRM_THRESHOLD: usize = 3;

/// Action type for bulk operations - determines message wording
pub(crate) enum Action {
    Modify,
    Complete,
}

impl Action {
    fn verb_present(&self) -> &'static str {
        match self {
            Action::Modify => "Modify",
            Action::Complete => "Complete",
        }
    }

    fn verb_past(&self) -> &'static str {
        match self {
            Action::Modify => "Modified",
            Action::Complete => "Completed",
        }
    }

    fn verb_ing(&self) -> &'static str {
        match self {
            Action::Modify => "Modifying",
            Action::Complete => "Completed",
        }
    }

    fn not_done_msg(&self) -> &'static str {
        match self {
            Action::Modify => "Task not modified.",
            Action::Complete => "Task not completed.",
        }
    }
}

pub(crate) fn confirm_empty_filter() -> anyhow::Result<()> {
    let ans = Confirm::new("This command has no filter, and will modify all (including completed and deleted) tasks. Are you sure?")
                .with_default(false)
                .prompt()?;
    if !ans {
        return Err(anyhow::anyhow!("Command prevented from running."));
    }
    Ok(())
}

/// Print diff for a task before confirmation (3+ tasks mode)
fn print_diff(task: &Task, modification: &TaskModification) {
    if let Some(new_desc) = &modification.description {
        println!(
            "  - Description will be changed from '{}' to '{}'.",
            task.description, new_desc
        );
    }

    // Print "End will be set" message if completing a task
    if let Some(Some(timestamp)) = modification.completed_at {
        if task.completed_at.is_none() {
            let date = Local.timestamp_opt(timestamp, 0).unwrap();
            println!("  - End will be set to '{}'.", date.format("%Y-%m-%d"));
        }
    }

    // Determine new status and print a single status change message
    let new_status = if matches!(modification.deleted_at, Some(Some(_))) {
        Some("deleted")
    } else if matches!(modification.completed_at, Some(Some(_))) {
        Some("completed")
    } else if modification.completed_at == Some(None) || modification.deleted_at == Some(None) {
        Some("pending")
    } else {
        None
    };

    if let Some(new_status) = new_status {
        let old_status = Status::get_status(task);
        println!(
            "  - Status will be changed from '{}' to '{}'.",
            old_status, new_status
        );
    }
    // TODO: Add diff for other attributes (project, tags, etc.)
}

pub(crate) fn get_display_id(task: &Task) -> String {
    match &task.index {
        Some(index) => index.to_string(),
        None => task.uid.to_string(),
    }
}

/// Confirmation result for bulk modify operations
enum ConfirmResult {
    Yes,  // Modify this task
    No,   // Skip this task
    All,  // Modify all remaining tasks
    Quit, // Skip all remaining tasks
}

/// Confirmation prompt for bulk modify operations (y/n/a/q)
fn confirm_bulk(
    display_id: &str,
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
fn print_action(action: &Action, task: &Task, modification: &TaskModification) {
    let display_id = get_display_id(task);
    let desc = match &modification.description {
        Some(d) => d,
        None => &task.description,
    };
    println!("{} task {} '{}'.", action.verb_ing(), display_id, desc);
}

/// Print result message (e.g., "Modified 1 task.")
pub(crate) fn print_action_result(action: &Action, count: usize) {
    match count {
        1 => println!("{} 1 task.", action.verb_past()),
        _ => println!("{} {} tasks.", action.verb_past(), count),
    }
}

/// Collect approved task IDs through bulk confirmation
pub(crate) fn collect_approved_ids<'a>(
    action: &Action,
    candidates: &[&'a Task],
    modification: &TaskModification,
    original_count: usize,
) -> anyhow::Result<Vec<&'a UniqueID>> {
    let needs_confirm = original_count >= BULK_CONFIRM_THRESHOLD;
    let mut approved: Vec<&UniqueID> = Vec::new();

    for (i, task) in candidates.iter().enumerate() {
        let display_id = get_display_id(task);

        let result = if needs_confirm {
            print_diff(task, modification);
            confirm_bulk(&display_id, &task.description, action)?
        } else {
            ConfirmResult::Yes
        };

        match result {
            ConfirmResult::Yes => {
                print_action(action, task, modification);
                approved.push(&task.uid);
                println!();
            }
            ConfirmResult::No => {
                println!("{}", action.not_done_msg());
                println!();
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

/// Filter pending tasks and print message for non-pending ones
pub(crate) fn filter_pending_tasks(tasks: &[Task]) -> Vec<&Task> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use dawn::domain::task::Index;

    fn make_task(desc: &str, index: Option<usize>, completed: bool, deleted: bool) -> Task {
        Task {
            uid: "abc12345678".parse::<UniqueID>().unwrap(),
            index: index.map(|i| Index::new(i).unwrap()),
            description: Description::new(desc).unwrap(),
            created_at: 0,
            completed_at: if completed { Some(1234567890) } else { None },
            deleted_at: if deleted { Some(1234567890) } else { None },
        }
    }

    // Tests for get_display_id()

    #[test]
    fn get_display_id_returns_index_when_present() {
        let task = make_task("test", Some(5), false, false);
        assert_eq!(get_display_id(&task), "5");
    }

    #[test]
    fn get_display_id_returns_uid_when_no_index() {
        let task = make_task("test", None, false, false);
        assert_eq!(get_display_id(&task), "abc12345678");
    }

    // Tests for filter_pending_tasks()

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
