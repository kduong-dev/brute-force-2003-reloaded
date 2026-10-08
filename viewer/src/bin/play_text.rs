//! HUD text in the game's own font: the body font atlas h_e4e4d2f4 (the menu's, see
//! `menu::AtlasFont`), its glyphs drawn 1 atlas pixel apart over a black outline and drop
//! shadow, as the captures show the HUD's words and numbers (todo/gate button.mp4,
//! medkits.mp4: a condensed bold face, capitals ~11-14 units tall, outlined dark).
//!
//! A `GameText` is placed in the HUD's 640 x 480 units: `x` is its left edge, centre or right
//! edge (by `align`), `y` the top of its line. Its glyphs are rebuilt when the text, colour or
//! window height changes; moving it (x, y) only moves the node.

use super::*;
use super::menu::AtlasFont;

const BODY_FONT: u32 = 0xE4E4_D2F4;
/// Gaps, in atlas pixels: between glyphs, and a space.
const SPACING: f32 = 1.0;
const SPACE: f32 = 4.0;
/// Outline and shadow, in atlas pixels.
const STROKE: f32 = 1.0;
const SHADOW: f32 = 1.5;

/// Colours sampled from the captures: messages and prompts (pale blue), ammo and "NEW"
/// (orange), the item box's name and count (red).
pub const MESSAGE_BLUE: Color = Color::srgb(0.73, 0.82, 0.98);
pub const AMMO_ORANGE: Color = Color::srgb(1.0, 0.65, 0.41);
pub const ITEM_RED: Color = Color::srgb(0.8, 0.35, 0.28);

#[derive(Resource)]
pub struct GameFont(AtlasFont);

#[derive(Clone, Copy, PartialEq)]
pub enum Align {
    Left,
    Centre,
    Right,
}

#[derive(Component)]
pub struct GameText {
    pub text: String,
    pub x: f32,
    pub y: f32,
    pub cap: f32,
    pub color: Color,
    pub align: Align,
    /// what's drawn: text, colour, pixels per unit
    drawn: Option<(String, Color, f32)>,
    /// its width in units (for centring and right alignment)
    width: f32,
}

/// A text node at (x, y) units with capitals `cap` units tall.
pub fn game_text(text: &str, x: f32, y: f32, cap: f32, color: Color, align: Align) -> impl Bundle {
    (GameText { text: text.to_string(), x, y, cap, color, align, drawn: None, width: 0.0 },
     Node { position_type: PositionType::Absolute, flex_direction: FlexDirection::Row, ..default() })
}

pub fn plugin(app: &mut App) {
    app.add_systems(OnEnter(AppState::Playing), load_font)
        .add_systems(PostUpdate, draw.before(bevy::ui::UiSystem::Layout).run_if(in_state(AppState::Playing)));
}

fn load_font(mut commands: Commands, mut game: ResMut<GameData>, mut images: ResMut<Assets<Image>>, font: Option<Res<GameFont>>) {
    if font.is_some() {
        return;
    }
    let Some((w, h, px)) = game.0.texture_rgba(BODY_FONT) else { return };
    let glyphs = AtlasFont::sequential(w, h, &px);
    let image = images.add(Image::new(
        bevy::render::render_resource::Extent3d { width: w, height: h, depth_or_array_layers: 1 },
        bevy::render::render_resource::TextureDimension::D2, px,
        bevy::render::render_resource::TextureFormat::Rgba8UnormSrgb, bevy::asset::RenderAssetUsages::default()));
    commands.insert_resource(GameFont(AtlasFont::new(image, glyphs)));
}

impl GameFont {
    /// `text`'s width in units with capitals `cap` units tall.
    fn width(&self, text: &str, cap: f32) -> f32 {
        let f = &self.0;
        let px: f32 = text.chars().map(|c| match (c, f.glyph(c)) {
            (' ', _) | (_, None) => SPACE + SPACING,
            (_, Some(g)) => g.rect.width() + SPACING,
        }).sum::<f32>() - SPACING;
        px.max(0.0) * cap / f.cap
    }

    /// The line's height in units.
    fn line(&self, cap: f32) -> f32 {
        (self.0.rise + self.0.drop) * cap / self.0.cap
    }
}

fn draw(mut commands: Commands, font: Option<Res<GameFont>>, windows: Query<&Window, With<PrimaryWindow>>,
        mut texts: Query<(Entity, &mut GameText, &mut Node)>) {
    let (Some(font), Ok(win)) = (font, windows.single()) else { return };
    // the HUD screen is 4:3 at the window's height
    let unit = win.height() / 480.0;
    for (e, mut t, mut node) in &mut texts {
        let want = (t.text.clone(), t.color, unit);
        if t.drawn.as_ref() != Some(&want) {
            t.width = font.width(&t.text, t.cap);
            t.drawn = Some(want);
            commands.entity(e).despawn_related::<Children>();
            spawn_glyphs(&mut commands, &font.0, e, &t.text, t.cap * unit, t.color);
        }
        let left = match t.align {
            Align::Left => t.x,
            Align::Centre => t.x - t.width / 2.0,
            Align::Right => t.x - t.width,
        };
        let (l, top) = (Val::Percent(left / 6.4), Val::Percent(t.y / 4.8));
        if node.left != l || node.top != top {
            node.left = l;
            node.top = top;
        }
        let h = Val::Px(font.line(t.cap) * unit);
        if node.height != h {
            node.height = h;
        }
    }
}

/// The glyphs of `text` (capitals `cap` pixels tall) as children of `parent`: each over its
/// black shadow and outline.
fn spawn_glyphs(commands: &mut Commands, f: &AtlasFont, parent: Entity, text: &str, cap: f32, color: Color) {
    let k = cap / f.cap;
    for c in text.chars() {
        let Some(g) = f.glyph(c).filter(|_| c != ' ') else {
            commands.spawn((Node { width: Val::Px(SPACE * k), margin: UiRect::right(Val::Px(SPACING * k)), ..default() }, ChildOf(parent)));
            continue;
        };
        let (w, h) = (g.rect.width() * k, g.rect.height() * k);
        let cell = commands.spawn((Node { width: Val::Px(w), height: Val::Px(h), flex_shrink: 0.0,
                                          margin: UiRect { top: Val::Px((f.rise - (g.base - g.rect.min.y)) * k), right: Val::Px(SPACING * k), ..default() },
                                          ..default() }, ChildOf(parent))).id();
        let mut layer = |dx: f32, dy: f32, color: Color| {
            commands.spawn((ImageNode { rect: Some(g.rect), color, ..ImageNode::new(f.image.clone()) },
                            Node { position_type: PositionType::Absolute, left: Val::Px(dx * k), top: Val::Px(dy * k),
                                   width: Val::Px(w), height: Val::Px(h), ..default() },
                            ChildOf(cell)));
        };
        let edge = Color::srgba(0.0, 0.0, 0.0, 0.85);
        layer(SHADOW, SHADOW, edge);
        for (dx, dy) in [(-STROKE, 0.0), (STROKE, 0.0), (0.0, -STROKE), (0.0, STROKE)] {
            layer(dx, dy, edge);
        }
        layer(0.0, 0.0, color);
    }
}
