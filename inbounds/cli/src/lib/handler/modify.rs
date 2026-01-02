use super::*;

/// Threshold for requiring individual confirmation on bulk modify operations
const BULK_CONFIRM_THRESHOLD: usize = 3;

/// Confirmation result for bulk modify operations
enum ConfirmResult {
    Yes,  // Modify this task
    No,   // Skip this task
    All,  // Modify all remaining tasks
    Quit, // Skip all remaining tasks
}

fn has_changes(task: &Task, modification: &TaskModification) -> bool {
    if let Some(new_desc) = &modification.description
        && &task.description != new_desc
    {
        return true;
    }
    false
}

fn get_display_id(task: &Task) -> String {
    match &task.index {
        Some(index) => index.to_string(),
        None => task.uid.to_string(),
    }
}

impl<TS: TaskService> Handler<TS> {
    pub fn modify(&self, raw_filters: &[String], args: &Modification) -> anyhow::Result<()> {
        let (filter, modification) = parser::parse_filter_with_modifications(raw_filters, args)?;
        if filter.is_empty() {
            let ans = Confirm::new("This command has no filter, and will modify all (including completed and deleted) tasks. Are you sure?")
                .with_default(false)
                .prompt()?;
            if !ans {
                return Err(anyhow::anyhow!("Command prevented from running."));
            }
        }

        let tasks = self.task_service.all(&filter)?;
        if tasks.is_empty() {
            println!("{}", "No tasks specified.".yellow());
            return Ok(());
        }
        if tasks.len() > 1 {
            println!("This command will alter {} tasks.", tasks.len());
        }

        if modification.is_empty() {
            Self::print_modify_result(0);
            return Ok(());
        }

        let candidates: Vec<&Task> = tasks
            .iter()
            .filter(|task| has_changes(task, &modification))
            .collect();
        if candidates.is_empty() {
            Self::print_modify_result(0);
            return Ok(());
        }

        let approved_ids = Self::collect_approved_ids(&candidates, &modification)?;
        if approved_ids.is_empty() {
            Self::print_modify_result(0);
            return Err(anyhow::anyhow!("Command prevented from running."));
        }

        self.task_service.modify(modification, &approved_ids)?;

        Self::print_modify_result(approved_ids.len());
        Self::print_not_pending_for_ids(&tasks, &approved_ids);
        Ok(())
    }

    fn collect_approved_ids<'a>(
        candidates: &[&'a Task],
        modification: &TaskModification,
    ) -> anyhow::Result<Vec<&'a UniqueID>> {
        let needs_confirm = candidates.len() >= BULK_CONFIRM_THRESHOLD;
        let mut approved: Vec<&UniqueID> = Vec::new();

        for (i, task) in candidates.iter().enumerate() {
            let display_id = get_display_id(task);
            let display_description = match &modification.description {
                Some(d) => d.to_string(),
                None => task.description.to_string(),
            };

            let result = if needs_confirm {
                Self::print_diff(task, modification);
                Self::confirm_bulk(&display_id, &display_description)?
            } else {
                ConfirmResult::Yes
            };

            match result {
                ConfirmResult::Yes => {
                    Self::print_modification(task, modification);
                    approved.push(&task.uid);
                }
                ConfirmResult::No => {
                    println!("Task not modified.");
                }
                ConfirmResult::All => {
                    for remaining in &candidates[i..] {
                        Self::print_modification(remaining, modification);
                        approved.push(&remaining.uid);
                    }
                    break;
                }
                ConfirmResult::Quit => {
                    println!("Task not modified.");
                    break;
                }
            }
        }
        Ok(approved)
    }

    fn print_modification(task: &Task, modification: &TaskModification) {
        let display_id = get_display_id(task);
        let desc = match &modification.description {
            Some(d) => d.to_string(),
            None => task.description.to_string(),
        };
        println!("Modifying task {} '{}'.", display_id, desc);
    }

    fn print_modify_result(count: usize) {
        if count == 1 {
            println!("Modified 1 task.");
        } else {
            println!("Modified {} tasks.", count);
        }
    }

    fn print_not_pending_for_ids(tasks: &[Task], ids: &[&UniqueID]) {
        tasks
            .iter()
            .filter(|t| ids.contains(&&t.uid))
            .filter(|t| t.completed_at.is_some() || t.deleted_at.is_some())
            .for_each(|t| {
                let status = if t.deleted_at.is_some() { "deleted" } else { "completed" };
                // TODO: Not implemented yet
                let msg = format!(
                    "Note: Modified task {} is {}. You may wish to make this task pending with task {} modify status:pending",
                    t.uid, status, t.uid,
                ).yellow();
                println!("{}", msg);
            });
    }

    /// Print diff for a task before confirmation (3+ tasks mode)
    fn print_diff(task: &Task, modification: &TaskModification) {
        if let Some(new_desc) = &modification.description {
            println!(
                "  - Description will be changed from '{}' to '{}'.",
                task.description, new_desc
            );
        }
        // TODO: Add diff for other attributes (project, tags, etc.)
    }

    /// Confirmation prompt for bulk modify operations (y/n/a/q)
    fn confirm_bulk(display_id: &str, description: &str) -> anyhow::Result<ConfirmResult> {
        let prompt = format!("Modify task {} '{}'?", display_id, description);
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use dawn::domain::task::{Description, Index, Task, TaskModification, UniqueID};

    fn make_task(desc: &str, index: Option<usize>) -> Task {
        Task {
            uid: "abc12345678".parse::<UniqueID>().unwrap(),
            index: index.map(|i| Index::new(i).unwrap()),
            description: Description::new(desc).unwrap(),
            created_at: 0,
            completed_at: None,
            deleted_at: None,
        }
    }

    #[test]
    fn has_changes_true_when_description_differs() {
        let task = make_task("old description", Some(1));
        let modification = TaskModification {
            description: Some(Description::new("new description").unwrap()),
        };
        assert!(has_changes(&task, &modification));
    }

    #[test]
    fn has_changes_false_when_description_same() {
        let task = make_task("same description", Some(1));
        let modification = TaskModification {
            description: Some(Description::new("same description").unwrap()),
        };
        assert!(!has_changes(&task, &modification));
    }

    #[test]
    fn has_changes_false_when_no_modification() {
        let task = make_task("description", Some(1));
        let modification = TaskModification { description: None };
        assert!(!has_changes(&task, &modification));
    }

    #[test]
    fn get_display_id_returns_index_when_present() {
        let task = make_task("test", Some(5));
        assert_eq!(get_display_id(&task), "5");
    }

    #[test]
    fn get_display_id_returns_uid_when_no_index() {
        let task = make_task("test", None);
        assert_eq!(get_display_id(&task), "abc12345678");
    }
}
