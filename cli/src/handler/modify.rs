use super::{
    update::{
        Action, collect_decisions, confirm_empty_filter, print_not_pending, print_result,
        validate_tasks,
    },
    *,
};
use crate::arg::Modification;

impl<TS: TaskService> Handler<TS> {
    pub(crate) fn modify(
        &self,
        raw_filters: &[String],
        modification: &Modification,
    ) -> Result<(), CliError> {
        let (filter, new_description) = filter::parse_mutation(raw_filters, &modification.mods);
        if filter.is_empty() {
            confirm_empty_filter()?;
        }

        let tasks = self.task_service.list(&filter)?;
        validate_tasks(&tasks)?;

        let action = Action::Modify;
        let new_status_ref = modification.status.as_ref();
        let (completed, deleted) = status_to_timestamps(new_status_ref)?;
        let task_modification = TaskModification {
            description: new_description,
            completed,
            deleted,
        };
        let all_refs: Vec<&Task> = tasks.iter().collect();
        if task_modification.is_empty() {
            print_result(&action, 0);
            // print footnote for all tasks
            print_not_pending(&all_refs, new_status_ref);
            return Ok(());
        }

        // filter tasks: actually modified
        let candidates: Vec<&Task> = tasks
            .iter()
            .filter(|task| has_changes(task, &task_modification))
            .collect();
        if candidates.is_empty() {
            print_result(&action, 0);
            // print footnote for all tasks
            print_not_pending(&all_refs, new_status_ref);
            return Ok(());
        }

        let decisions = collect_decisions(&action, &candidates, &task_modification, tasks.len())?;
        let approved = decisions.approved;
        let attempted = decisions.attempted;
        if approved.is_empty() {
            print_result(&action, 0);
            print_not_pending(&attempted, new_status_ref);
            return Err(CliError::Partial);
        }

        self.task_service.modify(&task_modification, &approved)?;
        print_result(&action, approved.len());
        print_not_pending(&attempted, new_status_ref);
        if candidates.len() > approved.len() {
            return Err(CliError::Partial);
        }
        Ok(())
    }
}

type TimestampUpdate = Option<Option<Timestamp>>;

fn status_to_timestamps(
    status: Option<&Status>,
) -> Result<(TimestampUpdate, TimestampUpdate), CliError> {
    let now = Timestamp::new(Local::now().timestamp()).map_err(CliError::usage)?;
    match status {
        None => Ok((None, None)),
        Some(Status::Pending) => Ok((Some(None), Some(None))),
        Some(Status::Completed) => Ok((Some(Some(now)), Some(None))),
        Some(Status::Deleted) => Ok((None, Some(Some(now)))),
    }
}

fn has_changes(task: &Task, modification: &TaskModification) -> bool {
    if let Some(new_desc) = &modification.description
        && &task.description != new_desc
    {
        return true;
    }

    /*
     *   modification.<col>   task.<col>  new.is_some() != task.is_some()  has_changes?  DB effect
     *   -------------------  ----------  -------------------------------  ------------  -----------
     *   None                 (any)       — (outer if-let skipped)         —             not in SET
     *   Some(None)           None        false != false  → false          false         NULL → NULL
     *   Some(None)           Some(t1)    false != true   → true           true          t1 → NULL
     *   Some(Some(t))        None        true  != false  → true           true          NULL → t
     *   Some(Some(t))        Some(t1)    true  != true   → false          false         IFNULL → t1
     */
    if let Some(new_completed) = &modification.completed
        && new_completed.is_some() != task.completed.is_some()
    {
        return true;
    }
    if let Some(new_deleted) = &modification.deleted
        && new_deleted.is_some() != task.deleted.is_some()
    {
        return true;
    }
    false
}
