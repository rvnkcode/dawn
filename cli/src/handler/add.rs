use dawn::domain::task::TaskCreation;

use super::*;

impl<TS: TaskService> Handler<TS> {
    pub(crate) fn add(&self, filter: &[String], words: &[String]) -> Result<(), CliError> {
        let description = compose_description(filter, words)?;
        self.task_service.add(&TaskCreation { description })?;
        let count = self.task_service.count_pending()?;
        println!("Created task {count}.");
        Ok(())
    }
}

fn compose_description(filter: &[String], words: &[String]) -> Result<Description, CliError> {
    let all: Vec<&str> = filter
        .iter()
        .chain(words.iter())
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect();
    Description::new(&all.join(" ")).map_err(CliError::usage)
}
