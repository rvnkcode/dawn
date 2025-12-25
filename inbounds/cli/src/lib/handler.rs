use crate::cli::Modification;
use crate::context::AppContext;
use crate::parser;
use crate::table::{AllRow, BaseTable, NextRow, TableRow};
use colored::Colorize;
use dawn::domain::task::port::TaskService;
use dawn::domain::task::{Description, Task, TaskCreation, TaskModification, UniqueID};
use inquire::{Confirm, Select};
use tabled::Tabled;

/// Threshold for requiring individual confirmation on bulk modify operations
const BULK_CONFIRM_THRESHOLD: usize = 3;

/// Confirmation result for bulk modify operations
enum ConfirmResult {
    Yes,  // Modify this task
    No,   // Skip this task
    All,  // Modify all remaining tasks
    Quit, // Skip all remaining tasks
}

pub struct Handler<TS: TaskService> {
    context: AppContext<TS>,
}

impl<TS: TaskService> Handler<TS> {
    pub fn new(context: AppContext<TS>) -> Self {
        Self { context }
    }

    pub fn add(&self, filters: &[String], args: &Modification) -> anyhow::Result<()> {
        let description = Self::compose_description(filters, &args.description)?;
        let request = TaskCreation { description };
        self.context.task_service.add(request)?;
        let count = self.context.task_service.count_pending();
        println!("Created task {}.", count);
        Ok(())
    }

    fn compose_description(
        filters: &[String],
        description: &[String],
    ) -> anyhow::Result<Description> {
        let description_text = filters
            .iter()
            .chain(description.iter())
            .map(|s| s.trim())
            .collect::<Vec<_>>()
            .join(" ");
        Ok(Description::new(&description_text)?)
    }

    fn display_table<R: TableRow + Tabled>(&self, tasks: Vec<Task>) -> anyhow::Result<()> {
        if tasks.is_empty() {
            println!("{}", "No matches.".yellow());
            return Ok(());
        }
        let table = BaseTable::<R>::new(tasks.into_iter())?;
        let count = table.len();
        println!("{}", table.render());
        println!();
        if count == 1 {
            println!("{} task", count);
        } else {
            println!("{} tasks", count);
        }
        Ok(())
    }

    pub fn next(&self, raw_filters: &[String]) -> anyhow::Result<()> {
        let filter = parser::parse_filter(raw_filters);
        let tasks = self.context.task_service.next(&filter)?;
        self.display_table::<NextRow>(tasks)
    }

    pub fn all(&self, raw_filters: &[String], args: &Modification) -> anyhow::Result<()> {
        let filter = parser::parse_en_passant_filter(raw_filters, &args.description);
        let tasks = self.context.task_service.all(&filter)?;
        self.display_table::<AllRow>(tasks)
    }

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

        let tasks = self.context.task_service.all(&filter)?;
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
            .filter(|task| Self::has_changes(task, &modification))
            .collect();
        if candidates.is_empty() {
            Self::print_modify_result(0);
            return Ok(());
        }

        let approved_ids = self.collect_approved_ids(&candidates, &modification)?;
        if !approved_ids.is_empty() {
            self.context
                .task_service
                .modify(modification, &approved_ids)?;
        }

        Self::print_modify_result(approved_ids.len());
        Self::print_not_pending_for_ids(&tasks, &approved_ids);
        Ok(())
    }

    fn has_changes(task: &Task, modification: &TaskModification) -> bool {
        if let Some(new_desc) = &modification.description {
            if &task.description != new_desc {
                return true;
            }
        }
        false
    }

    fn collect_approved_ids<'a>(
        &self,
        candidates: &[&'a Task],
        modification: &TaskModification,
    ) -> anyhow::Result<Vec<&'a UniqueID>> {
        let needs_confirm = candidates.len() >= BULK_CONFIRM_THRESHOLD;
        let mut approved: Vec<&UniqueID> = Vec::new();

        for (i, task) in candidates.iter().enumerate() {
            let display_id = Self::get_display_id(task);
            let display_description = modification
                .description
                .as_ref()
                .map(|d| d.to_string())
                .unwrap_or_else(|| task.description.to_string());

            let result = if needs_confirm {
                Self::print_diff(task, modification);
                Self::confirm_bulk(&display_id, &display_description)?
            } else {
                ConfirmResult::Yes
            };

            match result {
                ConfirmResult::Yes => {
                    Self::approve_task(task, modification, &mut approved);
                }
                ConfirmResult::No => {
                    println!("Task not modified.");
                }
                ConfirmResult::All => {
                    for remaining in &candidates[i..] {
                        Self::approve_task(remaining, modification, &mut approved);
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

    fn approve_task<'a>(
        task: &'a Task,
        modification: &TaskModification,
        approved: &mut Vec<&'a UniqueID>,
    ) {
        let display_id = Self::get_display_id(task);
        let desc = modification
            .description
            .as_ref()
            .map(|d| d.to_string())
            .unwrap_or_else(|| task.description.to_string());
        println!("Modifying task {} '{}'.", display_id, desc);
        approved.push(&task.uid);
    }

    fn get_display_id(task: &Task) -> String {
        if let Some(index) = &task.index {
            format!("{}", index)
        } else {
            format!("{}", task.uid)
        }
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
    use dawn::domain::{Filter, task::Task};

    // Mock TaskService for testing
    struct MockTaskService;
    impl TaskService for MockTaskService {
        fn add(&self, _req: TaskCreation) -> anyhow::Result<()> {
            unimplemented!("Not needed for the tests")
        }
        fn count_pending(&self) -> usize {
            unimplemented!("Not needed for the tests")
        }
        fn next(&self, _filter: &Filter) -> anyhow::Result<Vec<Task>> {
            unimplemented!("Not needed for the tests")
        }
        fn all(&self, _filter: &Filter) -> anyhow::Result<Vec<Task>> {
            unimplemented!("Not needed for the tests")
        }
        fn modify(
            &self,
            _modification: TaskModification,
            _targets: &[&UniqueID],
        ) -> anyhow::Result<()> {
            unimplemented!("Not needed for the tests")
        }
    }

    use crate::utils::strs;

    type TestHandler = Handler<MockTaskService>;

    #[test]
    fn test_compose_description_with_filters_and_description() {
        let filters = strs(&["urgent", "work"]);
        let description = strs(&["complete", "project"]);

        let result = TestHandler::compose_description(&filters, &description);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().to_string(), "urgent work complete project");
    }

    #[test]
    fn test_compose_description_with_only_description() {
        let filters = strs(&[]);
        let description = strs(&["buy", "milk"]);

        let result = TestHandler::compose_description(&filters, &description);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().to_string(), "buy milk");
    }

    #[test]
    fn test_compose_description_with_only_filters() {
        let filters = strs(&["urgent", "task"]);
        let description = strs(&[]);

        let result = TestHandler::compose_description(&filters, &description);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().to_string(), "urgent task");
    }

    #[test]
    fn test_compose_description_empty_arrays() {
        let filters = strs(&[]);
        let description = strs(&[]);

        let result = TestHandler::compose_description(&filters, &description);
        assert!(result.is_err());
    }

    #[test]
    fn test_compose_description_whitespace_only() {
        let filters = strs(&["  "]);
        let description = strs(&["   "]);

        let result = TestHandler::compose_description(&filters, &description);
        assert!(result.is_err());
    }

    #[test]
    fn test_compose_description_trims_whitespace() {
        let filters = strs(&["  urgent  "]);
        let description = strs(&["  task  "]);

        let result = TestHandler::compose_description(&filters, &description);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().to_string(), "urgent task");
    }

    #[test]
    fn test_compose_description_single_word() {
        let filters = strs(&[]);
        let description = strs(&["hello"]);

        let result = TestHandler::compose_description(&filters, &description);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().to_string(), "hello");
    }
}
