use chrono::{Local, TimeZone};
use dawn::domain::task::{Description, Task, TaskModification, UniqueID};
use inquire::{Confirm, Select};

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
    if let Some(Some(timestamp)) = modification.completed_at {
        let date = Local.timestamp_opt(timestamp, 0).unwrap();
        println!("  - End will be set to '{}'.", date.format("%Y-%m-%d"));
        println!("  - Status will be changed from 'pending' to 'completed'.");
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
