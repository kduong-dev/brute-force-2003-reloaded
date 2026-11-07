//! The test tools' gallery of the game's unused pictures and faces (#397): the textures that
//! nothing in the game's data or code refers to, shown at their own size so they can be seen.
//! Demo tooling, not the game: the key, the pages and the layout are the demo's choices.
//!
//!  - **What's in it**: the 40 textures the cut-content catalogue found unreferenced
//!    (todo/cut-content/catalogue.md, sections 1 and 6; textures-unreferenced.txt): not named by a
//!    material, an object type, a menu, a script, an effect or the XBE's strings and constants
//!    (`PICTURES`). Names are the ones recovered by hashing (the select pictures' and faces'
//!    `<name>.tga`, the XBE's three code-named ones); the rest read "(unnamed)". Each has the one
//!    line the catalogue says about it. They are **unused in the game**: nothing in it draws them,
//!    which the panel says.
//!  - **Where they come from**: the game's own texture libraries, read from the archives as the
//!    game's textures are (`texture::index`, `texture::decode`: DXT1, DXT3 and the Xbox's
//!    uncompressed formats), on a thread the first time the gallery opens: splash_screen (the front
//!    end's: the select pictures and menu pieces), common (the HUD faces and code-named textures)
//!    and the tutorial's. Not the catalogue's PNGs (`decompiled/cut/`).
//!  - **The pages** (`PAGES`): grouped as the ticket asks, cut characters, cut squad, faces, map,
//!    other (three pages), seven in all. Up / Down (or the wheel) step to the previous / next page,
//!    Page Up / Down to the first page of the previous / next group. A page lays its pictures out
//!    at their own size, one texel to a pixel, in cells of a fixed height (`CAPTION_HEIGHT`), each
//!    with its name, hash, library, size and format and the catalogue's line; a picture of 64
//!    pixels or less a side also shows magnified (`zoom_of`, nearest texel) beside it, since a
//!    32 x 32 face is small, and a thin strip (the 4 x 256 gradient, a four pixel line at its own
//!    size) stretched across its thin side. The panel is fully opaque, so the HUD behind it doesn't
//!    ghost through. The panel on the right (the tools' own) has the page's group and the keys. The
//!    textures are stored upside down, as the HUD's and the menus' are, so the nodes flip them.
//!  - **The key**: `[` (`KeyCode::BracketLeft`), in the tool registry (play_editor.rs `TOOLS`, so in
//!    edit mode too, and listed in its help). Every letter is another control's in the demo (the
//!    game's, the test map's, the hint and Squad Command panels'); `[` is the key right of P, the
//!    scenery menu's, and nothing in bf_play reads it. Esc closes it.
//!
//! Test hooks (play_testtools.rs `read_hooks`, on the controlled character's clock):
//! `BF_TEST_GALLERY=<s>[,<page>]` opens the gallery (at that page: its number from 1, or a group's
//! name, `faces`; entries separated by `;` page through it), `BF_TEST_KEYS` presses its keys
//! (`bracketleft`, `down`, `pagedown`). `BF_TOOLS_LOG=1` prints the pictures read (how many, how
//! long, any not found) and each page shown.

use std::sync::{mpsc::Receiver, Mutex};

use super::*;
use super::testtools::{Hook, Panel, Script, Tools};
use bf_viewer::bf::archive::Archive;
use bf_viewer::bf::bxml::{self, SchemaSet};
use bf_viewer::bf::texture;
use super::testmap::TestMap;

/// The gallery's key (see the module notes): the registry's entry for it is play_editor.rs's.
pub(super) const KEY: KeyCode = KeyCode::BracketLeft;
/// How long a gallery message shows (s): the tools'.
const MESSAGE_TIME: f32 = 2.0;
/// The gallery panel's stacking order above the HUD's (the radar's is 3, the menus' 20 to 25): over
/// them all but the mission's console and the loading cover (the demo's choice).
const GALLERY_Z: i32 = 30;
/// Where the picture panel sits: its margin from the window's top left (px) and its width (a share
/// of the window: five cells of CELL_WIDTH in a row at 1280 px; the tools' own panel, narrow while
/// the gallery is open, takes the right).
const PANEL_MARGIN: f32 = 12.0;
const PANEL_WIDTH: f32 = 72.0;
/// The panel's colour: dark and fully opaque, so the HUD behind it doesn't ghost through (at an alpha
/// of 0.92 the health bars and the radar showed through the faces' page; the demo's choice).
const PANEL_COLOUR: Color = Color::srgb(0.02, 0.02, 0.05);
/// The grey behind each picture, so the alpha of the faces' and pieces' edges shows as the
/// catalogue's contact sheets show it (the demo's choice).
const PICTURE_BACKGROUND: Color = Color::srgb(0.30, 0.30, 0.30);
/// A cell of a page is at least this wide (px): a 32 x 32 face with its x4 copy and the gap between
/// them, so every cell of the faces' page is as wide as a select picture's. The gap between cells,
/// and between a picture and its magnified copy; and the height every cell's picture row has at
/// least, and its caption area's (a name, three lines of facts and up to six of the catalogue's
/// line, 11 px text): fixed, so the page's rows don't depend on how the text wraps (the demo's
/// choices).
const CELL_WIDTH: f32 = 168.0;
const CELL_GAP: f32 = 12.0;
const ZOOM_GAP: f32 = 8.0;
const PICTURE_ROW: u32 = 128;
const CAPTION_HEIGHT: f32 = 150.0;
/// A picture whose sides are both at most this many texels is magnified beside itself, by the
/// largest whole factor up to ZOOM_MAX that keeps its longer side within ZOOM_BOX pixels. A thin
/// strip, a picture with a side under STRIP_UNDER texels and the other over ZOOM_UP_TO (the
/// 4 x 256 gradient, which at its own size is a four pixel line), is stretched across its thin side
/// by the whole factor that makes it STRIP_TO pixels (nearest texels: the demo's choices).
const ZOOM_UP_TO: u32 = 64;
const ZOOM_MAX: u32 = 4;
const ZOOM_BOX: u32 = 128;
const STRIP_UNDER: u32 = 8;
const STRIP_TO: u32 = 32;

/// The three texture libraries the pictures are in (data/<library>.tgz).
const SPLASH: &str = "splash_screen";
const COMMON: &str = "common";
const TUTORIAL: &str = "tutorial";

/// A page of the gallery: its group (the ticket's: cut characters, cut squad, faces, map, other)
/// and what the page holds.
struct Page {
    group: &'static str,
    title: &'static str,
}

/// The gallery's pages, in order (`Picture::page` indexes them). The ticket's groups are cut
/// characters, cut squad, faces, map and other; "other" holds 20 pictures of every size, so it takes
/// three pages (no page is more than two rows of cells).
const PAGES: [Page; 7] = [
    Page { group: "cut characters", title: "the cut deathmatch characters' select pictures (128 x 128)" },
    Page { group: "cut squad", title: "the cut Kingman Posse's squad picture (128 x 128)" },
    Page { group: "faces", title: "the cut HUD faces (32 x 32)" },
    Page { group: "map", title: "the cut eighth deathmatch map's picture (128 x 128)" },
    Page { group: "other", title: "title art, placeholders and the older select thumbnails" },
    Page { group: "other", title: "menu and HUD pieces" },
    Page { group: "other", title: "the textures default.xbe names, and a ramp" },
];

/// One of the game's unused pictures: its texture's name hash (the key of its library's index),
/// the library it's read from, the name recovered by hashing ("" where none was), its page and the
/// one line the catalogue says about it.
struct Picture {
    hash: u32,
    library: &'static str,
    name: &'static str,
    page: usize,
    about: &'static str,
}

/// The 40 unreferenced textures of the cut-content catalogue (textures-unreferenced.txt, "UNREFERENCED":
/// 5 in common, 5 faces in common and its copies, 27 in splash_screen, 3 in the tutorial), in the order
/// the pages show them. The hashes are the catalogue's and the ticket's; a name is one recovered by
/// hashing (the catalogue's section 6, the Kingman Posse draft's section 5, the XBE's own strings),
/// and `names_hash_to_their_pictures` checks each against its hash. The lines are the catalogue's
/// (sections 1.1 to 1.4) in a few words; "the cut" is the Kingman Posse and the monsters, whose flags,
/// strings and pictures survive without characters (todo/drafts/done/sdm-classes-kingman-monsters.md).
const PICTURES: [Picture; 40] = [
    // page 0: the cut deathmatch characters' select pictures, splash_screen
    Picture { hash: 0xFB53_F8EB, library: SPLASH, name: "kingmen.tga", page: 0, about: "Kingman in his grey suit, the Kingman Posse's boss (the name is misspelt: less sure)" },
    Picture { hash: 0xE0E3_68D2, library: SPLASH, name: "terrorist.tga", page: 0, about: "a black-armoured trooper: the Terrorist" },
    Picture { hash: 0xE1A1_9B31, library: SPLASH, name: "subversive.tga", page: 0, about: "another black-armoured trooper: the Subversive" },
    Picture { hash: 0xE373_0562, library: SPLASH, name: "mutant1.tga", page: 0, about: "a pink bare-chested mutant" },
    Picture { hash: 0xEF8B_9F92, library: SPLASH, name: "mutant2.tga", page: 0, about: "a big bare-chested mutant in white shorts: El Mutante" },
    Picture { hash: 0xF890_957C, library: SPLASH, name: "supermutant.tga", page: 0, about: "the Supermutant, tan with a green belt" },
    Picture { hash: 0x10B1_E538, library: SPLASH, name: "feral_meancast.tga", page: 0, about: "a hunched blue-scaled feral with a red crest and tail: Feral Meancast" },
    Picture { hash: 0x10B5_0B57, library: SPLASH, name: "thinker.tga", page: 0, about: "a pale four-legged creature with a long curved tail: the Shrike Thinker, which no character type carries" },
    Picture { hash: 0xE155_5C19, library: SPLASH, name: "lesserpriest.tga", page: 0, about: "a bald man in a red-and-gold robe, arms wide: the Lesser Priest" },
    Picture { hash: 0xE848_500E, library: SPLASH, name: "slave.tga", page: 0, about: "the thin bald figure: the Seer Slave" },
    // page 1: the group picture
    Picture { hash: 0x19F9_D1F0, library: SPLASH, name: "", page: 1, about: "the Kingman Posse's squad picture, by its content: a trooper, mutant1, the supermutant and Kingman" },
    // page 2: the faces (the HUD's 32 x 32 portraits): five in common with copies in other libraries, the Confed V-squad marine's, two splash_screen faces
    Picture { hash: 0x1CC0_787C, library: COMMON, name: "face_kingman.tga", page: 2, about: "Kingman's HUD face" },
    Picture { hash: 0x0BE4_44B1, library: COMMON, name: "face_terrorist.tga", page: 2, about: "the Terrorist's HUD face, a helmeted trooper" },
    Picture { hash: 0xF7F1_ACFF, library: COMMON, name: "face_subversive.tga", page: 2, about: "the Subversive's HUD face" },
    Picture { hash: 0xFC5F_519F, library: COMMON, name: "face_mutant.tga", page: 2, about: "the mutants' HUD face" },
    Picture { hash: 0xF05A_2AFC, library: COMMON, name: "face_supermutant.tga", page: 2, about: "the Supermutant's HUD face" },
    Picture { hash: 0xF0FB_436D, library: COMMON, name: "face_vsquad.tga", page: 2, about: "a blue-armoured, blue-faced helmeted trooper: the Confed V-squad marine's HUD face" },
    Picture { hash: 0x0FB2_55C3, library: SPLASH, name: "", page: 2, about: "a man's face, short hair, black collar (Tex-like)" },
    Picture { hash: 0xFA8D_087B, library: SPLASH, name: "", page: 2, about: "a dark-haired man in a grey jacket with a red patch (a Militia or terrorist face)" },
    // page 3: the map
    Picture { hash: 0x12A5_159A, library: SPLASH, name: "mp8.tga", page: 3, about: "an orange volcanic landscape, an antenna tower over a lava pool: a cut eighth deathmatch map's picture" },
    // page 4: title art, placeholders, the older select thumbnails
    Picture { hash: 0x1019_8036, library: SPLASH, name: "", page: 4, about: "the BRUTE FORCE title logo over a blue planet (not the used bflogo.tga)" },
    Picture { hash: 0xF60D_E1CD, library: SPLASH, name: "", page: 4, about: "the words \"temp\" and \"art\" in black on white: a placeholder" },
    Picture { hash: 0xF723_1AEB, library: COMMON, name: "", page: 4, about: "a soft smoke puff" },
    Picture { hash: 0xF155_9904, library: SPLASH, name: "", page: 4, about: "a blue bracket: a menu frame's side" },
    Picture { hash: 0x096B_4DCD, library: SPLASH, name: "follower.tga", page: 4, about: "an older 32 x 32 select thumbnail: the Seer Follower" },
    Picture { hash: 0xE122_A110, library: SPLASH, name: "feral outcast.tga", page: 4, about: "an older 32 x 32 select thumbnail: the Feral Outcast" },
    Picture { hash: 0xFAEA_CCD3, library: SPLASH, name: "outcast shaman.tga", page: 4, about: "an older 32 x 32 select thumbnail: the Outcast Shaman" },
    Picture { hash: 0x1543_FF89, library: SPLASH, name: "feral shaman.tga", page: 4, about: "an older 32 x 32 select thumbnail: the Feral Shaman" },
    Picture { hash: 0x165B_079E, library: SPLASH, name: "feral colonist.tga", page: 4, about: "an older 32 x 32 select thumbnail: the Feral Colonist" },
    Picture { hash: 0xF752_0FB0, library: SPLASH, name: "sub.tga", page: 4, about: "a helmeted black-armoured trooper: the Subversive's thumbnail" },
    // page 5: menu and HUD pieces
    Picture { hash: 0x0D9F_F511, library: SPLASH, name: "", page: 5, about: "a thin blue bar on black: a progress or title bar" },
    Picture { hash: 0xE1C5_3FDD, library: SPLASH, name: "", page: 5, about: "a rounded dark-blue frame corner" },
    Picture { hash: 0xF055_730C, library: SPLASH, name: "", page: 5, about: "a rounded dark-blue frame box" },
    Picture { hash: 0xFB08_FCEB, library: SPLASH, name: "", page: 5, about: "a dark-blue gradient" },
    Picture { hash: 0x158C_7726, library: TUTORIAL, name: "abutton.tga", page: 5, about: "a green orb with an A: the A-button icon" },
    Picture { hash: 0x1EF9_58F5, library: TUTORIAL, name: "", page: 5, about: "a yellow crescent in a ring, the same as the next" },
    Picture { hash: 0xEEAD_4EFF, library: TUTORIAL, name: "", page: 5, about: "a yellow crescent in a ring, the same as the last" },
    // page 6: the textures default.xbe names (the hologram material's) and a ramp
    Picture { hash: 0x16D9_931F, library: COMMON, name: "holoscanbrattonlayer1.tga", page: 6, about: "a row of dots, a hologram scan line: default.xbe names it (0x4070f8) for BF_HoloMaterial, which no material uses" },
    Picture { hash: 0xE53A_164B, library: COMMON, name: "rgbaGradient4x256.tga", page: 6, about: "a vertical gradient: default.xbe names it too (0x40712c)" },
    Picture { hash: 0x1A1E_C37F, library: COMMON, name: "", page: 6, about: "a grey ramp" },
];

/// A texture's format as the archive's index names it (tools/tex_tool.py `FORMATS`, the D3DFORMAT
/// codes), for a caption.
fn format_name(fmt: u32) -> String {
    match fmt {
        0x00 => "L8".into(),
        0x06 => "A8R8G8B8".into(),
        0x07 => "X8R8G8B8".into(),
        0x0B => "P8".into(),
        0x0C => "DXT1".into(),
        0x0E => "DXT3".into(),
        0x0F => "DXT5".into(),
        0x12 => "LIN_A8R8G8B8".into(),
        0x19 => "A8".into(),
        0x1E => "LIN_X8R8G8B8".into(),
        other => format!("format {other:#x}"),
    }
}

/// How a `w` x `h` picture is magnified beside itself, the factors across and down: (1, 1) for none.
/// When both sides are at most ZOOM_UP_TO texels, the largest whole factor up to ZOOM_MAX that keeps
/// the longer side within ZOOM_BOX pixels, both ways (a 32 x 32 face x4, a 64 x 4 scan line x2). A
/// thin strip is stretched across its thin side only, until that is STRIP_TO pixels (the 4 x 256
/// gradient x8 across, to 32 x 256). The 128 x 128 select pictures and the like aren't magnified.
fn zoom_of(w: u32, h: u32) -> (u32, u32) {
    if w <= ZOOM_UP_TO && h <= ZOOM_UP_TO {
        let z = (ZOOM_BOX / w.max(h).max(1)).clamp(1, ZOOM_MAX);
        return (z, z);
    }
    if w < STRIP_UNDER && h > ZOOM_UP_TO {
        return ((STRIP_TO / w.max(1)).max(1), 1);
    }
    if h < STRIP_UNDER && w > ZOOM_UP_TO {
        return (1, (STRIP_TO / h.max(1)).max(1));
    }
    (1, 1)
}

/// The page a hook names: its number from 1, or a group's name in any case (the first page of the
/// first group that contains it: `faces`, `other`).
fn page_named(s: &str) -> Option<usize> {
    let s = s.trim().to_ascii_lowercase();
    match s.parse::<usize>() {
        Ok(n) => (1..=PAGES.len()).contains(&n).then(|| n - 1),
        Err(_) => (!s.is_empty()).then(|| PAGES.iter().position(|p| p.group.contains(&s))).flatten(),
    }
}

/// The first page of the group after `page`'s (wrapping round to the first), Page Down's.
fn next_group(page: usize) -> usize {
    (page + 1..PAGES.len()).find(|&p| PAGES[p].group != PAGES[page].group).unwrap_or(0)
}

/// The first page of the group before `page`'s (wrapping round to the last group's), Page Up's.
fn previous_group(page: usize) -> usize {
    let first_of = |p: usize| (0..=p).rev().take_while(|&q| PAGES[q].group == PAGES[p].group).last().unwrap_or(p);
    let before = first_of(page);
    first_of(if before == 0 { PAGES.len() - 1 } else { before - 1 })
}

/// One picture as read from its library: its place in `PICTURES`, its size, format and mip count
/// from the index, and its top level as RGBA8.
struct Read {
    index: usize,
    w: u32,
    h: u32,
    fmt: u32,
    mips: u32,
    rgba: Vec<u8>,
}

/// What the loading thread sends: the pictures it found and how long it took (s), or why it
/// couldn't read the libraries.
type Pictures = Result<(Vec<Read>, f32), String>;

/// Reads the pictures of `library` (data/<library>.tgz: its textures-*.xmb indexes and .tex data,
/// as `Game::load_textures` reads them) that `PICTURES` lists, decoded to RGBA8.
fn read_library(dir: &std::path::Path, schemas: &SchemaSet, library: &str) -> Result<Vec<Read>, String> {
    let ar = Archive::open(&dir.join(format!("{library}.tgz")), |n| n.starts_with("textures-"))?;
    let mut out = vec![];
    for n in ar.find("textures-", ".xmb") {
        let Some(index) = ar.get(&n) else { continue };
        let root = bxml::parse(index, schemas).map_err(|e| format!("{n}: {e}"))?;
        let blob = ar.get(&n.replace(".xmb", ".tex")).unwrap_or(&[]);
        for t in texture::index(&root) {
            let Some(i) = PICTURES.iter().position(|p| p.library == library && p.hash == t.name) else { continue };
            if let Some(rgba) = texture::decode(&t, blob) {
                out.push(Read { index: i, w: t.w, h: t.h, fmt: t.fmt, mips: t.mips, rgba });
            }
        }
    }
    Ok(out)
}

/// Reads every picture on a thread (its libraries one after the other), as the music and sky
/// menus load theirs: the demo goes on meanwhile.
fn read_pictures(schemas: SchemaSet) -> Mutex<Receiver<Pictures>> {
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let t = std::time::Instant::now();
        let dir = data_dir();
        let got = [SPLASH, COMMON, TUTORIAL].iter().try_fold(vec![], |mut all, lib| {
            all.extend(read_library(&dir, &schemas, lib)?);
            Ok::<_, String>(all)
        });
        let _ = tx.send(got.map(|all| (all, t.elapsed().as_secs_f32())));
    });
    Mutex::new(rx)
}

/// A picture made ready: its image (nearest sampling: a texel stays a texel when it's magnified) and
/// its size, format and mip count for the caption.
struct Shown {
    image: Handle<Image>,
    w: u32,
    h: u32,
    fmt: u32,
    mips: u32,
}

/// The gallery's state: the page shown, the pictures once they're read (by `PICTURES` index: None
/// where the library had none by that hash), the thread reading them and what's drawn now.
#[derive(Resource, Default)]
struct Gallery {
    /// the page shown (a `PAGES` index)
    page: usize,
    /// the pictures once read: `None` until the thread is in
    shown: Option<Vec<Option<Shown>>>,
    /// the thread reading them, and whether it was started (once a session: the images stay)
    loading: Option<Mutex<Receiver<Pictures>>>,
    started: bool,
    /// why the libraries couldn't be read
    failed: Option<String>,
    /// the picture panel in the world and the page it was made for
    root: Option<(Entity, usize)>,
}

/// The picture panel's root node.
#[derive(Component)]
struct GalleryRoot;

/// The tools' panel's text while the gallery is open (play_testtools.rs `show_panel`): the page's
/// title, its place and the keys.
#[derive(Resource, Default)]
pub(super) struct GalleryText(pub(super) String);

/// The gallery's systems: its keys, hooks and pages after the character's update, before the tools'
/// panel is written.
pub fn plugin(app: &mut App) {
    app.init_resource::<Gallery>().init_resource::<GalleryText>()
        .add_systems(OnEnter(AppState::Playing), forget_panel.after(setup).run_if(resource_exists::<TestMap>))
        .add_systems(Update, gallery.after(update_player).before(super::testtools::show_panel).run_if(super::testtools::active));
}

/// A map starts: the picture panel of the map before is gone (`end_play` took its entities), so
/// the next opening makes it anew; the pictures read and the page stay.
fn forget_panel(mut gallery: ResMut<Gallery>, mut text: ResMut<GalleryText>) {
    gallery.root = None;
    text.0.clear();
}

/// The gallery: the hooks that open it or turn its page, its keys while it's open, the pictures
/// read on a thread the first time and made into images, the page's panel made when it changes and
/// taken down when the gallery closes, and the tools' panel's text.
#[allow(clippy::too_many_arguments)]
fn gallery(mut commands: Commands, keys: Res<ButtonInput<KeyCode>>, scroll: Res<AccumulatedMouseScroll>, mut tools: ResMut<Tools>,
           script: Res<Script>, mut gallery: ResMut<Gallery>, game: Res<GameData>, mut images: ResMut<Assets<Image>>,
           mut status: ResMut<UsePanel>, mut text: ResMut<GalleryText>) {
    let log = std::env::var("BF_TOOLS_LOG").is_ok();
    for hook in &script.now {
        if let Hook::Gallery(page) = hook {
            tools.panel = Panel::Gallery;
            match page.as_deref().map(|p| (p, page_named(p))) {
                Some((_, Some(p))) => gallery.page = p,
                Some((p, None)) => println!("test tools: no gallery page {p}"),
                None => {}
            }
        }
    }
    let open = tools.panel == Panel::Gallery;
    if !open {
        if let Some((e, _)) = gallery.root.take() {
            commands.entity(e).try_despawn();
        }
        if !text.0.is_empty() {
            text.0.clear();
        }
        return;
    }
    // the pictures: read on a thread the first time it opens
    if !gallery.started {
        gallery.started = true;
        gallery.loading = Some(read_pictures(game.0.schemas.clone()));
        if log {
            println!("gallery: reading {} pictures from {SPLASH}, {COMMON} and {TUTORIAL}", PICTURES.len());
        }
    }
    let got = gallery.loading.as_ref().and_then(|rx| match rx.lock().ok()?.try_recv() {
        Ok(x) => Some(x),
        Err(std::sync::mpsc::TryRecvError::Empty) => None,
        Err(std::sync::mpsc::TryRecvError::Disconnected) => Some(Err("the reading thread stopped".into())),
    });
    if let Some(got) = got {
        gallery.loading = None;
        match got {
            Ok((read, secs)) => {
                let mut shown: Vec<Option<Shown>> = PICTURES.iter().map(|_| None).collect();
                for r in read {
                    let mut image = Image::new(bevy::render::render_resource::Extent3d { width: r.w, height: r.h, depth_or_array_layers: 1 },
                                               bevy::render::render_resource::TextureDimension::D2, r.rgba,
                                               bevy::render::render_resource::TextureFormat::Rgba8UnormSrgb, bevy::asset::RenderAssetUsages::default());
                    image.sampler = bevy::image::ImageSampler::nearest();
                    shown[r.index] = Some(Shown { image: images.add(image), w: r.w, h: r.h, fmt: r.fmt, mips: r.mips });
                }
                let missing: Vec<String> = PICTURES.iter().zip(&shown).filter(|(_, s)| s.is_none()).map(|(p, _)| format!("h_{:08x} ({})", p.hash, p.library)).collect();
                if log || !missing.is_empty() {
                    println!("gallery: read {} of {} pictures in {secs:.2} s{}", PICTURES.len() - missing.len(), PICTURES.len(),
                             if missing.is_empty() { String::new() } else { format!("; not found: {}", missing.join(", ")) });
                }
                gallery.shown = Some(shown);
            }
            Err(e) => {
                eprintln!("gallery: couldn't read the pictures: {e}");
                status.message = Some(("No pictures: the libraries didn't read".into(), MESSAGE_TIME));
                gallery.failed = Some(e);
            }
        }
    }
    // the keys: Up / Down (or the wheel) a page, Page Up / Down a group
    let step = |k: KeyCode| keys.just_pressed(k) as i32;
    let wheel = if scroll.delta.y < 0.0 { 1 } else if scroll.delta.y > 0.0 { -1 } else { 0 };
    let d = step(KeyCode::ArrowDown) - step(KeyCode::ArrowUp) + wheel;
    gallery.page = (gallery.page as i32 + d).rem_euclid(PAGES.len() as i32) as usize;
    if keys.just_pressed(KeyCode::PageDown) {
        gallery.page = next_group(gallery.page);
    }
    if keys.just_pressed(KeyCode::PageUp) {
        gallery.page = previous_group(gallery.page);
    }
    // the page's panel, made again when the page changes or the pictures come in
    let page = gallery.page;
    let want = gallery.shown.is_some();
    if gallery.root.is_some_and(|(_, p)| p != page || !want) {
        if let Some((e, _)) = gallery.root.take() {
            commands.entity(e).try_despawn();
        }
    }
    if want && gallery.root.is_none() {
        let root = build_page(&mut commands, &gallery, page);
        gallery.root = Some((root, page));
        if log {
            let names: Vec<String> = PICTURES.iter().filter(|p| p.page == page)
                .map(|p| if p.name.is_empty() { format!("h_{:08x}", p.hash) } else { p.name.to_string() }).collect();
            println!("gallery: page {} / {} ({}): {}", page + 1, PAGES.len(), PAGES[page].group, names.join(", "));
        }
    }
    // the tools' panel (short lines: it sits at the right of the picture panel)
    let mut s = format!("UNUSED PICTURES   ([ / Esc: close)\npage {} / {}: {}\n", page + 1, PAGES.len(), PAGES[page].group);
    s += "Up / Down (wheel): page\nPage Up / Down: group\nthe game's own textures,\nnone used by its data or code";
    if gallery.loading.is_some() {
        s += "\n(reading the libraries...)";
    }
    if let Some(e) = &gallery.failed {
        s += &format!("\n(couldn't read them: {e})");
    }
    if text.0 != s {
        text.0 = s;
    }
}

/// A picture's node: `w` x `h` pixels (its own size, or magnified), grey behind it, and the image
/// turned over: the game's textures are stored with the bottom row first, as the HUD's and the
/// menus' pictures are (`ImageNode::flip_y`).
fn picture_node(image: &Handle<Image>, w: u32, h: u32) -> (Node, ImageNode, BackgroundColor) {
    (Node { width: Val::Px(w as f32), height: Val::Px(h as f32), flex_shrink: 0.0, ..default() },
     ImageNode { flip_y: true, ..ImageNode::new(image.clone()) }, BackgroundColor(PICTURE_BACKGROUND))
}

/// Makes page `page`'s panel: its header and a wrapping grid of its pictures, each in a cell of a
/// fixed height with the picture at its own size (and magnified beside itself if it's small or a
/// thin strip, `zoom_of`) and its caption below. Returns its root.
fn build_page(commands: &mut Commands, gallery: &Gallery, page: usize) -> Entity {
    let font = |size: f32| TextFont { font_size: size, ..default() };
    let root = commands.spawn((GalleryRoot, Name::new("gallery"), GlobalZIndex(GALLERY_Z), BackgroundColor(PANEL_COLOUR),
                               Node { position_type: PositionType::Absolute, left: Val::Px(PANEL_MARGIN), top: Val::Px(PANEL_MARGIN),
                                      width: Val::Percent(PANEL_WIDTH), flex_direction: FlexDirection::Column, row_gap: Val::Px(10.0),
                                      padding: UiRect::all(Val::Px(10.0)), ..default() })).id();
    commands.spawn((ChildOf(root), Text::new(format!("page {} / {}: {}", page + 1, PAGES.len(), PAGES[page].title)),
                    font(14.0), TextColor(Color::srgb(1.0, 0.85, 0.3))));
    commands.spawn((ChildOf(root), Text::new(format!("unused in the game: nothing in its data or code uses these. A picture of {ZOOM_UP_TO} pixels or less a side is also shown magnified beside itself, a thin strip stretched.")),
                    font(11.0), TextColor(Color::srgb(0.75, 0.75, 0.75))));
    let grid = commands.spawn((ChildOf(root), Node { flex_direction: FlexDirection::Row, flex_wrap: FlexWrap::Wrap, align_items: AlignItems::FlexStart,
                                                      align_content: AlignContent::FlexStart, column_gap: Val::Px(CELL_GAP), row_gap: Val::Px(14.0),
                                                      ..default() })).id();
    let shown = gallery.shown.as_deref().unwrap_or(&[]);
    for (i, p) in PICTURES.iter().enumerate().filter(|(_, p)| p.page == page) {
        let pic = shown.get(i).and_then(Option::as_ref);
        let (w, h) = pic.map_or((CELL_WIDTH as u32, 16), |s| (s.w, s.h));
        let (zx, zy) = zoom_of(w, h);
        let magnified = (zx, zy) != (1, 1);
        let row_width = w as f32 + if magnified { ZOOM_GAP + (w * zx) as f32 } else { 0.0 };
        // (the picture row is as tall as the picture or its magnified copy, at least PICTURE_ROW: a
        // 4 x 256 gradient makes its cell taller)
        let row_height = PICTURE_ROW.max(h * zy);
        let cell = commands.spawn((ChildOf(grid), Node { width: Val::Px(row_width.max(CELL_WIDTH)), height: Val::Px(row_height as f32 + 6.0 + CAPTION_HEIGHT),
                                                          flex_direction: FlexDirection::Column, row_gap: Val::Px(3.0), flex_shrink: 0.0, ..default() })).id();
        let row = commands.spawn((ChildOf(cell), Node { height: Val::Px(row_height as f32), flex_direction: FlexDirection::Row, column_gap: Val::Px(ZOOM_GAP),
                                                         align_items: AlignItems::FlexStart, ..default() })).id();
        // the picture at its own size, and the magnified copy
        match pic {
            Some(s) => {
                commands.spawn((ChildOf(row), picture_node(&s.image, s.w, s.h)));
                if magnified {
                    commands.spawn((ChildOf(row), picture_node(&s.image, s.w * zx, s.h * zy)));
                }
            }
            None => {
                commands.spawn((ChildOf(row), Text::new(format!("(not found in {})", p.library)), font(11.0), TextColor(Color::srgb(1.0, 0.4, 0.4))));
            }
        }
        // the caption: name (or none), hash, library, size and format, the catalogue's line
        let name = if p.name.is_empty() { "(unnamed)" } else { p.name };
        commands.spawn((ChildOf(cell), Text::new(name), font(13.0), TextColor(Color::WHITE)));
        // (a thin strip's copy is stretched one way only: said under its size)
        let strip = match (zx, zy) {
            (x, 1) if x > 1 => format!("\nright: stretched x{x} across"),
            (1, y) if y > 1 => format!("\nright: stretched x{y} down"),
            _ => String::new(),
        };
        let meta = match pic {
            Some(s) => format!("h_{:08x}\n{}\n{} x {}  {}{}{strip}", p.hash, p.library, s.w, s.h, format_name(s.fmt),
                               if s.mips > 1 { format!(", {} mips", s.mips) } else { String::new() }),
            None => format!("h_{:08x}\n{}", p.hash, p.library),
        };
        commands.spawn((ChildOf(cell), Text::new(meta), font(11.0), TextColor(Color::srgb(0.65, 0.75, 0.95))));
        commands.spawn((ChildOf(cell), Text::new(p.about), font(11.0), TextColor(Color::srgb(0.82, 0.82, 0.82))));
    }
    root
}

#[cfg(test)]
mod tests {
    use super::*;
    use bf_viewer::bf::hash::h;

    /// The catalogue's 40 unreferenced textures, each once: 10 in common (its 5 and the 5 faces that
    /// copies elsewhere share), 27 in splash_screen and 3 in the tutorial.
    #[test]
    fn forty_pictures_each_once() {
        assert_eq!(PICTURES.len(), 40);
        let mut hashes: Vec<u32> = PICTURES.iter().map(|p| p.hash).collect();
        hashes.sort_unstable();
        hashes.dedup();
        assert_eq!(hashes.len(), 40, "a hash is listed twice");
        let of = |lib: &str| PICTURES.iter().filter(|p| p.library == lib).count();
        assert_eq!((of(COMMON), of(SPLASH), of(TUTORIAL)), (10, 27, 3));
    }

    /// Every name is the one that hashes to its picture's hash (the recovered names are the
    /// catalogue's and the draft's: a typo here would show a wrong name).
    #[test]
    fn names_hash_to_their_pictures() {
        for p in PICTURES.iter().filter(|p| !p.name.is_empty()) {
            assert_eq!(h(p.name), p.hash, "{} isn't h_{:08x}", p.name, p.hash);
        }
        // (26 of the 40 have a recovered name; the other 14 have none)
        assert_eq!(PICTURES.iter().filter(|p| !p.name.is_empty()).count(), 26);
    }

    /// Every page has pictures, the pictures come in page order, and the ticket's groups are all
    /// there in its order (cut characters, cut squad, faces, map, other).
    #[test]
    fn pages_hold_the_tickets_groups_in_order() {
        for page in 0..PAGES.len() {
            assert!(PICTURES.iter().any(|p| p.page == page), "page {page} is empty");
        }
        assert!(PICTURES.windows(2).all(|w| w[0].page <= w[1].page));
        let groups: Vec<&str> = PAGES.iter().map(|p| p.group).collect();
        assert_eq!(groups, ["cut characters", "cut squad", "faces", "map", "other", "other", "other"]);
        // (the sizes the ticket lists: ten select pictures, the group picture, eight faces, the map; no
        // page more than ten cells: two rows of five)
        let on = |page: usize| PICTURES.iter().filter(|p| p.page == page).count();
        assert_eq!((on(0), on(1), on(2), on(3), on(4) + on(5) + on(6)), (10, 1, 8, 1, 20));
        assert!((0..PAGES.len()).all(|p| on(p) <= 10));
    }

    /// A hook names a page by its number from 1 or its group's name, any case; Page Up / Down
    /// jump to the first page of the previous / next group, wrapping.
    #[test]
    fn pages_by_name_and_group() {
        assert_eq!(page_named("1"), Some(0));
        assert_eq!(page_named("7"), Some(6));
        assert_eq!(page_named("0"), None);
        assert_eq!(page_named("8"), None);
        assert_eq!(page_named("Faces"), Some(2));
        assert_eq!(page_named("other"), Some(4));
        assert_eq!(page_named("squad"), Some(1));
        assert_eq!(page_named("nothing"), None);
        assert_eq!(page_named(""), None);
        // down: 0 -> 1 -> 2 -> 3 -> 4 (the "other" group, three pages) -> 0
        assert_eq!([0, 1, 2, 3, 4, 5, 6].map(next_group), [1, 2, 3, 4, 0, 0, 0]);
        // up: 0 -> the last group's first page (4); from any "other" page to the group before it
        assert_eq!([0, 1, 2, 3, 4, 5, 6].map(previous_group), [4, 0, 1, 2, 3, 3, 3]);
    }

    /// Small pictures are magnified beside themselves, by the largest whole factor up to 4 that keeps
    /// them within 128 px; big ones and long thin ones aren't.
    #[test]
    fn small_pictures_are_magnified() {
        assert_eq!(zoom_of(32, 32), (4, 4));
        assert_eq!(zoom_of(8, 16), (4, 4));
        assert_eq!(zoom_of(64, 4), (2, 2), "a side of 64 still counts as small");
        assert_eq!(zoom_of(128, 128), (1, 1));
        assert_eq!(zoom_of(32, 128), (1, 1));
        assert_eq!(zoom_of(512, 32), (1, 1));
    }

    /// A thin strip is stretched across its thin side only, up to 32 pixels: the 4 x 256 gradient
    /// (x8 across, 32 x 256), a long thin one the other way up; not a side of 8 texels, and not one
    /// that is small all round.
    #[test]
    fn a_thin_strip_is_stretched() {
        assert_eq!(zoom_of(4, 256), (8, 1));
        assert_eq!(zoom_of(65, 4), (1, 8), "one just over 64 long: a strip the other way up");
        assert_eq!(zoom_of(512, 2), (1, 16));
        assert_eq!(zoom_of(7, 256), (4, 1));
        assert_eq!(zoom_of(8, 256), (1, 1), "a side of 8 is not thin");
        assert_eq!(zoom_of(1, 100), (32, 1));
    }

    /// The captions name the formats the archives hold.
    #[test]
    fn formats_are_named() {
        assert_eq!(format_name(0x0E), "DXT3");
        assert_eq!(format_name(0x0C), "DXT1");
        assert_eq!(format_name(0x07), "X8R8G8B8");
        assert_eq!(format_name(0x06), "A8R8G8B8");
        assert_eq!(format_name(0x99), "format 0x99");
    }
}
