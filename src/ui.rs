use std::{
    ffi::OsStr,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
    time::SystemTime,
    usize,
};

use crate::{
    app::*,
    app_properties::{AppMode, AppProperties},
};
use chrono::{DateTime, Local};
use crossterm::style::style;
use devicons;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style, Stylize},
    text::{Line, Span, Text},
    widgets::*,
};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use std::fmt::Debug;
use std::ops::{Add, Div, Mul, Sub};

pub mod input;
mod layout;
pub mod theme;

pub trait ByteReadable:
    Copy
    + Debug
    + Add<Output = Self>
    + Sub<Output = Self>
    + Mul<Output = Self>
    + Div<Output = Self>
    + PartialOrd
    + PartialEq
{
    fn to_f64(self) -> f64;
    fn from_f64(f: f64) -> Self;

    fn byte_display(&self) -> String {
        const BYTE_TYPES: [&str; 5] = ["B", "KB", "MB", "GB", "TB"];
        let mut val = *self;
        let mut index = 0;
        let limit = Self::from_f64(1024.0);

        while val > limit && index < BYTE_TYPES.len() - 1 {
            val = val / limit;
            index += 1;
        }

        format!("{:.2}{}", val.to_f64(), BYTE_TYPES[index])
    }
}

macro_rules! impl_byte_readable {
    ($($t:ty),*) => {
        $(
            impl ByteReadable for $t {
                fn to_f64(self) -> f64 {
                    self as f64
                }

                fn from_f64(f: f64) -> Self {
                    f as $t
                }
            }
        )*
    };
}

impl_byte_readable!(u8, u16, u32, u64, usize, i32, i64, f32, f64);

//A major problem here, is that since the ui is updated every frame, there are a bunch of
//operations that will run on every frame that should be saved or cached. On big datasets, this
//causes UI lags and performance issues.
//
//Ex: generate_main_view() is an expensive operation, generating the List
//Maybe a singleton pattern could be something to solve some issues here (storing created objects,
//data)
pub struct UI<'a> {
    list: Option<Vec<Line<'a>>>,
}

impl<'a> UI<'a> {
    pub fn goon(&mut self) {}

    pub fn new(app_props: &AppProperties) -> Self {
        let mut ui = Self { list: None };
        ui.set_main_items(app_props);

        ui
    }

    /// Draws the current ui. This is used in the app loop to update every frame
    pub fn draw<'b>(&mut self, frame: &mut Frame<'b>, app_props: &mut AppProperties) {
        let chunks = layout::main_layout(frame);
        let header = layout::header_layout(frame);
        let footer = layout::footer_layout(frame);

        //GENERATE BACKGOUND (atm i dont want a background cuz of hyprland)
        //generate_background(app, frame);

        self.generate_statusbar(app_props, frame, footer[0]);
        self.generate_main_view(app_props, frame, chunks[1]);
        self.generate_preview(app_props, frame, chunks[2]);
        self.generate_searchbar(app_props, frame, header[0]);
    }

    /// Sets the items for the main screen, called by app when changing directories
    pub fn set_main_items(&mut self, app_props: &AppProperties) {
        let items = app_props.get_current_items();
        //Maybe this could be somehow in a different function or stored as state
        let mut list: Vec<Line> = Vec::new();

        let items = items.lock().unwrap().clone();
        items.iter().for_each(|i| {
            let name = i.file_name().unwrap().to_string_lossy().into_owned();
            let icon = devicons::icon_for_file(i, &Some(devicons::Theme::Dark));

            let rgb = hex::decode(icon.color.trim_matches('#'));
            let color = if i.is_dir() || !rgb.is_ok() {
                app_props.get_theme().get_fg()
            } else {
                let rgb = rgb.unwrap();
                Color::Rgb(rgb[0], rgb[1], rgb[2])
            };

            let line = Line::from(vec![
                Span::styled(format!("{} ", icon.icon), Style::default().fg(color)),
                Span::from(name),
            ]);
            list.push(Line::from(line));
        });
        self.list = Some(list);
        drop(items);
    }

    /// Pushes a new item to the main item list
    pub fn addto_main_items(&mut self, path: &PathBuf, app_props: &AppProperties) {
        let name = path.file_name().unwrap().to_string_lossy().into_owned();
        let icon = devicons::icon_for_file(path, &Some(devicons::Theme::Dark));

        let rgb = hex::decode(icon.color.trim_matches('#'));
        let color = if path.is_dir() || !rgb.is_ok() {
            app_props.get_theme().get_fg()
        } else {
            let rgb = rgb.unwrap();
            Color::Rgb(rgb[0], rgb[1], rgb[2])
        };

        let line = Line::from(vec![
            Span::styled(format!("{} ", icon.icon), Style::default().fg(color)),
            Span::from(name),
        ]);
        self.list.as_mut().unwrap().push(line);
    }

    /// Draws the main list of items in the directory. This is where you get the list view
    /// where you can move up, down and select different items to open.
    fn generate_main_view(&mut self, app_props: &mut AppProperties, frame: &mut Frame, area: Rect) {
        let block = Block::default()
            .borders(Borders::ALL)
            .title(format!(
                " {} ",
                app_props.get_current_path().to_string_lossy()
            ))
            .style(Style::default().fg(app_props.get_theme().get_fg()))
            .fg(app_props.get_theme().get_fg());

        if app_props.manager.is_searching() {
            self.set_main_items(app_props);
        }

        if !self.list.as_ref().unwrap().is_empty() {
            let list = List::new(self.list.clone().unwrap())
                .style(Style::default().fg(app_props.get_theme().get_fg()))
                .highlight_style(Style::default().fg(app_props.get_theme().get_ht()))
                .scroll_padding(5)
                .block(block.clone())
                .highlight_symbol(">> ");

            frame.render_stateful_widget(list.clone(), area, app_props.get_ml_state());
        } else {
            let empty_text = Paragraph::new("Directory Empty :(")
                .style(Style::default().fg(app_props.get_theme().get_pr()))
                .centered()
                .block(block);
            frame.render_widget(empty_text, area);
        }
    }

    pub fn generate_searchbar(
        &mut self,
        app_props: &mut AppProperties,
        frame: &mut Frame,
        area: Rect,
    ) {
        let block = Block::default()
            .borders(Borders::ALL)
            .title(String::from(" Search "))
            .style(Style::default().fg(app_props.get_theme().get_fg()))
            .fg(app_props.get_theme().get_fg());

        let val = if app_props.get_mode().eq(&AppMode::Search) {
            app_props.search_input.get_value().clone() + "|"
        } else {
            app_props.search_input.get_value().clone()
        };
        let input = Text::from(val);
        let input = Paragraph::new(input).block(block);

        frame.render_widget(input, area);
    }

    fn generate_preview(&mut self, app_props: &mut AppProperties, frame: &mut Frame, area: Rect) {
        let idx = app_props.get_ml_state().selected().unwrap_or(0);
        let block = Block::default()
            .borders(Borders::ALL)
            .title(format!(" Preview ",))
            .title_alignment(ratatui::layout::Alignment::Center)
            .style(Style::default().fg(app_props.get_theme().get_fg()))
            .fg(app_props.get_theme().get_fg());

        if let (None, _) = &app_props.cursor {
            let empty_text = Paragraph::new("Directory Empty :(")
                .style(Style::default().fg(app_props.get_theme().get_pr()))
                .centered()
                .block(block);
            frame.render_widget(empty_text, area);
            return;
        }
        let (path, metadata) = &app_props.cursor;
        let path = path.as_ref().unwrap();

        // This draws every frame, gotta fix that
        if path.is_file() {
            let file_content = app_props.manager.read_file(&path);

            if let Ok(file_content) = file_content {
                let paragraph = Paragraph::new(file_content)
                    .style(Style::default())
                    .fg(app_props.get_theme().get_fg())
                    .alignment(ratatui::layout::Alignment::Left)
                    .wrap(Wrap { trim: true })
                    .block(block);
                frame.render_widget(paragraph, area);
            }
        } else if path.is_dir() {
            //This could be added to another function so it can be reused
            let directory = app_props.manager.read_dir(&path);
            if let Ok(directory) = directory {
                if directory.is_empty() {
                    // Refactor this later, cuz 'empty text' gets generated too manny times.
                    let empty_text = Paragraph::new("Directory Empty :(")
                        .style(Style::default().fg(app_props.get_theme().get_pr()))
                        .centered()
                        .block(block);
                    frame.render_widget(empty_text, area);
                    return;
                }
                let mut list: Vec<Line> = Vec::new();
                directory.iter().for_each(|i| {
                    let name = i.file_name().unwrap().to_string_lossy().into_owned();
                    let icon = devicons::icon_for_file(i, &Some(devicons::Theme::Dark));

                    let rgb = hex::decode(icon.color.trim_matches('#'));
                    let color = if i.is_dir() || !rgb.is_ok() {
                        app_props.get_theme().get_fg()
                    } else {
                        let rgb = rgb.unwrap();
                        Color::Rgb(rgb[0], rgb[1], rgb[2])
                    };

                    let line = Line::from(vec![
                        Span::styled(format!("{} ", icon.icon), Style::default().fg(color)),
                        Span::from(name),
                    ]);
                    list.push(Line::from(line));
                });
                let list = List::new(list)
                    .style(Style::default().fg(app_props.get_theme().get_fg()))
                    .highlight_style(Style::default().fg(app_props.get_theme().get_ht()))
                    .scroll_padding(5)
                    .block(block.clone())
                    .highlight_symbol(">> ");
                frame.render_widget(list, area);
            }
            frame.render_widget(block, area);
        } else {
            let empty_text = Paragraph::new("Directory Empty :(")
                .style(Style::default().fg(app_props.get_theme().get_pr()))
                .centered()
                .block(block);
            frame.render_widget(empty_text, area);
        }
    }

    /// Generates the background for the current frame
    fn generate_background(app_props: &AppProperties, frame: &mut Frame) {
        let background =
            Block::default().style(Style::default().bg(app_props.get_theme().get_bg()));
        frame.render_widget(background, frame.area());
    }

    fn generate_statusbar(&self, app_props: &AppProperties, frame: &mut Frame, area: Rect) {
        let block = Block::default()
            .borders(Borders::ALL)
            .style(Style::default().fg(app_props.get_theme().get_fg()))
            .fg(app_props.get_theme().get_fg())
            .padding(Padding::horizontal(1));

        let cursor = &app_props.cursor;
        let mode = app_props.mode.to_string();
        let mut text = String::new();
        if let Some(md) = &cursor.1 {
            let size = md.len().byte_display();
            let datetime: DateTime<Local> = md.created().unwrap_or(SystemTime::UNIX_EPOCH).into();
            let datetime = datetime.format("%Y-%m-%d").to_string();

            text.push_str(&format!("{size} {datetime}"));
        }

        let perms_span = Span::styled(&text, Style::default().fg(app_props.get_theme().get_st()));
        let mode = &app_props.mode;
        let mode_span = match &mode {
            AppMode::Normal => Span::styled(
                mode.to_string(),
                Style::default().fg(app_props.get_theme().get_mt()),
            ),
            AppMode::Edit => Span::styled(
                mode.to_string(),
                Style::default().fg(app_props.get_theme().get_bg()),
            ),
            AppMode::Search => Span::styled(
                mode.to_string(),
                Style::default().fg(app_props.get_theme().get_ht()),
            ),
            AppMode::Compare => Span::styled(
                mode.to_string(),
                Style::default().fg(app_props.get_theme().get_ht()),
            ),
        };
        let space = (area.width as usize - 3).saturating_sub(mode.to_string().len() + text.len());
        let space = Span::styled(" ".repeat(space), Style::default());

        let line = Line::from(vec![mode_span, space, perms_span]).style(Style::default());

        let status_line = Paragraph::new(line).style(Style::default()).block(block);

        frame.render_widget(status_line, area);
    }
}
