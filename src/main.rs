use std::{env::args, io};
mod app;
mod prags;
use app::App;

fn main() -> io::Result<()> {
    let mut args = args();

    let starting_path = args.nth(1);

    let mut terminal = ratatui::init();
    let mut app = App::default();
    app.get_projects(starting_path);
    app.set_mode("projects");

    let _ = app.run(&mut terminal);
    ratatui::restore();

    Ok(())
}
