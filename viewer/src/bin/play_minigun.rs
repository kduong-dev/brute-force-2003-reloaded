//! IFSET_MINIGUN (#68): the item set of a gun whose function-type is 16, the RVG50 Minigun's,
//! from its per-frame hook FUN_00147f90 (vtable 0x39b8f8 +0, made by the item-set factory
//! 0x146920; the analyst's reading, todo/weapons-plan/answers-batch1.md §12):
//!
//!  - **The spin level** (a helper at weapon+0x1b4, its +0x14): 1 at once while the gun's cooldown
//!    (+0x218) runs with its fire flag set, else down at SPIN_DECAY a second to 0
//!    (FUN_00147e50): a 1 s spin-down. Nothing else depends on it: the fire loop's 15 a second
//!    holds from the first round, with no spin-up (#68 take01: the first round 0.1-0.2 s after
//!    the press, the raise).
//!  - **Its sounds** (FUN_00147dd0): at 1 the loop SPIN_LOOP starts, once (FUN_00147bc0 ->
//!    FUN_00147c20: a voice with a SPIN_FADE fade, 2D for the player's holder, else at the
//!    weapon); once the level is under 1 the loop gives way to SPIN_DOWN, once (FUN_00147d70),
//!    and the state clears when that ends (FUN_00147b90). Both are in the common bank.
//!  - An effect from the item's attached-effect list while it spins (FUN_00147ed0): the
//!    Minigun's list (its h_18b6ab72 slots) is empty, so nothing shows.
//!  - Holstered or dropped (FUN_00148030) the loop and the effect stop and the state clears.
//!
//! The barrel's turn (play.rs `update_weapons`, the model's spinning parts) is BARREL_TURN x the
//! level. `BF_SOUND_LOG` prints the loop's start and end.

use super::*;
use bevy::audio::Volume as AudioVolume;

/// IFSET_MINIGUN: function-type 16 (the IFSET_ name table at 0x409f18-0x40a140, numbered from
/// IFSET_NONE 0 at the bottom: plan §4.6, the factory 0x146920's order).
const IFSET_MINIGUN: i64 = 16;
/// FUN_00147e50: how fast the spin level falls once the gun stops firing (a second; the helper's
/// +0x10 = 1.0): a 1 s spin-down.
const SPIN_DECAY: f32 = 1.0;
/// FUN_00147bc0: the barrel's loop, started at spin 1 (h_0e8038d2, common bank).
const SPIN_LOOP: u32 = 0x0E80_38D2;
/// FUN_00147c20: the loop's voice fades in over this (s).
const SPIN_FADE: f32 = 0.1;
/// FUN_00147d70: the spin-down, once, as the level drops under 1 (h_f00b38e4, common bank).
const SPIN_DOWN: u32 = 0xF00B_38E4;
/// The barrel's turn at spin 1 (rad/s): the demo's (its own ease's top before #68). How the
/// game turns the barrel isn't traced (the model's joint, by an animation or the item set).
pub const BARREL_TURN: f32 = 28.0;
/// A squadmate's loop is heard at this share of its volume at most, falling off over
/// HEARD_REACH m to HEARD_FLOOR (play_sounds' rule for a squadmate's sounds).
const HEARD_SHARE: f32 = 0.8;
const HEARD_REACH: f32 = 35.0;
const HEARD_FLOOR: f32 = 0.15;

/// Whether a gun's item set is IFSET_MINIGUN (its function-type 16: the RVG50 Minigun).
pub fn spins(def: &WeaponDef) -> bool {
    def.function_type == IFSET_MINIGUN
}

/// One gun's spin (FUN_00147f90's helper): which of its holder's guns, its level, whether its loop
/// plays (and the loop's voice, none under BF_MUTE), how long the loop has played (its fade), the
/// spin-down's time left (the sound's length: the loop can't start again until it's out), the
/// holder's rounds fired at the last look (a round this frame counts as firing: a tap's shot
/// comes after the button is up), and whether a round has gone since the button went down (the
/// demo holds the raise before the first round in the gun's cooldown, play.rs `raise`, where the
/// game's cooldown runs only after a round: no spin before it).
struct Spin {
    weapon: usize,
    level: f32,
    looping: bool,
    voice: Option<Entity>,
    age: f32,
    down_left: f32,
    rounds: u32,
    shot: bool,
}

/// The spinning guns, by holder and hand (`shots::HANDS`).
#[derive(Resource, Default)]
struct Spins(HashMap<(Who, usize), Spin>);

/// The loop's voice entity.
#[derive(Component)]
struct SpinVoice;

/// The spin's system: after the fire loop's frame (update_player), before the guns are drawn.
pub fn plugin(app: &mut App) {
    app.init_resource::<Spins>()
        .add_systems(Update, spin_miniguns.after(update_player).before(update_weapons).run_if(in_state(AppState::Playing)))
        .add_systems(OnExit(AppState::Playing), stop_all);
}

/// FUN_00147f90 for every IFSET_MINIGUN gun in a hand: its spin level, its loop and spin-down,
/// and the holder's barrel turn (`Player::spin`). A gun no longer in a hand (switched, holstered,
/// its holder dead or gone) stops at once, loop and all (FUN_00148030).
#[allow(clippy::too_many_arguments)]
fn spin_miniguns(mut commands: Commands, time: Res<Time>, mut player: ResMut<Player>, mut squad: ResMut<Squad>, game: Res<GameData>,
                 mut state: ResMut<Spins>, mut cache: ResMut<SoundCache>, mut sources: ResMut<Assets<AudioSource>>,
                 mut sinks: Query<&mut AudioSink, With<SpinVoice>>) {
    let dt = frame_dt(&time);
    let listener = player.position;
    let now = player.sim_time;
    let mute = std::env::var("BF_MUTE").is_ok();
    let log = std::env::var("BF_SOUND_LOG").is_ok();
    let loop_volume = game.0.sounds.params(SPIN_LOOP).map_or(1.0, |p| p.volume);
    let down_volume = game.0.sounds.params(SPIN_DOWN).map_or(1.0, |p| p.volume);
    let mut held = vec![];
    for (k, u) in std::iter::once(&mut *player).chain(squad.0.iter_mut()).enumerate() {
        let who = u.who();
        // (heard as the player's own, or a squadmate's by its distance)
        let heard = if k == 0 { 1.0 } else { HEARD_SHARE * (1.0 - u.position.distance(listener) / HEARD_REACH).clamp(HEARD_FLOOR, 1.0) };
        let mut turn: f32 = 0.0;
        for hand in 0..shots::HANDS {
            let gun = if hand == 0 { Some((u.weapon, u.cooldown)) } else { u.second.map(|s| (s.weapon, s.cooldown)) };
            let Some((weapon, cooldown)) = gun.filter(|&(w, _)| !u.dead && u.holding
                && u.loaded.as_ref().and_then(|l| l.weapons.get(w)).is_some_and(|w| spins(&w.def))) else { continue };
            held.push((who, hand));
            let fresh = Spin { weapon, level: 0.0, looping: false, voice: None, age: 0.0, down_left: 0.0, rounds: u.shots_fired, shot: false };
            let st = state.0.entry((who, hand)).or_insert(fresh);
            if st.weapon != weapon {
                stop(&mut commands, st);
                *st = Spin { weapon, level: 0.0, looping: false, voice: None, age: 0.0, down_left: 0.0, rounds: u.shots_fired, shot: false };
            }
            // FUN_00147e50: 1 while the cooldown runs with the fire flag set (the button down,
            // or a round fired this frame) after a round, else falling
            let fired = u.shots_fired != st.rounds;
            st.rounds = u.shots_fired;
            if fired {
                st.shot = true;
            } else if !u.fire && cooldown <= 0.0 {
                st.shot = false;
            }
            st.level = if cooldown > 0.0 && (u.fire || fired) && st.shot { 1.0 } else { (st.level - SPIN_DECAY * dt).max(0.0) };
            // FUN_00147dd0: the loop at 1 (not while the spin-down plays: its flags clear when
            // that ends, FUN_00147b90; a reading), the spin-down in its place below 1
            if st.level >= 1.0 && !st.looping && st.down_left <= 0.0 {
                st.looping = true;
                st.age = 0.0;
                let handle = cache.0.entry(SPIN_LOOP).or_insert_with(|| game.0.sounds.wav(SPIN_LOOP)
                    .map(|w| sources.add(AudioSource { bytes: std::sync::Arc::from(w.into_boxed_slice()) }))).clone();
                if log {
                    println!("t={now:6.2} sound {SPIN_LOOP:08x} looping (the Minigun's spin, {}){}", unit_name(u),
                             if mute { ": BF_MUTE" } else if handle.is_none() { ": no sample data" } else { "" });
                }
                if let Some(h) = handle.filter(|_| !mute) {
                    st.voice = Some(commands.spawn((AudioPlayer::new(h), PlaybackSettings { volume: AudioVolume::Linear(0.0), ..PlaybackSettings::LOOP },
                                                    SpinVoice, Name::new("minigun spin"))).id());
                }
            } else if st.level < 1.0 && st.looping {
                stop(&mut commands, st);
                if log {
                    println!("t={now:6.2} sound {SPIN_LOOP:08x} stops (the Minigun's spin, {})", unit_name(u));
                }
                u.sound_queue.push((SPIN_DOWN, down_volume));
                // (the sample's length, decoded only here: a spin-down is rare, a frame isn't)
                st.down_left = game.0.sounds.pcm(SPIN_DOWN).map_or(0.0, |(rate, s)| s.len() as f32 / rate as f32);
            }
            st.down_left = (st.down_left - dt).max(0.0);
            if let Some(mut sink) = st.voice.and_then(|v| sinks.get_mut(v).ok()) {
                st.age += dt;
                sink.set_volume(AudioVolume::Linear(loop_volume * heard * (st.age / SPIN_FADE).min(1.0)));
            }
            turn = turn.max(st.level);
        }
        // (both hands' barrels turn together, play.rs)
        if u.loaded.as_ref().is_some_and(|l| l.weapons.iter().any(|w| spins(&w.def))) {
            u.spin = BARREL_TURN * turn;
        }
    }
    // guns no longer in a hand: stopped at once (FUN_00148030)
    state.0.retain(|key, st| {
        let keep = held.contains(key);
        if !keep {
            stop(&mut commands, st);
        }
        keep
    });
}

/// FUN_00148030 and the spin-down's start: the loop's voice gone.
fn stop(commands: &mut Commands, st: &mut Spin) {
    if let Some(v) = st.voice.take() {
        commands.entity(v).try_despawn();
    }
    st.looping = false;
}

/// Leaving play: every loop stops.
fn stop_all(mut commands: Commands, mut state: ResMut<Spins>) {
    for st in state.0.values_mut() {
        stop(&mut commands, st);
    }
    state.0.clear();
}
