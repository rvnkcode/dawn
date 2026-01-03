use super::*;

fn has_changes(task: &Task, modification: &TaskModification) -> bool {
    if let Some(new_desc) = &modification.description
        && &task.description != new_desc
    {
        return true;
    }

    if let Some(new_completed_at) = modification.completed_at {
        // Deleted task modified to completed task
        if task.deleted_at.is_some() {
            return true;
        }
        // Undo completed task to pending task
        if new_completed_at.is_none() && task.completed_at.is_some() {
            return true;
        }
        // Pending task modified to completed task
        if new_completed_at.is_some() && task.completed_at.is_none() {
            return true;
        }
    };

    if let Some(new_deleted_at) = modification.deleted_at {
        // Undo deleted task to pending task
        if new_deleted_at.is_none() && task.deleted_at.is_some() {
            return true;
        }
        // Pending task modified to deleted task
        if new_deleted_at.is_some() && task.deleted_at.is_none() {
            return true;
        }
    };

    false
}

impl<TS: TaskService> Handler<TS> {
    pub fn modify(&self, raw_filters: &[String], args: &Modification) -> anyhow::Result<()> {
        let (filter, modification) = parser::parse_filter_with_modifications(raw_filters, args)?;
        if filter.is_empty() {
            confirm_empty_filter()?;
        }

        // TODO: Refactoring?
        let tasks = self.task_service.all(&filter)?;
        if tasks.is_empty() {
            println!("{}", "No tasks specified.".yellow());
            return Ok(());
        }
        if tasks.len() > 1 {
            println!("This command will alter {} tasks.", tasks.len());
        }

        let action = Action::Modify;
        if modification.is_empty() {
            print_action_result(&action, 0);
            return Ok(());
        }

        let candidates: Vec<&Task> = tasks
            .iter()
            .filter(|task| has_changes(task, &modification))
            .collect();
        if candidates.is_empty() {
            print_action_result(&action, 0);
            return Ok(());
        }

        let approved_ids = collect_approved_ids(&action, &candidates, &modification, tasks.len())?;
        if approved_ids.is_empty() {
            print_action_result(&action, 0);
            return Ok(());
        }

        let status_changed =
            modification.completed_at.is_some() || modification.deleted_at.is_some();
        self.task_service.modify(modification, &approved_ids)?;

        print_action_result(&action, approved_ids.len());
        if !status_changed {
            Self::print_not_pending_for_ids(&tasks, &approved_ids);
        }
        Ok(())
    }

    fn print_not_pending_for_ids(tasks: &[Task], ids: &[&UniqueID]) {
        tasks
            .iter()
            .filter(|t| ids.contains(&&t.uid))
            .filter(|t| t.completed_at.is_some() || t.deleted_at.is_some())
            .for_each(|t| {
                let status = Status::get_status(t);
                let msg = format!(
                    "Note: Modified task {} is {}. You may wish to make this task pending with task {} modify --status pending",
                    t.uid, status.to_string(), t.uid,
                ).yellow();
                println!("{}", msg);
            });
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
            completed_at: None,
            deleted_at: None,
        };
        assert!(has_changes(&task, &modification));
    }

    #[test]
    fn has_changes_false_when_description_same() {
        let task = make_task("same description", Some(1));
        let modification = TaskModification {
            description: Some(Description::new("same description").unwrap()),
            completed_at: None,
            deleted_at: None,
        };
        assert!(!has_changes(&task, &modification));
    }

    #[test]
    fn has_changes_false_when_no_modification() {
        let task = make_task("description", Some(1));
        let modification = TaskModification {
            description: None,
            completed_at: None,
            deleted_at: None,
        };
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
