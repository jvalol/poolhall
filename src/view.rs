//! Where the player is standing, and walking round the table. See
//! `specs/0003-round-the-table.md`.

use glam::{vec3, Vec2, Vec3};

/// How far the eye starts from the middle of the table.
pub const BACK: f32 = 56.0;

/// And how near or far it can get.
///
/// Near enough to read a thin cut, far enough that the whole table is still in
/// the window. Closer than this and a ball fills the view; further and the
/// table is a postage stamp.
pub const CLOSEST: f32 = 16.0;
pub const FURTHEST: f32 = 80.0;

/// How far one notch of the wheel moves it.
///
/// Small. At 0.06 a flick of the wheel crossed the whole range, which is a
/// zoom you fight rather than one you use.
pub const BACK_PER_NOTCH: f32 = 0.018;

/// How high it can get and how low it can stoop, in radians.
///
/// Nearly flat to the cloth at one end and nearly straight down at the other,
/// which is a hair under five degrees and a hair under ninety.
///
/// Not flat, because a view along the cloth never meets it and there is
/// nowhere to point. Not exactly straight down either. The reason this spec
/// first gave was wrong: it said the aim beads would be a dot, when the beads
/// lie on the cloth and a plan view is the clearest look at an angle there is.
/// The real reason is the arithmetic. The eye looks along the up axis from
/// directly over the table, and a view matrix built from two parallel vectors
/// is nothing at all.
pub const LOWEST: f32 = 0.08;
pub const HIGHEST: f32 = 1.55;

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

/// How fast panning moves it, as a share of how far out the eye is per pixel.
///
/// A share and not a distance, for the reason a zoom notch is: panning from
/// close up wants small steps and panning from across the room wants large
/// ones, and one fixed number is wrong at both ends.
pub const PAN_PER_PIXEL: f32 = 0.0016;

/// How far off the middle of the table the eye may look, in table halves.
///
/// Panning with nothing to stop it is a camera lost in the black with no way
/// back, and there is no key spare to recentre with. A little past the cushion
/// is as far as anybody wants: enough to put a corner pocket in the middle of
/// the window, not enough to lose the table out of it.
pub const ROAM: f32 = 1.15;

/// Where the player is standing.
#[derive(Debug, Clone, Copy)]
pub struct View {
    /// Round the table, in radians.
    pub about: f32,
    /// And up from the cloth.
    pub above: f32,
    /// And how far out.
    pub back: f32,
    /// And what it is looking at, on the cloth.
    ///
    /// The table's middle until you pan off it. Without this the eye orbited
    /// one fixed point and nothing could bring a particular corner into the
    /// middle of the window: you could walk round the table and lean in, which
    /// is two of the three things a person wants from a camera.
    pub at: Vec3,
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
            back: BACK,
            at: Vec3::ZERO,
        }
    }

    /// Walks round by a drag of this many pixels.
    pub fn dragged(&mut self, across: f32, down: f32) {
        self.about -= across * TURN_PER_PIXEL;
        self.above = (self.above + down * RISE_PER_PIXEL).clamp(LOWEST, HIGHEST);
    }

    /// Leans in or out by this many notches of the wheel.
    ///
    /// A share of where it is rather than a fixed distance, so a notch moves it
    /// as much from close up as from far off. A fixed step crawls at one end
    /// and jumps at the other.
    pub fn zoomed(&mut self, notches: f32) {
        self.back = (self.back * (1.0 - notches * BACK_PER_NOTCH)).clamp(CLOSEST, FURTHEST);
    }

    /// Slides what it is looking at, by a drag of this many pixels.
    ///
    /// Along the cloth and in the eye's own directions, so dragging right moves
    /// the table right whichever side you are standing on. Up the screen is
    /// away from you, which is the ground going under the drag rather than the
    /// camera going over it: the other way round reads as pushing the table.
    pub fn panned(&mut self, across: f32, down: f32, half: Vec2) {
        let by = self.back * PAN_PER_PIXEL;
        let away = vec3(self.about.sin(), 0.0, self.about.cos());
        let right = vec3(away.z, 0.0, -away.x);

        self.at += right * (-across * by) + away * (-down * by);
        self.at.x = self.at.x.clamp(-half.x * ROAM, half.x * ROAM);
        self.at.z = self.at.z.clamp(-half.y * ROAM, half.y * ROAM);
        self.at.y = 0.0;
    }

    /// Where the eye is.
    pub fn eye(&self) -> Vec3 {
        let out = self.above.cos() * self.back;

        self.at
            + vec3(
                self.about.sin() * out,
                self.above.sin() * self.back,
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

    /// Spec 0003: panning slides what the eye looks at along the cloth, so a
    /// corner can be brought into the middle of the window.
    #[test]
    fn panning_slides_what_it_looks_at() {
        let half = glam::vec2(25.0, 14.0);
        let mut view = View::new();
        let was = view.at;

        view.panned(120.0, 0.0, half);

        assert_ne!(view.at, was, "the pan did nothing");
        assert_eq!(view.at.y, 0.0, "it left the cloth: {:?}", view.at);
        // and the eye went with it, rather than the eye staying put and the
        // aim swinging
        assert!(
            (view.eye() - view.at).length() - BACK < 1e-3,
            "it is no longer {} out",
            BACK
        );
    }

    /// Spec 0003: and it slides the way the eye is facing, not the way the
    /// world is laid out.
    #[test]
    fn panning_follows_where_you_are_standing() {
        let half = glam::vec2(25.0, 14.0);
        let mut behind = View::new();
        let mut beside = View::new();
        beside.about = 0.0;

        behind.panned(100.0, 0.0, half);
        beside.panned(100.0, 0.0, half);

        assert_ne!(
            behind.at, beside.at,
            "the same drag moved it the same way from two sides of the table"
        );
    }

    /// Spec 0003: and it cannot be panned off into the black.
    #[test]
    fn panning_stops_at_the_edge_of_the_table() {
        let half = glam::vec2(25.0, 14.0);
        let mut view = View::new();

        for _ in 0..500 {
            view.panned(400.0, 400.0, half);
        }

        assert!(
            view.at.x.abs() <= half.x * ROAM + 1e-3 && view.at.z.abs() <= half.y * ROAM + 1e-3,
            "it panned to {:?}, which is off the table",
            view.at
        );
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
    fn the_wheel_leans_in_and_out() {
        let mut view = View::new();
        let out = view.eye();

        view.zoomed(3.0);
        let near = view.eye();

        assert!(near.length() < out.length(), "it did not lean in");
        assert!(
            near.normalize().dot(out.normalize()) > 0.9999,
            "leaning in moved it round the table"
        );

        view.zoomed(-6.0);
        assert!(view.eye().length() > near.length(), "it did not lean out");
    }

    #[test]
    fn it_cannot_lean_past_the_table_or_into_a_ball() {
        let mut view = View::new();

        view.zoomed(1000.0);
        assert!(
            view.back >= CLOSEST,
            "it leaned past the table: {}",
            view.back
        );

        view.zoomed(-1000.0);
        assert!(view.back <= FURTHEST, "it leaned into the next room");
    }

    #[test]
    fn a_notch_is_worth_the_same_from_anywhere() {
        // a share of where it is rather than a fixed step, or it crawls at one
        // end and jumps at the other
        let mut near = View::new();
        near.back = CLOSEST * 1.5;
        let was_near = near.back;
        near.zoomed(1.0);

        let mut far = View::new();
        far.back = FURTHEST * 0.9;
        let was_far = far.back;
        far.zoomed(1.0);

        let near_share = (was_near - near.back) / was_near;
        let far_share = (was_far - far.back) / was_far;

        assert!(
            (near_share - far_share).abs() < 1e-5,
            "{} {}",
            near_share,
            far_share
        );
    }

    #[test]
    fn it_stops_a_hair_short_of_flat_and_of_straight_down() {
        let mut view = View::new();

        view.dragged(0.0, 10_000.0);
        assert!(view.above <= HIGHEST, "it went past the top");
        assert!(
            view.above < std::f32::consts::FRAC_PI_2,
            "it looks straight down the up axis, which is no view at all"
        );
        assert!(view.eye().y > 0.0);

        view.dragged(0.0, -20_000.0);
        assert!(view.above >= LOWEST, "it went under the cloth");
        assert!(view.above > 0.0, "it lies flat on the cloth");
        assert!(view.eye().y > 0.0, "the eye is under the table");
    }

    #[test]
    fn it_reaches_from_nearly_flat_to_nearly_overhead() {
        // the range is the point: a plan view is the clearest look at an angle
        // and a low one is the only way to read a thin cut
        let mut view = View::new();

        view.dragged(0.0, 10_000.0);
        let over = view.eye();

        view.dragged(0.0, -20_000.0);
        let low = view.eye();

        assert!(
            over.y > over.length() * 0.99,
            "the top of the range is not nearly overhead: {:?}",
            over
        );
        assert!(
            low.y < low.length() * 0.15,
            "the bottom of the range is not nearly flat: {:?}",
            low
        );
    }
}
