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
//!    test session): Left / Right pick a slot, Up / Down (the wheel; Page Up / Down a page) a type,
//!    Enter or a click puts a fresh one of that type in the slot (`change`: a member of another
//!    type replaced where it stands, an empty slot's at its formation place round the leader, the
//!    controlled character's changed in place: its position, heading, view and the squad's
//!    inventory kept), Delete empties it (not the controlled character's). In the map editor's
//!    edit mode, with the squad out of the world, it changes the roster the next play makes.
//!  - **The roster** (`Roster`): each map's slots, as its start made them and the menu changed
//!    them; the map editor's play (F5) makes its fresh squad from it (play_editor.rs `to_play`).
//!
//! Test hooks: BF_TEST_SQUAD, BF_TEST_FULL_SQUAD (the test map's start: `start_roster`);
//! BF_TEST_SQUAD_MENU=<s> (T) and BF_TEST_SQUAD_SLOT=<s>,<slot>,<type|empty>[;...] (the menu's
//! Enter or Delete), timed as the tools' hooks (play_testtools.rs `read_hooks`). BF_TOOLS_LOG=1
//! prints the start squad and each change.

use super::*;
use super::testtools::{fill_npc_list, npc_kind, title, Hook, NpcList, Panel, Script, Tools, LIST_LINES, MESSAGE_TIME};

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

/// The squad menu's picks: the slot (a CHARACTERS index) and the type (an index into the NPC
/// menu's list, `NpcList`). A map's: reset as each one starts.
#[derive(Resource, Default)]
struct Menu {
    slot: usize,
    pick: usize,
}

/// The squad menu's text, shown by play_testtools.rs's panel (`show_panel`).
#[derive(Resource, Default)]
pub(super) struct SquadText(pub(super) String);

/// The squad menu's systems: its state anew with each map, and the menu (after `read_input`,
/// before control passes: `squad_control`).
pub fn plugin(app: &mut App) {
    app.init_resource::<Menu>().init_resource::<SquadText>().init_resource::<Roster>()
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

/// A new map: the menu's picks anew (its slots are the map's: `begin_play` made its roster).
fn reset(mut menu: ResMut<Menu>, mut text: ResMut<SquadText>) {
    *menu = Menu::default();
    text.0.clear();
}

/// The character type a hook names (as BF_TEST_NPC names one: play_testtools.rs `npc_kind`), if
/// its model is loaded too (one without would have no body: `spawn_unit` can't make it; the NPC
/// menu lists only those).
fn named_type(game: &Game, s: &str) -> Option<u32> {
    npc_kind(game, s).filter(|&k| game.character_type(k).is_some_and(|t| game.has_model(t)))
}

/// The squad slot a hook names: 0-3, or brutus / flint / hawk / tex (its place, as BF_TEST_SELECT
/// and BF_TEST_HOSTILE name a squad member), or `me` (the controlled character's, `me`).
fn slot_named(s: &str, me: usize) -> Option<usize> {
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
const SQUAD_HOOKS: [&str; 8] = ["BF_TEST_SELECT", "BF_TEST_KILL", "BF_TEST_TARGET", "BF_TEST_SQUAD_SKILL", "BF_TEST_HOSTILE", "BF_TEST_MINE_FRIEND",
                                "BF_TEST_DIE", "BF_TEST_SUICIDE"];

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

/// The squad menu (T) and its hooks (see the module's notes), and its text. In play a change is
/// made at once (the bodies replaced go, the shots on their way are dropped when members move in
/// `Squad`), the roster with it; in edit mode the roster alone. A message says what changed or
/// why not; BF_TOOLS_LOG prints it. A press of V by a character with no skill says so (a type of
/// its own).
#[allow(clippy::too_many_arguments)]
fn squad_menu(mut commands: Commands, keys: Res<ButtonInput<KeyCode>>, scroll: Res<AccumulatedMouseScroll>, mut tools: ResMut<Tools>,
              script: Res<Script>, mut player: ResMut<Player>, mut squad: ResMut<Squad>, mut roster: ResMut<Roster>, mut menu: ResMut<Menu>,
              (mut hits, mut status, mode): (ResMut<AiHits>, ResMut<UsePanel>, Res<Mode>), game: Res<GameData>, mut npcs: ResMut<NpcList>,
              mut text: ResMut<SquadText>) {
    fill_npc_list(&mut npcs, &game.0);
    let types = &npcs.0;
    let log = std::env::var("BF_TOOLS_LOG").is_ok();
    let me = player.character;
    // (slot, its type or None, how it was asked for)
    let mut asks: Vec<(usize, Option<u32>)> = vec![];
    for hook in &script.now {
        match hook {
            Hook::SquadMenu => tools.panel = if tools.panel == Panel::Squad { Panel::None } else { Panel::Squad },
            Hook::SquadSlot { slot, kind } => {
                let Some(c) = slot_named(slot, me) else {
                    println!("test tools: no squad slot {slot}");
                    continue;
                };
                match kind.as_deref().map(|k| (k, named_type(&game.0, k))) {
                    None => asks.push((c, None)),
                    Some((_, Some(kind))) => asks.push((c, Some(kind))),
                    Some((k, None)) => {
                        println!("test tools: no character type {k} with its model in the loaded data");
                        status.message = Some((format!("No character type {k}"), MESSAGE_TIME));
                    }
                }
            }
            _ => {}
        }
    }
    let open = tools.panel == Panel::Squad;
    if open && !types.is_empty() {
        let step = |k: KeyCode| keys.just_pressed(k) as i32;
        let wheel = if scroll.delta.y < 0.0 { 1 } else if scroll.delta.y > 0.0 { -1 } else { 0 };
        let d = step(KeyCode::ArrowDown) - step(KeyCode::ArrowUp) + wheel + LIST_LINES as i32 * (step(KeyCode::PageDown) - step(KeyCode::PageUp));
        menu.pick = (menu.pick as i32 + d).rem_euclid(types.len() as i32) as usize;
        menu.slot = (menu.slot as i32 + step(KeyCode::ArrowRight) - step(KeyCode::ArrowLeft)).rem_euclid(SQUAD as i32) as usize;
        if keys.just_pressed(KeyCode::Enter) || keys.just_pressed(KeyCode::NumpadEnter) || tools.click {
            asks.push((menu.slot, Some(types[menu.pick].1)));
        }
        if keys.just_pressed(KeyCode::Delete) {
            asks.push((menu.slot, None));
        }
    }
    for (c, kind) in asks {
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
                let what = match kind {
                    Some(_) if c == player.character && !tools.edit => format!("You're {now} now"),
                    Some(_) => format!("Slot {}: {now}", c + 1),
                    None => format!("Slot {} emptied", c + 1),
                };
                status.message = Some((if tools.edit { format!("{what} (when play starts)") } else { what }, MESSAGE_TIME));
                if log {
                    let kind = kind.map_or(String::new(), |k| format!(" (h_{k:08x})"));
                    // (where its feet are, in play)
                    let at = std::iter::once(&*player).chain(squad.0.iter()).find(|u| u.character == c && u.in_squad() && !tools.edit)
                        .map_or(String::new(), |u| format!(" at {:.2}", u.position + Vec3::Y * GROUND));
                    println!("t {:.2}: squad {place}: {now}{kind}{at}, was {was}{}", player.sim_time, if tools.edit { " (the roster, for play)" } else { "" });
                }
            }
            Err(why) => {
                status.message = Some((why.clone(), MESSAGE_TIME));
                if log {
                    println!("t {:.2}: squad {place}: {now} refused: {why}", player.sim_time);
                }
            }
        }
    }
    // (V with no skill: a type of its own has none, the menu's doing)
    if keys.just_pressed(KeyCode::KeyV) && player.skill.data.is_none() && !player.dead && !player.stand_in {
        status.message = Some((format!("{} has no special skill", display_name(&player, &game.0)), MESSAGE_TIME));
    }
    let s = if open { panel_text(&menu, types, &roster, &player, &squad, &game.0, tools.edit, mode.deathmatch) } else { String::new() };
    if text.0 != s {
        text.0 = s;
    }
}

/// The squad menu's panel: the four slots (the one picked marked), who holds each (dead, or the
/// controlled character's), what a type of its own lacks, a window of the types round the one
/// picked, and the keys.
#[allow(clippy::too_many_arguments)]
fn panel_text(menu: &Menu, types: &[(String, u32)], roster: &Roster, player: &Player, squad: &Squad, game: &Game, edit: bool, deathmatch: bool) -> String {
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
                format!("{}{}{you}", label(k, types, game), if dead { "   DEAD" } else { "" })
            }
        };
        s += &format!("{mark} {}  {}'s place: {who}\n", c + 1, title(CHARACTERS[c]));
    }
    if own {
        s += "  (a type not of the squad: no skill, no voice but its death line,\n   its type's portrait or ?)\n";
    }
    if deathmatch {
        s += "  (played alone here: no squad to fill)\n";
    }
    s += "type (Up / Down, wheel, Page Up / Down):\n";
    let n = types.len();
    let first = menu.pick.saturating_sub(LIST_LINES / 2).min(n.saturating_sub(LIST_LINES));
    for (i, t) in types.iter().enumerate().take((first + LIST_LINES).min(n)).skip(first) {
        s += &format!("  {} {}\n", if i == menu.pick { ">" } else { " " }, t.0);
    }
    s += &format!("    ({} of {n} character types)\n", menu.pick + 1);
    let slot = menu.slot + 1;
    s += &if menu.slot == player.character {
        format!("click / Enter: {} it (slot {slot})", if edit { "the leader becomes" } else { "you become" })
    } else {
        format!("click / Enter: put it in slot {slot}   Delete: empty slot {slot}")
    };
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
}
