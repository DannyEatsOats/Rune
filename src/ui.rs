use crate::app::*;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span, Text},
    widgets::*,
    Frame,
};

mod layout;
pub mod theme;

/// Draws the current ui. This is used in the app loop to update every frame
pub fn ui(frame: &mut Frame, app: &mut App) {
    let chunks = layout::main_layout(frame);

    //GENERATE BACKGOUND
    generate_background(app, frame);

    //println!("ui update..."); //Replace this with "terminal.draw(..)"
}

/// Generates the background for the current frame
fn generate_background(app: &App, frame: &mut Frame) {
    let background = Block::default().style(Style::default().bg(app.get_theme().get_bg()));
    frame.render_widget(background, frame.area());
}
