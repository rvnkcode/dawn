use crate::table::date_format::{format_absolute, format_with_age};
use colored::control::SHOULD_COLORIZE;
use dawn::domain::task::Task;
use tabled::{
    Table, Tabled,
    settings::{Color, Padding, Style, object::Rows, themes::Colorization},
};

#[derive(Tabled)]
#[tabled(rename_all = "PascalCase")]
struct InfoRow {
    name: String,
    value: String,
}

pub(crate) struct InfoTable {
    rows: Vec<InfoRow>,
}

impl InfoTable {
    pub(crate) fn new(task: &Task, now: i64) -> anyhow::Result<Self> {
        let mut rows = vec![
            InfoRow {
                name: "ID".to_string(),
                value: task
                    .index
                    .as_ref()
                    .map_or_else(|| "-".to_string(), ToString::to_string),
            },
            InfoRow {
                name: "Description".to_string(),
                value: task.description.to_string(),
            },
            InfoRow {
                name: "Status".to_string(),
                value: task.status().to_string(),
            },
            InfoRow {
                name: "Entered".to_string(),
                value: format_with_age(&task.entry, now)?,
            },
        ];
        if let Some(completed) = &task.completed {
            rows.push(InfoRow {
                name: "End".to_string(),
                value: format_absolute(completed)?.to_string(),
            });
        }
        if let Some(deleted) = &task.deleted {
            rows.push(InfoRow {
                name: "Deleted".to_string(),
                value: format_absolute(deleted)?.to_string(),
            });
        }
        rows.push(InfoRow {
            name: "Last modified".to_string(),
            value: format_with_age(&task.modified, now)?,
        });
        rows.push(InfoRow {
            name: "UID".to_string(),
            value: task.uid.to_string(),
        });
        Ok(Self { rows })
    }

    pub(crate) fn render(&self) -> Table {
        let primary = Color::default();
        let secondary = Color::new("\u{1b}[48;5;234m", "\u{1b}[49m");
        let mut table = Table::new(&self.rows);
        table.with(Style::empty()).with(Padding::new(1, 0, 0, 0));
        if SHOULD_COLORIZE.should_colorize() {
            table
                .with(Colorization::rows([primary, secondary]))
                .modify(Rows::first(), Color::UNDERLINE);
        }
        table
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helper::task;
    use dawn::domain::task::{Index, Timestamp};

    #[test]
    fn render_includes_base_rows_for_pending_task() {
        let now = 1_000_000;
        let t = task(Some(Index::new(1).unwrap()), "buy milk", now - 30);
        let uid = t.uid.to_string();

        let table = InfoTable::new(&t, now).unwrap();
        let output = table.render().to_string();

        assert!(output.contains("ID"));
        assert!(output.contains("Description"));
        assert!(output.contains("Status"));
        assert!(output.contains("Entered"));
        assert!(output.contains("Last modified"));
        assert!(output.contains("UID"));
        assert!(output.contains("buy milk"));
        assert!(output.contains("Pending"));
        assert!(output.contains(&uid));
        assert!(!output.contains("End"));
        assert!(!output.contains("Deleted"));
    }

    #[test]
    fn render_uses_dash_for_id_when_index_is_none() {
        let now = 1_000_000;
        let t = task(None, "buy milk", now);

        let table = InfoTable::new(&t, now).unwrap();

        let id_row = table
            .rows
            .iter()
            .find(|r| r.name == "ID")
            .expect("ID row should exist");
        assert_eq!(id_row.value, "-");
    }

    #[test]
    fn render_includes_end_row_when_completed_is_set() {
        let now = 1_000_000;
        let mut t = task(Some(Index::new(1).unwrap()), "buy milk", now - 60);
        t.completed = Some(Timestamp::new(now - 10).unwrap());

        let table = InfoTable::new(&t, now).unwrap();
        let output = table.render().to_string();

        assert!(output.contains("End"));
        assert!(!output.contains("Deleted"));
    }

    #[test]
    fn render_includes_deleted_row_when_deleted_is_set() {
        let now = 1_000_000;
        let mut t = task(Some(Index::new(1).unwrap()), "buy milk", now - 60);
        t.deleted = Some(Timestamp::new(now - 10).unwrap());

        let table = InfoTable::new(&t, now).unwrap();
        let output = table.render().to_string();

        assert!(output.contains("Deleted"));
        assert!(!output.contains("End"));
    }

    #[test]
    fn render_includes_both_end_and_deleted_rows_when_set() {
        let now = 1_000_000;
        let mut t = task(Some(Index::new(1).unwrap()), "buy milk", now - 60);
        t.completed = Some(Timestamp::new(now - 20).unwrap());
        t.deleted = Some(Timestamp::new(now - 10).unwrap());

        let table = InfoTable::new(&t, now).unwrap();
        let output = table.render().to_string();

        assert!(output.contains("End"));
        assert!(output.contains("Deleted"));
    }
}
