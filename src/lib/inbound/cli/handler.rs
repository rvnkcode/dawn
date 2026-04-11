use crate::domain::task::{TaskCreation, port::TaskService};

pub struct Handler<TS: TaskService> {
    task_service: TS,
}

impl<TS: TaskService> Handler<TS> {
    pub fn new(task_service: TS) -> Self {
        Self { task_service }
    }

    pub fn add(&self, args: impl Into<TaskCreation>) -> anyhow::Result<()> {
        self.task_service.add(&args.into())?;
        let count = self.task_service.count_pending()?;
        println!("Created task {count}.");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::task::{Description, port::MockTaskService};

    fn creation(desc: &str) -> TaskCreation {
        TaskCreation {
            description: Description::new(desc).unwrap(),
        }
    }

    #[test]
    fn add_returns_ok_on_success() {
        let mut mock = MockTaskService::new();
        mock.expect_add().returning(|_| Ok(()));
        mock.expect_count_pending().returning(|| Ok(1));

        let handler = Handler::new(mock);
        assert!(handler.add(creation("test")).is_ok());
    }

    #[test]
    fn add_propagates_add_error() {
        let mut mock = MockTaskService::new();
        mock.expect_add()
            .returning(|_| Err(anyhow::anyhow!("add failed")));
        mock.expect_count_pending().never();

        let handler = Handler::new(mock);
        let result = handler.add(creation("test"));

        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "add failed");
    }

    #[test]
    fn add_propagates_count_pending_error() {
        let mut mock = MockTaskService::new();
        mock.expect_add().returning(|_| Ok(()));
        mock.expect_count_pending()
            .returning(|| Err(anyhow::anyhow!("count failed")));

        let handler = Handler::new(mock);
        let result = handler.add(creation("test"));

        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "count failed");
    }
}
