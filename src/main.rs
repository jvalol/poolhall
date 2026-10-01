//! Fifteen balls in a triangle, and a cue ball to break them with. See `specs/`.

mod paint;
mod poolhall_game;
mod rules;
mod shot;
mod table;
mod view;

use blitzkit::start;
use poolhall_game::PoolhallGame;

fn main() {
    start("poolhall", Box::new(PoolhallGame::new()));
}
