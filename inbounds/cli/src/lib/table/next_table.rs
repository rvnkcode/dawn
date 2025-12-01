use chrono::Local;
use dawn::domain::task::Task;
use tabled::{
    Table,
    settings::{Color, Padding, Style, object::Rows, themes::Colorization},
};

use crate::table::NextRow;

pub struct NextTable {
    rows: Vec<NextRow>,
}

impl NextTable {
    pub fn new(tasks: impl Iterator<Item = Task>) -> anyhow::Result<Self> {
        let now = Local::now().timestamp();
        let rows = tasks
            .map(|task| NextRow::new(task, &now))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Self { rows })
    }

    pub fn len(&self) -> usize {
        self.rows.len()
    }

    pub fn render(&self) -> Table {
        let primary = Color::default();
        let secondary = Color::new("\u{1b}[48;5;234m", "\u{1b}[49m");
        let mut table = Table::new(&self.rows);
        table
            .with(Style::empty())
            .with(Colorization::rows([primary, secondary]))
            .with(Padding::new(1, 0, 0, 0))
            .modify(Rows::first(), Color::UNDERLINE);
        table
    }
}
