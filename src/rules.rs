//! A run: the balls, whose turn it would be if there were two of you, what is
//! legal and what is a foul. See `specs/0001-the-rack.md`.

use blitzkit::collision::{sweep_sphere, Sphere};
use blitzkit::physics::{step, Body};
use glam::{Quat, Vec3};

use crate::shot::{self, ball};
use crate::table::{self, BALLS, EIGHT};

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
    /// Struck the eight first while anything else was still up.
    EightFirst,
}

/// What the shot now rolling has done so far.
#[derive(Debug, Clone, Default)]
struct Happenings {
    /// Whether the eight was the only ball left when this shot was taken.
    ///
    /// Taken at the shot and not at the end of it, because potting the last
    /// other ball in the same shot does not make the eight legal to have hit
    /// first. What matters is what was on the table when you struck it.
    eight_alone: bool,
    first_hit: Option<usize>,
    reached_a_rail: bool,
    potted: Vec<usize>,
}

pub struct Run {
    pub bodies: Vec<Body>,
    /// Which way round each ball has got to, which is its spin added up. Only
    /// drawing reads it, and only a striped ball shows it. See spec 0002.
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

    /// Which way a ball is facing. The engine turns a body by its spin as of
    /// spec 0034, so this reads what it did rather than repeating it here.
    pub fn facing(&self, ball: usize) -> Quat {
        self.bodies[ball].orientation
    }

    pub fn cue(&self) -> Vec3 {
        self.bodies[CUE].position
    }

    /// Whether the eight is the only numbered ball still up.
    ///
    /// Which is when it stops being the ball you must not touch and becomes
    /// the ball you have to.
    pub fn only_the_eight_is_up(&self) -> bool {
        (1..=BALLS).all(|ball| ball == EIGHT || self.down[ball])
    }

    /// Whether the cue ball could be put down here: on the table, and not
    /// inside a ball that is already on it.
    pub fn is_free(&self, at: Vec3) -> bool {
        if !table::is_on_the_table(at) {
            return false;
        }

        (1..=BALLS)
            .filter(|ball| !self.down[*ball])
            .all(|ball| self.bodies[ball].position.distance(at) > table::BALL_RADIUS * 2.0)
    }

    /// How many balls are still on the table.
    ///
    /// Any of them may be hit first and any may be potted: this spec had the
    /// lowest ball first, which is nine ball's rule, and Jake took it out. What
    /// is left is potting nine balls in as few shots as you can.
    pub fn left(&self) -> usize {
        (1..=BALLS).filter(|ball| !self.down[*ball]).count()
    }

    /// Whether the cue ball may be put here.
    ///
    /// Free of the other balls and on the table, and behind the head string if
    /// nothing has been struck yet. You break from the kitchen; after that a
    /// scratch is ball in hand and the whole table is yours.
    pub fn may_place(&self, at: Vec3) -> bool {
        self.is_free(at) && (self.shots > 0 || table::in_the_kitchen(at))
    }

    /// Whether the placing now on offer is kitchen only, which is the break
    /// and nothing else.
    pub fn kitchen_only(&self) -> bool {
        self.in_hand && self.shots == 0
    }

    /// Slides the cue ball along the cloth while it is being carried.
    ///
    /// Unlike `place`, this does not put it down: it is the ball following
    /// your hand, and it stays in hand until you let go. A spot it may not
    /// have leaves it where it was, so the ball sticks at the edge of what is
    /// allowed rather than vanishing or following you off the table.
    pub fn carry_to(&mut self, at: Vec3) {
        if !self.in_hand || self.phase != Phase::Aiming || !self.may_place(at) {
            return;
        }

        self.bodies[CUE].position = at;
        self.bodies[CUE].velocity = Vec3::ZERO;
        self.bodies[CUE].spin = Vec3::ZERO;
    }

    /// Puts the cue ball somewhere, which is only allowed in hand.
    pub fn place(&mut self, at: Vec3) {
        if !self.in_hand || self.phase != Phase::Aiming || !self.may_place(at) {
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
        self.doing = Happenings {
            eight_alone: self.only_the_eight_is_up(),
            ..Happenings::default()
        };
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

        // and the cloth takes the turn out of whatever is no longer going
        // anywhere, per spec 0002. The engine spends a spin by rolling the ball
        // along, which a ball wedged in a cluster cannot do.
        for (n, at) in playing.iter().enumerate() {
            let went = (self.bodies[*at].position - before[n].0).length();
            // what it did, not what it meant to do. A ball shouldering into its
            // neighbours keeps a velocity it cannot spend, so asking the
            // velocity says it is travelling while the table says it has not
            // moved in half a second.
            if went >= shot::STILL * dt {
                continue;
            }

            // and the speed with it, because the two feed each other: a ball
            // shouldering its neighbours keeps a velocity it cannot spend, the
            // contact turns that back into spin every step, and the spin is
            // replenished as fast as the cloth takes it away.
            let body = &mut self.bodies[*at];
            let slowing = 1.0 + shot::SETTLES * dt;
            body.spin /= slowing;
            body.velocity.x /= slowing;
            body.velocity.z /= slowing;
            if body.spin.length() < shot::STILL {
                body.spin = Vec3::ZERO;
            }
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
            // either the ball swept into a cushion this step, or it is sitting
            // against one now having not been a step ago. The second is the one
            // that catches a ball arriving slowly, which is most of them.
            self.doing.reached_a_rail = playing.iter().zip(before).any(|(ball, (was, going))| {
                self.met_a_rail(*was, *going, dt)
                    || (Self::on_a_rail(self.bodies[*ball].position) && !Self::on_a_rail(*was))
            });
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

    /// Whether a ball is against a cushion where it stands.
    ///
    /// Asked of where the ball ended up, rather than swept along where it was
    /// going. The sweep is the path the ball meant to take over one step and
    /// the engine resolves the contact inside that step, so the two disagree by
    /// a hair and the hair is the whole answer: a ball came up to a cushion at
    /// three a second, the sweep reached two thousandths short of touching it,
    /// and by the next step the engine had already turned the ball so the path
    /// no longer pointed at the rail. Nothing ever met a rail and every shot
    /// was a foul, the break included.
    ///
    /// The engine knows about the contact. This asks what happened instead of
    /// working out again what should have.
    fn on_a_rail(at: Vec3) -> bool {
        let reach = table::BALL_RADIUS + table::RAIL_SLACK;

        table::cushions().iter().any(|rail| {
            let near = at.clamp(rail.min, rail.max);

            at.distance_squared(near) <= reach * reach
        })
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
                Some(EIGHT) if !self.doing.eight_alone => Some(Foul::EightFirst),
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
    use crate::table::EIGHT;
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

    /// Spec 0002: and no ball turns on the spot. A pool ball that has stopped
    /// travelling has stopped turning, and one that has not is a thing a table
    /// does not do.
    ///
    /// Only through `Run`, which is why this lives here and not in `shot`. A
    /// lone ball given spin and no speed converts it to a roll and is done
    /// inside two seconds, so measured against the solver on its own there is
    /// nothing to find. It takes a crowd: a ball shouldered by its neighbours
    /// has nowhere to roll to, keeps a velocity it cannot spend, and the
    /// contact turns that back into spin every step. Before the cloth was given
    /// the job of taking it away, two balls off an ordinary break sat turning
    /// for most of a second.
    #[test]
    fn nothing_turns_on_the_spot() {
        let mut run = Run::new();
        run.shoot(Vec3::X, shot::HARDEST, glam::vec2(0.0, 0.0));

        // a fifth of a second: at the rates this happens at, a couple of
        // degrees, which is under what an eye picks out
        let most = (0.2 / shot::STEP) as usize;
        let mut spot = [0usize; BALLS + 1];

        for _ in 0..shot::LONGEST {
            let before: Vec<Vec3> = (0..=BALLS).map(|ball| run.bodies[ball].position).collect();
            let facing: Vec<Quat> = (0..=BALLS).map(|ball| run.facing(ball)).collect();
            run.step(shot::STEP);

            for ball in 0..=BALLS {
                if run.is_down(ball) {
                    continue;
                }

                let moved = (run.bodies[ball].position - before[ball]).length();
                let turned = run.facing(ball).angle_between(facing[ball]);
                // a tenth of a degree a step is six degrees a second, which you
                // see on a ball with a number on it
                if turned > 0.0017 && moved < 0.0005 {
                    spot[ball] += 1;
                } else {
                    spot[ball] = 0;
                }

                assert!(
                    spot[ball] <= most,
                    "ball {} turned on the spot for {:.2}s",
                    ball,
                    spot[ball] as f32 * shot::STEP
                );
            }

            if run.phase() != Phase::Rolling {
                return;
            }
        }

        panic!("the break never settled");
    }

    /// Spec 0001: and a break is not a foul.
    ///
    /// The one that was wrong. A ball arriving at a cushion slowly is never
    /// caught by the sweep: it came up at three a second, the sweep reached two
    /// thousandths short of touching, and by the next step the engine had
    /// already turned it so the path no longer pointed at the rail. Nothing
    /// ever met a rail, so every shot in the game was a foul and the cue ball
    /// came back to hand after every one of them, the break included.
    #[test]
    fn a_break_is_not_a_foul() {
        let mut run = Run::new();
        run.place(table::head_spot());
        run.shoot(Vec3::X, shot::HARDEST, glam::vec2(0.0, 0.0));
        settle(&mut run);

        assert!(
            !matches!(run.last, Some(Outcome::Foul(Foul::NoRail))),
            "a full break was called no rail"
        );
        assert!(!run.in_hand, "a full break put the cue ball back in hand");
    }

    /// Spec 0001: hitting the eight first is a foul, until it is all there is.
    #[test]
    fn hitting_the_eight_first_is_a_foul() {
        let mut run = only(&[1, EIGHT]);
        // the eight square in front of the cue ball, the one off to the side
        run.bodies[EIGHT].position = vec3(0.0, table::BALL_RADIUS, 0.0);
        run.bodies[1].position = vec3(0.0, table::BALL_RADIUS, 8.0);
        run.place(vec3(-16.0, table::BALL_RADIUS, 0.0));

        run.shoot(Vec3::X, shot::HARDEST, glam::vec2(0.0, 0.0));
        settle(&mut run);

        assert_eq!(
            run.last,
            Some(Outcome::Foul(Foul::EightFirst)),
            "hitting the eight first was allowed"
        );
        assert!(run.in_hand, "the foul did not give the ball back");
    }

    /// Spec 0001: and once it is the only ball up it is the ball to hit.
    #[test]
    fn the_eight_alone_is_the_ball_to_hit() {
        let mut run = only(&[EIGHT]);
        run.bodies[EIGHT].position = vec3(0.0, table::BALL_RADIUS, 0.0);
        run.place(vec3(-16.0, table::BALL_RADIUS, 0.0));
        assert!(run.only_the_eight_is_up(), "something else is still up");

        run.shoot(Vec3::X, shot::HARDEST, glam::vec2(0.0, 0.0));
        settle(&mut run);

        assert_ne!(
            run.last,
            Some(Outcome::Foul(Foul::EightFirst)),
            "the eight was refused when it was the only ball left"
        );
    }

    /// Spec 0001: and potting the last other ball during the same shot does
    /// not excuse it. What counts is the table you struck the eight from.
    ///
    /// Asked of the record rather than staged as a trick shot: lining up a
    /// cue ball, the eight and a pocketable one took longer to arrange than
    /// the rule took to write, and a shot that misses proves nothing either
    /// way. What this needs to know is when the question is asked.
    #[test]
    fn clearing_up_during_the_same_shot_does_not_excuse_it() {
        let mut run = only(&[1, EIGHT]);
        run.bodies[EIGHT].position = vec3(0.0, table::BALL_RADIUS, 0.0);
        run.bodies[1].position = vec3(0.0, table::BALL_RADIUS, 8.0);
        run.place(vec3(-16.0, table::BALL_RADIUS, 0.0));

        run.shoot(Vec3::X, shot::HARDEST, glam::vec2(0.0, 0.0));
        assert!(
            !run.doing.eight_alone,
            "the eight was alone with the one still up"
        );

        // the one goes down while the shot is still rolling, which is the case
        // this is about
        run.down[1] = true;
        assert!(run.only_the_eight_is_up(), "the one is still up");

        settle(&mut run);

        assert_eq!(
            run.last,
            Some(Outcome::Foul(Foul::EightFirst)),
            "clearing the table during the shot excused hitting the eight first"
        );
    }

    /// Spec 0001: a soft shot still reaches a ball across the table.
    ///
    /// Rolling resistance is a steady slowing, so distance goes as the square
    /// of speed, and charging straight into a speed put nearly all the useful
    /// range at the top of the wind-up. A third of a wind-up fouled on every
    /// one of a hundred and sixty shots, almost all of them for never touching
    /// anything at all.
    #[test]
    fn a_third_of_a_wind_up_still_reaches_a_ball() {
        let mut run = only(&[1]);
        run.bodies[1].position = vec3(0.0, table::BALL_RADIUS, 0.0);
        run.place(vec3(-16.0, table::BALL_RADIUS, 0.0));

        // sixteen units, which is a third of the table and an ordinary length
        // of shot
        run.shoot(
            Vec3::X,
            shot::struck(shot::HARDEST / 3.0),
            glam::vec2(0.0, 0.0),
        );
        settle(&mut run);

        assert!(
            !matches!(run.last, Some(Outcome::Foul(Foul::Missed))),
            "a third of a wind-up could not reach a ball sixteen away"
        );
    }

    /// A run with everything cleared off the table but the balls named.
    fn only(balls: &[usize]) -> Run {
        let mut run = Run::new();
        for ball in 1..=BALLS {
            if balls.contains(&ball) {
                continue;
            }
            if ball == EIGHT {
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
    fn a_new_rack_has_not_turned() {
        let run = Run::new();

        for ball in 0..=BALLS {
            assert_eq!(run.facing(ball), Quat::IDENTITY);
        }
    }

    #[test]
    fn a_rolling_ball_turns() {
        let mut run = only(&[1]);
        lined_up(&mut run, 1, 6.0);

        run.shoot(Vec3::X, shot::HARDEST, vec2(0.0, 0.0));
        settle(&mut run);

        assert_ne!(run.facing(CUE), Quat::IDENTITY, "the cue ball never turned");
        assert_ne!(run.facing(1), Quat::IDENTITY, "the one never turned");
    }

    #[test]
    fn a_ball_turns_the_way_it_rolls() {
        // rolling along +x turns it about -z, which is what a ball not
        // slipping does. Spec 0030 spins it through friction and nothing else.
        let mut run = only(&[1]);
        lined_up(&mut run, 1, 20.0);

        run.shoot(Vec3::X, shot::HARDEST * 0.4, vec2(0.0, 0.0));
        for _ in 0..60 {
            run.step(shot::STEP);
        }

        let (axis, angle) = run.facing(CUE).to_axis_angle();
        let way = axis * angle;

        assert!(way.z < 0.0, "it turned about {:?}", way);
        assert!(way.x.abs() < way.z.abs(), "it turned sideways: {:?}", way);
    }

    #[test]
    fn a_still_ball_does_not_turn() {
        let mut run = only(&[1]);
        lined_up(&mut run, 1, 6.0);
        // the fifteen is parked and nothing goes near it
        let was = run.facing(BALLS);

        run.shoot(Vec3::X, shot::HARDEST * 0.3, vec2(0.0, 0.0));
        settle(&mut run);

        assert_eq!(run.facing(BALLS), was);
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

        run.place(vec3(-15.0, table::BALL_RADIUS, 2.0));

        assert!(!run.in_hand, "it is still in hand after being put down");

        // and a second click does not pick it up again
        let down = run.cue();
        run.place(vec3(-14.0, table::BALL_RADIUS, 1.0));

        assert_eq!(run.cue(), down, "a later click moved it");
    }

    #[test]
    fn the_cue_ball_cannot_be_put_down_off_the_table() {
        // the cursor ray meets the cloth's plane wherever it is pointed, and
        // that plane goes on for ever
        let mut run = Run::new();
        let on_the_spot = run.cue();

        run.place(vec3(HALF_LONG + 20.0, table::BALL_RADIUS, 0.0));
        assert_eq!(run.cue(), on_the_spot, "it went down past the foot rail");

        run.place(vec3(0.0, table::BALL_RADIUS, HALF_WIDE + 20.0));
        assert_eq!(run.cue(), on_the_spot, "it went down past the side rail");

        assert!(run.in_hand, "a refused placing ended being in hand");
    }

    #[test]
    fn the_cue_ball_cannot_be_put_down_inside_another() {
        let mut run = only(&[1]);
        let on_the_spot = run.cue();
        // in the kitchen, because that is where the break is placed from and
        // the rack is the far side of the line
        let one = vec3(-16.0, table::BALL_RADIUS, 0.0);
        run.bodies[1].position = one;

        run.place(one);
        assert_eq!(run.cue(), on_the_spot, "it went down inside the one");

        // but touching it is close enough
        let beside = one - Vec3::X * table::BALL_RADIUS * 2.2;
        run.place(beside);
        assert_eq!(run.cue(), beside);
    }

    /// Spec 0001: the break is from the kitchen, and a scratch is not.
    #[test]
    fn the_break_is_from_behind_the_head_string() {
        let mut run = Run::new();
        assert!(run.kitchen_only(), "the break is not from the kitchen");

        let up_table = vec3(0.0, table::BALL_RADIUS, 0.0);
        assert!(!run.may_place(up_table), "the break may be placed up table");
        run.place(up_table);
        assert!(run.in_hand, "an illegal placing put the ball down anyway");

        let behind = vec3(-15.0, table::BALL_RADIUS, 2.0);
        assert!(run.may_place(behind), "the kitchen is refused");
        run.place(behind);
        assert_eq!(run.cue(), behind);

        // and after a scratch the whole table is yours. The object balls go
        // away first, so this asks about the head string and not about which
        // bit of cloth a broken rack happens to be sitting on.
        run.shoot(Vec3::X, shot::HARDEST, glam::vec2(0.0, 0.0));
        settle(&mut run);
        for ball in 1..=BALLS {
            run.down[ball] = true;
        }
        run.in_hand = true;

        assert!(!run.kitchen_only(), "a scratch is still kitchen only");
        assert!(
            run.may_place(vec3(table::HALF_LONG * 0.5, table::BALL_RADIUS, 0.0)),
            "ball in hand is still refused up table"
        );
    }

    /// Spec 0001: and carrying it does not put it down.
    #[test]
    fn carrying_the_cue_ball_keeps_it_in_hand() {
        let mut run = Run::new();
        let behind = vec3(-15.0, table::BALL_RADIUS, 2.0);

        run.carry_to(behind);
        assert_eq!(run.cue(), behind, "it did not follow");
        assert!(run.in_hand, "carrying it put it down");

        // and a spot it may not have leaves it where it was
        run.carry_to(vec3(0.0, table::BALL_RADIUS, 0.0));
        assert_eq!(run.cue(), behind, "it followed past the head string");

        run.place(run.cue());
        assert!(!run.in_hand, "letting go did not put it down");
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
        let mut run = only(&[BALLS]);
        // the last ball in the jaws, the cue ball behind it
        run.bodies[BALLS].position = vec3(HALF_LONG - 3.0, table::BALL_RADIUS, HALF_WIDE - 3.0);
        run.bodies[CUE].position = vec3(HALF_LONG - 7.0, table::BALL_RADIUS, HALF_WIDE - 7.0);
        run.in_hand = false;
        for ball in 1..BALLS {
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
