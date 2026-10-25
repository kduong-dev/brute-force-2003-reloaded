//! Pass-through rays (issue #238): an instant ray (bullet-type 4, BTYPE_S_BEAM_EFFECT) whose
//! bullet passes through what it hits, from default.xbe's FUN_002317e0 -> FUN_00230860 (the hit
//! loop), FUN_00230400 (the multi-hit ray) and FUN_002318e0 (one hit). The Rail RVR and the Rail
//! CLVR: h_11caf57b true, h_1835f15e (n1) 1, h_fe4f3035 (n2) 500. Every other instant ray stops at
//! its first hit as before (play.rs's shot readers).
//!
//!  - **How many hits** (FUN_00230860, the weapon data's +0x64 / +0x68 / +0x6c): one, or n1 + n2
//!    with the pass flag. Two or more go through the multi-hit ray (FUN_00230400): the world query
//!    FUN_000dd8e0 with its list, from the muzzle (the scoped eye when scoped, as the shot's
//!    origin is) along the ray over the bullet's range in the holder's hands (FUN_002324c0), the
//!    weapons' filter (no blockers; and no liquids, its object-type mask 0x400000), nearest first,
//!    up to n1 + n2 of them (and the collector's 256, 0x2b7af0). Its collector keeps every hit
//!    the ray caster hands it (#287): a closed object is met twice, where the ray goes in and
//!    where it comes out, the terrain's floor once (`Arena::ray_shot_all`, play_bolts.rs
//!    `world_all`); a character once (`units_met`: the loop below skips its other bones).
//!  - **The loop** (FUN_00230860): each hit in turn takes one off a budget: a character not yet
//!    hit (its object's type 3, a list of those met so far) takes one off n2, anything else (an
//!    object, the terrain: the record's +0x44 empty or not a character) one off n1; a character met
//!    again is passed over, neither dealt nor counted (0x230bf9-0x230c14); the hit is dealt
//!    (FUN_002318e0: a loose object's push, then the weapon's damage handler); after the hit that
//!    drives either budget below 0 the loop stops. So a Rail goes through every character on its
//!    line and into the first object, whose way out drives n1 below 0: it stops at the object's
//!    far side, dealt twice, and never reaches a character behind it (#287 take02, the supply
//!    crate; take03, the base gate's leaf; the CLVR's take17). The terrain's floor is one
//!    crossing: a Rail fired into the ground goes on through it into a character below (take04),
//!    and stops at the next surface it meets. One that runs out of hits before a budget does goes
//!    on to its full range. The distance written back (FUN_00230860's `*param_5`) is the stopping
//!    hit's, or the full range: the beam's length (play.rs `projectiles`).
//!  - **Each hit** (FUN_00230400 builds the records; the damage handler 0x22d0b0 through
//!    play_bolts.rs's `deal`): a direct hit, so a roll of its own (FUN_0022c4d0 -> FUN_0022c940,
//!    from the game's random state, `GameRandom`, x the shooter's skill's factor), x the falloff
//!    at its distance over the range (FUN_00223780 with the Damage's mode: none for the Rails),
//!    x the struck part on a character (FUN_00125940), x0.7 on the shooter's own side; then the
//!    victim's step and type factor. The world plays its material's hit effect, debris and sound
//!    and takes the hole; a breakable takes the roll; a Sentry goes off; a body on the ground is
//!    shoved. All in the shot's frame: the game's ray deals its hits as it's cast, where the
//!    demo's other instant rays land theirs at the demo's INSTANT_SHOT speed.
//!
//! Not done: FUN_00230400's x3 on a character's hit while the shooter is scoped (char+0x7a8:
//! no pass-through gun has a scope, dead data); the squad AI's ray blocked by a squad member in
//! the way (FUN_0005beb0, EVT_SHOT_BLOCKED_BY_FRIEND 34: the demo's AI fires through its
//! squadmates, plan §4.2); the push on loose objects (x3 with the pass flag: the demo's loose
//! pickups are knocked by the line, play_bolts.rs `Sweeps`).
//!
//! `BF_SHOT_LOG=1` prints each pass-through ray's hits (what, how far, the budgets left) and where
//! it stops.

use super::*;
use super::bolts::{deal, units_met, world_all, Kind, Strike, Sweeps};
use super::damage::Difficulty;

/// The most hits the multi-hit ray's collector keeps: its hit method 0x2b7af0 takes one while its
/// count (+0x60) is under 0x100. A Rail's n1 + n2 (501) is cut to it.
const MULTI_HITS: i64 = 0x100;

/// The system, after the characters have moved and fired (`update_player`, which leaves these
/// shots alone, `Shot::swept`), before the shots are drawn (`update_weapons`, then `projectiles`,
/// which draws a pass-through ray's beam to the `dist` set here: named, where it held only through
/// `play_sounds`' place in play.rs's chain before #287) and before the grenades (a Sentry struck
/// goes off this frame), the scenery (`ObjectHits`), the pickups (`Sweeps`), the hits' blood
/// (play_fx.rs) and the sounds.
pub fn plugin(app: &mut App) {
    app.add_systems(Update, pass_rays.after(update_player).before(super::grenade::GrenadeSystems).before(super::pickups::PickupSystems)
        .before(super::fx::FxSystems).before(play_sounds).before(update_weapons).after(super::bolts::BoltSystems).run_if(in_state(AppState::Playing)));
}

/// This frame's pass-through rays (`Shot::pass`), each shooter's: every hit along the line dealt
/// in turn until a budget runs out (see the module notes), and the shot's `dist` set to where it
/// stops (its beam's length) with its hits dealt (`hit` off: play.rs `projectiles` only draws it).
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
fn pass_rays(mut commands: Commands, mut player: ResMut<Player>, mut squad: ResMut<Squad>, mut game: ResMut<GameData>,
             mut ale: Option<ResMut<bf_viewer::ale_fx::AleAssets>>, (mut images, mut materials): (ResMut<Assets<Image>>, ResMut<Assets<StandardMaterial>>),
             (mut chance, mut holes, mut mines, mut objects, mut sweeps): (ResMut<GameRandom>, ResMut<super::fx::HoleRequests>,
                                                                           ResMut<super::sentry::MineTargets>, ResMut<super::scenery::ObjectHits>, ResMut<Sweeps>),
             difficulty: Res<Difficulty>, test: Option<Res<super::testmap::TestMap>>, camera: Query<&GlobalTransform, With<MainCamera>>) {
    // (each shooter's pass-through shots: who, its team, whether the controlled character, the
    // shot's place in its list)
    let mut rays: Vec<(Who, u8, bool, usize, Shot)> = vec![];
    for (k, u) in std::iter::once(&*player).chain(squad.0.iter()).enumerate() {
        rays.extend(u.shots.iter().enumerate().filter(|(_, s)| s.pass).map(|(i, s)| (u.who(), u.team, k == 0, i, *s)));
    }
    if rays.is_empty() {
        return;
    }
    let log = std::env::var("BF_SHOT_LOG").is_ok();
    let listener = camera.iter().next().map(|c| c.translation());
    let instant_kill = test.as_ref().is_some_and(|t| t.instant_kill);
    for (owner, team, human, index, s) in rays {
        let data = game.0.weapons.get(&s.weapon).map(|d| d.shots.clone()).unwrap_or_default();
        let (mut walls, mut characters) = (data.object_hits, data.character_hits);
        let most = (walls + characters).clamp(1, MULTI_HITS) as usize;
        let len = s.range;
        // the multi-hit ray: every crossing of the world's surfaces on the line and every
        // character, nearest first, up to n1 + n2 of them (and the collector's MULTI_HITS)
        let mut met = world_all(s.origin, s.dir, len);
        {
            let units: Vec<&Player> = std::iter::once(&*player).chain(squad.0.iter()).collect();
            met.extend(units_met(s.origin, s.dir, len, owner, None, &units, &mines));
        }
        met.sort_by(|a, b| a.t.total_cmp(&b.t));
        met.truncate(most);
        let now = player.sim_time;
        // where it stops: the hit that drives a budget below 0, else its full range
        let mut end = len;
        for m in met {
            match m.what.kind() {
                Kind::Character => characters -= 1,
                Kind::Object | Kind::Terrain => walls -= 1,
            }
            // a direct hit's own roll (FUN_0022c4d0 -> FUN_0022c940), x the shooter's skill
            let (lo, hi) = shot_rolls(&s);
            let roll = lo + (hi - lo) * chance.unit();
            let skill = unit_mut(&mut player, &mut squad, owner).map_or(1.0, |(u, _)| skills::roll_factor(u));
            let strike = Strike { shot: &s, owner, team, human, roll, skill, falloff: s.falloff_at(m.t) };
            let what = deal(&mut commands, &mut game.0, ale.as_deref_mut(), &mut images, &mut materials, &strike, m, s.origin, s.dir,
                            &mut player, &mut squad, &mut chance, &mut holes, &mut mines, &mut objects, &difficulty, listener, instant_kill);
            let stops = walls < 0 || characters < 0;
            if log {
                println!("t={now:.2} ray of {} {:.1} m out: {what} (walls {walls} characters {characters} left){}",
                         game.0.weapons.get(&s.weapon).map_or("?", |d| d.label.as_str()), m.t, if stops { "; it stops there" } else { "" });
            }
            if stops {
                end = m.t;
                break;
            }
        }
        if log && end >= len {
            println!("t={now:.2} ray of {} goes on to its full range, {len:.1} m", game.0.weapons.get(&s.weapon).map_or("?", |d| d.label.as_str()));
        }
        sweeps.0.push((s.origin, s.dir, end));
        // the shot as drawn: as far as it went, its hits dealt
        if let Some((u, _)) = unit_mut(&mut player, &mut squad, owner) {
            if let Some(shot) = u.shots.get_mut(index) {
                shot.dist = end;
                shot.hit = false;
                shot.body = false;
            }
        }
    }
}
