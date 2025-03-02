use std::sync::Arc;

use crate::app::*;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span, Text},
    widgets::*,
};
use tokio::sync::Mutex;
use tokio::task;

mod layout;
pub mod theme;

/// Draws the current ui. This is used in the app loop to update every frame
pub fn ui<'a>(frame: &mut Frame<'a>, app: &mut App) {
    let chunks = layout::main_layout(frame);

    //GENERATE BACKGOUND (atm i dont want a background cuz of hyprland)
    //generate_background(app, frame);

    generate_main_view(app, frame, chunks[1]);
}

/// Draws the main list of items in the directory. This is where you get the list view
/// where you can move up, down and select different items to open.
fn generate_main_view(app: &mut App, frame: &mut Frame, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .style(Style::default());

    let items = app.get_current_items();

    println!("ui update ...");

    for i in items.lock().unwrap().iter() {
        //println!("{i:?}");
    }
}

/// Generates the background for the current frame
fn generate_background(app: &App, frame: &mut Frame) {
    let background = Block::default().style(Style::default().bg(app.get_theme().get_bg()));
    frame.render_widget(background, frame.area());
}
