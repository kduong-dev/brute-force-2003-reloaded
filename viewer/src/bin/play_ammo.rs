//! Ammo (#114): the squad's shared reserve, the automatic reload, dry fire, and the recharging
//! clips of the LZR family.
//!
//!  - **Reserve.** The rounds a gun reloads from are the squad's inventory, one count per
//!    ammo-type, not the gun's: default.xbe's FUN_0022da80 looks the weapon's ammo-type up in the
//!    holder's inventory (FUN_0014cc30(1, type)). So the Foley and the L-Shot share 11mm Ammo
//!    (type 1), the MK-ASLT and the Minigun High ROF Ammo (type 2). The counts start at the stack
//!    limit of the level's ammo item of that type (objecttypes `<inventory>` entries with
//!    function-type 14 = IFSET_AMMO_BOX, whose h_e5f51266 is the ammo-type; see `stack_limits`).
//!  - **Automatic reload.** The game has no reload button. When the fire loop (FUN_0022f2a0)
//!    finds the clip empty as the cooldown runs out with the trigger held, FUN_002327f0 calls the
//!    character's FUN_00120d40, which moves the rounds out of the reserve (FUN_0022daf0) and
//!    starts the reload motion; letting go of the trigger on an empty clip starts it too
//!    (measured: the L-Shot's reserve drops 0.07 s after the release, todo/61 take01 and take03).
//!    play.rs's `step_player` plays the reload clip and fills the clip at its magazine-in event.
//!    R reloads early: **the demo's own** (the product owner's choice), not the game's.
//!  - **Dry fire.** On an empty clip with nothing to reload, each trigger cycle (once per
//!    1 / rate while held, FUN_002327f0 setting the cooldown) plays the weapon's empty-fire sound
//!    (h_e6acaff7: f740ecdd for ballistic guns, matched in the takes; 16bbf95c for energy guns).
//!  - **Recharging clips.** A gun with an ammo-regen (h_1d1e0e9c: the LZRs) never reloads; its
//!    clip gains a round every ammo-regen seconds while it isn't full and its cooldown is out
//!    (FUN_0022ebc0), stowed or not, by its own cooldown (`Charge`). The HUD shows "N Regen".
//!
//! Test hooks: BF_TEST_RESERVE=<n> starts every ammo-type's reserve at n; BF_TEST_CLIP=<n>
//! starts the player's clips at n; BF_TEST_TRIGGER=<from>-<to>[,<from>-<to>...] holds the
//! trigger over those windows (s, overriding BF_TEST_FIRE); BF_TEST_SWITCH=<s>[,<s>...] presses
//! the weapon switch once at each time (owning the switch for the run, Q included);
//! BF_TEST_KNOCK=<s> knocks the player down then; BF_AMMO_LOG prints the reserve, each reload,
//! dry shot, recharged round and the switch hint.

use super::*;

/// The ammo items' stack limits by ammo-type, for a level that doesn't define the item: the
/// stack-limit of the `<inventory>` entry with function-type 14 and that h_e5f51266, which is the
/// same in every objecttypes that has it (11mm 50, High ROF 600, Sonic 200, Particle 200, Rail
/// 400, Shotgun 80, Cutter 120, Bio 200, Rocket 40, Energy 200). Types without an item (the
/// LZRs' 11) have no reserve.
const STACK_LIMITS: [(i64, i64); 10] = [(1, 50), (2, 600), (3, 200), (4, 200), (5, 400), (6, 80), (7, 120), (8, 200), (10, 40), (12, 200)];
/// IFSET_AMMO_BOX (default.xbe's IFSET_ name table at 0x3be3cc): an ammo item's function-type.
const AMMO_ITEM: i64 = 14;
/// How loud the dry-fire click plays (the demo's mix, as the reload sound's).
const DRY_VOLUME: f32 = 0.8;

/// The squad's reserve: rounds carried per ammo-type, shared by every member and every gun of
/// that type. Filled once per level (`stock`).
#[derive(Resource, Default)]
pub struct Reserve {
    rounds: HashMap<i64, i64>,
    stocked: bool,
}

impl Reserve {
    /// Rounds of `ammo_type` carried.
    pub fn get(&self, ammo_type: i64) -> i64 {
        self.rounds.get(&ammo_type).copied().unwrap_or(0)
    }

    /// Takes up to `n` rounds of `ammo_type`; how many there were.
    fn take(&mut self, ammo_type: i64, n: i64) -> i64 {
        let have = self.rounds.entry(ammo_type).or_insert(0);
        let took = n.min(*have).max(0);
        *have -= took;
        took
    }
}

/// How long (s) the HUD shows the held weapon's name over its count after a pickup, and the fade
/// at its end (todo/49 take04: "Bower 20" from 7.0 s, fading 9.55-9.85 s, the icon, faint
/// meanwhile, coming back as it goes).
pub const NAME_TIME: f32 = 2.85;
pub const NAME_FADE: f32 = 0.3;

/// The held weapon's name on the HUD after a pickup: seconds left (set by the test map's rack).
#[derive(Resource, Default)]
pub struct HeldName(pub f32);

/// The HUD's "[Y] switch to %s." is up. FUN_00120d40 posts it (message 0x52) where it would
/// start a reload but the clip and the reserve are both empty: on the empty trigger cycle (the
/// dry click) or at the release. In the takes it came 0.1-0.2 s after the last round (the
/// Bower's tap released, the MK's and Minigun's held trigger at their next cycle), stayed, went
/// with a switch away and was back on the switch back. A recharging gun posts 0x53 instead,
/// which shows nothing (todo/50 take03 5.2 s).
#[derive(Resource, Default)]
pub struct SwitchHint(pub bool);

pub fn plugin(app: &mut App) {
    app.init_resource::<Reserve>()
        .init_resource::<HeldName>()
        .init_resource::<SwitchHint>()
        .add_systems(OnEnter(AppState::Playing), |mut commands: Commands| {
            commands.insert_resource(Reserve::default());
            commands.insert_resource(HeldName::default());
            commands.insert_resource(SwitchHint::default());
        })
        .add_systems(Update, (
            (stock, test_clip, test_trigger).chain().after(read_input).before(update_player),
            (reload_and_dry_fire, recharge, tick_name).chain().after(update_player).before(play_sounds),
        ).run_if(in_state(AppState::Playing)));
}

/// The ammo item stack limits of the loaded data, by ammo-type (STACK_LIMITS for the rest).
fn stack_limits(game: &Game) -> HashMap<i64, i64> {
    let mut out: HashMap<i64, i64> = STACK_LIMITS.into_iter().collect();
    for item in game.items.values().filter(|i| i.function == AMMO_ITEM && i.kind > 0 && i.stack_limit > 0) {
        out.insert(item.kind, item.stack_limit);
    }
    out
}

/// Fill the reserve once the level's data is in: a full stack of every ammo-type. The game's
/// counts are the campaign's (the takes start with full stacks of 11mm, Sonic and Shotgun, and
/// half of High ROF and Cutter); a full stack is the demo's choice.
fn stock(mut reserve: ResMut<Reserve>, game: Res<GameData>, player: Res<Player>) {
    if reserve.stocked || player.loaded.is_none() {
        return;
    }
    let test = std::env::var("BF_TEST_RESERVE").ok().and_then(|v| v.parse::<i64>().ok());
    reserve.rounds = stack_limits(&game.0).into_iter().map(|(t, n)| (t, test.unwrap_or(n))).collect();
    reserve.stocked = true;
    if std::env::var("BF_AMMO_LOG").is_ok() {
        let mut list: Vec<_> = reserve.rounds.iter().collect();
        list.sort();
        println!("reserve: {}", list.iter().map(|(t, n)| format!("type {t} {n}")).collect::<Vec<_>>().join(", "));
    }
}

/// BF_TEST_CLIP=<n>: the player's clips start at n (once, when the first character is in).
fn test_clip(mut player: ResMut<Player>, mut done: Local<bool>) {
    let Some(n) = std::env::var("BF_TEST_CLIP").ok().and_then(|v| v.parse::<i64>().ok()) else { return };
    if *done || player.loaded.is_none() {
        return;
    }
    *done = true;
    for a in player.ammo.iter_mut() {
        a[0] = n.min(a[0]).max(0);
    }
}

/// BF_TEST_TRIGGER=<from>-<to>,...: the trigger held over those windows; BF_TEST_SWITCH=<s>,...:
/// the weapon switch pressed at those times; BF_TEST_KNOCK=<s>: the player knocked down then
/// (thrown back, as by a blast), once.
fn test_trigger(mut player: ResMut<Player>, mut last_switch: Local<f32>, mut knocked: Local<bool>) {
    let t = player.sim_time;
    if let Some(at) = std::env::var("BF_TEST_KNOCK").ok().and_then(|v| v.parse::<f32>().ok()) {
        if !*knocked && t >= at && player.loaded.is_some() {
            *knocked = true;
            let back = Quat::from_rotation_y(player.yaw) * Vec3::Z;
            player.knock_request = Some(back * 4.0 + Vec3::Y * 1.5);
        }
    }
    if let Ok(v) = std::env::var("BF_TEST_TRIGGER") {
        player.fire = v.split(',').filter_map(|w| w.split_once('-'))
            .filter_map(|(a, b)| Some((a.trim().parse::<f32>().ok()?, b.trim().parse::<f32>().ok()?)))
            .any(|(a, b)| (a..b).contains(&t));
    }
    // (once per time given: the time it last pressed is kept, so it's one press whatever the
    // frame rate; the hook owns the switch for the run, Q included)
    if let Ok(v) = std::env::var("BF_TEST_SWITCH") {
        let due = v.split(',').filter_map(|x| x.trim().parse::<f32>().ok()).filter(|&s| t >= s && *last_switch < s).fold(None, |a: Option<f32>, s| Some(a.map_or(s, |a| a.max(s))));
        player.switch_pressed = due.is_some();
        if let Some(s) = due {
            *last_switch = s;
        }
    }
}

/// Whether `u` is up to a trigger pull: holding its gun, not mid-action (the fire block's own
/// conditions in `step_player`, less the ammo).
fn ready(u: &Player) -> bool {
    u.holding && u.switching.is_none() && u.reloading.is_none() && u.throwing.is_none() && u.using.is_none()
        && !matches!(u.action, Action::Dodge { .. }) && !u.on_all_fours && !knocked_down(u)
}

/// Each character's held gun: start a reload (empty clip, as the game does; or R, the demo's),
/// or click on an empty one with nothing to load.
fn reload_and_dry_fire(mut player: ResMut<Player>, mut squad: ResMut<Squad>, mut reserve: ResMut<Reserve>, mut hint: ResMut<SwitchHint>) {
    let log = std::env::var("BF_AMMO_LOG").is_ok();
    for (k, u) in std::iter::once(&mut *player).chain(squad.0.iter_mut()).enumerate() {
        let Some(l) = u.loaded.take() else { continue };
        let clicked = reload_or_click(u, &l, &mut reserve, log);
        // the player's switch hint: up at the dry click or the release, while the gun stays dry
        if k == 0 {
            let dry = l.weapons.get(u.weapon).is_some_and(|w| w.def.ammo_regen <= 0.0 && reserve.get(w.def.ammo_type) == 0)
                && u.ammo.get(u.weapon).is_some_and(|a| a[0] == 0 && a[1] == 0)
                && u.reloading.is_none() && u.switching.is_none() && !u.dead;
            let up = dry && (hint.0 || clicked || !u.fire);
            if log && up != hint.0 {
                println!("t={:.2} switch hint {} (weapon {}, clip {:?}, fire {}, switching {}, reloading {})", u.sim_time, if up { "up" } else { "down" },
                         u.weapon, u.ammo.get(u.weapon), u.fire, u.switching.is_some(), u.reloading.is_some());
            }
            hint.0 = up;
        }
        u.loaded = Some(l);
    }
}

/// `reload_and_dry_fire` for one character (`l`: its loaded model, taken out of it). Whether it
/// clicked dry.
fn reload_or_click(u: &mut Player, l: &Loaded, reserve: &mut Reserve, log: bool) -> bool {
    u.reload_wanted |= u.reload_pressed;
    let w = u.weapon;
    let (Some(held), Some(&[clip, _])) = (l.weapons.get(w), u.ammo.get(w)) else { return false };
    if u.dead {
        u.reload_wanted = false;
        return false;
    }
    let mut clicked = false;
    let def = &held.def;
    let size = def.ammo.max(1);
    // (knocked down: step_player doesn't run for them, so nothing may start here either)
    let busy = u.reloading.is_some() || u.switching.is_some() || u.throwing.is_some() || u.using.is_some() || knocked_down(u);
    // a recharging gun never reloads (FUN_00120d40's ammo-regen branch, message 0x53); the test
    // map's NPCs (play_testtools.rs) aren't the squad: they reload from a bottomless reserve of
    // their own (the demo's choice)
    let npc = u.npc.is_some();
    let carried = if def.ammo_regen > 0.0 { 0 } else if npc { size } else { reserve.get(def.ammo_type) };
    // FUN_002327f0 runs when the cooldown is out with the trigger held; FUN_00121170's action 4
    // at the release (the measured reload starts)
    // (the empty cycle itself runs in the fire loop's game frames, play_shots.rs)
    let empty = u.fire_state.take_empty();
    let empty_cycle = clip == 0 && (!u.fire || empty);
    if !busy && carried > 0 && clip < size && (empty_cycle || u.reload_wanted) {
        // FUN_0022daf0: the rounds leave the reserve now (the HUD's reserve drops at the start);
        // the clip shows 0 until the magazine-in event
        let took = if npc { size - clip } else { reserve.take(def.ammo_type, size - clip) };
        u.ammo[w] = [0, took];
        u.fire_state.reset_burst();                         // FUN_0022daf0
        u.reloading = Some(Reload { weapon: w, time: 0.0, fill: clip + took, filled: false });
        u.aim_hold = u.aim_hold.min(0.3);
        // FUN_00120d40 calls FUN_001249b0 (out of the scope, with its sound) first when the
        // holder's +0x7a8 is set: read as the scope (medium confidence, weapons spec section 0)
        u.scope_level = 0;
        if log {
            println!("t={:.2} {} reloads {}: {took} rounds, reserve type {} now {}", u.sim_time, CHARACTERS[u.character % CHARACTERS.len()],
                     def.label, def.ammo_type, reserve.get(def.ammo_type));
        }
    } else if u.fire && clip == 0 && empty && carried == 0 && ready(u) {
        // dry fire: the empty-fire sound, the cooldown set as for a shot (FUN_002327f0)
        if def.empty_sound != 0 {
            u.sound_queue.push((def.empty_sound, DRY_VOLUME));
        }
        // (the cooldown is already set, 1 / rate or the scoped rate, and the burst cleared:
        // the fire loop's empty cycle, play_shots.rs)
        if let Some(c) = u.regen.get_mut(w) {
            c.cooldown = u.cooldown;
        }
        clicked = true;
        if log {
            println!("t={:.2} {} dry fire {} (sound h_{:08x})", u.sim_time, CHARACTERS[u.character % CHARACTERS.len()], def.label, def.empty_sound);
        }
    }
    if !busy {
        u.reload_wanted = false;
    }
    clicked
}

/// A weapon's own recharge state (the game's weapon object): the timer toward the next round
/// (+0x220), its own fire cooldown (+0x218: the demo's `Player::cooldown` is the character's,
/// and also holds the demo's own 0.2 s raise and 0.1 s after a switch, which the game's weapon
/// doesn't have), and the clip last seen (a drop is a shot) and whether a switch was under way.
#[derive(Clone, Copy, Default)]
pub struct Charge {
    timer: f32,
    cooldown: f32,
    clip_seen: i64,
    switching: bool,
}

/// The LZRs' clips (FUN_0022ebc0, every weapon object every frame): while a clip isn't full and
/// its gun's cooldown is out, a timer runs, and at the weapon's ammo-regen seconds the clip
/// gains a round; full, or cooling down, the timer starts again. So the first round comes back
/// cooldown + ammo-regen after the last shot (the takes: LZR-23 1.6-1.7 s), and a stowed gun
/// recharges too. The cooldown is set to 1 / rate by a shot (FUN_0022f0e0), a dry click
/// (FUN_002327f0) and the start of a weapon switch, on the gun put away: FUN_0011e130 starts the
/// slot change (FUN_00122da0 / 00122940 / 00122b20, which test the slot at +0x5b0) and then calls
/// FUN_0022dc00 on the held weapon. That's the takes' gap on the switch away and none on the way
/// back (LZR-50 20.996 -> 23.202 s with Y at 21.83: 21.83 + 0.27 + 1.0 = 23.10; LZR-23 21.512 ->
/// 23.419 with Y at 21.7: 21.7 + 0.4 + 1.25 = 23.35).
fn recharge(time: Res<Time>, mut player: ResMut<Player>, mut squad: ResMut<Squad>) {
    let dt = frame_dt(&time);
    let log = std::env::var("BF_AMMO_LOG").is_ok();
    for u in std::iter::once(&mut *player).chain(squad.0.iter_mut()) {
        let Some(l) = u.loaded.as_ref() else { continue };
        let n = l.weapons.len();
        let scoped = u.scope > 0.5;
        let guns: Vec<(f32, i64, f32)> = l.weapons.iter().map(|w| (w.def.ammo_regen, w.def.ammo.max(1), shots::period(&w.def, scoped))).collect();
        u.regen.resize(n, Charge::default());
        let switching = u.switching.is_some();
        for (k, (every, size, period)) in guns.into_iter().enumerate() {
            if every <= 0.0 || k >= u.ammo.len() {
                continue;
            }
            let clip = u.ammo[k][0];
            let c = &mut u.regen[k];
            // a shot (the clip dropped), or a switch starting with this gun in hand
            if clip < c.clip_seen || (switching && !c.switching && k == u.weapon) {
                c.cooldown = period;
            }
            c.switching = switching;
            c.cooldown = (c.cooldown - dt).max(0.0);
            if clip < size && c.cooldown <= 0.0 {
                c.timer += dt;
                if c.timer >= every {
                    c.timer = 0.0;
                    u.ammo[k][0] += 1;
                    if log {
                        println!("t={:.2} {} weapon {k} recharged to {}", u.sim_time, CHARACTERS[u.character % CHARACTERS.len()], u.ammo[k][0]);
                    }
                }
            } else {
                c.timer = 0.0;
            }
            u.regen[k].clip_seen = u.ammo[k][0];
        }
    }
}

/// The pickup name's time runs out.
fn tick_name(time: Res<Time>, mut name: ResMut<HeldName>) {
    name.0 = (name.0 - frame_dt(&time)).max(0.0);
}
