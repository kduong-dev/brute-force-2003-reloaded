//! A character in a liquid (#152): what a liquid does to the bodies in it, as default.xbe does it
//! (todo/146-liquids leads.md §2-§4, answer-spec.md Q1 and Q9, spec.md Part 4; the code map's
//! Levels rows FUN_00221af0-FUN_00222400), checked against #146's takes 01-03 (Brutus walking
//! into sdm_e13's lava, HP 75 and 5000, logged at 20 Hz) and #28's take08 (Tex running into the
//! tutorial's lava pit on STANDARD).
//!
//!  - **Who's in.** The liquid's phantom (FUN_002218e0) reports each hit-part body of a living
//!    character (PID_BONE: the eleven bones of its motion set's `<collision>` list; the movement
//!    capsule never pairs with it, FUN_00169a30) as it enters or leaves the liquid's collision, a
//!    closed volume under the surface. FUN_00221af0 maps the body to its part (FUN_00115000) and
//!    keeps a wade entry per part (FUN_00222310 / FUN_00222400), and FUN_00220e30 counts them
//!    (char+0x104). Here a part is in while the lowest corner of its collision surface, posed as
//!    drawn (`damage::part_lowest`), is inside the volume (`Arena::liquid_inside`); a body without
//!    parts counts as one at its feet.
//!  - **The damage.** Only the liquid type's character rate, h_e0489ea0 (`LIQUID_CHARACTER_RATE`:
//!    lava 30, sludge 50, toxic waste 30, the waters and chloride 0): a frame on which one of a
//!    living character's parts enters queues one hit of the whole rate (0x221b98), any frame with a
//!    part in queues the rate x dt (FUN_00221ec0, 0x2222c7), and the queue holds one hit per
//!    object a frame (FUN_00221a40), so an entering frame deals the rate once. No depth (nothing
//!    measures it), no tick, nothing more for going under: fully under, the parts are still in the
//!    volume and the rate runs on (take03: 30.2 HP/s 2.9-3 m under, alive 9 s).
//!  - **The hit.** Each queued hit goes to the character's damage handler as DTYPE_FLAME, no
//!    attacker (0x221780 -> 0x125600): the difficulty step (STANDARD's squad x0.6: take08's 18 a
//!    hit and 18 a second; SDM none: the takes' 30), Vengar's 0.85 and the type's factor; no hit
//!    part. It floors nobody (take03: thirteen entry hits of 30, Brutus walking on), and draws no
//!    blood but FUN_00124a70's decals, which the entry hits are big enough for and the per-frame
//!    shares never (play.rs `burn`, play_fx.rs).
//!  - Falls and landings are play.rs's: a fall into a liquid hurts by the drop to its bed
//!    (FUN_0012b740 has no liquid check), and the landing's sound is left out while a part is in
//!    one (0x12d842).
//!
//! Test hooks: BF_LIQUID_LOG=1 prints each character's parts in a liquid as the count changes, its
//! entry hits and its hit points each second in one; BF_TEST_WALK=1 makes BF_TEST_GOTO walk, as
//! the takes' half-stick (play.rs `autopilot`).

use super::*;
use bf_viewer::bf::character::LIQUID_CHARACTER_RATE;

/// DTYPE_FLAME, the damage type the liquid's update gives its hits (the record's +4, 0x221780;
/// the DTYPE_ table's 5): what a character type's factors are looked up by (FUN_002232d0).
const DTYPE_FLAME: i64 = 5;

/// A character's hit parts in the liquids: the game's wade entries for them (FUN_00222310 /
/// FUN_00222400) and their count (char+0x104, `in_liquid`), as of the last game frame the
/// liquids ran (`burn_in_liquids`).
#[derive(Clone, Debug, Default)]
pub struct Wading {
    /// each part (its bone; 0 for a body without parts, at its feet) and the liquid it's in (its
    /// index among the level's liquids, `Arena::liquid_inside`)
    pub parts: Vec<(u32, Option<u32>)>,
    /// each part's lowest point at the end of the last frame drawn (`damage::part_lowest`): where
    /// a game frame between two drawn ones finds it (`burn_in_liquids`)
    lows: Vec<(u32, Vec3)>,
    /// time (s) since the last game frame the liquids ran (`TICK`s)
    since: f32,
    /// (BF_LIQUID_LOG: the game time of the last hit points line)
    logged: f32,
}

impl Wading {
    /// How many of its parts are in a liquid (char+0x104).
    pub fn count(&self) -> usize {
        self.parts.iter().filter(|p| p.1.is_some()).count()
    }
}

/// Whether any of `u`'s parts is in a liquid (char+0x104 > 0): what the game's landing tests
/// (0x12d842).
pub fn in_liquid(u: &Player) -> bool {
    u.wading.count() > 0
}

/// The liquids' damage, once the characters have moved and been posed.
pub fn plugin(app: &mut App) {
    app.add_systems(Update, burn_in_liquids.after(update_player).run_if(in_state(AppState::Playing)));
}

/// What the liquids do to each character (see the module notes), a game frame at a time: the
/// liquid's update runs once a game frame (`shots::TICK`, 30 Hz), and one hit a frame is the
/// rule, so a body's parts that enter on one frame give one hit and on two give two (take08 of
/// #28: Tex's fall into the pit, 5 hits of 18 over 0.27 s; the demo the same at 30 Hz steps).
/// A game frame between two drawn ones finds the parts' lowest points between where the two put
/// them (linear), so BF_CAPTURE's 15 Hz steps and a 60 Hz screen deal what the game's 30 Hz does.
/// Nothing while the game's clock is stopped (`frame_dt`).
fn burn_in_liquids(time: Res<Time>, mut player: ResMut<Player>, mut squad: ResMut<Squad>, game: Res<GameData>,
                   difficulty: Option<Res<damage::Difficulty>>) {
    let dt = frame_dt(&time);
    let (Some(a), Some(difficulty)) = (world::arena(), difficulty) else { return };
    if dt <= 0.0 {
        return;
    }
    let log = std::env::var("BF_LIQUID_LOG").is_ok();
    let now = player.sim_time;
    for u in std::iter::once(&mut *player).chain(squad.0.iter_mut()) {
        if u.dead || u.absent || u.stand_in {
            u.wading = Wading::default();
            continue;
        }
        // its parts' lowest points now (a body knocked down keeps where its parts were: the
        // demo's ragdoll has no part bodies to test); without parts, one at its feet
        let lows = if u.knock.is_some() {
            u.wading.lows.clone()
        } else {
            let lows = damage::part_lowest(u, &game.0);
            if lows.is_empty() { vec![(0, u.position + Vec3::Y * (GROUND + u.height.max(0.0)))] } else { lows }
        };
        let was = std::mem::take(&mut u.wading.lows);
        u.wading.since += dt;
        while u.wading.since >= shots::TICK && !u.dead {
            u.wading.since -= shots::TICK;
            // this game frame's place in the drawn frame (1: its end), and the parts there
            let k = (1.0 - u.wading.since / dt).clamp(0.0, 1.0);
            let parts: Vec<(u32, Option<u32>)> = lows.iter().map(|&(bone, low)| {
                let from = was.iter().find(|w| w.0 == bone).map_or(low, |w| w.1);
                (bone, a.liquid_inside(from.lerp(low, k)).map(|l| l.0))
            }).collect();
            game_frame(u, &game.0, &a, &difficulty, parts, now - u.wading.since, log);
        }
        u.wading.lows = lows;
        if log && u.wading.count() > 0 && now - u.wading.logged >= 1.0 {
            u.wading.logged = now;
            println!("t {now:.2}: {} in a liquid, {} parts: {:.2} hp", unit_name(u), u.wading.count(), u.health);
        }
    }
}

/// One game frame of the liquids on `u`, its parts now in `parts`, at game time `at`: per liquid
/// it has a part in, the whole rate if a part entered it this frame (a part in it that wasn't),
/// else the rate's share of the frame (FUN_00221af0, FUN_00221ec0, 0x221610).
fn game_frame(u: &mut Player, game: &Game, a: &world::Arena, difficulty: &damage::Difficulty, parts: Vec<(u32, Option<u32>)>, at: f32, log: bool) {
    let mut liquids: Vec<(u32, bool)> = vec![];
    for &(bone, l) in &parts {
        let Some(l) = l else { continue };
        let entered = !u.wading.parts.iter().any(|&(b, was)| b == bone && was == Some(l));
        match liquids.iter_mut().find(|x| x.0 == l) {
            Some(x) => x.1 |= entered,
            None => liquids.push((l, entered)),
        }
    }
    let before = u.wading.count();
    u.wading.parts = parts;
    for (l, entered) in liquids {
        let Some((kind, rate)) = a.liquid(l).map(|t| (t.kind, t.damage[LIQUID_CHARACTER_RATE])) else { continue };
        if rate <= 0.0 || u.dead {
            continue;
        }
        let mut hit = damage::Hit::new(if entered { rate } else { rate * shots::TICK }, "liquid", DTYPE_FLAME);
        let amount = if entered { hit.land(u, game, difficulty, at) } else { hit.on(u, game, difficulty) };
        if entered || amount >= u.health {
            let point = u.position + Vec3::Y * GROUND;
            burn(u, game, amount, point);
        } else {
            u.health -= amount;
        }
        if log && entered {
            println!("t {at:.2}: {} {} of its parts in liquid {l} (type {kind}): a part in, hit {amount:.2}: {:.2} hp", unit_name(u),
                     u.wading.count(), u.health);
        }
        if log && u.dead {
            println!("t {at:.2}: {} dies in liquid {l} (type {kind}), {} of its parts in", unit_name(u), u.wading.count());
        }
    }
    if log && u.wading.count() != before {
        println!("t {at:.2}: {} {} of its parts in a liquid (was {before}): {:.2} hp", unit_name(u), u.wading.count(), u.health);
    }
}
