//! Squad commands, the AI's side (#318, the first slice of #33; todo/33-squad-commands leads §1,
//! §3, §5.2-§6): what each squad member does by its brain's command mode (play_brain.rs
//! `Command`, brain+0x58), which the Squad Command menu's orders set (FUN_000c1280; the menu
//! itself is #319, the order lines #320). Every function cited is in viewer/docs/code-map.md.
//!
//!  - **The modes.** The squad starts in FOLLOW (the brain's constructor) with combat and cover
//!    allowed. An order changes the mode with FUN_00018040: the allow flags by the mode
//!    (FUN_00018000: WINGMAN sets allow_combat and allow_cover, FOLLOW, GUARD and MOVE_OUT clear
//!    them), the goals dropped and the mode's base goal drawn again from its personality (the
//!    type's `<ai>` names one per mode: the four's follow h_169b2725, guard h_155e52d6, moveout
//!    h_0f69b8b2, the wingman h_ea20a17d their empty wingman ones inherit).
//!    - **Stand Ground** (GUARD): GOAL_IDLE_VARY with the guard personality's only pause,
//!      PAUSE_CROUCH: it kneels where it stands and stays (`guard`).
//!    - **Fire At Will** (WINGMAN) and **Cover Me** (FOLLOW): GOAL_WINGMAN_FOLLOW, at the code's
//!      8, 10, 7, 7, 4 m for WINGMAN against 1.4 times them for FOLLOW (FUN_00020b10). The demo's
//!      follow is still its own formation (play.rs `squad_ai`, FORMATION: #331 is the game's
//!      state machine for the squad, #243 for the marines), so FOLLOW keeps it as it was and
//!      WINGMAN closes it in by 1 / 1.4
//!      (`spacing`).
//!    - **Move To** (MOVE_OUT): GOAL_MOVE_OUT walks to the point at the dash, and within 2 m it's
//!      GUARD, with the line e635ecc6 (`move_out`). The game's route over the world points is
//!      #242's: here a straight walk, passing the controlled character when it's in the way (the
//!      game's route round a squadmate, FUN_00041870 -> FUN_000594a0), given up (GUARD,
//!      e635ecc6, as a failed walk is, 0x46256) when it stalls.
//!  - **Holding fire.** Without allow_combat the attack check refuses to fire (FUN_00024f80 at
//!    0x250c5, after raising the gun): such a member turns to its target, gun up, and holds fire
//!    (`gate`). An event whose entry in the mode's personality has the quick goal h_f0233972
//!    grants combat for 10 s (play_brain.rs `GRANT_TIME`), each of its own shots starting the
//!    10 s again: being hit (EVT_DAMAGED), shot at (EVT_SHOT_AT), being a hostile brain's target
//!    (EVT_SQUAD_IMA_TARGET, play_brain.rs `tell_targets`), or a squad mate firing within the
//!    member's h_e3d8aeef, 20 m (EVT_SQUAD_MATE_FIRING, `mate_firing`), in FOLLOW, GUARD and
//!    MOVE_OUT alike. So after Cover Me the squad holds fire until an enemy picks one of it out,
//!    it's fired on or one of the squad (the player too) opens up; a fresh squad (combat allowed
//!    from the start) and Fire At Will engage at once.
//!  - **The hand-over** (`hand_over`, FUN_0005cc20, `taken`, FUN_000c8e80, and `released`,
//!    FUN_00018230): handed from a character more than 50 m from the one taken, the one left
//!    stands its ground (GUARD) and the members following it hold too within 5 m of where it is,
//!    else move out to it; the one taken in MOVE_OUT is set to GUARD for when control passes on
//!    again; the one left has its allow flags set by its mode again (in FOLLOW: holding fire).
//!  - **Toggle Ability** (#333: GOAL_TOGGLE_COMBAT_SKILL, 127, which the menu pushes over the
//!    member's goal, `toggle_ability`): it sends the skill's toggle, the command V sends, every
//!    frame until the skill has switched (`toggle_skill`), the member standing meanwhile (Tex and
//!    Flint get up first). The skill then runs under the AI as under the player (play_skills.rs:
//!    its drain, its ends at 0 stamina, at death and its own, its look); no personality switches
//!    it again (#35 leads §5: none uses goal 127), so it stays on till one of those or the next
//!    order. The mode is left as it is.
//!
//! Test hooks: `BF_TEST_ORDER=<member>,<mode>[,<x>,<z>][@<s>][;...]` gives orders without the
//! menu (`test_orders`; Toggle Ability without the menu is play_skills.rs's
//! `BF_TEST_SQUAD_SKILL`); `BF_ORDER_LOG=1` logs the mode changes, grants, arrivals and the Toggle
//! Ability goal's push and end, `=2` also a row of the squad's state ten times a second (the
//! takes' watch logs' columns).

use super::*;
use bf_viewer::bf::personality::{self, EVT_IDLE_VARY_DATA, EVT_SQUAD_MATE_FIRING, GOAL_PAUSE_CROUCH, QG_ALLOW_COMBAT, QG_ALLOW_COVER};
use brain::{Mode, ALLOW_COMBAT, ALLOW_COVER};

/// The test hook and the log run beside play.rs's `update_player`, which runs the modes
/// (`update`, `gate`, `think`, `mate_firing`) and `squad_control` the hand-over's.
pub fn plugin(app: &mut App) {
    app.add_systems(Update, test_orders.after(read_input).before(update_player).run_if(in_state(AppState::Playing)))
        .add_systems(Update, watch.after(update_player).run_if(in_state(AppState::Playing)));
}

/// How much closer WINGMAN's follow keeps than FOLLOW's: FUN_00020b10's distances are 8, 10, 7,
/// 7 and 4 m, and 11.2, 14, 9.8, 9.8 and 5.6 m (1.4 times them) when its goal+0x18, the brain's
/// mode, is PCMD_FOLLOW. The demo's formation (play.rs FORMATION, SLOT_START, SLOT_DASH) was
/// measured from the squad in its default mode, FOLLOW, so in WINGMAN its places and the
/// distances it sets off and dashes at are those times 8 / 11.2. (What the five distances do in
/// the game's follow: code-map.md's FUN_00020b10 row; the demo's follow doesn't run them, #331.)
const WINGMAN_SPACING: f32 = 8.0 / 11.2;
/// GUARD's kneel, this long after the mode change (s): a fit. The takes' Hawk began her
/// stand2crouch 0.24 s of game time after Stand Ground (take07) and 0.52 s after arriving at a
/// Move To point (take08), against the follow's kneel after standing still a while
/// (play.rs KNEEL_AFTER). The game draws the guard personality's one pause, PAUSE_CROUCH, as
/// GOAL_IDLE_VARY starts; what the delay is made of (the goal machine's turn, the motion's
/// blend) isn't traced.
const GUARD_KNEEL: f32 = 0.3;
/// MOVE_OUT's point is reached within this (m, across): GOAL_MOVE_OUT's update (FUN_00045fa0)
/// walks the route with FUN_00026d80 at 2.0. Take08: Hawk stopped 1.9-2.0 m short of the point.
const MOVE_OUT_ARRIVE: f32 = 2.0;
/// MOVE_OUT walks the last bit (m): play.rs `squad_ai`'s own `dist < 2.5`. Take08's Hawk slowed
/// from the dash to a run 5.6 m out and to a walk 2.8 m out (her locomotion's, not the goal's:
/// it asks for the dash all the way, FUN_0004d970 mode 4).
const MOVE_OUT_WALK: f32 = 2.5;
/// MOVE_OUT's walk is given up when it hasn't come this much closer (m) in STALL_TIME s, after
/// STALL_GRACE s in the mode (a kneeling member stands up first: 0.77 s in take08): the demo's
/// stand-in for a failed route (play_brain.rs's walk uses the same 0.1 m in 1 s), which the game
/// ends in GUARD with e635ecc6 (FUN_00045fa0 at 0x46256).
const STALL_PROGRESS: f32 = 0.1;
const STALL_TIME: f32 = 1.0;
const STALL_GRACE: f32 = 1.0;
/// MOVE_OUT passes the controlled character with this much room (m) when its straight way would
/// take it closer: the demo's stand-in, until #242's routes, for the route round a squadmate in
/// its way. FUN_00041870 (from the AI update) finds one within 8 m whose direction and the
/// member's way have a dot over 0.75, and gets a route round it (FUN_000594a0); only with no route
/// and a player's character in the way (FUN_00112800) does the member wait, counting brain+0xdc,
/// and after 7 s in MOVE_OUT it's GUARD with h_12d51a1c (no block: silent). In open ground there's
/// a route, so take08's Hawk ran past Tex at 1.5 m without a pause: the wait isn't done. The
/// room: two bodies' radii (play.rs BODY_RADIUS) and 0.4 m, the demo's.
const PASS_CLEAR: f32 = 1.2;
/// The hand-over's far case (FUN_0005cc20): the character left and the one taken more than this
/// apart (m; 2500, its square, at 0x3a5348), and a follower of the one left within HOLD_NEAR m
/// (25 at 0x3a5030) of its world point holds there rather than moving out to it.
const HAND_OVER_FAR: f32 = 50.0;
const HOLD_NEAR: f32 = 5.0;
/// The line on reaching the Move To point (GOAL_MOVE_OUT's end, FUN_00045fa0 through the gate
/// FUN_00140a60, play_chatter.rs `say`): e635ecc6, a block in all four chatter files, probability
/// 100, no replay delay, priority 6, a 3 s wait, heard at any distance (todo/36-squad-voice
/// leads.md's tag table). Take08: Hawk's gate call 0.15 s before the watch log shows her GUARD,
/// her sound 0fc11d92 0.3 s after.
const LINE_ARRIVED: u32 = 0xE635_ECC6;
/// The base goal each mode draws (its personality's EVT_BASE: the GOAL_ name table 0x3bcbb0),
/// for the log: GOAL_WINGMAN_FOLLOW 75, GOAL_IDLE_VARY 2, GOAL_MOVE_OUT 72.
fn base_goal(mode: Mode) -> (i64, &'static str) {
    match mode {
        Mode::Guard => (2, "IDLE_VARY"),
        Mode::MoveOut => (72, "MOVE_OUT"),
        _ => (75, "WINGMAN_FOLLOW"),
    }
}

/// BF_ORDER_LOG's level: 0 off, 1 the events, 2 the 10 Hz rows too.
fn log_level() -> u8 {
    std::env::var("BF_ORDER_LOG").ok().map_or(0, |v| v.trim().parse().unwrap_or(1))
}

/// The scale of the follow's formation in `mode` (see WINGMAN_SPACING): 1 in FOLLOW.
pub fn spacing(mode: Mode) -> f32 {
    if mode == Mode::Wingman { WINGMAN_SPACING } else { 1.0 }
}

/// The personality `u`'s brain runs: its type's `<ai>` one for its mode (`TypeAi::personalities`,
/// in the PCMD_ order), 0 for a type with none.
fn personality_of(u: &Player, game: &Game) -> u32 {
    game.character_type(u.kind).map_or(0, |t| t.ai.personalities[u.command.mode.index()])
}

/// The idle pauses `u` draws in its mode (its personality's EVT_IDLE_VARY_DATA entry, play.rs
/// `squad_ai`): FOLLOW's PAUSE_CROUCH 15 s, PAUSE_CAUTIOUS 12 s and PAUSE 6 s, WINGMAN's first
/// two (the table squad.md had for the squad), GUARD's PAUSE_CROUCH alone.
fn pauses(u: &Player, game: &Game) -> Vec<(i64, f32, f32)> {
    game.personality_entry(personality_of(u, game), EVT_IDLE_VARY_DATA).map(|e| e.pauses()).unwrap_or_default()
}

/// One frame of `u`'s command (an AI squad member's, before it thinks): the events posted to it
/// since the last one, each through its mode's personality (an entry with a priority over 0:
/// the quick goals QG_ALLOW_COMBAT, a 10 s grant, and QG_ALLOW_COVER; FUN_0003a4a0), then its
/// clocks (`Command::tick`: the grant running out clears allow_combat). `now`: the log's clock.
pub fn update(u: &mut Player, game: &Game, dt: f32, now: f32) {
    let log = log_level() > 0;
    let events = std::mem::take(&mut u.command.events);
    if events != 0 {
        let p = personality_of(u, game);
        for ev in (0..64).filter(|e| events & (1u64 << e) != 0) {
            let Some(e) = game.personality_entry(p, ev).filter(|e| e.priority > 0.0) else { continue };
            if e.has_quick(QG_ALLOW_COMBAT) {
                let was = u.command.may_fire();
                u.command.grant_combat();
                if log && !was {
                    println!("t={now:.2} order {}: {} grants combat for {} s (personality h_{p:08x}, mode {})", unit_name(u), event_name(ev),
                             brain::GRANT_TIME, u.command.mode.name());
                }
            }
            if e.has_quick(QG_ALLOW_COVER) {
                u.command.allow_cover();
            }
        }
    }
    if u.command.tick(dt) && log {
        println!("t={now:.2} order {}: the grant runs out, allow {:x}{}", unit_name(u), u.command.allow,
                 if u.command.may_fire() { "" } else { ": holds fire" });
    }
}

/// An event's name for the log (the EVT_ name table 0x3bdb98).
fn event_name(ev: i64) -> String {
    match ev {
        personality::EVT_DAMAGED => "EVT_DAMAGED".into(),
        personality::EVT_SHOT_AT => "EVT_SHOT_AT".into(),
        personality::EVT_SQUAD_IMA_TARGET => "EVT_SQUAD_IMA_TARGET".into(),
        EVT_SQUAD_MATE_FIRING => "EVT_SQUAD_MATE_FIRING".into(),
        _ => format!("event {ev}"),
    }
}

/// The attack check's allow_combat test (FUN_00024f80, 0x250c5) on an AI squad member's `target`
/// (a hostile's chest it would fire at) and `toward` (where it turns with nothing to fire at):
/// without allow_combat it fires at nothing and turns to the target instead, the gun up (the check
/// raises it, FUN_00123ea0 at 0x2509c, before refusing). Anyone else's pass as they are.
pub fn gate(u: &Player, target: Option<Vec3>, toward: Option<Vec3>) -> (Option<Vec3>, Option<Vec3>) {
    if u.in_squad() && u.brain.is_none() && !u.command.may_fire() {
        (None, target.or(toward))
    } else {
        (target, toward)
    }
}

/// One frame of an AI squad member `m`'s goal in its mode (in place of play.rs `squad_ai`, which
/// is FOLLOW's and WINGMAN's: `leader`, `slot`, `heading` are its), firing at `target` and turning
/// to `toward` as `gate` left them; or, while the Squad Command menu's Toggle Ability has its goal
/// on top, that goal alone (`toggle_skill`).
#[allow(clippy::too_many_arguments)]
pub fn think(m: &mut Player, leader: &Player, slot: usize, heading: f32, dt: f32, target: Option<Vec3>, toward: Option<Vec3>, game: &Game) {
    if toggle_skill(m, dt) {
        return;
    }
    match m.command.mode {
        Mode::Guard => guard(m, dt, target, toward),
        Mode::MoveOut => move_out(m, leader, dt, target, toward),
        _ => {
            let pauses = pauses(m, game);
            squad_ai(m, leader, slot, heading, dt, target, toward, &pauses);
        }
    }
}

/// GUARD (Stand Ground): GOAL_IDLE_VARY where it stands, its one pause PAUSE_CROUCH (the guard
/// personality h_155e52d6's EVT_IDLE_VARY_DATA: 15 s, drawn again): it kneels GUARD_KNEEL after
/// the mode change and stays down (Brutus stands, as in the follow: play.rs `squad_ai`); one
/// kneeling already (in the follow's crouch pause, or the controlled character left crouched in
/// the hand-over's far case) stays down, its pause carried into the crouch, keeping the way it
/// faces. ENEMY_SEEN draws GOAL_ATTACK_GUARD_AND_COVER: it fires from where it is once
/// combat is allowed (`gate`). Its personality takes no dodge (EVT_DAMAGED_BY_PC and
/// EVT_SHOT_BLOCKED_BY_FRIEND have negative priorities there): it doesn't step out of the
/// leader's line of fire nor sidestep his hits.
fn guard(m: &mut Player, dt: f32, target: Option<Vec3>, toward: Option<Vec3>) {
    m.move_input = Vec2::ZERO;
    m.sprint = false;
    m.walk = false;
    m.jump_pressed = false;
    m.dodge_pressed = false;
    m.dodge_request = None;
    m.still_time += dt;
    m.idle_pause = GOAL_PAUSE_CROUCH;
    m.crouch_wanted = m.data_slot() != 0 && (m.crouch_wanted || m.command.since >= GUARD_KNEEL);
    // (its own way: its "camera" along its facing, level)
    m.cam_yaw = m.yaw;
    m.cam_pitch = 0.0;
    m.face_yaw = Some(m.yaw);
    let turned = target.is_none() && toward.is_some_and(|p| face_toward(m, p));
    m.aim = target.is_some() || turned;
    ai_fire(m, target, dt);
    m.switch_pressed = false;
    m.reload_pressed = false;
    m.throw_held = false;
    m.next_surface = false;
}

/// The Squad Command menu's Toggle Ability to `u` (FUN_000c1280's order 5 for a member with a
/// brain, 0xc1591): GOAL_TOGGLE_COMBAT_SKILL (127) pushed by FUN_00024540 in its mode 2, on top of
/// the goal stack and started at once, over whatever the member was doing (#35 take18: Hawk's goal
/// 15 ATTACK_GUARD_AND_COVER, then 127 from the order to the switch, then 15 again). Its start
/// (0x36e80, the goal holder's vtable +0x688, which the goal table FUN_00012b40 gives goal 127)
/// keeps the skill's state (`Command::toggle`: its on flag with an owner, brain+0x3c4 bit 6) and,
/// for a skill whose toggle wants its character standing (its +0x11c, `skills::
/// stands_to_toggle`: Tex's and Flint's), has the brain stand (0x3f0e0 with 0 clears brain+0x3cc,
/// the crouch wanted: `crouch_wanted`). Its other branch, for a skill whose +0x10c says it needs
/// the brain's target (its position kept at skill+0xc, the start refused when 0x34b20 says so
/// within 20 m; read no further), is none of the four skills' (0x20e200). No mode change and no
/// lines (FUN_000c1630 is for orders 0-3). The dead, those out of the world and a type without a
/// skill take none (the selection refuses the dead, FUN_000c8470; the four always have one). A
/// second order while one runs keeps the state again. `now`: the log's clock. True when pushed.
pub fn toggle_ability(u: &mut Player, now: f32) -> bool {
    if u.dead || u.absent || !u.in_squad() || u.skill.kind.is_none() {
        return false;
    }
    u.command.toggle = Some(u.skill.on);
    let stands = skills::stands_to_toggle(u);
    if stands {
        u.crouch_wanted = false;
    }
    if log_level() > 0 || std::env::var("BF_SKILL_LOG").is_ok() {
        println!("t={now:.2} order {}: GOAL_TOGGLE_COMBAT_SKILL (127) pushed, the skill {}{}", unit_name(u), if u.skill.on { "on" } else { "off" },
                 if stands { ", standing up first" } else { "" });
    }
    true
}

/// GOAL_TOGGLE_COMBAT_SKILL's update (0x36f60, the goal holder's vtable +0x690), each frame its goal
/// is on top of `m`'s stack (`Command::toggle`). The goal machine (FUN_000377b0) runs the top goal
/// alone, and this one neither moves nor fires: the member stands where it is (`hold`), kneeling
/// still if it knelt (take18's Hawk: motion state 6 from the order to the switch), its mode's goal
/// waiting under it. True while it runs. In the code's order:
///  - the skill's +0x118 would send the toggle and end it: no for the four (0x20e200);
///  - a skill that wants its character standing (+0x11c, Tex's, Flint's) waits while the character
///    kneels or crouch-walks (char+0x1f8, motion states 6 and 7: `Action::Crouched`), sending
///    nothing, for the stand-up its start asked for;
///  - the skill's state not the one its start kept: done (it returns 2 and the machine pops it;
///    the mode's goal goes on this frame);
///  - else CT_TOGGLE_SPECIAL, the command V sends (the character's vtable +0x15c with 8:
///    `Skill::ordered`), every frame until the skill has switched: a toggle refused (Tex's or
///    Hawk's clip under way, a reload, under 10% to switch on: the member stands until its stamina
///    is back to it) is asked again. No time limit was found (the push's -1.0 at the goal
///    record's +0x8c, read nowhere traced): inferred.
///
/// The AI doesn't run while the game is paused, nor this: the menu pushes it as the order is given
/// and its fade-out holds the game 0.3 s more, so the first command comes as the game runs again
/// (#35 take18: goal 127 from 7.09 s, the order at 7.0; Hawk stealthed at 8.30, the fade and her
/// clip's 0.83 s event after it).
fn toggle_skill(m: &mut Player, dt: f32) -> bool {
    let Some(kept) = m.command.toggle else { return false };
    if paused() {
        hold(m, dt);
        return true;
    }
    let waits = skills::stands_to_toggle(m) && matches!(m.action, Action::Crouched);
    if !waits && m.skill.on != kept {
        m.command.toggle = None;
        if log_level() > 0 || std::env::var("BF_SKILL_LOG").is_ok() {
            println!("t={:.2} order {}: GOAL_TOGGLE_COMBAT_SKILL done, the skill {}", m.sim_time, unit_name(m), if m.skill.on { "on" } else { "off" });
        }
        return false;
    }
    hold(m, dt);
    if !waits {
        m.skill.ordered = true;
    }
    true
}

/// A goal that neither moves nor fires (GOAL_TOGGLE_COMBAT_SKILL's): no move, no fire, the posture
/// as it is (`crouch_wanted` left), facing as it faces; the idle pause's clock left where the
/// mode's goal had it.
fn hold(m: &mut Player, dt: f32) {
    m.move_input = Vec2::ZERO;
    m.sprint = false;
    m.walk = false;
    m.jump_pressed = false;
    m.dodge_pressed = false;
    m.dodge_request = None;
    m.cam_yaw = m.yaw;
    m.cam_pitch = 0.0;
    m.face_yaw = Some(m.yaw);
    m.aim = false;
    ai_fire(m, None, dt);
    m.switch_pressed = false;
    m.reload_pressed = false;
    m.throw_held = false;
    m.next_surface = false;
}

/// MOVE_OUT (Move To): GOAL_MOVE_OUT's walk to the point (FUN_00045fa0), straight here (#242 has
/// the routes): at the dash (FUN_0004d970 mode 4), the last MOVE_OUT_WALK m at a walk. (The game
/// runs instead while brain+0x3c4 bit 10 is set: the character inside one of the level record's
/// +0x114 prisms, FUN_0004cb60 / FUN_000e6e00; what those volumes are isn't read, so the demo
/// dashes. The leads read the bit as "an enemy known": it isn't.) Within
/// MOVE_OUT_ARRIVE m it's GUARD (the full mode change) and says e635ecc6. Given up, GUARD with
/// e635ecc6 too, when the walk stalls; not on into a liquid that burns (play_liquids.rs, as the
/// follow), which stalls it. The controlled character on its way is passed with PASS_CLEAR of room
/// (the game's route round it: the wait FUN_00041870 makes when there's none isn't done, #242).
/// ENEMY_SEEN draws GOAL_ATTACK_MOVE_OUT: the same walk facing the target, firing once combat is
/// allowed. Without a point (none given) the route is to its own place: it's there, GUARD at once.
fn move_out(m: &mut Player, leader: &Player, dt: f32, target: Option<Vec3>, toward: Option<Vec3>) {
    let now = m.sim_time;
    let here = m.position;
    let to = m.command.point.map_or(Vec2::ZERO, |p| p.xz() - here.xz());
    let dist = to.length();
    if dist < MOVE_OUT_ARRIVE {
        arrive(m, now, if m.command.point.is_some() { "reaches" } else { "has no" }, dist);
        guard(m, dt, target, toward);
        return;
    }
    // (past the controlled character when it's on the way)
    let blocker = (!leader.dead && !leader.absent).then(|| leader.position.xz());
    let way = pass_round(here.xz(), here.xz() + to, blocker);
    // the walk's progress: given up after STALL_TIME s without coming closer
    if dist < m.command.best - STALL_PROGRESS {
        (m.command.best, m.command.stalled) = (dist, 0.0);
    } else if m.command.since > STALL_GRACE {
        m.command.stalled += dt;
    }
    if m.command.stalled >= STALL_TIME {
        arrive(m, now, "stalls", dist);
        guard(m, dt, target, toward);
        return;
    }
    m.jump_pressed = false;
    m.dodge_pressed = false;
    m.dodge_request = None;
    m.still_time = 0.0;
    m.idle_pause = 0;
    m.idle_left = 0.0;
    m.crouch_wanted = false;
    // facing its way, or the target (ATTACK_MOVE_OUT faces it as it walks); the step's move is in
    // its "camera"'s frame, so it's made after the fire's turn
    m.cam_yaw = (-way.x).atan2(-way.y);
    m.cam_pitch = 0.0;
    m.face_yaw = Some(m.cam_yaw);
    let turned = target.is_none() && toward.is_some_and(|p| face_toward(m, p));
    m.aim = target.is_some() || turned;
    ai_fire(m, target, dt);
    let shore = world::arena().is_some_and(|a| liquids::burning_ahead(&a, m, way));
    if shore {
        m.move_input = Vec2::ZERO;
        m.sprint = false;
        m.walk = false;
    } else {
        let (fwd, right) = (Vec2::new(-m.cam_yaw.sin(), -m.cam_yaw.cos()), Vec2::new(m.cam_yaw.cos(), -m.cam_yaw.sin()));
        m.move_input = Vec2::new(way.dot(right), way.dot(fwd));
        m.sprint = dist > MOVE_OUT_WALK;
        m.walk = dist < MOVE_OUT_WALK;
    }
    m.switch_pressed = false;
    m.reload_pressed = false;
    m.throw_held = false;
    m.next_surface = false;
}

/// MOVE_OUT's way (a unit vector in x, z) from `here` to `point` past `blocker` (the controlled
/// character, if it's in the world): straight, but with the blocker ahead (before the point)
/// within PASS_CLEAR of the straight line, aimed beside it, PASS_CLEAR off it on the far side of
/// the line, until it's behind. The demo's stand-in for the game's route round it (see
/// PASS_CLEAR).
fn pass_round(here: Vec2, point: Vec2, blocker: Option<Vec2>) -> Vec2 {
    let to = point - here;
    let dist = to.length().max(1e-4);
    let straight = to / dist;
    let Some(b) = blocker else { return straight };
    let off = b - here;
    let (ahead, side) = (off.dot(straight), straight.perp_dot(off));
    if ahead <= 0.0 || ahead >= dist || side.abs() >= PASS_CLEAR {
        return straight;
    }
    let across = Vec2::new(-straight.y, straight.x);
    let beside = b - across * if side >= 0.0 { PASS_CLEAR } else { -PASS_CLEAR };
    (beside - here).normalize_or(straight)
}

/// MOVE_OUT's end (FUN_00045fa0's tail, and its failed walk at 0x46256): the point given up
/// (brain+0x8ac = 0), GUARD by the full mode change, and the line e635ecc6 asked of the chatter
/// gate (play_chatter.rs `say`). `why` and `dist` for the log.
fn arrive(m: &mut Player, now: f32, why: &str, dist: f32) {
    if log_level() > 0 {
        println!("t={now:.2} order {}: {why} the Move To point ({dist:.1} m off, at ({:.1}, {:.1})): GUARD, asks e635ecc6", unit_name(m), m.position.x,
                 m.position.z);
    }
    set(m, Mode::Guard, None);
    m.command.point = None;
    chatter::say(m, LINE_ARRIVED);
}

/// `u`'s mode set to `mode` (`Command::set_mode`, FUN_00018040) and, when it changed, its goals
/// with it: the follow's or the walk's state and the pause under way dropped (the goal stack
/// emptied, the base goal drawn again), so it stands up and goes, but a kneel carries into GUARD's
/// own. True when it changed.
fn set(u: &mut Player, mode: Mode, point: Option<Vec3>) -> bool {
    let kneeling = u.crouch_wanted;
    if !u.command.set_mode(mode, point) {
        return false;
    }
    u.still_time = 0.0;
    u.idle_pause = 0;
    u.idle_left = 0.0;
    u.crouch_wanted = kneeling && u.command.mode == Mode::Guard;
    true
}

/// An order to `u` (FUN_000c1280's part for one member with a brain, orders 0-3: Stand Ground
/// GUARD, Fire At Will WINGMAN, Cover Me FOLLOW, Move To MOVE_OUT with its point): the ordering
/// player's character becomes its leader (FUN_0005bda0: in the demo every member follows the
/// controlled character already), then the mode change, skipped for a mode it has (the menu's
/// `!=` tests for 0-2; MOVE_OUT goes through to the new point). A dead member takes none: the
/// selection refuses it (FUN_000c8470: none or dead, char+0x70 bit 0, or held by another player's
/// controller, or not in slots 1-4 of the squad list), and the demo also passes over one out of
/// the world (`Player::absent`). `now`: the log's clock. True when the mode changed.
pub fn order(u: &mut Player, mode: Mode, point: Option<Vec3>, now: f32) -> bool {
    if u.dead || u.absent || !u.in_squad() {
        return false;
    }
    let was = (u.command.mode, u.command.allow);
    let changed = set(u, mode, point);
    if log_level() > 0 {
        let (g, name) = base_goal(u.command.mode);
        match changed {
            true => println!("t={now:.2} order {}: {} -> {} (goal {g} {name}), allow {:x} -> {:x}{}", unit_name(u), was.0.name(), u.command.mode.name(), was.1,
                             u.command.allow, point.map_or(String::new(), |p| format!(", to ({:.1}, {:.1}, {:.1})", p.x, p.y, p.z))),
            false => println!("t={now:.2} order {}: {} already: nothing changes (allow {:x})", unit_name(u), mode.name(), u.command.allow),
        }
    }
    changed
}

/// The squad's EVT_SQUAD_MATE_FIRING (FUN_0022f060 from the weapon's fire, 0x22f403, for a holder
/// of a squad type: FUN_00045530): each squad member, the controlled character among them, that
/// fired a round since the last frame (its `shots_fired` against `Command::rounds`) posts it to
/// every other squad member within that member's h_e3d8aeef (`TypeAi::mate_radius`, 3D). An AI
/// member's own running grant starts again at its shot (`Command::fired`). After the frame's
/// steps (play.rs `update_player`); the events are taken in the next `update`.
pub fn mate_firing(player: &mut Player, squad: &mut [Player], game: &Game) {
    let mut firers = vec![];
    for (k, u) in std::iter::once(&mut *player).chain(squad.iter_mut()).enumerate() {
        if u.shots_fired == u.command.rounds {
            continue;
        }
        u.command.rounds = u.shots_fired;
        if u.dead || !(u.in_squad() || u.brain.as_ref().is_some_and(|b| b.follows())) {
            continue;
        }
        if k > 0 {
            u.command.fired();
        }
        firers.push((u.who(), u.position));
    }
    for (who, at) in firers {
        for u in std::iter::once(&mut *player).chain(squad.iter_mut()) {
            if u.who() == who || u.dead || u.absent || !u.in_squad() {
                continue;
            }
            let r = game.character_type(u.kind).map_or(0.0, |t| t.ai.mate_radius);
            if u.position.distance(at) < r {
                u.command.post(EVT_SQUAD_MATE_FIRING);
            }
        }
    }
}

/// The hand-over's re-order (FUN_0005cc20, from FUN_000c8f70 at the select widget's countdown):
/// with `old`, the character control leaves, more than HAND_OVER_FAR m from `squad[new]`, the one
/// it goes to, `old` is set to GUARD, and every other living member following it (the demo's AI
/// members all follow the controlled character) in FOLLOW or WINGMAN to GUARD within HOLD_NEAR m
/// of `old`'s world point (char+0x24c: the demo has none, `old`'s place stands in), else MOVE_OUT
/// to it. Then they all follow the one taken (FUN_0005cbc0): the demo's AI follows the controlled
/// character anyway. The lines (by `old`'s mode, the answers) are #320's.
pub fn hand_over(old: &mut Player, squad: &mut [Player], new: usize) {
    let Some(taken) = squad.get(new) else { return };
    let apart = old.position.distance(taken.position);
    if apart <= HAND_OVER_FAR {
        return;
    }
    let (now, at) = (old.sim_time, old.position);
    if log_level() > 0 {
        println!("t={now:.2} order: the hand-over from {} to {} is {apart:.1} m: over {HAND_OVER_FAR} m, {} stands its ground", unit_name(old),
                 unit_name(taken), unit_name(old));
    }
    order(old, Mode::Guard, None, now);
    for (j, u) in squad.iter_mut().enumerate() {
        if j == new || u.dead || u.absent || !u.in_squad() || u.brain.is_some() || !matches!(u.command.mode, Mode::Follow | Mode::Wingman) {
            continue;
        }
        if u.position.xz().distance(at.xz()) <= HOLD_NEAR {
            order(u, Mode::Guard, None, now);
        } else {
            order(u, Mode::MoveOut, Some(at), now);
        }
    }
}

/// The one control has gone to (`u`, the controlled character now): its goals popped
/// (FUN_0008a720 -> FUN_00018150: a Toggle Ability's goal still on goes, the skill as it is), and
/// set from MOVE_OUT to GUARD (FUN_000c8e80, after FUN_0008a720 passes control), so it holds where
/// it's left.
pub fn taken(u: &mut Player) {
    u.command.toggle = None;
    if u.command.mode == Mode::MoveOut {
        let now = u.sim_time;
        order(u, Mode::Guard, None, now);
    }
}

/// The one control has left (`u`, an AI member now): its brain released (FUN_0008a720 ->
/// FUN_00018230), its mode kept and its allow flags set by it again (`Command::released`): left in
/// FOLLOW it holds fire until an event allows it (#33 take02).
pub fn released(u: &mut Player) {
    let was = u.command.allow;
    u.command.released();
    if log_level() > 0 && was != u.command.allow {
        println!("t={:.2} order {}: left by the player in {}: allow {was:x} -> {:x}", u.sim_time, unit_name(u), u.command.mode.name(), u.command.allow);
    }
}

/// One BF_TEST_ORDER entry: who (None: everyone but the controlled character, the menu's
/// "everyone" selection), the mode, MOVE_OUT's point (x, z) and when (s).
#[derive(Clone, Debug)]
struct TestOrder {
    who: Option<String>,
    mode: Mode,
    point: Option<Vec2>,
    at: f32,
}

/// The mode a hook names: its PCMD_ number (0-5) or a name: `guard` / `stand` (Stand Ground),
/// `wingman` / `fire` (Fire At Will), `follow` / `cover` (Cover Me), `moveout` / `move` (Move
/// To), `freewill`, `formation`.
fn mode_named(s: &str) -> Option<Mode> {
    let s = s.trim().to_ascii_lowercase();
    if let Ok(n) = s.parse::<usize>() {
        return Mode::from_index(n);
    }
    Some(match s.as_str() {
        "guard" | "stand" | "standground" | "stand-ground" => Mode::Guard,
        "wingman" | "fire" | "fireatwill" | "fire-at-will" => Mode::Wingman,
        "follow" | "cover" | "coverme" | "cover-me" => Mode::Follow,
        "moveout" | "move" | "move-out" | "moveto" | "move-to" => Mode::MoveOut,
        "freewill" => Mode::Freewill,
        "formation" => Mode::Formation,
        _ => return None,
    })
}

/// BF_TEST_ORDER's entries (`test_orders`); a bad one is logged and skipped.
fn parse_orders(v: &str) -> Vec<TestOrder> {
    let mut out = vec![];
    for entry in v.split(';').map(str::trim).filter(|e| !e.is_empty()) {
        let (body, at) = match entry.split_once('@') {
            Some((b, t)) => (b, t.trim().parse::<f32>().ok()),
            None => (entry, Some(1.0)),
        };
        let parts: Vec<&str> = body.split(',').map(str::trim).collect();
        let mode = parts.get(1).and_then(|m| mode_named(m));
        let xz: Vec<f32> = parts.iter().skip(2).filter_map(|x| x.parse::<f32>().ok()).collect();
        let point = (xz.len() >= 2).then(|| Vec2::new(xz[0], xz[1]));
        match (parts.first().filter(|w| !w.is_empty()), mode, at) {
            (Some(w), Some(mode), Some(at)) if mode != Mode::MoveOut || point.is_some() => {
                let who = (!w.eq_ignore_ascii_case("all")).then(|| w.to_string());
                out.push(TestOrder { who, mode, point, at });
            }
            _ => println!("BF_TEST_ORDER: \"{entry}\" isn't <member>,<mode>[,<x>,<z>][@<s>] (Move To needs its x, z): skipped"),
        }
    }
    out
}

/// BF_TEST_ORDER=<member>,<mode>[,<x>,<z>][@<s>][;...]: each order given at its time (s of the
/// controlled character's clock; 1 s without one) as the Squad Command menu gives it
/// (FUN_000c1280 through `order`), to `<member>` (0-3 or brutus / flint / hawk / tex as
/// BF_TEST_SELECT names one, `me` the controlled character, `all` every member but the
/// controlled character) in `<mode>` (`mode_named`: guard, wingman, follow, moveout or 0-5);
/// Move To's point is x, z on the floor there. E.g. `BF_TEST_ORDER="all,fire@1;all,cover@3"`
/// (take12's Fire At Will, then Cover Me), `BF_TEST_ORDER=hawk,move,-70.8,-79.5@2` (take08's).
/// No lines are said (#320). The hook is read once, at the first frame: each order with whether
/// it's been given (a bad entry is logged that once).
fn test_orders(mut player: ResMut<Player>, mut squad: ResMut<Squad>, mut orders: Local<Option<Vec<(TestOrder, bool)>>>) {
    let orders = orders.get_or_insert_with(|| {
        std::env::var("BF_TEST_ORDER").map_or(vec![], |v| parse_orders(&v).into_iter().map(|o| (o, false)).collect())
    });
    let now = player.sim_time;
    let me = player.character;
    for (o, done) in orders.iter_mut() {
        if *done || now < o.at {
            continue;
        }
        *done = true;
        let slot = o.who.as_deref().map(|w| testsquad::slot_named(w, me));
        if slot.is_some_and(|s| s.is_none()) {
            println!("BF_TEST_ORDER: no squad member \"{}\": skipped", o.who.as_deref().unwrap_or(""));
            continue;
        }
        // (Move To's point: on the floor under 200 m there)
        let point = o.point.map(|p| Vec3::new(p.x, floor_y(p.x, p.y, 200.0, world::Body::Character), p.y));
        let player = &mut *player;
        for u in std::iter::once(&mut *player).chain(squad.0.iter_mut()) {
            let picked = match slot {
                Some(s) => s == Some(u.character),
                None => u.character != me,
            };
            if picked && u.in_squad() {
                order(u, o.mode, point, now);
            }
        }
    }
}

/// BF_ORDER_LOG=2: ten times a second a row of the squad's state, as the takes' watch logs have
/// it (todo/33-squad-commands notes.md): each member's mode, base goal and allow flags, where it
/// is, its distance to the controlled character, and what it's doing (fire; dash, run, walk on
/// the move; standing, aim with the gun up and not firing, kneel or stand). `(you)` marks the
/// controlled character; the speed is over the time since the last row.
fn watch(player: Res<Player>, squad: Res<Squad>, mut last: Local<(f32, HashMap<usize, Vec2>)>) {
    if log_level() < 2 || player.sim_time < last.0 + 0.1 - 1e-4 {
        return;
    }
    let dt = (player.sim_time - last.0).max(1e-3);
    last.0 = player.sim_time;
    let mut row = |u: &Player, me: bool| {
        let (g, name) = base_goal(u.command.mode);
        let speed = last.1.insert(u.character, u.position.xz()).map_or(0.0, |p| p.distance(u.position.xz()) / dt);
        let doing = if u.dead { "dead" } else if u.fire { "fire" } else if speed > 0.3 {
            if u.sprint { "dash" } else if u.walk { "walk" } else { "run" }
        } else if u.aim { "aim" } else if u.crouch_wanted { "kneel" } else { "stand" };
        format!("{}{} m{} g{g}{name} a{:x}{} ({:.1},{:.1}) d{:.1} {doing} {speed:.1}m/s", unit_name(u), if me { "(you)" } else { "" }, u.command.mode.index(),
                u.command.allow & (ALLOW_COMBAT | ALLOW_COVER), u.command.grant.map_or(String::new(), |t| format!(" grant {t:.1}")), u.position.x, u.position.z,
                u.position.distance(player.position))
    };
    let mut rows = vec![row(&player, true)];
    rows.extend(squad.0.iter().filter(|u| u.in_squad()).map(|u| row(u, false)));
    println!("t={:.2} watch | {}", player.sim_time, rows.join(" | "));
}

#[cfg(test)]
mod tests {
    use super::*;
    use brain::Command;

    /// A fresh squad brain is in FOLLOW with combat allowed (the constructor's `| 3`: every take's
    /// fresh start reads a3), and Cover Me to it changes nothing (take11).
    #[test]
    fn a_fresh_member_fights_and_cover_me_changes_nothing() {
        let mut c = Command::default();
        assert_eq!(c.mode, Mode::Follow);
        assert!(c.may_fire());
        assert!(!c.set_mode(Mode::Follow, None));
        assert_eq!(c.allow, ALLOW_COMBAT | ALLOW_COVER);
    }

    /// Fire At Will sets both flags; Cover Me after it clears them (take12's a0), a grant allows
    /// combat for GRANT_TIME, its own shots start it again, and its running out clears it.
    #[test]
    fn cover_me_after_fire_at_will_holds_fire_until_granted() {
        let mut c = Command::default();
        assert!(c.set_mode(Mode::Wingman, None));
        assert_eq!(c.allow, ALLOW_COMBAT | ALLOW_COVER);
        assert!(c.set_mode(Mode::Follow, None));
        assert_eq!(c.allow, 0);
        assert!(!c.may_fire());
        // (a shot of its own with no grant running starts none)
        c.fired();
        assert!(c.grant.is_none());
        c.grant_combat();
        assert!(c.may_fire());
        assert!(!c.tick(6.0));
        c.fired();
        assert_eq!(c.grant, Some(brain::GRANT_TIME));
        assert!(!c.tick(9.9));
        assert!(c.may_fire());
        assert!(c.tick(0.2));
        assert!(!c.may_fire());
        assert!(c.grant.is_none());
    }

    /// In WINGMAN the grant's timer is stopped and running out never clears combat (FUN_0001afe0
    /// skips mode 4); FREEWILL becomes WINGMAN for a squad brain (FUN_00018040).
    #[test]
    fn wingman_keeps_combat_and_freewill_becomes_wingman() {
        let mut c = Command::default();
        c.set_mode(Mode::Guard, None);
        c.grant_combat();
        assert!(c.set_mode(Mode::Freewill, None));
        assert_eq!(c.mode, Mode::Wingman);
        assert!(c.grant.is_none());
        assert!(!c.tick(30.0));
        assert!(c.may_fire());
    }

    /// GUARD and MOVE_OUT clear the flags; MOVE_OUT is set again for each new point.
    #[test]
    fn move_out_takes_each_new_point() {
        let mut c = Command::default();
        let (a, b) = (Vec3::new(1.0, 0.0, 2.0), Vec3::new(5.0, 0.0, 6.0));
        assert!(c.set_mode(Mode::MoveOut, Some(a)));
        assert_eq!(c.allow, 0);
        assert!(c.set_mode(Mode::MoveOut, Some(b)));
        assert_eq!(c.point, Some(b));
        assert!(c.set_mode(Mode::Guard, None));
        assert!(!c.set_mode(Mode::Guard, None));
    }

    /// Events are queued by number for the next update.
    #[test]
    fn events_are_queued() {
        let mut c = Command::default();
        c.post(personality::EVT_DAMAGED);
        c.post(EVT_SQUAD_MATE_FIRING);
        c.post(99);
        assert_eq!(c.events, (1 << 13) | (1 << 44));
    }

    /// BF_TEST_ORDER's entries: a member or `all`, a mode by name or number, Move To's point, the
    /// time (1 s without one); Move To without its point is skipped.
    #[test]
    fn test_orders_parse() {
        let o = parse_orders("all,fire@4.5; hawk,move,-70.8,-79.5@25.5 ;flint,3;brutus,move@2;x");
        assert_eq!(o.len(), 3);
        assert!(o[0].who.is_none() && o[0].mode == Mode::Wingman && o[0].at == 4.5);
        assert_eq!(o[1].who.as_deref(), Some("hawk"));
        assert_eq!((o[1].mode, o[1].point, o[1].at), (Mode::MoveOut, Some(Vec2::new(-70.8, -79.5)), 25.5));
        assert_eq!((o[2].mode, o[2].at), (Mode::Guard, 1.0));
        assert_eq!(mode_named("cover"), Some(Mode::Follow));
        assert_eq!(mode_named("Stand"), Some(Mode::Guard));
        assert_eq!(mode_named("7"), None);
    }

    /// A member kneeling when it's set to GUARD stays down (no stand-up and kneel again); one
    /// standing kneels GUARD_KNEEL after the order; Brutus stays on his feet.
    #[test]
    fn stand_ground_keeps_a_kneel() {
        let dt = 1.0 / 15.0;
        let mut kneeling = Player::new(2);
        kneeling.crouch_wanted = true;
        assert!(set(&mut kneeling, Mode::Guard, None));
        guard(&mut kneeling, dt, None, None);
        assert!(kneeling.crouch_wanted);
        let mut standing = Player::new(2);
        assert!(set(&mut standing, Mode::Guard, None));
        guard(&mut standing, dt, None, None);
        assert!(!standing.crouch_wanted);
        standing.command.tick(GUARD_KNEEL);
        guard(&mut standing, dt, None, None);
        assert!(standing.crouch_wanted);
        let mut brutus = Player::new(0);
        brutus.crouch_wanted = true;
        set(&mut brutus, Mode::Guard, None);
        brutus.command.tick(GUARD_KNEEL);
        guard(&mut brutus, dt, None, None);
        assert!(!brutus.crouch_wanted);
    }

    /// The character the player leaves has its allow flags set by its mode again (FUN_00018230):
    /// FOLLOW's a3 goes a0 (take02), WINGMAN keeps both.
    #[test]
    fn a_member_left_by_the_player_has_its_flags_set_by_its_mode() {
        let mut c = Command::default();
        assert!(c.may_fire());
        c.released();
        assert_eq!(c.allow, 0);
        c.set_mode(Mode::Wingman, None);
        c.released();
        assert_eq!(c.allow, ALLOW_COMBAT | ALLOW_COVER);
    }

    /// Move To past the controlled character (round 3's failure: take08's geometry, Tex nearly on
    /// Hawk's way, made her wait 7 s and stand her ground). Her straight way from her kneel
    /// (-89, -86.4) to the point (-70.8, -79.5) passes Tex at (-83.39, -82.79) 1.39 m off, room
    /// enough: straight on, as take08's Hawk ran past him. With him right on the line the way
    /// bends to pass him PASS_CLEAR off, and runs straight again once he's behind, or off the
    /// line, or beyond the point.
    #[test]
    fn move_to_passes_the_controlled_character() {
        let (point, tex) = (Vec2::new(-70.8, -79.5), Vec2::new(-83.39, -82.79));
        let start = Vec2::new(-89.0, -86.4);
        let straight = (point - start).normalize();
        assert_eq!(pass_round(start, point, Some(tex)), straight);
        // (head on: 6.5 m ahead on the line)
        let ahead = start + straight * 6.5;
        let way = pass_round(start, point, Some(ahead));
        assert!(way.dot(straight) < 0.999 && way.dot(straight) > 0.9);
        let miss = (ahead - start).perp_dot(way).abs();
        assert!((miss - PASS_CLEAR).abs() < 0.05, "passes {miss:.2} m off");
        // past him: straight again; nobody in the way, the point before him, or him off the line:
        // straight
        let past = ahead + straight;
        let on = |way: Vec2, to: Vec2| way.dot(to.normalize()) > 0.9999;
        assert!(on(pass_round(past, point, Some(ahead)), point - past));
        assert!(on(pass_round(start, point, None), straight));
        assert!(on(pass_round(start, start + straight * 3.0, Some(ahead)), straight));
        assert!(on(pass_round(start, point, Some(ahead + Vec2::new(-straight.y, straight.x) * 3.0)), straight));
    }

    /// WINGMAN's formation is FOLLOW's times 8 / 11.2 (FUN_00020b10's two sets).
    #[test]
    fn wingman_spacing() {
        assert_eq!(spacing(Mode::Follow), 1.0);
        assert!((spacing(Mode::Wingman) * 11.2 - 8.0).abs() < 1e-5);
    }

    /// A squad member `character` with the skill of `skill_set` (CSKILL_: 1 Brutus, 2 Flint, 3 Tex,
    /// 4 Hawk), the campaign's rates.
    fn with_skill(character: usize, skill_set: i64) -> Player {
        let mut u = Player::new(character);
        let data = bf_viewer::bf::character::SkillData { skill_set, max_stamina: 100.0, cost: 2.0, recovery: 1.2, stamina_update: true, ..Default::default() };
        u.skill = skills::Skill::new(Some(&data), &bf_viewer::bf::character::CombatSkills::default());
        u
    }

    /// Toggle Ability (#333): the goal keeps the skill's state and sends the toggle every frame
    /// until it has switched, standing the member meanwhile; then it's done and the mode goes on,
    /// unchanged. A dead member or one without a skill takes none.
    #[test]
    fn toggle_ability_sends_until_the_skill_switches() {
        let dt = 1.0 / 15.0;
        let mut brutus = with_skill(0, 1);
        brutus.move_input = Vec2::ONE;
        assert!(toggle_ability(&mut brutus, 0.0));
        assert_eq!(brutus.command.toggle, Some(false));
        for _ in 0..3 {
            assert!(toggle_skill(&mut brutus, dt));
            assert!(std::mem::take(&mut brutus.skill.ordered), "the toggle sent every frame till it switches");
            assert_eq!(brutus.move_input, Vec2::ZERO);
        }
        // (play_skills.rs's step carries it out: on)
        brutus.skill.on = true;
        assert!(!toggle_skill(&mut brutus, dt));
        assert!(!brutus.skill.ordered && brutus.command.toggle.is_none());
        assert_eq!(brutus.command.mode, Mode::Follow);
        // off again the same way, the state kept on
        assert!(toggle_ability(&mut brutus, 0.0));
        assert_eq!(brutus.command.toggle, Some(true));
        let mut dead = with_skill(2, 4);
        dead.dead = true;
        assert!(!toggle_ability(&mut dead, 0.0));
        let mut none = Player::new(1);
        assert!(!toggle_ability(&mut none, 0.0));
    }

    /// Tex's and Flint's skills want them standing (their +0x11c): the goal's start stands them up
    /// and its update sends nothing while they kneel; Brutus's and Hawk's leave the posture (take18's
    /// Hawk kneeling from the order to the switch).
    #[test]
    fn toggle_ability_stands_tex_and_flint_up_first() {
        let dt = 1.0 / 15.0;
        for (character, skill_set, stands) in [(3, 3, true), (1, 2, true), (2, 4, false), (0, 1, false)] {
            let mut u = with_skill(character, skill_set);
            u.crouch_wanted = true;
            u.action = Action::Crouched;
            assert!(toggle_ability(&mut u, 0.0));
            assert_eq!(u.crouch_wanted, !stands, "{character}: the posture");
            assert!(toggle_skill(&mut u, dt));
            assert_eq!(u.skill.ordered, !stands, "{character}: a command while kneeling");
            u.skill.ordered = false;
            u.action = Action::None;
            assert!(toggle_skill(&mut u, dt));
            assert!(u.skill.ordered, "{character}: standing, the command");
        }
    }

    /// The goal goes with the goal stack: a mode change (FUN_00039f00) or the player taking the
    /// member (FUN_00018150); an order to the mode it has keeps it (no change, no reset).
    #[test]
    fn toggle_ability_ends_with_the_goal_stack() {
        let mut u = with_skill(2, 4);
        toggle_ability(&mut u, 0.0);
        assert!(!set(&mut u, Mode::Follow, None));
        assert!(u.command.toggle.is_some());
        assert!(set(&mut u, Mode::Guard, None));
        assert!(u.command.toggle.is_none());
        toggle_ability(&mut u, 0.0);
        taken(&mut u);
        assert!(u.command.toggle.is_none());
    }
}
