//! What the balls are painted. See `specs/0002-solids-and-stripes.md`.
//!
//! A stripe is drawn rather than loaded, the way marble's checker is: white with
//! a band across the middle of it is cheaper to write than to ship.

use blitzkit::texture::TextureData;

use crate::table::{BALLS, EIGHT};

/// How big a ball's texture is.
///
/// It was 64, which is plenty for a band and nowhere near enough for a number.
/// A numeral wants about twenty pixels across to be a numeral rather than a
/// smudge, and the circle it sits in is an eighth of the way round the ball.
pub const SIZE: u32 = 256;

/// Where the band starts and ends, as a fraction of the way down.
///
/// The middle of the texture's v is the ball's equator on blitzkit's sphere, so
/// a band about the middle is a band round the ball.
///
/// Wider than a real ball's. From a camera above a table you see more of a
/// ball's poles than its band, so a true stripe read as a white ball with a
/// line on it. This is the width that reads as striped from up there, and the
/// realism it costs is worth less than knowing which ball you are looking at.
pub const BAND_FROM: f32 = 0.20;
pub const BAND_TO: f32 = 0.80;

/// The white a ball is, which is not quite white.
pub const WHITE: [u8; 3] = [243, 240, 232];

/// What a number is printed in.
pub const INK: [u8; 3] = [24, 22, 20];

/// How far a number's circle reaches from its pole, in radians.
///
/// One over each pole, which is two, the count a real ball has. A real one
/// wears them on its equator instead, and from a camera above a table that is
/// edge on: the circle showed as a white sliver at the rim and the numeral was
/// turned away. Up is where you are looking from, so up is where the number
/// goes. The same trade the band makes by being wider than a real ball's.
///
/// Four tenths of a radian is a cap about eight tenths of the ball across,
/// which is what a real number circle measures.
pub const SPOT_CAP: f32 = 0.4;

/// How much of that cap the numeral fills.
pub const SPOT_FILL: f32 = 0.78;

/// How tall a numeral is in the font below, and how wide.
pub const GLYPH: (usize, usize) = (5, 7);

/// The ten digits, five across and seven down, a row to a number with the top
/// bit leftmost.
///
/// Drawn rather than loaded, the way the band is. A font file for ten glyphs
/// that are never set in a line is more to ship and more to go wrong.
const DIGITS: [[u8; 7]; 10] = [
    [
        0b01110, 0b10001, 0b10011, 0b10101, 0b11001, 0b10001, 0b01110,
    ],
    [
        0b00100, 0b01100, 0b00100, 0b00100, 0b00100, 0b00100, 0b01110,
    ],
    [
        0b01110, 0b10001, 0b00001, 0b00010, 0b00100, 0b01000, 0b11111,
    ],
    [
        0b11111, 0b00010, 0b00100, 0b00010, 0b00001, 0b10001, 0b01110,
    ],
    [
        0b00010, 0b00110, 0b01010, 0b10010, 0b11111, 0b00010, 0b00010,
    ],
    [
        0b11111, 0b10000, 0b11110, 0b00001, 0b00001, 0b10001, 0b01110,
    ],
    [
        0b00110, 0b01000, 0b10000, 0b11110, 0b10001, 0b10001, 0b01110,
    ],
    [
        0b11111, 0b00001, 0b00010, 0b00100, 0b01000, 0b01000, 0b01000,
    ],
    [
        0b01110, 0b10001, 0b10001, 0b01110, 0b10001, 0b10001, 0b01110,
    ],
    [
        0b01110, 0b10001, 0b10001, 0b01111, 0b00001, 0b00010, 0b01100,
    ],
];

/// How many balls are solid, and so which are striped.
pub const SOLIDS: usize = 7;

/// Whether this ball wears a band.
///
/// The eight is black and solid, which is why this is not simply "above seven".
pub fn is_striped(ball: usize) -> bool {
    ball > SOLIDS && ball != EIGHT && ball <= BALLS
}

/// The hue a striped ball takes: the one seven below it, which is how a real
/// set is made.
pub fn partner(ball: usize) -> usize {
    // nine is the first striped ball and takes the first hue, so the eight is
    // what this counts from rather than the seven solids
    ball - EIGHT - 1
}

/// The pixels of one, as straight RGBA.
///
/// Separate from the texture because a `TextureData` keeps its pixels to itself,
/// and a band nothing can read is a band nothing can check.
pub fn band(colour: [u8; 3]) -> Vec<u8> {
    let from = (SIZE as f32 * BAND_FROM) as u32;
    let to = (SIZE as f32 * BAND_TO) as u32;
    let mut pixels = Vec::with_capacity((SIZE * SIZE * 4) as usize);

    for y in 0..SIZE {
        let paint = if (from..to).contains(&y) {
            colour
        } else {
            WHITE
        };
        for _ in 0..SIZE {
            pixels.extend_from_slice(&[paint[0], paint[1], paint[2], 255]);
        }
    }

    pixels
}

/// A whole ball: its base, and its number in a white circle on each side.
pub fn ball(number: usize, hue: [u8; 3]) -> TextureData {
    TextureData::from_pixels(SIZE, SIZE, face(number, hue))
}

/// The pixels of one, as straight RGBA.
///
/// The number sits on the equator and nowhere else. blitzkit's sphere runs u
/// round and v pole to pole, so a circle drawn square in the texture comes out
/// round on the ball only where the two agree, and they agree at the middle.
/// Towards either pole u is stretched by the cosine of the latitude and a
/// numeral there smears sideways into a band. Nothing here curves the glyphs,
/// because the ball does that: what is drawn flat on the equator is read off a
/// curved surface, which is the whole of the trick and the reason the circle
/// cannot wander off it.
pub fn face(number: usize, hue: [u8; 3]) -> Vec<u8> {
    let mut pixels = if is_striped(number) {
        band(hue)
    } else {
        let mut flat = Vec::with_capacity((SIZE * SIZE * 4) as usize);
        for _ in 0..SIZE * SIZE {
            flat.extend_from_slice(&[hue[0], hue[1], hue[2], 255]);
        }
        flat
    };

    for north in [true, false] {
        cap(&mut pixels, number, north);
    }

    pixels
}

/// Stamps one white circle with a number in it, over a pole.
///
/// Over a pole and not round the equator, which is where this started and is
/// where a real ball wears it. From a camera above a table you are looking at
/// the top of a ball: an equator is edge on, and a circle there showed as a
/// white sliver at the rim with the numeral turned away. The same reason the
/// band is wider than a real ball's.
///
/// A pole is where the sphere's own stretch is worst, so the drawing is done in
/// the stretch rather than against it. The texture's x is longitude and its y
/// is latitude, so a cap over a pole is a strip across the whole width: every
/// column of it meets at the one point. Each texel is turned back into a place
/// on the flat the pole touches, `lat` out from the middle at `lon` round, and
/// the glyph is read there. Drawn square into the strip instead, a numeral
/// comes out smeared into a ring, which is what the stretch does to anything
/// that ignores it.
fn cap(pixels: &mut [u8], number: usize, north: bool) {
    let digits: Vec<usize> = if number >= 10 {
        vec![number / 10, number % 10]
    } else {
        vec![number]
    };
    // the glyph's own grid, in the flat's units, kept square so a numeral is
    // not stretched by being two digits wide
    let across = digits.len() * GLYPH.0 + digits.len().saturating_sub(1);
    let span = (across.max(GLYPH.1)) as f32 / SPOT_FILL;

    let rows = (SIZE as f32 * SPOT_CAP / std::f32::consts::PI).ceil() as u32 + 1;
    for row in 0..rows.min(SIZE) {
        let y = if north { row } else { SIZE - 1 - row };
        let lat = (row as f32 + 0.5) / SIZE as f32 * std::f32::consts::PI;
        if lat > SPOT_CAP {
            continue;
        }

        for x in 0..SIZE {
            let lon = (x as f32 + 0.5) / SIZE as f32 * std::f32::consts::TAU;
            let at = ((y * SIZE + x) * 4) as usize;
            pixels[at..at + 3].copy_from_slice(&WHITE);

            // where this texel lands on the flat, in glyph cells
            let out = lat / SPOT_CAP * span * 0.5;
            let (across_at, down_at) = (out * lon.cos(), out * lon.sin());
            let column = (across_at + across as f32 * 0.5).floor();
            let line = (down_at + GLYPH.1 as f32 * 0.5).floor();
            if column < 0.0 || line < 0.0 || line >= GLYPH.1 as f32 {
                continue;
            }

            let (column, line) = (column as usize, line as usize);
            let digit = column / (GLYPH.0 + 1);
            let in_digit = column % (GLYPH.0 + 1);
            if digit >= digits.len() || in_digit >= GLYPH.0 {
                continue;
            }

            if DIGITS[digits[digit]][line] & (1 << (GLYPH.0 - 1 - in_digit)) != 0 {
                pixels[at..at + 3].copy_from_slice(&INK);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const RED: [u8; 3] = [210, 40, 36];

    fn pixel(pixels: &[u8], x: u32, y: u32) -> [u8; 3] {
        let at = ((y * SIZE + x) * 4) as usize;

        [pixels[at], pixels[at + 1], pixels[at + 2]]
    }

    #[test]
    fn a_stripe_is_white_with_a_band() {
        let pixels = band(RED);

        assert_eq!(pixels.len(), (SIZE * SIZE * 4) as usize);
        assert_eq!(pixel(&pixels, 0, 0), WHITE, "the top is not white");
        assert_eq!(
            pixel(&pixels, 0, SIZE - 1),
            WHITE,
            "the bottom is not white"
        );
        assert_eq!(pixel(&pixels, 0, SIZE / 2), RED, "the middle is not banded");
    }

    #[test]
    fn the_band_is_the_colour_asked_for() {
        for colour in [RED, [20, 60, 180], [250, 200, 40]] {
            let pixels = band(colour);

            assert_eq!(pixel(&pixels, SIZE / 2, SIZE / 2), colour);
        }
    }

    #[test]
    fn a_stripe_is_opaque() {
        assert!(band(RED).chunks(4).all(|pixel| pixel[3] == 255));
    }

    #[test]
    fn the_band_goes_all_the_way_round() {
        // every column is the same, which is what makes it a band rather than a
        // patch: a ball turning about its own axis looks the same all the way
        let pixels = band(RED);

        for x in 0..SIZE {
            assert_eq!(pixel(&pixels, x, SIZE / 2), RED, "column {} is bare", x);
        }
    }

    #[test]
    fn seven_are_striped_and_the_eight_is_not() {
        let striped: Vec<usize> = (1..=BALLS).filter(|ball| is_striped(*ball)).collect();

        assert_eq!(striped.len(), SOLIDS, "{:?}", striped);
        assert!(!is_striped(EIGHT), "the eight is striped");

        // nine takes the one's hue and fifteen the seven's, which is how a real
        // set is made
        assert_eq!(
            partner(EIGHT + 1),
            0,
            "the nine does not take the one's hue"
        );
        assert_eq!(
            partner(BALLS),
            SOLIDS - 1,
            "the fifteen does not take the seven's"
        );

        for ball in &striped {
            let partner = partner(*ball);

            assert!(partner < SOLIDS, "ball {} partners {}", ball, partner);
        }
    }

    #[test]
    fn it_is_the_size_it_says() {
        let skin = ball(1, RED);

        assert_eq!(skin.width(), SIZE);
        assert_eq!(skin.height(), SIZE);
    }

    /// Spec 0002: every ball carries its own number, over each pole.
    #[test]
    fn every_ball_wears_its_number() {
        let rows = (SIZE as f32 * SPOT_CAP / std::f32::consts::PI) as u32;

        for number in 1..=BALLS {
            let pixels = face(number, RED);

            // near the rim of the cap, which is inside the circle and outside
            // any numeral. The pole itself is where the numeral sits.
            for (which, row) in [("north", rows - 2), ("south", SIZE - 1 - (rows - 2))] {
                assert_eq!(
                    pixel(&pixels, 0, row),
                    WHITE,
                    "ball {} has no circle at its {} pole",
                    number,
                    which
                );
            }

            let inked = (0..rows)
                .flat_map(|row| [row, SIZE - 1 - row])
                .any(|row| (0..SIZE).any(|x| pixel(&pixels, x, row) == INK));
            assert!(inked, "ball {} has an empty circle", number);
        }
    }

    /// Spec 0002: the circle reaches only as far down the ball as it says, and
    /// the equator is clear. A cap that ran on would be a white ball.
    #[test]
    fn the_circle_stops_short_of_the_middle() {
        // a solid one, because a striped ball's poles are white from the band
        // and white there would prove nothing
        let pixels = face(1, RED);
        let rows = (SIZE as f32 * SPOT_CAP / std::f32::consts::PI) as u32 + 2;

        for y in [rows, SIZE / 2, SIZE - 1 - rows] {
            for x in 0..SIZE {
                let at = pixel(&pixels, x, y);
                assert_ne!(at, INK, "there is ink at {}, {}, below the cap", x, y);
                assert_ne!(at, WHITE, "the circle reaches {}, {}", x, y);
            }
        }
    }

    /// Spec 0002: and the numeral is drawn in the stretch rather than against
    /// it, which is what keeps it a numeral at a pole.
    ///
    /// Every column of the texture meets at the pole, so a glyph drawn square
    /// into the strip would put ink in the very top row all the way across. One
    /// drawn in polar coordinates puts the middle of the numeral there and
    /// nothing else: the top row is one point on the ball, not a line.
    #[test]
    fn the_numeral_is_not_smeared_round_the_pole() {
        let pixels = face(1, RED);
        let inked = (0..SIZE).filter(|x| pixel(&pixels, *x, 0) == INK).count();

        assert!(
            inked == 0 || inked == SIZE as usize,
            "the top row is {} of {} inked, so the glyph is being read as a ring",
            inked,
            SIZE
        );
    }

    /// Spec 0002: two different balls are two different pictures. The one test
    /// that would have caught painting every ball the same number.
    #[test]
    fn no_two_balls_look_alike() {
        let faces: Vec<Vec<u8>> = (1..=BALLS).map(|ball| face(ball, RED)).collect();

        for (n, one) in faces.iter().enumerate() {
            for (m, other) in faces.iter().enumerate().skip(n + 1) {
                assert_ne!(one, other, "balls {} and {} are the same", n + 1, m + 1);
            }
        }
    }
}
