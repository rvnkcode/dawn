use crate::cli::Modification;
use crate::context::AppContext;
use colored::Colorize;
use dawn::domain::task::Description;
use dawn::domain::task::port::TaskService;

/// Handler that processes all CLI commands
pub struct Handler<TS: TaskService> {
    context: AppContext<TS>,
}

impl<TS: TaskService> Handler<TS> {
    pub fn new(context: AppContext<TS>) -> Self {
        Self { context }
    }

    pub fn add(&self, filters: &[String], args: &Modification) {
        let description =
            Self::compose_description(filters, &args.description).unwrap_or_else(|e| {
                eprintln!("{}", e.to_string().white().on_red());
                std::process::exit(1);
            });

        let task = self
            .context
            .task_service
            .add(description)
            .unwrap_or_else(|e| {
                eprintln!("Error: {}", e.to_string().white().on_red());
                std::process::exit(1);
            });

        println!("Created task {}.", task.index);
    }

    fn compose_description(
        filters: &[String],
        description: &[String],
    ) -> anyhow::Result<Description> {
        let mut parts = Vec::new();
        parts.extend_from_slice(filters);
        parts.extend_from_slice(description);

        let description_text = parts.join(" ");
        Ok(Description::new(&description_text)?)
    }
}
