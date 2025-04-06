use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap},
};

fn main_vertical_layot(frame: &mut Frame) -> std::rc::Rc<[ratatui::layout::Rect]> {
    Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(10),
            Constraint::Percentage(80),
            Constraint::Percentage(10),
        ])
        .split(frame.area())
}

pub fn main_layout(frame: &mut Frame) -> std::rc::Rc<[ratatui::layout::Rect]> {
    let vertical_chunks = main_vertical_layot(frame);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(25), //20
            Constraint::Percentage(45), //55
            Constraint::Percentage(30), //25
        ])
        .split(vertical_chunks[1])
}

pub fn header_layout(frame: &mut Frame) -> std::rc::Rc<[ratatui::layout::Rect]> {
    let vertical_chunks = main_vertical_layot(frame);
    Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(100)])
        .split(vertical_chunks[0])
}

pub fn footer_layout(frame: &mut Frame) -> std::rc::Rc<[ratatui::layout::Rect]> {
    let vertical_chunks = main_vertical_layot(frame);
    Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(100)])
        .split(vertical_chunks[2])
}

pub fn view_layout(frame: &mut Frame) -> std::rc::Rc<[ratatui::layout::Rect]> {
    let vertical_chunks = main_vertical_layot(frame);
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(30), //20
            Constraint::Percentage(40), //55
            Constraint::Percentage(30), //25
        ])
        .split(vertical_chunks[1])
}
