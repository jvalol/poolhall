//! The window: the table, the rack, aiming, and where on the ball you hit it.
//! See `specs/0001-the-rack.md`.

use blitzkit::camera::Camera;
use blitzkit::geometry::Geometry;
use blitzkit::keyboard::{KeyboardInput, KeyboardKey, KeyboardKeyState};
use blitzkit::mesh::{MeshData, Transform};
use blitzkit::mouse::{MouseButton, MouseInput};
use blitzkit::renderer::render_text::{RenderText, TextRenderer};
use blitzkit::renderer::scene::{MeshId, Scene};
use blitzkit::renderer::Renderer;
use blitzkit::sound::SoundSystem;
use blitzkit::Game;
use glam::{vec2, vec3, vec4, Vec2, Vec3, Vec4};

use crate::rules::{Foul, Outcome, Phase, Run, CUE};
use crate::shot::{self, HARDEST, SOFTEST};
use crate::table::{self, BALLS, BALL_RADIUS, HALF_LONG, HALF_WIDE, POCKET_MOUTH};

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

/// Where the camera sits. Straight down the table and high, because the game is
/// read off the cloth.
/// High and a little back from the head rail, so the whole table is in the
/// window and the far pockets are still something you can point at.
const EYE: Vec3 = vec3(-26.0, 42.0, 0.0);

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

pub struct DiamondGame {
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
    sphere: Option<MeshId>,
    block: Option<MeshId>,
    quitting: bool,
}

impl Default for DiamondGame {
    fn default() -> Self {
        Self::new()
    }
}

impl DiamondGame {
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
            sphere: None,
            block: None,
            quitting: false,
        }
    }

    /// Which way the shot goes: from the cue ball towards what is pointed at.
    pub fn way(&self) -> Vec3 {
        self.aim
    }

    fn line(&self, n: usize) -> Vec2 {
        vec2(HUD_LEFT, HUD_TOP + n as f32 * HUD_APART)
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
                _ => String::from("point with the mouse, hold to shoot"),
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

impl Game for DiamondGame {
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
    }

    fn update(
        &mut self,
        dt: f32,
        _geometry: &mut Geometry,
        text_renderer: &mut TextRenderer,
        _sound_system: &SoundSystem,
    ) {
        text_renderer.reset();

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

        for (n, text) in self.readout().into_iter().enumerate() {
            text_renderer.push_render_text(RenderText {
                position: self.line(n),
                color: vec4(1.0, 1.0, 1.0, 0.9),
                size: HUD_SIZE,
                text,
                ..Default::default()
            });
        }
    }

    fn draw(&mut self, scene: &mut Scene, camera: &mut Camera) {
        camera.position = EYE;
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

            scene.push_colored(
                sphere,
                &Transform::at(self.run.bodies[ball].position)
                    .with_scale(Vec3::splat(BALL_RADIUS * 2.0)),
                PAINT[ball - 1],
            );
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

    fn is_quitting(&self) -> bool {
        self.quitting
    }

    fn focus_changed(&mut self, _focus: bool) {}
}

#[cfg(test)]
mod tests {
    use super::*;

    fn aimed(game: &mut DiamondGame, cursor: Vec2) -> Vec3 {
        let mut scene = Scene::new();
        let mut camera = Camera::new();

        game.cursor_moved(cursor);
        game.draw(&mut scene, &mut camera);

        game.way()
    }

    #[test]
    fn the_cursor_points_the_shot() {
        let mut game = DiamondGame::new();

        let left = aimed(&mut game, vec2(0.2, 0.5));
        let right = aimed(&mut game, vec2(0.8, 0.5));

        assert!(left != right, "the cursor did nothing");
        assert_eq!(left.y, 0.0, "the shot left the cloth");
        assert!((left.length() - 1.0).abs() < 1e-5);
    }

    #[test]
    fn a_cursor_off_the_window_does_not_move_the_shot() {
        let mut game = DiamondGame::new();
        let was = aimed(&mut game, vec2(0.3, 0.5));

        let mut scene = Scene::new();
        let mut camera = Camera::new();
        game.draw(&mut scene, &mut camera);

        assert_eq!(game.way(), was);
    }

    #[test]
    fn the_arrow_keys_move_the_tip_and_stay_on_the_ball() {
        let mut game = DiamondGame::new();
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
        let mut game = DiamondGame::new();
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
        let mut game = DiamondGame::new();
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
        let mut game = DiamondGame::new();
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
    fn the_readout_does_not_pile_up() {
        let mut game = DiamondGame::new();
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
        let game = DiamondGame::new();
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
