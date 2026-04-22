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
                value: format_absolute(completed)?,
            });
        }
        if let Some(deleted) = &task.deleted {
            rows.push(InfoRow {
                name: "Deleted".to_string(),
                value: format_absolute(deleted)?,
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
