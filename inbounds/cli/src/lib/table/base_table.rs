use chrono::Local;
use dawn::domain::task::Task;
use tabled::{
    Table, Tabled,
    settings::{Color, Padding, Style, object::Rows, themes::Colorization},
};

pub trait TableRow: Sized {
    fn new(task: Task, now: &i64) -> anyhow::Result<Self>;
}

pub struct BaseTable<R> {
    rows: Vec<R>,
}

impl<R: TableRow + Tabled> BaseTable<R> {
    pub fn new(tasks: impl Iterator<Item = Task>) -> anyhow::Result<Self> {
        let now = Local::now().timestamp();
        let rows = tasks
            .map(|task| R::new(task, &now))
            .collect::<anyhow::Result<Vec<R>, anyhow::Error>>()?;
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
