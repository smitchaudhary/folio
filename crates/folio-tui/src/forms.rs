use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style, Stylize},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
};
use std::collections::HashMap;

pub struct FormField {
    pub label: String,
    pub value: String,
    pub is_focused: bool,
    pub field_type: FieldType,
}

pub enum FieldType {
    Text,
    Dropdown {
        options: Vec<String>,
        selected: usize,
    },
}

#[derive(Clone, Copy, PartialEq)]
pub enum FormType {
    Add,
    Edit,
}

pub struct ItemForm {
    pub fields: HashMap<String, FormField>,
    pub focused_field: String,
    pub is_visible: bool,
    pub form_type: FormType,
}

impl ItemForm {
    pub fn new(form_type: FormType) -> Self {
        let mut fields = HashMap::new();

        fields.insert(
            "name".to_string(),
            FormField {
                label: "Name".to_string(),
                value: String::new(),
                is_focused: form_type == FormType::Add,
                field_type: FieldType::Text,
            },
        );

        fields.insert(
            "type".to_string(),
            FormField {
                label: "Type".to_string(),
                value: "article".to_string(),
                is_focused: false,
                field_type: FieldType::Dropdown {
                    options: vec![
                        "article".to_string(),
                        "video".to_string(),
                        "blog".to_string(),
                        "other".to_string(),
                    ],
                    selected: 0,
                },
            },
        );

        fields.insert(
            "author".to_string(),
            FormField {
                label: "Author".to_string(),
                value: String::new(),
                is_focused: false,
                field_type: FieldType::Text,
            },
        );

        fields.insert(
            "link".to_string(),
            FormField {
                label: "Link".to_string(),
                value: String::new(),
                is_focused: false,
                field_type: FieldType::Text,
            },
        );

        fields.insert(
            "note".to_string(),
            FormField {
                label: "Note".to_string(),
                value: String::new(),
                is_focused: false,
                field_type: FieldType::Text,
            },
        );

        Self {
            fields,
            focused_field: "name".to_string(),
            is_visible: false,
            form_type,
        }
    }

    pub fn populate_fields(&mut self, item: &folio_core::Item) {
        if let Some(name_field) = self.fields.get_mut("name") {
            name_field.value = item.name.clone();
        }

        if let Some(type_field) = self.fields.get_mut("type") {
            let type_str = match item.item_type {
                folio_core::ItemType::Article => "article",
                folio_core::ItemType::Video => "video",
                folio_core::ItemType::Blog => "blog",
                folio_core::ItemType::Other => "other",
            };
            type_field.value = type_str.to_string();

            if let FieldType::Dropdown { options, selected } = &mut type_field.field_type {
                *selected = options.iter().position(|o| o == type_str).unwrap_or(0);
            }
        }

        if let Some(author_field) = self.fields.get_mut("author") {
            author_field.value = item.author.clone();
        }

        if let Some(link_field) = self.fields.get_mut("link") {
            link_field.value = item.link.clone();
        }

        if let Some(note_field) = self.fields.get_mut("note") {
            note_field.value = item.note.clone();
        }
    }

    pub fn toggle_visibility(&mut self) {
        self.is_visible = !self.is_visible;
        if self.is_visible {
            match self.form_type {
                FormType::Add => {
                    for field in self.fields.values_mut() {
                        if let FieldType::Dropdown { selected, .. } = &mut field.field_type {
                            *selected = 0;
                            field.value = match field.label.as_str() {
                                "Type" => "article".to_string(),
                                _ => String::new(),
                            };
                        } else {
                            field.value = String::new();
                        }
                        field.is_focused = false;
                    }
                    if let Some(name_field) = self.fields.get_mut("name") {
                        name_field.is_focused = true;
                    }
                    self.focused_field = "name".to_string();
                }
                FormType::Edit => {
                    for field in self.fields.values_mut() {
                        field.is_focused = false;
                    }
                    if let Some(name_field) = self.fields.get_mut("name") {
                        name_field.is_focused = true;
                    }
                    self.focused_field = "name".to_string();
                }
            }
        }
    }

    pub fn focus_next(&mut self) {
        let field_order = vec!["name", "type", "author", "link", "note"];
        let current_index = field_order
            .iter()
            .position(|&f| f == self.focused_field)
            .unwrap_or(0);
        let next_index = (current_index + 1) % field_order.len();

        if let Some(current_field) = self.fields.get_mut(&self.focused_field) {
            current_field.is_focused = false;
        }

        self.focused_field = field_order[next_index].to_string();

        if let Some(next_field) = self.fields.get_mut(&self.focused_field) {
            next_field.is_focused = true;
        }
    }

    pub fn focus_prev(&mut self) {
        let field_order = vec!["name", "type", "author", "link", "note"];
        let current_index = field_order
            .iter()
            .position(|&f| f == self.focused_field)
            .unwrap_or(0);
        let prev_index = (current_index + field_order.len() - 1) % field_order.len();

        if let Some(current_field) = self.fields.get_mut(&self.focused_field) {
            current_field.is_focused = false;
        }

        self.focused_field = field_order[prev_index].to_string();

        if let Some(prev_field) = self.fields.get_mut(&self.focused_field) {
            prev_field.is_focused = true;
        }
    }

    pub fn handle_key(&mut self, key: crossterm::event::KeyEvent) {
        if !self.is_visible {
            return;
        }

        match key.code {
            crossterm::event::KeyCode::Tab => {
                self.focus_next();
            }
            crossterm::event::KeyCode::BackTab => {
                self.focus_prev();
            }
            crossterm::event::KeyCode::Enter => {
                if let Some(field) = self.fields.get(&self.focused_field) {
                    if let FieldType::Dropdown { .. } = field.field_type {
                        if let Some(field) = self.fields.get_mut(&self.focused_field) {
                            if let FieldType::Dropdown { options, selected } = &mut field.field_type
                            {
                                *selected = (*selected + 1) % options.len();
                                field.value = options[*selected].clone();
                            }
                        }
                    }
                }
            }
            crossterm::event::KeyCode::Char(c) => {
                if let Some(field) = self.fields.get_mut(&self.focused_field) {
                    if let FieldType::Text = field.field_type {
                        field.value.push(c);
                    }
                }
            }
            crossterm::event::KeyCode::Backspace => {
                if let Some(field) = self.fields.get_mut(&self.focused_field) {
                    if let FieldType::Text = field.field_type {
                        field.value.pop();
                    }
                }
            }
            _ => {}
        }
    }

    pub fn get_field_value(&self, field_name: &str) -> Option<&String> {
        self.fields.get(field_name).map(|f| &f.value)
    }

    pub fn render(&self, frame: &mut Frame) {
        if !self.is_visible {
            return;
        }

        let area = frame.size();
        let popup_area = centered_rect(60, 90, area);

        frame.render_widget(Clear, popup_area);

        let title = match self.form_type {
            FormType::Add => "Add Item",
            FormType::Edit => "Edit Item",
        };

        let block = Block::default()
            .title(title)
            .borders(Borders::ALL)
            .style(Style::default().bg(Color::Black).fg(Color::White));

        frame.render_widget(block, popup_area);

        let inner_area = Layout::default()
            .direction(Direction::Vertical)
            .margin(2)
            .constraints([
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Min(1),
            ])
            .split(popup_area);

        let field_order = vec!["name", "type", "author", "link", "note"];

        for (i, field_name) in field_order.iter().enumerate() {
            if let Some(field) = self.fields.get(*field_name) {
                let field_area = inner_area[i];

                let label = Paragraph::new(format!("{}:", field.label))
                    .style(Style::default().fg(Color::Cyan).bold());

                let value_style = if field.is_focused {
                    Style::default().fg(Color::Black).bg(Color::Cyan).bold()
                } else {
                    Style::default().fg(Color::White)
                };

                let value_text = match &field.field_type {
                    FieldType::Text => field.value.clone(),
                    FieldType::Dropdown { .. } => {
                        format!(
                            "{} {}",
                            field.value,
                            if field.is_focused { "▼" } else { "" }
                        )
                    }
                };

                let value = Paragraph::new(value_text).style(value_style);

                let field_layout = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([Constraint::Length(10), Constraint::Min(1)])
                    .split(field_area);

                frame.render_widget(label, field_layout[0]);
                frame.render_widget(value, field_layout[1]);
            }
        }

        let instructions_text = match self.form_type {
            FormType::Add => {
                "Tab: Next field, Shift+Tab: Previous field, Enter: Toggle dropdown, Esc: Cancel"
            }
            FormType::Edit => {
                "Tab: Next field, Shift+Tab: Previous field, Enter: Toggle dropdown, Esc: Cancel, Ctrl+S: Save"
            }
        };

        let instructions = Paragraph::new(instructions_text)
            .style(Style::default().fg(Color::Gray).italic())
            .wrap(Wrap { trim: true });

        frame.render_widget(instructions, inner_area[6]);
    }
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
