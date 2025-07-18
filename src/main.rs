use tokio;

use rune::app::*;
use std::io;

#[tokio::main]
async fn main() -> io::Result<()> {
    let mut terminal = ratatui::init();
    let app_result = App::run(&mut terminal);
    ratatui::restore();
    app_result
}
