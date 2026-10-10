//! The Sonic grenade's ring (#81): "Triggers a powerful sonic blast upon impact."
//!
//! Its blast is play_grenade.rs's like any other (timer 0: off at the first contact; the
//! explosion h_faced66c: Damage 39-71.5, damage-type 7 = DTYPE_SONIC, radius 10; the effect type
//! h_0c18f6cc: the ALE effect grenade_sonic - the dome, the ring and the godray shafts - with
//! light_sonic_grenade and the sound f36fb063; decal h_fb24bcb7), but its damage isn't dealt
//! 0.1 s after it goes off: it reaches each body when the ring does.
//!
//! The recording (todo/Sonic Grenade.mp4, the reference agent's frames): the health bars drop
//! at +0.17 s for a body ~2 m out, +0.23 s and +0.47 s for ones farther off; an enemy ~1 m away
//! lurched back and stayed up; no camera shake. That the hits come with the ring is an
//! inference from those times. Here every body within the radius when it goes off is hurt
//! once, RING_SPEED after it per metre (no sooner than play_grenade.rs's DAMAGE_DELAY), by the
//! other blasts' rule (`hurt_by_blast`: Damage max falling to nothing at the radius), the
//! thrower by `self_damage`; nobody is knocked down by it.

use super::*;
use super::grenade::GrenadeKits;
use bf_viewer::bf::hash::h;

/// The ALE effect of the Sonic's blast (in the explosion's effect type's list): a blast with it
/// deals its damage as its ring passes (`carries_damage`).
const RING_EFFECT: &str = "grenade_sonic";
/// How fast the damage spreads out (m/s): sonic_grenade.emt's particle speed (Emitter_Pressure)
/// at its first key, 11.68 - the dome's first particles. With it a body 2 m out is hit at
/// +0.17 s, as recorded; the recording's +0.47 s hit would be ~5.5 m out (its distance wasn't
/// measured).
const RING_SPEED: f32 = 11.68;
/// The earliest a body is hit after the blast (s): the other blasts' delay (play_grenade.rs's
/// DAMAGE_DELAY, the Sonic recording's tint 3 game frames after its flash).
const RING_FIRST: f32 = 0.1;
/// The thrower's damage (`self_damage`): SELF_PEAK x Damage max at the blast, falling to nothing
/// at SELF_REACH m. Fitted to the recording's two throws, not the game's rule (unknown): Tex
/// lost ~28 HP at ~2 m from his first Sonic (T1) and nothing at ~5 m from his second (T2: no
/// tint, his health frame flashed red but the bar didn't move; he had ~9.5 HP, so a flat share
/// would have killed him). 0.653 x 71.5 x (1 - 2/5) = 28. Two points: a thrower at 1, 3 and 4 m
/// would settle the shape (a new shot).
const SELF_PEAK: f32 = 0.653;
const SELF_REACH: f32 = 5.0;
/// The least damage (HP) that brings the player's red tint: smaller hits hurt without it. The
/// recording's T2 (Tex ~5 m out, his bar unmoved) had no tint, where the falloff above still
/// gives a few tenths of an HP at 5.0 m (the distance is from the pelvis, 1 m up). The demo's
/// choice, a guess: the tint isn't scaled with the damage because only the Frag's ~12.5 HP
/// tint was measured.
const TINT_MIN: f32 = 1.0;

/// Sonic blasts this frame: (grenade type, where, the thrower's character), from
/// play_grenade.rs's `fly_grenades`.
#[derive(Resource, Default)]
pub struct RingRequests(pub Vec<(usize, Vec3, usize)>);

/// A body the ring will reach: the blast (its type, where, who threw it), the body
/// (`Player::who`), when (s after the blast) and the time since the blast.
struct RingHit {
    kind: usize,
    at: Vec3,
    thrower: usize,
    target: Who,
    due: f32,
    age: f32,
}

/// The ring's hits still to come.
#[derive(Resource, Default)]
pub(super) struct RingHits(Vec<RingHit>);

/// The ring requests and hits, emptied when a map starts.
pub fn plugin(app: &mut App) {
    app.init_resource::<RingRequests>().init_resource::<RingHits>()
        .add_systems(OnEnter(AppState::Playing), clear);
    // (`ring` runs in play_grenade.rs's chain, right after the blasts that request it)
}

/// A new map: no rings.
fn clear(mut requests: ResMut<RingRequests>, mut hits: ResMut<RingHits>) {
    requests.0.clear();
    hits.0.clear();
}

/// Whether a grenade type's damage comes with its ring (its effect type has RING_EFFECT: the
/// Sonic).
pub fn carries_damage(kit: &super::grenade::GrenadeKit) -> bool {
    kit.blast_fx.effects.contains(&h(RING_EFFECT))
}

/// The thrower's damage from their own Sonic `d` m away (see SELF_PEAK).
fn self_damage(kit: &super::grenade::GrenadeKit, d: f32) -> f32 {
    SELF_PEAK * kit.blast.damage * (1.0 - d / SELF_REACH).max(0.0)
}

/// New Sonic blasts schedule a hit on every living body within the radius, for when the ring
/// gets to it; due hits hurt (`hurt_by_blast`, the thrower by `self_damage`), with the red tint
/// for the player if it hurt them by TINT_MIN or more. A body that has left the radius by then
/// is missed. Nobody is knocked down by the ring: the recording's Tex took 28 HP at ~2 m and
/// an enemy ~1 m away
/// lurched back, and both stayed up (a hit this hard floors a body two times in three
/// otherwise, play.rs's KNOCKDOWN_DAMAGE). A knock-down already pending from another hit that
/// frame is kept.
#[allow(clippy::too_many_arguments)]
pub(super) fn ring(time: Res<Time>, mut player: ResMut<Player>, mut squad: ResMut<Squad>, game: Res<GameData>, kits: Option<Res<GrenadeKits>>,
                   mut requests: ResMut<RingRequests>, mut hits: ResMut<RingHits>, mut tint: ResMut<super::grenade::ScreenTint>,
                   test: Option<Res<super::testmap::TestMap>>) {
    let Some(kits) = kits else { return };
    let log = std::env::var("BF_GRENADE_LOG").is_ok();
    // (hits already out age first: one requested this frame is at +0 s, not +1 frame)
    let dt = frame_dt(&time);
    for hit in &mut hits.0 {
        hit.age += dt;
    }
    for (kind, at, thrower) in requests.0.drain(..) {
        let Some(kit) = kits.0.get(kind) else { continue };
        for u in std::iter::once(&*player).chain(squad.0.iter()).filter(|u| !u.dead) {
            let d = u.position.distance(at);
            if d >= kit.blast.blast_radius {
                continue;
            }
            let due = (d / RING_SPEED).max(RING_FIRST);
            if log {
                println!("t {:.2}: {} ring reaches {} ({d:.1} m) at +{due:.2} s", player.sim_time, kit.def.label, CHARACTERS[u.character]);
            }
            hits.0.push(RingHit { kind, at, thrower, target: u.who(), due, age: 0.0 });
        }
    }
    let instant = test.as_ref().is_some_and(|t| t.instant_kill);
    let p = &mut *player;
    hits.0.retain_mut(|hit| {
        if hit.age < hit.due {
            return true;
        }
        let Some(kit) = kits.0.get(hit.kind) else { return false };
        if let Some((u, controlled)) = unit_mut(p, &mut squad, hit.target) {
            let own = self_damage(kit, u.position.distance(hit.at));
            // (the test map's instant kill: anyone but the player)
            let kill = instant && !controlled;
            // no knock-down: `hurt` floors a body only off its cooldown, so it's held up for
            // the call (and its log says nothing of one)
            let cooldown = u.knock_cooldown;
            u.knock_cooldown = cooldown.max(1.0);
            let hurt = super::grenade::hurt_by_blast(u, &game.0, kit, hit.at, own, hit.thrower, kill);
            u.knock_cooldown = cooldown;
            if hurt.is_some_and(|d| d >= TINT_MIN) && controlled {
                tint.0 = 0.0;
            }
        }
        if log {
            println!("t {:.2}: {} ring hits {:?} at +{:.2} s", p.sim_time, kit.def.label, hit.target, hit.age);
        }
        false
    });
}
