//! Fifteen balls in a triangle, and a cue ball to break them with. See `specs/`.

mod diamond_game;
mod rules;
mod shot;
mod table;

use blitzkit::start;
use diamond_game::DiamondGame;

fn main() {
    start("diamond", Box::new(DiamondGame::new()));
}
