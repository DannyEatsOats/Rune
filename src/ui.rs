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

/// Draws the current ui. This is used in the app loop to update every frame
pub fn ui<'a>(frame: &mut Frame<'a>, app: &mut App) {
    let chunks = layout::main_layout(frame);
    let vchunks = layout::header_layout(frame);
    //GENERATE BACKGOUND (atm i dont want a background cuz of hyprland)
    //generate_background(app, frame);

    generate_main_view(app, frame, chunks[1]);
    generate_searchbar(app, frame, vchunks[0]);
}

/// Draws the main list of items in the directory. This is where you get the list view
/// where you can move up, down and select different items to open.
fn generate_main_view(app: &mut App, frame: &mut Frame, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(format!(" {} ", app.get_current_path().to_string_lossy()))
        .style(Style::default().fg(app.get_theme().get_fg()))
        .fg(app.get_theme().get_fg());

    let items = app.get_current_items();
    //Maybe this could be somehow in a different function or stored as state
    let mut list: Vec<Line> = Vec::new();

    let items = items.lock().unwrap();

    items.iter().for_each(|i| {
        let name = i.file_name().unwrap().to_string_lossy();
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

    if !items.is_empty() {
        let list = List::new(list)
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

    drop(items);
}

pub fn generate_searchbar(app: &mut App, frame: &mut Frame, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(String::from("Search"))
        .style(Style::default().fg(app.get_theme().get_fg()))
        .fg(app.get_theme().get_fg());

    let input = Text::from(app.search_input.get_value().clone());
    let input = Paragraph::new(input).block(block);

    frame.render_widget(input, area);
}

/// Generates the background for the current frame
fn generate_background(app: &App, frame: &mut Frame) {
    let background = Block::default().style(Style::default().bg(app.get_theme().get_bg()));
    frame.render_widget(background, frame.area());
}
