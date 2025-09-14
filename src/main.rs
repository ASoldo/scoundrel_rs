mod app;
mod cards;
mod deck;
mod game;
mod ui;

use anyhow::Result;

fn main() -> Result<()> {
    app::run()
}
