//! The window: the table, the rack, aiming, and where on the ball you hit it.
//! See `specs/0001-the-rack.md`.

use blitzkit::camera::Camera;
use blitzkit::geometry::Geometry;
use blitzkit::keyboard::{KeyboardInput, KeyboardKey, KeyboardKeyState};
use blitzkit::mesh::{MeshData, Transform};
use blitzkit::mouse::{MouseButton, MouseInput};
use blitzkit::notice;
use blitzkit::renderer::render_text::{RenderText, TextRenderer};
use blitzkit::renderer::scene::{MeshId, Scene, TextureId};
use blitzkit::renderer::Renderer;
use blitzkit::sound::SoundSystem;
use blitzkit::Game;
use glam::{vec2, vec3, vec4, Vec2, Vec3, Vec4};

use crate::paint;
use crate::rules::{Foul, Outcome, Phase, Run, CUE};
use crate::shot::{self, HARDEST, SOFTEST};
use crate::table::{self, BALLS, BALL_RADIUS, HALF_LONG, HALF_WIDE, POCKET_MOUTH};
use crate::view::View;

const HUD_LEFT: f32 = 20.0;
const HUD_TOP: f32 = 20.0;
const HUD_SIZE: f32 = 20.0;
const HUD_APART: f32 = HUD_SIZE * 1.6;

/// How long holding the button takes to reach the hardest shot.
const TO_FULL: f32 = 1.2;

/// How fast the arrow keys move the tip around the face of the cue ball, in
/// radii a second, and how far off the middle they may take it.
const TIP_PER_SECOND: f32 = 1.4;
const TIP_LIMIT: f32 = 0.9;

const CLOTH: Vec4 = vec4(0.13, 0.36, 0.26, 1.0);

/// How tight the cloth's highlight is.
///
/// Very, which is how you say matte with the one number there is. The engine
/// gives a shininess and no specular strength, and the term is the angle raised
/// to that power: a small number is a wide highlight, not a weak one. At the
/// default of 32 the sun laid a white wash across the middle of a surface fifty
/// units across and the cloth read as wet vinyl. Baize scatters.
const CLOTH_SHEEN: f32 = 320.0;
const RAIL: Vec4 = vec4(0.28, 0.17, 0.11, 1.0);
const POCKET: Vec4 = vec4(0.03, 0.04, 0.05, 1.0);
const CUE_BALL: Vec4 = vec4(0.97, 0.96, 0.92, 1.0);
const AIM: Vec4 = vec4(1.0, 0.95, 0.75, 1.0);

/// The head string, drawn while the break is being placed.
///
/// Chalk on baize rather than paint: a real table marks it with two diamonds
/// and nothing across the cloth, but two diamonds are no help at all when the
/// question is whether the ball under your hand is behind the line.
const LINE: Vec4 = vec4(0.72, 0.78, 0.74, 1.0);

/// What each numbered ball is painted, one through fifteen.
///
/// Seven hues, the eight in black, and the same seven again paler for the
/// stripes. The engine can texture a sphere, which is how marble wears its
/// checker, so real stripes are a change this can take later.
const PAINT: [Vec4; BALLS] = [
    vec4(0.95, 0.80, 0.15, 1.0),
    vec4(0.15, 0.32, 0.80, 1.0),
    vec4(0.82, 0.16, 0.14, 1.0),
    vec4(0.45, 0.20, 0.62, 1.0),
    vec4(0.95, 0.52, 0.12, 1.0),
    vec4(0.12, 0.58, 0.32, 1.0),
    vec4(0.52, 0.14, 0.16, 1.0),
    vec4(0.08, 0.08, 0.10, 1.0),
    vec4(0.99, 0.92, 0.62, 1.0),
    vec4(0.62, 0.72, 0.95, 1.0),
    vec4(0.95, 0.63, 0.62, 1.0),
    vec4(0.78, 0.65, 0.88, 1.0),
    vec4(0.99, 0.78, 0.58, 1.0),
    vec4(0.62, 0.86, 0.70, 1.0),
    vec4(0.84, 0.60, 0.61, 1.0),
];

pub struct PoolhallGame {
    run: Run,
    /// Where the cursor is, in pixels, while it is over the window.
    pointing: Option<Vec2>,
    /// Where on the table it is pointing, if anywhere.
    aimed_at: Option<Vec3>,
    /// Which way the shot goes. Kept rather than worked out fresh, because the
    /// cursor can sit on top of the cue ball, and the direction from a ball to
    /// itself is nothing at all. Putting the ball down under the cursor does
    /// exactly that, so a shot taken straight afterwards was refused in
    /// silence.
    aim: Vec3,
    /// Where on the cue ball's face the tip goes, in radii.
    tip: Vec2,
    /// Up, down, left, right, held. The cue tip, with shift down.
    nudging: [bool; 4],
    /// W, S, A, D held: round the table and in and out.
    walking_keys: [bool; 4],
    power: f32,
    charging: bool,
    /// Where the player is standing, per spec 0003.
    view: View,
    /// Whether the cue ball is in the hand and being moved, which is the left
    /// button held down after taking hold of it.
    carrying: bool,
    /// Whether shift is held, which gives each key its second job: the arrows
    /// put the cue tip instead of sliding, and W and S stand you up instead of
    /// leaning you in.
    sliding: bool,
    sphere: Option<MeshId>,
    block: Option<MeshId>,
    /// One band per striped ball, in ball order from nine upwards.
    /// One per ball, in order, each carrying that ball's colour and number.
    faces: Vec<TextureId>,
    quitting: bool,
    /// Whether this run is only here to be photographed, and whether the break
    /// has been taken. See `refresh-screenshots` in the project above.
    ///
    /// The opening frame is a rack and a cue ball, which is a photograph of a
    /// table nobody has played on. The break is what a pool table looks like.
    staged: bool,
    broken: bool,
}

impl Default for PoolhallGame {
    fn default() -> Self {
        Self::new()
    }
}

impl PoolhallGame {
    pub fn new() -> Self {
        Self {
            run: Run::new(),
            pointing: None,
            aimed_at: None,
            aim: Vec3::X,
            tip: Vec2::ZERO,
            nudging: [false; 4],
            walking_keys: [false; 4],
            power: 0.0,
            charging: false,
            view: View::new(),
            carrying: false,
            sliding: false,
            sphere: None,
            block: None,
            faces: Vec::new(),
            quitting: false,
            staged: crate::staged(),
            broken: false,
        }
    }

    /// Which way the shot goes: from the cue ball towards what is pointed at.
    pub fn way(&self) -> Vec3 {
        self.aim
    }

    /// The face this ball wears: its colour, its band if it has one, and its
    /// number.
    fn face(&self, ball: usize) -> Option<TextureId> {
        self.faces.get(ball - 1).copied()
    }

    /// Whether the cursor is over the cue ball, which is what you take hold of.
    fn on_the_cue_ball(&self) -> bool {
        let Some(at) = self.aimed_at else {
            return false;
        };
        let cue = self.run.cue();

        // a little wider than the ball, because a ball is a small thing to hit
        // with a cursor and missing it does nothing at all
        (vec3(at.x, cue.y, at.z) - cue).length() <= BALL_RADIUS * 1.6
    }

    /// The hue a ball is painted. A striped one takes the one seven below it,
    /// which is how a real set is made, and wears it as a band rather than all
    /// over.
    fn hue(ball: usize) -> [u8; 3] {
        let paint = if paint::is_striped(ball) {
            PAINT[paint::partner(ball)]
        } else {
            PAINT[ball - 1]
        };

        [
            (paint.x * 255.0) as u8,
            (paint.y * 255.0) as u8,
            (paint.z * 255.0) as u8,
        ]
    }

    fn line(&self, n: usize) -> Vec2 {
        vec2(HUD_LEFT, HUD_TOP + n as f32 * HUD_APART)
    }

    /// How hard the staged break is and how long it is given to settle.
    const POSED_SETTLES_FOR: f32 = 12.0;

    /// Breaks the rack for the camera and lets it come to rest. See
    /// `refresh-screenshots`.
    ///
    /// On the first frame rather than over twelve real seconds, because the
    /// shutter is on a timer and will not wait for fifteen balls to stop.
    fn pose(&mut self) {
        self.broken = true;

        // up the table into the apex, a little off square. Measured over power
        // and angle: dead on leaves the readout saying "foul: nothing reached
        // a cushion", and at six hundredths the cue ball goes down. Full power
        // at four hundredths breaks clean and fouls nothing.
        let cue = self.run.cue();
        let apex = Vec3::new(0.0, cue.y, 0.0);
        let way = (apex - cue).normalize_or_zero();
        let across = Vec3::new(way.z, 0.0, -way.x);
        self.run.shoot(
            (way + across * 0.04).normalize(),
            shot::HARDEST,
            glam::Vec2::ZERO,
        );

        let mut at = 0.0;
        while at < Self::POSED_SETTLES_FOR {
            self.run.step(shot::STEP);
            at += shot::STEP;
        }
    }

    /// The readout, on a panel. White on a lit table is white on whatever the
    /// table happens to be showing, so the lines get something to sit on.
    /// See blitzkit's spec 0038.
    fn say(&self, geometry: &mut Geometry, text_renderer: &mut TextRenderer) {
        let lines: Vec<RenderText> = self
            .readout()
            .into_iter()
            .enumerate()
            .map(|(n, text)| RenderText {
                position: self.line(n),
                color: vec4(1.0, 1.0, 1.0, 0.9),
                size: HUD_SIZE,
                text,
                ..Default::default()
            })
            .collect();

        // nothing else here draws in 2D, and the engine does not clear this
        // between frames
        geometry.reset();
        if let Some(frame) = notice::framing_all(&lines) {
            for quad in frame.iter() {
                geometry.push_quad(quad);
            }
        }

        for line in lines {
            text_renderer.push_render_text(line);
        }
    }

    fn readout(&self) -> Vec<String> {
        if self.run.phase() == Phase::Over {
            return vec![
                format!("the table is clear in {}", shots(self.run.shots())),
                format!("{} along the way", fouls(self.run.fouls())),
                String::from("press r to rack them again"),
            ];
        }

        let left = match self.run.left() {
            1 => String::from("one ball left"),
            left => format!("{} balls left", left),
        };

        vec![
            format!("{}, {}", left, shots(self.run.shots())),
            match self.run.last {
                Some(Outcome::Foul(why)) => said(why).to_string(),
                _ if self.run.kitchen_only() => {
                    String::from("drag the cue ball behind the line to break")
                }
                _ if self.run.in_hand => String::from("ball in hand: drag the cue ball anywhere"),
                _ => String::from("point and hold to shoot. wasd walks, arrows slide"),
            },
            format!(
                "shift and the arrows put the tip at {:+.1} across, {:+.1} up",
                self.tip.x, self.tip.y
            ),
        ]
    }
}

/// "1 shot" and "2 shots".
fn shots(taken: u32) -> String {
    if taken == 1 {
        String::from("1 shot")
    } else {
        format!("{} shots", taken)
    }
}

fn fouls(taken: u32) -> String {
    match taken {
        0 => String::from("no fouls"),
        1 => String::from("one foul"),
        _ => format!("{} fouls", taken),
    }
}

/// What to say about a foul.
fn said(why: Foul) -> &'static str {
    match why {
        Foul::Missed => "foul: you hit nothing",
        Foul::Scratched => "foul: the cue ball went down",
        Foul::NoRail => "foul: nothing reached a cushion",
    }
}

/// Where a cursor ray meets the cloth the balls sit on.
fn on_the_cloth(camera: &Camera, cursor: Vec2) -> Option<Vec3> {
    let ray = camera.ray_through(cursor);

    if ray.direction.y.abs() < 1e-6 {
        return None;
    }

    let along = (BALL_RADIUS - ray.origin.y) / ray.direction.y;

    (along > 0.0).then(|| ray.at(along))
}

impl Game for PoolhallGame {
    fn initialize(
        &mut self,
        _geometry: &mut Geometry,
        _text_renderer: &mut TextRenderer,
        _sound_system: &SoundSystem,
        _window_size: (f32, f32),
    ) {
    }

    fn load(&mut self, renderer: &mut Renderer) {
        self.sphere = Some(renderer.add_mesh(&MeshData::sphere(24, 16)));
        self.block = Some(renderer.add_mesh(&MeshData::cube()));

        // every ball and not only the striped ones, because every ball carries
        // its number and a number is a texture
        self.faces = (1..=BALLS)
            .map(|ball| renderer.add_texture(&paint::ball(ball, Self::hue(ball))))
            .collect();
    }

    fn update(
        &mut self,
        dt: f32,
        geometry: &mut Geometry,
        text_renderer: &mut TextRenderer,
        _sound_system: &SoundSystem,
    ) {
        text_renderer.reset();

        if self.staged {
            if !self.broken {
                self.pose();
            }
            self.say(geometry, text_renderer);
            return;
        }

        // the feet: W and S lean in and out, A and D walk round
        let round = (self.walking_keys[3] as i32 - self.walking_keys[2] as i32) as f32;
        let inout = (self.walking_keys[0] as i32 - self.walking_keys[1] as i32) as f32;
        if self.sliding {
            // shift turns the feet into standing up and stooping
            self.view.stooped(inout, dt);
        } else if round != 0.0 || inout != 0.0 {
            self.view.walked(round, inout, dt);
        }

        let across = (self.nudging[3] as i32 - self.nudging[2] as i32) as f32;
        let up = (self.nudging[0] as i32 - self.nudging[1] as i32) as f32;

        if self.sliding {
            // the cue tip, which is the fine control and so wants the modifier
            self.tip = vec2(
                (self.tip.x + across * TIP_PER_SECOND * dt).clamp(-TIP_LIMIT, TIP_LIMIT),
                (self.tip.y + up * TIP_PER_SECOND * dt).clamp(-TIP_LIMIT, TIP_LIMIT),
            );
        } else if across != 0.0 || up != 0.0 {
            // and the head: the arrows slide what you are looking at, the way
            // they look around in the arcade
            let half = glam::vec2(
                table::HALF_LONG + table::APRON,
                table::HALF_WIDE + table::APRON,
            );

            self.view.slid(across, up, dt, half);
        }

        if self.charging {
            self.power = (self.power + HARDEST / TO_FULL * dt).min(HARDEST);
        }

        if self.run.phase() == Phase::Rolling {
            let mut left = dt;
            while left > 0.0 {
                self.run.step(shot::STEP);
                left -= shot::STEP;
            }
        }

        self.say(geometry, text_renderer);
    }

    fn draw(&mut self, scene: &mut Scene, camera: &mut Camera) {
        camera.position = self.view.eye();
        camera.target = self.view.at;

        if let Some(cursor) = self.pointing {
            self.aimed_at = on_the_cloth(camera, cursor);
        }

        // the ball follows the hand that has hold of it
        if self.carrying {
            if let Some(at) = self.aimed_at {
                self.run.carry_to(vec3(at.x, BALL_RADIUS, at.z));
            }
        }

        if let Some(at) = self.aimed_at {
            let off = at - self.run.cue();
            let way = vec3(off.x, 0.0, off.z);

            // a cursor on top of the ball points nowhere, and the shot keeps
            // whatever it had rather than taking a direction from nothing
            if way.length() > BALL_RADIUS {
                self.aim = way.normalize();
            }
        }

        let (Some(sphere), Some(block)) = (self.sphere, self.block) else {
            return;
        };

        scene.push_material(
            block,
            &Transform::at(vec3(0.0, -1.0, 0.0)).with_scale(vec3(
                (HALF_LONG + 3.0) * 2.0,
                2.0,
                (HALF_WIDE + 3.0) * 2.0,
            )),
            CLOTH,
            CLOTH_SHEEN,
        );

        // the head string, while it is the line you have to break from behind.
        // A kitchen you cannot see is a rule you cannot follow: without this
        // the click simply did nothing and the table said nothing about why.
        if self.run.kitchen_only() {
            scene.push_colored(
                block,
                &Transform::at(vec3(table::head_string(), 0.004, 0.0)).with_scale(vec3(
                    0.12,
                    0.008,
                    table::HALF_WIDE * 2.0,
                )),
                LINE,
            );
        }

        for rail in table::cushions() {
            let size = rail.max - rail.min;
            scene.push_colored(
                block,
                &Transform::at((rail.min + rail.max) * 0.5).with_scale(size),
                RAIL,
            );
        }

        // a squashed sphere, so a pocket is round rather than square, and
        // barely shiny. The specular in spec 0012 is not tinted by the material
        // colour, so a black thing still takes a full white highlight, and the
        // lobe is `pow(dot, shininess)`: a low shininess is a huge one. This
        // came out as a white blob at the default and whiter at a shininess of
        // one, which is how it ended up square in the first place.
        for pocket in table::pockets() {
            scene.push_material(
                sphere,
                &Transform::at(vec3(pocket.x, 0.02, pocket.z)).with_scale(vec3(
                    POCKET_MOUTH * 1.8,
                    0.08,
                    POCKET_MOUTH * 1.8,
                )),
                POCKET,
                256.0,
            );
        }

        for ball in 1..=BALLS {
            if self.run.is_down(ball) {
                continue;
            }

            let at = Transform::at(self.run.bodies[ball].position)
                .with_rotation(self.run.facing(ball))
                .with_scale(Vec3::splat(BALL_RADIUS * 2.0));

            // the whole of a ball's paint is in its texture now, its number
            // included, so what it is drawn with is near enough white and the
            // colour comes from the picture
            match self.face(ball) {
                Some(face) => scene.push_textured(sphere, face, &at, CUE_BALL, 64.0),
                None => scene.push_colored(sphere, &at, PAINT[ball - 1]),
            }
        }

        scene.push_colored(
            sphere,
            &Transform::at(self.run.bodies[CUE].position)
                .with_scale(Vec3::splat(BALL_RADIUS * 2.0)),
            CUE_BALL,
        );

        // the shot, drawn as beads along it, as long as it is hard
        if self.run.phase() == Phase::Aiming {
            let along = self.way();
            let far = 2.0 + self.power / HARDEST * 8.0;

            for bead in 1..=10 {
                let out = bead as f32 / 10.0 * far;
                scene.push_colored(
                    sphere,
                    &Transform::at(self.run.cue() + along * (BALL_RADIUS + out))
                        .with_scale(Vec3::splat(0.14)),
                    AIM,
                );
            }
        }
    }

    fn process_keyboard(&mut self, input: KeyboardInput) {
        let down = input.state == KeyboardKeyState::Pressed;

        match input.key {
            KeyboardKey::Up => self.nudging[0] = down,
            KeyboardKey::Down => self.nudging[1] = down,
            KeyboardKey::Left => self.nudging[2] = down,
            KeyboardKey::Right => self.nudging[3] = down,
            KeyboardKey::LShift | KeyboardKey::RShift => self.sliding = down,
            KeyboardKey::W => self.walking_keys[0] = down,
            KeyboardKey::S => self.walking_keys[1] = down,
            KeyboardKey::A => self.walking_keys[2] = down,
            KeyboardKey::D => self.walking_keys[3] = down,
            KeyboardKey::Space if down => self.tip = Vec2::ZERO,
            KeyboardKey::R if down => {
                if self.run.phase() == Phase::Over {
                    self.run = Run::new();
                    self.power = 0.0;
                    self.tip = Vec2::ZERO;
                }
            }
            KeyboardKey::Escape if down => self.quitting = true,
            _ => {}
        }
    }

    fn process_mouse(&mut self, input: MouseInput) {
        if input.button != MouseButton::Left || self.run.phase() != Phase::Aiming {
            return;
        }

        // in hand, the ball is picked up and carried rather than teleported.
        // Clicking the cloth put it wherever the cursor happened to be, which
        // moved it on any stray click and gave no way to try a spot and think
        // better of it.
        if self.run.in_hand {
            if input.is_pressed() {
                self.carrying = self.on_the_cue_ball();
            } else if self.carrying {
                self.carrying = false;
                self.run.place(self.run.cue());
            }
            return;
        }

        if input.is_pressed() {
            self.charging = true;
            self.power = SOFTEST;
            return;
        }

        if self.charging {
            self.charging = false;
            self.run.shoot(self.way(), self.power, self.tip);
            self.power = 0.0;
        }
    }

    fn cursor_moved(&mut self, position: Vec2) {
        self.pointing = Some(position);
    }

    /// The mouse moved in device units, which keeps arriving while a button is
    /// held. Only the walk reads it; the aim reads where the cursor is.
    fn mouse_motion(&mut self, _delta: Vec2) {}

    /// The wheel leans in and out, per spec 0003.
    fn mouse_wheel(&mut self, delta: Vec2) {
        self.view.zoomed(delta.y);
    }

    fn is_quitting(&self) -> bool {
        self.quitting
    }

    fn focus_changed(&mut self, _focus: bool) {}
}

#[cfg(test)]
mod tests {
    use super::*;

    fn aimed(game: &mut PoolhallGame, cursor: Vec2) -> Vec3 {
        let mut scene = Scene::new();
        let mut camera = Camera::new();

        game.cursor_moved(cursor);
        game.draw(&mut scene, &mut camera);

        game.way()
    }

    #[test]
    fn the_cursor_points_the_shot() {
        let mut game = PoolhallGame::new();

        let left = aimed(&mut game, vec2(0.2, 0.5));
        let right = aimed(&mut game, vec2(0.8, 0.5));

        assert!(left != right, "the cursor did nothing");
        assert_eq!(left.y, 0.0, "the shot left the cloth");
        assert!((left.length() - 1.0).abs() < 1e-5);
    }

    #[test]
    fn a_cursor_off_the_window_does_not_move_the_shot() {
        let mut game = PoolhallGame::new();
        let was = aimed(&mut game, vec2(0.3, 0.5));

        let mut scene = Scene::new();
        let mut camera = Camera::new();
        game.draw(&mut scene, &mut camera);

        assert_eq!(game.way(), was);
    }

    #[test]
    fn the_arrow_keys_move_the_tip_and_stay_on_the_ball() {
        let mut game = PoolhallGame::new();
        let mut text = TextRenderer::new();
        let sound = SoundSystem::new();
        let mut geometry = Geometry::new();

        for key in [KeyboardKey::LShift, KeyboardKey::Down] {
            game.process_keyboard(KeyboardInput::new(key, KeyboardKeyState::Pressed, false));
        }
        for _ in 0..600 {
            game.update(1.0 / 60.0, &mut geometry, &mut text, &sound);
        }

        assert!(game.tip.y < 0.0, "the tip did not go low");
        assert!(
            game.tip.y >= -TIP_LIMIT,
            "the tip left the ball at {}",
            game.tip.y
        );
    }

    #[test]
    fn space_puts_the_tip_back_in_the_middle() {
        let mut game = PoolhallGame::new();
        game.tip = vec2(0.7, -0.6);

        game.process_keyboard(KeyboardInput::new(
            KeyboardKey::Space,
            KeyboardKeyState::Pressed,
            false,
        ));

        assert_eq!(game.tip, Vec2::ZERO);
    }

    #[test]
    fn in_hand_a_click_places_the_cue_ball_rather_than_shooting() {
        let mut game = PoolhallGame::new();
        assert!(game.run.in_hand, "the break is from hand");

        let mut scene = Scene::new();
        let mut camera = Camera::new();
        game.cursor_moved(vec2(0.45, 0.55));
        game.draw(&mut scene, &mut camera);

        game.process_mouse(MouseInput::new(
            MouseButton::Left,
            blitzkit::mouse::ButtonState::Pressed,
        ));

        assert_eq!(game.run.phase(), Phase::Aiming, "it shot while in hand");
        assert_eq!(game.run.shots(), 0);
        assert!(!game.charging, "it started winding up while in hand");
    }

    /// Spec 0001: the cue ball is taken hold of and carried, and letting go
    /// puts it down. Every click used to put it wherever the cursor happened
    /// to be, so nothing ever got as far as winding up a shot and a stray
    /// click moved the ball.
    #[test]
    fn the_cue_ball_is_carried_and_put_down() {
        let mut game = PoolhallGame::new();
        let press = MouseInput::new(MouseButton::Left, blitzkit::mouse::ButtonState::Pressed);
        let let_go = MouseInput::new(MouseButton::Left, blitzkit::mouse::ButtonState::Released);

        // a press away from the ball takes hold of nothing
        game.aimed_at = Some(vec3(-16.0, BALL_RADIUS, 6.0));
        game.process_mouse(press);
        assert!(!game.carrying, "it took hold of the cloth");
        game.process_mouse(let_go);
        assert!(game.run.in_hand, "a press on the cloth put the ball down");

        // a press on the ball picks it up, and it is still in hand
        game.aimed_at = Some(game.run.cue());
        game.process_mouse(press);
        assert!(game.carrying, "it did not take hold of the ball");
        assert!(game.run.in_hand, "taking hold of it put it down");

        // and letting go puts it down without taking a shot
        game.process_mouse(let_go);
        assert!(!game.carrying, "it is still being carried");
        assert!(!game.run.in_hand, "letting go did not put it down");
        assert_eq!(game.run.shots(), 0, "putting it down took a shot");

        // after which a press winds up a shot
        game.process_mouse(press);
        assert!(game.charging, "the next press did not wind up a shot");
    }

    #[test]
    fn the_keys_walk_and_the_mouse_is_for_the_shot() {
        let mut game = PoolhallGame::new();
        let mut text = TextRenderer::new();
        let sound = SoundSystem::new();
        let mut geometry = Geometry::new();
        let was = game.view.eye();

        game.process_keyboard(KeyboardInput::new(
            KeyboardKey::A,
            KeyboardKeyState::Pressed,
            false,
        ));
        for _ in 0..30 {
            game.update(1.0 / 60.0, &mut geometry, &mut text, &sound);
        }

        assert_ne!(game.view.eye(), was, "A did not walk round");
        assert_eq!(game.run.shots(), 0, "walking took a shot");

        // and letting go stops the walk
        game.process_keyboard(KeyboardInput::new(
            KeyboardKey::A,
            KeyboardKeyState::Released,
            false,
        ));
        let standing = game.view.eye();
        for _ in 0..30 {
            game.update(1.0 / 60.0, &mut geometry, &mut text, &sound);
        }

        assert_eq!(
            game.view.eye(),
            standing,
            "it kept walking after letting go"
        );

        // the mouse moving does nothing to where you stand
        game.mouse_motion(vec2(300.0, 120.0));
        assert_eq!(game.view.eye(), standing, "the mouse moved the eye");
    }

    /// Spec 0003: walking round moves the eye and leaves the table where it is.
    #[test]
    fn walking_moves_the_eye_and_not_the_table() {
        let mut game = PoolhallGame::new();
        let mut scene = Scene::new();
        let mut camera = Camera::new();
        let mut text = TextRenderer::new();
        let sound = SoundSystem::new();
        let mut geometry = Geometry::new();

        game.draw(&mut scene, &mut camera);
        let was = camera.position;

        game.process_keyboard(KeyboardInput::new(
            KeyboardKey::D,
            KeyboardKeyState::Pressed,
            false,
        ));
        for _ in 0..30 {
            game.update(1.0 / 60.0, &mut geometry, &mut text, &sound);
        }
        game.draw(&mut scene, &mut camera);

        assert_ne!(camera.position, was, "the eye did not move");
        assert_eq!(camera.target, Vec3::ZERO, "the table moved");
        assert!(
            (camera.position.length() - was.length()).abs() < 1e-2,
            "it walked towards the table"
        );
    }

    /// Spec 0003: and the arrows slide what you are looking at, which is the
    /// one thing walking round cannot do.
    #[test]
    fn the_arrows_slide_what_you_are_looking_at() {
        let mut game = PoolhallGame::new();
        let mut scene = Scene::new();
        let mut camera = Camera::new();
        let mut text = TextRenderer::new();
        let sound = SoundSystem::new();
        let mut geometry = Geometry::new();

        game.process_keyboard(KeyboardInput::new(
            KeyboardKey::Left,
            KeyboardKeyState::Pressed,
            false,
        ));
        for _ in 0..30 {
            game.update(1.0 / 60.0, &mut geometry, &mut text, &sound);
        }
        game.draw(&mut scene, &mut camera);

        assert_ne!(camera.target, Vec3::ZERO, "the arrows did not slide");
        assert_eq!(game.tip, Vec2::ZERO, "the arrows moved the cue tip as well");
    }

    #[test]
    fn the_readout_does_not_pile_up() {
        let mut game = PoolhallGame::new();
        let mut text = TextRenderer::new();
        let sound = SoundSystem::new();
        let mut geometry = Geometry::new();

        game.update(1.0 / 60.0, &mut geometry, &mut text, &sound);
        let after_one = text.render_texts.len();

        for _ in 0..100 {
            game.update(1.0 / 60.0, &mut geometry, &mut text, &sound);
        }

        assert_eq!(text.render_texts.len(), after_one, "the readout piled up");
        assert!(after_one > 0);
    }

    #[test]
    fn the_readout_lines_are_evenly_spaced() {
        let game = PoolhallGame::new();
        let lines: Vec<Vec2> = (0..game.readout().len()).map(|n| game.line(n)).collect();

        for pair in lines.windows(2) {
            assert_eq!(pair[0].x, pair[1].x);
            assert_eq!(pair[1].y - pair[0].y, HUD_APART);
        }

        assert!(game.line(1).y - game.line(0).y > HUD_SIZE);
    }

    #[test]
    fn one_shot_is_not_one_shots() {
        assert_eq!(shots(0), "0 shots");
        assert_eq!(shots(1), "1 shot");
        assert_eq!(fouls(0), "no fouls");
        assert_eq!(fouls(1), "one foul");
        assert_eq!(fouls(3), "3 fouls");
    }
}
