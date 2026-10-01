//! What a ball is made of, and running a shot until everything stops. See
//! `specs/0001-the-rack.md`.

use blitzkit::physics::Body;
use glam::{vec3, Vec3};

use crate::table::{cloth, cushions, BALL_RADIUS};

/// How hard the hardest shot is, in units a second.
pub const HARDEST: f32 = 20.0;

/// And the softest, which is what a press with no hold behind it sends.
///
/// Not nothing. carom learned this from the other side: a shot of nothing rolls
/// nowhere and hands the turn straight back, which is the game ignoring you.
pub const SOFTEST: f32 = HARDEST * 0.1;

/// How slowly everything has to be going before a shot is over.
pub const STILL: f32 = 0.08;

/// What the physics runs at while a shot is rolling, whatever the frame rate.
pub const STEP: f32 = 1.0 / 120.0;

/// How many steps a shot may take before something is wrong. Thirty seconds.
pub const LONGEST: usize = (30.0 / STEP) as usize;

/// Down, and how hard.
///
/// Twelve rather than anything larger. blitzkit's settling speed is a fixed
/// 0.6 and gravity puts `g * dt` back into a resting body every step, so at 24
/// that is a third of the threshold and a ball sits trembling on the cloth for
/// ever. carom found that the slow way.
pub const GRAVITY: Vec3 = vec3(0.0, -12.0, 0.0);

/// How much of a collision comes back.
///
/// This is ball on ball, which is what the player is aiming with, so the real
/// number goes here. The cushions get it too, because restitution lives on the
/// body and blitzkit's static world has none, so the rails bounce harder than
/// rails do. That is written down rather than hidden by detuning the balls.
pub const BOUNCE: f32 = 0.93;

/// How much the cloth grips.
///
/// Low on purpose. Friction turns backspin into forward roll, so a gripping
/// cloth eats the draw before the balls meet: blitzkit spec 0032 measured the
/// same bottom strike ending behind the contact at 0.3 and past it at 0.95.
/// A game about where the cue ball stops wants the cloth that lets it come
/// back.
pub const GRIP: f32 = 0.3;

/// How much a roll costs, per blitzkit spec 0031.
///
/// Measured rather than reasoned: see `shot::tests::a_ball_can_run_the_table`.
pub const ROLLING: f32 = 0.1;

/// What a ball weighs. They all weigh the same, cue ball included, which is
/// near enough true and is one number rather than two.
pub const BALL_MASS: f32 = 1.0;

/// A ball resting at `at`.
pub fn ball(at: Vec3) -> Body {
    Body::new(at, BALL_RADIUS, BALL_MASS)
        .with_restitution(BOUNCE)
        .with_friction(GRIP)
        .with_rolling(ROLLING)
}

/// Where the cue tip meets the ball, given how far off its middle to strike.
///
/// `off` is in ball radii: `(0, 0)` is dead centre, `(0, -1)` the very bottom,
/// `(1, 0)` the right edge. The point is on the face the cue comes from, which
/// is the side the shot is going away from.
pub fn tip(middle: Vec3, way: Vec3, off: glam::Vec2) -> Vec3 {
    let across = vec3(-way.z, 0.0, way.x).normalize_or_zero();

    middle - way.normalize_or_zero() * BALL_RADIUS
        + across * off.x * BALL_RADIUS
        + Vec3::Y * off.y * BALL_RADIUS
}

/// Everything a shot runs into: the cloth and the cushions.
pub fn world() -> Vec<blitzkit::collision::Aabb> {
    let mut all = vec![cloth()];
    all.extend(cushions());

    all
}

/// Whether everything has stopped.
pub fn nothing_is_moving(bodies: &[Body]) -> bool {
    bodies.iter().all(|body| body.velocity.length() < STILL)
}

#[cfg(test)]
mod tests {
    use super::*;
    use blitzkit::physics::step;
    use glam::vec2;

    use crate::table::HALF_LONG;

    fn at(x: f32, z: f32) -> Vec3 {
        vec3(x, BALL_RADIUS, z)
    }

    /// Runs until nothing is moving, and says how many steps that took.
    fn settle(bodies: &mut [Body]) -> usize {
        for taken in 1..=LONGEST {
            step(bodies, &world(), GRAVITY, STEP);
            if nothing_is_moving(bodies) {
                return taken;
            }
        }

        LONGEST
    }

    #[test]
    fn a_ball_can_run_the_table() {
        // the number ROLLING is chosen for. A table is 44 long, and a shot
        // that cannot reach the far rail and come back is a shot that cannot
        // be played for position.
        let mut bodies = [ball(at(-HALF_LONG * 0.5, 0.0)).with_velocity(vec3(HARDEST, 0.0, 0.0))];

        let mut ran = 0.0;
        let mut was = bodies[0].position;
        for _ in 1..=LONGEST {
            step(&mut bodies, &world(), GRAVITY, STEP);
            ran += bodies[0].position.distance(was);
            was = bodies[0].position;
            if nothing_is_moving(&bodies) {
                break;
            }
        }

        assert!(
            ran > HALF_LONG * 2.0,
            "the hardest shot ran {}, and the table is {} long",
            ran,
            HALF_LONG * 2.0
        );
        assert!(
            ran < HALF_LONG * 6.0,
            "the hardest shot ran {}, which is three tables",
            ran
        );
    }

    /// Strikes the cue ball `off` its middle into another ball, and says where
    /// it ended up along the way it was going.
    fn after_the_hit(off: glam::Vec2) -> f32 {
        let way = Vec3::X;
        let mut bodies = [ball(at(-3.0, 0.0)), ball(at(0.0, 0.0))];
        let tip = tip(bodies[0].position, way, off);
        bodies[0].strike(way * HARDEST * 0.4, tip);

        settle(&mut bodies);

        bodies[0].position.x
    }

    /// Where the two of them touch, a ball's width short of the one standing
    /// still. A cue ball that ends behind this came back.
    const TOUCHED_AT: f32 = -1.0;

    #[test]
    fn low_draws_the_cue_ball_back() {
        let low = after_the_hit(vec2(0.0, -0.9));

        assert!(
            low < TOUCHED_AT,
            "it ended at {}, and they touched at {}",
            low,
            TOUCHED_AT
        );
    }

    #[test]
    fn high_runs_the_cue_ball_on() {
        let high = after_the_hit(vec2(0.0, 0.9));
        let flat = after_the_hit(vec2(0.0, 0.0));

        assert!(high > flat + 0.8, "high {} flat {}", high, flat);
    }

    #[test]
    fn the_whole_range_is_in_order() {
        let heights = [-0.9f32, -0.5, 0.0, 0.5, 0.9];
        let ends: Vec<f32> = heights
            .iter()
            .map(|h| after_the_hit(vec2(0.0, *h)))
            .collect();

        for pair in ends.windows(2) {
            assert!(pair[1] > pair[0], "{:?} for heights {:?}", ends, heights);
        }
    }

    #[test]
    fn side_changes_the_angle_off_a_rail() {
        // the one thing side does on this engine. A cushion's normal is
        // horizontal, so a spin about the vertical is no longer about the
        // contact normal and has somewhere to act.
        // from under one side pocket at the middle of a rail length, so the
        // ball meets cushion rather than going straight down a pocket
        let way = vec3(0.0, 0.0, 1.0);
        let from = at(HALF_LONG * 0.5, 0.0);
        let mut plain = [ball(from)];
        let mut sided = [ball(from)];

        plain[0].strike(
            way * HARDEST * 0.5,
            tip(plain[0].position, way, vec2(0.0, 0.0)),
        );
        sided[0].strike(
            way * HARDEST * 0.5,
            tip(sided[0].position, way, vec2(0.9, 0.0)),
        );

        settle(&mut plain);
        settle(&mut sided);

        assert!(
            (plain[0].position.x - sided[0].position.x).abs() > 0.5,
            "plain came back to {} and sided to {}",
            plain[0].position.x,
            sided[0].position.x
        );
    }

    #[test]
    fn the_tip_lands_on_the_ball() {
        let middle = at(0.0, 0.0);

        for off in [
            vec2(0.0, 0.0),
            vec2(0.9, 0.0),
            vec2(0.0, -0.9),
            vec2(0.5, 0.5),
        ] {
            let where_ = tip(middle, Vec3::X, off);

            assert!(
                where_.distance(middle) <= BALL_RADIUS * 1.74,
                "a tip at {:?} is {} from the middle",
                off,
                where_.distance(middle)
            );
            assert!(where_.x <= middle.x, "the cue reached round the far side");
        }
    }
}
