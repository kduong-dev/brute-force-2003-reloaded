//! Text in the game's own font records (#226): the body font h_0c626fd5 at 10 (#241), 12, 14 and
//! 16 points and the heading font at 24, drawn glyph by glyph as default.xbe's text renderer draws them
//! (FUN_000ccd20), in any of bf_play's states (the front end, a briefing, a map). The end screen
//! (play_endmenu.rs), the menu system's status line ("Saving...", play_mission.rs), the front
//! end's campaign screens and the tutorial's panels (play_panels.rs, #241) use it.
//!
//! A record (FUN_000cca40 reads one into a font slot) holds the font's line height, its
//! texture's size, its first and last characters and per character the glyph's cell on the
//! texture (u0 v0 u1 v1), the offset the pen moves before it, its width and the advance after it.
//! FUN_000cd5d0 registers each on an atlas with a v scale and offset: the 10-point record
//! (0x3c0b68) on h_f09bc36b at 0.5 + 0.6640625 (its lower, smaller size: from y 340), the 12-point record
//! (0x3c2088) on h_e4e4d2f4 at 0.5 + 0.5859375 (the atlas's lower, smaller size: its glyphs from
//! y 300), the 16-point record (0x3c4ac8) on h_f09bc36b at 1 + 0, the heading's 24-point record
//! (0x3c7b90) on h_e0afcd52 at 1 + 0. The tables below are those records read from default.xbe,
//! each cell's corners already in its atlas's pixels as the renderer samples them: (the record's
//! texture size x the coordinate - 1) / the size (0x40d644-0x40d650 hold the 1s), v through the
//! registration's scale and offset.
//!
//! The renderer (FUN_000ccd20) moves the pen by a glyph's offset, draws its cell (width + 1) x
//! (line height + 1) units from the pen less half a unit, then moves the pen by its advance; a
//! character out of the record's range is drawn as 0x82, a space out of it moves the pen as far
//! as "1" would; `^cN` picks colour N of the text's colour list; a tab goes to the next multiple
//! of 50 units from the screen's left. Flag 1 right-aligns the line on x (x less its width,
//! floored), flag 2 centres it. FUN_000ccae0 measures a line the same way (its tabs from the
//! line's start). A static text draws its shadow first, one unit down and right in its style's
//! h_ed70ff4f colour (FUN_0010a420), then itself.

use std::collections::HashMap;

use bf_viewer::bf::character::Game;

use super::*;

/// A font record (see the module notes): its atlas (a texture name), line height, the first
/// character it has and per character from it its cell's corners on the atlas (pixels), offset,
/// width and advance.
pub struct Record {
    pub atlas: u32,
    pub line: f32,
    pub first: u32,
    pub glyphs: &'static [(f32, f32, f32, f32, i8, u8, u8)],
}

/// The body font h_0c626fd5 at 10 points: record 0x3c0b68 (line height 16, its texture 256 x
/// 256, characters 32-255) on the atlas h_f09bc36b (v x 0.5 + 0.6640625). The tutorial panels'
/// tips (style h_e2f74de0, the only 10-point style in game-options: #241).
pub const BODY10: Record = Record { atlas: 0xF09B_C36B, line: 16.0, first: 32, glyphs: &REC10_GLYPHS };
/// The body font h_0c626fd5 at 12 points: record 0x3c2088 (line height 19, its texture 256 x 256,
/// characters 32-255) on the atlas h_e4e4d2f4 (v x 0.5 + 0.5859375). The menus' small texts: the
/// status line, the end screen's lists, the legends' words.
pub const BODY12: Record = Record { atlas: 0xE4E4_D2F4, line: 19.0, first: 32, glyphs: &REC12_GLYPHS };
/// The body font at 14 points: record 0x3c35a8 (line height 23, 256 x 512, characters 32-255) on
/// h_e4e4d2f4 (v x 1: the atlas's top, larger size). A front-end menu item whose style names no
/// font draws in it: the CAMPAIGN screen's choices ("CAMPAIGN" 81 units of ink in take01 at
/// 9.4 s, the record's 83 of pen).
pub const BODY14: Record = Record { atlas: 0xE4E4_D2F4, line: 23.0, first: 32, glyphs: &REC14_GLYPHS };
/// The body font at 16 points: record 0x3c4ac8 (line height 25, 256 x 512, characters 32-255) on
/// h_f09bc36b (v x 1). The end screen's labels and values ("Objectives", "2 / 2").
pub const BODY16: Record = Record { atlas: 0xF09B_C36B, line: 25.0, first: 32, glyphs: &REC16_GLYPHS };
/// The heading font ("heading") at 24 points: record 0x3c7b90 (line height 35, 256 x 512,
/// characters 46-223: no space, which moves the pen as "1" does) on h_e0afcd52. The end screen's
/// title ("MISSION SUCCESSFUL").
pub const HEADING24: Record = Record { atlas: 0xE0AF_CD52, line: 35.0, first: 46, glyphs: &HEADING24_GLYPHS };
/// The character a record lacks is drawn as (FUN_000ccd20: 0x82).
const MISSING: u32 = 0x82;

/// A menu style's record by its font: the heading font at 24 points, the body font at 16, 14, 12
/// or 10 by its point size; a style with no font (h_ea5acf71, the listboxes' h_e9ca5a60 and
/// h_0a6d21f5) draws in the body font at 14 points (take01: "CAMPAIGN" 81 units of ink, "New
/// Campaign" 116: the 14-point record's 83 and 116 of pen).
pub fn record_of(style: &bf_viewer::bf::menus::MenuStyle) -> &'static Record {
    match style.font {
        Some((f, _)) if f == bf_viewer::bf::hash::h("heading") => &HEADING24,
        Some((_, p)) if p >= 15.0 => &BODY16,
        Some((_, p)) if p >= 13.0 => &BODY14,
        Some((_, p)) if p >= 11.0 => &BODY12,
        Some(_) => &BODY10,
        None => &BODY14,
    }
}
/// Tab stops (FUN_000ccd20, FUN_000ccae0): every 50 units.
const TAB: f32 = 50.0;

/// Placing a line across its x (FUN_000ccd20's flags 1 and 2).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Across {
    Left,
    Right,
    Centre,
}

impl Record {
    /// A character's entry: its own, or, out of the record's range, 0x82's (a space then moves the
    /// pen as "1" does: `advance`).
    fn entry(&self, c: char) -> Option<&(f32, f32, f32, f32, i8, u8, u8)> {
        let code = c as u32;
        let last = self.first + self.glyphs.len() as u32 - 1;
        let code = if (self.first..=last).contains(&code) { code } else { MISSING };
        self.glyphs.get((code.checked_sub(self.first)?) as usize)
    }

    /// How far a character moves the pen: its offset and advance; a space out of the record's
    /// range as far as "1".
    pub fn advance(&self, c: char) -> f32 {
        let c = if c == ' ' && (c as u32) < self.first { '1' } else { c };
        self.entry(c).map_or(0.0, |g| (g.4 as i32 + g.6 as i32) as f32)
    }

    /// A line's width (FUN_000ccae0 on one line): the pen's furthest point, tabs from the line's
    /// start, `^cN` taking no room.
    pub fn measure(&self, line: &str) -> f32 {
        let chars: Vec<char> = line.chars().collect();
        let (mut x, mut width) = (0.0f32, 0.0f32);
        let mut i = 0;
        while i < chars.len() {
            match chars[i] {
                '\n' => break,
                '\t' => x += TAB - (x as i32 % TAB as i32) as f32,
                '^' if chars.get(i + 1) == Some(&'c') => {
                    i += 3;
                    continue;
                }
                c => {
                    x += self.advance(c);
                    width = width.max(x);
                }
            }
            i += 1;
        }
        width
    }

    /// A line's height: the line height and one (the cell; FUN_000ccae0's measure, the static
    /// text's line pitch, FUN_0010abb0).
    pub fn height(&self) -> f32 {
        self.line + 1.0
    }
}

/// The atlases the records draw from, as images (installed at boot, or before a map that starts
/// straight away).
#[derive(Resource, Default)]
pub struct FontImages(HashMap<u32, Handle<Image>>);

impl FontImages {
    /// A record's atlas image.
    pub fn of(&self, rec: &Record) -> Option<Handle<Image>> {
        self.0.get(&rec.atlas).cloned()
    }
}

/// The atlases' pixels, gathered from the game data (on the boot thread).
pub struct FontData(Vec<(u32, (u32, u32, Vec<u8>))>);

/// Gather the records' atlases (see FontData).
pub fn gather(game: &mut Game) -> FontData {
    FontData([BODY12.atlas, BODY16.atlas, HEADING24.atlas].into_iter().filter_map(|t| game.texture_rgba(t).map(|px| (t, px))).collect())
}

/// Put the atlases in the world as images.
pub fn install(world: &mut World, data: FontData) {
    let mut images = world.resource_mut::<Assets<Image>>();
    let map = data.0.into_iter().map(|(t, (w, hgt, px))| (t, images.add(Image::new(
        bevy::render::render_resource::Extent3d { width: w, height: hgt, depth_or_array_layers: 1 },
        bevy::render::render_resource::TextureDimension::D2, px,
        bevy::render::render_resource::TextureFormat::Rgba8UnormSrgb, bevy::asset::RenderAssetUsages::default())))).collect();
    world.insert_resource(FontImages(map));
}

/// A line of text in a record, placed on the 640 x 480 screen of its parent (a node the size of
/// the screen): its text (`^cN` picks colour N of `colours`), where (x by `across`; y its top),
/// its shadow's colour (drawn one unit down and right first), its glyphs' alpha and its scale
/// (FUN_000ccd20's: every offset, cell and advance times it). Rebuilt when any of it but the
/// alpha changes, which its glyphs take as they are (`draw`).
#[derive(Component, Clone)]
pub struct RecordText {
    pub text: String,
    pub x: f32,
    pub y: f32,
    pub record: &'static Record,
    pub colours: Vec<Color>,
    pub shadow: Option<Color>,
    pub across: Across,
    /// what's drawn: (text, x, y, scale bits) and the alpha, to tell when to rebuild
    drawn: Option<(String, u32, u32, u32)>,
    drawn_alpha: f32,
    /// the alpha all its glyphs are drawn at (a fade)
    pub alpha: f32,
    /// the scale (1: the record's own size)
    pub scale: f32,
}

impl RecordText {
    /// Have its glyphs made again in the next frame (their nodes were taken: a map's end takes
    /// what was made during it; or its colours changed).
    pub fn redraw(&mut self) {
        self.drawn = None;
    }
}

/// A glyph's own colour (its text's or its `^cN` colour, before the text's alpha).
#[derive(Component)]
struct Glyph(Color);

/// A record text's node (see RecordText): absolute, the whole screen, its glyphs inside it.
pub fn record_text(text: &str, x: f32, y: f32, record: &'static Record, colours: Vec<Color>, shadow: Option<Color>, across: Across) -> impl Bundle {
    (RecordText { text: text.to_string(), x, y, record, colours, shadow, across, drawn: None, drawn_alpha: 1.0, alpha: 1.0, scale: 1.0 },
     Node { position_type: PositionType::Absolute, left: Val::Px(0.0), top: Val::Px(0.0), width: Val::Percent(100.0), height: Val::Percent(100.0),
            ..default() })
}

/// The record texts' drawing system (`draw`): what writes their nodes before the UI's layout
/// (play_ui_guard.rs orders its net after it).
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct DrawRecords;

/// The record texts' drawing, in every state (the status line shows over the front end and the
/// briefings too). Its glyph nodes are made before the UI's frame starts: UiSystem::Prepare gives
/// each node its camera and scale, UiSystem::Stack lists the nodes to draw and the visibility
/// propagation shows them; a node made after any of them isn't drawn until the next frame (and a
/// text made anew every frame, sliding or typing, wouldn't show: the front end's captures).
pub fn plugin(app: &mut App) {
    app.init_resource::<FontImages>()
        .add_systems(PostUpdate, draw.in_set(DrawRecords).before(bevy::ui::UiSystem::Prepare).before(bevy::ui::UiSystem::Stack)
            .before(bevy::render::view::VisibilitySystems::VisibilityPropagate));
}

/// Rebuild the glyphs of each record text that changed, or give them its new alpha (see
/// RecordText).
fn draw(mut commands: Commands, images: Res<FontImages>, mut texts: Query<(Entity, &mut RecordText, Option<&Children>)>,
        mut glyphs: Query<(&Glyph, &mut ImageNode)>) {
    for (e, mut t, children) in &mut texts {
        // (a number the placing code computed that isn't finite is 0 and logged once, #349)
        let what = |t: &RecordText, field: &str| format!("record text {:?}'s {field}", t.text);
        if !(t.x.is_finite() && t.y.is_finite() && t.scale.is_finite() && t.alpha.is_finite()) {
            // (x and y to 0, the scale and the alpha to 1)
            if !t.x.is_finite() {
                t.x = super::ui_guard::finite_with(|| what(&t, "x"), t.x);
            }
            if !t.y.is_finite() {
                t.y = super::ui_guard::finite_with(|| what(&t, "y"), t.y);
            }
            if !t.scale.is_finite() {
                super::ui_guard::finite_with(|| what(&t, "scale"), t.scale);
                t.scale = 1.0;
            }
            if !t.alpha.is_finite() {
                super::ui_guard::finite_with(|| what(&t, "alpha"), t.alpha);
                t.alpha = 1.0;
            }
        }
        let want = (t.text.clone(), t.x.to_bits(), t.y.to_bits(), t.scale.to_bits());
        if t.drawn.as_ref() == Some(&want) {
            if t.drawn_alpha != t.alpha {
                t.drawn_alpha = t.alpha;
                for &c in children.into_iter().flatten() {
                    if let Ok((g, mut node)) = glyphs.get_mut(c) {
                        node.color = g.0.with_alpha(g.0.alpha() * t.alpha);
                    }
                }
            }
            continue;
        }
        let Some(image) = images.of(t.record) else { continue };
        t.drawn = Some(want);
        t.drawn_alpha = t.alpha;
        commands.entity(e).despawn_related::<Children>();
        let width = t.record.measure(&t.text) * t.scale;
        // (FUN_000ccd20: right-aligned or centred lines start on a whole unit)
        let left = match t.across {
            Across::Left => t.x,
            Across::Right => (t.x - width).floor(),
            Across::Centre => (t.x - width * 0.5).floor(),
        };
        let look = Look { image: &image, rec: t.record, alpha: t.alpha, scale: t.scale };
        if let Some(s) = t.shadow {
            spawn_glyphs(&mut commands, e, &look, &t.text, left + 1.0, t.y + 1.0, &[s]);
        }
        spawn_glyphs(&mut commands, e, &look, &t.text, left, t.y, &t.colours);
    }
}

/// How a line's glyphs are drawn: the atlas, the record, the alpha and the scale.
struct Look<'a> {
    image: &'a Handle<Image>,
    rec: &'a Record,
    alpha: f32,
    scale: f32,
}

/// One line's glyphs (FUN_000ccd20, see the module notes) as children of `parent`, the pen from
/// (x, y): each cell (width + 1) x (line + 1) units (times the scale) from the pen plus its
/// offset, less half a unit. `colours[0]` is the text's colour, `^cN` picks colour N (a single
/// colour, a shadow's, stays).
fn spawn_glyphs(commands: &mut Commands, parent: Entity, look: &Look, text: &str, x: f32, y: f32, colours: &[Color]) {
    let (image, rec, alpha, k) = (look.image, look.rec, look.alpha, look.scale);
    let chars: Vec<char> = text.chars().collect();
    let mut pen = x;
    let mut colour = colours.first().copied().unwrap_or(Color::WHITE);
    let mut i = 0;
    while i < chars.len() {
        match chars[i] {
            '\n' => break,
            '\t' => {
                pen += (TAB - (pen as i32 % TAB as i32) as f32) * k;
                i += 1;
                continue;
            }
            '^' if chars.get(i + 1) == Some(&'c') => {
                if let Some(c) = chars.get(i + 2).and_then(|d| d.to_digit(10)).and_then(|n| colours.get(n as usize)) {
                    if colours.len() > 1 {
                        colour = *c;
                    }
                }
                i += 3;
                continue;
            }
            ' ' if (' ' as u32) < rec.first => {
                pen += rec.advance(' ') * k;
                i += 1;
                continue;
            }
            c => {
                let Some(&(u0, v0, u1, v1, offset, w, adv)) = rec.entry(c) else {
                    i += 1;
                    continue;
                };
                pen += offset as f32 * k;
                // (a space's cell is empty: not drawn)
                if c != ' ' {
                    let (left, top) = (pen - 0.5, y - 0.5);
                    let (cw, ch) = ((w as f32 + 1.0) * k, (rec.line + 1.0) * k);
                    commands.spawn((ImageNode { rect: Some(Rect::new(u0.max(0.0), v0.max(0.0), u1, v1)),
                                                color: colour.with_alpha(colour.alpha() * alpha), ..ImageNode::new(image.clone()) },
                                    Glyph(colour),
                                    Node { position_type: PositionType::Absolute, left: Val::Percent(left / 6.4), top: Val::Percent(top / 4.8),
                                           width: Val::Percent(cw / 6.4), height: Val::Percent(ch / 4.8), ..default() },
                                    ChildOf(parent)));
                }
                pen += adv as f32 * k;
            }
        }
        i += 1;
    }
}

/// The 10-point record's glyphs (see BODY10): the cell's corners on h_f09bc36b in pixels, offset,
/// width, advance; characters 32-255. Read from default.xbe 0x3c0b68.
const REC10_GLYPHS: [(f32, f32, f32, f32, i8, u8, u8); 224] = [
    (-1.00, 338.00, 1.01, 355.07, 0, 1, 3), (2.01, 338.00, 6.03, 355.07, 0, 3, 4), (7.03, 338.00, 13.05, 355.07, 0, 5, 5), (14.06, 338.00, 22.09, 355.07, 0, 7, 7), (23.09, 338.00, 31.13, 355.07, 0, 7, 7), (32.13, 338.00, 42.17, 355.07, 0, 9, 9),
    (43.17, 338.00, 52.21, 355.07, 0, 8, 8), (53.21, 338.00, 57.23, 355.07, 0, 3, 3), (58.23, 338.00, 64.25, 355.07, 0, 5, 5), (65.26, 338.00, 71.28, 355.07, 0, 5, 5), (72.29, 338.00, 79.31, 355.07, 0, 6, 6), (80.32, 338.00, 88.35, 355.07, 0, 7, 7),
    (89.35, 338.00, 94.37, 355.07, 0, 4, 4), (95.38, 338.00, 100.40, 355.07, 0, 4, 4), (101.40, 338.00, 105.42, 355.07, 0, 3, 3), (106.42, 338.00, 113.45, 355.07, 0, 6, 6), (114.45, 338.00, 122.48, 355.07, 0, 7, 7), (123.49, 338.00, 128.51, 355.07, 1, 4, 6),
    (129.51, 338.00, 137.54, 355.07, 0, 7, 7), (138.55, 338.00, 146.58, 355.07, 0, 7, 7), (147.58, 338.00, 155.61, 355.07, 0, 7, 7), (156.62, 338.00, 164.65, 355.07, 0, 7, 7), (165.65, 338.00, 173.68, 355.07, 0, 7, 7), (174.69, 338.00, 182.72, 355.07, 0, 7, 7),
    (183.72, 338.00, 191.75, 355.07, 0, 7, 7), (192.76, 338.00, 200.79, 355.07, 0, 7, 7), (201.79, 338.00, 205.81, 355.07, 0, 3, 4), (206.81, 338.00, 211.83, 355.07, 0, 4, 4), (212.84, 338.00, 218.86, 355.07, 1, 5, 6), (219.86, 338.00, 225.89, 355.07, 1, 5, 6),
    (226.89, 338.00, 232.91, 355.07, 1, 5, 6), (233.92, 338.00, 240.95, 355.07, 0, 6, 6), (-1.00, 355.07, 10.04, 372.13, 0, 10, 11), (11.05, 355.07, 20.08, 372.13, 0, 8, 7), (21.09, 355.07, 28.11, 372.13, 1, 6, 7), (29.12, 355.07, 37.15, 372.13, 0, 7, 7),
    (38.15, 355.07, 46.18, 372.13, 1, 7, 7), (47.19, 355.07, 53.21, 372.13, 1, 5, 6), (54.22, 355.07, 60.24, 372.13, 1, 5, 5), (61.24, 355.07, 69.27, 372.13, 0, 7, 8), (70.28, 355.07, 77.31, 372.13, 1, 6, 7), (78.31, 355.07, 81.32, 372.13, 1, 2, 3),
    (82.33, 355.07, 88.35, 372.13, 0, 5, 6), (89.35, 355.07, 96.38, 372.13, 1, 6, 6), (97.38, 355.07, 103.41, 372.13, 1, 5, 5), (104.41, 355.07, 112.44, 372.13, 1, 7, 8), (113.45, 355.07, 120.47, 372.13, 1, 6, 7), (121.48, 355.07, 130.51, 372.13, 0, 8, 8),
    (131.52, 355.07, 138.55, 372.13, 1, 6, 6), (139.55, 355.07, 148.58, 372.13, 0, 8, 8), (149.59, 355.07, 156.62, 372.13, 1, 6, 6), (157.62, 355.07, 165.65, 372.13, 0, 7, 7), (166.65, 355.07, 173.68, 372.13, 0, 6, 6), (174.69, 355.07, 181.71, 372.13, 1, 6, 7),
    (182.72, 355.07, 191.75, 372.13, 0, 8, 7), (192.76, 355.07, 203.80, 372.13, 0, 10, 10), (204.80, 355.07, 212.84, 372.13, 0, 7, 7), (213.84, 355.07, 221.87, 372.13, 0, 7, 6), (222.87, 355.07, 229.90, 372.13, 0, 6, 6), (230.91, 355.07, 235.93, 372.13, 0, 4, 4),
    (236.93, 355.07, 243.96, 372.13, 0, 6, 6), (244.96, 355.07, 249.98, 372.13, 0, 4, 4), (-1.00, 372.13, 7.03, 389.20, 0, 7, 7), (8.04, 372.13, 16.07, 389.20, 0, 7, 7), (17.07, 372.13, 22.09, 389.20, 2, 4, 6), (23.09, 372.13, 30.12, 389.20, 0, 6, 6),
    (31.13, 372.13, 39.16, 389.20, 0, 7, 7), (40.16, 372.13, 47.19, 389.20, 0, 6, 6), (48.19, 372.13, 55.22, 389.20, 0, 6, 7), (56.22, 372.13, 63.25, 389.20, 0, 6, 6), (64.25, 372.13, 70.28, 389.20, 0, 5, 4), (71.28, 372.13, 78.31, 389.20, 0, 6, 7),
    (79.31, 372.13, 86.34, 389.20, 0, 6, 7), (87.35, 372.13, 91.36, 389.20, 0, 3, 3), (92.36, 372.13, 97.38, 389.20, -1, 4, 4), (98.39, 372.13, 106.42, 389.20, 0, 7, 6), (107.42, 372.13, 111.44, 389.20, 0, 3, 3), (112.44, 372.13, 123.49, 389.20, 0, 10, 10),
    (124.49, 372.13, 131.52, 389.20, 0, 6, 7), (132.52, 372.13, 139.55, 389.20, 0, 6, 7), (140.55, 372.13, 148.58, 389.20, 0, 7, 7), (149.59, 372.13, 156.62, 389.20, 0, 6, 7), (157.62, 372.13, 163.64, 389.20, 0, 5, 5), (164.65, 372.13, 170.67, 389.20, 0, 5, 5),
    (171.67, 372.13, 177.70, 389.20, 0, 5, 4), (178.70, 372.13, 185.73, 389.20, 0, 6, 7), (186.73, 372.13, 193.76, 389.20, 0, 6, 6), (194.76, 372.13, 204.80, 389.20, 0, 9, 9), (205.81, 372.13, 213.84, 389.20, 0, 7, 6), (214.84, 372.13, 222.87, 389.20, 0, 7, 6),
    (223.88, 372.13, 229.90, 389.20, 0, 5, 5), (230.91, 372.13, 235.93, 389.20, 0, 4, 4), (236.93, 372.13, 240.95, 389.20, 2, 3, 5), (241.95, 372.13, 246.97, 389.20, 0, 4, 4), (-1.00, 389.20, 7.03, 406.27, 0, 7, 7), (8.04, 389.20, 20.08, 406.27, 1, 11, 12),
    (21.09, 389.20, 33.13, 406.27, 1, 11, 12), (34.14, 389.20, 46.18, 406.27, 1, 11, 12), (47.19, 389.20, 59.24, 406.27, 1, 11, 12), (60.24, 389.20, 72.29, 406.27, 1, 11, 12), (73.29, 389.20, 85.34, 406.27, 1, 11, 12), (86.34, 389.20, 98.39, 406.27, 1, 11, 12),
    (99.39, 389.20, 111.44, 406.27, 1, 11, 12), (112.44, 389.20, 124.49, 406.27, 1, 11, 12), (125.49, 389.20, 137.54, 406.27, 1, 11, 12), (138.55, 389.20, 150.59, 406.27, 1, 11, 12), (151.60, 389.20, 163.64, 406.27, 1, 11, 12), (164.65, 389.20, 176.69, 406.27, 1, 11, 12),
    (177.70, 389.20, 189.75, 406.27, 1, 11, 12), (190.75, 389.20, 202.80, 406.27, 1, 11, 12), (203.80, 389.20, 215.85, 406.27, 1, 11, 12), (216.85, 389.20, 228.90, 406.27, 1, 11, 12), (-1.00, 406.27, 11.05, 423.33, 1, 11, 12), (12.05, 406.27, 24.10, 423.33, 1, 11, 12),
    (25.10, 406.27, 37.15, 423.33, 1, 11, 12), (38.15, 406.27, 50.20, 423.33, 1, 11, 12), (51.20, 406.27, 63.25, 423.33, 1, 11, 12), (64.25, 406.27, 76.30, 423.33, 1, 11, 12), (77.31, 406.27, 89.35, 423.33, 1, 11, 12), (90.36, 406.27, 102.40, 423.33, 1, 11, 12),
    (103.41, 406.27, 115.45, 423.33, 1, 11, 12), (116.46, 406.27, 128.51, 423.33, 1, 11, 12), (129.51, 406.27, 141.56, 423.33, 1, 11, 12), (142.56, 406.27, 154.61, 423.33, 1, 11, 12), (155.61, 406.27, 167.66, 423.33, 1, 11, 12), (168.66, 406.27, 180.71, 423.33, 1, 11, 12),
    (181.71, 406.27, 193.76, 423.33, 1, 11, 12), (194.76, 406.27, 206.81, 423.33, 1, 11, 12), (207.82, 406.27, 209.82, 423.33, 0, 1, 3), (210.83, 406.27, 214.84, 423.33, 1, 3, 3), (215.85, 406.27, 223.88, 423.33, 0, 7, 7), (224.88, 406.27, 232.91, 423.33, 0, 7, 7),
    (233.92, 406.27, 241.95, 423.33, 0, 7, 7), (-1.00, 423.33, 7.03, 440.40, 0, 7, 7), (8.04, 423.33, 12.05, 440.40, 2, 3, 5), (13.05, 423.33, 21.09, 440.40, 0, 7, 7), (22.09, 423.33, 29.12, 440.40, 1, 6, 7), (30.12, 423.33, 39.16, 440.40, 0, 8, 8),
    (40.16, 423.33, 46.18, 440.40, 0, 5, 5), (47.19, 423.33, 55.22, 440.40, 0, 7, 7), (56.22, 423.33, 63.25, 440.40, 0, 6, 7), (64.25, 423.33, 69.27, 440.40, 0, 4, 4), (70.28, 423.33, 79.31, 440.40, 0, 8, 8), (80.32, 423.33, 88.35, 440.40, 0, 7, 7),
    (89.35, 423.33, 94.37, 440.40, 0, 4, 5), (95.38, 423.33, 103.41, 440.40, 0, 7, 7), (104.41, 423.33, 110.44, 440.40, 0, 5, 5), (111.44, 423.33, 117.46, 440.40, 0, 5, 5), (118.47, 423.33, 123.49, 440.40, 2, 4, 6), (124.49, 423.33, 132.52, 440.40, 0, 7, 7),
    (133.53, 423.33, 142.56, 440.40, 0, 8, 8), (143.56, 423.33, 146.58, 440.40, 1, 2, 3), (147.58, 423.33, 152.60, 440.40, 2, 4, 6), (153.60, 423.33, 157.62, 440.40, 1, 3, 4), (158.62, 423.33, 164.65, 440.40, 0, 5, 5), (165.65, 423.33, 173.68, 440.40, 0, 7, 7),
    (174.69, 423.33, 185.73, 440.40, 0, 10, 10), (186.73, 423.33, 197.78, 440.40, 0, 10, 10), (198.78, 423.33, 209.82, 440.40, 0, 10, 10), (210.83, 423.33, 217.85, 440.40, 0, 6, 6), (218.86, 423.33, 227.89, 440.40, 0, 8, 7), (228.90, 423.33, 237.93, 440.40, 0, 8, 7),
    (238.94, 423.33, 247.97, 440.40, 0, 8, 7), (-1.00, 440.40, 8.04, 457.47, 0, 8, 7), (9.04, 440.40, 18.07, 457.47, 0, 8, 7), (19.08, 440.40, 28.11, 457.47, 0, 8, 7), (29.12, 440.40, 41.16, 457.47, -1, 11, 11), (42.17, 440.40, 50.20, 457.47, 0, 7, 7),
    (51.20, 440.40, 57.23, 457.47, 1, 5, 6), (58.23, 440.40, 64.25, 457.47, 1, 5, 6), (65.26, 440.40, 71.28, 457.47, 1, 5, 6), (72.29, 440.40, 79.31, 457.47, 1, 6, 6), (80.32, 440.40, 85.34, 457.47, -1, 4, 5), (86.34, 440.40, 91.36, 457.47, 0, 4, 4),
    (92.36, 440.40, 97.38, 457.47, 0, 4, 4), (98.39, 440.40, 105.42, 457.47, -1, 6, 5), (106.42, 440.40, 115.45, 457.47, 0, 8, 8), (116.46, 440.40, 123.49, 457.47, 1, 6, 7), (124.49, 440.40, 133.53, 457.47, 0, 8, 8), (134.53, 440.40, 143.56, 457.47, 0, 8, 8),
    (144.57, 440.40, 153.60, 457.47, 0, 8, 8), (154.61, 440.40, 163.64, 457.47, 0, 8, 8), (164.65, 440.40, 173.68, 457.47, 0, 8, 8), (174.69, 440.40, 182.72, 457.47, 0, 7, 7), (183.72, 440.40, 192.76, 457.47, 0, 8, 8), (193.76, 440.40, 200.79, 457.47, 1, 6, 7),
    (201.79, 440.40, 208.82, 457.47, 1, 6, 7), (209.82, 440.40, 216.85, 457.47, 1, 6, 7), (217.85, 440.40, 224.88, 457.47, 1, 6, 7), (225.89, 440.40, 233.92, 457.47, 0, 7, 6), (234.92, 440.40, 241.95, 457.47, 1, 6, 6), (-1.00, 457.47, 7.03, 474.53, 0, 7, 7),
    (8.04, 457.47, 15.06, 474.53, 0, 6, 6), (16.07, 457.47, 23.09, 474.53, 0, 6, 6), (24.10, 457.47, 31.13, 474.53, 0, 6, 6), (32.13, 457.47, 39.16, 474.53, 0, 6, 6), (40.16, 457.47, 47.19, 474.53, 0, 6, 6), (48.19, 457.47, 55.22, 474.53, 0, 6, 6),
    (56.22, 457.47, 66.26, 474.53, 0, 9, 9), (67.27, 457.47, 74.29, 474.53, 0, 6, 6), (75.30, 457.47, 82.33, 474.53, 0, 6, 6), (83.33, 457.47, 90.36, 474.53, 0, 6, 6), (91.36, 457.47, 98.39, 474.53, 0, 6, 6), (99.39, 457.47, 106.42, 474.53, 0, 6, 6),
    (107.42, 457.47, 112.44, 474.53, -1, 4, 4), (113.45, 457.47, 118.47, 474.53, 0, 4, 3), (119.47, 457.47, 124.49, 474.53, 0, 4, 3), (125.49, 457.47, 132.52, 474.53, -1, 6, 4), (133.53, 457.47, 141.56, 474.53, 0, 7, 7), (142.56, 457.47, 149.59, 474.53, 0, 6, 7),
    (150.59, 457.47, 157.62, 474.53, 0, 6, 7), (158.62, 457.47, 165.65, 474.53, 0, 6, 7), (166.65, 457.47, 173.68, 474.53, 0, 6, 7), (174.69, 457.47, 181.71, 474.53, 0, 6, 7), (182.72, 457.47, 189.75, 474.53, 0, 6, 7), (190.75, 457.47, 198.78, 474.53, 0, 7, 7),
    (199.78, 457.47, 207.82, 474.53, 0, 7, 7), (208.82, 457.47, 215.85, 474.53, 0, 6, 7), (216.85, 457.47, 223.88, 474.53, 0, 6, 7), (224.88, 457.47, 231.91, 474.53, 0, 6, 7), (232.91, 457.47, 239.94, 474.53, 0, 6, 7), (240.95, 457.47, 248.98, 474.53, 0, 7, 6),
    (-1.00, 474.53, 7.03, 491.60, 0, 7, 7), (8.04, 474.53, 16.07, 491.60, 0, 7, 6),
];
/// The 12-point record's glyphs (see BODY12): the cell's corners on h_e4e4d2f4 in pixels, offset,
/// width, advance; characters 32-255. Read from default.xbe 0x3c2088.
const REC12_GLYPHS: [(f32, f32, f32, f32, i8, u8, u8); 224] = [
    (-1.00, 298.00, 1.01, 318.08, 0, 1, 4), (2.01, 298.00, 6.03, 318.08, 1, 3, 3), (7.03, 298.00, 14.06, 318.08, 0, 6, 6), (15.06, 298.00, 24.10, 318.08, 0, 8, 8), (25.10, 298.00, 34.14, 318.08, 0, 8, 8), (35.14, 298.00, 48.19, 318.08, 0, 12, 12),
    (49.20, 298.00, 60.24, 318.08, 0, 10, 10), (61.24, 298.00, 65.26, 318.08, 0, 3, 3), (66.26, 298.00, 73.29, 318.08, 0, 6, 6), (74.29, 298.00, 81.32, 318.08, 0, 6, 6), (82.33, 298.00, 90.36, 318.08, 0, 7, 8), (91.36, 298.00, 100.40, 318.08, 0, 8, 8),
    (101.40, 298.00, 106.42, 318.08, 0, 4, 4), (107.42, 298.00, 113.45, 318.08, 0, 5, 5), (114.45, 298.00, 119.47, 318.08, 0, 4, 4), (120.47, 298.00, 128.51, 318.08, 0, 7, 7), (129.51, 298.00, 138.55, 318.08, 0, 8, 8), (139.55, 298.00, 145.57, 318.08, 1, 5, 7),
    (146.58, 298.00, 154.61, 318.08, 1, 7, 7), (155.61, 298.00, 164.65, 318.08, 0, 8, 8), (165.65, 298.00, 174.69, 318.08, 0, 8, 8), (175.69, 298.00, 184.73, 318.08, 0, 8, 8), (185.73, 298.00, 194.76, 318.08, 0, 8, 8), (195.77, 298.00, 204.80, 318.08, 0, 8, 8),
    (205.81, 298.00, 214.84, 318.08, 0, 8, 8), (215.85, 298.00, 224.88, 318.08, 0, 8, 8), (225.89, 298.00, 230.91, 318.08, 0, 4, 5), (231.91, 298.00, 236.93, 318.08, 0, 4, 4), (237.93, 298.00, 244.96, 318.08, 1, 6, 7), (-1.00, 318.08, 7.03, 338.16, 1, 7, 7),
    (8.04, 318.08, 15.06, 338.16, 1, 6, 7), (16.07, 318.08, 24.10, 338.16, 0, 7, 7), (25.10, 318.08, 39.16, 338.16, 0, 13, 13), (40.16, 318.08, 50.20, 338.16, 0, 9, 9), (51.20, 318.08, 60.24, 338.16, 1, 8, 8), (61.24, 318.08, 71.28, 338.16, 0, 9, 9),
    (72.29, 318.08, 81.32, 338.16, 1, 8, 9), (82.33, 318.08, 90.36, 338.16, 1, 7, 7), (91.36, 318.08, 99.39, 338.16, 1, 7, 7), (100.40, 318.08, 110.44, 338.16, 0, 9, 9), (111.44, 318.08, 120.47, 338.16, 1, 8, 9), (121.48, 318.08, 125.49, 338.16, 1, 3, 4),
    (126.50, 318.08, 134.53, 338.16, 0, 7, 7), (135.53, 318.08, 144.57, 338.16, 1, 8, 8), (145.57, 318.08, 153.60, 338.16, 1, 7, 6), (154.61, 318.08, 164.65, 338.16, 1, 9, 10), (165.65, 318.08, 174.69, 338.16, 1, 8, 9), (175.69, 318.08, 185.73, 338.16, 0, 9, 10),
    (186.73, 318.08, 194.76, 338.16, 1, 7, 8), (195.77, 318.08, 205.81, 338.16, 0, 9, 10), (206.81, 318.08, 215.85, 338.16, 1, 8, 8), (216.85, 318.08, 225.89, 338.16, 0, 8, 8), (226.89, 318.08, 234.92, 338.16, 0, 7, 7), (-1.00, 338.16, 8.04, 358.24, 1, 8, 9),
    (9.04, 338.16, 19.08, 358.24, 0, 9, 9), (20.08, 338.16, 34.14, 358.24, 0, 13, 12), (35.14, 338.16, 44.18, 358.24, 0, 8, 8), (45.18, 338.16, 54.22, 358.24, 0, 8, 8), (55.22, 338.16, 64.25, 358.24, 0, 8, 8), (65.26, 338.16, 70.28, 358.24, 1, 4, 4),
    (71.28, 338.16, 79.31, 358.24, 0, 7, 7), (80.32, 338.16, 85.34, 358.24, 0, 4, 5), (86.34, 338.16, 95.38, 358.24, 0, 8, 8), (96.38, 338.16, 105.42, 358.24, 0, 8, 8), (106.42, 338.16, 111.44, 358.24, 3, 4, 7), (112.44, 338.16, 120.47, 358.24, 0, 7, 8),
    (121.48, 338.16, 129.51, 358.24, 1, 7, 7), (130.51, 338.16, 138.55, 358.24, 0, 7, 7), (139.55, 338.16, 148.58, 358.24, 0, 8, 8), (149.59, 338.16, 157.62, 358.24, 0, 7, 8), (158.62, 338.16, 165.65, 358.24, 0, 6, 5), (166.65, 338.16, 175.69, 358.24, 0, 8, 8),
    (176.69, 338.16, 184.73, 358.24, 1, 7, 7), (185.73, 338.16, 189.75, 358.24, 1, 3, 3), (190.75, 338.16, 196.77, 358.24, -1, 5, 5), (197.78, 338.16, 205.81, 358.24, 1, 7, 7), (206.81, 338.16, 210.83, 358.24, 1, 3, 3), (211.83, 338.16, 223.88, 358.24, 1, 11, 11),
    (224.88, 338.16, 232.91, 358.24, 1, 7, 7), (233.92, 338.16, 242.95, 358.24, 0, 8, 8), (-1.00, 358.24, 7.03, 378.31, 1, 7, 7), (8.04, 358.24, 17.07, 378.31, 0, 8, 8), (18.07, 358.24, 24.10, 378.31, 1, 5, 5), (25.10, 358.24, 33.13, 378.31, 0, 7, 7),
    (34.14, 358.24, 41.16, 378.31, 0, 6, 5), (42.17, 358.24, 50.20, 378.31, 1, 7, 7), (51.20, 358.24, 59.24, 378.31, 0, 7, 7), (60.24, 358.24, 72.29, 378.31, 0, 11, 11), (73.29, 358.24, 82.33, 378.31, 0, 8, 8), (83.33, 358.24, 92.36, 378.31, 0, 8, 8),
    (93.37, 358.24, 101.40, 378.31, 0, 7, 7), (102.40, 358.24, 108.43, 378.31, 0, 5, 5), (109.43, 358.24, 113.45, 378.31, 3, 3, 5), (114.45, 358.24, 120.47, 378.31, 0, 5, 5), (121.48, 358.24, 130.51, 378.31, 0, 8, 8), (131.52, 358.24, 144.57, 378.31, 2, 12, 14),
    (145.57, 358.24, 158.62, 378.31, 2, 12, 14), (159.63, 358.24, 172.68, 378.31, 2, 12, 14), (173.68, 358.24, 186.73, 378.31, 2, 12, 14), (187.74, 358.24, 200.79, 378.31, 2, 12, 14), (201.79, 358.24, 214.84, 378.31, 2, 12, 14), (215.85, 358.24, 228.90, 378.31, 2, 12, 14),
    (-1.00, 378.31, 12.05, 398.39, 2, 12, 14), (13.05, 378.31, 26.11, 398.39, 2, 12, 14), (27.11, 378.31, 40.16, 398.39, 2, 12, 14), (41.16, 378.31, 54.22, 398.39, 2, 12, 14), (55.22, 378.31, 68.27, 398.39, 2, 12, 14), (69.27, 378.31, 82.33, 398.39, 2, 12, 14),
    (83.33, 378.31, 96.38, 398.39, 2, 12, 14), (97.38, 378.31, 110.44, 398.39, 2, 12, 14), (111.44, 378.31, 124.49, 398.39, 2, 12, 14), (125.49, 378.31, 138.55, 398.39, 2, 12, 14), (139.55, 378.31, 152.60, 398.39, 2, 12, 14), (153.60, 378.31, 166.65, 398.39, 2, 12, 14),
    (167.66, 378.31, 180.71, 398.39, 2, 12, 14), (181.71, 378.31, 194.76, 398.39, 2, 12, 14), (195.77, 378.31, 208.82, 398.39, 2, 12, 14), (209.82, 378.31, 222.87, 398.39, 2, 12, 14), (-1.00, 398.39, 12.05, 418.47, 2, 12, 14), (13.05, 398.39, 26.11, 418.47, 2, 12, 14),
    (27.11, 398.39, 40.16, 418.47, 2, 12, 14), (41.16, 398.39, 54.22, 418.47, 2, 12, 14), (55.22, 398.39, 68.27, 418.47, 2, 12, 14), (69.27, 398.39, 82.33, 418.47, 2, 12, 14), (83.33, 398.39, 96.38, 418.47, 2, 12, 14), (97.38, 398.39, 110.44, 418.47, 2, 12, 14),
    (111.44, 398.39, 124.49, 418.47, 2, 12, 14), (125.49, 398.39, 138.55, 418.47, 2, 12, 14), (139.55, 398.39, 141.56, 418.47, 0, 1, 4), (142.56, 398.39, 146.58, 418.47, 0, 3, 4), (147.58, 398.39, 155.61, 418.47, 1, 7, 7), (156.62, 398.39, 165.65, 418.47, 0, 8, 8),
    (166.65, 398.39, 175.69, 418.47, 0, 8, 8), (176.69, 398.39, 186.73, 418.47, 0, 9, 9), (187.74, 398.39, 191.75, 418.47, 3, 3, 5), (192.76, 398.39, 201.79, 418.47, 0, 8, 8), (202.80, 398.39, 209.82, 418.47, 2, 6, 8), (210.83, 398.39, 221.87, 418.47, 0, 10, 10),
    (222.87, 398.39, 228.90, 418.47, 0, 5, 6), (229.90, 398.39, 238.94, 418.47, 0, 8, 9), (-1.00, 418.47, 8.04, 438.55, 0, 8, 8), (9.04, 418.47, 15.06, 438.55, 0, 5, 5), (16.07, 418.47, 27.11, 438.55, 0, 10, 10), (28.11, 418.47, 37.15, 438.55, 0, 8, 8),
    (38.15, 418.47, 44.18, 438.55, 0, 5, 6), (45.18, 418.47, 54.22, 438.55, 0, 8, 8), (55.22, 418.47, 62.25, 438.55, 0, 6, 6), (63.25, 418.47, 70.28, 438.55, 0, 6, 6), (71.28, 418.47, 76.30, 438.55, 3, 4, 7), (77.31, 418.47, 86.34, 438.55, 0, 8, 8),
    (87.35, 418.47, 97.38, 438.55, 0, 9, 9), (98.39, 418.47, 102.40, 438.55, 1, 3, 4), (103.41, 418.47, 108.43, 438.55, 3, 4, 7), (109.43, 418.47, 114.45, 438.55, 1, 4, 5), (115.45, 418.47, 122.48, 438.55, 0, 6, 6), (123.49, 418.47, 132.52, 438.55, 1, 8, 8),
    (133.53, 418.47, 146.58, 438.55, 0, 12, 13), (147.58, 418.47, 161.64, 438.55, 0, 13, 13), (162.64, 418.47, 175.69, 438.55, 0, 12, 13), (176.69, 418.47, 184.73, 438.55, 0, 7, 7), (185.73, 418.47, 195.77, 438.55, 0, 9, 9), (196.77, 418.47, 206.81, 438.55, 0, 9, 9),
    (207.82, 418.47, 217.85, 438.55, 0, 9, 9), (218.86, 418.47, 228.90, 438.55, 0, 9, 9), (229.90, 418.47, 239.94, 438.55, 0, 9, 9), (-1.00, 438.55, 9.04, 458.63, 0, 9, 9), (10.04, 438.55, 24.10, 458.63, -1, 13, 14), (25.10, 438.55, 35.14, 458.63, 0, 9, 9),
    (36.15, 438.55, 44.18, 458.63, 1, 7, 7), (45.18, 438.55, 53.21, 458.63, 1, 7, 7), (54.22, 438.55, 62.25, 458.63, 1, 7, 7), (63.25, 438.55, 71.28, 458.63, 1, 7, 7), (72.29, 438.55, 77.31, 458.63, 0, 4, 5), (78.31, 438.55, 83.33, 458.63, 1, 4, 4),
    (84.33, 438.55, 91.36, 458.63, -1, 6, 6), (92.36, 438.55, 99.39, 458.63, -1, 6, 6), (100.40, 438.55, 110.44, 458.63, 0, 9, 10), (111.44, 438.55, 120.47, 458.63, 1, 8, 9), (121.48, 438.55, 131.52, 458.63, 0, 9, 10), (132.52, 438.55, 142.56, 458.63, 0, 9, 10),
    (143.56, 438.55, 153.60, 458.63, 0, 9, 10), (154.61, 438.55, 164.65, 458.63, 0, 9, 10), (165.65, 438.55, 175.69, 458.63, 0, 9, 10), (176.69, 438.55, 185.73, 458.63, 0, 8, 8), (186.73, 438.55, 197.78, 458.63, 0, 10, 10), (198.78, 438.55, 207.82, 458.63, 1, 8, 9),
    (208.82, 438.55, 217.85, 458.63, 1, 8, 9), (218.86, 438.55, 227.89, 458.63, 1, 8, 9), (228.90, 438.55, 237.93, 458.63, 1, 8, 9), (238.94, 438.55, 247.97, 458.63, 0, 8, 8), (-1.00, 458.63, 7.03, 478.71, 1, 7, 8), (8.04, 458.63, 17.07, 478.71, 1, 8, 8),
    (18.07, 458.63, 26.11, 478.71, 0, 7, 8), (27.11, 458.63, 35.14, 478.71, 0, 7, 8), (36.15, 458.63, 44.18, 478.71, 0, 7, 8), (45.18, 458.63, 53.21, 478.71, 0, 7, 8), (54.22, 458.63, 62.25, 478.71, 0, 7, 8), (63.25, 458.63, 71.28, 478.71, 0, 7, 8),
    (72.29, 458.63, 84.33, 478.71, 0, 11, 11), (85.34, 458.63, 93.37, 478.71, 0, 7, 7), (94.37, 458.63, 102.40, 478.71, 0, 7, 8), (103.41, 458.63, 111.44, 478.71, 0, 7, 8), (112.44, 458.63, 120.47, 478.71, 0, 7, 8), (121.48, 458.63, 129.51, 478.71, 0, 7, 8),
    (130.51, 458.63, 135.53, 478.71, 0, 4, 4), (136.54, 458.63, 141.56, 478.71, 1, 4, 3), (142.56, 458.63, 149.59, 478.71, -1, 6, 5), (150.59, 458.63, 157.62, 478.71, -1, 6, 5), (158.62, 458.63, 167.66, 478.71, 0, 8, 8), (168.66, 458.63, 176.69, 478.71, 1, 7, 7),
    (177.70, 458.63, 186.73, 478.71, 0, 8, 8), (187.74, 458.63, 196.77, 478.71, 0, 8, 8), (197.78, 458.63, 206.81, 478.71, 0, 8, 8), (207.82, 458.63, 216.85, 478.71, 0, 8, 8), (217.85, 458.63, 226.89, 478.71, 0, 8, 8), (227.89, 458.63, 236.93, 478.71, 0, 8, 8),
    (237.93, 458.63, 247.97, 478.71, 0, 9, 8), (-1.00, 478.71, 7.03, 498.78, 1, 7, 7), (8.04, 478.71, 16.07, 498.78, 1, 7, 7), (17.07, 478.71, 25.10, 498.78, 1, 7, 7), (26.11, 478.71, 34.14, 498.78, 1, 7, 7), (35.14, 478.71, 44.18, 498.78, 0, 8, 8),
    (45.18, 478.71, 53.21, 498.78, 1, 7, 7), (54.22, 478.71, 63.25, 498.78, 0, 8, 8),
];

/// The 14-point record's glyphs (see BODY14): on h_e4e4d2f4's top size; characters 32-255. Read
/// from default.xbe 0x3c35a8.
const REC14_GLYPHS: [(f32, f32, f32, f32, i8, u8, u8); 224] = [
    (-1.00, -1.00, 1.01, 23.05, 0, 1, 4), (2.01, -1.00, 7.03, 23.05, 1, 4, 4), (8.04, -1.00, 17.07, 23.05, 0, 8, 8), (18.07, -1.00, 29.12, 23.05, 0, 10, 10), (30.12, -1.00, 39.16, 23.05, 1, 8, 9), (40.16, -1.00, 55.22, 23.05, 0, 14, 14),
    (56.22, -1.00, 68.27, 23.05, 0, 11, 11), (69.27, -1.00, 74.29, 23.05, 0, 4, 4), (75.30, -1.00, 83.33, 23.05, 0, 7, 7), (84.33, -1.00, 92.36, 23.05, 0, 7, 7), (93.37, -1.00, 103.41, 23.05, 0, 9, 9), (104.41, -1.00, 113.45, 23.05, 1, 8, 9),
    (114.45, -1.00, 120.47, 23.05, 0, 5, 5), (121.48, -1.00, 128.51, 23.05, 0, 6, 6), (129.51, -1.00, 134.53, 23.05, 0, 4, 5), (135.53, -1.00, 145.57, 23.05, 0, 9, 8), (146.58, -1.00, 157.62, 23.05, 0, 10, 10), (158.62, -1.00, 165.65, 23.05, 1, 6, 9),
    (166.65, -1.00, 175.69, 23.05, 1, 8, 9), (176.69, -1.00, 187.74, 23.05, 0, 10, 10), (188.74, -1.00, 199.78, 23.05, 0, 10, 10), (200.79, -1.00, 211.83, 23.05, 0, 10, 10), (212.84, -1.00, 223.88, 23.05, 0, 10, 10), (224.88, -1.00, 235.93, 23.05, 0, 10, 10),
    (-1.00, 23.05, 10.04, 47.09, 0, 10, 10), (11.05, 23.05, 22.09, 47.09, 0, 10, 10), (23.09, 23.05, 28.11, 47.09, 0, 4, 6), (29.12, 23.05, 35.14, 47.09, 0, 5, 5), (36.15, 23.05, 45.18, 47.09, 1, 8, 9), (46.18, 23.05, 55.22, 47.09, 1, 8, 9),
    (56.22, 23.05, 65.26, 47.09, 1, 8, 9), (66.26, 23.05, 76.30, 47.09, 0, 9, 9), (77.31, 23.05, 92.36, 47.09, 1, 14, 14), (93.37, 23.05, 105.42, 47.09, 0, 11, 11), (106.42, 23.05, 117.46, 47.09, 1, 10, 10), (118.47, 23.05, 129.51, 47.09, 1, 10, 10),
    (130.51, 23.05, 141.56, 47.09, 1, 10, 11), (142.56, 23.05, 151.60, 47.09, 1, 8, 9), (152.60, 23.05, 161.64, 47.09, 1, 8, 8), (162.64, 23.05, 172.68, 47.09, 1, 9, 10), (173.68, 23.05, 183.72, 47.09, 1, 9, 11), (184.73, 23.05, 188.74, 47.09, 1, 3, 4),
    (189.75, 23.05, 198.78, 47.09, 0, 8, 9), (199.78, 23.05, 210.83, 47.09, 1, 10, 9), (211.83, 23.05, 220.87, 47.09, 1, 8, 8), (221.87, 23.05, 233.92, 47.09, 1, 11, 12), (-1.00, 47.09, 9.04, 71.14, 1, 9, 10), (10.04, 47.09, 21.09, 71.14, 1, 10, 11),
    (22.09, 47.09, 32.13, 71.14, 1, 9, 9), (33.13, 47.09, 44.18, 71.14, 1, 10, 11), (45.18, 47.09, 56.22, 71.14, 1, 10, 9), (57.23, 47.09, 67.27, 71.14, 0, 9, 10), (68.27, 47.09, 78.31, 71.14, 0, 9, 9), (79.31, 47.09, 89.35, 71.14, 1, 9, 10),
    (90.36, 47.09, 102.40, 71.14, 0, 11, 11), (103.41, 47.09, 119.47, 71.14, 0, 15, 15), (120.47, 47.09, 131.52, 71.14, 0, 10, 10), (132.52, 47.09, 143.56, 71.14, 0, 10, 9), (144.57, 47.09, 154.61, 71.14, 0, 9, 9), (155.61, 47.09, 160.63, 71.14, 1, 4, 5),
    (161.64, 47.09, 171.67, 71.14, 0, 9, 8), (172.68, 47.09, 177.70, 71.14, 1, 4, 5), (178.70, 47.09, 189.75, 71.14, 0, 10, 10), (190.75, 47.09, 201.79, 71.14, 0, 10, 10), (202.80, 47.09, 208.82, 71.14, 3, 5, 9), (209.82, 47.09, 218.86, 71.14, 0, 8, 9),
    (219.86, 47.09, 228.90, 71.14, 1, 8, 9), (229.90, 47.09, 239.94, 71.14, 0, 9, 9), (-1.00, 71.14, 9.04, 95.19, 0, 9, 10), (10.04, 71.14, 20.08, 95.19, 0, 9, 9), (21.09, 71.14, 29.12, 95.19, 0, 7, 6), (30.12, 71.14, 40.16, 95.19, 0, 9, 10),
    (41.16, 71.14, 50.20, 95.19, 1, 8, 9), (51.20, 71.14, 55.22, 95.19, 1, 3, 4), (56.22, 71.14, 62.25, 95.19, -1, 5, 6), (63.25, 71.14, 73.29, 95.19, 1, 9, 8), (74.29, 71.14, 78.31, 95.19, 1, 3, 4), (79.31, 71.14, 93.37, 95.19, 1, 13, 14),
    (94.37, 71.14, 103.41, 95.19, 1, 8, 9), (104.41, 71.14, 114.45, 95.19, 0, 9, 10), (115.45, 71.14, 124.49, 95.19, 1, 8, 9), (125.49, 71.14, 135.53, 95.19, 0, 9, 10), (136.54, 71.14, 143.56, 95.19, 1, 6, 6), (144.57, 71.14, 153.60, 95.19, 0, 8, 8),
    (154.61, 71.14, 162.64, 95.19, 0, 7, 6), (163.64, 71.14, 172.68, 95.19, 1, 8, 9), (173.68, 71.14, 183.72, 95.19, 0, 9, 8), (184.73, 71.14, 198.78, 95.19, 0, 13, 13), (199.78, 71.14, 209.82, 95.19, 0, 9, 9), (210.83, 71.14, 220.87, 95.19, 0, 9, 9),
    (221.87, 71.14, 230.91, 95.19, 0, 8, 8), (231.91, 71.14, 238.94, 95.19, 0, 6, 6), (-1.00, 95.19, 2.01, 119.23, 4, 2, 6), (3.02, 95.19, 10.04, 119.23, 0, 6, 6), (11.05, 95.19, 22.09, 119.23, 0, 10, 10), (23.09, 95.19, 39.16, 119.23, 2, 15, 17),
    (40.16, 95.19, 56.22, 119.23, 2, 15, 17), (57.23, 95.19, 73.29, 119.23, 2, 15, 17), (74.29, 95.19, 90.36, 119.23, 2, 15, 17), (91.36, 95.19, 107.42, 119.23, 2, 15, 17), (108.43, 95.19, 124.49, 119.23, 2, 15, 17), (125.49, 95.19, 141.56, 119.23, 2, 15, 17),
    (142.56, 95.19, 158.62, 119.23, 2, 15, 17), (159.63, 95.19, 175.69, 119.23, 2, 15, 17), (176.69, 95.19, 192.76, 119.23, 2, 15, 17), (193.76, 95.19, 209.82, 119.23, 2, 15, 17), (210.83, 95.19, 226.89, 119.23, 2, 15, 17), (-1.00, 119.23, 15.06, 143.28, 2, 15, 17),
    (16.07, 119.23, 32.13, 143.28, 2, 15, 17), (33.13, 119.23, 49.20, 143.28, 2, 15, 17), (50.20, 119.23, 66.26, 143.28, 2, 15, 17), (67.27, 119.23, 83.33, 143.28, 2, 15, 17), (84.33, 119.23, 100.40, 143.28, 2, 15, 17), (101.40, 119.23, 117.46, 143.28, 2, 15, 17),
    (118.47, 119.23, 134.53, 143.28, 2, 15, 17), (135.53, 119.23, 151.60, 143.28, 2, 15, 17), (152.60, 119.23, 168.66, 143.28, 2, 15, 17), (169.67, 119.23, 185.73, 143.28, 2, 15, 17), (186.73, 119.23, 202.80, 143.28, 2, 15, 17), (203.80, 119.23, 219.86, 143.28, 2, 15, 17),
    (-1.00, 143.28, 15.06, 167.33, 2, 15, 17), (16.07, 143.28, 32.13, 167.33, 2, 15, 17), (33.13, 143.28, 49.20, 167.33, 2, 15, 17), (50.20, 143.28, 66.26, 167.33, 2, 15, 17), (67.27, 143.28, 83.33, 167.33, 2, 15, 17), (84.33, 143.28, 100.40, 167.33, 2, 15, 17),
    (101.40, 143.28, 117.46, 167.33, 2, 15, 17), (118.47, 143.28, 134.53, 167.33, 2, 15, 17), (135.53, 143.28, 137.54, 167.33, 0, 1, 4), (138.55, 143.28, 143.56, 167.33, 0, 4, 5), (144.57, 143.28, 153.60, 167.33, 1, 8, 9), (154.61, 143.28, 164.65, 167.33, 1, 9, 9),
    (165.65, 143.28, 176.69, 167.33, 0, 10, 10), (177.70, 143.28, 188.74, 167.33, 0, 10, 11), (189.75, 143.28, 192.76, 167.33, 4, 2, 6), (193.76, 143.28, 204.80, 167.33, 0, 10, 10), (205.81, 143.28, 213.84, 167.33, 2, 7, 10), (214.84, 143.28, 227.89, 167.33, 0, 12, 12),
    (228.90, 143.28, 234.92, 167.33, 1, 5, 6), (-1.00, 167.33, 10.04, 191.38, 0, 10, 10), (11.05, 167.33, 20.08, 191.38, 1, 8, 9), (21.09, 167.33, 28.11, 191.38, 0, 6, 6), (29.12, 167.33, 42.17, 191.38, 0, 12, 12), (43.17, 167.33, 54.22, 191.38, 0, 10, 10),
    (55.22, 167.33, 62.25, 191.38, 0, 6, 7), (63.25, 167.33, 72.29, 191.38, 1, 8, 9), (73.29, 167.33, 80.32, 191.38, 1, 6, 7), (81.32, 167.33, 89.35, 191.38, 0, 7, 8), (90.36, 167.33, 96.38, 191.38, 3, 5, 9), (97.38, 167.33, 108.43, 191.38, 0, 10, 10),
    (109.43, 167.33, 121.48, 191.38, 0, 11, 11), (122.48, 167.33, 127.50, 191.38, 1, 4, 5), (128.51, 167.33, 134.53, 191.38, 3, 5, 9), (135.53, 167.33, 140.55, 191.38, 1, 4, 7), (141.56, 167.33, 149.59, 191.38, 0, 7, 7), (150.59, 167.33, 161.64, 191.38, 0, 10, 10),
    (162.64, 167.33, 177.70, 191.38, 0, 14, 15), (178.70, 167.33, 194.76, 191.38, 0, 15, 15), (195.77, 167.33, 212.84, 191.38, -1, 16, 16), (213.84, 167.33, 223.88, 191.38, 0, 9, 9), (224.88, 167.33, 236.93, 191.38, 0, 11, 11), (-1.00, 191.38, 11.05, 215.42, 0, 11, 11),
    (12.05, 191.38, 24.10, 215.42, 0, 11, 11), (25.10, 191.38, 37.15, 215.42, 0, 11, 11), (38.15, 191.38, 50.20, 215.42, 0, 11, 11), (51.20, 191.38, 63.25, 215.42, 0, 11, 11), (64.25, 191.38, 81.32, 215.42, -1, 16, 16), (82.33, 191.38, 93.37, 215.42, 1, 10, 10),
    (94.37, 191.38, 103.41, 215.42, 1, 8, 9), (104.41, 191.38, 113.45, 215.42, 1, 8, 9), (114.45, 191.38, 123.49, 215.42, 1, 8, 9), (124.49, 191.38, 133.53, 215.42, 1, 8, 9), (134.53, 191.38, 140.55, 215.42, -1, 5, 6), (141.56, 191.38, 147.58, 215.42, 0, 5, 5),
    (148.58, 191.38, 156.62, 215.42, -1, 7, 6), (157.62, 191.38, 165.65, 215.42, -1, 7, 6), (166.65, 191.38, 178.70, 215.42, 0, 11, 11), (179.71, 191.38, 189.75, 215.42, 1, 9, 10), (190.75, 191.38, 201.79, 215.42, 1, 10, 11), (202.80, 191.38, 213.84, 215.42, 1, 10, 11),
    (214.84, 191.38, 225.89, 215.42, 1, 10, 11), (226.89, 191.38, 237.93, 215.42, 1, 10, 11), (-1.00, 215.42, 10.04, 239.47, 1, 10, 11), (11.05, 215.42, 22.09, 239.47, 0, 10, 10), (23.09, 215.42, 36.15, 239.47, 0, 12, 12), (37.15, 215.42, 47.19, 239.47, 1, 9, 10),
    (48.19, 215.42, 58.23, 239.47, 1, 9, 10), (59.24, 215.42, 69.27, 239.47, 1, 9, 10), (70.28, 215.42, 80.32, 239.47, 1, 9, 10), (81.32, 215.42, 92.36, 239.47, 0, 10, 9), (93.37, 215.42, 103.41, 239.47, 1, 9, 9), (104.41, 215.42, 114.45, 239.47, 1, 9, 10),
    (115.45, 215.42, 124.49, 239.47, 0, 8, 9), (125.49, 215.42, 134.53, 239.47, 0, 8, 9), (135.53, 215.42, 144.57, 239.47, 0, 8, 9), (145.57, 215.42, 154.61, 239.47, 0, 8, 9), (155.61, 215.42, 164.65, 239.47, 0, 8, 9), (165.65, 215.42, 174.69, 239.47, 0, 8, 9),
    (175.69, 215.42, 189.75, 239.47, 0, 13, 13), (190.75, 215.42, 200.79, 239.47, 0, 9, 9), (201.79, 215.42, 211.83, 239.47, 0, 9, 9), (212.84, 215.42, 222.87, 239.47, 0, 9, 9), (223.88, 215.42, 233.92, 239.47, 0, 9, 9), (234.92, 215.42, 244.96, 239.47, 0, 9, 9),
    (-1.00, 239.47, 5.02, 263.52, -1, 5, 6), (6.03, 239.47, 12.05, 263.52, 0, 5, 5), (13.05, 239.47, 21.09, 263.52, -1, 7, 6), (22.09, 239.47, 30.12, 263.52, -1, 7, 6), (31.13, 239.47, 41.16, 263.52, 0, 9, 10), (42.17, 239.47, 51.20, 263.52, 1, 8, 9),
    (52.21, 239.47, 62.25, 263.52, 0, 9, 10), (63.25, 239.47, 73.29, 263.52, 0, 9, 10), (74.29, 239.47, 84.33, 263.52, 0, 9, 10), (85.34, 239.47, 95.38, 263.52, 0, 9, 10), (96.38, 239.47, 106.42, 263.52, 0, 9, 10), (107.42, 239.47, 116.46, 263.52, 1, 8, 9),
    (117.46, 239.47, 128.51, 263.52, 0, 10, 10), (129.51, 239.47, 138.55, 263.52, 1, 8, 9), (139.55, 239.47, 148.58, 263.52, 1, 8, 9), (149.59, 239.47, 158.62, 263.52, 1, 8, 9), (159.63, 239.47, 168.66, 263.52, 1, 8, 9), (169.67, 239.47, 179.71, 263.52, 0, 9, 9),
    (180.71, 239.47, 189.75, 263.52, 1, 8, 9), (190.75, 239.47, 200.79, 263.52, 0, 9, 9),
];

/// The 16-point record's glyphs (see BODY16): on h_f09bc36b; characters 32-255. Read from
/// default.xbe 0x3c4ac8.
const REC16_GLYPHS: [(f32, f32, f32, f32, i8, u8, u8); 224] = [
    (-1.00, -1.00, 1.01, 25.05, 0, 1, 5), (2.01, -1.00, 7.03, 25.05, 1, 4, 5), (8.04, -1.00, 17.07, 25.05, 0, 8, 8), (18.07, -1.00, 30.12, 25.05, 0, 11, 11), (31.13, -1.00, 41.16, 25.05, 1, 9, 10), (42.17, -1.00, 58.23, 25.05, 0, 15, 15),
    (59.24, -1.00, 73.29, 25.05, 0, 13, 12), (74.29, -1.00, 79.31, 25.05, 0, 4, 4), (80.32, -1.00, 88.35, 25.05, 0, 7, 7), (89.35, -1.00, 97.38, 25.05, 0, 7, 7), (98.39, -1.00, 109.43, 25.05, 0, 10, 10), (110.44, -1.00, 120.47, 25.05, 1, 9, 10),
    (121.48, -1.00, 127.50, 25.05, 0, 5, 6), (128.51, -1.00, 135.53, 25.05, 0, 6, 7), (136.54, -1.00, 141.56, 25.05, 1, 4, 4), (142.56, -1.00, 153.60, 25.05, 0, 10, 9), (154.61, -1.00, 166.65, 25.05, 0, 11, 11), (167.66, -1.00, 175.69, 25.05, 1, 7, 10),
    (176.69, -1.00, 186.73, 25.05, 1, 9, 10), (187.74, -1.00, 198.78, 25.05, 0, 10, 11), (199.78, -1.00, 211.83, 25.05, 0, 11, 11), (212.84, -1.00, 224.88, 25.05, 0, 11, 11), (225.89, -1.00, 237.93, 25.05, 0, 11, 11), (-1.00, 25.05, 11.05, 51.10, 0, 11, 11),
    (12.05, 25.05, 24.10, 51.10, 0, 11, 11), (25.10, 25.05, 37.15, 51.10, 0, 11, 11), (38.15, 25.05, 43.17, 51.10, 1, 4, 5), (44.18, 25.05, 51.20, 51.10, 0, 6, 6), (52.21, 25.05, 62.25, 51.10, 1, 9, 10), (63.25, 25.05, 73.29, 51.10, 1, 9, 10),
    (74.29, 25.05, 84.33, 51.10, 1, 9, 10), (85.34, 25.05, 95.38, 51.10, 0, 9, 10), (96.38, 25.05, 112.44, 51.10, 1, 15, 16), (113.45, 25.05, 126.50, 51.10, 0, 12, 12), (127.50, 25.05, 139.55, 51.10, 1, 11, 11), (140.55, 25.05, 152.60, 51.10, 1, 11, 11),
    (153.60, 25.05, 165.65, 51.10, 1, 11, 12), (166.65, 25.05, 176.69, 51.10, 1, 9, 10), (177.70, 25.05, 187.74, 51.10, 1, 9, 9), (188.74, 25.05, 199.78, 51.10, 1, 10, 11), (200.79, 25.05, 212.84, 51.10, 1, 11, 12), (213.84, 25.05, 218.86, 51.10, 1, 4, 5),
    (219.86, 25.05, 229.90, 51.10, 0, 9, 10), (230.91, 25.05, 242.95, 51.10, 1, 11, 11), (-1.00, 51.10, 9.04, 77.15, 1, 9, 9), (10.04, 51.10, 23.09, 77.15, 1, 12, 13), (24.10, 51.10, 35.14, 77.15, 1, 10, 12), (36.15, 51.10, 48.19, 77.15, 1, 11, 12),
    (49.20, 51.10, 60.24, 77.15, 1, 10, 10), (61.24, 51.10, 73.29, 77.15, 1, 11, 12), (74.29, 51.10, 86.34, 77.15, 1, 11, 11), (87.35, 51.10, 98.39, 77.15, 0, 10, 11), (99.39, 51.10, 110.44, 77.15, 0, 10, 10), (111.44, 51.10, 122.48, 77.15, 1, 10, 12),
    (123.49, 51.10, 136.54, 77.15, 0, 12, 12), (137.54, 51.10, 155.61, 77.15, 0, 17, 16), (156.62, 51.10, 168.66, 77.15, 0, 11, 11), (169.67, 51.10, 181.71, 77.15, 0, 11, 10), (182.72, 51.10, 193.76, 77.15, 0, 10, 10), (194.76, 51.10, 200.79, 77.15, 1, 5, 5),
    (201.79, 51.10, 212.84, 77.15, 0, 10, 9), (213.84, 51.10, 219.86, 77.15, 0, 5, 6), (220.87, 51.10, 232.91, 77.15, 0, 11, 11), (-1.00, 77.15, 11.05, 103.20, 0, 11, 11), (12.05, 77.15, 18.07, 103.20, 4, 5, 9), (19.08, 77.15, 29.12, 103.20, 0, 9, 10),
    (30.12, 77.15, 40.16, 103.20, 1, 9, 10), (41.16, 77.15, 50.20, 103.20, 1, 8, 9), (51.20, 77.15, 61.24, 103.20, 1, 9, 10), (62.25, 77.15, 72.29, 103.20, 1, 9, 9), (73.29, 77.15, 81.32, 103.20, 0, 7, 6), (82.33, 77.15, 92.36, 103.20, 1, 9, 10),
    (93.37, 77.15, 103.41, 103.20, 1, 9, 10), (104.41, 77.15, 109.43, 103.20, 1, 4, 5), (110.44, 77.15, 118.47, 103.20, -2, 7, 8), (119.47, 77.15, 130.51, 103.20, 1, 10, 9), (131.52, 77.15, 136.54, 103.20, 1, 4, 5), (137.54, 77.15, 152.60, 103.20, 1, 14, 15),
    (153.60, 77.15, 163.64, 103.20, 1, 9, 10), (164.65, 77.15, 174.69, 103.20, 1, 9, 10), (175.69, 77.15, 185.73, 103.20, 1, 9, 10), (186.73, 77.15, 196.77, 103.20, 1, 9, 10), (197.78, 77.15, 205.81, 103.20, 1, 7, 7), (206.81, 77.15, 216.85, 103.20, 0, 9, 9),
    (217.85, 77.15, 225.89, 103.20, 0, 7, 7), (226.89, 77.15, 236.93, 103.20, 1, 9, 10), (-1.00, 103.20, 9.04, 129.25, 0, 9, 9), (10.04, 103.20, 26.11, 129.25, 0, 15, 14), (27.11, 103.20, 38.15, 129.25, 0, 10, 10), (39.16, 103.20, 50.20, 129.25, 0, 10, 10),
    (51.20, 103.20, 61.24, 129.25, 0, 9, 9), (62.25, 103.20, 70.28, 129.25, 0, 7, 7), (71.28, 103.20, 75.30, 129.25, 4, 3, 7), (76.30, 103.20, 84.33, 129.25, 0, 7, 7), (85.34, 103.20, 97.38, 129.25, 0, 11, 11), (98.39, 103.20, 116.46, 129.25, 2, 17, 19),
    (117.46, 103.20, 135.53, 129.25, 2, 17, 19), (136.54, 103.20, 154.61, 129.25, 2, 17, 19), (155.61, 103.20, 173.68, 129.25, 2, 17, 19), (174.69, 103.20, 192.76, 129.25, 2, 17, 19), (193.76, 103.20, 211.83, 129.25, 2, 17, 19), (212.84, 103.20, 230.91, 129.25, 2, 17, 19),
    (-1.00, 129.25, 17.07, 155.31, 2, 17, 19), (18.07, 129.25, 36.15, 155.31, 2, 17, 19), (37.15, 129.25, 55.22, 155.31, 2, 17, 19), (56.22, 129.25, 74.29, 155.31, 2, 17, 19), (75.30, 129.25, 93.37, 155.31, 2, 17, 19), (94.37, 129.25, 112.44, 155.31, 2, 17, 19),
    (113.45, 129.25, 131.52, 155.31, 2, 17, 19), (132.52, 129.25, 150.59, 155.31, 2, 17, 19), (151.60, 129.25, 169.67, 155.31, 2, 17, 19), (170.67, 129.25, 188.74, 155.31, 2, 17, 19), (189.75, 129.25, 207.82, 155.31, 2, 17, 19), (208.82, 129.25, 226.89, 155.31, 2, 17, 19),
    (-1.00, 155.31, 17.07, 181.36, 2, 17, 19), (18.07, 155.31, 36.15, 181.36, 2, 17, 19), (37.15, 155.31, 55.22, 181.36, 2, 17, 19), (56.22, 155.31, 74.29, 181.36, 2, 17, 19), (75.30, 155.31, 93.37, 181.36, 2, 17, 19), (94.37, 155.31, 112.44, 181.36, 2, 17, 19),
    (113.45, 155.31, 131.52, 181.36, 2, 17, 19), (132.52, 155.31, 150.59, 181.36, 2, 17, 19), (151.60, 155.31, 169.67, 181.36, 2, 17, 19), (170.67, 155.31, 188.74, 181.36, 2, 17, 19), (189.75, 155.31, 207.82, 181.36, 2, 17, 19), (208.82, 155.31, 226.89, 181.36, 2, 17, 19),
    (-1.00, 181.36, 17.07, 207.41, 2, 17, 19), (18.07, 181.36, 36.15, 207.41, 2, 17, 19), (37.15, 181.36, 39.16, 207.41, 0, 1, 5), (40.16, 181.36, 45.18, 207.41, 1, 4, 5), (46.18, 181.36, 56.22, 207.41, 1, 9, 10), (57.23, 181.36, 68.27, 207.41, 1, 10, 10),
    (69.27, 181.36, 81.32, 207.41, 0, 11, 11), (82.33, 181.36, 94.37, 207.41, 0, 11, 12), (95.38, 181.36, 99.39, 207.41, 4, 3, 7), (100.40, 181.36, 112.44, 207.41, 0, 11, 11), (113.45, 181.36, 121.48, 207.41, 3, 7, 10), (122.48, 181.36, 136.54, 207.41, 0, 13, 13),
    (137.54, 181.36, 144.57, 207.41, 1, 6, 7), (145.57, 181.36, 157.62, 207.41, 0, 11, 11), (158.62, 181.36, 168.66, 207.41, 1, 9, 10), (169.67, 181.36, 176.69, 207.41, 0, 6, 7), (177.70, 181.36, 191.75, 207.41, 0, 13, 13), (192.76, 181.36, 204.80, 207.41, 0, 11, 11),
    (205.81, 181.36, 213.84, 207.41, 0, 7, 7), (214.84, 181.36, 224.88, 207.41, 1, 9, 10), (225.89, 181.36, 233.92, 207.41, 1, 7, 8), (234.92, 181.36, 243.96, 207.41, 0, 8, 9), (-1.00, 207.41, 5.02, 233.46, 4, 5, 9), (6.03, 207.41, 18.07, 233.46, 0, 11, 11),
    (19.08, 207.41, 32.13, 233.46, 0, 12, 12), (33.13, 207.41, 38.15, 233.46, 1, 4, 5), (39.16, 207.41, 45.18, 233.46, 4, 5, 9), (46.18, 207.41, 52.21, 233.46, 1, 5, 8), (53.21, 207.41, 61.24, 233.46, 1, 7, 7), (62.25, 207.41, 74.29, 233.46, 0, 11, 11),
    (75.30, 207.41, 92.36, 233.46, 0, 16, 17), (93.37, 207.41, 111.44, 233.46, 0, 17, 17), (112.44, 207.41, 130.51, 233.46, -1, 17, 18), (131.52, 207.41, 141.56, 233.46, 1, 9, 9), (142.56, 207.41, 155.61, 233.46, 0, 12, 12), (156.62, 207.41, 169.67, 233.46, 0, 12, 12),
    (170.67, 207.41, 183.72, 233.46, 0, 12, 12), (184.73, 207.41, 197.78, 233.46, 0, 12, 12), (198.78, 207.41, 211.83, 233.46, 0, 12, 12), (212.84, 207.41, 225.89, 233.46, 0, 12, 12), (-1.00, 233.46, 17.07, 259.51, -1, 17, 17), (18.07, 233.46, 30.12, 259.51, 1, 11, 11),
    (31.13, 233.46, 41.16, 259.51, 1, 9, 10), (42.17, 233.46, 52.21, 259.51, 1, 9, 10), (53.21, 233.46, 63.25, 259.51, 1, 9, 10), (64.25, 233.46, 74.29, 259.51, 1, 9, 10), (75.30, 233.46, 81.32, 259.51, 0, 5, 6), (82.33, 233.46, 88.35, 259.51, 1, 5, 5),
    (89.35, 233.46, 97.38, 259.51, 0, 7, 6), (98.39, 233.46, 106.42, 259.51, 0, 7, 6), (107.42, 233.46, 120.47, 259.51, 0, 12, 13), (121.48, 233.46, 132.52, 259.51, 1, 10, 12), (133.53, 233.46, 145.57, 259.51, 1, 11, 12), (146.58, 233.46, 158.62, 259.51, 1, 11, 12),
    (159.63, 233.46, 171.67, 259.51, 1, 11, 12), (172.68, 233.46, 184.73, 259.51, 1, 11, 12), (185.73, 233.46, 197.78, 259.51, 1, 11, 12), (198.78, 233.46, 208.82, 259.51, 1, 9, 10), (209.82, 233.46, 223.88, 259.51, 0, 13, 13), (224.88, 233.46, 235.93, 259.51, 1, 10, 12),
    (-1.00, 259.51, 10.04, 285.56, 1, 10, 12), (11.05, 259.51, 22.09, 285.56, 1, 10, 12), (23.09, 259.51, 34.14, 285.56, 1, 10, 12), (35.14, 259.51, 47.19, 285.56, 0, 11, 10), (48.19, 259.51, 59.24, 285.56, 1, 10, 10), (60.24, 259.51, 71.28, 285.56, 1, 10, 11),
    (72.29, 259.51, 82.33, 285.56, 0, 9, 10), (83.33, 259.51, 93.37, 285.56, 0, 9, 10), (94.37, 259.51, 104.41, 285.56, 0, 9, 10), (105.42, 259.51, 115.45, 285.56, 0, 9, 10), (116.46, 259.51, 126.50, 285.56, 0, 9, 10), (127.50, 259.51, 137.54, 285.56, 0, 9, 10),
    (138.55, 259.51, 153.60, 285.56, 0, 14, 15), (154.61, 259.51, 163.64, 285.56, 1, 8, 9), (164.65, 259.51, 174.69, 285.56, 1, 9, 9), (175.69, 259.51, 185.73, 285.56, 1, 9, 9), (186.73, 259.51, 196.77, 285.56, 1, 9, 9), (197.78, 259.51, 207.82, 285.56, 1, 9, 9),
    (208.82, 259.51, 214.84, 285.56, 0, 5, 6), (215.85, 259.51, 221.87, 285.56, 1, 5, 5), (222.87, 259.51, 230.91, 285.56, -1, 7, 7), (231.91, 259.51, 239.94, 285.56, -1, 7, 7), (-1.00, 285.56, 9.04, 311.61, 1, 9, 10), (10.04, 285.56, 20.08, 311.61, 1, 9, 10),
    (21.09, 285.56, 31.13, 311.61, 1, 9, 10), (32.13, 285.56, 42.17, 311.61, 1, 9, 10), (43.17, 285.56, 53.21, 311.61, 1, 9, 10), (54.22, 285.56, 64.25, 311.61, 1, 9, 10), (65.26, 285.56, 75.30, 311.61, 1, 9, 10), (76.30, 285.56, 86.34, 311.61, 1, 9, 10),
    (87.35, 285.56, 99.39, 311.61, 0, 11, 11), (100.40, 285.56, 110.44, 311.61, 1, 9, 10), (111.44, 285.56, 121.48, 311.61, 1, 9, 10), (122.48, 285.56, 132.52, 311.61, 1, 9, 10), (133.53, 285.56, 143.56, 311.61, 1, 9, 10), (144.57, 285.56, 155.61, 311.61, 0, 10, 10),
    (156.62, 285.56, 166.65, 311.61, 1, 9, 10), (167.66, 285.56, 178.70, 311.61, 0, 10, 10),
];

/// The heading's 24-point record's glyphs (see HEADING24): on h_e0afcd52; characters 46-223. Read
/// from default.xbe 0x3c7b90.
const HEADING24_GLYPHS: [(f32, f32, f32, f32, i8, u8, u8); 178] = [
    (-1.00, -1.00, 4.02, 35.07, 2, 4, 5), (5.02, -1.00, 27.11, 35.07, -2, 21, 19), (28.11, -1.00, 50.20, 35.07, 1, 21, 22), (51.20, -1.00, 56.22, 35.07, 2, 4, 7), (57.23, -1.00, 78.31, 35.07, 1, 20, 20), (79.31, -1.00, 98.39, 35.07, 1, 18, 19),
    (99.39, -1.00, 121.48, 35.07, 1, 21, 22), (122.48, -1.00, 141.56, 35.07, 2, 18, 20), (142.56, -1.00, 163.64, 35.07, 1, 20, 21), (164.65, -1.00, 183.72, 35.07, 1, 18, 19), (184.73, -1.00, 205.81, 35.07, 1, 20, 22), (206.81, -1.00, 227.89, 35.07, 1, 20, 21),
    (228.90, -1.00, 233.92, 35.07, 2, 4, 5), (234.92, -1.00, 240.95, 35.07, 1, 5, 6), (-1.00, 35.07, 14.06, 71.14, 1, 14, 15), (15.06, 35.07, 32.13, 71.14, 1, 16, 17), (33.13, 35.07, 48.19, 71.14, 1, 14, 15), (49.20, 35.07, 69.27, 71.14, 2, 19, 20),
    (70.28, 35.07, 94.37, 71.14, 1, 23, 24), (95.38, 35.07, 119.47, 71.14, 0, 23, 23), (120.47, 35.07, 139.55, 71.14, 2, 18, 20), (140.55, 35.07, 158.62, 71.14, 1, 17, 18), (159.63, 35.07, 179.71, 71.14, 2, 19, 21), (180.71, 35.07, 198.78, 71.14, 1, 17, 19),
    (199.78, 35.07, 217.85, 71.14, 2, 17, 18), (-1.00, 71.14, 19.08, 107.21, 1, 19, 21), (20.08, 71.14, 39.16, 107.21, 2, 18, 20), (40.16, 71.14, 45.18, 107.21, 2, 4, 7), (46.18, 71.14, 62.25, 107.21, 0, 15, 17), (63.25, 71.14, 83.33, 107.21, 2, 19, 19),
    (84.33, 71.14, 100.40, 107.21, 2, 15, 15), (101.40, 71.14, 126.50, 107.21, 2, 24, 26), (127.50, 71.14, 147.58, 107.21, 2, 19, 21), (148.58, 71.14, 170.67, 107.21, 1, 21, 22), (171.67, 71.14, 191.75, 107.21, 2, 19, 20), (192.76, 71.14, 215.85, 107.21, 1, 22, 23),
    (-1.00, 107.21, 19.08, 143.28, 2, 19, 21), (20.08, 107.21, 41.16, 143.28, 1, 20, 21), (42.17, 107.21, 62.25, 143.28, 0, 19, 19), (63.25, 107.21, 83.33, 143.28, 2, 19, 21), (84.33, 107.21, 108.43, 143.28, 0, 23, 23), (109.43, 107.21, 144.57, 143.28, 0, 34, 34),
    (145.57, 107.21, 167.66, 143.28, 0, 21, 20), (168.66, 107.21, 190.75, 143.28, 0, 21, 21), (191.75, 107.21, 211.83, 143.28, 1, 19, 20), (212.84, 107.21, 221.87, 143.28, 2, 8, 9), (-1.00, 143.28, 21.09, 179.35, -2, 21, 19), (22.09, 143.28, 31.13, 179.35, 1, 8, 10),
    (32.13, 143.28, 49.20, 179.35, 0, 16, 16), (50.20, 143.28, 67.27, 179.35, 0, 16, 16), (68.27, 143.28, 76.30, 179.35, 3, 7, 13), (77.31, 143.28, 94.37, 179.35, 1, 16, 18), (95.38, 143.28, 112.44, 179.35, 2, 16, 17), (113.45, 143.28, 128.51, 179.35, 1, 14, 15),
    (129.51, 143.28, 145.57, 179.35, 2, 15, 17), (146.58, 143.28, 161.64, 179.35, 1, 14, 15), (162.64, 143.28, 174.69, 179.35, 1, 11, 11), (175.69, 143.28, 191.75, 179.35, 2, 15, 17), (192.76, 143.28, 208.82, 179.35, 2, 15, 17), (209.82, 143.28, 214.84, 179.35, 2, 4, 7),
    (215.85, 143.28, 224.88, 179.35, -1, 8, 11), (-1.00, 179.35, 15.06, 215.42, 2, 15, 15), (16.07, 179.35, 21.09, 215.42, 2, 4, 7), (22.09, 179.35, 45.18, 215.42, 2, 22, 24), (46.18, 179.35, 62.25, 215.42, 2, 15, 17), (63.25, 179.35, 81.32, 215.42, 1, 17, 18),
    (82.33, 179.35, 98.39, 215.42, 2, 15, 17), (99.39, 179.35, 116.46, 215.42, 1, 16, 18), (117.46, 179.35, 131.52, 215.42, 2, 13, 13), (132.52, 179.35, 149.59, 215.42, 1, 16, 17), (150.59, 179.35, 161.64, 215.42, 1, 10, 11), (162.64, 179.35, 178.70, 215.42, 2, 15, 17),
    (179.71, 179.35, 198.78, 215.42, 0, 18, 18), (199.78, 179.35, 222.87, 215.42, 2, 22, 24), (-1.00, 215.42, 17.07, 251.49, 0, 17, 17), (18.07, 215.42, 35.14, 251.49, 1, 16, 17), (36.15, 215.42, 52.21, 251.49, 1, 15, 16), (53.21, 215.42, 64.25, 251.49, 1, 10, 11),
    (65.26, 215.42, 70.28, 251.49, 6, 4, 10), (71.28, 215.42, 82.33, 251.49, 1, 10, 11), (83.33, 215.42, 100.40, 251.49, 3, 16, 18), (101.40, 215.42, 108.43, 251.49, 1, 6, 7), (109.43, 215.42, 132.52, 251.49, 1, 22, 23), (133.53, 215.42, 140.55, 251.49, 1, 6, 7),
    (141.56, 215.42, 148.58, 251.49, 1, 6, 7), (149.59, 215.42, 156.62, 251.49, 1, 6, 7), (157.62, 215.42, 164.65, 251.49, 1, 6, 7), (165.65, 215.42, 172.68, 251.49, 1, 6, 7), (173.68, 215.42, 180.71, 251.49, 1, 6, 7), (181.71, 215.42, 188.74, 251.49, 1, 6, 7),
    (189.75, 215.42, 196.77, 251.49, 1, 6, 7), (197.78, 215.42, 204.80, 251.49, 1, 6, 7), (205.81, 215.42, 212.84, 251.49, 1, 6, 7), (213.84, 215.42, 220.87, 251.49, 1, 6, 7), (221.87, 215.42, 228.90, 251.49, 1, 6, 7), (229.90, 215.42, 236.93, 251.49, 1, 6, 7),
    (237.93, 215.42, 244.96, 251.49, 1, 6, 7), (-1.00, 251.49, 6.03, 287.56, 1, 6, 7), (7.03, 251.49, 14.06, 287.56, 1, 6, 7), (15.06, 251.49, 22.09, 287.56, 1, 6, 7), (23.09, 251.49, 30.12, 287.56, 1, 6, 7), (31.13, 251.49, 38.15, 287.56, 1, 6, 7),
    (39.16, 251.49, 46.18, 287.56, 1, 6, 7), (47.19, 251.49, 54.22, 287.56, 1, 6, 7), (55.22, 251.49, 62.25, 287.56, 1, 6, 7), (63.25, 251.49, 70.28, 287.56, 1, 6, 7), (71.28, 251.49, 78.31, 287.56, 1, 6, 7), (79.31, 251.49, 86.34, 287.56, 1, 6, 7),
    (87.35, 251.49, 94.37, 287.56, 1, 6, 7), (95.38, 251.49, 102.40, 287.56, 1, 6, 7), (103.41, 251.49, 110.44, 287.56, 1, 6, 7), (111.44, 251.49, 118.47, 287.56, 1, 6, 7), (119.47, 251.49, 126.50, 287.56, 1, 6, 7), (127.50, 251.49, 134.53, 287.56, 1, 6, 7),
    (135.53, 251.49, 137.54, 287.56, 0, 1, 8), (138.55, 251.49, 143.56, 287.56, 2, 4, 7), (144.57, 251.49, 158.62, 287.56, 2, 13, 14), (159.63, 251.49, 178.70, 287.56, 2, 18, 19), (179.71, 251.49, 200.79, 287.56, 0, 20, 21), (201.79, 251.49, 222.87, 287.56, 0, 20, 20),
    (-1.00, 287.56, 4.02, 323.63, 6, 4, 10), (5.02, 287.56, 24.10, 323.63, 1, 18, 19), (25.10, 287.56, 36.15, 323.63, 3, 10, 13), (37.15, 287.56, 60.24, 323.63, 1, 22, 24), (61.24, 287.56, 72.29, 323.63, 1, 10, 12), (73.29, 287.56, 84.33, 323.63, 1, 10, 12),
    (85.34, 287.56, 100.40, 323.63, 1, 14, 15), (101.40, 287.56, 110.44, 323.63, 0, 8, 7), (111.44, 287.56, 134.53, 323.63, 1, 22, 24), (135.53, 287.56, 152.60, 323.63, 0, 16, 16), (153.60, 287.56, 164.65, 323.63, 1, 10, 11), (165.65, 287.56, 182.72, 323.63, 1, 16, 17),
    (183.72, 287.56, 195.77, 323.63, 2, 11, 13), (196.77, 287.56, 208.82, 323.63, 2, 11, 13), (209.82, 287.56, 217.85, 323.63, 6, 7, 10), (-1.00, 323.63, 15.06, 359.70, 2, 15, 17), (16.07, 323.63, 36.15, 359.70, 1, 19, 20), (37.15, 323.63, 42.17, 359.70, 2, 4, 5),
    (43.17, 323.63, 49.20, 359.70, 6, 5, 10), (50.20, 323.63, 54.22, 359.70, 6, 3, 9), (55.22, 323.63, 67.27, 359.70, 1, 11, 11), (68.27, 323.63, 79.31, 359.70, 2, 10, 11), (80.32, 323.63, 107.42, 359.70, 7, 26, 28), (108.43, 323.63, 136.54, 359.70, 7, 27, 28),
    (137.54, 323.63, 169.67, 359.70, 2, 31, 33), (170.67, 323.63, 190.75, 359.70, 1, 19, 21), (191.75, 323.63, 215.85, 359.70, 0, 23, 23), (-1.00, 359.70, 23.09, 395.77, 0, 23, 23), (24.10, 359.70, 48.19, 395.77, 0, 23, 23), (49.20, 359.70, 73.29, 395.77, 0, 23, 23),
    (74.29, 359.70, 98.39, 395.77, 0, 23, 23), (99.39, 359.70, 123.49, 395.77, 0, 23, 23), (124.49, 359.70, 157.62, 395.77, 0, 32, 33), (158.62, 359.70, 176.69, 395.77, 1, 17, 18), (177.70, 359.70, 195.77, 395.77, 1, 17, 19), (196.77, 359.70, 214.84, 395.77, 1, 17, 19),
    (-1.00, 395.77, 17.07, 431.85, 1, 17, 19), (18.07, 395.77, 36.15, 431.85, 1, 17, 19), (37.15, 395.77, 45.18, 431.85, -1, 7, 10), (46.18, 395.77, 54.22, 431.85, 2, 7, 7), (55.22, 395.77, 64.25, 431.85, 0, 8, 9), (65.26, 395.77, 73.29, 431.85, 1, 7, 8),
    (74.29, 395.77, 96.38, 431.85, 0, 21, 23), (97.38, 395.77, 117.46, 431.85, 2, 19, 21), (118.47, 395.77, 140.55, 431.85, 1, 21, 22), (141.56, 395.77, 163.64, 431.85, 1, 21, 22), (164.65, 395.77, 186.73, 431.85, 1, 21, 22), (187.74, 395.77, 209.82, 431.85, 1, 21, 22),
    (-1.00, 431.85, 21.09, 467.92, 1, 21, 22), (22.09, 431.85, 39.16, 467.92, 1, 16, 17), (40.16, 431.85, 62.25, 467.92, 1, 21, 22), (63.25, 431.85, 83.33, 467.92, 2, 19, 21), (84.33, 431.85, 104.41, 467.92, 2, 19, 21), (105.42, 431.85, 125.49, 467.92, 2, 19, 21),
    (126.50, 431.85, 146.58, 467.92, 2, 19, 21), (147.58, 431.85, 169.67, 467.92, 0, 21, 21), (170.67, 431.85, 190.75, 467.92, 2, 19, 20), (191.75, 431.85, 208.82, 467.92, 2, 16, 17),
];

#[cfg(test)]
mod tests {
    use super::*;

    /// The records' widths: take01's "Saving..." caption, its ink from x 72.7 to 126.7 (54 units:
    /// 35.85-36.82 s) in the 12-point record, whose pen goes one unit past the last dot's ink.
    #[test]
    fn saving_is_as_wide_as_the_take() {
        let w = BODY12.measure("Saving...");
        assert!((w - 55.0).abs() <= 1.0, "{w}");
        // the 14-point record: take01's "CAMPAIGN" (81 units of ink) and "New Campaign" (116)
        assert!((BODY14.measure("CAMPAIGN") - 83.0).abs() <= 1.0);
        assert!((BODY14.measure("New Campaign") - 116.0).abs() <= 1.0);
        // a space out of the heading's range moves as "1"
        assert_eq!(HEADING24.measure("1 1"), 3.0 * HEADING24.measure("1"));
    }
}
