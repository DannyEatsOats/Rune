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

pub fn ui(frame: &mut Frame, app: &mut App) {
    let chunks = layout::main_layout(frame);
    println!("ui update..."); //Replace this with "terminal.draw(..)"
}
