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
const RAIL: Vec4 = vec4(0.28, 0.17, 0.11, 1.0);
const POCKET: Vec4 = vec4(0.03, 0.04, 0.05, 1.0);
const CUE_BALL: Vec4 = vec4(0.97, 0.96, 0.92, 1.0);
const AIM: Vec4 = vec4(1.0, 0.95, 0.75, 1.0);

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
    /// Up, down, left, right, held.
    nudging: [bool; 4],
    power: f32,
    charging: bool,
    /// Where the player is standing, per spec 0003.
    view: View,
    /// Whether they are walking round the table, which is the right button
    /// held: the left one is already aiming and shooting.
    walking: bool,
    sphere: Option<MeshId>,
    block: Option<MeshId>,
    /// One band per striped ball, in ball order from nine upwards.
    stripes: Vec<TextureId>,
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
            power: 0.0,
            charging: false,
            view: View::new(),
            walking: false,
            sphere: None,
            block: None,
            stripes: Vec::new(),
            quitting: false,
            staged: crate::staged(),
            broken: false,
        }
    }

    /// Which way the shot goes: from the cue ball towards what is pointed at.
    pub fn way(&self) -> Vec3 {
        self.aim
    }

    /// The band this ball wears, if it wears one.
    fn striped(&self, ball: usize) -> Option<TextureId> {
        if !paint::is_striped(ball) {
            return None;
        }

        self.stripes.get(paint::partner(ball)).copied()
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
                _ if self.run.in_hand => String::from("ball in hand: click to place it"),
                _ => String::from("point and hold to shoot, right drag to walk round"),
            },
            format!(
                "arrow keys put the tip at {:+.1} across, {:+.1} up",
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

        self.stripes = (1..=BALLS)
            .filter(|ball| paint::is_striped(*ball))
            .map(|ball| {
                let hue = PAINT[paint::partner(ball)];
                let band = [
                    (hue.x * 255.0) as u8,
                    (hue.y * 255.0) as u8,
                    (hue.z * 255.0) as u8,
                ];

                renderer.add_texture(&paint::stripe(band))
            })
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

        let across = (self.nudging[3] as i32 - self.nudging[2] as i32) as f32;
        let up = (self.nudging[0] as i32 - self.nudging[1] as i32) as f32;
        self.tip = vec2(
            (self.tip.x + across * TIP_PER_SECOND * dt).clamp(-TIP_LIMIT, TIP_LIMIT),
            (self.tip.y + up * TIP_PER_SECOND * dt).clamp(-TIP_LIMIT, TIP_LIMIT),
        );

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
        camera.target = Vec3::ZERO;

        if let Some(cursor) = self.pointing {
            self.aimed_at = on_the_cloth(camera, cursor);
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

        scene.push_colored(
            block,
            &Transform::at(vec3(0.0, -1.0, 0.0)).with_scale(vec3(
                (HALF_LONG + 3.0) * 2.0,
                2.0,
                (HALF_WIDE + 3.0) * 2.0,
            )),
            CLOTH,
        );

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

            // a striped ball is white with a band painted on, so the colour it
            // is drawn with is white and the band comes from the texture
            match self.striped(ball) {
                Some(band) => scene.push_textured(sphere, band, &at, CUE_BALL, 64.0),
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
        if input.button == MouseButton::Right {
            self.walking = input.is_pressed();
            return;
        }

        if input.button != MouseButton::Left || self.run.phase() != Phase::Aiming {
            return;
        }

        // in hand, a click puts the cue ball down rather than shooting
        if self.run.in_hand {
            if input.is_pressed() {
                if let Some(at) = self.aimed_at {
                    self.run.place(vec3(at.x, BALL_RADIUS, at.z));
                }
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
    fn mouse_motion(&mut self, delta: Vec2) {
        if self.walking {
            self.view.dragged(delta.x, delta.y);
        }
    }

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

        game.process_keyboard(KeyboardInput::new(
            KeyboardKey::Down,
            KeyboardKeyState::Pressed,
            false,
        ));
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

    #[test]
    fn placing_the_cue_ball_lets_the_next_click_shoot() {
        // every click used to put the ball down again, so nothing ever got as
        // far as winding up a shot
        let mut game = PoolhallGame::new();
        let mut scene = Scene::new();
        let mut camera = Camera::new();
        game.cursor_moved(vec2(0.45, 0.55));
        game.draw(&mut scene, &mut camera);

        let press = MouseInput::new(MouseButton::Left, blitzkit::mouse::ButtonState::Pressed);
        let let_go = MouseInput::new(MouseButton::Left, blitzkit::mouse::ButtonState::Released);

        game.process_mouse(press);
        assert!(
            !game.run.in_hand,
            "it is still in hand after being put down"
        );
        assert_eq!(game.run.shots(), 0, "putting it down took a shot");

        game.process_mouse(press);
        assert!(game.charging, "the next press did not wind up a shot");

        game.process_mouse(let_go);
        assert_eq!(game.run.shots(), 1, "letting go did not shoot");
    }

    #[test]
    fn the_right_button_walks_and_the_left_shoots() {
        let mut game = PoolhallGame::new();
        let was = game.view.eye();

        // the right button is the one nothing else is using
        game.process_mouse(MouseInput::new(
            MouseButton::Right,
            blitzkit::mouse::ButtonState::Pressed,
        ));
        game.mouse_motion(vec2(120.0, 40.0));

        assert_ne!(game.view.eye(), was, "the right drag did not walk");
        assert_eq!(game.run.shots(), 0, "walking took a shot");

        // and letting go stops the walk
        game.process_mouse(MouseInput::new(
            MouseButton::Right,
            blitzkit::mouse::ButtonState::Released,
        ));
        let standing = game.view.eye();
        game.mouse_motion(vec2(200.0, 0.0));

        assert_eq!(
            game.view.eye(),
            standing,
            "it kept walking after letting go"
        );
    }

    #[test]
    fn walking_moves_the_eye_and_not_the_table() {
        let mut game = PoolhallGame::new();
        let mut scene = Scene::new();
        let mut camera = Camera::new();

        game.draw(&mut scene, &mut camera);
        let was = camera.position;

        game.process_mouse(MouseInput::new(
            MouseButton::Right,
            blitzkit::mouse::ButtonState::Pressed,
        ));
        game.mouse_motion(vec2(300.0, 0.0));
        game.draw(&mut scene, &mut camera);

        assert_ne!(camera.position, was, "the eye did not move");
        assert_eq!(camera.target, Vec3::ZERO, "the table moved");
        assert!(
            (camera.position.length() - was.length()).abs() < 1e-2,
            "it walked towards the table"
        );
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
