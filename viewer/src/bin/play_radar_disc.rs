//! The radar's disc, its lit circles and its view wedge (#347): what the radar widget (vtable
//! 0x3a006c, its draw 0xaefb0) lays down before its dots, drawn from the game's own textures with
//! the colours its pixel shader gives them. Written from default.xbe's code (the addresses are in
//! the comments and in viewer/docs/code-map.md, "HUD"); the takes confirm it: six zones' colours of
//! `todo/33-squad-commands` take03 (1.0 s) over its dusk ground come out of these formulas to
//! within 4 levels, 1.6 on average, and `todo/28-tutorial` take07's (5.0 s) to within 7, 3.2 on
//! average, over a ground that varies across the radar (viewer/docs/play.md, "Radar", "The disc,
//! the circles and the wedge"). Before #347 the disc was h_17d0ee37 stretched to 141 units at 0.85
//! alpha, the circle and the wedge one procedural white image at 0.18, none of it the game's.
//!
//! The draw makes three passes over the radar's 128 x 128 quad, each blended as the game's
//! alpha blend (SRC_ALPHA / INV_SRC_ALPHA, set at 0xaf040-0xaf0f6):
//!  1. **The disc**: the widget's own texture (+0x20, h_17d0ee37 by the constructor's FUN_000ba6d0),
//!     white and whole, at its size (+0xc, +0xe: 128 x 128), 1 texel a unit. The sprite draw
//!     0xba950 does it (0xaefb0 -> `call [vtable + 4]`).
//!  2. **The squad's circles** (0xaf1a0, its first part): the radar keeps a list of records (the
//!     vector at +0x5c, 32 bytes each: FUN_000aef00 builds one for the viewer and for each living
//!     member of the viewer's squad and hands it to FUN_000b0fe0), each its point on the radar
//!     (FUN_000b0550: whole units), its colour 255 255 255, its alpha, whether it is the viewer
//!     and its radius, (range - (1 - (1 - e)^2) (range - low)) x the radar's units a metre
//!     (+0x94), the range the character's type's `radar-range` (+0x2a0, 60 m for the four; the
//!     easing e is char+0x57c, taken as 0 as in play_hud.rs `radar_reach`). The first three
//!     records that aren't the viewer's go on texture stages 1 to 3 as the ring texture
//!     (+0x30, h_1fbc4e99: a hard-edged white disc of 41 texels' radius, alpha only) with
//!     coordinates centred on the record's point and 39 / (128 radius) of a texture a unit
//!     (`k`: 1 / 39 x radius is the 0xaf34c scale, the 128 the widget's height), so the disc's
//!     circle comes out 1.05 x the record's radius. Pixel shader 0x24 (the D3DPIXELSHADERDEF at
//!     0x3e0320, set by FUN_000a4d00 with ESI 0x24; its combiners decoded here from the dwords)
//!     makes the pass's colour and alpha: R0.a = min(1, T1.a + T2.a + T3.a) x T0.a x V0.a (the
//!     disc texture's own alpha, the vertex's alpha the widget's: whole) and R1.rgb =
//!     T0.rgb x C0 + C1 with the constants FUN_000a4d80 loads from 0xaf524-0xaf571:
//!     C0 = (0.6, 0.4, 1.0), C1 = (0.4, 0.2, 0.2). So over the disc's teal the members' circles
//!     are lilac, drawn where the disc is, as bright as the disc's own alpha lets them.
//!  3. **The viewer's pass** (0xaf1a0's second part, 0xaf67b-0xaf830): the viewer's record, if
//!     there is one (FUN_000aef00 gives a dead character none), on the same shader with its
//!     constants C0 = 0.4 and C1 = 0.3, grey (0xaf618-0xaf665): stage 1 the ring at the viewer's
//!     point (so its own circle) and stage 2 the wedge texture (+0x38, h_fcebfc70: a triangle,
//!     its apex at the texture's middle, its base across the top, DXT3) sampled with
//!     u = -f + x / 128 (1 + 2 f) across the quad, v from 1 at its top to 0 at its bottom and
//!     the edges clamped (0xaf771-0xaf7d4), f = 0.5 / tan(half the camera's field of view)
//!     (0x40d730 / FUN_0006aa40's tangent: the live, zoomed one). So the wedge's apex is at the
//!     quad's middle, its base 52.7 units across at the quad's top for the follow camera's 70
//!     degrees, and it narrows as the scope zooms. Its colour is T0.rgb x 0.4 + 0.3: a pale grey
//!     over the teal, the same as the viewer's circle, which a lone viewer (the tutorial's Tex)
//!     shows as one shape with it (take07) and a squad's lilac circles lie under.
//!
//! The console blends the stored (sRGB) values; the UI here blends in linear light (the main
//! texture is an sRGB target), where the same layer over a dark ground comes out lighter (the
//! disc's teal about 14 levels in blue over take03's ground). So each layer is drawn with the
//! game's alpha and the colour that gives the game's result over a reference ground, GROUND,
//! solved against what the layers under it make of that ground (`stand_in`): a demo
//! approximation, not the game's, exact over GROUND and within 2 levels of the game's blend over
//! the takes' dark grounds (see GROUND for the figures over others). The game's stored-value
//! blend over the whole HUD is a shared machinery question (play_scope_view.rs draws its passes
//! in those values for the same reason); it isn't done here.
//!
//! The three layers are 128 x 128 images of 1 texel a unit, `ImageNode`s at the radar panel's
//! corner (play_hud.rs `RADAR_FRAME_AT`) set up with the HUD (`layers`); the first is the disc's
//! (static), the other two are redrawn when a record's point (whole units) or the wedge changes.
//! No test hook of its own: `BF_TEST_GOTO` moves the squad round the viewer, the scope's hooks
//! (`BF_TEST_SCOPE`) narrow the wedge.

use super::*;
use super::hud::{radar_point, radar_reach, RADAR_UNITS};

/// The disc texture (the radar widget's +0x20, FUN_000adbc0 via FUN_000ba6d0): 128 x 128 DXT3, a
/// teal ball shaded lighter to the top left, alpha 0.53 inside and 0.6 round its rim, 0 outside
/// the circle (columns 21-107).
const DISC: u32 = 0x17D0_EE37;
/// The ring texture (+0x30): 128 x 128 A8, 1 inside 41 texels from its middle, 0 outside.
const RING: u32 = 0x1FBC_4E99;
/// The wedge texture (+0x38): 128 x 128 DXT3, a triangle from the middle (the apex, 1 texel) to
/// the top rows (the base, 114 texels across), alpha 1, 0 elsewhere.
const WEDGE: u32 = 0xFCEB_FC70;
/// The radar quad's side in units (the widget's +0xc and +0xe, FUN_000adbc0), and in texels of the
/// layers' images.
const SIZE: usize = 128;
/// How many non-viewer records get a circle: the pass has three texture stages for them (0xaf33e
/// compares the count with 3).
const CIRCLES: usize = 3;
/// The squad circles' pass constants (FUN_000a4d80 from the floats at 0xaf524-0xaf571): C0
/// multiplies the disc's colour, C1 is added.
const CIRCLE_C0: [f32; 3] = [0.6, 0.4, 1.0];
const CIRCLE_C1: [f32; 3] = [0.4, 0.2, 0.2];
/// The viewer pass's (0xaf618-0xaf665): grey.
const VIEWER_C0: f32 = 0.4;
const VIEWER_C1: f32 = 0.3;
/// The wedge's width term, 0x40d730 (0.5): f = this over the tangent of half the camera's field
/// of view across (FUN_0006aa40 returns that tangent).
const WEDGE_HALF: f32 = 0.5;
/// The ground the demo's linear-light blend is made to agree with the game's stored-value blend
/// over (`stand_in`), a stored value: the dark ground behind the radar in the takes (take03
/// (27, 22, 32) = 0.10, take07's grass (40, 34, 15) = 0.14). Over those grounds, and the
/// tutorial's grass in the demo (43, 46, 22), the three passes come out within 2 levels of the
/// game's blend (the mean over the disc-only, circle and wedge zones: 0.7-2.0 at this value,
/// 2.4-3.6 at 0.2, 7-9 with no stand-in), over e34 by day (88, 80, 57) and the test map's
/// floor (89, 91, 85) within 4-5 (3.5-4.8 with none: a brighter ground is where it costs). A
/// fit, not the game's.
const GROUND: f32 = 0.15;

/// A texture's texels as stored values 0-1 in display order (the archives store them bottom row
/// first).
struct Tex {
    w: usize,
    h: usize,
    px: Vec<[f32; 4]>,
}

impl Tex {
    /// `rgba` (`Game::texture_rgba`: w x h, bottom row first) in display order.
    fn new(w: usize, h: usize, rgba: &[u8]) -> Tex {
        let px = (0..w * h).map(|i| {
            let (x, y) = (i % w, i / w);
            let at = ((h - 1 - y) * w + x) * 4;
            [0, 1, 2, 3].map(|c| rgba[at + c] as f32 / 255.0)
        }).collect();
        Tex { w, h, px }
    }

    /// The texel (x, y), display order.
    fn at(&self, x: usize, y: usize) -> [f32; 4] {
        self.px[y * self.w + x]
    }

    /// Bilinear at (u, v) in 0-1 with v down the display order, texel centres at (i + 0.5) / size
    /// and the edge texels carried on past the edges (the pass's CLAMP addressing, set at
    /// 0xaf443-0xaf4f9 through FUN_0006a610).
    fn sample(&self, u: f32, v: f32) -> [f32; 4] {
        let fx = (u * self.w as f32 - 0.5).clamp(0.0, (self.w - 1) as f32);
        let fy = (v * self.h as f32 - 0.5).clamp(0.0, (self.h - 1) as f32);
        let (x0, y0) = (fx.floor() as usize, fy.floor() as usize);
        let (x1, y1) = ((x0 + 1).min(self.w - 1), (y0 + 1).min(self.h - 1));
        let (tx, ty) = (fx - x0 as f32, fy - y0 as f32);
        let (a, b, c, d) = (self.at(x0, y0), self.at(x1, y0), self.at(x0, y1), self.at(x1, y1));
        [0, 1, 2, 3].map(|k| (a[k] * (1.0 - tx) + b[k] * tx) * (1.0 - ty) + (c[k] * (1.0 - tx) + d[k] * tx) * ty)
    }
}

/// What the dynamic layers are drawn from, from the radar's state: each circle's centre in units
/// from the quad's top left and its radius in units, the viewer's radius (none: no viewer
/// record, the dead one's) and the wedge's f. Equal inputs draw the same layers, so a frame whose
/// inputs match the last one's leaves the images alone.
#[derive(Clone, Debug, PartialEq)]
struct Inputs {
    circles: Vec<(f32, f32, f32)>,
    viewer: Option<f32>,
    f: f32,
}

/// The radar's three layers (see the module's notes): the textures they're made of, their images
/// (the disc's, the squad circles', the viewer's) and the inputs the last two were drawn from.
#[derive(Resource)]
pub(super) struct RadarLayers {
    disc: Tex,
    ring: Tex,
    wedge: Tex,
    /// the images, in the order they're laid down
    pub(super) images: [Handle<Image>; 3],
    last: Option<Inputs>,
    /// the warnings already given (a non-finite input is dropped, once each)
    warned: bool,
}

/// sRGB's decode (stored value to linear light) and its inverse.
fn lin_exact(c: f32) -> f32 {
    if c <= 0.04045 { c / 12.92 } else { ((c + 0.055) / 1.055).powf(2.4) }
}

fn enc_exact(c: f32) -> f32 {
    if c <= 0.003_130_8 { c * 12.92 } else { 1.055 * c.powf(1.0 / 2.4) - 0.055 }
}

/// How finely the tables of `lin` and `enc` step over 0-1 (the interpolation's error is under
/// 2e-5: the curves' second derivative times step^2 / 8, at their worst just above the toes).
const LIN_STEPS: usize = 1024;
const ENC_STEPS: usize = 4096;

/// The curves tabled, made on first use: a redraw of the layers needs about 30 thousand of them
/// (`powf` made it 3 ms for three circles and the viewer's pass; the tables 0.9).
struct Curves {
    lin: Vec<f32>,
    enc: Vec<f32>,
}

fn curves() -> &'static Curves {
    static CURVES: std::sync::OnceLock<Curves> = std::sync::OnceLock::new();
    CURVES.get_or_init(|| Curves {
        lin: (0..=LIN_STEPS).map(|i| lin_exact(i as f32 / LIN_STEPS as f32)).collect(),
        enc: (0..=ENC_STEPS).map(|i| enc_exact(i as f32 / ENC_STEPS as f32)).collect(),
    })
}

/// A table's value at `c` (0-1), linearly interpolated between its entries.
fn lookup(table: &[f32], c: f32) -> f32 {
    let steps = table.len() - 1;
    let t = c.clamp(0.0, 1.0) * steps as f32;
    let i = (t as usize).min(steps - 1);
    table[i] + (table[i + 1] - table[i]) * (t - i as f32)
}

/// A stored value as linear light.
fn lin(c: f32) -> f32 {
    lookup(&curves().lin, c)
}

/// Linear light as a stored value (clamped to 0-1; the toe, where the curve is a straight line,
/// exact).
fn enc(c: f32) -> f32 {
    let c = c.clamp(0.0, 1.0);
    if c <= 0.003_130_8 { c * 12.92 } else { lookup(&curves().enc, c) }
}

/// The game's blend (SRC_ALPHA / INV_SRC_ALPHA on the stored values) of `rgb` at alpha `a` over
/// `ground`.
fn blend(ground: [f32; 3], rgb: [f32; 3], a: f32) -> [f32; 3] {
    [0, 1, 2].map(|c| ground[c] * (1.0 - a) + rgb[c] * a)
}

/// A layer of the game's, `rgb` (stored values) at alpha `a`, laid over `ground` (what the
/// layers under it make of GROUND, per channel), put as the layer that the UI's linear-light
/// blend turns into the same result over that ground as the console's blend of stored values:
/// the alpha is kept and the colour solved for, per channel, clamped to what a colour can be
/// (the disc's dark red can't go lower than 0). Exact at GROUND; over a brighter or darker
/// ground the two blends part (see the module's notes): a demo approximation.
fn stand_in(rgb: [f32; 3], a: f32, ground: [f32; 3]) -> [f32; 3] {
    if a <= 1e-4 {
        return rgb;
    }
    let want = blend(ground, rgb, a);
    [0, 1, 2].map(|c| enc((lin(want[c]) - lin(ground[c]) * (1.0 - a)) / a))
}

impl RadarLayers {
    /// One texel of a pass: the game's colour and alpha put through `stand_in` over `ground`, as
    /// the image's bytes (an sRGB image: the colour is stored encoded, the alpha straight).
    fn put(out: &mut [u8], i: usize, rgb: [f32; 3], a: f32, ground: [f32; 3]) {
        let rgb = stand_in(rgb, a, ground);
        let byte = |v: f32| (v.clamp(0.0, 1.0) * 255.0 + 0.5) as u8;
        out[i * 4..i * 4 + 4].copy_from_slice(&[byte(rgb[0]), byte(rgb[1]), byte(rgb[2]), byte(a)]);
    }

    /// The ring's alpha over the quad's pixel centre (px, py) for a circle at (cx, cy) of the
    /// record radius `r` (all in units): the coordinates (px - cx) x 39 / (128 r) round 0.5.
    fn ring_alpha(&self, (px, py): (f32, f32), (cx, cy, r): (f32, f32, f32)) -> f32 {
        let k = RADAR_UNITS / (SIZE as f32 * r);
        self.ring.sample(0.5 + (px - cx) * k, 0.5 + (py - cy) * k)[3]
    }

    /// The disc layer: the disc texture itself, texel for texel.
    fn disc_layer(&self) -> Vec<u8> {
        let mut out = vec![0u8; SIZE * SIZE * 4];
        for i in 0..SIZE * SIZE {
            let d = self.disc.px[i];
            Self::put(&mut out, i, [d[0], d[1], d[2]], d[3], [GROUND; 3]);
        }
        out
    }

    /// The squad circles' layer and the viewer's, for `inputs` (see the module's notes, passes 2
    /// and 3), as RGBA bytes.
    fn dynamic_layers(&self, inputs: &Inputs) -> (Vec<u8>, Vec<u8>) {
        let mut circles = vec![0u8; SIZE * SIZE * 4];
        let mut viewer = vec![0u8; SIZE * SIZE * 4];
        // the viewer's circle is the radar's middle
        let middle = inputs.viewer.map(|r| (SIZE as f32 / 2.0, SIZE as f32 / 2.0, r));
        for i in 0..SIZE * SIZE {
            let d = self.disc.px[i];
            if d[3] <= 0.0 {
                continue;
            }
            let p = ((i % SIZE) as f32 + 0.5, (i / SIZE) as f32 + 0.5);
            // (each pass is solved against what the passes under it make of the reference ground)
            let mut ground = blend([GROUND; 3], [d[0], d[1], d[2]], d[3]);
            let sum: f32 = inputs.circles.iter().map(|&c| self.ring_alpha(p, c)).sum();
            if sum > 0.0 {
                let rgb = [0, 1, 2].map(|c| (d[c] * CIRCLE_C0[c] + CIRCLE_C1[c]).clamp(0.0, 1.0));
                let alpha = sum.min(1.0) * d[3];
                Self::put(&mut circles, i, rgb, alpha, ground);
                ground = blend(ground, rgb, alpha);
            }
            if let Some(m) = middle {
                let ring = self.ring_alpha(p, m);
                // (u across the quad, v the display row over the quad's height: 0 at its top)
                let u = -inputs.f + p.0 / SIZE as f32 * (1.0 + 2.0 * inputs.f);
                let wedge = self.wedge.sample(u, p.1 / SIZE as f32)[3];
                let alpha = (ring + wedge).min(1.0) * d[3];
                if alpha > 0.0 {
                    let rgb = [0, 1, 2].map(|c| (d[c] * VIEWER_C0 + VIEWER_C1).clamp(0.0, 1.0));
                    Self::put(&mut viewer, i, rgb, alpha, ground);
                }
            }
        }
        (circles, viewer)
    }
}

/// An `Image` of the radar's quad from RGBA bytes.
fn quad_image(data: Vec<u8>) -> Image {
    Image::new(bevy::render::render_resource::Extent3d { width: SIZE as u32, height: SIZE as u32, depth_or_array_layers: 1 },
               bevy::render::render_resource::TextureDimension::D2, data,
               bevy::render::render_resource::TextureFormat::Rgba8UnormSrgb, bevy::asset::RenderAssetUsages::default())
}

/// The radar's three layers for a map's HUD (`setup_hud`): the textures decoded and the three
/// images made (the disc's drawn at once, the others empty until `update`). None where a texture
/// isn't found (the viewer's own copy of the game data lacks common.tgz's).
pub(super) fn layers(game: &mut Game, images: &mut Assets<Image>) -> Option<RadarLayers> {
    let mut tex = |name: u32| {
        let found = game.texture_rgba(name).map(|(w, h, px)| Tex::new(w as usize, h as usize, &px));
        if found.is_none() {
            warn!("radar texture h_{name:08x} not found: the radar's disc is left out");
        }
        found
    };
    let (disc, ring, wedge) = (tex(DISC)?, tex(RING)?, tex(WEDGE)?);
    // (the layers' maths takes the three at the quad's size, as the game's textures are)
    if [&disc, &ring, &wedge].iter().any(|t| t.w != SIZE || t.h != SIZE) {
        warn!("radar textures aren't {SIZE} x {SIZE}: the radar's disc is left out");
        return None;
    }
    let mut layers = RadarLayers {
        disc, ring, wedge,
        images: [Handle::default(), Handle::default(), Handle::default()],
        last: None,
        warned: false,
    };
    let empty = || quad_image(vec![0; SIZE * SIZE * 4]);
    layers.images = [images.add(quad_image(layers.disc_layer())), images.add(empty()), images.add(empty())];
    Some(layers)
}

/// The radar's records for this frame (FUN_000ae580 -> FUN_000ae7c0 -> FUN_000aef00): the squad's
/// living members within the viewer's reach, at their points on the radar (whole units, FUN_000b0550;
/// one beyond the max-range on the circle, FUN_000b0ad0), the first CIRCLES of them, with their
/// radii; and the viewer's own. The squad is the demo's `in_squad` members, the ones play_hud.rs
/// draws blips for. None of it when a value isn't finite: the record is left out.
fn inputs(player: &Player, squad: &Squad, game: &Game) -> Inputs {
    let max_range = game.radar.max_range.max(1.0);
    let scale = RADAR_UNITS / max_range;
    let fwd = Vec3::new(-player.cam_yaw.sin(), 0.0, -player.cam_yaw.cos());
    let range = |p: &Player| game.character_type(p.kind).and_then(|t| t.skill.as_ref()).map(|s| s.radar_range)
        .filter(|r| r.is_finite() && *r > 0.0);
    let half = SIZE as f32 / 2.0;
    let mut circles = vec![];
    for m in squad.0.iter().filter(|m| m.in_squad() && !m.dead && !m.absent && !m.stand_in) {
        let rel = Vec3::new(m.position.x - player.position.x, 0.0, m.position.z - player.position.z);
        if !rel.is_finite() || !fwd.is_finite() {
            continue;
        }
        // (FUN_000ae7c0: beyond the max-range the record is on the circle; within it, only within
        // the viewer's reach, FUN_000b0670)
        let (at, inside) = if rel.length() < max_range { (rel, true) } else { (rel.normalize_or_zero() * max_range, false) };
        if inside && radar_reach(player, game, m.position, max_range) <= 0.0 {
            continue;
        }
        let Some(r) = range(m) else { continue };
        let p = radar_point(at, fwd, scale);
        circles.push((half + p.x, half + p.y, r * scale));
        if circles.len() == CIRCLES {
            break;
        }
    }
    // the viewer: none while it's dead (FUN_000aef00 returns for a dead character: no record, so
    // no wedge and no circle of its own)
    let viewer = if player.dead || player.absent { None } else { range(player).map(|r| r * scale) };
    // f = 0.5 / tan(half the field of view across), the zoomed one
    let tan = super::targeting::GAME_TAN_ACROSS / player.zoom.max(1.0);
    Inputs { circles, viewer, f: WEDGE_HALF / tan }
}

/// Redraws the squad circles' and the viewer's layers when their inputs changed.
fn update(player: Res<Player>, squad: Res<Squad>, game: Res<GameData>, layers: Option<ResMut<RadarLayers>>, mut images: ResMut<Assets<Image>>) {
    let Some(mut layers) = layers else { return };
    let now = inputs(&player, &squad, &game.0);
    // (a non-finite radius or wedge would make the layers' texels NaN: drawn as none)
    if !now.f.is_finite() || now.circles.iter().any(|c| !(c.0.is_finite() && c.1.is_finite() && c.2.is_finite() && c.2 > 0.0))
        || now.viewer.is_some_and(|r| !(r.is_finite() && r > 0.0)) {
        if !layers.warned {
            layers.warned = true;
            warn!("radar disc: a non-finite or empty input ({now:?}) left the circles and wedge undrawn");
        }
        return;
    }
    if layers.last.as_ref() == Some(&now) {
        return;
    }
    let (circles, viewer) = layers.dynamic_layers(&now);
    for (handle, data) in [(&layers.images[1], circles), (&layers.images[2], viewer)] {
        if let Some(image) = images.get_mut(handle) {
            image.data = Some(data);
        }
    }
    layers.last = Some(now);
}

pub fn plugin(app: &mut App) {
    // (once the characters have moved and the camera has: before the radar's dots, which draw over it)
    app.add_systems(Update, update.after(update_player).run_if(in_state(AppState::Playing)));
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Synthetic textures at the game's sizes: a disc of alpha 0.5 and colour (0.1, 0.4, 0.6)
    /// inside 43 texels of the middle, a ring of 41 texels' radius, a wedge from the middle to
    /// the top (half as wide as it's high in texels, base on row 6).
    fn layers() -> RadarLayers {
        let n = SIZE;
        let rgba = |f: &dyn Fn(f32, f32) -> [f32; 4]| -> Vec<u8> {
            // (stored bottom row first, as the archives)
            (0..n * n).flat_map(|i| {
                let (x, y) = (i % n, n - 1 - i / n);
                f(x as f32 + 0.5 - 64.0, y as f32 + 0.5 - 64.0).map(|v| (v * 255.0 + 0.5) as u8)
            }).collect()
        };
        let disc = rgba(&|x, y| if x.hypot(y) <= 43.0 { [0.1, 0.4, 0.6, 0.5] } else { [0.1, 0.4, 0.6, 0.0] });
        let ring = rgba(&|x, y| [1.0, 1.0, 1.0, if x.hypot(y) <= 41.0 { 1.0 } else { 0.0 }]);
        // (display y runs down: the base at the top is y < 0 after the flip above)
        let wedge = rgba(&|x, y| [1.0, 1.0, 1.0, if y < 0.0 && y > -58.0 && x.abs() <= -y { 1.0 } else { 0.0 }]);
        RadarLayers {
            disc: Tex::new(n, n, &disc), ring: Tex::new(n, n, &ring), wedge: Tex::new(n, n, &wedge),
            images: Default::default(), last: None, warned: false,
        }
    }

    /// A texel's bytes (an sRGB image's colour, a straight alpha) laid over `ground` (stored
    /// values) the way the UI blends them, in linear light: the stored result.
    fn ui_blend(ground: [f32; 3], px: &[u8]) -> [f32; 3] {
        let a = px[3] as f32 / 255.0;
        [0, 1, 2].map(|c| enc(lin(ground[c]) * (1.0 - a) + lin(px[c] as f32 / 255.0) * a))
    }

    /// The stand-in over its own ground is the game's blend of stored values: the linear-light
    /// blend of (colour', alpha) over a ground equals ground (1 - a) + colour a, as stored
    /// values, in every channel the colour range can reach.
    #[test]
    fn the_stand_in_agrees_at_the_ground() {
        for (rgb, a, ground) in [([0.06, 0.3, 0.48], 0.53, [GROUND; 3]), ([0.7, 0.5, 0.2], 0.2, [GROUND; 3]), ([0.9, 0.9, 0.9], 0.6, [0.3, 0.2, 0.1]),
                                 ([0.1, 0.1, 0.4], 0.05, [GROUND; 3])] {
            let c = stand_in(rgb, a, ground);
            let game = blend(ground, rgb, a);
            for k in 0..3 {
                let ours = enc(lin(ground[k]) * (1.0 - a) + lin(c[k]) * a);
                // (a channel the colour range can't reach is clamped: only the ones that fit are exact)
                if c[k] > 0.001 && c[k] < 0.999 {
                    assert!((game[k] - ours).abs() < 2e-3, "channel {k}: game {} ours {ours}", game[k]);
                }
            }
        }
        // no alpha, no change
        assert_eq!(stand_in([0.2, 0.4, 0.6], 0.0, [GROUND; 3]), [0.2, 0.4, 0.6]);
    }

    /// The three layers stacked over GROUND give the game's three passes over it (to the bytes'
    /// resolution), where the colours aren't clamped: the disc, the squad's circles and the
    /// viewer's pass at a texel under all three.
    #[test]
    fn the_stack_agrees_at_the_ground() {
        let l = layers();
        let (circles, viewer) = l.dynamic_layers(&Inputs { circles: vec![(64.0, 64.0, 19.5)], viewer: Some(19.5), f: 0.7 });
        let disc = l.disc_layer();
        let at = (70 * SIZE + 62) * 4;
        let ground = [GROUND; 3];
        let a = 128.0 / 255.0;
        let d = [0.1_f32, 0.4, 0.6];
        let kc = [0, 1, 2].map(|c| (d[c] * CIRCLE_C0[c] + CIRCLE_C1[c]).clamp(0.0, 1.0));
        let kw = [0, 1, 2].map(|c| d[c] * VIEWER_C0 + VIEWER_C1);
        let game = blend(blend(blend(ground, d, a), kc, a), kw, a);
        let ours = ui_blend(ui_blend(ui_blend(ground, &disc[at..at + 4]), &circles[at..at + 4]), &viewer[at..at + 4]);
        for k in 0..3 {
            assert!((game[k] - ours[k]).abs() < 0.012, "channel {k}: game {} ours {}", game[k], ours[k]);
        }
    }

    /// A lone viewer: its circle (1.05 times its radius) and its wedge are one shape, both of the
    /// pale grey; nothing outside the disc.
    #[test]
    fn a_lone_viewer_has_circle_and_wedge() {
        let l = layers();
        // 19.5 units' radius: the ring texture 64 units across, its disc 20.5 units in radius
        let (circles, viewer) = l.dynamic_layers(&Inputs { circles: vec![], viewer: Some(19.5), f: 0.5 / 0.7002 });
        let alpha = |layer: &[u8], x: usize, y: usize| layer[(y * SIZE + x) * 4 + 3];
        // the squad layer is empty
        assert!(circles.iter().skip(3).step_by(4).all(|&a| a == 0));
        // the middle: the disc's alpha 0.5 (128), the circle's 20 units out: also; 22 out: none
        assert_eq!(alpha(&viewer, 64, 64), 128);
        assert_eq!(alpha(&viewer, 64, 64 + 19), 128);
        assert_eq!(alpha(&viewer, 64 + 22, 64), 0);
        // the wedge: 40 units above the middle, 15 across is inside (half-angle 22 degrees: 16.5
        // units), 20 across is outside
        assert_eq!(alpha(&viewer, 64 + 15, 64 - 40), 128);
        assert_eq!(alpha(&viewer, 64 + 20, 64 - 40), 0);
        // and none below the middle, outside the circle
        assert_eq!(alpha(&viewer, 64, 64 + 30), 0);
        // the colour: the disc's (0.1, 0.4, 0.6) x 0.4 + 0.3 = (0.34, 0.46, 0.54), before the stand-in,
        // over the disc laid on GROUND
        let a = 128.0 / 255.0;
        let want = stand_in([0.34, 0.46, 0.54], a, blend([GROUND; 3], [0.1, 0.4, 0.6], a));
        let at = (64 * SIZE + 64) * 4;
        let byte = |v: f32| (v * 255.0 + 0.5) as u8;
        for k in 0..3 {
            assert!((viewer[at + k] as i32 - byte(want[k]) as i32).abs() <= 1, "channel {k}");
        }
    }

    /// The squad's circles are lilac (the disc x (0.6, 0.4, 1.0) + (0.4, 0.2, 0.2)), where the
    /// ring covers, the alpha of three overlapping ones capped at the disc's.
    #[test]
    fn the_squads_circles_add_up_to_the_discs_alpha() {
        let l = layers();
        let at = |x: f32| (64.0 + x, 64.0, 19.5);
        let (circles, _) = l.dynamic_layers(&Inputs { circles: vec![at(-3.0), at(0.0), at(3.0)], viewer: None, f: 0.7 });
        let px = |x: usize, y: usize| &circles[(y * SIZE + x) * 4..(y * SIZE + x) * 4 + 4];
        // three overlap at the middle: the alpha is the disc's 0.5, not 1.5 of it
        assert_eq!(px(64, 64)[3], 128);
        // the colour before the stand-in: (0.1 x 0.6 + 0.4, 0.4 x 0.4 + 0.2, 0.6 + 0.2) = (0.46, 0.36, 0.8),
        // over the disc laid on GROUND
        let a = 128.0 / 255.0;
        let want = stand_in([0.46, 0.36, 0.8], a, blend([GROUND; 3], [0.1, 0.4, 0.6], a));
        assert!((px(64, 64)[2] as i32 - (want[2] * 255.0 + 0.5) as i32).abs() <= 1);
        // the union reaches the right-hand circle's edge, 3 + 20.5 = 23.5 units out: inside at 22,
        // outside at 25
        assert_eq!(px(64 + 22, 64)[3], 128);
        assert_eq!(px(64 + 25, 64)[3], 0);
        // no viewer: no viewer layer
        let (_, none) = l.dynamic_layers(&Inputs { circles: vec![], viewer: None, f: 0.7 });
        assert!(none.iter().all(|&b| b == 0));
    }

    /// A narrower view (the scope's zoom) narrows the wedge: f grows as the tangent falls.
    #[test]
    fn the_scope_narrows_the_wedge() {
        let l = layers();
        let wide = l.dynamic_layers(&Inputs { circles: vec![], viewer: Some(19.5), f: 0.5 / 0.7002 }).1;
        let narrow = l.dynamic_layers(&Inputs { circles: vec![], viewer: Some(19.5), f: 0.5 / 0.35 }).1;
        let count = |v: &[u8]| v.iter().skip(3).step_by(4).filter(|&&a| a > 0).count();
        assert!(count(&narrow) < count(&wide));
    }

    /// The tabled curves are the exact ones to a thousandth of a level's worth.
    #[test]
    fn the_tabled_curves_are_exact() {
        for i in 0..=1000 {
            let c = i as f32 / 1000.0;
            assert!((lin(c) - lin_exact(c)).abs() < 2e-5, "lin({c})");
            assert!((enc(c) - enc_exact(c)).abs() < 2e-5, "enc({c})");
        }
    }

    /// The inputs of a viewer with no squad and one with a dead viewer: a lone viewer's circle only.
    #[test]
    fn equal_inputs_compare_equal() {
        let a = Inputs { circles: vec![(60.0, 64.0, 6.3)], viewer: Some(19.5), f: 0.71 };
        assert_eq!(a, a.clone());
        let mut b = a.clone();
        b.circles[0].0 += 1.0;
        assert_ne!(a, b);
    }
}
