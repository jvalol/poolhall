//! Where the player is standing, and walking round the table. See
//! `specs/0003-round-the-table.md`.

use glam::{vec3, Vec3};

/// How far the eye is from the middle of the table.
pub const BACK: f32 = 50.0;

/// How high it can get and how low it can stoop, in radians.
///
/// Never flat, because a view along the cloth cannot point at anywhere on it,
/// and never straight down, because the aim beads would be a dot.
pub const LOWEST: f32 = 0.18;
pub const HIGHEST: f32 = 1.35;

/// Where it starts: behind the head rail and well up.
///
/// The head rail is the negative x end, and `eye` lays the angle out as
/// `(sin, cos)`, so behind it is minus a quarter turn rather than a half. A
/// half turn put the eye along the side rail, and the test that was meant to
/// catch it passed because `sin(PI)` is a hair negative.
pub const FROM: f32 = -std::f32::consts::FRAC_PI_2;
pub const ABOVE: f32 = 0.95;

/// How fast dragging moves it, in radians a pixel.
pub const TURN_PER_PIXEL: f32 = 0.006;
pub const RISE_PER_PIXEL: f32 = 0.004;

/// Where the player is standing.
#[derive(Debug, Clone, Copy)]
pub struct View {
    /// Round the table, in radians.
    pub about: f32,
    /// And up from the cloth.
    pub above: f32,
}

impl Default for View {
    fn default() -> Self {
        Self::new()
    }
}

impl View {
    pub fn new() -> Self {
        Self {
            about: FROM,
            above: ABOVE,
        }
    }

    /// Walks round by a drag of this many pixels.
    pub fn dragged(&mut self, across: f32, down: f32) {
        self.about -= across * TURN_PER_PIXEL;
        self.above = (self.above + down * RISE_PER_PIXEL).clamp(LOWEST, HIGHEST);
    }

    /// Where the eye is.
    pub fn eye(&self) -> Vec3 {
        let out = self.above.cos() * BACK;

        vec3(
            self.about.sin() * out,
            self.above.sin() * BACK,
            self.about.cos() * out,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_starts_behind_the_head_rail_and_above_the_cloth() {
        let view = View::new();
        let eye = view.eye();

        // behind the head rail, not merely a hair the right side of the middle
        assert!(
            eye.x < -BACK * 0.5,
            "it does not start behind the head rail: {:?}",
            eye
        );
        assert!(
            eye.z.abs() < BACK * 0.1,
            "it starts off to one side: {:?}",
            eye
        );
        assert!(eye.y > 0.0, "it starts under the table: {:?}", eye);
        assert!((eye.length() - BACK).abs() < 1e-3);
    }

    #[test]
    fn dragging_walks_round_the_table() {
        let mut view = View::new();
        let was = view.eye();

        view.dragged(200.0, 0.0);
        let now = view.eye();

        assert_ne!(now, was, "the drag did nothing");
        assert!(
            (now.length() - was.length()).abs() < 1e-3,
            "it walked towards the table rather than round it"
        );
        assert!(
            (now.y - was.y).abs() < 1e-3,
            "walking round changed the height"
        );
    }

    #[test]
    fn walking_right_round_comes_back() {
        let mut view = View::new();
        let was = view.eye();

        view.dragged(std::f32::consts::TAU / TURN_PER_PIXEL, 0.0);

        assert!((view.eye() - was).length() < 1e-2, "{:?}", view.eye());
    }

    #[test]
    fn dragging_up_and_down_changes_the_height() {
        let mut view = View::new();
        let was = view.eye().y;

        view.dragged(0.0, 100.0);
        assert!(view.eye().y > was, "it did not rise");

        view.dragged(0.0, -300.0);
        assert!(view.eye().y < was, "it did not stoop");
    }

    #[test]
    fn it_never_lies_flat_or_looks_straight_down() {
        let mut view = View::new();

        view.dragged(0.0, 10_000.0);
        assert!(view.above <= HIGHEST, "it went past the top");
        assert!(view.eye().y > 0.0);

        view.dragged(0.0, -20_000.0);
        assert!(view.above >= LOWEST, "it went under the cloth");
        assert!(view.eye().y > 0.0, "the eye is under the table");
    }
}
