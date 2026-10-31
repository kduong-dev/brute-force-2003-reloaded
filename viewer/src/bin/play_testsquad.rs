//! The test map's squad (#302): who holds the squad's four slots. Demo tooling, not the game (the
//! game's squad is always Brutus, Flint, Hawk and Tex, each in its own place; the keys, the start
//! and the layout are the product owner's choices).
//!
//!  - **The slots** are the squad's four places (`Player::character`): the HUD's portraits round
//!    the radar (Brutus's bottom, Flint's right, Hawk's left, Tex's top), their health channels,
//!    and the keys 1-4 that take control of them. In a test session each holds any character type
//!    the NPC menu offers (the four, then every loaded type: #128), or no one. A member is a real
//!    squad member, not an NPC (`Player::npc` unset): it has a formation place and the squad AI,
//!    control passes to it (1-4, BF_TEST_SELECT, the hand-over after a death), the HUD shows it
//!    and it takes the squad's pickups. Its data is its type's (play.rs `Player::data_slot`): a
//!    squad type in another's place is still itself (its skill, voice, camera, Brutus not
//!    kneeling); any other type has no skill, says only its death line, reacts, shields and dies
//!    as its type does (play_enemies.rs), and shows its type's portrait (the deathmatch types and
//!    some of the militia have one), else a "?" (play_hud.rs).
//!  - **The start** (`start`, from play.rs `begin_play`): the test map (`--test`, `--edit`, and
//!    the flat floor of a level switch) starts with the player's character (Tex, or
//!    BF_CHARACTER's) alone; the hooks below give it more, and a recipe with one of the squad's own
//!    hooks (SQUAD_HOOKS: BF_TEST_SELECT, BF_TEST_KILL, BF_TEST_TARGET ..., BF_TEST_DIE) the four,
//!    as before.
//!  - **A lone death** (`respawn`): on the test map, dying with nobody to hand over to brings the
//!    player back where it stood RESPAWN_TIME later (the demo's rule).
//!    Any other map keeps its own squad.
//!  - **The menu** (T: its key is the tool registry's, play_editor.rs `TOOLS`; on any map of a
//!    test session): Left / Right pick a slot, Up / Down (the wheel; Page Up / Down a page of
//!    ROWS) a row of the list: the predefined squads first (below), then the character types. It
//!    opens with the first type picked, as before the squads were added (`Row::default`), and its
//!    window on the top of the list with the squads in view (`window_top`). The panel keeps above
//!    the HUD's item box in every state (the tests count its lines). On a type,
//!    Enter or a click puts a fresh one of that type in the slot (`change`: a member of another
//!    type replaced where it stands, an empty slot's at its formation place round the leader, the
//!    controlled character's changed in place: its position, heading, view and the squad's
//!    inventory kept), Delete empties it (not the controlled character's). In the map editor's
//!    edit mode, with the squad out of the world, it changes the roster the next play makes.
//!  - **The predefined squads** (#339, `Preset`, `presets`): the campaign squad (common's Brutus,
//!    Flint, Hawk and Tex) and the five squads squad deathmatch offers (game-options' `<squad>`
//!    entries, `Game::option_squads`: Brute Force, Feral Hunting Pack, Shrike Raiders, Seer Coven
//!    and Ferguson Platoon, by the game's own names). On a squad, Enter or a click fills all four
//!    slots at once, each by the single-slot path above (`change`, the slots in the keys' order
//!    Brutus, Flint, Hawk, Tex): your own slot becomes its member in place, the others appear at
//!    their formation places round you (a member already there is replaced where it stands). A
//!    squad's members are the four types of its library: each of its game-options characters
//!    (h_fed0689a) names its type there (`GameCharacter::squad_type`, h_fa8c572a), and a match
//!    loads the library (FUN_000d5f60). Each sits in the place its type's h_072ab4af names, the
//!    HUD slot the game puts a character at on the radar (Tex 1 top, Flint 2 right, Brutus 3
//!    bottom, Hawk 4 left: FUN_000c84e0, `seat`).
//!  - **The roster** (`Roster`): each map's slots, as its start made them and the menu changed
//!    them; the map editor's play (F5) makes its fresh squad from it (play_editor.rs `to_play`).
//!
//! Test hooks: BF_TEST_SQUAD, BF_TEST_FULL_SQUAD (the test map's start: `start_roster`);
//! BF_TEST_SQUAD_MENU=<s> (T), BF_TEST_SQUAD_SLOT=<s>,<slot>,<type|empty>[;...] (the menu's
//! Enter or Delete on a type), BF_TEST_SQUAD_PRESET=<s>,<squad>[;...] (its Enter on a predefined
//! squad) and BF_TEST_SQUAD_PICK=<s>,<squad|type>[,<slot>] (the row and slot its keys pick),
//! timed as the tools' hooks (play_testtools.rs `read_hooks`). BF_TOOLS_LOG=1 prints the start
//! squad, the squads the menu lists and each change.

use super::*;
use super::testtools::{fill_npc_list, npc_kind, title, Hook, NpcList, Panel, Script, Tools, MESSAGE_TIME};

/// Each squad slot's type (None: empty), by CHARACTERS index: the map's squad as its start made
/// it and the squad menu has changed it since (a map's: `begin_play` makes it with the squad). The
/// map editor's play makes its fresh squad from it (`apply`).
#[derive(Resource, Clone, Copy, Debug, Default)]
pub(super) struct Roster(pub(super) [Option<u32>; SQUAD]);

impl Roster {
    /// The slots of `player` and its `squad`'s members (not the NPCs).
    pub(super) fn of(player: &Player, squad: &Squad) -> Roster {
        let mut r = [None; SQUAD];
        for u in std::iter::once(player).chain(squad.0.iter().filter(|m| m.in_squad())) {
            if let Some(s) = r.get_mut(u.character) {
                *s = Some(u.kind);
            }
        }
        Roster(r)
    }
}

/// The squad menu's picks: the slot (a CHARACTERS index), the row of its list picked (a predefined
/// squad, `Presets`, or a type of the NPC menu's list, `NpcList`) and the first row of the window
/// the panel shows (`top`, a position in the list: the squads, then the types). A map's: reset as
/// each one starts.
#[derive(Resource, Default)]
struct Menu {
    slot: usize,
    /// the row picked: the first character type to start with, as the menu was before the squads
    /// (#339) were added to it. Enter on a squad puts four slots at once, so it's chosen on purpose
    pick: Row,
    top: usize,
}

/// How many rows of the list the panel shows at once, and how far Page Up / Down jump (the demo's
/// choice: the tools' panel has to end above the HUD's item box in every state, which `PANEL_LINES`
/// in the tests counts; the NPC and object menus show LIST_LINES, 9).
const ROWS: usize = 7;

/// A predefined squad (#339): the type each of the four slots holds. The campaign squad and the
/// five squad deathmatch squads of game-options, as `presets` makes them from the loaded data.
#[derive(Clone, Debug)]
struct Preset {
    /// its name in the menu: the game's own for a squad deathmatch squad (game-options' `<squad
    /// name>`, a string of the string table: "Shrike Raiders"), the demo's for the campaign's
    name: String,
    /// where its types are defined (`CharacterType::place`: "common", "mp_common/sdm_shrike")
    place: String,
    /// its library as a hook may name it: the squad's game-options `library-name` ("sdm_shrike"),
    /// "common" for the campaign's
    library: String,
    /// the type each slot (a CHARACTERS index) holds, None for an empty one (every squad of the
    /// data has four members, one for each place)
    slots: [Option<u32>; SQUAD],
}

/// The menu's predefined squads (`presets`), made once a map's data is in (`squad_menu`) and
/// again with each map.
#[derive(Resource, Default)]
struct Presets(Vec<Preset>);

/// The squad menu's text, shown by play_testtools.rs's panel (`show_panel`).
#[derive(Resource, Default)]
pub(super) struct SquadText(pub(super) String);

/// The squad menu's systems: its state anew with each map, and the menu (after `read_input`,
/// before control passes: `squad_control`).
pub fn plugin(app: &mut App) {
    app.init_resource::<Menu>().init_resource::<SquadText>().init_resource::<Roster>().init_resource::<Presets>()
        .add_systems(OnEnter(AppState::Playing), reset.after(setup).run_if(resource_exists::<super::testmap::TestMap>))
        .add_systems(Update, squad_menu.after(read_input).after(super::editor::open_tools).before(squad_control).run_if(super::testtools::active))
        .add_systems(Update, respawn.after(super::deathcam::death_cam).before(update_hud).run_if(super::testtools::active))
        .add_systems(Update, mirror_roster.run_if(in_state(AppState::Playing)));
}

/// How long (s) a lone death on the test map lasts before the player respawns (`respawn`): the
/// death camera's time to its hand-over (play_deathcam.rs DEATH_CAM_TIME, 4 s), so a death lasts
/// as long with or without a squad to hand over to. The demo's choice.
const RESPAWN_TIME: f32 = 4.0;

/// Dying alone on the test map respawns the player (#302; the product owner's rule for the test
/// map, not the game's, where a lone death ends the mission): once the controlled character is
/// dead with nobody to hand over to (no living squad member in the world, no hand-over under
/// way: the death camera's next member, play_deathcam.rs), RESPAWN_TIME later it comes back where
/// it stood, a fresh one of its type (`change` on its own slot: its body taken away, its view,
/// the hooks' clock and the squad's inventory kept; the guns, blood and memory chip it left stay).
/// The dead character's own clock stops (`step_player` counts it), so meanwhile this counts the
/// hooks' clock on (`Player::sim_time`): the respawn and the hooks due during the wait and after it
/// keep time. Only on the test map (`testmap::on_test_floor`; a level a test session loads keeps
/// the main game's rule), and not for the map editor's stand-in (counted dead).
#[allow(clippy::too_many_arguments)]
fn respawn(mut commands: Commands, time: Res<Time>, mut player: ResMut<Player>, mut squad: ResMut<Squad>, mut hits: ResMut<AiHits>,
           (test, current, mode): (Option<Res<super::testmap::TestMap>>, Res<CurrentMap>, Res<Mode>), mut status: ResMut<UsePanel>,
           mut wait: Local<f32>) {
    let alone = player.dead && !player.stand_in && player.select.is_none()
        && !squad.0.iter().any(|m| !m.dead && !m.absent && m.in_squad() && m.loaded.is_some());
    if !alone || !super::testmap::on_test_floor(test.as_deref(), &current) {
        *wait = 0.0;
        return;
    }
    let dt = frame_dt(&time);
    player.sim_time += dt;
    *wait += dt;
    if *wait < RESPAWN_TIME {
        return;
    }
    *wait = 0.0;
    let (c, kind, who) = (player.character, player.kind, unit_name(&player));
    let Ok(ch) = change(c, Some(kind), &mut player, &mut squad, mode.deathmatch) else { return };
    for e in ch.gone {
        commands.entity(e).despawn();
    }
    if ch.moved {
        hits.0.clear();
    }
    status.message = Some(("Respawned".into(), MESSAGE_TIME));
    if std::env::var("BF_TOOLS_LOG").is_ok() {
        println!("t {:.2}: {who} died alone: respawned after {RESPAWN_TIME} s at {:.2}, its heading {:.0} deg", player.sim_time,
                 player.position + Vec3::Y * GROUND, player.yaw.to_degrees());
    }
}

/// Each squad slot's type as the map's roster has it, for the logs that know a squad member by its
/// slot alone (play.rs `who_name`: the combat log's shooter, a blast's or a cloud's thrower): 0
/// until a roster is made. Kept by `mirror_roster`.
static SLOT_KINDS: [std::sync::atomic::AtomicU32; SQUAD] = [const { std::sync::atomic::AtomicU32::new(0) }; SQUAD];

/// The type squad slot `c` holds on this map (`SLOT_KINDS`), if a roster says so.
pub(super) fn slot_kind(c: usize) -> Option<u32> {
    SLOT_KINDS.get(c).map(|k| k.load(std::sync::atomic::Ordering::Relaxed)).filter(|&k| k != 0)
}

/// Keeps `SLOT_KINDS` to the map's roster as it changes (a map's start, the squad menu).
fn mirror_roster(roster: Res<Roster>) {
    if !roster.is_changed() {
        return;
    }
    for (slot, kind) in SLOT_KINDS.iter().zip(roster.0) {
        slot.store(kind.unwrap_or(0), std::sync::atomic::Ordering::Relaxed);
    }
}

/// A new map: the menu's picks anew (its slots are the map's: `begin_play` made its roster), and
/// the predefined squads to be made again from the data it loads.
fn reset(mut menu: ResMut<Menu>, mut text: ResMut<SquadText>, mut presets: ResMut<Presets>) {
    *menu = Menu::default();
    text.0.clear();
    presets.0.clear();
}

/// Seats a squad's members in the four places (#339): each member is `(its type, its type's
/// h_072ab4af)`, the HUD slot the game puts the character at on the radar (FUN_000c84e0 adds it to
/// the HUD's squad list at its type's +0x150: Tex 1 top, Flint 2 right, Brutus 3 bottom, Hawk 4
/// left; the d-pad and the Squad Command menu go by it too, play_squad_menu.rs `SLOT_ORDER`).
/// Every squad of the data has the four slots once each, so each member lands in the place its
/// type names. A member whose slot is out of 1-4 or already taken (not in the data: a guess) takes
/// the first place still free in the keys' order, after the others are seated.
fn seat(members: &[(u32, i64)]) -> [Option<u32>; SQUAD] {
    let mut places = [None; SQUAD];
    let mut over = vec![];
    for &(kind, slot) in members {
        match slot.checked_sub(1).and_then(|i| usize::try_from(i).ok()).and_then(|i| super::squad_menu::SLOT_ORDER.get(i)).copied() {
            Some(c) if places[c].is_none() => places[c] = Some(kind),
            _ => over.push(kind),
        }
    }
    for kind in over {
        if let Some(c) = (0..SQUAD).find(|&c| places[c].is_none()) {
            places[c] = Some(kind);
        }
    }
    places
}

/// The menu's predefined squads (#339), from the loaded data:
///  - the campaign squad: common's Brutus, Flint, Hawk and Tex, each in its own place (the demo's
///    name for it; the game has none);
///  - then game-options' squad deathmatch squads (`Game::option_squads`, the file's order), by the
///    game's names. Each member is a game-options character (`h_fed0689a`, e.g. `mp_tex`), and the
///    type it stands for is its `squad_type` (h_fa8c572a: the type of that name in the squad's
///    library, e.g. sdm_bf's Tex h_fa6ca831; the library a match loads, FUN_000d5f60, holds
///    exactly the squad's four). It sits in the place its type's h_072ab4af names (`seat`).
///
/// A squad one of whose types isn't loaded with its model (the NPC menu's rule too) is left out,
/// with a line in the log.
fn presets(game: &Game) -> Vec<Preset> {
    let loaded = |k: u32| game.character_type(k).is_some_and(|t| game.has_model(t));
    let place = |k: u32| game.character_type(k).map_or_else(String::new, |t| t.place());
    let mut out = vec![Preset {
        name: "Campaign squad".into(),
        place: place(squad_type(0)),
        library: "common".into(),
        slots: std::array::from_fn(|c| Some(squad_type(c))),
    }];
    for s in &game.option_squads {
        let name = game.strings.get(&s.name).cloned().unwrap_or_else(|| format!("squad h_{:08x}", s.name));
        let kinds: Vec<Option<u32>> = s.members.iter()
            .map(|a| game.option_characters.iter().find(|c| c.archetype == *a).map(|c| c.squad_type).filter(|&k| loaded(k))).collect();
        if kinds.is_empty() || kinds.iter().any(Option::is_none) {
            println!("test map: squad {name} ({}): a member's type isn't loaded with its model: left out", s.library);
            continue;
        }
        let members: Vec<(u32, i64)> = kinds.into_iter().flatten().map(|k| (k, game.character_type(k).map_or(0, |t| t.footstep_type))).collect();
        out.push(Preset { name, place: place(members[0].0), library: s.library.clone(), slots: seat(&members) });
    }
    out
}

/// The predefined squad a hook names (#339): its number in the menu's list from 0, its name (any
/// case: "shrike raiders"), its library ("sdm_shrike", or "shrike" for short; "common" for the
/// campaign's), else the first whose name contains it ("campaign", "raiders").
fn preset_named(list: &[Preset], s: &str) -> Option<usize> {
    let s = s.trim().to_ascii_lowercase();
    if s.is_empty() {
        return None;
    }
    if let Ok(i) = s.parse::<usize>() {
        return (i < list.len()).then_some(i);
    }
    let short = |l: &str| l.to_ascii_lowercase().strip_prefix("sdm_").map(str::to_string);
    list.iter().position(|p| p.name.to_ascii_lowercase() == s || p.library.to_ascii_lowercase() == s || short(&p.library).is_some_and(|l| l == s))
        .or_else(|| list.iter().position(|p| p.name.to_ascii_lowercase().contains(&s)))
}

/// What a row of the menu's list is: a predefined squad (an index into `Presets`) or a character
/// type (an index into the NPC menu's list). The squads come first.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Row {
    Squad(usize),
    Type(usize),
}

impl Default for Row {
    /// The first character type: the row the menu opened on before the squads were added (#339),
    /// whatever number of squads the data gives.
    fn default() -> Row {
        Row::Type(0)
    }
}

/// The row at position `pick` of the list, with `squads` predefined squads before the types.
fn row(pick: usize, squads: usize) -> Row {
    if pick < squads { Row::Squad(pick) } else { Row::Type(pick - squads) }
}

/// Where a row stands in the list (the inverse of `row`).
fn position(r: Row, squads: usize) -> usize {
    match r {
        Row::Squad(i) => i,
        Row::Type(i) => squads + i,
    }
}

/// The first row of the window (ROWS rows of a list of `rows`) that holds position `at`, moved as
/// little as it takes from where it was (`top`): the window stays put while the pick moves inside
/// it and scrolls a row at a time past its edges, so the menu opens on its first rows with the
/// squads in view and the first type picked at the bottom.
fn window_top(top: usize, at: usize, rows: usize) -> usize {
    top.max((at + 1).saturating_sub(ROWS)).min(at).min(rows.saturating_sub(ROWS))
}

/// What the menu is asked to do in a frame: put a type in a slot (None: empty it), or fill all
/// four slots from a predefined squad (an index into `Presets`).
#[derive(Clone, Copy, Debug)]
enum Ask {
    Slot(usize, Option<u32>),
    Squad(usize),
}

/// The character type a hook names (as BF_TEST_NPC names one: play_testtools.rs `npc_kind`), if
/// its model is loaded too (one without would have no body: `spawn_unit` can't make it; the NPC
/// menu lists only those).
fn named_type(game: &Game, s: &str) -> Option<u32> {
    npc_kind(game, s).filter(|&k| game.character_type(k).is_some_and(|t| game.has_model(t)))
}

/// The squad slot a hook names: 0-3, or brutus / flint / hawk / tex (its place, as BF_TEST_SELECT
/// and BF_TEST_HOSTILE name a squad member), or `me` (the controlled character's, `me`). Also
/// BF_TEST_ORDER's (play_orders.rs).
pub fn slot_named(s: &str, me: usize) -> Option<usize> {
    match s.trim().to_ascii_lowercase().as_str() {
        "me" => Some(me),
        n => n.parse::<usize>().ok().filter(|&i| i < SQUAD).or_else(|| CHARACTERS[..SQUAD].iter().position(|c| *c == n)),
    }
}

/// The test hooks that act on the squad's members (play.rs, play_skills.rs, play_sentry.rs), and
/// the two deaths whose recipes look at the hand-over after them (BF_TEST_DIE, play.rs; X's
/// BF_TEST_SUICIDE, play_testmap.rs): a recipe with one was written for the squad of four, before
/// #302, so with one set the test map starts with the four (`start_roster`) and the recipe runs
/// as it did, its members at the places they had. (`BF_TEST_SQUAD=none` with them: a lone death,
/// `respawn`.)
const SQUAD_HOOKS: [&str; 10] = ["BF_TEST_SELECT", "BF_TEST_KILL", "BF_TEST_TARGET", "BF_TEST_SQUAD_SKILL", "BF_TEST_HOSTILE", "BF_TEST_MINE_FRIEND",
                                 "BF_TEST_DIE", "BF_TEST_SUICIDE", "BF_TEST_ORDER", "BF_TEST_COMMAND"];

/// The first of SQUAD_HOOKS that's set, if one is.
fn squad_hook() -> Option<&'static str> {
    SQUAD_HOOKS.into_iter().find(|k| std::env::var(k).is_ok())
}

/// The test map's start squad (#302): the player's character `leader` (BF_CHARACTER's, Tex by
/// default) in its own slot, and beside it
///  - BF_TEST_FULL_SQUAD=1: the other three, each in its own slot (the squad before #302);
///  - BF_TEST_SQUAD=<entry>[,<entry>...]: those, each a type as BF_TEST_NPC names one (a squad
///    name or 0-3, a display name, `name@archive`, a `<base name>`, `h_xxxxxxxx`): a squad
///    member's type in its own slot if that's free, any other in the first free one (in the keys'
///    order), or `<slot>:<type>` in that slot (as BF_TEST_SQUAD_SLOT names one: the leader's sets
///    its type); `none`: nobody;
///  - else with one of SQUAD_HOOKS set the other three too (`squad_hook`), else no one.
///
/// With why each slot is filled, for the log.
fn start_roster(game: &Game, leader: usize) -> (Roster, Vec<String>) {
    let mut r = [None; SQUAD];
    r[leader] = Some(squad_type(leader));
    let mut why = vec![format!("{} leads", CHARACTERS[leader])];
    let full = std::env::var("BF_TEST_FULL_SQUAD").is_ok_and(|v| v != "0");
    if full || (std::env::var("BF_TEST_SQUAD").is_err() && squad_hook().is_some()) {
        for (c, s) in r.iter_mut().enumerate() {
            *s = Some(squad_type(c));
        }
        why.push(format!("{}: the four", if full { "BF_TEST_FULL_SQUAD" } else { squad_hook().unwrap_or_default() }));
    } else if let Ok(list) = std::env::var("BF_TEST_SQUAD") {
        for entry in list.split(',').map(str::trim).filter(|e| !e.is_empty() && !e.eq_ignore_ascii_case("none")) {
            // (`<slot>:<type>`, when what's before the colon is a slot)
            let (slot, name) = match entry.split_once(':').and_then(|(s, n)| slot_named(s, leader).map(|c| (c, n))) {
                Some((c, n)) => (Some(c), n),
                None => (None, entry),
            };
            let Some(kind) = named_type(game, name) else {
                println!("test map: BF_TEST_SQUAD: no character type {name} with its model in the loaded data: skipped");
                continue;
            };
            let own = Some(squad_of(kind)).filter(|&c| c < SQUAD && r[c].is_none());
            match slot.or(own).or_else(|| (0..SQUAD).find(|&c| r[c].is_none())) {
                Some(c) => {
                    r[c] = Some(kind);
                    why.push(format!("BF_TEST_SQUAD {entry}: slot {c} {}", game.type_label(kind)));
                }
                None => println!("test map: BF_TEST_SQUAD: no free slot for {entry}: skipped"),
            }
        }
    }
    (Roster(r), why)
}

/// The test map's start (play.rs `begin_play`, with `player` and `squad` just made: no bodies
/// yet): its slots as `start_roster` has them, BF_TEST_TARGET's squadmate (the first member) then
/// put in the line of fire again, wherever the slots left it (play.rs `test_target`; with the
/// four, where it was).
pub(super) fn start(game: &Game, player: &mut Player, squad: &mut Squad, deathmatch: bool) {
    let (roster, why) = start_roster(game, player.character);
    apply(&roster, player, squad, deathmatch);
    if let Some(m) = squad.0.first_mut().filter(|m| m.in_squad()) {
        super::test_target(m);
    }
    if std::env::var("BF_TOOLS_LOG").is_ok() {
        let who: Vec<String> = roster.0.iter().enumerate().filter_map(|(c, k)| k.map(|k| format!("{c} {}", kind_name(k, game)))).collect();
        println!("test map: the squad at the start: {} ({})", who.join(", "), why.join("; "));
    }
}

/// Makes the squad `player` leads match `roster` (`change` for each slot that differs; one that
/// already holds its type is left as it was made, so the four give the start of before #302 to
/// the frame): one just made, with no bodies yet (the test map's start; the map editor's play,
/// play_editor.rs `to_play`).
pub(super) fn apply(roster: &Roster, player: &mut Player, squad: &mut Squad, deathmatch: bool) {
    for (c, &kind) in roster.0.iter().enumerate() {
        let now = if c == player.character { Some(player.kind) } else { squad.0.iter().find(|m| m.character == c && m.in_squad()).map(|m| m.kind) };
        if now == kind || (c == player.character && kind.is_none()) {
            continue;
        }
        let _ = change(c, kind, player, squad, deathmatch);
    }
}

/// What a slot's change did: the entities to take out of the world (the character's body and its
/// slide puff, as the NPC tool's removal does) and whether members moved in `Squad` (the shots on
/// their way are counted by place in it: `AiHits`, `Player::pending_hits`).
struct Change {
    gone: Vec<Entity>,
    moved: bool,
}

/// A fresh character of type `kind` for squad slot `c` in `old`'s place: on the floor where it
/// stood, facing its way, with its clock (the hooks' when it's the controlled one; `change` puts a
/// member on the leader's), its side, its
/// test hook hold, the name the level's triggers know it by and whether it's in the world yet; the
/// controlled one keeps its view (the aim, and the follow camera's state, so the camera eases to
/// the new one's boom), the help panel, the test hooks' marks and the squad's shared inventory
/// (its grenades refilled to the start's as its body is made, as at a respawn: play_inventory.rs
/// `stock`). The rest starts afresh: full health, its type's guns, no power-ups; `spawn_player`
/// makes its body.
fn fresh(c: usize, kind: u32, old: &Player, controlled: bool) -> Player {
    let mut p = Player::new(c);
    p.kind = kind;
    let feet = floor_y(old.position.x, old.position.z, old.position.y + GROUND + STEP_UP, world::Body::Character);
    p.position = Vec3::new(old.position.x, feet - GROUND, old.position.z);
    p.prev_xz = p.position.xz();
    p.fall_from = feet;
    p.yaw = old.yaw;
    p.cam_yaw = old.cam_yaw;
    p.cam_pitch = old.cam_pitch;
    p.sim_time = old.sim_time;
    p.team = old.team;
    p.test_hold = old.test_hold;
    p.trigger_name = old.trigger_name;
    p.absent = old.absent;
    p.held = old.held;
    p.surface = usize::MAX;
    if controlled {
        p.rig = old.rig.clone();
        p.show_help = old.show_help;
        (p.test_selected, p.test_scope_presses, p.test_medkit_used) = (old.test_selected, old.test_scope_presses, old.test_medkit_used);
        (p.grenades, p.medkits, p.medkit_kind, p.others, p.stocked, p.item) =
            (old.grenades.clone(), old.medkits, old.medkit_kind, old.others.clone(), old.stocked, old.item);
    }
    p
}

/// Squad slot `c` to type `kind` (None: emptied) in the squad `player` leads (see the module's
/// notes): the controlled character's slot changes its type in place (it can't be emptied); a
/// member's is replaced where it stands, or removed; an empty one gets a fresh member at its
/// formation place round the leader, after the squad's members and before the NPCs (`Squad`
/// keeps them first: the squad AI's formation places and the death camera's next member go by
/// that order). None in deathmatch (the map is played alone). What it did, or why not.
fn change(c: usize, kind: Option<u32>, player: &mut Player, squad: &mut Squad, deathmatch: bool) -> Result<Change, String> {
    let mut gone = vec![];
    let mut take = |u: &mut Player| {
        if let Some(l) = u.loaded.take() {
            gone.push(l.root);
        }
        if let Some(e) = u.slide_fx.take() {
            gone.push(e);
        }
    };
    if c == player.character {
        let Some(kind) = kind else { return Err("Your own slot can't be emptied: give it a type".into()) };
        let p = fresh(c, kind, player, true);
        take(player);
        *player = p;
        return Ok(Change { gone, moved: false });
    }
    let at = squad.0.iter().position(|m| m.character == c && m.in_squad());
    match (at, kind) {
        (Some(i), Some(kind)) => {
            let mut m = fresh(c, kind, &squad.0[i], false);
            // (on the leader's clock: a dead member's stopped at its death)
            m.sim_time = player.sim_time;
            take(&mut squad.0[i]);
            squad.0[i] = m;
            Ok(Change { gone, moved: false })
        }
        (Some(i), None) => {
            take(&mut squad.0[i]);
            squad.0.remove(i);
            Ok(Change { gone, moved: true })
        }
        (None, Some(_)) if deathmatch => Err("No squad here: the map is played alone".into()),
        (None, Some(kind)) => {
            let place = squad.0.iter().position(|m| !m.in_squad()).unwrap_or(squad.0.len());
            let mut m = Player::new(c);
            m.kind = kind;
            // (round the leader's feet, facing its way, its floor looked for 1.5 m above them)
            let feet = player.position + Vec3::Y * GROUND;
            super::place_in_formation(&mut m, Some((feet + Vec3::Y * 0.5, player.yaw)), place);
            m.yaw = player.yaw;
            m.cam_yaw = player.yaw;
            m.sim_time = player.sim_time;
            squad.0.insert(place, m);
            Ok(Change { gone, moved: place + 1 < squad.0.len() })
        }
        (None, None) => Ok(Change { gone, moved: false }),
    }
}

/// A type's name in the menu: the NPC menu's label (with where it's defined and its HP), else
/// its name.
fn label(kind: u32, types: &[(String, u32)], game: &Game) -> String {
    types.iter().find(|t| t.1 == kind).map_or_else(|| kind_name(kind, game), |t| t.0.clone())
}

/// A type's short name for a message or a squad's list of members: a squad type's capitalised
/// ("Tex"), any other's display name ("Hunter Lord"), without where it's defined or its HP.
fn short_name(kind: u32, game: &Game) -> String {
    match squad_of(kind) {
        OTHER => game.type_label(kind),
        c => title(CHARACTERS[c]),
    }
}

/// Makes the predefined squads once a map's data is in (`presets`); with BF_TOOLS_LOG it lists
/// them, each with who holds which place.
fn fill_presets(list: &mut Presets, game: &Game) {
    if !list.0.is_empty() {
        return;
    }
    list.0 = presets(game);
    if std::env::var("BF_TOOLS_LOG").is_ok() {
        println!("squad menu: {} predefined squads", list.0.len());
        for p in &list.0 {
            let who: Vec<String> = p.slots.iter().enumerate().map(|(c, k)| match k {
                Some(k) => format!("{c} {} h_{k:08x}", kind_name(*k, game)),
                None => format!("{c} empty"),
            }).collect();
            println!("  {} ({}): {}", p.name, p.place, who.join(", "));
        }
    }
}

/// The squad menu (T) and its hooks (see the module's notes), and its text. In play a change is
/// made at once (the bodies replaced go, the shots on their way are dropped when members move in
/// `Squad`), the roster with it; in edit mode the roster alone. A message says what changed or
/// why not; BF_TOOLS_LOG prints it. A press of V by a character with no skill says so (a type of
/// its own). A predefined squad (a hook, Enter or a click on one of its rows) is its four slots
/// put one after the other through the same steps as a single slot (#339), the message and the
/// log then saying the squad.
#[allow(clippy::too_many_arguments)]
fn squad_menu(mut commands: Commands, keys: Res<ButtonInput<KeyCode>>, scroll: Res<AccumulatedMouseScroll>, mut tools: ResMut<Tools>,
              script: Res<Script>, mut player: ResMut<Player>, mut squad: ResMut<Squad>, mut roster: ResMut<Roster>, mut menu: ResMut<Menu>,
              (mut hits, mut status, mode): (ResMut<AiHits>, ResMut<UsePanel>, Res<Mode>), game: Res<GameData>, mut npcs: ResMut<NpcList>,
              mut presets: ResMut<Presets>, mut text: ResMut<SquadText>) {
    fill_npc_list(&mut npcs, &game.0);
    fill_presets(&mut presets, &game.0);
    let (types, squads) = (&npcs.0, &presets.0);
    let log = std::env::var("BF_TOOLS_LOG").is_ok();
    let me = player.character;
    let mut asks: Vec<Ask> = vec![];
    for hook in &script.now {
        match hook {
            Hook::SquadMenu => tools.panel = if tools.panel == Panel::Squad { Panel::None } else { Panel::Squad },
            Hook::SquadSlot { slot, kind } => {
                let Some(c) = slot_named(slot, me) else {
                    println!("test tools: no squad slot {slot}");
                    continue;
                };
                match kind.as_deref().map(|k| (k, named_type(&game.0, k))) {
                    None => asks.push(Ask::Slot(c, None)),
                    Some((_, Some(kind))) => asks.push(Ask::Slot(c, Some(kind))),
                    Some((k, None)) => {
                        println!("test tools: no character type {k} with its model in the loaded data");
                        status.message = Some((format!("No character type {k}"), MESSAGE_TIME));
                    }
                }
            }
            Hook::SquadPreset(name) => match preset_named(squads, name) {
                Some(i) => asks.push(Ask::Squad(i)),
                None => {
                    println!("test tools: no predefined squad {name} (they are: {})", squads.iter().map(|p| p.name.as_str()).collect::<Vec<_>>().join(", "));
                    status.message = Some((format!("No predefined squad {name}"), MESSAGE_TIME));
                }
            },
            Hook::SquadPick { row, slot } => {
                // (a squad first, else a type)
                let at = preset_named(squads, row).map(Row::Squad)
                    .or_else(|| named_type(&game.0, row).and_then(|k| types.iter().position(|t| t.1 == k)).map(Row::Type));
                match at {
                    Some(at) => {
                        tools.panel = Panel::Squad;
                        menu.pick = at;
                    }
                    None => println!("test tools: no squad or character type {row} in the squad menu's list"),
                }
                if let Some(c) = slot.as_deref().and_then(|s| slot_named(s, me)) {
                    menu.slot = c;
                }
            }
            _ => {}
        }
    }
    let open = tools.panel == Panel::Squad;
    let rows = squads.len() + types.len();
    if open && rows > 0 {
        let step = |k: KeyCode| keys.just_pressed(k) as i32;
        let wheel = if scroll.delta.y < 0.0 { 1 } else if scroll.delta.y > 0.0 { -1 } else { 0 };
        let d = step(KeyCode::ArrowDown) - step(KeyCode::ArrowUp) + wheel + ROWS as i32 * (step(KeyCode::PageDown) - step(KeyCode::PageUp));
        let at = (position(menu.pick, squads.len()).min(rows - 1) as i32 + d).rem_euclid(rows as i32) as usize;
        menu.pick = row(at, squads.len());
        menu.slot = (menu.slot as i32 + step(KeyCode::ArrowRight) - step(KeyCode::ArrowLeft)).rem_euclid(SQUAD as i32) as usize;
        if keys.just_pressed(KeyCode::Enter) || keys.just_pressed(KeyCode::NumpadEnter) || tools.click {
            asks.push(match menu.pick {
                Row::Squad(i) => Ask::Squad(i),
                Row::Type(i) => Ask::Slot(menu.slot, Some(types[i].1)),
            });
        }
        // (Delete empties the slot picked, whatever row is)
        if keys.just_pressed(KeyCode::Delete) {
            asks.push(Ask::Slot(menu.slot, None));
        }
    }
    // (the window follows the pick, whichever of the keys and the hooks moved it)
    if rows > 0 {
        menu.top = window_top(menu.top, position(menu.pick, squads.len()).min(rows - 1), rows);
    }
    // each slot to change: the slot, its type (None: empty), and the predefined squad it's part of
    let mut puts: Vec<(usize, Option<u32>, Option<usize>)> = vec![];
    for ask in asks {
        match ask {
            Ask::Slot(c, kind) => puts.push((c, kind, None)),
            Ask::Squad(i) => puts.extend((0..SQUAD).map(|c| (c, squads[i].slots[c], Some(i)))),
        }
    }
    // each predefined squad put: its index, how many slots took, why the others didn't (the first
    // reason), and what you are if your slot did
    let mut told: Vec<(usize, usize, Option<String>, Option<String>)> = vec![];
    for (c, kind, via) in puts {
        if let Some(i) = via.filter(|&i| !told.iter().any(|t| t.0 == i)) {
            told.push((i, 0, None, None));
            if log {
                let p = &squads[i];
                println!("t {:.2}: squad {} ({}) in all four slots", player.sim_time, p.name, p.place);
            }
        }
        let place = format!("slot {} ({}'s place)", c + 1, title(CHARACTERS[c]));
        let was = roster.0[c].map_or("empty".to_string(), |k| label(k, types, &game.0));
        let now = kind.map_or("empty".to_string(), |k| label(k, types, &game.0));
        // edit mode: the squad is out of the world, the roster is what the next play makes (with
        // play's refusals: the leader's slot not emptied, no member on a map played alone)
        let done = if tools.edit {
            if c == player.character && kind.is_none() {
                Err("The leader's slot can't be emptied: give it a type".to_string())
            } else if c != player.character && kind.is_some() && mode.deathmatch {
                Err("No squad here: the map is played alone".to_string())
            } else {
                Ok(())
            }
        } else {
            change(c, kind, &mut player, &mut squad, mode.deathmatch).map(|ch| {
                for e in ch.gone {
                    commands.entity(e).despawn();
                }
                if ch.moved {
                    hits.0.clear();
                    player.pending_hits.clear();
                }
            })
        };
        match done {
            Ok(()) => {
                roster.0[c] = kind;
                let you = kind.filter(|_| c == player.character && !tools.edit);
                let what = match kind {
                    Some(_) if you.is_some() => format!("You're {now} now"),
                    Some(_) => format!("Slot {}: {now}", c + 1),
                    None => format!("Slot {} emptied", c + 1),
                };
                match via.and_then(|i| told.iter_mut().find(|t| t.0 == i)) {
                    // (a predefined squad says itself once it's all put, below)
                    Some(t) => {
                        t.1 += 1;
                        if let Some(k) = you {
                            t.3 = Some(short_name(k, &game.0));
                        }
                    }
                    None => status.message = Some((if tools.edit { format!("{what} (when play starts)") } else { what }, MESSAGE_TIME)),
                }
                if log {
                    let kind = kind.map_or(String::new(), |k| format!(" (h_{k:08x})"));
                    // (where its feet are, in play)
                    let at = std::iter::once(&*player).chain(squad.0.iter()).find(|u| u.character == c && u.in_squad() && !tools.edit)
                        .map_or(String::new(), |u| format!(" at {:.2}", u.position + Vec3::Y * GROUND));
                    println!("t {:.2}: squad {place}: {now}{kind}{at}, was {was}{}", player.sim_time, if tools.edit { " (the roster, for play)" } else { "" });
                }
            }
            Err(why) => {
                match via.and_then(|i| told.iter_mut().find(|t| t.0 == i)) {
                    Some(t) => {
                        t.2.get_or_insert(why.clone());
                    }
                    None => status.message = Some((why.clone(), MESSAGE_TIME)),
                }
                if log {
                    println!("t {:.2}: squad {place}: {now} refused: {why}", player.sim_time);
                }
            }
        }
    }
    // a predefined squad put: what it did, in one message (the last one asked, if several)
    for (i, took, why, you) in told {
        let name = &squads[i].name;
        let mut what = format!("Squad: {name}");
        // (a map played alone's reason as "played alone": the message is long enough, and the panel says the rest)
        let notes: Vec<String> = [you.map(|y| format!("you're {y} now")),
                                  why.map(|w| format!("{} not put: {}", SQUAD - took, w.strip_prefix("No squad here: the map is ").unwrap_or(&w)))]
            .into_iter().flatten().collect();
        if !notes.is_empty() {
            what += &format!(" ({})", notes.join("; "));
        }
        if tools.edit {
            what += " (when play starts)";
        }
        if log {
            println!("t {:.2}: squad {name}: {took} of {SQUAD} slots put{}", player.sim_time, if tools.edit { " (the roster, for play)" } else { "" });
        }
        status.message = Some((what, MESSAGE_TIME));
    }
    // (V with no skill: a type of its own has none, the menu's doing)
    if keys.just_pressed(KeyCode::KeyV) && player.skill.data.is_none() && !player.dead && !player.stand_in {
        status.message = Some((format!("{} has no special skill", display_name(&player, &game.0)), MESSAGE_TIME));
    }
    let label_of = |k: u32| label(k, types, &game.0);
    let short_of = |k: u32| short_name(k, &game.0);
    let s = if open {
        panel_text(&menu, types, squads, &roster, &player, &squad, (&label_of, &short_of), tools.edit, mode.deathmatch)
    } else {
        String::new()
    };
    if text.0 != s {
        text.0 = s;
    }
}

/// A predefined squad's row in the menu's list: its name and where its types are defined.
fn squad_label(p: &Preset) -> String {
    format!("{} ({})", p.name, p.place)
}

/// The squad menu's panel: the four slots (the one picked marked), who holds each (dead, or the
/// controlled character's), what a type of its own lacks, a window of ROWS rows of the list (the
/// predefined squads, then the types, a heading each) and, for the row picked, the keys or, for a
/// squad, who it would put in each place. `names` are the type's long label (with where it's
/// defined and its HP: a slot's and the types' rows) and short name (a squad's members).
///
/// It has to end above the HUD's item box (see `PANEL_LINES` in the tests) in every state, the
/// worst being four slots of long names with a squad picked: so the type-of-its-own note is a
/// line, a squad's members take two lines (two places a line), and the list shows ROWS rows.
#[allow(clippy::too_many_arguments)]
fn panel_text(menu: &Menu, types: &[(String, u32)], squads: &[Preset], roster: &Roster, player: &Player, squad: &Squad,
              names: (&dyn Fn(u32) -> String, &dyn Fn(u32) -> String), edit: bool, deathmatch: bool) -> String {
    let (label_of, short_of) = names;
    let mut s = String::from("SQUAD   (T / Esc: close)\nslot (Left / Right):\n");
    let mut own = false;
    for c in 0..SQUAD {
        let mark = if c == menu.slot { ">" } else { " " };
        let who = match roster.0[c] {
            None => "(empty)".to_string(),
            Some(k) => {
                own |= squad_of(k) == OTHER;
                let dead = !edit && std::iter::once(player).chain(squad.0.iter()).any(|u| u.character == c && u.in_squad() && u.dead);
                let you = if c == player.character { if edit { "   (leads at F5)" } else { "   (you)" } } else { "" };
                format!("{}{}{you}", label_of(k), if dead { "   DEAD" } else { "" })
            }
        };
        s += &format!("{mark} {}  {}'s place: {who}\n", c + 1, title(CHARACTERS[c]));
    }
    if own {
        s += "  (a type of its own: no skill, a death line only)\n";
    }
    if deathmatch {
        s += "  (played alone here: only your slot changes)\n";
    }
    s += "squad or type (Up / Down, wheel, Page Up / Down):\n";
    let n = squads.len() + types.len();
    let (at, first) = (position(menu.pick, squads.len()), menu.top.min(n.saturating_sub(ROWS)));
    let mut group = "";
    for i in first..(first + ROWS).min(n) {
        let (heading, label) = match row(i, squads.len()) {
            Row::Squad(k) => ("predefined squads", squad_label(&squads[k])),
            Row::Type(k) => ("character types", types[k].0.clone()),
        };
        if heading != group {
            group = heading;
            s += &format!("  [{heading}]\n");
        }
        s += &format!("  {} {label}\n", if i == at { ">" } else { " " });
    }
    let slot = menu.slot + 1;
    match menu.pick {
        Row::Squad(k) => {
            let p = &squads[k];
            s += &format!("click / Enter: {} in all four slots:\n", p.name);
            // who goes in each place, two a line in the order of the slot rows
            for pair in [[0, 1], [2, 3]] {
                let line: Vec<String> = pair.iter().map(|&c| {
                    let who = p.slots[c].map_or("(empty)".to_string(), short_of);
                    let you = if c == player.character { if edit { " (leads)" } else { " (you)" } } else { "" };
                    format!("{}: {who}{you}", title(CHARACTERS[c]))
                }).collect();
                s += &format!("  {}\n", line.join("   "));
            }
        }
        Row::Type(k) => {
            s += &format!("    ({} of {} character types)\n", k + 1, types.len());
            s += &if menu.slot == player.character {
                format!("click / Enter: {} it (slot {slot})", if edit { "the leader becomes" } else { "you become" })
            } else {
                format!("click / Enter: put it in slot {slot}   Delete: empty slot {slot}")
            };
        }
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A type of none of the squad's four (m09_a's Shrike Soldier's key).
    const SHRIKE: u32 = 0xFCEC_1024;

    /// Tex leading at (0, 8) facing -z, with the squad `members` (slots) at the flat floor's
    /// formation, then an NPC Brutus.
    fn squad(members: &[usize]) -> (Player, Squad) {
        let mut tex = Player::new(3);
        tex.position = Vec3::new(0.0, 0.0, 8.0);
        tex.sim_time = 4.0;
        let mut list: Vec<Player> = members.iter().enumerate().map(|(i, &c)| {
            let mut m = Player::new(c);
            m.position = Vec3::new(i as f32, 0.0, 0.0);
            m
        }).collect();
        let mut npc = Player::new(0);
        npc.npc = Some(super::super::testtools::Npc { id: 1, fight: false, face: 0.0 });
        list.push(npc);
        (tex, Squad(list))
    }

    #[test]
    fn slots_by_name_number_or_me() {
        assert_eq!(slot_named("0", 3), Some(0));
        assert_eq!(slot_named("Hawk", 3), Some(2));
        assert_eq!(slot_named("me", 1), Some(1));
        assert_eq!(slot_named("4", 3), None);
        assert_eq!(slot_named("shrike", 3), None);
    }

    #[test]
    fn the_roster_is_the_squads_slots_not_its_npcs() {
        let (tex, squad) = squad(&[1]);
        let r = Roster::of(&tex, &squad);
        assert_eq!(r.0, [None, Some(squad_type(1)), None, Some(squad_type(3))]);
    }

    #[test]
    fn the_controlled_slot_changes_type_in_place_and_cant_be_emptied() {
        let (mut tex, mut squad) = squad(&[]);
        tex.grenades = vec![3, 4];
        assert!(change(3, None, &mut tex, &mut squad, false).is_err());
        let ch = change(3, Some(SHRIKE), &mut tex, &mut squad, false).expect("changed");
        assert!(!ch.moved);
        assert_eq!((tex.character, tex.kind), (3, SHRIKE));
        assert!(tex.own_type());
        // (where it stood, on the clock it had, with the squad's grenades)
        assert_eq!((tex.position.x, tex.position.z, tex.sim_time), (0.0, 8.0, 4.0));
        assert_eq!(tex.grenades, vec![3, 4]);
    }

    #[test]
    fn a_member_is_replaced_where_it_stands_or_removed() {
        let (mut tex, mut squad) = squad(&[0, 2]);
        let ch = change(2, Some(SHRIKE), &mut tex, &mut squad, false).expect("replaced");
        assert!(!ch.moved);
        let m = &squad.0[1];
        assert_eq!((m.character, m.kind, m.position.x, m.sim_time), (2, SHRIKE, 1.0, 4.0));
        assert!(m.in_squad());
        let ch = change(0, None, &mut tex, &mut squad, false).expect("emptied");
        assert!(ch.moved);
        assert_eq!(squad.0.len(), 2);
        assert_eq!(squad.0[0].character, 2);
        assert!(squad.0[1].npc.is_some());
    }

    /// A predefined squad of made-up types, for the name tests.
    fn preset(name: &str, library: &str) -> Preset {
        Preset { name: name.into(), place: format!("mp_common/{library}"), library: library.into(), slots: [Some(1), Some(2), Some(3), Some(4)] }
    }

    #[test]
    fn a_squads_members_sit_in_the_places_their_hud_slots_name() {
        // sdm_bf in game-options' order (Tex, Brutus, Hawk, Flint: h_072ab4af 1, 3, 4, 2): Brutus's
        // place (bottom, slot 3) is the second member's, Tex's (top, slot 1) the first's
        assert_eq!(seat(&[(10, 1), (11, 3), (12, 4), (13, 2)]), [Some(11), Some(13), Some(12), Some(10)]);
        // sdm_seers (hound 4, priest 2, follower 3, shadoon 1): the order of the list doesn't matter
        assert_eq!(seat(&[(20, 4), (21, 2), (22, 3), (23, 1)]), [Some(22), Some(21), Some(20), Some(23)]);
        // a squad of the data's four has the four slots once each: every place filled
        assert!(seat(&[(1, 1), (2, 2), (3, 3), (4, 4)]).iter().all(Option::is_some));
    }

    #[test]
    fn a_member_with_no_slot_of_its_own_takes_the_first_free_place() {
        // slot 1 twice, 0 (no slot) and 7: the first takes Tex's place, the others the free ones
        // in the keys' order (Brutus's, Flint's, Hawk's)
        assert_eq!(seat(&[(10, 1), (11, 1), (12, 0), (13, 7)]), [Some(11), Some(12), Some(13), Some(10)]);
        // fewer than four leaves the rest empty
        assert_eq!(seat(&[(10, 2)]), [None, Some(10), None, None]);
    }

    #[test]
    fn a_hook_names_a_squad_by_number_name_library_or_a_word() {
        let list = [preset("Campaign squad", "common"), preset("Brute Force", "sdm_bf"), preset("Shrike Raiders", "sdm_shrike")];
        assert_eq!(preset_named(&list, "2"), Some(2));
        assert_eq!(preset_named(&list, "3"), None);
        assert_eq!(preset_named(&list, "shrike raiders"), Some(2));
        assert_eq!(preset_named(&list, " SDM_BF "), Some(1));
        assert_eq!(preset_named(&list, "shrike"), Some(2));
        assert_eq!(preset_named(&list, "common"), Some(0));
        assert_eq!(preset_named(&list, "campaign"), Some(0));
        assert_eq!(preset_named(&list, "raiders"), Some(2));
        assert_eq!(preset_named(&list, "seers"), None);
        // (nothing named is no squad, not the first)
        assert_eq!(preset_named(&list, " "), None);
    }

    #[test]
    fn the_list_has_the_squads_before_the_types() {
        assert_eq!(row(0, 6), Row::Squad(0));
        assert_eq!(row(5, 6), Row::Squad(5));
        assert_eq!(row(6, 6), Row::Type(0));
        assert_eq!(row(7, 0), Row::Type(7));
    }

    #[test]
    fn a_squad_put_slot_by_slot_fills_the_four_places_round_the_leader() {
        // Tex alone (slot 3) takes a squad of four others: the leader changes in place, the other
        // three appear in the keys' order at the formation places, before the NPC
        let (mut tex, mut squad) = squad(&[]);
        let kinds = [0xA000_0001, 0xA000_0002, 0xA000_0003, SHRIKE];
        for (c, kind) in kinds.into_iter().enumerate() {
            change(c, Some(kind), &mut tex, &mut squad, false).expect("put");
        }
        assert_eq!((tex.character, tex.kind, tex.position.x, tex.position.z), (3, SHRIKE, 0.0, 8.0));
        assert_eq!(squad.0.iter().map(|m| (m.character, m.kind, m.npc.is_some())).collect::<Vec<_>>(),
                   vec![(0, 0xA000_0001, false), (1, 0xA000_0002, false), (2, 0xA000_0003, false), (0, squad_type(0), true)]);
        assert_eq!(Roster::of(&tex, &squad).0, kinds.map(Some));
        // (in deathmatch the three are refused and the leader's change stands)
        let (mut tex, mut squad) = squad_alone();
        assert!(change(0, Some(kinds[0]), &mut tex, &mut squad, true).is_err());
        assert!(change(3, Some(kinds[3]), &mut tex, &mut squad, true).is_ok());
        assert_eq!(squad.0.iter().filter(|m| m.in_squad()).count(), 0);
    }

    /// Tex alone, no NPC.
    fn squad_alone() -> (Player, Squad) {
        (Player::new(3), Squad(vec![]))
    }

    #[test]
    fn an_empty_slot_gets_a_member_before_the_npcs() {
        let (mut tex, mut squad) = squad(&[1]);
        let ch = change(0, Some(SHRIKE), &mut tex, &mut squad, false).expect("filled");
        // (the NPC after it moved down a place)
        assert!(ch.moved);
        assert_eq!(squad.0.iter().map(|m| (m.character, m.npc.is_some())).collect::<Vec<_>>(), vec![(1, false), (0, false), (0, true)]);
        let m = &squad.0[1];
        assert_eq!((m.kind, m.sim_time), (SHRIKE, 4.0));
        // (at its formation place round Tex: FORMATION[1] at half spacing, ahead of him)
        assert!((m.position.x - 1.75).abs() < 1e-4 && (m.position.z - 4.0).abs() < 1e-4, "{:?}", m.position);
        // (none in deathmatch, and an empty slot emptied changes nothing)
        assert!(change(2, Some(SHRIKE), &mut tex, &mut squad, true).is_err());
        assert!(!change(2, None, &mut tex, &mut squad, false).expect("nothing").moved);
    }

    #[test]
    fn a_roster_makes_a_fresh_squad_match_it() {
        let (mut tex, mut squad) = squad(&[0, 1, 2]);
        apply(&Roster([None, Some(SHRIKE), Some(squad_type(2)), Some(squad_type(1))]), &mut tex, &mut squad, false);
        assert_eq!(tex.kind, squad_type(1));
        assert_eq!(squad.0.iter().filter(|m| m.in_squad()).map(|m| (m.character, m.kind)).collect::<Vec<_>>(),
                   vec![(1, SHRIKE), (2, squad_type(2))]);
    }

    #[test]
    fn the_menu_opens_on_the_first_type_with_the_squads_in_view() {
        // (the row it opened on before the squads were added, so Enter straight after T changes
        // one slot, not four: whatever number of squads the data gives)
        let m = Menu::default();
        assert_eq!(m.pick, Row::Type(0));
        for squads in [0, 1, 6] {
            assert_eq!(row(position(m.pick, squads), squads), Row::Type(0));
        }
        assert_eq!(position(m.pick, 6), 6);
        // (and its window starts at the top of the list: the six squads and that type at its foot)
        assert_eq!(window_top(m.top, position(m.pick, 6), 6 + 128), 0);
        assert_eq!(position(Row::Squad(3), 6), 3);
    }

    #[test]
    fn the_window_scrolls_a_row_at_a_time_past_its_edges() {
        let rows = 6 + 128;
        // inside it stays where it was, past an edge it moves just far enough
        assert_eq!(window_top(0, ROWS - 1, rows), 0);
        assert_eq!(window_top(0, ROWS, rows), 1);
        assert_eq!(window_top(3, 5, rows), 3);
        assert_eq!(window_top(3, 3 + ROWS - 1, rows), 3);
        assert_eq!(window_top(3, 3 + ROWS, rows), 4);
        assert_eq!(window_top(3, 2, rows), 2);
        // wrapping round either end, and a list shorter than the window
        assert_eq!(window_top(0, rows - 1, rows), rows - ROWS);
        assert_eq!(window_top(rows - ROWS, 0, rows), 0);
        assert_eq!(window_top(5, 2, 4), 0);
    }

    /// How many lines the panel may take once wrapped. It starts 18% down the window (130 px of 720,
    /// play_testtools.rs `spawn_panel`) with 8 px of padding, its 13-point lines are 15.6 px (31
    /// lines made it end at y 630 in the tester's capture), and it has to end above the HUD's item
    /// box, whose top is at y 565 of 720 at 1280 x 720 (about 555 with a margin): 26 lines.
    const PANEL_LINES: usize = 26;

    /// How many characters of a line fit before the panel wraps it: at most 36% of the window's
    /// width (461 px at 1280) less 8 px of padding each side, at the font's advance; the captures'
    /// longest first line before a wrap, "> 1  Brutus's place: Shrike Soldier (mp_common/sdm_shrike,",
    /// holds 58.
    const PANEL_COLUMNS: usize = 58;

    /// The lines `text` takes in the panel: a greedy word wrap at PANEL_COLUMNS (the panel breaks
    /// lines at spaces).
    fn wrapped_lines(text: &str) -> usize {
        text.trim_end().split('\n').map(|line| {
            let (mut rows, mut width) = (1, 0);
            for (i, word) in line.split(' ').enumerate() {
                let (len, gap) = (word.chars().count(), usize::from(i > 0));
                if width > 0 && width + gap + len > PANEL_COLUMNS {
                    rows += 1;
                    width = len;
                } else {
                    width += gap + len;
                }
            }
            rows
        }).sum()
    }

    /// The six predefined squads as the data gives them (testtools.md "The predefined squads"): the
    /// name, the library and the four places' members with their hit points, in the order of the
    /// slots (Brutus's, Flint's, Hawk's, Tex's).
    const DATA: [(&str, &str, [(&str, u32); 4]); 6] = [
        ("Campaign squad", "common", [("Brutus", 75), ("Flint", 55), ("Hawk", 50), ("Tex", 70)]),
        ("Brute Force", "sdm_bf", [("Brutus", 75), ("Flint", 55), ("Hawk", 50), ("Tex", 70)]),
        ("Feral Hunting Pack", "sdm_feral", [("Feral Shaman", 70), ("Feral Outcast", 75), ("Outcast Shaman", 70), ("Feral Colonist", 75)]),
        ("Shrike Raiders", "sdm_shrike", [("Shrike Soldier", 50), ("Shrike Heavy", 55), ("Shrike Hound", 45), ("Hunter Lord", 80)]),
        ("Seer Coven", "sdm_seers", [("Seer Follower", 75), ("Seer Priest", 85), ("Fire Hound", 45), ("Shadoon", 95)]),
        ("Ferguson Platoon", "sdm_ferg", [("Confed Marine", 80), ("McTavish", 80), ("Gunthar Ghent", 80), ("Militia", 80)]),
    ];

    /// The type of squad `s`'s member at place `c`, made up (none is one of the campaign four, so the
    /// panel's note about a type of its own shows: the longer case).
    fn kind_of(s: usize, c: usize) -> u32 {
        0xB000_0000 | (s as u32) << 4 | c as u32
    }

    /// A made-up type's NPC menu label ("Shrike Soldier (mp_common/sdm_shrike, 50 hp)") and short name.
    fn long_label(k: u32) -> String {
        let (s, c) = (((k >> 4) & 0xF) as usize, (k & 0xF) as usize);
        format!("{} (mp_common/{}, {} hp)", DATA[s].2[c].0, DATA[s].1, DATA[s].2[c].1)
    }

    fn short_label(k: u32) -> String {
        DATA[((k >> 4) & 0xF) as usize].2[(k & 0xF) as usize].0.to_string()
    }

    #[test]
    fn the_panel_ends_above_the_item_box_in_every_state() {
        let squads: Vec<Preset> = DATA.iter().enumerate().map(|(s, d)| Preset {
            name: d.0.into(), place: format!("mp_common/{}", d.1), library: d.1.into(), slots: std::array::from_fn(|c| Some(kind_of(s, c))),
        }).collect();
        // the types: the four of the squad, then 124 of the longest labels (the menu lists 128 on the test map)
        let types: Vec<(String, u32)> = (0..128).map(|i| (long_label(kind_of(2 + i % 4, i % 4)) + if i % 3 == 0 { " h_0571d345" } else { "" }, 0xC000_0000 + i as u32)).collect();
        let (mut worst, mut states) = (0, 0);
        for edit in [false, true] {
            for held in 0..squads.len() {
                // every squad's four in the slots, any of the four the player's, any slot picked
                for me in 0..SQUAD {
                    for dead in [false, true] {
                        let mut player = Player::new(me);
                        player.dead = dead;
                        let roster = Roster(squads[held].slots);
                        let others = Squad(vec![]);
                        let picks = (0..squads.len()).map(Row::Squad).chain([Row::Type(0), Row::Type(5), Row::Type(127)]);
                        for pick in picks {
                            for slot in [0, me] {
                                let menu = Menu { slot, pick, top: window_top(0, position(pick, squads.len()), squads.len() + types.len()) };
                                let text = panel_text(&menu, &types, &squads, &roster, &player, &others, (&long_label, &short_label), edit, false);
                                let lines = wrapped_lines(&text);
                                assert!(lines <= PANEL_LINES, "{lines} lines (edit {edit}, {} held, you {me}, {pick:?}):\n{text}", squads[held].name);
                                (worst, states) = (worst.max(lines), states + 1);
                            }
                        }
                    }
                }
            }
        }
        // (a map played alone: the roster is the player's slot alone)
        let alone = Roster([None, None, None, Some(kind_of(2, 3))]);
        for pick in (0..squads.len()).map(Row::Squad) {
            let menu = Menu { slot: 3, pick, top: 0 };
            let text = panel_text(&menu, &types, &squads, &alone, &Player::new(3), &Squad(vec![]), (&long_label, &short_label), false, true);
            assert!(wrapped_lines(&text) <= PANEL_LINES, "{text}");
        }
        // (the fixture reaches long panels: its long names wrap the slot rows)
        println!("the worst of {states} states takes {worst} of {PANEL_LINES} lines");
        assert!(worst >= 20, "the worst of {states} states takes {worst} lines: the fixture isn't the worst case");
    }
}
