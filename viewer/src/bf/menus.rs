//! The game's menus (common/game-options-<lang>.xmb, its `<game-menus>`), read as data: each
//! menu's place and size and its items (texts, pictures, button legends), the menu styles they
//! name (colour, font and size, alignment) and the picture sets (#224). bf_play draws the
//! tutorial's panels from it (src/bin/play_panels.rs, viewer/docs/hints.md); the objectives menu
//! (play_objectives.rs) and the front end (play_menu.rs) still lay theirs out by hand.
//!
//!   <game-menus>
//!     <menu-style iD="h_09ced527">                            a style (its parser near 0x19c186)
//!       <alignment h_fc9aaf63="0" h_0c9d2ce1="0"/>            across, down: 0 left / top, 2 centred
//!       <color-normal>240 240 240 255</color-normal>          the text's colour
//!       <h_ed70ff4f>0 0 0 255</h_ed70ff4f>                     its shadow's (an untextured picture's fill)
//!       <h_0ee47ad4 h_f890a8bc="h_0c626fd5" point-size="16"/>  its font and size (h_15ed2b6e a second)
//!     <h_1544c137 iD="h_0d0ce2e2">                            a picture set: pieces drawn together
//!       <h_1b74bf95 texture=.. h_06712921=.. alignment=..><color/><position/><size/>
//!     <menus>
//!       <data name="h_f0ea44dc" controller=..><position>85 60</position><size>470 360</size></data>
//!       <h_e275fb80>                                          its items, in order (the parser near 0x19bf17)
//!         <static-text text="h_166f0785"><item-data name="Title" style-id=..><position/><size/>
//!         <h_0ac3cc5c [h_0d802f88="h_0d0ce2e2"]><item-data ..><data texture=.. ../>   a picture
//!         <h_09dc0d2f button-string="h_1d3f3cb3" h_eeb60269="h_18291278"><item-data ..>  a legend
//!         <h_16a61c91 ..><item-data name="h_1966467f">       the panels' A target (0x100ea0 closes the menu on it)

use std::collections::HashMap;

use bevy::math::Vec2;

use super::bxml::Element;
use super::hash::h;

/// A menu's items, h_e275fb80 (the `<menus>` element's second child).
const H_ITEMS: u32 = 0xE275_FB80;
/// A picture item, h_0ac3cc5c (one of the menu parser's item types, 0x19bf47).
const H_PICTURE: u32 = 0x0AC3_CC5C;
/// A picture item's set, h_0d802f88: a `<h_1544c137>` picture set by its iD (the tutorial's
/// panels' frame, h_0d0ce2e2).
const H_PICTURE_SET_REF: u32 = 0x0D80_2F88;
/// A button legend item, h_09dc0d2f (0x19bf40): `button-string` (the icon font's letter, a string
/// id) and h_eeb60269 its label (a string id).
const H_LEGEND: u32 = 0x09DC_0D2F;
/// A legend's label, h_eeb60269 (0x19bdb4).
const H_LEGEND_LABEL: u32 = 0xEEB6_0269;
/// A picture set, h_1544c137, and its pieces, h_1b74bf95.
const H_PICTURE_SET: u32 = 0x1544_C137;
const H_PIECE: u32 = 0x1B74_BF95;
/// A piece drawn upside down, h_06712921 (the frame's bottom bar).
const H_FLIPPED: u32 = 0x0671_2921;
/// A piece's alignment h_ef37425a: drawn at its texture's own size (the frame's bars, size 0 0;
/// the panels' controller picture, 256 x 256, and the WEAPONS panel's gun, 128 x 64: take07 63 s).
/// The other one the data uses, h_fd2e2796, stretches it to its size (the frame's body, 470 x 296).
const H_OWN_SIZE: u32 = 0xEF37_425A;
/// A style's shadow colour, h_ed70ff4f (a static text's shadow 1 unit down and right, FUN_0010a420;
/// the panels' full-screen picture with no texture is this colour: black at 180, take04's
/// background at 0.28 of its value under the panels).
const H_STYLE_EDGE: u32 = 0xED70_FF4F;
/// A style's font, h_0ee47ad4, and its second, h_15ed2b6e (a legend's word): h_f890a8bc the
/// font's name, `point-size` its size.
const H_STYLE_FONT: u32 = 0x0EE4_7AD4;
const H_STYLE_FONT2: u32 = 0x15ED_2B6E;
const H_FONT_NAME: u32 = 0xF890_A8BC;
/// A style's alignment across, h_fc9aaf63, and down, h_0c9d2ce1: 0 left / top, 2 centred (the
/// objectives menu's title h_e53b67a8, 2 2, is centred both ways: take04); 1 is taken as right /
/// bottom, **a guess** (no panel's style has it).
const H_ALIGN_ACROSS: u32 = 0xFC9A_AF63;
const H_ALIGN_DOWN: u32 = 0x0C9D_2CE1;

/// A piece of a picture: a texture (0 none: an untextured quad) in a colour (RGBA), at a place
/// and size relative to its item, at the texture's own size or stretched to its size, maybe
/// upside down.
#[derive(Clone, Debug, Default)]
pub struct PicturePiece {
    pub texture: u32,
    pub color: [u8; 4],
    pub at: Vec2,
    pub size: Vec2,
    /// h_06712921
    pub flipped: bool,
    /// drawn at `size` (alignment h_fd2e2796), else at the texture's own size (H_OWN_SIZE)
    pub stretched: bool,
}

/// What a menu item is.
#[derive(Clone, Debug)]
pub enum ItemKind {
    /// `<static-text text>`: its string id
    Text(u32),
    /// `<h_0ac3cc5c>`: a picture set by its iD (h_0d802f88), or its own piece (its `<data>`)
    Picture { set: u32, piece: Option<PicturePiece> },
    /// `<h_09dc0d2f>`: a button legend, its `button-string` and h_eeb60269 label (string ids)
    Legend { button: u32, label: u32 },
    /// any other item type (the element's name): buttons, lists, the panels' h_16a61c91 accept
    /// target
    Other(u32),
}

/// A menu item: what it is, its `<item-data>` name and style, and its place and size relative to
/// the menu.
#[derive(Clone, Debug)]
pub struct MenuItem {
    pub kind: ItemKind,
    pub name: u32,
    /// `style-id` (0 none)
    pub style: u32,
    pub at: Vec2,
    pub size: Vec2,
}

/// A menu (`<menus>`): its name (`<data name>`), place and size on the 640 x 480 screen, and its
/// items in the data's order.
#[derive(Clone, Debug, Default)]
pub struct GameMenu {
    pub name: u32,
    pub at: Vec2,
    pub size: Vec2,
    pub items: Vec<MenuItem>,
}

/// A menu style (`<menu-style iD>`): what's given of it.
#[derive(Clone, Debug, Default)]
pub struct MenuStyle {
    /// `color-normal` (RGBA)
    pub color: Option<[u8; 4]>,
    /// h_ed70ff4f (see H_STYLE_EDGE)
    pub edge: Option<[u8; 4]>,
    /// h_0ee47ad4: the font's name and point size
    pub font: Option<(u32, f32)>,
    /// h_15ed2b6e: the second font (a legend's word)
    pub font2: Option<(u32, f32)>,
    /// `<alignment>` across and down (see H_ALIGN_ACROSS); 0 0 when not given
    pub align: (i64, i64),
}

/// The game's menus, styles and picture sets, by name hash; empty if the file isn't there.
#[derive(Clone, Debug, Default)]
pub struct GameMenus {
    pub menus: HashMap<u32, GameMenu>,
    pub styles: HashMap<u32, MenuStyle>,
    pub pictures: HashMap<u32, Vec<PicturePiece>>,
}

/// An element's text as numbers (`<position>85 60</position>`).
fn numbers(e: Option<&Element>) -> Vec<f32> {
    e.and_then(|e| e.text.as_ref()).map(|t| t.floats()).unwrap_or_default()
}

/// A child's two numbers (a position or a size), 0 0 without them.
fn pair(e: &Element, child: u32) -> Vec2 {
    let n = numbers(e.child(child));
    Vec2::new(n.first().copied().unwrap_or(0.0), n.get(1).copied().unwrap_or(0.0))
}

/// A child's four numbers as an RGBA colour, if it has them.
fn color(e: &Element, child: u32) -> Option<[u8; 4]> {
    let n = numbers(e.child(child));
    (n.len() >= 4).then(|| [n[0] as u8, n[1] as u8, n[2] as u8, n[3] as u8])
}

/// A name attribute's hash, 0 when it's empty or not given.
fn name_of(e: &Element, k: u32) -> u32 {
    e.attr(k).and_then(|v| v.as_hash()).filter(|&n| n != h("")).unwrap_or(0)
}

/// A true / false attribute.
fn flag(e: &Element, k: u32) -> bool {
    e.attr(k).and_then(|v| v.as_i64()).is_some_and(|v| v != 0)
}

/// A picture piece (`<h_1b74bf95>` of a set, or a picture item's `<data>`).
fn piece(e: &Element) -> PicturePiece {
    PicturePiece {
        texture: name_of(e, h("texture")),
        color: color(e, h("color")).unwrap_or([255; 4]),
        at: pair(e, h("position")),
        size: pair(e, h("size")),
        flipped: flag(e, H_FLIPPED),
        stretched: name_of(e, h("alignment")) != H_OWN_SIZE,
    }
}

/// A style's font element: (its name, its point size).
fn font(e: &Element, k: u32) -> Option<(u32, f32)> {
    let f = e.child(k)?;
    Some((name_of(f, H_FONT_NAME), f.attr(h("point-size")).and_then(|v| v.as_f32())?))
}

impl GameMenus {
    /// The menus of a game-options file's root (its `<game-menus>`).
    pub fn from_root(root: &Element) -> GameMenus {
        let mut out = GameMenus::default();
        let Some(gm) = root.walk().into_iter().find(|e| e.name == h("game-menus")) else { return out };
        for e in &gm.children {
            if e.name == h("menu-style") {
                let align = e.child(h("alignment")).map_or((0, 0), |a| {
                    let n = |k: u32| a.attr(k).and_then(|v| v.as_i64()).unwrap_or(0);
                    (n(H_ALIGN_ACROSS), n(H_ALIGN_DOWN))
                });
                let style = MenuStyle { color: color(e, h("color-normal")), edge: color(e, H_STYLE_EDGE), font: font(e, H_STYLE_FONT),
                                        font2: font(e, H_STYLE_FONT2), align };
                // (the first of two styles of one iD: the data repeats some, alike)
                out.styles.entry(name_of(e, h("iD"))).or_insert(style);
            } else if e.name == H_PICTURE_SET {
                out.pictures.entry(name_of(e, h("iD"))).or_insert_with(|| e.children_named(H_PIECE).map(piece).collect());
            } else if e.name == h("menus") {
                let Some(data) = e.child(h("data")) else { continue };
                let items = e.child(H_ITEMS).map_or(vec![], |list| list.children.iter().map(|c| {
                    let item = c.child(h("item-data"));
                    let kind = if c.name == h("static-text") {
                        ItemKind::Text(name_of(c, h("text")))
                    } else if c.name == H_PICTURE {
                        ItemKind::Picture { set: name_of(c, H_PICTURE_SET_REF), piece: c.child(h("data")).map(piece) }
                    } else if c.name == H_LEGEND {
                        ItemKind::Legend { button: name_of(c, h("button-string")), label: name_of(c, H_LEGEND_LABEL) }
                    } else {
                        ItemKind::Other(c.name)
                    };
                    MenuItem { kind, name: item.map_or(0, |i| name_of(i, h("name"))), style: item.map_or(0, |i| name_of(i, h("style-id"))),
                               at: item.map_or(Vec2::ZERO, |i| pair(i, h("position"))), size: item.map_or(Vec2::ZERO, |i| pair(i, h("size"))) }
                }).collect());
                let name = name_of(data, h("name"));
                out.menus.entry(name).or_insert(GameMenu { name, at: pair(data, h("position")), size: pair(data, h("size")), items });
            }
        }
        out
    }
}
