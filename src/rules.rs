//! A run: the balls, whose turn it would be if there were two of you, what is
//! legal and what is a foul. See `specs/0001-the-rack.md`.

use blitzkit::collision::{sweep_sphere, Sphere};
use blitzkit::physics::{step, Body};
use glam::Vec3;

use crate::shot::{self, ball};
use crate::table::{self, BALLS};

/// The cue ball is body zero and the nine numbered balls follow it, so a ball's
/// number is its index and nothing has to be searched for.
pub const CUE: usize = 0;

/// Where a run has got to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Phase {
    /// Pointing, with nothing moving.
    Aiming,
    /// A shot is rolling. Nothing can be aimed until it stops.
    Rolling,
    /// The nine is down.
    Over,
}

/// What a shot turned out to be.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    /// Legal, and nothing went down.
    Dry,
    /// Legal, and something did.
    Potted,
    /// Not legal. The cue ball is in hand.
    Foul(Foul),
}

/// Why a shot was a foul.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Foul {
    /// Hit nothing at all.
    Missed,
    /// Potted the cue ball.
    Scratched,
    /// Nothing reached a cushion after the balls met.
    NoRail,
}

/// What the shot now rolling has done so far.
#[derive(Debug, Clone, Default)]
struct Happenings {
    first_hit: Option<usize>,
    reached_a_rail: bool,
    potted: Vec<usize>,
}

pub struct Run {
    pub bodies: Vec<Body>,
    /// Which balls are off the table. The cue ball is never down for long: it
    /// comes back in hand, so it is not counted here.
    down: [bool; BALLS + 1],
    shots: u32,
    fouls: u32,
    phase: Phase,
    rolled: usize,
    doing: Happenings,
    /// What the last shot came to, for the readout to say.
    pub last: Option<Outcome>,
    /// Set after a foul: the cue ball may be placed anywhere.
    pub in_hand: bool,
}

impl Default for Run {
    fn default() -> Self {
        Self::new()
    }
}

impl Run {
    pub fn new() -> Self {
        let mut bodies = vec![ball(table::head_spot())];
        bodies.extend(table::rack().into_iter().map(ball));

        Self {
            bodies,
            down: [false; BALLS + 1],
            shots: 0,
            fouls: 0,
            phase: Phase::Aiming,
            rolled: 0,
            doing: Happenings::default(),
            last: None,
            in_hand: true,
        }
    }

    pub fn phase(&self) -> Phase {
        self.phase
    }

    pub fn shots(&self) -> u32 {
        self.shots
    }

    pub fn fouls(&self) -> u32 {
        self.fouls
    }

    pub fn is_down(&self, ball: usize) -> bool {
        self.down.get(ball).copied().unwrap_or(false)
    }

    pub fn cue(&self) -> Vec3 {
        self.bodies[CUE].position
    }

    /// How many balls are still on the table.
    ///
    /// Any of them may be hit first and any may be potted: this spec had the
    /// lowest ball first, which is nine ball's rule, and Jake took it out. What
    /// is left is potting nine balls in as few shots as you can.
    pub fn left(&self) -> usize {
        (1..=BALLS).filter(|ball| !self.down[*ball]).count()
    }

    /// Puts the cue ball somewhere, which is only allowed in hand.
    pub fn place(&mut self, at: Vec3) {
        if !self.in_hand || self.phase != Phase::Aiming {
            return;
        }

        self.bodies[CUE].position = at;
        self.bodies[CUE].velocity = Vec3::ZERO;
        self.bodies[CUE].spin = Vec3::ZERO;

        // putting it down is what ends being in hand. Without this every click
        // put the ball somewhere and none of them ever got as far as a shot.
        self.in_hand = false;
    }

    /// Takes the shot: an impulse, struck `off` the middle of the cue ball.
    pub fn shoot(&mut self, way: Vec3, speed: f32, off: glam::Vec2) {
        if self.phase != Phase::Aiming || way.length_squared() < 1e-6 || speed < shot::STILL {
            return;
        }

        let cue = self.bodies[CUE].position;
        let tip = shot::tip(cue, way, off);
        self.bodies[CUE].strike(way.normalize() * speed, tip);

        self.shots += 1;
        self.phase = Phase::Rolling;
        self.rolled = 0;
        self.doing = Happenings::default();
        self.in_hand = false;
    }

    /// One step of a rolling shot: move what is in play, write down what
    /// happened, and see whether the shot is over.
    pub fn step(&mut self, dt: f32) {
        if self.phase != Phase::Rolling {
            return;
        }

        let playing: Vec<usize> = (0..=BALLS).filter(|ball| !self.down[*ball]).collect();
        let mut moving: Vec<Body> = playing.iter().map(|at| self.bodies[*at]).collect();
        // where each ball was and where it was going, which is the path the
        // engine is about to sweep
        let before: Vec<(Vec3, Vec3)> = moving
            .iter()
            .map(|body| (body.position, body.velocity))
            .collect();

        step(&mut moving, &shot::world(), shot::GRAVITY, dt);

        for (n, at) in playing.iter().enumerate() {
            self.bodies[*at] = moving[n];
        }

        self.watch(&playing, &before, dt);

        self.rolled += 1;
        if !shot::nothing_is_moving(&moving) && self.rolled < shot::LONGEST {
            return;
        }

        self.finish();
    }

    /// Writes down what this step did: the first ball the cue ball met, whether
    /// anything has reached a cushion since, and what went down.
    fn watch(&mut self, playing: &[usize], before: &[(Vec3, Vec3)], dt: f32) {
        let cue = self.bodies[CUE].position;

        if self.doing.first_hit.is_none() {
            let touching = table::BALL_RADIUS * 2.0 + 1e-3;
            let mut met: Option<usize> = None;
            for ball in playing.iter().skip(1) {
                if self.bodies[*ball].position.distance(cue) <= touching {
                    // the lowest of any it meets in the same step, which is the
                    // kindest reading of a simultaneous contact
                    met = Some(met.map_or(*ball, |was: usize| was.min(*ball)));
                }
            }
            self.doing.first_hit = met;
        } else if !self.doing.reached_a_rail {
            self.doing.reached_a_rail = playing
                .iter()
                .zip(before)
                .any(|(_, (was, going))| self.met_a_rail(*was, *going, dt));
        }

        // taken off the table the moment it reaches the jaws, not when the shot
        // ends. Past the cushions there is only apron and then nothing, and a
        // ball with nothing under it falls for ever: a falling body is a body
        // still moving, so the shot would never end. carom waited twenty
        // seconds for exactly this.
        for ball in playing {
            if table::is_potted(self.bodies[*ball].position) {
                self.doing.potted.push(*ball);
                self.down[*ball] = true;
                self.bodies[*ball].velocity = Vec3::ZERO;
                self.bodies[*ball].spin = Vec3::ZERO;
            }
        }
    }

    /// Whether a ball meets a cushion on the path it is about to take.
    ///
    /// Along where it was going, not from where it was to where it ended up.
    /// Two wrong answers came before this one. Asking whether a ball is near a
    /// rail misses it: a ball bounces inside the step it touches, so it never
    /// ends one at touching distance, and the closest this one got was 0.536
    /// against a ball radius of 0.5. Sweeping from where it was to where it
    /// ended up misses it too, because a step containing a bounce has a net
    /// displacement pointing away from the rail.
    ///
    /// Where it was and where it was going is the path `through_the_world`
    /// sweeps, and `sweep_sphere` is what it sweeps with.
    fn met_a_rail(&self, was: Vec3, going: Vec3, dt: f32) -> bool {
        let path = going * dt;
        if path.length_squared() < 1e-12 {
            return false;
        }

        let body = Sphere::new(was, table::BALL_RADIUS);

        table::cushions()
            .iter()
            .any(|rail| sweep_sphere(&body, path, rail).is_some())
    }

    /// The shot is over: take down what went down, and say what it was.
    fn finish(&mut self) {
        let potted = std::mem::take(&mut self.doing.potted);
        let scratched = potted.contains(&CUE);

        // the cue ball always comes back, however it left
        self.down[CUE] = false;

        let foul = if scratched {
            Some(Foul::Scratched)
        } else {
            match self.doing.first_hit {
                None => Some(Foul::Missed),
                Some(_) if !self.doing.reached_a_rail && potted.is_empty() => Some(Foul::NoRail),
                _ => None,
            }
        };

        if let Some(why) = foul {
            self.fouls += 1;
            self.in_hand = true;
            self.last = Some(Outcome::Foul(why));
            self.bodies[CUE].position = table::head_spot();
            self.bodies[CUE].velocity = Vec3::ZERO;
            self.bodies[CUE].spin = Vec3::ZERO;
        } else {
            self.last = Some(if potted.is_empty() {
                Outcome::Dry
            } else {
                Outcome::Potted
            });
        }

        // an empty table ends it, and a foul does not take a ball back out of a
        // pocket
        self.phase = if self.left() == 0 {
            Phase::Over
        } else {
            Phase::Aiming
        };

        // whatever is still rolling stops: the shot is over
        for body in &mut self.bodies {
            body.velocity = Vec3::ZERO;
            body.spin = Vec3::ZERO;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::table::NINE;
    use glam::{vec2, vec3};

    use crate::table::{HALF_LONG, HALF_WIDE};

    /// Runs a shot out without drawing it, and says how many steps it took.
    fn settle(run: &mut Run) -> usize {
        for taken in 1..=shot::LONGEST {
            run.step(shot::STEP);
            if run.phase() != Phase::Rolling {
                return taken;
            }
        }

        shot::LONGEST
    }

    /// A run with everything cleared off the table but the balls named.
    fn only(balls: &[usize]) -> Run {
        let mut run = Run::new();
        for ball in 1..=BALLS {
            if balls.contains(&ball) {
                continue;
            }
            if ball == NINE {
                // parked in a corner, out of every shot these tests take.
                // Taking the nine down ends the run, and a run that is over
                // cannot be shot.
                run.bodies[ball].position =
                    vec3(-HALF_LONG + 2.0, table::BALL_RADIUS, -HALF_WIDE + 2.0);
                continue;
            }
            run.down[ball] = true;
            run.bodies[ball].position = vec3(0.0, -50.0, 0.0);
        }

        run
    }

    /// Puts a ball in front of the cue ball, a straight shot down the table.
    fn lined_up(run: &mut Run, ball: usize, away: f32) {
        run.bodies[CUE].position = vec3(-HALF_LONG * 0.5, table::BALL_RADIUS, 0.0);
        run.bodies[ball].position = vec3(-HALF_LONG * 0.5 + away, table::BALL_RADIUS, 0.0);
        run.in_hand = false;
    }

    #[test]
    fn a_new_run_is_a_full_rack() {
        let run = Run::new();

        assert_eq!(run.shots(), 0);
        assert_eq!(run.fouls(), 0);
        assert_eq!(run.left(), BALLS);
        assert_eq!(run.phase(), Phase::Aiming);
        assert!(run.in_hand, "the break is from hand");
    }

    #[test]
    fn any_ball_first_is_legal() {
        // this had to be the lowest ball on the table, which is nine ball's
        // rule and the one Jake took out
        let mut run = only(&[1, 5]);
        lined_up(&mut run, 5, 6.0);
        run.bodies[1].position = vec3(0.0, table::BALL_RADIUS, HALF_WIDE * 0.7);

        run.shoot(Vec3::X, shot::HARDEST, vec2(0.0, 0.0));
        settle(&mut run);

        assert_eq!(run.fouls(), 0, "{:?}", run.last);
    }

    #[test]
    fn missing_everything_is_a_foul() {
        let mut run = only(&[1]);
        run.bodies[CUE].position = vec3(0.0, table::BALL_RADIUS, 0.0);
        run.bodies[1].position = vec3(0.0, table::BALL_RADIUS, HALF_WIDE * 0.6);
        run.in_hand = false;

        run.shoot(Vec3::X, shot::HARDEST * 0.3, vec2(0.0, 0.0));
        settle(&mut run);

        assert_eq!(run.last, Some(Outcome::Foul(Foul::Missed)));
    }

    #[test]
    fn potting_the_cue_ball_is_a_foul() {
        let mut run = only(&[1]);
        // in the jaws of a corner pocket, pointed at it
        run.bodies[CUE].position = vec3(HALF_LONG - 4.0, table::BALL_RADIUS, HALF_WIDE - 4.0);
        run.bodies[1].position = vec3(HALF_LONG - 3.0, table::BALL_RADIUS, HALF_WIDE - 3.0);
        run.in_hand = false;

        // struck high, so the cue ball runs on through the one it hits and
        // follows it down. A flat hit stops it dead on the contact, which is
        // the stun shot and the whole reason spec 0032 exists.
        run.shoot(vec3(1.0, 0.0, 1.0), shot::HARDEST, vec2(0.0, 0.9));
        settle(&mut run);

        assert_eq!(run.last, Some(Outcome::Foul(Foul::Scratched)));
        assert_eq!(
            run.cue(),
            table::head_spot(),
            "it did not come back in hand"
        );
        assert!(run.in_hand);
    }

    #[test]
    fn no_rail_after_contact_is_a_foul() {
        // the two of them in the middle of the table, tapped: they touch, they
        // stop, and nothing reaches a cushion
        let mut run = only(&[1]);
        lined_up(&mut run, 1, 2.0);
        run.bodies[CUE].position = vec3(0.0, table::BALL_RADIUS, 0.0);
        run.bodies[1].position = vec3(2.0, table::BALL_RADIUS, 0.0);

        run.shoot(Vec3::X, 2.0, vec2(0.0, 0.0));
        settle(&mut run);

        assert_eq!(run.last, Some(Outcome::Foul(Foul::NoRail)));
    }

    #[test]
    fn a_foul_costs_a_shot_and_the_cue_ball() {
        let mut run = only(&[1]);
        run.bodies[CUE].position = vec3(0.0, table::BALL_RADIUS, 0.0);
        run.bodies[1].position = vec3(0.0, table::BALL_RADIUS, HALF_WIDE * 0.6);
        run.in_hand = false;

        run.shoot(Vec3::X, shot::HARDEST * 0.3, vec2(0.0, 0.0));
        settle(&mut run);

        assert_eq!(run.shots(), 1, "a foul is still a shot");
        assert_eq!(run.fouls(), 1);
        assert!(run.in_hand, "the cue ball is not in hand");

        // and in hand means it can be put somewhere
        let somewhere = vec3(-5.0, table::BALL_RADIUS, 3.0);
        run.place(somewhere);
        assert_eq!(run.cue(), somewhere);
    }

    #[test]
    fn putting_it_down_ends_being_in_hand() {
        let mut run = only(&[1]);
        assert!(run.in_hand, "the break is from hand");

        run.place(vec3(-5.0, table::BALL_RADIUS, 2.0));

        assert!(!run.in_hand, "it is still in hand after being put down");

        // and a second click does not pick it up again
        let down = run.cue();
        run.place(vec3(1.0, table::BALL_RADIUS, 1.0));

        assert_eq!(run.cue(), down, "a later click moved it");
    }

    #[test]
    fn the_cue_ball_cannot_be_moved_unless_it_is_in_hand() {
        let mut run = only(&[1]);
        lined_up(&mut run, 1, 6.0);
        let was = run.cue();

        run.place(vec3(3.0, table::BALL_RADIUS, 3.0));

        assert_eq!(run.cue(), was);
    }

    #[test]
    fn an_empty_table_ends_it() {
        let mut run = only(&[9]);
        // the last ball in the jaws, the cue ball behind it
        run.bodies[9].position = vec3(HALF_LONG - 3.0, table::BALL_RADIUS, HALF_WIDE - 3.0);
        run.bodies[CUE].position = vec3(HALF_LONG - 7.0, table::BALL_RADIUS, HALF_WIDE - 7.0);
        run.in_hand = false;
        for ball in 1..NINE {
            run.down[ball] = true;
        }

        run.shoot(vec3(1.0, 0.0, 1.0), shot::HARDEST * 0.5, vec2(0.0, 0.0));
        settle(&mut run);

        assert_eq!(run.left(), 0, "something is still up: {:?}", run.last);
        assert_eq!(run.phase(), Phase::Over);
    }

    #[test]
    fn a_table_with_one_ball_left_is_not_over() {
        let mut run = only(&[1, 2]);
        lined_up(&mut run, 1, 6.0);

        run.shoot(Vec3::X, shot::HARDEST, vec2(0.0, 0.0));
        settle(&mut run);

        assert!(run.left() > 0);
        assert_eq!(run.phase(), Phase::Aiming);
    }

    #[test]
    fn a_potted_ball_is_out_of_play() {
        let mut run = only(&[1, 2]);
        run.down[2] = true;
        let resting = vec3(4.0, table::BALL_RADIUS, 4.0);
        run.bodies[2].position = resting;
        lined_up(&mut run, 1, 6.0);

        run.shoot(Vec3::X, shot::HARDEST, vec2(0.0, 0.0));
        settle(&mut run);

        assert_eq!(run.bodies[2].position, resting, "a potted ball was moved");
    }

    #[test]
    fn no_shot_runs_for_ever() {
        // every aim off the break, at everything there is
        for turn in 0..16 {
            let about = turn as f32 / 16.0 * std::f32::consts::TAU;
            let way = vec3(about.sin(), 0.0, about.cos());

            let mut run = Run::new();
            run.shoot(way, shot::HARDEST, vec2(0.0, 0.0));

            let taken = settle(&mut run);

            assert!(taken < shot::LONGEST, "a break aimed {:?} never ended", way);
        }
    }

    #[test]
    fn nothing_in_play_leaves_the_table() {
        // a ball goes off the table through a pocket, which is the point of a
        // pocket. What must not happen is one still in play falling off the
        // world, which is what carom did for twenty seconds a shot.
        for turn in 0..16 {
            let about = turn as f32 / 16.0 * std::f32::consts::TAU;
            let way = vec3(about.sin(), 0.0, about.cos());

            let mut run = Run::new();
            run.shoot(way, shot::HARDEST, vec2(0.0, 0.0));
            settle(&mut run);

            for ball in 0..=BALLS {
                if run.is_down(ball) {
                    continue;
                }
                let at = run.bodies[ball].position;
                assert!(
                    at.y > 0.0 && at.x.abs() < HALF_LONG + 4.0 && at.z.abs() < HALF_WIDE + 4.0,
                    "ball {} in play is at {:?} after a break aimed {:?}",
                    ball,
                    at,
                    way
                );
            }
        }
    }
}
