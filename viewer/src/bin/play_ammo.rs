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
//!    (FUN_0022ebc0), stowed or not. The HUD shows "N Regen" for it.
//!
//! Test hooks: BF_TEST_RESERVE=<n> starts every ammo-type's reserve at n; BF_TEST_CLIP=<n>
//! starts the player's clips at n; BF_TEST_TRIGGER=<from>-<to>[,<from>-<to>...] holds the
//! trigger over those windows (s, overriding BF_TEST_FIRE); BF_TEST_SWITCH=<s>[,<s>...] presses
//! the weapon switch at those times; BF_AMMO_LOG prints each reload, dry shot and recharged
//! round.

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

/// How long (s) the HUD shows the held weapon's name over its count after a pickup (todo/49
/// take04: "Bower 20" from 7.0 s, fading 9.55-9.85 s, the icon fading in as it goes): the hold
/// and half the fade.
pub const NAME_TIME: f32 = 2.7;

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
/// the weapon switch pressed at those times.
fn test_trigger(mut player: ResMut<Player>) {
    let t = player.sim_time;
    if let Ok(v) = std::env::var("BF_TEST_TRIGGER") {
        player.fire = v.split(',').filter_map(|w| w.split_once('-'))
            .filter_map(|(a, b)| Some((a.trim().parse::<f32>().ok()?, b.trim().parse::<f32>().ok()?)))
            .any(|(a, b)| (a..b).contains(&t));
    }
    if let Ok(v) = std::env::var("BF_TEST_SWITCH") {
        let step = capture_step();
        player.switch_pressed = v.split(',').filter_map(|x| x.trim().parse::<f32>().ok()).any(|s| t - step < s && t >= s);
    }
}

/// Whether `u` is up to a trigger pull: holding its gun, not mid-action (the fire block's own
/// conditions in `step_player`, less the ammo).
fn ready(u: &Player) -> bool {
    u.holding && u.switching.is_none() && u.reloading.is_none() && u.throwing.is_none() && u.using.is_none()
        && !matches!(u.action, Action::Dodge { .. }) && !u.on_all_fours
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
    let busy = u.reloading.is_some() || u.switching.is_some() || u.throwing.is_some() || u.using.is_some();
    // a recharging gun never reloads (FUN_00120d40's ammo-regen branch, message 0x53); the test
    // map's NPCs (play_testtools.rs) aren't the squad: they reload from a bottomless reserve of
    // their own (the demo's choice)
    let npc = u.npc.is_some();
    let carried = if def.ammo_regen > 0.0 { 0 } else if npc { size } else { reserve.get(def.ammo_type) };
    // FUN_002327f0 runs when the cooldown is out with the trigger held; FUN_00121170's action 4
    // at the release (the measured reload starts)
    let empty_cycle = clip == 0 && (!u.fire || u.cooldown <= 0.0);
    if !busy && carried > 0 && clip < size && (empty_cycle || u.reload_wanted) {
        // FUN_0022daf0: the rounds leave the reserve now (the HUD's reserve drops at the start);
        // the clip shows 0 until the magazine-in event
        let took = if npc { size - clip } else { reserve.take(def.ammo_type, size - clip) };
        u.ammo[w] = [0, took];
        u.reloading = Some(Reload { weapon: w, time: 0.0, fill: clip + took, filled: false });
        u.aim_hold = u.aim_hold.min(0.3);
        // FUN_00120d40 calls FUN_001249b0 (out of the scope, with its sound) first when the
        // holder's +0x7a8 is set: read as the scope (medium confidence, weapons spec section 0)
        u.scope_level = 0;
        if log {
            println!("t={:.2} {} reloads {}: {took} rounds, reserve type {} now {}", u.sim_time, CHARACTERS[u.character % CHARACTERS.len()],
                     def.label, def.ammo_type, reserve.get(def.ammo_type));
        }
    } else if u.fire && clip == 0 && u.cooldown <= 0.0 && carried == 0 && ready(u) {
        // dry fire: the empty-fire sound, the cooldown set as for a shot (FUN_002327f0)
        if def.empty_sound != 0 {
            u.sound_queue.push((def.empty_sound, DRY_VOLUME));
        }
        u.cooldown = 1.0 / def.rate.max(0.2);
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

/// The LZRs' clips (FUN_0022ebc0, every weapon object every frame): while a clip isn't full and
/// its gun's cooldown is out, a timer runs, and at the weapon's ammo-regen seconds the clip
/// gains a round; full, or cooling down after a shot, the timer starts again. So the first round
/// comes back cooldown + ammo-regen after the last shot (the takes: LZR-23 1.6-1.7 s), and a
/// stowed gun recharges too.
fn recharge(time: Res<Time>, mut player: ResMut<Player>, mut squad: ResMut<Squad>) {
    let dt = frame_dt(&time);
    let log = std::env::var("BF_AMMO_LOG").is_ok();
    for u in std::iter::once(&mut *player).chain(squad.0.iter_mut()) {
        let Some(l) = u.loaded.as_ref() else { continue };
        let n = l.weapons.len();
        let regen: Vec<(f32, i64)> = l.weapons.iter().map(|w| (w.def.ammo_regen, w.def.ammo.max(1))).collect();
        u.regen.resize(n, 0.0);
        for (k, (every, size)) in regen.into_iter().enumerate() {
            if every <= 0.0 || k >= u.ammo.len() {
                continue;
            }
            // (the demo keeps one cooldown per character: the held gun's; a stowed one's is out)
            let cooling = k == u.weapon && u.cooldown > 0.0;
            if u.ammo[k][0] < size && !cooling {
                u.regen[k] += dt;
                if u.regen[k] < every {
                    continue;
                }
                u.ammo[k][0] += 1;
                if log {
                    println!("t={:.2} {} weapon {k} recharged to {}", u.sim_time, CHARACTERS[u.character % CHARACTERS.len()], u.ammo[k][0]);
                }
            }
            u.regen[k] = 0.0;
        }
    }
}

/// The pickup name's time runs out.
fn tick_name(time: Res<Time>, mut name: ResMut<HeldName>) {
    name.0 = (name.0 - frame_dt(&time)).max(0.0);
}
