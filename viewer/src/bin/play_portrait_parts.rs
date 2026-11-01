//! The radar portraits' parts: what the game draws beside and over each slot's portrait besides
//! the face itself. FUN_000c3680 gives every portrait widget (class 0xe9535a, 0xc3550) five parts
//! in this order, which is also their update order: the **health flash** (class 0x1f0aff78), the
//! selection tint (0xfc8a6dfb, play_hud.rs `selected_face`) and the three **icons** (the speech
//! bubble, the "!" and the scope ring: not here yet); the slot's **health channel** (class
//! 0x2a9b279) is the radar's own widget between that portrait and the next. This module is their
//! rules, tables and colours; the nodes that show them are play_hud.rs's (`SquadPart::Portrait`,
//! `ChannelFrame`, `ChannelFill`).
//!
//! * **The health flash** (`Flash`): a hit (health under last frame's) sets a level to at least
//!   1.5, and under 20% of the most health, a level of 0 restarts at 1.0, so the portrait is red
//!   while the level is over 0.5: 1.0 s after a hit, and 0.5 s of every second while the member is
//!   hurt that badly. The channel's frame goes red with it and its fill pale in a hit's first
//!   half second. A dead member's portrait (its skull) is (128, 128, 128).
//! * **The health channel** (`CHANNELS`): a 32 x 32 frame sprite (a blue outline) and a 32 x 32 fill
//!   sprite (a gradient) at the radar panel's diagonals. The fill is drawn in one flat colour by a
//!   pixel program that keeps only the gradient's texels at or above the empty fraction
//!   (`empty_fraction`, `fill_shows`).
//! * **The tint** (`tint_texels`): the sprite draw (FUN_000ba950) selects one of two pixel programs
//!   by the colour it is given: plain texture x colour while red, green and blue are all equal
//!   (the white of an ordinary sprite, the dead skull's grey), and, as soon as one differs (the
//!   flash's red, the icons' blue-white, the channel's red and pink), the program at 0x3e0140.
//!
//! The pixel program, read from the combiner words in default.xbe (D3DPIXELSHADERDEF at 0x3e0140,
//! chosen by the sprite draw's FUN_000ba820 as technique 0x23): it takes the sprite's texel T and
//! its colour V, and writes V x min(1, T.r^2 + T.g^2 + T.b^2) (the dot product of the texel with
//! itself, a grey that squares the texture's contrast, times the colour), with the texel's alpha.
//! The flash's red face, the "!" and the bubble in blue-white, and the red frame outline all come
//! out of it. Measured texel by texel on the texture and the video, the ground taken from the
//! rest frame: Hawk's red face (255, 128, 128) in take01 of todo/36-squad-voice is (143, 74, 77)
//! on screen against the program's (143, 77, 80), where a plain colour multiply gives (99, 55, 68)
//! (the video's red channel rising 45 levels where a multiply leaves it); Tex's in take02 of
//! todo/354-portrait-health-flash is (108, 56.7, 59.5) against (107.8, 56.5, 59.0).

use super::*;

/// The fraction of its most health under which a member's portrait blinks: the flash part's +0x38,
/// set by its attach at 0xc431c (the float 0x3e4ccccd).
pub const LOW_HEALTH: f32 = 0.2;
/// The level a hit sets (at least): `maxss` with 0x3fc00000 at 0xc4424, FUN_000c43b0.
pub const HIT_LEVEL: f32 = 1.5;
/// The level the low-health blink starts at, once the last one has run out (0xc446c, the float 1.0).
pub const BLINK_LEVEL: f32 = 1.0;
/// The portrait and the channel's frame are red while the level is over this (0x3a4f74, 0.5: the
/// compares at 0xc44b5 and 0xc33df).
pub const RED_ABOVE: f32 = 0.5;
/// Above this level (0x3a4f78, 1.0) the whiteness is falling from a hit's 255 (0xc4330).
pub const SWELL_ABOVE: f32 = 1.0;
/// The whiteness at rest (`mov edx, 0x80` at 0xc4338), and the half every other value is measured
/// from (0x3a5178, 128.0).
pub const HALF: u8 = 128;

/// The dead skull's colour: the flash part sets the portrait's colour bytes (128, 128, 128) when
/// the member's health is 0 (0xc452f-0xc453a). Red, green and blue equal, so the plain program.
pub const SKULL_GREY: u8 = 128;
/// The channel's frame while its slot's level is over RED_ABOVE: FUN_000b8820's bytes a0 20 20 ff
/// (0xc33ef); the frame is drawn white (FUN_000b88a0) otherwise.
pub const FRAME_RED: [u8; 3] = [160, 32, 32];
/// The channel's fill colour: the bytes 0xc0 0x57 0x5e 0xff the draw stores at 0xc341f-0xc342e
/// (take01's fill measures (189, 86, 93) through the video).
pub const FILL_RGB: [u8; 3] = [192, 87, 94];
/// One slot's health channel (see the module's text). The textures are common's, found by their
/// hashes in FUN_000c3080 (0xc3253, 0xc326b for slots 1 and 3; 0xc322e, 0xc323f for 2 and 4).
#[derive(Clone, Copy, Debug)]
pub struct Channel {
    /// the frame sprite's texture (the widget's +0x20), a 32 x 32 blue outline
    pub frame: u32,
    /// the fill sprite's texture (+0x30), a 32 x 32 gradient
    pub fill: u32,
    /// where both are drawn: (x, y, width, height) from the radar panel's corner, the table
    /// 0x3c8da8's row for the slot (FUN_000c3080 reads x, y, then the size into +0xe and +0xc); a
    /// negative size is the sprite mirrored, the rect reaching back from (x, y)
    pub rect: (f32, f32, f32, f32),
}

/// The four channels by character (Brutus, Flint, Hawk, Tex: the order of `CHARACTERS`), the
/// table 0x3c8da8's rows for their slots 3, 2, 4 and 1. Each lies between its portrait and the
/// next on the radar's diagonals, the bright end of its gradient at the portrait.
pub const CHANNELS: [Channel; 4] = [
    Channel { frame: 0xE070_6E07, fill: 0xE684_DA10, rect: (48.0, 116.0, -32.0, -32.0) },
    Channel { frame: 0x00FF_5CFB, fill: 0x1CEE_1256, rect: (115.0, 108.0, -32.0, -32.0) },
    Channel { frame: 0x00FF_5CFB, fill: 0x1CEE_1256, rect: (13.0, 21.0, 32.0, 32.0) },
    Channel { frame: 0xE070_6E07, fill: 0xE684_DA10, rect: (80.0, 12.0, 32.0, 32.0) },
];

/// The colour a portrait is drawn in this frame, as the flash part's update left it.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum FaceTint {
    /// white: the face as it is
    #[default]
    White,
    /// (255, a, a): red, with the flash's whiteness a (`whiteness`)
    Red(u8),
    /// (128, 128, 128): the member has no health left
    Grey,
}

/// One slot's health flash part (class 0x1f0aff78, the widget's +0x34 and +0x3c).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Flash {
    /// the member's health at the last update (+0x34; 0 as the attach 0xc4310 makes it)
    pub last: f32,
    /// the flash level (+0x3c): 0 at rest, 1.5 after a hit, 1.0 starting a low-health blink, falling
    /// by the frame's time to 0
    pub level: f32,
    /// the portrait's colour from the last update
    pub face: FaceTint,
}

impl Flash {
    /// One update of the part (0xc43b0), `health` and `max` the member's (char+0x54 and +0x50) and
    /// `dt` the HUD's frame time:
    ///  - health 0: the level is 0; health under last frame's: the level is at least HIT_LEVEL;
    ///    otherwise under LOW_HEALTH of the most with the level 0: BLINK_LEVEL;
    ///  - the portrait's colour is white, red with `whiteness` while the level is over RED_ABOVE,
    ///    its level then counting down by `dt` to 0; grey for health 0 (0xc452f), whatever else.
    ///
    /// The colour takes the level before this frame's countdown, as the code does (the call to
    /// 0xc4330 at 0xc44c6 comes before the subtraction at 0xc44ea); the channel reads the level
    /// after it (`frame_red`, `push`).
    pub fn update(&mut self, health: f32, max: f32, dt: f32) {
        let health = health.max(0.0);
        if health == 0.0 {
            self.level = 0.0;
        } else if self.last > health {
            self.level = self.level.max(HIT_LEVEL);
        } else if health / max < LOW_HEALTH && self.level == 0.0 {
            self.level = BLINK_LEVEL;
        }
        self.last = health;
        self.face = FaceTint::White;
        if self.level != 0.0 {
            if self.level > RED_ABOVE {
                self.face = FaceTint::Red(whiteness(self.level));
            }
            self.level = (self.level - dt).max(0.0);
        }
        if health == 0.0 {
            self.face = FaceTint::Grey;
        }
    }

    /// Whether the channel's frame is red (0xc33df-0xc33ef: the level over RED_ABOVE, read at the
    /// channel's draw, after the part's update).
    pub fn frame_red(&self) -> bool {
        self.level > RED_ABOVE
    }

    /// The whiteness the channel's fill is pushed by (0xc3449 calls 0xc4330 on the level the same
    /// way).
    pub fn whiteness(&self) -> u8 {
        whiteness(self.level)
    }
}

/// The flash parts of the four slots, by character (the resource play_hud.rs steps each frame,
/// `update_flashes`). A map's start makes it afresh: the parts are attached with a last health of 0
/// and a level of 0.
#[derive(Resource, Default)]
pub struct Flashes(pub [Flash; 4]);

/// FUN_000c4330 (no Ghidra function): the whiteness for a flash level, 128 unless the level is
/// over SWELL_ABOVE, and then 255 - 128 (1 - x^3) with x = 2 (level - 1) (0xc436a-0xc439a): 255 at a
/// hit's level 1.5, falling to 127 at 1.0, steeply at first (143 at a quarter second, 135 at 0.3).
pub fn whiteness(level: f32) -> u8 {
    if level > SWELL_ABOVE {
        let x = (level - SWELL_ABOVE) * 2.0;
        (255 - ((1.0 - x * x * x) * HALF as f32) as i32).clamp(0, 255) as u8
    } else {
        HALF
    }
}

/// A colour pushed toward white by a whiteness a, as the channel's fill is while a hit's flash
/// runs (0xc3449-0xc34df): each of red, green and blue gains `(255 - c) x (a - 128) / 128`, truncated
/// toward zero (`cvttss2si`; 0x3a5238 is the 1/128). Nothing at a = 128; (254, 253, 253) of the
/// fill's (192, 87, 94) at a = 255.
pub fn pushed(rgb: [u8; 3], a: u8) -> [u8; 3] {
    let k = (a as i32 - HALF as i32) as f32 * 0.007_812_5;
    rgb.map(|c| (c as i32 + ((255 - c as i32) as f32 * k) as i32).clamp(0, 255) as u8)
}

/// What the channel's fill shows of a member's health (FUN_000c32c0 into +0x34): 1 minus the
/// health in whole hundredths of its most (`floor(100 x health / max)`, 0x3a4fec and 0x3a4f98), 1.0
/// with no member. The gradient shows its texels at or above it (`fill_shows`).
pub fn empty_fraction(health: f32, max: f32) -> f32 {
    let hundredths = (health * 100.0 / max) as i32;
    1.0 - hundredths as f32 * 0.01
}

/// The empty fraction as the pixel program's constant: the draw hands the program's c0 alpha
/// through FUN_000a4d80, which clamps it to 0..1 and rounds 255 x it to a byte (`cvtss2si`, round
/// to nearest even).
pub fn empty_byte(f: f32) -> u8 {
    (f.clamp(0.0, 1.0) * 255.0).round_ties_even() as u8
}

/// Whether the fill's pixel program (the definition at 0x3e0050, technique 0x22: the sprite draw
/// takes it when its twelfth argument is 1, which the channel's fill draw alone sets) keeps a
/// texel of blue value `blue`: its first combiner forms the texel's blue minus the constant, the
/// next two add 1/2 and a mux on that sum's alpha takes the texel's own alpha when it is at least
/// 1/2 and 0 below it, so a texel shows while its blue is at least the empty byte. Its colour is
/// the sprite's flat colour, not the texture's. (Take01: Hawk's channel shows the half of the
/// gradient at the portrait end at 52 of 100, the third at 32.)
pub fn fill_shows(blue: u8, empty: u8) -> bool {
    blue >= empty
}

/// The sprite draw's tint program (see the module's text) on a texture's RGBA texels (any layout:
/// four bytes a texel, alpha kept): each of the colour's red, green and blue x min(1, r^2 + g^2 +
/// b^2) of the texel's, rounded. A white texel gives the colour itself; a darker one a squared
/// share of it.
pub fn tint_texels(px: &[u8], rgb: [u8; 3]) -> Vec<u8> {
    px.chunks_exact(4).flat_map(|t| {
        let (r, g, b) = (t[0] as f32 / 255.0, t[1] as f32 / 255.0, t[2] as f32 / 255.0);
        let grey = (r * r + g * g + b * b).min(1.0);
        let out = |c: u8| (c as f32 * grey).round() as u8;
        [out(rgb[0]), out(rgb[1]), out(rgb[2]), t[3]]
    }).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Steps a part through `seconds` of frames of `dt` with the member's health `h` (of 100),
    /// returning how long the portrait was red.
    fn red_for(f: &mut Flash, h: f32, seconds: f32, dt: f32) -> f32 {
        let mut red = 0.0;
        for _ in 0..((seconds / dt).round() as usize) {
            f.update(h, 100.0, dt);
            if matches!(f.face, FaceTint::Red(_)) {
                red += dt;
            }
        }
        red
    }

    /// 0xc4330: 128 at rest and through the second half of the flash, then a hit's 255 falling to 127
    /// along (1 - x^3) in the first half second.
    #[test]
    fn whiteness_falls_from_a_hit() {
        assert_eq!(whiteness(0.0), 128);
        assert_eq!(whiteness(0.7), 128);
        assert_eq!(whiteness(1.0), 128);
        assert_eq!(whiteness(1.5), 255);
        assert_eq!(whiteness(1.25), 255 - (0.875f32 * 128.0) as i32 as u8);
        assert!((127..=129).contains(&whiteness(1.001)));
        // monotone down from the hit to the half second
        let mut last = 255;
        for i in 0..=50 {
            let a = whiteness(1.5 - i as f32 * 0.01);
            assert!(a <= last, "{a} after {last}");
            last = a;
        }
    }

    /// A hit: the portrait is red for 1.0 s (the level 1.5 down to 0.5), normal for the last
    /// half second, then the part is quiet (take01: red 17.4-18.4 s, 22.2-23.0 ...).
    #[test]
    fn a_hit_reddens_the_face_for_a_second() {
        let mut f = Flash::default();
        f.update(100.0, 100.0, 0.016);
        assert_eq!(f.face, FaceTint::White);
        f.update(92.0, 100.0, 0.016);
        // the hit's own frame is whiteness 255: white's colour, drawn by the plain program
        assert_eq!(f.face, FaceTint::Red(255));
        let red = red_for(&mut f, 92.0, 2.0, 0.01);
        assert!((red - 0.99).abs() < 0.03, "red for {red}");
        assert_eq!(f.face, FaceTint::White);
        assert_eq!(f.level, 0.0);
    }

    /// Under 20% the flash restarts at 1.0 as it ends: red 0.5 s, normal 0.5 s, over and over, from
    /// the first update (the attach's last health 0 isn't a hit) until healed.
    #[test]
    fn under_a_fifth_it_blinks() {
        let mut f = Flash::default();
        let dt = 0.01;
        let mut seen = Vec::new();
        for i in 0..400 {
            f.update(10.0, 65.0, dt);
            seen.push(matches!(f.face, FaceTint::Red(_)));
            let _ = i;
        }
        // starts red (1.0 > 0.5), 0.5 s each way
        assert!(seen[0] && seen[10] && seen[49]);
        assert!(!seen[55] && !seen[99]);
        assert!(seen[105] && seen[149]);
        assert!(!seen[155]);
        let red = seen.iter().filter(|&&r| r).count() as f32 * dt;
        assert!((red - 2.0).abs() < 0.1, "red for {red} of 4 s");
        // healed over a fifth: the blink finishes its second and stops
        for _ in 0..200 {
            f.update(60.0, 65.0, dt);
        }
        assert_eq!(f.face, FaceTint::White);
        assert_eq!(f.level, 0.0);
        // exactly a fifth isn't under it
        let mut g = Flash::default();
        g.update(13.0, 65.0, dt);
        assert_eq!(g.face, FaceTint::White);
    }

    /// `todo/354-portrait-health-flash` take05: Tex hit from 115 to 15 (13%): the hit's flash runs its
    /// 1.5 s (the level 1.472 on the first row after the hit, 0 at 1.5 s from it: gt 1.562 -> 3.062),
    /// and the update after the level reaches 0 restarts the blink at 1.0 (the next row 0.957); the
    /// face is red 1.0 s, normal 0.5 s, red 0.5 s, normal 0.5 s. A hit that leaves the member
    /// above a fifth (take03: 115 -> 75 -> 35, take04: Hawk 65 -> 45 -> 25) just ends after 1.5 s.
    #[test]
    fn a_hit_runs_into_the_blink_only_under_a_fifth() {
        let dt = 0.03;
        let run = |before: f32, after: f32, max: f32, frames: usize| {
            let mut f = Flash::default();
            f.update(before, max, dt);
            let mut seq = Vec::new();
            for i in 0..frames {
                f.update(after, max, dt);
                seq.push((matches!(f.face, FaceTint::Red(_)), f.level));
                let _ = i;
            }
            seq
        };
        // the hit's first frame: 1.5 less one frame
        let mut f = Flash::default();
        f.update(115.0, 115.0, dt);
        f.update(15.0, 115.0, dt);
        assert!((f.level - 1.47).abs() < 1e-5, "{}", f.level);
        // 13%: red for 33 frames (1.0 s), normal to frame 50, then the blink: red again at once
        let seq = run(115.0, 15.0, 115.0, 130);
        assert!(seq[..32].iter().all(|s| s.0), "red through the first second");
        assert!(seq[34..49].iter().all(|s| !s.0), "normal in the flash's last half second");
        let restart = 40 + seq[40..].iter().position(|s| s.1 > 0.9).expect("the blink restarts");
        assert!((49..53).contains(&restart), "restarts at frame {restart}: 1.5 s after the hit");
        assert!(seq[restart..restart + 14].iter().all(|s| s.0), "red 0.5 s");
        assert!(seq[restart + 18..restart + 31].iter().all(|s| !s.0), "normal 0.5 s");
        // 30% (take03's second hit, 75 -> 35 of 115): nothing restarts
        let seq = run(75.0, 35.0, 115.0, 130);
        assert!(seq[55..].iter().all(|s| !s.0 && s.1 == 0.0), "quiet after the hit's 1.5 s");
        // Hawk's 65 -> 45 -> 25 of 65 (38%): the same
        let seq = run(45.0, 25.0, 65.0, 130);
        assert!(seq[55..].iter().all(|s| !s.0 && s.1 == 0.0));
    }

    /// take01: Hawk's health written from 65 to 10 (15%) is a hit to the part (the level 1.465 on the
    /// first row after the write), and held there the blink restarts at 1.0 every 1.0 s of frames
    /// plus the frame the restart takes (the rows at gt 2.36, 3.40, 4.37, 5.41 ...).
    #[test]
    fn a_held_low_health_restarts_every_second() {
        let dt = 0.025;
        let mut f = Flash::default();
        f.update(65.0, 65.0, dt);
        f.update(10.0, 65.0, dt);
        assert!(f.level > 1.45 && f.level < 1.5);
        let mut restarts = Vec::new();
        let mut was = f.level;
        for i in 0..600 {
            f.update(10.0, 65.0, dt);
            if f.level > was + 0.5 {
                restarts.push(i as f32 * dt);
            }
            was = f.level;
        }
        // 1.5 s after the hit, then every second and a frame (the countdown to 0 takes the frame
        // that reads it, then the update that sees 0 restarts)
        assert!(restarts.len() >= 11, "{restarts:?}");
        for w in restarts.windows(2) {
            assert!((w[1] - w[0] - 1.0).abs() < 2.0 * dt, "period {}", w[1] - w[0]);
        }
    }

    /// A hit during a blink or another hit's flash starts the 1.5 again, never lowers the level.
    #[test]
    fn a_hit_keeps_the_higher_level() {
        let mut f = Flash::default();
        f.update(100.0, 100.0, 0.01);
        f.update(90.0, 100.0, 0.01);
        red_for(&mut f, 90.0, 0.8, 0.01);
        let before = f.level;
        assert!(before < 1.5 && before > 0.5);
        f.update(80.0, 100.0, 0.01);
        assert!(f.level > 1.45, "{}", f.level);
        // health back up isn't a hit
        let mut g = Flash::default();
        g.update(50.0, 100.0, 0.01);
        g.update(80.0, 100.0, 0.01);
        assert_eq!(g.level, 0.0);
    }

    /// Dead (health 0): the level is 0 and the portrait (128, 128, 128).
    #[test]
    fn the_dead_are_grey() {
        let mut f = Flash::default();
        f.update(50.0, 100.0, 0.01);
        f.update(40.0, 100.0, 0.01);
        assert!(f.level > 0.0);
        f.update(0.0, 100.0, 0.01);
        assert_eq!(f.face, FaceTint::Grey);
        assert_eq!(f.level, 0.0);
        f.update(0.0, 100.0, 0.01);
        assert_eq!((f.face, f.level), (FaceTint::Grey, 0.0));
        assert!(!f.frame_red());
    }

    /// The channel reads the level after the countdown: its frame is red while it's over 0.5.
    #[test]
    fn the_frame_follows_the_level() {
        let mut f = Flash::default();
        f.update(100.0, 100.0, 0.01);
        f.update(50.0, 100.0, 0.01);
        assert!(f.frame_red());
        // (the level after the first frame's countdown: 1.49, whiteness 248)
        assert!(f.whiteness() > 240);
        red_for(&mut f, 50.0, 1.2, 0.01);
        assert!(!f.frame_red());
        assert_eq!(f.whiteness(), 128);
    }

    /// 0xc3449-0xc34df on the fill's colour: untouched at 128, (254, 253, 253) at a hit's 255, a
    /// step under 128 truncates toward zero.
    #[test]
    fn the_fill_is_pushed_toward_white() {
        assert_eq!(pushed(FILL_RGB, 128), FILL_RGB);
        assert_eq!(pushed(FILL_RGB, 255), [254, 253, 253]);
        // (half way: 63 / 2 = 31.5 and 161 / 2 = 80.5 truncate)
        assert_eq!(pushed(FILL_RGB, 192), [223, 171, 174]);
        assert_eq!(pushed(FILL_RGB, 127), [192, 86, 93]);
    }

    /// FUN_000c32c0: the health in whole hundredths, take01's Hawk: 52 of 100 and 32 of 100.
    #[test]
    fn the_empty_fraction_is_in_hundredths() {
        assert!((empty_fraction(100.0, 100.0)).abs() < 1e-6);
        assert!((empty_fraction(52.0, 100.0) - 0.48).abs() < 1e-6);
        assert!((empty_fraction(32.0, 100.0) - 0.68).abs() < 1e-6);
        // 52.9 hundredths is 52
        assert!((empty_fraction(52.9, 100.0) - 0.48).abs() < 1e-6);
        assert_eq!(empty_fraction(0.0, 100.0), 1.0);
        assert_eq!(empty_byte(0.0), 0);
        assert_eq!(empty_byte(1.0), 255);
        assert_eq!(empty_byte(0.48), 122);
    }

    /// The thresholded fill: all of the gradient at full health, none of it at none (the brightest
    /// texel's blue is 249), the portrait end first.
    #[test]
    fn the_fill_keeps_the_bright_end() {
        assert!(fill_shows(8, 0) && fill_shows(249, 0));
        assert!(!fill_shows(249, 255));
        assert!(fill_shows(123, 122) && !fill_shows(121, 122) && fill_shows(122, 122));
    }

    /// The tint program: a white texel is the colour, black is black, the alpha stays, and a mid
    /// grey (128) gives 3 x 0.25 = 0.75 of it, not the 0.5 a multiply would.
    #[test]
    fn the_tint_squares_the_texel() {
        // (the icons' colour, bytes c0 d7 ff)
        let out = tint_texels(&[255, 255, 255, 170, 0, 0, 0, 255, 128, 128, 128, 255, 200, 40, 40, 90], [192, 215, 255]);
        assert_eq!(&out[0..4], &[192, 215, 255, 170]);
        assert_eq!(&out[4..8], &[0, 0, 0, 255]);
        let k = 3.0 * (128.0f32 / 255.0).powi(2);
        assert_eq!(out[8], (192.0 * k).round() as u8);
        assert_eq!(out[11], 255);
        // a texel whose r^2 + g^2 + b^2 is over 1 clamps: (200, 40, 40) -> 0.62 + 0.02 + 0.02
        assert_eq!(out[12], (192.0 * (0.6151 + 0.0246 + 0.0246f32)).round() as u8);
        assert_eq!(out[15], 90);
    }

    /// The red face of the footage: Hawk's texture through the program at (255, 128, 128) is far
    /// redder than the same through a multiply (the video's red channel rose where a multiply
    /// leaves it).
    #[test]
    fn red_stays_bright() {
        // a mid blue-grey face texel (127, 140, 167)
        let t = [127u8, 140, 167, 187];
        let o = tint_texels(&t, [255, 128, 128]);
        assert!(o[0] > 220 && o[1] < 130 && o[2] < 130, "{o:?}");
        let multiply = (127.0f32, 140.0 * 0.502, 167.0 * 0.502);
        assert!(o[0] as f32 > multiply.0 * 1.5);
    }

    /// Each channel's rect is inside the radar panel (128 square) and its texture pair is the slot's:
    /// slots 1 and 3 one pair, 2 and 4 the other (FUN_000c3080's two branches).
    #[test]
    fn channels_lie_on_the_radar() {
        for (c, ch) in CHANNELS.iter().enumerate() {
            let (x, y, w, h) = ch.rect;
            let (x0, x1) = (x.min(x + w), x.max(x + w));
            let (y0, y1) = (y.min(y + h), y.max(y + h));
            assert!(x0 >= 0.0 && x1 <= 128.0 && y0 >= 0.0 && y1 <= 128.0, "channel {c}: {:?}", ch.rect);
            assert_eq!((w.abs(), h.abs()), (32.0, 32.0));
        }
        // Brutus (slot 3) and Tex (slot 1), then Flint (2) and Hawk (4)
        assert_eq!(CHANNELS[0].frame, CHANNELS[3].frame);
        assert_eq!(CHANNELS[1].frame, CHANNELS[2].frame);
        assert_ne!(CHANNELS[0].frame, CHANNELS[1].frame);
    }
}
