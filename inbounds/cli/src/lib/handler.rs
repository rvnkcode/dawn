use crate::cli::Modification;
use crate::context::AppContext;
use crate::table::NextTable;
use colored::Colorize;
use dawn::domain::task::port::TaskService;
use dawn::domain::task::{Description, TaskCreation};

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

    // TODO: Filtering
    pub fn next(&self) -> anyhow::Result<()> {
        let tasks = self.context.task_service.next()?;
        if tasks.is_empty() {
            println!("{}", "No matches.".yellow());
            return Ok(());
        }
        let table = NextTable::new(tasks.into_iter())?;
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use dawn::domain::task::Task;

    // Mock TaskService for testing
    struct MockTaskService;
    impl TaskService for MockTaskService {
        fn add(&self, _req: TaskCreation) -> anyhow::Result<()> {
            unimplemented!("Not needed for the tests")
        }
        fn count_pending(&self) -> usize {
            unimplemented!("Not needed for the tests")
        }
        fn next(&self) -> anyhow::Result<Vec<Task>> {
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
