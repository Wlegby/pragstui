#![allow(unused)]
#![allow(clippy::single_match)]
#![allow(dead_code)]

use std::{
    env, io,
    ops::Index,
    path::{Path, PathBuf},
    process::Command,
    vec,
};

use ratatui::{
    DefaultTerminal, Frame,
    buffer::Buffer,
    crossterm::{
        event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
        terminal::EndSynchronizedUpdate,
    },
    layout::{Constraint, Direction, Layout, Margin, Position, Rect},
    style::{Color, Style, Stylize},
    symbols::{
        border::{self, QUADRANT_BLOCK},
        scrollbar::Set,
    },
    text::{Line, Text},
    widgets::{
        Block, BorderType, List, ListDirection, ListItem, ListState, Paragraph, Scrollbar,
        ScrollbarOrientation, ScrollbarState, Widget, Wrap,
    },
};

use crate::prags::Project;

#[derive(Debug, Default)]
pub struct App {
    controls: String,

    projects: Vec<Project>,
    tags: Vec<(String, bool)>,

    proj_list_state: ListState,
    tags_list_state: ListState,

    input: InputField,
    selected: Select,
    exit: bool,
}

#[derive(Debug, Default, Clone, PartialEq)]
enum Select {
    #[default]
    Projects,
    Tags,
}

#[derive(Debug, Default, Clone)]
struct InputField {
    text: String,
    char_idx: usize,
    mode: InputMode,
}

#[derive(Debug, Default, Copy, Clone, PartialEq)]
enum InputMode {
    #[default]
    Normal,
    Editing,
}

impl App {
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }

        Ok(())
    }

    fn draw(&mut self, frame: &mut Frame) {
        self.render_window(frame);
    }

    fn handle_events(&mut self) -> io::Result<()> {
        match event::read()? {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.handle_key_event(key_event)
            }
            _ => {}
        }
        Ok(())
    }

    fn open_zed(&mut self) {
        if let Some(s) = self.proj_list_state.selected() {
            let mut tags_list = Vec::new();
            let mut selected_tags = Vec::new();
            self.tags.iter().for_each(|(tag, selected)| {
                tags_list.push(format!(" [{}] {}", if *selected { "X" } else { " " }, tag));
                if *selected {
                    selected_tags.push(tag);
                }
            });

            let mut matches = Vec::new();
            if selected_tags.is_empty() {
                self.projects.iter().for_each(|p| {
                    if p.name.contains(&self.input.text) {
                        matches.push(p)
                    }
                });
            } else {
                self.projects.iter().for_each(|p| {
                    if p.name.contains(&self.input.text) {
                        let mut contains_tag = false;
                        for t in p.tags.iter() {
                            if selected_tags.contains(&t) {
                                contains_tag = true;
                                break;
                            }
                        }
                        if contains_tag {
                            matches.push(p)
                        }
                    }
                });
            }
            matches.sort_by(|a, b| a.name.cmp(&b.name));

            let toml_path = matches[s].path.clone();
            let path = PathBuf::from(toml_path);

            println!("{}", path.parent().unwrap().to_str().unwrap());

            Command::new("zeditor").arg(path.parent().unwrap()).spawn();

            self.exit();
        }
    }

    pub fn set_mode(&mut self, mode: &str) {
        match mode {
            "editing" => self.controls = String::from("Escape/Enter: exit typing"),
            "projects" => {
                self.controls = String::from(
                    "j: down -- k: up -- Space/Enter: select & open zed -- q: exit -- c: clear -- l: goto tags",
                )
            }
            "tags" => {
                self.controls = String::from(
                    "j: down -- k: up -- Space: Toggle tag -- q: exit -- c: clear -- l: goto projects",
                )
            }
            _ => {}
        }
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) {
        match self.input.mode {
            InputMode::Editing => {
                match key_event.code {
                    KeyCode::Esc => {
                        self.input.mode = InputMode::Normal;
                        self.set_mode(match self.selected {
                            Select::Projects => "projects",
                            Select::Tags => "tags",
                        });
                    }
                    KeyCode::Enter => {
                        self.input.mode = InputMode::Normal;
                        self.set_mode(match self.selected {
                            Select::Projects => "projects",
                            Select::Tags => "tags",
                        });
                    }
                    KeyCode::Char(c) => self.input.enter_char(c),
                    KeyCode::Backspace => self.input.delete_char(),
                    KeyCode::Right => self.input.move_cursor_right(),
                    KeyCode::Left => self.input.move_cursor_left(),
                    _ => {}
                };
            }
            InputMode::Normal => {
                match self.selected {
                    Select::Projects => match key_event.code {
                        KeyCode::Char('j') => self.proj_list_state.select_next(),
                        KeyCode::Char('k') => self.proj_list_state.select_previous(),
                        KeyCode::Char(' ') => self.open_zed(),
                        KeyCode::Enter => self.open_zed(),
                        _ => {}
                    },
                    Select::Tags => match key_event.code {
                        KeyCode::Char('j') => self.tags_list_state.select_next(),
                        KeyCode::Char('k') => self.tags_list_state.select_previous(),
                        KeyCode::Char(' ') => {
                            let selected = if let Some(s) = self.tags_list_state.selected() {
                                s
                            } else {
                                self.tags_list_state.select(Some(0));
                                0
                            };
                            self.tags[selected].1 = !self.tags[selected].1
                        }
                        _ => {}
                    },
                }

                match key_event.code {
                    KeyCode::Char('i') => {
                        self.input.mode = InputMode::Editing;
                        self.set_mode("editing");
                    }
                    KeyCode::Char('q') => self.exit(),
                    KeyCode::Char('c') => {
                        for t in &mut self.tags {
                            t.1 = false;
                        }
                        self.input.clear();
                    }
                    KeyCode::Char('h') => {
                        self.selected = Select::Projects;
                        self.set_mode("projects");
                    }

                    KeyCode::Char('l') => {
                        self.selected = Select::Tags;
                        self.set_mode("tags");
                    }

                    _ => {}
                }
            }
        }
    }

    fn exit(&mut self) {
        self.exit = true;
    }

    pub fn get_projects(&mut self, path: Option<String>) {
        self.projects = Project::default().get_all(path);

        let mut tags = Vec::new();
        self.projects
            .iter()
            .for_each(|p| p.tags.iter().for_each(|t| tags.push((t.clone(), false))));

        tags.sort();
        tags.dedup();

        self.tags = tags;
    }

    fn render_window(&mut self, frame: &mut Frame) {
        let center = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![
                Constraint::Percentage(10),
                Constraint::Min(0),
                Constraint::Percentage(10),
            ])
            .split(frame.area());

        let main = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![Constraint::Min(0), Constraint::Max(2)])
            .split(center[1]);

        let layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![Constraint::Percentage(70), Constraint::Percentage(30)])
            .split(main[0]);

        let left = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![Constraint::Length(3), Constraint::Min(0)])
            .split(layout[0]);

        frame.render_widget(
            Paragraph::new(Line::from(self.controls.clone()).left_aligned()),
            main[1],
        );

        frame.render_widget(
            Paragraph::new(vec![
                Line::from(format!(" {}", self.input.text.clone())).left_aligned(),
            ])
            .wrap(Wrap { trim: true })
            .block(
                Block::bordered()
                    .border_type(BorderType::Rounded)
                    .title(Line::from(" Input: "))
                    .title_bottom(Line::from(format!(" {:?} ", self.input.mode)).right_aligned()),
            )
            .style(match self.input.mode {
                InputMode::Normal => Style::default(),
                InputMode::Editing => Style::default().fg(Color::Yellow),
            }),
            left[0],
        );

        #[allow(clippy::cast_possible_truncation)]
        if self.input.mode == InputMode::Editing {
            frame.set_cursor_position(Position::new(
                left[0].positions().nth(0).unwrap().x + self.input.char_idx as u16 + 1,
                left[0].positions().nth(0).unwrap().y + 1,
            ));
        }

        let mut tags_list = Vec::new();
        let mut selected_tags = Vec::new();
        self.tags.iter().for_each(|(tag, selected)| {
            tags_list.push(format!(" [{}] {}", if *selected { "X" } else { " " }, tag));
            if *selected {
                selected_tags.push(tag);
            }
        });

        let mut matches = Vec::new();
        if selected_tags.is_empty() {
            self.projects.iter().for_each(|p| {
                if p.name.contains(&self.input.text) {
                    matches.push(format!(
                        "{}  --  ({})",
                        p.name.clone(),
                        PathBuf::from(p.path.clone())
                            .parent()
                            .unwrap()
                            .file_name()
                            .unwrap()
                            .to_str()
                            .unwrap(),
                    ))
                }
            });
        } else {
            self.projects.iter().for_each(|p| {
                if p.name.contains(&self.input.text) {
                    let mut contains_tag = false;
                    for t in p.tags.iter() {
                        if selected_tags.contains(&t) {
                            contains_tag = true;
                            break;
                        }
                    }
                    if contains_tag {
                        matches.push(format!(
                            "{}  --  ({})",
                            p.name.clone(),
                            PathBuf::from(p.path.clone())
                                .parent()
                                .unwrap()
                                .file_name()
                                .unwrap()
                                .to_str()
                                .unwrap(),
                        ))
                    }
                }
            });
        }
        matches.sort();

        frame.render_stateful_widget(
            List::new(matches)
                .block(
                    Block::bordered()
                        .border_type(BorderType::Double)
                        .style(
                            if self.selected == Select::Projects
                                && self.input.mode == InputMode::Normal
                            {
                                Style::default().fg(Color::Yellow)
                            } else {
                                Style::default()
                            },
                        )
                        .title(Line::from(" Projects ").centered()),
                )
                .style(Style::default().fg(Color::White))
                .highlight_style(Style::new().bold().fg(Color::LightBlue))
                .highlight_symbol("•")
                .repeat_highlight_symbol(true)
                .direction(ListDirection::TopToBottom)
                .scroll_padding(5),
            left[1],
            &mut self.proj_list_state,
        );

        frame.render_stateful_widget(
            Scrollbar::new(ScrollbarOrientation::VerticalRight),
            left[1].inner(Margin {
                vertical: 1,
                horizontal: 0,
            }),
            &mut ScrollbarState::new(self.projects.len())
                .position(self.proj_list_state.selected().unwrap_or_default()),
        );

        frame.render_stateful_widget(
            List::new(tags_list)
                .block(
                    Block::bordered()
                        .border_type(BorderType::Double)
                        .title(Line::from(" Tags ").centered())
                        .style(
                            if self.selected == Select::Tags && self.input.mode == InputMode::Normal
                            {
                                Style::default().fg(Color::Yellow)
                            } else {
                                Style::default()
                            },
                        ),
                )
                .style(Style::default().fg(Color::White))
                .highlight_style(Style::new().bold().fg(Color::LightBlue))
                .highlight_symbol("•")
                .repeat_highlight_symbol(true)
                .direction(ListDirection::TopToBottom)
                .scroll_padding(5),
            layout[1],
            &mut self.tags_list_state,
        );

        frame.render_stateful_widget(
            Scrollbar::new(ScrollbarOrientation::VerticalRight),
            layout[1].inner(Margin {
                vertical: 1,
                horizontal: 0,
            }),
            &mut ScrollbarState::new(self.tags.len())
                .position(self.tags_list_state.selected().unwrap_or_default()),
        );
    }
}

impl InputField {
    fn move_cursor_left(&mut self) {
        let cursor_moved_left = self.char_idx.saturating_sub(1);
        self.char_idx = self.clamp_cursor(cursor_moved_left);
    }

    fn move_cursor_right(&mut self) {
        let cursor_moved_right = self.char_idx.saturating_add(1);
        self.char_idx = self.clamp_cursor(cursor_moved_right);
    }

    fn enter_char(&mut self, new_char: char) {
        let index = self.byte_index();
        self.text.insert(index, new_char);
        self.move_cursor_right();
    }

    fn byte_index(&self) -> usize {
        self.text
            .char_indices()
            .map(|(i, _)| i)
            .nth(self.char_idx)
            .unwrap_or(self.text.len())
    }

    fn delete_char(&mut self) {
        let is_not_cursor_leftmost = self.char_idx != 0;
        if is_not_cursor_leftmost {
            let current_index = self.char_idx;
            let from_left_to_current_index = current_index - 1;

            let before_char_to_delete = self.text.chars().take(from_left_to_current_index);
            let after_char_to_delete = self.text.chars().skip(current_index);

            self.text = before_char_to_delete.chain(after_char_to_delete).collect();
            self.move_cursor_left();
        }
    }

    fn clear(&mut self) {
        self.text.clear();
        self.char_idx = 0;
    }

    fn clamp_cursor(&self, new_cursor_pos: usize) -> usize {
        new_cursor_pos.clamp(0, self.text.chars().count())
    }
}
