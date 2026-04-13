use chrono::Utc;
use dawn::domain::task::Task;
use tabled::{
    Table, Tabled,
    settings::{Color, Padding, Style, object::Rows, themes::Colorization},
};

pub trait TableRow: Sized {
    fn new(task: Task, now: i64) -> anyhow::Result<Self>;
}

pub struct BaseTable<R> {
    rows: Vec<R>,
}

impl<R: TableRow + Tabled> BaseTable<R> {
    pub fn new(tasks: impl Iterator<Item = Task>) -> anyhow::Result<Self> {
        let now = Utc::now().timestamp();
        let rows = tasks
            .map(|task| R::new(task, now))
            .collect::<anyhow::Result<Vec<_>>>()?;
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{table::NextRow, test_helper::task};
    use dawn::domain::task::Index;

    #[test]
    fn new_with_empty_iterator_has_zero_len() {
        let table = BaseTable::<NextRow>::new(std::iter::empty()).unwrap();
        assert_eq!(table.len(), 0);
    }

    #[test]
    fn new_collects_all_rows() {
        let tasks = [
            task(Some(Index::new(1).unwrap()), "buy milk", 0),
            task(Some(Index::new(2).unwrap()), "walk dog", 0),
        ];
        let table = BaseTable::<NextRow>::new(tasks.into_iter()).unwrap();
        assert_eq!(table.len(), 2);
    }

    #[test]
    fn new_propagates_error_when_row_construction_fails() {
        let tasks = [task(None, "buy milk", 0)];
        assert!(BaseTable::<NextRow>::new(tasks.into_iter()).is_err());
    }

    #[test]
    fn render_includes_headers_and_row_data() {
        let tasks = [task(Some(Index::new(1).unwrap()), "buy milk", 0)];
        let table = BaseTable::<NextRow>::new(tasks.into_iter()).unwrap();
        let output = table.render().to_string();
        assert!(output.contains("ID"));
        assert!(output.contains("Age"));
        assert!(output.contains("Description"));
        assert!(output.contains("buy milk"));
    }
}
