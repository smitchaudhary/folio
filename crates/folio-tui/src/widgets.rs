use ratatui::{
    Frame,
    layout::{Constraint, Rect},
    style::{Color, Style, Stylize},
    widgets::{Block, Borders, Row, Table, TableState},
};

use crate::state::{AppState, View};

pub struct ItemsTable;

impl ItemsTable {
    pub fn render(frame: &mut Frame, app_state: &AppState, area: Rect, table_state: &TableState) {
        let (header, title, border_style) = match app_state.current_view {
            View::Inbox => {
                let title = if let Some(filter) = &app_state.filter {
                    format!(" Inbox (filtered: {}) ", filter)
                } else {
                    " Inbox ".to_string()
                };
                (
                    Row::new(vec!["ID", "S", "Name", "Type", "Added", "Author", "Link"])
                        .style(Style::default().fg(Color::White).bold()),
                    title,
                    Style::default().fg(Color::Green).bold(),
                )
            }
            View::Archive => {
                let title = if let Some(filter) = &app_state.filter {
                    format!(" Archive (filtered: {}) ", filter)
                } else {
                    " Archive ".to_string()
                };
                (
                    Row::new(vec!["ID", "R", "Name", "Author", "Done On", "Type", "Note", "Link"])
                        .style(Style::default().fg(Color::White).bold()),
                    title,
                    Style::default().fg(Color::Blue).bold(),
                )
            }
        };

        let visible_items = app_state.visible_items();

        let rows = visible_items
            .iter()
            .enumerate()
            .map(
                |(display_index, (_id, item))| match app_state.current_view {
                    View::Inbox => {
                        let status_char = item.status.display_emoji();

                        let status_style = match item.status {
                            folio_core::Status::Todo => Style::default().fg(Color::Gray),
                            folio_core::Status::Doing => Style::default().fg(Color::Yellow).bold(),
                            folio_core::Status::Done => Style::default().fg(Color::Green).bold(),
                        };

                        let item_type = item.item_type.abbreviation();

                        let added_date = item.added_at.format("%Y-%m-%d").to_string();

                        let link_display = if item.link.len() > 25 {
                            format!("{}...", &item.link[..22])
                        } else {
                            item.link.clone()
                        };

                        Row::new(vec![
                            (display_index + 1).to_string(),
                            status_char.to_string(),
                            item.name.clone(),
                            item_type.to_string(),
                            added_date,
                            item.author.clone(),
                            link_display,
                        ])
                        .style(status_style)
                    }
                    View::Archive => {
                        let reference_char = item.kind.display_emoji();

                        let reference_style = match item.kind {
                            folio_core::Kind::Normal => Style::default().fg(Color::White),
                            folio_core::Kind::Reference => Style::default().fg(Color::Cyan).bold(),
                        };

                        let done_date = item
                            .finished_at
                            .as_ref()
                            .map(|dt| dt.format("%Y-%m-%d").to_string())
                            .unwrap_or_default();

                        let item_type = item.item_type.abbreviation();

                        let note = if item.note.is_empty() {
                            "–".to_string()
                        } else {
                            item.note.clone()
                        };

                        let link_display = if item.link.len() > 20 {
                            format!("{}...", &item.link[..17])
                        } else {
                            item.link.clone()
                        };

                        Row::new(vec![
                            (display_index + 1).to_string(),
                            reference_char.to_string(),
                            item.name.clone(),
                            item.author.clone(),
                            done_date,
                            item_type.to_string(),
                            note,
                            link_display,
                        ])
                        .style(reference_style)
                    }
                },
            );

        let widths = match app_state.current_view {
            View::Inbox => vec![
                Constraint::Length(3),
                Constraint::Length(2),
                Constraint::Percentage(30),
                Constraint::Length(5),
                Constraint::Length(10),
                Constraint::Percentage(20),
                Constraint::Percentage(25),
            ],
            View::Archive => vec![
                Constraint::Length(3),
                Constraint::Length(2),
                Constraint::Percentage(25),
                Constraint::Percentage(15),
                Constraint::Length(10),
                Constraint::Length(5),
                Constraint::Percentage(10),
                Constraint::Percentage(20),
            ],
        };

        let table = Table::new(rows, &widths)
            .header(header)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(title)
                    .border_style(border_style),
            )
            .row_highlight_style(Style::default().fg(Color::Black).bg(Color::Cyan).bold())
            .highlight_symbol("▶");

        frame.render_stateful_widget(table, area, &mut table_state.clone());
    }
}
