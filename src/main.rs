mod app;
use app::App;

use std::io;

fn main() -> io::Result<()> {
    ratatui::run(|terminal| App::default().run(terminal))
}
