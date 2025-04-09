use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
};

use crate::{
    app::*,
    app_properties::{AppMode, AppProperties},
};
use devicons;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style, Stylize},
    text::{Line, Span, Text},
    widgets::*,
};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

pub mod input;
mod layout;
pub mod theme;

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

        if let None = &app_props.cursor {
            return;
        }
        let cursor = app_props.cursor.as_ref().unwrap();

        // This draws every frame, gotta fix that
        if cursor.is_file() {
            let file_content = app_props.manager.read_file(cursor);

            if let Ok(file_content) = file_content {
                let paragraph = Paragraph::new(file_content)
                    .style(Style::default())
                    .fg(app_props.get_theme().get_fg())
                    .alignment(ratatui::layout::Alignment::Left)
                    .wrap(Wrap { trim: true })
                    .block(block);
                frame.render_widget(paragraph, area);
            }
        } else if cursor.is_dir() {
            //This could be added to another function so it can be reused
            let directory = app_props.manager.read_dir(cursor);
            if let Ok(directory) = directory {
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
            /*
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
            */
            frame.render_widget(block, area);
        } else {
            frame.render_widget(block, area);
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
            .fg(app_props.get_theme().get_fg());

        let app_mode = Paragraph::new(app_props.mode.to_string())
            .style(Style::default().fg(app_props.get_theme().get_mt()))
            .block(block);

        // Add directory and/or file properties here

        frame.render_widget(app_mode, area);
    }
}
