mod add;
mod delete;
mod done;
mod modify;
mod purge;
mod update;

use chrono::{Local, Utc};
use dawn::domain::task::{
    Description, Filter, Status, Task, TaskModification, Timestamp, port::TaskService,
};
use tabled::Tabled;
use uuid::Uuid;

use crate::{
    error::CliError,
    filter::{self, DefaultCommand},
    table::{AllRow, BaseTable, InfoTable, NextRow, base::TableRow},
};

pub(crate) struct Handler<TS: TaskService> {
    task_service: TS,
}

impl<TS: TaskService> Handler<TS> {
    pub(crate) fn new(task_service: TS) -> Self {
        Self { task_service }
    }

    pub(crate) fn all(&self, pre: &[String], post: &[String]) -> Result<(), CliError> {
        let filter = filter::parse_report(pre, post);
        let tasks = self.task_service.list(&filter)?;
        display_list_table::<AllRow>(tasks)
    }

    pub(crate) fn default(&self, raw_filters: &[String]) -> Result<(), CliError> {
        match filter::parse_default(raw_filters) {
            DefaultCommand::Next(filter) => self.next(filter),
            DefaultCommand::Info(filter) => self.info(&filter),
        }
    }

    fn next(&self, filter: Filter) -> Result<(), CliError> {
        let filter = filter.with_report_status(Status::Pending);
        let tasks = self.task_service.list(&filter)?;
        display_list_table::<NextRow>(tasks)
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

fn display_list_table<R: TableRow + Tabled>(tasks: Vec<Task>) -> Result<(), CliError> {
    if tasks.is_empty() {
        return Err(CliError::NoMatch);
    }
    let table = BaseTable::<R>::new(tasks.into_iter())?;
    println!("{}", table.render());
    println!();
    let count = table.count();
    if count == 1 {
        println!("{} task", count);
    } else {
        println!("{} tasks", count);
    }
    Ok(())
}
