use crate::error::CliError;
use crate::table::{BaseTable, NextRow};
use colored::Colorize;
use dawn::domain::task::{Description, Filter, TaskCreation, port::TaskService};

pub(crate) struct Handler<TS: TaskService> {
    task_service: TS,
}

impl<TS: TaskService> Handler<TS> {
    pub(crate) fn new(task_service: TS) -> Self {
        Self { task_service }
    }

    pub(crate) fn add(&self, filter: &[String], words: &[String]) -> Result<(), CliError> {
        let all: Vec<&str> = filter
            .iter()
            .chain(words.iter())
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .collect();
        let description = Description::new(&all.join(" ")).map_err(CliError::usage)?;
        self.task_service.add(&TaskCreation { description })?;
        let count = self.task_service.count_pending()?;
        println!("Created task {count}.");
        Ok(())
    }

    pub(crate) fn next(&self, filter: &Filter) -> Result<(), CliError> {
        let tasks = self.task_service.list(filter)?;
        if tasks.is_empty() {
            println!("{}", "No matches.".yellow());
            return Err(CliError::NoMatch);
        }
        let table = BaseTable::<NextRow>::new(tasks.into_iter())?;
        let count = table.count();
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
