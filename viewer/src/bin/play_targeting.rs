//! Flint's targeting marker (#124): the HUD widget her Advanced Targeting shows, h_121cf7bb (the
//! data's h_f192e10e), which her skill's +0xc4 has the HUD factory make for the controlled
//! character (0x13cd30 -> FUN_000b9120 builds it with FUN_000b45c0, vtable 0x39fe08;
//! FUN_000b46a0 adds its two corners, h_f7aa2d9f and h_09ec2dd6). Read from default.xbe (every
//! function cited is in docs/code-map.md), checked against #35's takes 15-17, 28, 29 and 31
//! (todo/35-special-skills, spec Part 5). The skill itself is play_skills.rs's `Targeting`.
//!
//! Each frame (the skill's +0xc4, run for the controlled character by the target selection,
//! 0xbd0c6; the marker's update 0xb4730; its corners' 0xb4ee0):
//!  - **On and the lock** (0x13cd30): the marker's on flag (+0x84 bit 0) is the skill's, its
//!    target (+0x68 / +0x6c) the crosshair's lock (the reticle controller's +0xa0,
//!    play_reticle.rs `Reticle::lock`). While on, the lock box is the skill's 0.4 of the screen.
//!  - **Fill** (+0x7c): up at 1.6 a second while on (0.625 s; the takes' logs 0.56-0.61 s), down
//!    at 2 while off (0.5 s; 0.46-0.57); the marker is hidden at 0.
//!  - **Pulse** (+0x80, +0x84 bit 1): while on, a ramp up to 2 at 1 a second, then down, the
//!    flag's bit 1 set on the way down (the takes' log: it toggled every 2.06 s); the alpha is
//!    the fill x (1 - 0.16 x the ramp), x 255 into a byte (0xb4891-0xb48c5).
//!  - **Where**: at the crosshair's centre (FUN_000ab390: (320, 192), (320, 240) scoped). With a
//!    lock and on, the target's bounding sphere (FUN_00022bc0; `Character::bounds`) projected: its
//!    centre's offset from the crosshair (+0x70 / +0x74) and a size, 640 x its radius / its depth x
//!    the zoom (+0x78), in the game's 70-degree view (`GAME_TAN_ACROSS`).
//!  - **The corners** (the top left, vtable 0x39fb28, and the bottom right, 0x39fab0; both the
//!    64 x 64 texture h_e1b45a4c, the bottom right drawn at -64 x -64 from its point: turned half
//!    a turn) each head for a point (0xb49e0 / 0xb4bf0):
//!    - with a lock, the target's box: its centre -/+ (max(0.5 x size, 32), max(0.75 x size, 32)),
//!      kept inside the HUD's safe area (`Game::hud_safe`, 3 units further in: FUN_000b2770 and
//!      its three neighbours). A lock while off (the crosshair's own) gives a size of 0: a 64 x 64
//!      box at the crosshair, as the marker fades;
//!    - on without one, the wide box: -/+ (0.4 x 640 / 2, 0.4 x 480 / 2) = (128, 96), the skill's
//!      box shares (its +0x24 / +0x28);
//!    - off: (-256, -96) and (384, 96): toward the screen's left edge, past its right one. So
//!      switched on they slide in from the sides (take15 2.5 s: the left one in place, the right
//!      one still 120 units out).
//!
//!    They move at 450 units a second inward and 900 outward across, 400 up and down (0xb4f20;
//!    the corners' +0x40 / +0x44 / +0x48, set at 0xb51e0 / 0xb5340): the takes' wide to tight in
//!    0.25-0.3 s, tight to wide in 0.1-0.15 s. (The game moves them whole units a frame; the demo
//!    by the frame's time.)
//!  - **The strips**, with a lock only: over the top left corner the eight orange glyphs of
//!    h_0f00136e (64 x 8 at (-28, -6)) are the target's health, a glyph for each 15 hit points
//!    rounded up, at most 8 (0xb5140, the rest clipped off the strip's left, 0xbac46): take15's
//!    999-hp Shrike shows all eight. Under the bottom right one a third of the green wave
//!    h_0e3d0ad2 (48 x 8 at (-36, -2)) scrolls at a quarter of the texture a second (0xb5310).
//!  - **A doubled roll** (the skill's flag 4, `skills::Skill::double_hit`) draws the corners in
//!    (255, 0, 0) for a frame while there's a lock (0xb50e3): the cyan texture x red, nearly black.
//!
//! The game draws all four additively (blend mode 1, 0xbaa24); the demo blends them over, the
//! texels' and the fade's alpha raised to SRGB_ALPHA: the same over the takes' dark ground, a
//! little dimmer over a light one. The marker is the controlled character's: it shows only
//! while that character's skill has one (Flint's), and starts over (off, empty) when control
//! passes, the demo's choice (the game keeps it with her skill, +0x4c, but only the controlled
//! character's +0xc4 runs).
//!
//! BF_TARGETING_LOG=1 prints the marker's state when it changes (on, the lock, the fill at 0 or
//! 1, where the corners head), =2 every frame.

use super::*;
use super::reticle::{crosshair_centre, View};

/// The marker's textures, by name (FUN_000a7770 at 0xb4e41, 0xb5238, 0xb539b): the corner (64 x
/// 64, a cyan bracket corner with its bend ~10 texels in from its top left, drawn 1:1), the
/// glyph strip (64 x 8, eight orange glyphs), the wave strip (128 x 8, green).
const CORNER_TEX: u32 = 0xE1B4_5A4C;
const GLYPH_TEX: u32 = 0x0F00_136E;
const WAVE_TEX: u32 = 0x0E3D_0AD2;
/// The HUD's screen: the view's width and height (the marker's view, +0x34's +0x20 / +0x22).
const SCREEN: Vec2 = Vec2::new(640.0, 480.0);
/// The fill (+0x7c): up at FILL_UP a second while on (0x3a5088, 0xb476b), down at FILL_DOWN while
/// off (twice the frame's time, 0xb4845).
const FILL_UP: f32 = 1.6;
const FILL_DOWN: f32 = 2.0;
/// The pulse (+0x80): up to PULSE_TOP (0x3a5008) and back down, at 1 a second, while on; the
/// alpha loses PULSE_DIM of the fill for each unit of it (0x3a5390, 0xb48a9).
const PULSE_TOP: f32 = 2.0;
const PULSE_DIM: f32 = 0.16;
/// The tight box's half-size: the target's size x TIGHT_X across (0x3a4f74, 0xb49fe) and x
/// TIGHT_Y down (0x3a510c, 0xb4a3a), at least TIGHT_MIN each (0x42000000 at 0xb4a05 / 0xb4a40).
const TIGHT_X: f32 = 0.5;
const TIGHT_Y: f32 = 0.75;
const TIGHT_MIN: f32 = 32.0;
/// The target's size on the HUD: SIZE_SCALE (0x3a5180, 0xb498e) x its bounding sphere's radius /
/// its depth x the zoom (FUN_0006c6f0: the unzoomed view's tangent over the zoomed one's).
const SIZE_SCALE: f32 = 640.0;
/// That size is in the game's view, its follow camera's 70 degrees across (tan 35 degrees, as
/// ale_fx.rs's LOD_TAN): twice the sphere's projected radius x this, zoomed or not. The size is
/// taken as that share of the sphere's projection in the demo's view (play.rs FOV_Y), which since
/// #269 is the game's own (55.4 degrees down: the share is 1), so the box keeps its place round the
/// body as in the takes (take15 3.0 s: the Shrike Heavy, its sphere 1.24 m, 7 m from Flint in a box
/// 105 x 162 units). play_stealth.rs scales Hawk's refraction by it the same way.
pub(super) const GAME_TAN_ACROSS: f32 = 0.700_207_5;
/// Switched off, the corners head for the screen's middle less (the top left) or plus (the
/// bottom right) OFF_X across (0x3a5290: 0xb4b7f, 0xb4d8d).
const OFF_X: f32 = 64.0;
/// The corners' speeds, units a second: across inward and outward (the top left's +0x48 / +0x44,
/// the bottom right's +0x44 / +0x48: 0x43e10000 and 0x44610000), up and down (+0x40,
/// 0x43c80000).
const SLIDE_IN: f32 = 450.0;
const SLIDE_OUT: f32 = 900.0;
const SLIDE_Y: f32 = 400.0;
/// The corner texture's size on the HUD (+0x4c / +0x50: 64 the top left, 0xb51ea; -64 the bottom
/// right, 0xb534a).
const CORNER: f32 = 64.0;
/// The glyph strip: 64 x 8 at GLYPH_AT from the top left corner's point (0xb52bb-0xb52da); a
/// glyph for each GLYPH_HP of the target's hit points (+0x60 15), rounded up (FUN_002eac34, the
/// CRT's ceil: its control word 0x1b3f rounds up), at most GLYPHS (+0x5c), each GLYPH_W wide
/// (+0x58): the hidden ones' width is clipped off the strip's left.
const GLYPH_AT: Vec2 = Vec2::new(-28.0, -6.0);
const GLYPH_STRIP: Vec2 = Vec2::new(64.0, 8.0);
const GLYPH_HP: f32 = 15.0;
const GLYPHS: f32 = 8.0;
const GLYPH_W: f32 = 8.0;
/// The wave strip: WAVE_SIZE at WAVE_AT from the bottom right corner's point (0xb540f-0xb542f),
/// WAVE_SHOWN of the texture across (+0x60 0.33, 0x3ea8f5c3), scrolling at WAVE_SPEED of it a
/// second (+0x5c 0.25), wrapped at WAVE_WRAP (+0x58 3).
const WAVE_AT: Vec2 = Vec2::new(-36.0, -2.0);
const WAVE_SIZE: Vec2 = Vec2::new(48.0, 8.0);
/// The wave texture's width in texels (h_0e3d0ad2 is 128 x 8).
const WAVE_TEX_W: f32 = 128.0;
const WAVE_SHOWN: f32 = 0.33;
const WAVE_SPEED: f32 = 0.25;
const WAVE_WRAP: f32 = 3.0;
/// The HUD's safe area (FUN_000b2870 left, 0xb2800 top, FUN_000b28e0 right, FUN_000b2770 bottom):
/// the game-options' margins (`Game::hud_safe`) SAFE_IN further in (3.0 at 0x39a848), the right
/// and bottom ones measured from SAFE_RIGHT / SAFE_BOTTOM (0x283, 0x1de): x 51-592, y 39-439.
const SAFE_IN: f32 = 3.0;
const SAFE_RIGHT: f32 = 643.0;
const SAFE_BOTTOM: f32 = 478.0;
/// The console blends in sRGB values, Bevy's UI in linear light: the texels' alpha and the
/// marker's are raised to this, so they look over a dark ground as the game's (the skill label's
/// fade does the same, play_hud.rs).
const SRGB_ALPHA: f32 = 2.2;

/// The marker's state, the controlled character's (see the module's notes; the game's widget
/// fields in brackets).
#[derive(Resource, Default)]
pub struct Marker {
    /// the character whose skill it's for (None: no marker shown)
    owner: Option<Who>,
    /// its skill is on (+0x84 bit 0)
    on: bool,
    /// how far it's filled in, 0-1 (+0x7c): play_reticle.rs fades the crosshair by it
    pub fill: f32,
    /// the pulse and its way (+0x80, +0x84 bit 1)
    pulse: f32,
    pulse_down: bool,
    /// the crosshair's lock (+0x6c)
    lock: Option<Who>,
    /// the locked target's centre's offset from the crosshair and its size, while on (+0x70 /
    /// +0x74, +0x78)
    target: (Vec2, f32),
    /// where it sits (the crosshair's centre, +8 / +0xa) and its alpha (+0x10 / 255)
    place: Vec2,
    alpha: f32,
    /// the corners' offsets from it (+4 / +6): the top left, the bottom right
    corners: [Vec2; 2],
    /// the glyphs shown (the target's health), the wave's scroll (the bottom right's +0x38)
    glyphs: f32,
    wave: f32,
    /// a doubled roll this frame (the corners' +0x54 bit 0)
    red: bool,
    /// what BF_TARGETING_LOG printed last
    logged: String,
}

impl Marker {
    /// A new marker for `owner`, off and empty, its corners where off sends them.
    fn new(owner: Option<Who>, shares: [f32; 2]) -> Self {
        let mut m = Marker { owner, ..default() };
        m.corners = [Self::off_point(0, shares), Self::off_point(1, shares)];
        m
    }

    /// Where corner `k` heads while off (0xb4b6a / 0xb4d78).
    fn off_point(k: usize, shares: [f32; 2]) -> Vec2 {
        let y = shares[1] * SCREEN.y * 0.5;
        if k == 0 { Vec2::new(-(SCREEN.x * 0.5 - OFF_X), -y) } else { Vec2::new(SCREEN.x * 0.5 + OFF_X, y) }
    }
}

/// The marker's pieces on the HUD.
#[derive(Component, Clone, Copy, PartialEq)]
enum MarkerPart {
    /// 0 the top left corner, 1 the bottom right
    Corner(usize),
    Glyphs,
    Wave,
}

pub fn plugin(app: &mut App) {
    app.init_resource::<Marker>()
        .add_systems(OnEnter(AppState::Playing), setup_marker.after(super::hud::HudSetup))
        // (once the reticle has its lock; before the HUD draws the crosshair by its fill)
        .add_systems(Update, update_marker.after(super::reticle::update_reticle).before(update_weapons).run_if(in_state(AppState::Playing)));
}

/// A marker texture as a UI image, its alpha raised to SRGB_ALPHA; `twice` lays it twice side
/// by side (the wave, so a window that wraps past its right edge reads on).
fn marker_image(game: &mut Game, images: &mut Assets<Image>, name: u32, twice: bool) -> Option<Handle<Image>> {
    let (w, h, px) = game.texture_rgba(name)?;
    let mut out = Vec::with_capacity(px.len() * if twice { 2 } else { 1 });
    for row in px.chunks_exact(w as usize * 4) {
        for _ in 0..if twice { 2 } else { 1 } {
            for p in row.chunks_exact(4) {
                out.extend_from_slice(&[p[0], p[1], p[2], ((p[3] as f32 / 255.0).powf(SRGB_ALPHA) * 255.0).round() as u8]);
            }
        }
    }
    Some(images.add(Image::new(
        bevy::render::render_resource::Extent3d { width: if twice { w * 2 } else { w }, height: h, depth_or_array_layers: 1 },
        bevy::render::render_resource::TextureDimension::D2, out,
        bevy::render::render_resource::TextureFormat::Rgba8UnormSrgb,
        bevy::asset::RenderAssetUsages::default())))
}

/// The marker's pieces, hidden, over the HUD (spawned after it: drawn on top, the scope's frame
/// included, where take15's glyph strip shows), in the HUD's 640 x 480 screen.
fn setup_marker(mut commands: Commands, mut game: ResMut<GameData>, mut images: ResMut<Assets<Image>>) {
    commands.insert_resource(Marker::default());
    if std::env::var("BF_NO_HUD").is_ok() {
        return;
    }
    let (corner, glyphs, wave) = (marker_image(&mut game.0, &mut images, CORNER_TEX, false), marker_image(&mut game.0, &mut images, GLYPH_TEX, false),
                                  marker_image(&mut game.0, &mut images, WAVE_TEX, true));
    let window = commands.spawn(Node { position_type: PositionType::Absolute, width: Val::Percent(100.0), height: Val::Percent(100.0),
                                       justify_content: JustifyContent::Center, ..default() }).id();
    let screen = commands.spawn((Node { height: Val::Percent(100.0), aspect_ratio: Some(4.0 / 3.0), ..default() }, ChildOf(window))).id();
    // (stored bottom row first, as all the game's textures: drawn flipped; the bottom right
    // corner is the texture turned half a turn)
    let node = |img: &Option<Handle<Image>>, fx: bool, fy: bool| ImageNode { flip_x: fx, flip_y: fy, ..ImageNode::new(img.clone().unwrap_or_default()) };
    commands.spawn((ChildOf(screen), super::hud::at(0.0, 0.0, CORNER, CORNER), node(&corner, false, true), MarkerPart::Corner(0), Visibility::Hidden));
    commands.spawn((ChildOf(screen), super::hud::at(0.0, 0.0, CORNER, CORNER), node(&corner, true, false), MarkerPart::Corner(1), Visibility::Hidden));
    commands.spawn((ChildOf(screen), super::hud::at(0.0, 0.0, GLYPH_STRIP.x, GLYPH_STRIP.y), node(&glyphs, false, true), MarkerPart::Glyphs, Visibility::Hidden));
    commands.spawn((ChildOf(screen), super::hud::at(0.0, 0.0, WAVE_SIZE.x, WAVE_SIZE.y), node(&wave, false, true), MarkerPart::Wave, Visibility::Hidden));
}

/// A character's bounding sphere in its model's frame made from its vertices, for a model whose
/// mesh gives none (`Character::bounds`, the game's: FUN_00022bc0 asks the object's +0xac, 0x158ca0
/// -> its model's sphere, FUN_00236670): the bind pose's vertices' box centre and the farthest
/// vertex from it.
fn model_sphere(model: &Character) -> (Vec3, f32) {
    let points = model.geosets.iter().flat_map(|g| g.positions.iter().map(|&p| Vec3::from(p)));
    let (lo, hi) = points.clone().fold((Vec3::MAX, Vec3::MIN), |(lo, hi), p| (lo.min(p), hi.max(p)));
    if lo.x > hi.x {
        return (Vec3::ZERO, 0.0);
    }
    let centre = (lo + hi) * 0.5;
    (centre, points.map(|p| p.distance(centre)).fold(0.0, f32::max))
}

/// The marker each frame (see the module's notes), and its pieces on the HUD.
#[allow(clippy::too_many_arguments)]
fn update_marker(time: Res<Time>, mut player: ResMut<Player>, mut squad: ResMut<Squad>, reticle: Res<reticle::Reticle>, game: Res<GameData>,
                 camera: Query<(&Transform, &Projection), With<MainCamera>>, mut marker: ResMut<Marker>,
                 mut parts: Query<(&MarkerPart, &mut Visibility, &mut Node, &mut ImageNode)>, mut spheres: Local<HashMap<u32, (Vec3, f32)>>) {
    let dt = frame_dt(&time);
    let shares = game.0.combat_skills.tracking.box_shares;
    // the controlled character's skill's marker (the skill's +0xc4 makes it); control passing
    // starts it over
    let owner = skills::has_marker(&player).then(|| player.who());
    if marker.owner != owner {
        *marker = Marker::new(owner, shares);
    }
    // (a doubled roll's flag: the controlled character's marker shows it, everyone's is cleared)
    let double = std::mem::take(&mut player.skill.double_hit);
    for m in squad.0.iter_mut() {
        m.skill.double_hit = false;
    }
    if owner.is_some() {
        // 0x13cd30: on, and the crosshair's lock
        marker.on = player.skill.on;
        marker.lock = reticle.lock;
        // 0xb4730: the fill and the pulse
        if marker.on {
            marker.fill = (marker.fill + dt * FILL_UP).min(1.0);
            if marker.pulse_down {
                marker.pulse -= dt;
                if marker.pulse < 0.0 {
                    marker.pulse = 0.0;
                    marker.pulse_down = false;
                }
            } else {
                marker.pulse += dt;
                if marker.pulse > PULSE_TOP {
                    marker.pulse = PULSE_TOP;
                    marker.pulse_down = true;
                }
            }
        } else {
            marker.fill = (marker.fill - dt * FILL_DOWN).max(0.0);
        }
        let k = marker.fill;
        marker.alpha = if k > 0.0 { ((k - k * marker.pulse * PULSE_DIM) * 255.0).trunc() / 255.0 } else { 0.0 };
        marker.place = crosshair_centre(&player);
        // the locked target's place and size while on (else none: 0xb4999)
        marker.target = (Vec2::ZERO, 0.0);
        let locked = marker.lock.filter(|_| marker.on).and_then(|who| squad.0.iter().find(|u| u.who() == who && !u.dead));
        if let (Some(u), Ok((tr, Projection::Perspective(pp)))) = (locked, camera.single()) {
            if let Some(l) = u.loaded.as_ref() {
                let (local, radius) = *spheres.entry(u.kind).or_insert_with(|| {
                    let s = l.model.bounds.unwrap_or_else(|| model_sphere(&l.model));
                    if std::env::var("BF_TARGETING_LOG").is_ok() {
                        println!("marker: {}'s bounding sphere: centre {:.3}, radius {:.3} (from the {}; the vertices' {:?})", unit_name(u), s.0, s.1,
                                 if l.model.bounds.is_some() { "mesh's geosets" } else { "vertices" }, model_sphere(&l.model));
                    }
                    s
                });
                let root = Mat4::from_rotation_translation(Quat::from_rotation_y(u.yaw), u.position + Vec3::Y * (u.height + u.lift_now));
                let view = View::of(tr, pp);
                if let Some((at, depth)) = view.project(root.transform_point3(local)) {
                    // (640 r / d x the zoom in the game's view: GAME_TAN_ACROSS of the sphere's
                    // projected diameter, here in the demo's)
                    marker.target = (at - marker.place, SIZE_SCALE * radius / depth * GAME_TAN_ACROSS / view.tan_across());
                }
            }
        }
        // the glyphs: the target's health in 15s, rounded up (0xb5140); none without a lock
        marker.glyphs = marker.lock.and_then(|who| squad.0.iter().find(|u| u.who() == who))
            .map_or(0.0, |u| (u.health / GLYPH_HP).ceil().clamp(0.0, GLYPHS));
        // the corners head for their points at their speeds (0xb4f20)
        let (safe, place) = (game.0.hud_safe, marker.place);
        let low = Vec2::new(safe[0].max(0.0) + SAFE_IN, safe[1].max(0.0) + SAFE_IN) - place;
        let high = SCREEN - Vec2::new((SCREEN.x + safe[0] - SAFE_RIGHT).max(0.0) + SAFE_IN, (SCREEN.y + safe[1] - SAFE_BOTTOM).max(0.0) + SAFE_IN) - place;
        for k in 0..2 {
            let sign = if k == 0 { -1.0 } else { 1.0 };
            let goal = if marker.lock.is_some() {
                let (off, size) = marker.target;
                let half = Vec2::new((size * TIGHT_X).max(TIGHT_MIN), (size * TIGHT_Y).max(TIGHT_MIN));
                let p = off + half * sign;
                if k == 0 { p.max(low) } else { p.min(high) }
            } else if marker.on {
                Vec2::new(shares[0] * SCREEN.x * 0.5, shares[1] * SCREEN.y * 0.5) * sign
            } else {
                Marker::off_point(k, shares)
            };
            let d = goal - marker.corners[k];
            let (left, right) = if k == 0 { (SLIDE_OUT, SLIDE_IN) } else { (SLIDE_IN, SLIDE_OUT) };
            let step_x = if d.x < 0.0 { (-dt * left).max(d.x) } else { (dt * right).min(d.x) };
            marker.corners[k] += Vec2::new(step_x, d.y.clamp(-dt * SLIDE_Y, dt * SLIDE_Y));
        }
        // the wave scrolls whether it shows or not (0xb5310)
        marker.wave = (marker.wave + dt * WAVE_SPEED) % WAVE_WRAP;
        marker.red = double && marker.lock.is_some();
        log_marker(&mut marker, player.sim_time);
    }
    // ---- its pieces ----
    let shown = owner.is_some() && marker.fill > 0.0;
    let fade = marker.alpha.powf(SRGB_ALPHA);
    // (a doubled roll's red is the corners' own colour, 0xb50e3; the strips draw in white,
    // 0xb52ca / 0xb541f)
    let white = Color::srgba(1.0, 1.0, 1.0, fade);
    let corner = if marker.red { Color::srgba(1.0, 0.0, 0.0, fade) } else { white };
    let place = marker.place;
    for (part, mut vis, mut node, mut img) in &mut parts {
        let (show, at, size) = match *part {
            MarkerPart::Corner(0) => (shown, place + marker.corners[0], Vec2::splat(CORNER)),
            MarkerPart::Corner(_) => (shown, place + marker.corners[1] - Vec2::splat(CORNER), Vec2::splat(CORNER)),
            MarkerPart::Glyphs => {
                let clip = (GLYPHS - marker.glyphs) * GLYPH_W;
                img.rect = Some(Rect::new(clip, 0.0, GLYPH_STRIP.x, GLYPH_STRIP.y));
                (shown && marker.lock.is_some() && clip < GLYPH_STRIP.x, place + marker.corners[0] + GLYPH_AT + Vec2::new(clip, 0.0),
                 Vec2::new(GLYPH_STRIP.x - clip, GLYPH_STRIP.y))
            }
            MarkerPart::Wave => {
                // (a window of the texture laid twice: u from the scroll's fraction)
                let u0 = marker.wave.fract() * WAVE_TEX_W;
                img.rect = Some(Rect::new(u0, 0.0, u0 + WAVE_TEX_W * WAVE_SHOWN, WAVE_SIZE.y));
                (shown && marker.lock.is_some(), place + marker.corners[1] + WAVE_AT, WAVE_SIZE)
            }
        };
        let want = if show { Visibility::Inherited } else { Visibility::Hidden };
        if *vis != want {
            *vis = want;
        }
        if !show {
            continue;
        }
        let n = super::hud::at(at.x, at.y, size.x, size.y);
        (node.left, node.top, node.width, node.height) = (n.left, n.top, n.width, n.height);
        let colour = if matches!(part, MarkerPart::Corner(_)) { corner } else { white };
        if img.color != colour {
            img.color = colour;
        }
    }
}

/// BF_TARGETING_LOG: the marker's state when it changes (=2: every frame).
fn log_marker(marker: &mut Marker, now: f32) {
    let Ok(log) = std::env::var("BF_TARGETING_LOG") else { return };
    let mode = if marker.lock.is_some() { "tight" } else if marker.on { "wide" } else { "off" };
    let key = format!("{} {mode} {:?} fill {}", marker.on, marker.lock, if marker.fill >= 1.0 { "1" } else if marker.fill <= 0.0 { "0" } else { "part" });
    if log != "2" && key == marker.logged {
        return;
    }
    let place = marker.place;
    let (a, b) = (place + marker.corners[0], place + marker.corners[1]);
    println!("t {now:.2}: marker {}, {mode}, lock {:?}, fill {:.2}, alpha {:.2}, target {:+.1} size {:.1}, corners ({:.1}, {:.1}) ({:.1}, {:.1}), glyphs {}",
             if marker.on { "on" } else { "off" }, marker.lock, marker.fill, marker.alpha, marker.target.0, marker.target.1, a.x, a.y, b.x, b.y, marker.glyphs);
    marker.logged = key;
}
