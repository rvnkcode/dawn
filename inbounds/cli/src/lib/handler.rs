mod add;
mod delete;
mod done;
mod modify;
pub mod status;
pub use status::Status;
mod update;

use crate::cli::Modification;
use crate::parser;
use crate::table::{AllRow, BaseTable, NextRow, TableRow};
use colored::Colorize;
use dawn::domain::task::port::TaskService;
use dawn::domain::task::{Description, Task, TaskCreation, TaskModification, UniqueID};
use tabled::Tabled;

// Re-export for submodules
pub(crate) use update::*;

pub struct Handler<TS: TaskService> {
    task_service: TS,
}

impl<TS: TaskService> Handler<TS> {
    pub fn new(task_service: TS) -> Self {
        Self { task_service }
    }

    pub fn next(&self, raw_filters: &[String]) -> anyhow::Result<()> {
        let filter = parser::parse_filter(raw_filters);
        let tasks = self.task_service.next(&filter)?;
        Self::display_table::<NextRow>(tasks)
    }

    pub fn all(&self, raw_filters: &[String], args: &Modification) -> anyhow::Result<()> {
        let filter = parser::parse_en_passant_filter(raw_filters, &args.description);
        let tasks = self.task_service.all(&filter)?;
        Self::display_table::<AllRow>(tasks)
    }

    fn display_table<R: TableRow + Tabled>(tasks: Vec<Task>) -> anyhow::Result<()> {
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
}
