//! What the balls are painted. See `specs/0002-solids-and-stripes.md`.
//!
//! A stripe is drawn rather than loaded, the way marble's checker is: white with
//! a band across the middle of it is cheaper to write than to ship.

use blitzkit::texture::TextureData;

use crate::table::{BALLS, EIGHT};

/// How big a stripe texture is.
pub const SIZE: u32 = 64;

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

/// A stripe: white, with a band of `colour` across its middle.
pub fn stripe(colour: [u8; 3]) -> TextureData {
    TextureData::from_pixels(SIZE, SIZE, band(colour))
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
        let skin = stripe(RED);

        assert_eq!(skin.width(), SIZE);
        assert_eq!(skin.height(), SIZE);
    }
}
