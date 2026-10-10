//! The Gas grenade's poison cloud (issue #76; the measurements are the reference agent's, from
//! the user's xemu recording "Gas Grenade": Tex walks into one cloud and is killed by a second).
//!
//! The Gas (h_e8c67904) goes off 1 s after it lands like the other thrown grenades
//! (play_grenade.rs): its effect type h_065e4b95 plays the gas-grenade cloud, the
//! smoke-grenade-flsh flash and the smoke_grenade-shrap sparks, and its sound h_1b35643e. Its
//! explosion h_0cc6cb8c (`<Damage max=97.5 min=48 damage-type=4 radius=3 h_04ea9251=5.5>`)
//! does nothing at once - the recording's health bar doesn't move on the blast - so
//! play_grenade.rs hands a blast whose Damage has an h_04ea9251 (`WeaponDef::damage_time`) to
//! this module instead of dealing it (`GasClouds::release`). Here, everyone within the radius
//! loses health steadily (`poison`) for h_04ea9251 seconds:
//!  - the rate is Damage max over h_04ea9251 (97.5 / 5.5 = 17.7 HP/s), the same anywhere in
//!    the radius. The recording drains Tex linearly at 15.5 HP/s in the first cloud (walking in
//!    from its edge 2.2 s after it went off, without the rate rising as he came closer: no
//!    falloff seen) and 16.9 HP/s in the second (~2 m from it); max over the time is the data's
//!    nearest value. Whether the game rolls min..max per cloud isn't known: that would average
//!    13.2 HP/s, below both clouds. A guess fitted to the recording;
//!  - it starts GAS_DELAY after the blast and lasts h_04ea9251 seconds. The recording's drain
//!    stops 5.80 s after the first cloud went off (the demo's 5.58 s, 0.2 s short);
//!  - the thrower is poisoned like anyone (it was Tex's own cloud), times each character's
//!    factor for damage-type 4 (`Game::damage_factor`: Flint's x0.05);
//!  - no blood, no red tint (the recording's picture only goes olive in the cloud), no hurt
//!    chatter; a character it kills dies as any other (`hurt`).
//!
//! Not recorded, so not done: squadmates leaving or avoiding the cloud.

use super::*;
use super::grenade::{GrenadeKits, Thrown};

/// The poison starts this long after the blast: the second cloud's drain starts 5 frames of the
/// 60 fps recording after the blast's first frame (844 -> 849). Measured.
const GAS_DELAY: f32 = 5.0 / 60.0;
/// A poison cloud: the grenade type, where it went off (and the controlled character's sim time
/// then, for BF_GRENADE_LOG), who threw it, its age (s; advanced by the frame's time, not by a
/// character's sim time, which stops while they're dead or down and changes at a hand-over),
/// whether it went off this frame, and per character it has reached: (character, its age when
/// first and last poisoning them, HP taken) for BF_GRENADE_LOG.
pub struct GasCloud {
    kind: usize,
    at: Vec3,
    born: f32,
    thrower: usize,
    age: f32,
    fresh: bool,
    victims: Vec<(usize, f32, f32, f32)>,
}

/// The poison clouds hanging where Gas grenades went off.
#[derive(Resource, Default)]
pub struct GasClouds(Vec<GasCloud>);

impl GasClouds {
    /// A blast of grenade type `kind` at `at` (at sim time `now`, for the log), by character
    /// `thrower`, whose damage is dealt over time (its Damage h_04ea9251 > 0): its cloud poisons
    /// from then on.
    pub fn release(&mut self, kind: usize, at: Vec3, thrower: usize, now: f32) {
        self.0.push(GasCloud { kind, at, born: now, thrower, age: 0.0, fresh: true, victims: vec![] });
    }
}

pub fn plugin(app: &mut App) {
    app.init_resource::<GasClouds>()
        .add_systems(OnEnter(AppState::Playing), |mut clouds: ResMut<GasClouds>| clouds.0.clear())
        // (after the frame's blasts: a cloud starts ageing the frame after it went off)
        .add_systems(Update, (test_drop, poison).chain().after(super::grenade::GrenadeSystems).before(play_sounds)
            .run_if(in_state(AppState::Playing)));
}

/// Test hook: BF_TEST_DROP=<s>[,<s>...] sets off the selected grenade type at the controlled
/// character's feet at each time (to stand in a Gas cloud; any type works).
fn test_drop(mut player: ResMut<Player>, kits: Option<Res<GrenadeKits>>, mut done: Local<usize>) {
    let Some(times) = std::env::var("BF_TEST_DROP").ok().map(|v| v.split(',').filter_map(|x| x.trim().parse::<f32>().ok()).collect::<Vec<_>>())
        else { return };
    let Some(kits) = kits else { return };
    let Item::Grenade(kind) = player.item else { return };
    if *done < times.len() && player.sim_time >= times[*done] && kind < kits.0.len() {
        *done += 1;
        let (x, z) = (player.position.x, player.position.z);
        let pos = Vec3::new(x, floor_y(x, z, player.position.y + GROUND + 1.0) + super::grenade::GRENADE_RADIUS, z);
        player.thrown.push(Thrown { pos, velocity: Vec3::ZERO, kind, landed: true });
    }
}

/// The clouds' poison: everyone within a cloud's radius (the same distance the blasts use)
/// loses Damage max / h_04ea9251 HP a second, times their factor for its damage-type, from
/// GAS_DELAY after the blast for h_04ea9251 seconds; then the cloud stops hurting. The test
/// map's instant kill kills a squadmate it reaches (the player's grenades do there).
fn poison(time: Res<Time>, mut player: ResMut<Player>, mut squad: ResMut<Squad>, game: Res<GameData>, kits: Option<Res<GrenadeKits>>,
          mut clouds: ResMut<GasClouds>, test: Option<Res<super::testmap::TestMap>>) {
    let Some(kits) = kits else { return };
    let dt = frame_dt(&time);
    let instant = test.as_ref().is_some_and(|t| t.instant_kill);
    let log = std::env::var("BF_GRENADE_LOG").is_ok();
    clouds.0.retain_mut(|c| {
        let Some(kit) = kits.0.get(c.kind) else { return false };
        let (radius, span) = (kit.blast.blast_radius, kit.blast.damage_time);
        let rate = kit.blast.damage / span.max(1e-3);
        // the part of this frame inside the poisoning window (a cloud from this frame's blast
        // starts ageing next frame)
        let (from, to) = (c.age, if c.fresh { c.age } else { c.age + dt });
        c.age = to;
        if c.fresh && log {
            println!("t {:.2}: {} cloud at {:.2}: {rate:.1} HP/s within {radius} m, from +{GAS_DELAY:.2} s for {span} s (damage-type {})",
                     c.born, kit.def.label, c.at, kit.blast.damage_type);
        }
        c.fresh = false;
        let now = c.age;
        let inside = (to.min(GAS_DELAY + span) - from.max(GAS_DELAY)).max(0.0);
        if inside > 0.0 {
            let leader = std::iter::once((&mut *player, false));
            for (u, kill) in leader.chain(squad.0.iter_mut().map(|u| (u, instant))) {
                if u.dead || u.position.distance(c.at) >= radius {
                    continue;
                }
                let factor = game.0.damage_factor(CHARACTERS[u.character], kit.blast.damage_type);
                // (the thrower is the squad's: a test map NPC of their character isn't them)
                let amount = if kill && u.who() != Who::Squad(c.thrower) { u.health } else { rate * inside * factor };
                if amount >= u.health {
                    // its last breath: dies as from any hurt
                    hurt(u, &game.0, amount, HURT_CHATTER, Vec3::Y, u.position, -1);
                    // (without the hurt's blood: the recording's gassed Tex falls without any)
                    u.blood.pop();
                } else {
                    u.health -= amount;
                }
                match c.victims.iter_mut().find(|v| v.0 == u.character) {
                    Some(v) => { v.2 = now; v.3 += amount; }
                    None => c.victims.push((u.character, now, now, amount)),
                }
            }
        }
        let over = c.age >= GAS_DELAY + span;
        if over && log {
            println!("t {:.2}: {} cloud over, {:.2} s after its blast", player.sim_time, kit.def.label, c.age);
            for &(who, first, last, total) in &c.victims {
                let u = std::iter::once(&*player).chain(squad.0.iter()).find(|u| u.character == who);
                println!("  {} poisoned from +{first:.2} to +{last:.2} s: {total:.1} HP ({:.1} HP/s), health {:.1} / {:.0}", CHARACTERS[who],
                         total / (last - first).max(1e-3), u.map_or(0.0, |u| u.health), u.map_or(0.0, |u| u.max_health));
            }
        }
        !over
    });
}
