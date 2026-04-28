mod modify;
mod update;

use crate::error::CliError;
use crate::filter::{self, DefaultCommand};
use crate::table::{BaseTable, InfoTable, NextRow};
use chrono::Utc;
use dawn::domain::task::{
    Description, Filter, Status, Task, TaskCreation, TaskModification, UniqueID, port::TaskService,
};

// Re-export for submodules
pub(crate) use update::*;

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

    pub(crate) fn default(&self, raw_filters: &[String]) -> Result<(), CliError> {
        match filter::parse_and_determine_command(raw_filters) {
            DefaultCommand::Next(filter) => self.next(filter),
            DefaultCommand::Info(filter) => self.info(&filter),
        }
    }

    fn next(&self, filter: Filter) -> Result<(), CliError> {
        let filter = filter.with_statuses([Status::Pending]);
        let tasks = self.task_service.list(&filter)?;
        if tasks.is_empty() {
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

    fn info(&self, filter: &Filter) -> Result<(), CliError> {
        let tasks = self.task_service.list(filter)?;
        if tasks.is_empty() {
            return Err(CliError::NoMatch);
        }
        let now = Utc::now().timestamp();
        for (i, task) in tasks.iter().enumerate() {
            if i > 0 {
                println!();
            }
            let table = InfoTable::new(task, now)?;
            println!("{}", table.render());
        }
        Ok(())
    }
}
