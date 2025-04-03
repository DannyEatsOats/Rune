use crate::app::*;
use devicons;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style, Stylize},
    text::{Line, Span, Text},
    widgets::*,
};

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
    chunks: Option<std::rc::Rc<[ratatui::layout::Rect]>>,
    vchunks: Option<std::rc::Rc<[ratatui::layout::Rect]>>,
    list: Option<Vec<Line<'a>>>,
}

impl<'a> UI<'a> {
    pub fn goon(&mut self) {}

    pub fn new(app: &App) -> Self {
        let mut ui = Self {
            chunks: None,
            vchunks: None,
            list: None,
        };
        ui.set_main_items(app);

        ui
    }

    /// Draws the current ui. This is used in the app loop to update every frame
    pub fn draw<'b>(&mut self, frame: &mut Frame<'b>, app: &mut App) {
        if self.chunks.is_some() {
            //GENERATE BACKGOUND (atm i dont want a background cuz of hyprland)
            //generate_background(app, frame);

            self.generate_main_view(app, frame, self.chunks.clone().unwrap()[1]);
            self.generate_searchbar(app, frame, self.vchunks.clone().unwrap()[0]);
        } else {
            self.chunks = Some(layout::main_layout(frame));
            self.vchunks = Some(layout::header_layout(frame));
        }
    }

    pub fn set_main_items(&mut self, app: &App) {
        let items = app.get_current_items();
        //Maybe this could be somehow in a different function or stored as state
        let mut list: Vec<Line> = Vec::new();

        let items = items.lock().unwrap().clone();
        items.iter().for_each(|i| {
            let name = i.file_name().unwrap().to_string_lossy().into_owned();
            let icon = devicons::icon_for_file(i, &Some(devicons::Theme::Dark));

            let rgb = hex::decode(icon.color.trim_matches('#'));
            let color = if i.is_dir() || !rgb.is_ok() {
                app.get_theme().get_fg()
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

    /// Draws the main list of items in the directory. This is where you get the list view
    /// where you can move up, down and select different items to open.
    fn generate_main_view(&mut self, app: &mut App, frame: &mut Frame, area: Rect) {
        let block = Block::default()
            .borders(Borders::ALL)
            .title(format!(" {} ", app.get_current_path().to_string_lossy()))
            .style(Style::default().fg(app.get_theme().get_fg()))
            .fg(app.get_theme().get_fg());

        if !self.list.as_ref().unwrap().is_empty() {
            //Somehow here I'll need to recieve a signal from the app to update UI
            self.set_main_items(app);
            let list = List::new(self.list.clone().unwrap())
                .style(Style::default().fg(app.get_theme().get_fg()))
                .highlight_style(Style::default().fg(app.get_theme().get_ht()))
                .scroll_padding(5)
                .block(block.clone())
                .highlight_symbol(">> ");

            //frame.render_widget(list.clone(), area);
            frame.render_stateful_widget(list.clone(), area, app.get_ml_state());
        } else {
            frame.render_widget(block, area);
        }
    }

    pub fn generate_searchbar(&mut self, app: &mut App, frame: &mut Frame, area: Rect) {
        let block = Block::default()
            .borders(Borders::ALL)
            .title(String::from(" Search "))
            .style(Style::default().fg(app.get_theme().get_fg()))
            .fg(app.get_theme().get_fg());

        let val = if app.get_mode().eq(&AppMode::Search) {
            app.search_input.get_value().clone() + "|"
        } else {
            app.search_input.get_value().clone()
        };
        let input = Text::from(val);
        let input = Paragraph::new(input).block(block);

        frame.render_widget(input, area);
    }

    /// Generates the background for the current frame
    fn generate_background(app: &App, frame: &mut Frame) {
        let background = Block::default().style(Style::default().bg(app.get_theme().get_bg()));
        frame.render_widget(background, frame.area());
    }
}
