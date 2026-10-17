//! The damage plate (#228, the tutorial's step 9 of #28; viewer/docs/triggers.md, "The damage
//! plate"): a proximity detector that hurts what's in it while it's enabled. A child of
//! play_triggers.rs: its class is the runtime's detector (`ClassState::Detector` with its
//! `damage`), and its system deals the hits to the characters. Written from default.xbe's code;
//! take15 (B2) measures it.
//!
//! The class h_130818c0 (vtable 0x396530; its parser 0x1fa740 reads `damage-type` +0x128,
//! h_e022d922 +0x124 and h_18499bb9 +0x120 after the proximity detector's 0x1fa520, `radius`
//! +0x118; its constructor 0x1fa680: radius 5, amount 0, falloff 0, damage-type 1): a proximity
//! detector (its sphere of `radius`, its occupants and their ENTER / EXIT as any detector's), whose
//! update (0x1fa7e0) runs the detector's (0x1f9680) and then, while it's enabled, FUN_001f8510: for
//! every occupant in its list (+0xdc: from its first frame in the volume, until 0.2 s after it
//! leaves) that's in the world and not dead, the amount x the falloff (FUN_001f8420: by the
//! distance from the trigger to the occupant against the radius) x dt, as a damage record
//! (FUN_00222f60) of `damage-type` from the trigger's place to the character's handler (0x125600:
//! its difficulty step and its factor for the type, play_damage.rs), with no attacker. A negative
//! amount heals instead: the hit points raised by it x dt, at most to the most.
//!
//! The tutorial's one, trigger_damage_plate524 at (27.9, 17.4, 67.6): BALLISTIC 20 a second, flat,
//! a 5 m sphere (its data's `size` 4 2 4 isn't read: a proximity detector's parser takes `radius`
//! alone), squad types. The show_patrol_2 script's 103s at 0.108 s start timer528 (1 s) and enable
//! the plate (USER1 -> ENABLE); the timer's pulse disables it. Tex, warped to direction_point2 at
//! 0.09 s, 2.6 m from its middle, takes 20 x 0.6 (STANDARD's step on the squad) = 12 a second for
//! the second: take15 measured 115 -> 102.7 over clk 182803-183813 (12.2 a second). The script has
//! no 122, so the world's cutscene bit is off and the handler deals it.
//!
//! The hit isn't a weapon's: no hit part, no damage-multiplier (FUN_00125940 is the weapons' record
//! builder's), no push. The first hit on a character goes through play.rs `hurt` (its grunt, but no
//! blood: take15 shows none); the rest of its frames' small amounts come off its hit points
//! directly, as play_scenery.rs's damage areas do, unless one would kill it. Not done: the HUD's
//! damage direction indicator (FUN_000b2ef0, FUN_000b3010), which take15 shows on all four sides,
//! and the health bar's flash (viewer/docs/triggers.md, "Not done").
//!
//! Test hook: BF_COMBAT_LOG=1 prints each hit from a plate (this frame's, the total so far and the
//! hit points left), the first with its factors (`Hit::report`).

use super::*;
use super::super::damage::{Difficulty, Hit};

/// One frame's hit from a damage plate, waiting for the world (`deal`): the character by its
/// detector name (`actor_name`), the amount this frame (negative: healing), its damage-type,
/// where it came from (the trigger's place) and which trigger (for the log).
pub(crate) struct PlateHit {
    victim: u32,
    amount: f32,
    damage_type: i64,
    from: Vec3,
    by: String,
}

/// The characters a damage plate has hurt (by detector name and trigger), for the first-hit rule
/// and the log: the amount dealt so far and when the last hit was. Reset as each map starts.
#[derive(Resource, Default)]
struct PlateVictims(HashMap<(u32, String), (f32, f32)>);

/// The plates' frame: after the triggers' (`super::run`), which collected the hits.
pub fn plugin(app: &mut App) {
    app.init_resource::<PlateVictims>()
        .add_systems(OnEnter(AppState::Playing), |mut commands: Commands| commands.insert_resource(PlateVictims::default()))
        .add_systems(Update, deal.after(super::run).run_if(in_state(AppState::Playing)));
}

/// FUN_001f8420: the share of a damage plate's amount at distance `d` from its middle, its radius
/// `r` (vtable +0x64, 0x1f8890: +0x118), by its falloff h_e022d922: 0 all of it; 1 (1 - d / r) and
/// 2 (1 - d / r)^2 within r; 3 all of it to r / 2, then 1 - 2 (d / r - 0.5) to r; none beyond r for
/// 1-3, and none for any other mode.
fn falloff(mode: i64, d: f32, r: f32) -> f32 {
    let within = d <= r && r > 0.0;
    match mode {
        0 => 1.0,
        1 if within => 1.0 - d / r,
        2 if within => (1.0 - d / r) * (1.0 - d / r),
        3 if within && d <= r * 0.5 => 1.0,
        3 if within => 1.0 - 2.0 * (d / r - 0.5),
        _ => 0.0,
    }
}

impl Triggers {
    /// FUN_001f8510, from the update of damage plate `i` while it's enabled (0x1fa7e0): each
    /// occupant that's a living character in the world (`actors`) takes this frame's share, its
    /// amount x the falloff at its distance x dt, as a hit for `deal`. A plain detector has no
    /// damage and does nothing here.
    pub(super) fn plate_damage(&mut self, i: usize, dt: f32, actors: &[Actor]) {
        let ClassState::Detector { volume, occupants, damage: Some(d), .. } = &self.live[i].state else { return };
        let Some(at) = self.live[i].t.transform else { return };
        let middle = at.w_axis.truncate();
        // (+0x64: the proximity detector's radius; a plate's box has none, its half diagonal)
        let radius = match volume { Volume::Sphere(r) => *r, Volume::Plate(s) => s.length() };
        let mut hits = vec![];
        for o in occupants {
            let Some(a) = actors.iter().find(|a| a.name == o.name) else { continue };
            // (the object's place, its transform's +0x24: a character's feet here)
            let share = falloff(d.falloff, a.feet.distance(middle), radius);
            let amount = d.amount * share * dt;
            if amount != 0.0 {
                hits.push(PlateHit { victim: o.name, amount, damage_type: d.damage_type, from: middle, by: self.label(self.live[i].t.name) });
            }
        }
        self.plate_hits.extend(hits);
    }
}

/// The plates' hits this frame on the characters (see the module notes): healing raises the hit
/// points (at most to the most); damage goes through the character's handler (`Hit::on`: the
/// difficulty step and the factor for its type; none during a cutscene), the first hit from a
/// plate through play.rs `hurt`, the rest straight off the hit points unless it would kill.
#[allow(clippy::too_many_arguments)]
fn deal(mut tr: ResMut<Triggers>, mut player: ResMut<Player>, mut squad: ResMut<Squad>, game: Res<GameData>, difficulty: Res<Difficulty>,
        mut victims: ResMut<PlateVictims>) {
    let hits = std::mem::take(&mut tr.plate_hits);
    if hits.is_empty() {
        return;
    }
    let now = player.sim_time;
    let log = std::env::var("BF_COMBAT_LOG").is_ok();
    for h in hits {
        let Some(u) = std::iter::once(&mut *player).chain(squad.0.iter_mut()).find(|u| actor_name(u) == h.victim && !u.dead && !u.absent) else { continue };
        if h.amount < 0.0 {
            u.health = (u.health - h.amount).min(u.max_health);
            continue;
        }
        // (the handler's early exit, 0x125600: nobody is hurt while a cutscene has the world)
        if super::super::script::in_cutscene() {
            continue;
        }
        let mut hit = Hit::new(h.amount, "a damage plate", h.damage_type);
        let amount = hit.on(u, &game.0, &difficulty);
        if amount <= 0.0 {
            continue;
        }
        let key = (h.victim, h.by.clone());
        let first = !victims.0.contains_key(&key);
        let middle = u.position + Vec3::Y * (GROUND + BODY_HEIGHT * 0.5);
        if first || amount >= u.health {
            if first {
                hit.report(u, &game.0, now);
            }
            // (no push: the record's is 0, FUN_00222f60's; and no blood: take15 shows none on Tex
            // through the plate's second, 7.25-8.3 s, as the demo's burning liquids draw none)
            let bled = u.blood.len();
            hurt(u, &game.0, amount, HURT_CHATTER, Vec3::ZERO, middle, -1);
            u.blood.truncate(bled);
        } else {
            u.health -= amount;
        }
        let entry = victims.0.entry(key).or_insert((0.0, now));
        entry.0 += amount;
        entry.1 = now;
        if log {
            println!("t {now:.2}: {} hurts {} by {:.2} this frame ({:.2} so far, from {:.1}): {:.1} / {:.0}", h.by, unit_name(u), amount, entry.0, h.from,
                     u.health, u.max_health);
        }
    }
}
