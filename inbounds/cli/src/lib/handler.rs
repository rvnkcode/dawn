use crate::cli::Modification;
use crate::context::AppContext;
use dawn::domain::task::port::TaskService;
use dawn::domain::task::{Description, Task};

/// Handler that processes all CLI commands
pub struct Handler<TS: TaskService> {
    context: AppContext<TS>,
}

impl<TS: TaskService> Handler<TS> {
    pub fn new(context: AppContext<TS>) -> Self {
        Self { context }
    }

    pub fn add(&self, filters: &[String], args: &Modification) -> anyhow::Result<Task> {
        let description = Self::compose_description(filters, &args.description)?;
        self.context.task_service.add(description)
    }

    fn compose_description(
        filters: &[String],
        description: &[String],
    ) -> anyhow::Result<Description> {
        let description_text = filters
            .iter()
            .chain(description.iter())
            .map(|s| s.as_str())
            .collect::<Vec<_>>()
            .join(" ");

        Ok(Description::new(&description_text)?)
    }
}
