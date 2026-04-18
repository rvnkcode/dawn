use crate::table::{BaseTable, NextRow};
use colored::Colorize;
use dawn::domain::task::{Filter, TaskCreation, port::TaskService};

pub(crate) struct Handler<TS: TaskService> {
    task_service: TS,
}

impl<TS: TaskService> Handler<TS> {
    pub(crate) fn new(task_service: TS) -> Self {
        Self { task_service }
    }

    pub(crate) fn add(&self, args: impl Into<TaskCreation>) -> anyhow::Result<()> {
        self.task_service.add(&args.into())?;
        let count = self.task_service.count_pending()?;
        println!("Created task {count}.");
        Ok(())
    }

    pub(crate) fn next(&self, filter: &Filter) -> anyhow::Result<()> {
        let tasks = self.task_service.list(filter)?;
        if tasks.is_empty() {
            println!("{}", "No matches.".yellow());
            return Ok(());
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
