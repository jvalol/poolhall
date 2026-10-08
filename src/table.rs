//! The cloth, the cushions, the pockets and the rack. See
//! `specs/0001-the-rack.md`.

use blitzkit::collision::Aabb;
use glam::{vec2, vec3, Vec3};

/// Every ball is this. One across, so every other number here is in balls.
pub const BALL_RADIUS: f32 = 0.5;

/// The playing surface, measured out from the middle. Twice as long as it is
/// wide, which is what a table is.
pub const HALF_LONG: f32 = 22.0;
pub const HALF_WIDE: f32 = 11.0;

/// How close a ball's middle has to get to a pocket to be down it.
///
/// A real pocket is about two balls across at the mouth, and a ball is down
/// once its middle is past the jaws rather than when the whole of it is
/// through.
pub const POCKET_MOUTH: f32 = 1.6;

/// How thick the cushions are, and how high.
const RAIL_THICK: f32 = 1.0;
const RAIL_HIGH: f32 = 1.4;

/// How wide a gap each pocket leaves between cushions.
const RAIL_GAP: f32 = 2.4;

/// How far the cloth reaches past the rails, so a ball in a pocket gap is
/// standing on something until it is taken off the table.
pub const APRON: f32 = 3.0;

/// How close to a cushion counts as against it.
///
/// A hair, because the solver settles a ball a hair off the face rather than
/// exactly on it, and a test for exact contact never fires.
pub const RAIL_SLACK: f32 = 0.02;

/// The cloth, a solid with its top at y zero.
///
/// One piece, with no holes in it. The spec first called for pockets cut out of
/// it, so a ball that reached one fell through and was caught on the way down.
/// That is prettier and it is also how carom lost half a minute a shot: a body
/// with nothing under it falls for ever, a falling body is a body still moving,
/// and the shot never ends. A ball here is potted by reaching the jaws and is
/// taken off the table, and nothing ever falls.
pub fn cloth() -> Aabb {
    // wider than the playing surface, so a ball on its way through a pocket gap
    // still has something under it for the step or two before it is potted
    Aabb::from_center_size(
        vec3(0.0, -1.0, 0.0),
        vec3((HALF_LONG + APRON) * 2.0, 2.0, (HALF_WIDE + APRON) * 2.0),
    )
}

/// Where the six pockets are: one at each corner, one in each long rail.
pub fn pockets() -> Vec<Vec3> {
    let mut at = Vec::with_capacity(6);

    for along in [-HALF_LONG, HALF_LONG] {
        for across in [-HALF_WIDE, HALF_WIDE] {
            at.push(vec3(along, BALL_RADIUS, across));
        }
    }
    for across in [-HALF_WIDE, HALF_WIDE] {
        at.push(vec3(0.0, BALL_RADIUS, across));
    }

    at
}

/// The cushions: six lengths of rail with a gap at every pocket.
///
/// Two along each long side, one at each end. The gaps are what a ball goes
/// through on its way to a pocket, so they are real geometry rather than a
/// special case in the rules.
pub fn cushions() -> Vec<Aabb> {
    let mut rails = Vec::with_capacity(6);
    let middle = RAIL_HIGH * 0.5;
    let out = RAIL_THICK * 0.5;

    // the long sides, in two lengths each: corner pocket to side pocket
    for across in [-HALF_WIDE, HALF_WIDE] {
        for half in [-1.0f32, 1.0] {
            rails.push(Aabb::from_center_size(
                vec3(
                    half * HALF_LONG * 0.5,
                    middle,
                    across + across.signum() * out,
                ),
                vec3(HALF_LONG - RAIL_GAP, RAIL_HIGH, RAIL_THICK),
            ));
        }
    }

    // and the two ends, corner pocket to corner pocket
    for along in [-HALF_LONG, HALF_LONG] {
        rails.push(Aabb::from_center_size(
            vec3(along + along.signum() * out, middle, 0.0),
            vec3(RAIL_THICK, RAIL_HIGH, HALF_WIDE * 2.0 - RAIL_GAP),
        ));
    }

    rails
}

/// Whether a ball could rest here: on the cloth, inside the cushions, and not
/// hanging over a pocket.
///
/// The cursor ray meets the cloth's plane wherever it is pointed, and that plane
/// goes on for ever, so without this the cue ball could be put down in the dark
/// beyond the rails.
pub fn is_on_the_table(at: Vec3) -> bool {
    let room = BALL_RADIUS + 1e-3;

    at.x.abs() < HALF_LONG - room && at.z.abs() < HALF_WIDE - room && !is_potted(at)
}

/// Whether a ball at this position is down a pocket.
pub fn is_potted(at: Vec3) -> bool {
    pockets()
        .iter()
        .any(|pocket| vec2(at.x - pocket.x, at.z - pocket.z).length() < POCKET_MOUTH)
}

/// How many balls are racked, and which of them is which.
pub const BALLS: usize = 15;
/// The ball at the apex, and the one buried in the middle of the triangle.
pub const ONE: usize = 1;
pub const EIGHT: usize = 8;

/// How far apart the rack sits them, middle to middle.
///
/// A hair over touching. A real rack is pressed together, and this is as close
/// as the engine can be asked for without starting the balls inside each other.
const TIGHT: f32 = 1.002;

/// Where the apex of the rack sits: a quarter of the table from the foot rail.
pub fn foot_spot() -> Vec3 {
    vec3(HALF_LONG * 0.5, BALL_RADIUS, 0.0)
}

/// Where the cue ball breaks from: a quarter of the table from the head rail.
pub fn head_spot() -> Vec3 {
    vec3(head_string(), BALL_RADIUS, 0.0)
}

/// Where the head string runs, which is the line the kitchen ends at.
///
/// A quarter of the way up the table from the head rail, which is where it is
/// on a real one and where the head spot has always sat.
pub fn head_string() -> f32 {
    -HALF_LONG * 0.5
}

/// Whether a spot is in the kitchen: on the table, and behind the head string.
///
/// On the line counts as behind it, the way it does in the hand: a ball split
/// by the line is in, and the head spot itself is the obvious place to break
/// from.
pub fn in_the_kitchen(at: Vec3) -> bool {
    is_on_the_table(at) && at.x <= head_string()
}

/// The rack, as a position for each ball from one to fifteen.
///
/// A triangle pointing at the head of the table: one at the apex, eight in the
/// middle where it is hardest to reach, and the rest wherever they fall.
pub fn rack() -> Vec<Vec3> {
    let apex = foot_spot();
    let row = TIGHT * 0.866; // the height of a triangle of touching balls
    let rows: [usize; 5] = [1, 2, 3, 4, 5];

    // one, then the rest in order, with the nine put in the middle afterwards
    let mut spots = Vec::with_capacity(BALLS);
    for (n, count) in rows.iter().enumerate() {
        let along = apex.x + n as f32 * row;
        for seat in 0..*count {
            let across = (seat as f32 - (*count as f32 - 1.0) * 0.5) * TIGHT;
            spots.push(vec3(along, BALL_RADIUS, across));
        }
    }

    // the apex is the one, the middle of the third row is the eight, and the
    // others take what is left in the order they come
    let middle = 1 + 2 + 1;
    let mut at = vec![Vec3::ZERO; BALLS];
    at[ONE - 1] = spots[0];
    at[EIGHT - 1] = spots[middle];

    // the seats left over, handed to the balls that have no seat of their own.
    // Counting a ball number up alongside the seats put the eight back in the
    // pile and left the last ball at the origin.
    let mut spare = spots
        .iter()
        .enumerate()
        .filter(|(seat, _)| *seat != 0 && *seat != middle)
        .map(|(_, spot)| *spot);

    for ball in 1..=BALLS {
        if ball == ONE || ball == EIGHT {
            continue;
        }
        at[ball - 1] = spare.next().expect("a seat for every ball");
    }

    at
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_rack_is_a_triangle() {
        let at = rack();

        assert_eq!(at.len(), BALLS);

        for ball in &at {
            assert!(
                !is_potted(*ball),
                "a ball starts down a pocket at {:?}",
                ball
            );
            assert!(
                ball.x.abs() < HALF_LONG && ball.z.abs() < HALF_WIDE,
                "a ball starts off the table at {:?}",
                ball
            );
            assert_eq!(ball.y, BALL_RADIUS, "a ball starts off the cloth");
        }

        for (n, one) in at.iter().enumerate() {
            for other in at.iter().skip(n + 1) {
                assert!(
                    one.distance(*other) >= BALL_RADIUS * 2.0,
                    "two balls start inside each other, {:?} and {:?}",
                    one,
                    other
                );
            }
        }

        // five rows of 1, 2, 3, 2, 1, which is what makes it a diamond
        let mut rows: Vec<f32> = at.iter().map(|ball| ball.x).collect();
        rows.sort_by(|a, b| a.partial_cmp(b).unwrap());
        rows.dedup_by(|a, b| (*a - *b).abs() < 1e-3);

        assert_eq!(rows.len(), 5, "it is not five rows deep: {:?}", rows);

        // and each row is wider than the last, which is what makes it a
        // triangle rather than the diamond this started as
        let mut wide: Vec<usize> = rows
            .iter()
            .map(|row| at.iter().filter(|ball| (ball.x - row).abs() < 1e-3).count())
            .collect();
        wide.dedup();

        assert_eq!(wide, vec![1, 2, 3, 4, 5], "the rows are not a triangle");
    }

    #[test]
    fn the_one_leads_and_the_eight_is_buried() {
        let at = rack();
        let one = at[ONE - 1];
        let eight = at[EIGHT - 1];

        for (n, ball) in at.iter().enumerate() {
            if n + 1 == ONE {
                continue;
            }
            assert!(
                ball.x > one.x - 1e-3,
                "ball {} is in front of the one",
                n + 1
            );
        }

        assert!(eight.x > one.x, "the eight is not behind the one");
        assert!(eight.z.abs() < 1e-3, "the eight is not on the middle line");

        // the middle of the triangle: two rows in front of it and two behind
        let rows: Vec<f32> = {
            let mut all: Vec<f32> = at.iter().map(|ball| ball.x).collect();
            all.sort_by(|a, b| a.partial_cmp(b).unwrap());
            all.dedup_by(|a, b| (*a - *b).abs() < 1e-3);
            all
        };

        assert!(
            (eight.x - rows[2]).abs() < 1e-3,
            "the eight is not in the third row"
        );
    }

    #[test]
    fn a_ball_can_only_rest_on_the_table() {
        assert!(is_on_the_table(vec3(0.0, BALL_RADIUS, 0.0)), "the middle");
        assert!(is_on_the_table(head_spot()), "the head spot");
        assert!(is_on_the_table(foot_spot()), "the foot spot");

        for ball in rack() {
            assert!(is_on_the_table(ball), "a racked ball at {:?}", ball);
        }

        // past the rails, which is where the cloth's plane carries on to
        assert!(!is_on_the_table(vec3(HALF_LONG + 5.0, BALL_RADIUS, 0.0)));
        assert!(!is_on_the_table(vec3(0.0, BALL_RADIUS, HALF_WIDE + 5.0)));
        assert!(!is_on_the_table(vec3(-500.0, BALL_RADIUS, 500.0)));

        // and hanging over the jaws of a pocket
        for pocket in pockets() {
            assert!(!is_on_the_table(pocket), "a pocket at {:?}", pocket);
        }

        // a ball touching a cushion is still on the table
        assert!(is_on_the_table(vec3(
            HALF_LONG - BALL_RADIUS - 0.05,
            BALL_RADIUS,
            0.0
        )));
    }

    #[test]
    fn a_ball_in_the_jaws_is_potted() {
        for pocket in pockets() {
            assert!(is_potted(pocket), "a ball in a pocket is not potted");
            assert!(
                is_potted(pocket + vec3(0.0, 0.0, POCKET_MOUTH * 0.5)),
                "a ball in the jaws is not potted"
            );
        }
    }

    #[test]
    fn the_rails_are_not_pockets() {
        // the middle of a rail, and the middle of the table
        assert!(!is_potted(vec3(HALF_LONG * 0.5, BALL_RADIUS, HALF_WIDE)));
        assert!(!is_potted(vec3(0.0, BALL_RADIUS, 0.0)));

        for ball in rack() {
            assert!(!is_potted(ball));
        }
    }

    #[test]
    fn a_ball_can_reach_every_pocket() {
        // the gaps between cushions are what a ball goes through, so a pocket
        // walled off by its own rails is a pocket nothing can be potted in
        for pocket in pockets() {
            for rail in cushions() {
                let inside = pocket.x > rail.min.x
                    && pocket.x < rail.max.x
                    && pocket.z > rail.min.z
                    && pocket.z < rail.max.z;

                assert!(!inside, "a rail covers the pocket at {:?}", pocket);
            }
        }
    }
}
