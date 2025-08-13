use ratatui::{
    Frame,
    layout::{Constraint, Rect},
    style::{Style, Stylize},
    widgets::{Block, Borders, Row, Table, TableState},
};

use crate::state::{AppState, View};

pub struct ItemsTable;

impl ItemsTable {
    pub fn render(frame: &mut Frame, app_state: &AppState, area: Rect, table_state: &TableState) {
        let (header, title) = match app_state.current_view {
            View::Inbox => (
                Row::new(vec!["ID", "S", "Name", "Type", "Added", "Author"])
                    .style(Style::default().bold()),
                "Inbox",
            ),
            View::Archive => (
                Row::new(vec!["ID", "R", "Name", "Done On", "Type", "Note"])
                    .style(Style::default().bold()),
                "Archive",
            ),
        };

        let rows = app_state
            .current_items()
            .iter()
            .enumerate()
            .map(|(i, item)| match app_state.current_view {
                View::Inbox => {
                    let status_char = match item.status {
                        folio_core::Status::Todo => "T",
                        folio_core::Status::Doing => "▶",
                        folio_core::Status::Done => "✓",
                    };

                    let item_type = match item.item_type {
                        folio_core::ItemType::Article => "art.",
                        folio_core::ItemType::Video => "vid.",
                        folio_core::ItemType::Blog => "blog",
                        folio_core::ItemType::Other => "oth.",
                    };

                    let added_date = item.added_at.format("%Y-%m-%d").to_string();

                    Row::new(vec![
                        (i + 1).to_string(),
                        status_char.to_string(),
                        item.name.clone(),
                        item_type.to_string(),
                        added_date,
                        item.author.clone(),
                    ])
                }
                View::Archive => {
                    let reference_char = match item.kind {
                        folio_core::Kind::Normal => "✓",
                        folio_core::Kind::Reference => "☆",
                    };

                    let done_date = item
                        .finished_at
                        .as_ref()
                        .map(|dt| dt.format("%Y-%m-%d").to_string())
                        .unwrap_or_else(|| "".to_string());

                    let item_type = match item.item_type {
                        folio_core::ItemType::Article => "art.",
                        folio_core::ItemType::Video => "vid.",
                        folio_core::ItemType::Blog => "blog",
                        folio_core::ItemType::Other => "oth.",
                    };

                    let note = if item.note.is_empty() {
                        "–".to_string()
                    } else {
                        item.note.clone()
                    };

                    Row::new(vec![
                        (i + 1).to_string(),
                        reference_char.to_string(),
                        item.name.clone(),
                        done_date,
                        item_type.to_string(),
                        note,
                    ])
                }
            });

        let widths = match app_state.current_view {
            View::Inbox => vec![
                Constraint::Length(3),
                Constraint::Length(2),
                Constraint::Percentage(40),
                Constraint::Length(5),
                Constraint::Length(10),
                Constraint::Percentage(30),
            ],
            View::Archive => vec![
                Constraint::Length(3),
                Constraint::Length(2),
                Constraint::Percentage(40),
                Constraint::Length(10),
                Constraint::Length(5),
                Constraint::Percentage(30),
            ],
        };

        let table = Table::new(rows, &widths)
            .header(header)
            .block(Block::default().borders(Borders::ALL).title(title))
            .highlight_style(Style::default().reversed())
            .highlight_symbol(">>");

        frame.render_stateful_widget(table, area, &mut table_state.clone());
    }
}
