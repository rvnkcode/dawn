use chrono::Local;
use colored::control::SHOULD_COLORIZE;
use dawn::domain::task::Task;
use tabled::{
    Table, Tabled,
    settings::{
        Color, Padding, Style,
        object::{ObjectIterator, Rows},
        themes::Colorization,
    },
};

pub(crate) trait TableRow: Sized {
    fn new(task: Task, now: i64) -> anyhow::Result<Self>;
}

pub(crate) struct BaseTable<R> {
    rows: Vec<R>,
}

impl<R: TableRow + Tabled> BaseTable<R> {
    pub(crate) fn new(tasks: impl Iterator<Item = Task>) -> anyhow::Result<Self> {
        let now = Local::now().timestamp();
        let rows = tasks
            .map(|task| R::new(task, now))
            .collect::<anyhow::Result<Vec<_>>>()?;
        Ok(Self { rows })
    }

    pub(crate) fn count(&self) -> usize {
        self.rows.len()
    }

    pub(crate) fn render(&self) -> Table {
        let mut table = Table::new(&self.rows);
        table.with(Style::empty()).with(Padding::new(1, 0, 0, 0));
        if SHOULD_COLORIZE.should_colorize() {
            let secondary = Color::new("\u{1b}[48;5;234m", "\u{1b}[49m");
            table
                .with(Colorization::exact([secondary], Rows::new(2..).step_by(2)))
                .modify(Rows::first(), Color::UNDERLINE);
        }
        table
    }
}

#[cfg(test)]
mod tests {
    use dawn::domain::task::Index;

    use super::*;
    use crate::{table::NextRow, test_helper::task};

    #[test]
    fn render_includes_headers_and_row_data() {
        let tasks = [task(Some(Index::new(1).unwrap()), "buy milk", 0)];

        let table = BaseTable::<NextRow>::new(tasks.into_iter()).unwrap();
        let output = table.render().to_string();

        assert!(output.contains("ID"));
        assert!(output.contains("Age"));
        assert!(output.contains("Description"));
        assert!(output.contains("buy milk"));
        assert_eq!(table.count(), 1);
    }
}
