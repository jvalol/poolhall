//! Fifteen balls in a triangle, and a cue ball to break them with. See `specs/`.

mod paint;
mod poolhall_game;
mod rules;
mod shot;
mod table;
mod view;

use blitzkit::start;
use poolhall_game::PoolhallGame;

/// Whether this run is only here to be photographed, for `refresh-screenshots`
/// in the project above.
pub fn staged() -> bool {
    std::env::args().any(|arg| arg == "--screenshot")
}

fn main() {
    start("poolhall", Box::new(PoolhallGame::new()));
}
