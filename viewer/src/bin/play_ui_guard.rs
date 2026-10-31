//! Keeping non-finite numbers out of the UI, and finding where they come from (#349).
//!
//! The product owner's crash (main, 2026-10-31, not reproduced): `bf_play` panicked in bevy_ui's
//! `ui_layout_system` with "f32 clamp, max NaN", after the ALE warning for effect h_eb4128d7
//! (`rgun_side_s`, the Rail RVR's empty side-arc effect, in its flight effect object h_0969d0fb:
//! the warning is the game's own empty effect, logged when the first Rail shot loads it, so it says
//! only that a Rail RVR was fired shortly before). The panic is bevy_ui 0.16.1's
//! `BorderRadius::resolve_single_corner`, `.clamp(0., 0.5 * node_size.min_element())`
//! (ui_node.rs:2448), which `update_uinode_geometry_recursive` runs for every node (each `Node`
//! has a `BorderRadius`, 0 by default) and which panics on a NaN bound: a node whose laid-out
//! size is NaN.
//!
//! What makes one NaN was measured by hand (`BF_UI_GUARD_TEST` below, the net off, the radar's
//! blips given a NaN or an infinity, as `Val::Px` and as `Val::Percent`, one field at a time; the
//! table is in viewer/docs/play.md, "Keeping non-finite numbers out of the UI"):
//!  - the **margin** reproduces the product owner's panic to the word ("min > max, or either was
//!    NaN. min = 0.0, max = NaN", in `ui_layout_system`);
//!  - `left` or `top` panics too, but the next frame in `ui_picking` and `ui_focus_system`
//!    ("Rect size must be positive");
//!  - the width, height, `right`, `bottom`, minimums and maximums, padding, border, gap, flex basis,
//!    flex factors and aspect ratio don't panic.
//!
//! The only margins this crate writes are its glyph nodes' (play_text.rs `spawn_glyphs`,
//! play_menu.rs `text`), from the glyph scale `cap / font cap` where `cap` is a constant times the
//! window's height over 480, so as far as reading the code goes, nothing makes one NaN while the
//! window has a height (a minimised window has none: that case is guarded below). Which number went
//! NaN or infinite in the product owner's run, and where, isn't known: the Rail RVR's shot is the
//! only lead, and no run reproduced it (the attempts are in the same section).
//!
//! So three things, none of them a fix of the cause (the ticket stays open for the backtrace):
//!  - **At the source**: `finite` and `percent` turn a computed number that isn't finite into 0
//!    and say so once, naming the node and the call site (`#[track_caller]`); the radar's blips,
//!    the HUD's `at`, the bars' widths, the text nodes' places and the target marker's box call
//!    them (play_hud.rs, play_text.rs, play_fonts.rs, play_targeting.rs), and the window's unit
//!    (height / 480) that a text or a ring is scaled by is checked before it divides.
//!  - **A net** (`net`): every `Node` changed this frame, in every state (the front end's and the
//!    briefing's glyphs carry margins as the game texts' do), is scanned before the layout, after
//!    the systems that write nodes in PostUpdate (the text and the fonts), and any non-finite length is
//!    reset (a position or size to `Val::Auto`, a margin, padding, border or gap to 0) with a
//!    warning naming the node (its `Name` or its nearest named ancestor's, and its components, so
//!    the writer's own marker shows: `SquadPart` ...); a `UiScale` that isn't finite and above 0
//!    is put back to 1. Nothing non-finite reaches `ui_layout_system` through a `Node`'s style any
//!    more, whoever wrote it. (A size a `ContentSize` measure gives, an image's or a text's, isn't
//!    a style and isn't scanned.)
//!  - **A watch** (`watch`): the first frame a character's position, facing, stamina or health, or
//!    the camera's heading, isn't finite, a warning names it, the time and what it was doing, with
//!    its last good position: that frame is the cause's, the UI guards only keep it from the
//!    layout. (It doesn't repair the character: a NaN position is a bug elsewhere to be found,
//!    not hidden.)
//!
//! Test hooks: `BF_UI_GUARD_TEST=<s>[,<field>[,px][,inf]]` at that sim time sets a field of the
//! style (`width` by default; `scan`'s names) of the nodes that have a border radius of their own,
//! the radar's blips and dot, to NaN, as a `Val::Percent` or, with `px`, a `Val::Px`, and with
//! `inf` to infinity, after the frame's writers (the net must reset it: no panic); or
//! `BF_UI_GUARD_TEST=<s>,member` sets the first squad member's position to NaN (the watch names
//! it, the blip's guard hides its dot). `BF_UI_GUARD_OFF=1` turns the net off, which shows the
//! panic it prevents.

use std::collections::{HashMap, HashSet};
use std::sync::{Mutex, OnceLock};

use super::*;

/// What's been reported (a key for each: a call site and a name, a node, a character), so each
/// thing is logged once however many frames it goes on.
fn reported() -> &'static Mutex<HashSet<String>> {
    static REPORTED: OnceLock<Mutex<HashSet<String>>> = OnceLock::new();
    REPORTED.get_or_init(|| Mutex::new(HashSet::new()))
}

/// Logs `message` as a warning the first time `key` comes; returns whether it did.
pub(crate) fn once(key: &str, message: impl FnOnce() -> String) -> bool {
    let first = reported().lock().is_ok_and(|mut seen| seen.insert(key.to_string()));
    if first {
        warn!("{}", message());
    }
    first
}

/// A computed UI number: `v` if it's finite, else 0, the first time from this call site saying
/// what it was and which node (`what`, made only then) it was for.
#[track_caller]
pub(crate) fn finite_with(what: impl FnOnce() -> String, v: f32) -> f32 {
    if v.is_finite() {
        return v;
    }
    let at = std::panic::Location::caller();
    let what = what();
    once(&format!("{what} @ {at}"), || format!("UI guard: {what} was {v} at {at}: drawn as 0 (#349)"));
    0.0
}

/// `finite_with` for a name already to hand.
#[track_caller]
pub(crate) fn finite(what: &str, v: f32) -> f32 {
    finite_with(|| what.to_string(), v)
}

/// `Val::Percent` of a computed number (`finite`).
#[track_caller]
pub(crate) fn percent(what: &str, v: f32) -> Val {
    Val::Percent(finite(what, v))
}

/// Whether a length is a finite number (or auto).
fn val_ok(v: Val) -> bool {
    match v {
        Val::Auto => true,
        Val::Px(x) | Val::Percent(x) | Val::Vw(x) | Val::Vh(x) | Val::VMin(x) | Val::VMax(x) => x.is_finite(),
    }
}

/// The non-finite lengths and numbers of a node's style, reset in place: a position, size or flex
/// basis to auto, a margin, padding, border or gap to 0, the aspect ratio to none, the flex factors
/// to their defaults. Returns each field's name and what it was; empty if the node was clean.
fn scan(node: &mut Node) -> Vec<String> {
    let mut bad = vec![];
    macro_rules! length {
        ($name:expr, $field:expr, $fallback:expr) => {
            if !val_ok($field) {
                bad.push(format!("{} {:?}", $name, $field));
                $field = $fallback;
            }
        };
    }
    length!("left", node.left, Val::Auto);
    length!("right", node.right, Val::Auto);
    length!("top", node.top, Val::Auto);
    length!("bottom", node.bottom, Val::Auto);
    length!("width", node.width, Val::Auto);
    length!("height", node.height, Val::Auto);
    length!("min_width", node.min_width, Val::Auto);
    length!("min_height", node.min_height, Val::Auto);
    length!("max_width", node.max_width, Val::Auto);
    length!("max_height", node.max_height, Val::Auto);
    length!("flex_basis", node.flex_basis, Val::Auto);
    length!("row_gap", node.row_gap, Val::Px(0.0));
    length!("column_gap", node.column_gap, Val::Px(0.0));
    length!("margin.left", node.margin.left, Val::Px(0.0));
    length!("margin.right", node.margin.right, Val::Px(0.0));
    length!("margin.top", node.margin.top, Val::Px(0.0));
    length!("margin.bottom", node.margin.bottom, Val::Px(0.0));
    length!("padding.left", node.padding.left, Val::Px(0.0));
    length!("padding.right", node.padding.right, Val::Px(0.0));
    length!("padding.top", node.padding.top, Val::Px(0.0));
    length!("padding.bottom", node.padding.bottom, Val::Px(0.0));
    length!("border.left", node.border.left, Val::Px(0.0));
    length!("border.right", node.border.right, Val::Px(0.0));
    length!("border.top", node.border.top, Val::Px(0.0));
    length!("border.bottom", node.border.bottom, Val::Px(0.0));
    if node.aspect_ratio.is_some_and(|a| !a.is_finite()) {
        bad.push(format!("aspect_ratio {:?}", node.aspect_ratio));
        node.aspect_ratio = None;
    }
    if !node.flex_grow.is_finite() {
        bad.push(format!("flex_grow {}", node.flex_grow));
        node.flex_grow = 0.0;
    }
    if !node.flex_shrink.is_finite() {
        bad.push(format!("flex_shrink {}", node.flex_shrink));
        node.flex_shrink = 1.0;
    }
    bad
}

/// The net: each `Node` written this frame, scanned before the layout (see the module notes). A
/// node's style is cloned and scanned (a style with no grid tracks is plain data), the original
/// written back only if something was reset, so a clean node isn't marked changed again.
fn net(mut commands: Commands, mut nodes: Query<(Entity, &mut Node, Option<&Name>), Changed<Node>>, parents: Query<&ChildOf>, names: Query<&Name>,
       mut ui_scale: ResMut<UiScale>) {
    // (the scale every length is multiplied by, set by the menus and the briefing from the window:
    // 0, NaN or infinite would make every node's size so, whatever its style)
    if !(ui_scale.0.is_finite() && ui_scale.0 > 0.0) {
        once("ui scale", || format!("UI guard: UiScale was {}: reset to 1 (#349)", ui_scale.0));
        ui_scale.0 = 1.0;
    }
    for (e, mut node, name) in &mut nodes {
        let mut copy = (*node).clone();
        let bad = scan(&mut copy);
        if bad.is_empty() {
            continue;
        }
        *node = copy;
        // (named, or by the nearest named ancestor: most nodes have no name)
        let who = name.map(|n| format!("\"{n}\"")).unwrap_or_else(|| {
            let mut at = e;
            for _ in 0..8 {
                let Ok(up) = parents.get(at) else { break };
                at = up.parent();
                if let Ok(n) = names.get(at) {
                    return format!("under \"{n}\"");
                }
            }
            "unnamed".to_string()
        });
        if once(&format!("node {e:?}"), || format!("UI guard: {e:?} ({who}) had a non-finite style, reset: {} (#349); its components follow", bad.join(", "))) {
            // (its components, to name the writer: queued, the world isn't ours here)
            commands.queue(move |world: &mut World| {
                if let Ok(infos) = world.inspect_entity(e) {
                    let list: Vec<String> = infos.map(|i| i.name().rsplit("::").next().unwrap_or("?").to_string()).collect();
                    warn!("UI guard: {e:?}'s components: {}", list.join(", "));
                }
            });
        }
    }
}

/// Whether the net is on (BF_UI_GUARD_OFF=1 turns it off, to see the crash it prevents).
fn net_on() -> bool {
    std::env::var("BF_UI_GUARD_OFF").is_err()
}

/// The watch's memory: each character's last finite position by who it is.
#[derive(Default)]
struct Seen {
    last: HashMap<Who, Vec3>,
}

/// The watch (see the module notes): the player's and each squad member's position and the
/// camera's heading, each logged the first time it isn't finite, with the time, the character's
/// name, what it was doing and its last good position.
fn watch(player: Res<Player>, squad: Res<Squad>, mut seen: Local<Seen>) {
    for u in std::iter::once(&*player).chain(squad.0.iter()) {
        let who = u.who();
        // (the health the HUD's bars are sized by, and its maximum)
        if !(u.health.is_finite() && u.max_health.is_finite()) {
            once(&format!("health {who:?}"), || format!(
                "UI guard: {}'s health is {} of {} at sim t={:.2} (dead {}, controlled {}): the HUD's bars are sized by it, #349's lead",
                unit_name(u), u.health, u.max_health, player.sim_time, u.dead, std::ptr::eq(u, &*player)));
        }
        // (the facing angle the target marker's box is placed by, and the stamina the bar is sized by)
        if !(u.yaw.is_finite() && u.stamina.is_finite()) {
            once(&format!("yaw or stamina {who:?}"), || format!(
                "UI guard: {}'s yaw is {} and stamina {} at sim t={:.2} (position {:?}, gait {:?}, action {:?}, dead {}): #349's lead",
                unit_name(u), u.yaw, u.stamina, player.sim_time, u.position, u.gait, u.action, u.dead));
        }
        if u.position.is_finite() {
            seen.last.insert(who, u.position);
            continue;
        }
        let last = seen.last.get(&who).copied();
        once(&format!("position {who:?}"), || format!(
            "UI guard: {}'s position is {:?} at sim t={:.2} (last finite {last:?}; gait {:?}, action {:?}, dead {}, absent {}, controlled {}): the first frame it went non-finite, #349's lead",
            unit_name(u), u.position, player.sim_time, u.gait, u.action, u.dead, u.absent, std::ptr::eq(u, &*player)));
    }
    if !player.cam_yaw.is_finite() || !player.cam_pitch.is_finite() {
        once("camera heading", || format!("UI guard: the camera heading is yaw {} pitch {} at sim t={:.2} (the radar's blips and the HUD's placement follow it): #349's lead",
                                           player.cam_yaw, player.cam_pitch, player.sim_time));
    }
}

/// What BF_UI_GUARD_TEST sets to NaN: the first squad member's position, or a field of the
/// style of each node that has a border radius of its own (the radar's blips and dot).
#[derive(Clone, PartialEq)]
enum Test {
    Member,
    /// a field of the style by name, whether the bad number is a `Val::Px` (else a `Val::Percent`),
    /// and the number (NaN, or infinity)
    Field(String, bool, f32),
}

/// BF_UI_GUARD_TEST's `<s>[,member|<field>[,px][,inf]]`: when, and what is set (the nodes' width
/// by default, as a `Val::Percent`; `px` makes it a `Val::Px`, `inf` an infinity, not a NaN). The
/// fields are `scan`'s: width, height, left, right, top, bottom, margin, padding, border, aspect,
/// grow, shrink, basis, gap, min_width, max_width, min_height, max_height.
fn test_hook_args() -> Option<(f32, Test)> {
    let v = std::env::var("BF_UI_GUARD_TEST").ok()?;
    let mut it = v.split(',').map(str::trim);
    let at = it.next()?.parse().ok()?;
    Some((at, match it.next() {
        Some("member") => Test::Member,
        Some(field) => {
            let flags: Vec<&str> = it.collect();
            Test::Field(field.to_string(), flags.contains(&"px"), if flags.contains(&"inf") { f32::INFINITY } else { f32::NAN })
        }
        None => Test::Field("width".to_string(), false, f32::NAN),
    }))
}

/// Sets the style field `field` (a name from `test_hook_args`) of `node` to `bad` (NaN or
/// infinity), as a `Val::Px` if `px`, else a `Val::Percent`; whether it's a field.
fn poison(node: &mut Node, field: &str, px: bool, bad: f32) -> bool {
    let nan = if px { Val::Px(bad) } else { Val::Percent(bad) };
    match field {
        "width" => node.width = nan,
        "height" => node.height = nan,
        "left" => node.left = nan,
        "right" => node.right = nan,
        "top" => node.top = nan,
        "bottom" => node.bottom = nan,
        "margin" => node.margin = UiRect::all(nan),
        "padding" => node.padding = UiRect::all(nan),
        "border" => node.border = UiRect::all(nan),
        "aspect" => node.aspect_ratio = Some(bad),
        "grow" => node.flex_grow = bad,
        "shrink" => node.flex_shrink = bad,
        "basis" => node.flex_basis = nan,
        "gap" => (node.row_gap, node.column_gap) = (nan, nan),
        "min_width" => node.min_width = nan,
        "max_width" => node.max_width = nan,
        "min_height" => node.min_height = nan,
        "max_height" => node.max_height = nan,
        _ => return false,
    }
    true
}

/// BF_UI_GUARD_TEST (see the module notes), a member's position set in the frame's start so the
/// whole chain runs.
fn test_member(mut squad: ResMut<Squad>, player: Res<Player>, mut done: Local<bool>) {
    let Some((at, Test::Member)) = test_hook_args() else { return };
    if *done || player.sim_time < at {
        return;
    }
    if let Some(m) = squad.0.iter_mut().find(|m| m.in_squad() && !m.dead && !m.absent && !m.stand_in) {
        *done = true;
        m.position = Vec3::NAN;
        println!("t={:.2}: BF_UI_GUARD_TEST: {}'s position set to NaN", player.sim_time, unit_name(m));
    }
}

/// BF_UI_GUARD_TEST (see the module notes), the nodes with a border radius of their own (the
/// radar's blips and the viewer's dot: every `Node` has a `BorderRadius` component, 0 by default)
/// given a NaN in one field of their style after the frame's writers.
fn test_node(player: Res<Player>, mut done: Local<bool>, mut nodes: Query<(&mut Node, &BorderRadius)>) {
    let Some((at, Test::Field(field, px, bad))) = test_hook_args() else { return };
    if *done || player.sim_time < at {
        return;
    }
    *done = true;
    let mut n = 0;
    for (mut node, radius) in &mut nodes {
        if radius.top_left != Val::ZERO && poison(&mut node, &field, px, bad) {
            n += 1;
        }
    }
    println!("t={:.2}: BF_UI_GUARD_TEST: the {field} of {n} node(s) with a border radius set to {bad} ({})", player.sim_time, if px { "px" } else { "percent" });
}

/// The watch and the member hook while playing, and in PostUpdate the node hook (while playing) then
/// the net (in every state: the front end's and the briefing's glyph nodes carry margins too, the
/// one field that reproduces the layout panic), after the systems that write nodes there (the game
/// texts and the record texts) and before the layout reads them.
pub fn plugin(app: &mut App) {
    app.add_systems(Update, (watch.after(update_player), test_member.before(update_player)).run_if(in_state(AppState::Playing)))
        // (after the PostUpdate systems that write nodes, before the layout reads them)
        .add_systems(PostUpdate, (test_node.run_if(in_state(AppState::Playing)), net.run_if(net_on)).chain().after(super::text::draw)
            .after(super::fonts::DrawRecords).before(bevy::ui::UiSystem::Layout));
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A computed number that isn't finite is 0, whatever the node; a finite one is untouched.
    #[test]
    fn a_number_that_is_not_finite_is_zero() {
        assert_eq!(finite("test node", 3.5), 3.5);
        assert_eq!(finite("test node a", f32::NAN), 0.0);
        assert_eq!(finite("test node b", f32::INFINITY), 0.0);
        assert_eq!(percent("test node c", f32::NEG_INFINITY), Val::Percent(0.0));
        assert_eq!(percent("test node d", 12.5), Val::Percent(12.5));
    }

    /// Each thing is logged once however often it comes.
    #[test]
    fn each_report_comes_once() {
        assert!(once("test once key", || "x".into()));
        assert!(!once("test once key", || unreachable!()));
    }

    /// The net resets what isn't finite and nothing else, and says which fields.
    #[test]
    fn the_net_resets_only_what_is_not_finite() {
        let mut node = Node { left: Val::Percent(f32::NAN), top: Val::Px(4.0), width: Val::Percent(f32::INFINITY), height: Val::Auto,
                              margin: UiRect::left(Val::Px(f32::NAN)), aspect_ratio: Some(f32::NAN), flex_grow: f32::NAN, ..default() };
        let bad = scan(&mut node);
        assert_eq!(bad.len(), 5, "{bad:?}");
        assert_eq!((node.left, node.top, node.width, node.height), (Val::Auto, Val::Px(4.0), Val::Auto, Val::Auto));
        assert_eq!((node.margin.left, node.aspect_ratio, node.flex_grow), (Val::Px(0.0), None, 0.0));
        // clean now: nothing more to say or write
        assert!(scan(&mut node).is_empty());
        assert!(scan(&mut Node { left: Val::Percent(50.0), ..default() }).is_empty());
    }

    /// The net runs outside the Playing state too (#349's review: the front end's and the
    /// briefing's glyph nodes carry margins, the field that reproduces the layout panic): in the
    /// Menu state a node whose margin isn't finite is reset, a clean one left alone, and a
    /// `UiScale` that isn't finite and above 0 is put back to 1. The plugin's other systems stay
    /// Playing's (none of their resources exist here).
    #[test]
    fn the_net_runs_in_every_state() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, bevy::state::app::StatesPlugin)).insert_state(AppState::Menu).init_resource::<UiScale>().add_plugins(plugin);
        let bad = app.world_mut().spawn(Node { margin: UiRect::right(Val::Px(f32::NAN)), ..default() }).id();
        let fine = app.world_mut().spawn(Node { margin: UiRect::right(Val::Px(3.0)), ..default() }).id();
        app.world_mut().resource_mut::<UiScale>().0 = f32::NAN;
        app.update();
        assert_eq!(app.world().get::<Node>(bad).unwrap().margin.right, Val::Px(0.0));
        assert_eq!(app.world().get::<Node>(fine).unwrap().margin.right, Val::Px(3.0));
        assert_eq!(app.world().resource::<UiScale>().0, 1.0);
    }

    /// The hook's fields are the net's: each field `poison` can set, in either unit and either bad
    /// number, `scan` finds and resets, and a name it doesn't know is refused.
    #[test]
    fn the_net_covers_every_field_the_hook_can_poison() {
        let fields = ["width", "height", "left", "right", "top", "bottom", "margin", "padding", "border", "aspect", "grow", "shrink", "basis", "gap",
                      "min_width", "max_width", "min_height", "max_height"];
        for field in fields {
            for (px, bad) in [(false, f32::NAN), (true, f32::NAN), (false, f32::INFINITY), (true, f32::NEG_INFINITY)] {
                let mut node = Node::default();
                assert!(poison(&mut node, field, px, bad), "{field}");
                assert!(!scan(&mut node).is_empty(), "{field} {px} {bad}: not found");
                assert!(scan(&mut node).is_empty(), "{field} {px} {bad}: not reset");
            }
        }
        assert!(!poison(&mut Node::default(), "colour", false, f32::NAN));
    }
}
