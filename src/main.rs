//! Fifteen balls in a triangle, and a cue ball to break them with. See `specs/`.

mod diamond_game;
mod paint;
mod rules;
mod shot;
mod table;
mod view;

use blitzkit::start;
use diamond_game::DiamondGame;

fn main() {
    start("diamond", Box::new(DiamondGame::new()));
}
