//! The brains of the characters the level's triggers spawn (#227, the tutorial's step 8 of #28;
//! viewer/docs/enemies.md, "The level's characters"): what such a character does on its own, read
//! from default.xbe's AI (the brain at char+0x830 and its goal machine FUN_000377b0; every
//! function cited is in viewer/docs/code-map.md, "AI"). The squad's own AI (play.rs `squad_ai`)
//! and the test map's NPCs (play_testtools.rs `npc_ai`) have no brain, but see by the same sensor
//! (`Sensor`, `glance`; #197).
//!
//!  - **The spawn** gives the brain its home (FUN_000dfd90 -> FUN_00017b60, then FUN_002096e0 ->
//!    FUN_00018510): the spawn-trigger's `home-point`; without one, its world point h_131de144,
//!    kept when it's within 5 m of where the copy stands (FUN_0014f7a0; the game picks a nav
//!    point near the copy otherwise, the demo the copy's own spot). With a home-point the brain is
//!    sent home at once (FUN_00018510 posts EVT_RETURN_HOME, 31): GOAL_RETURN_HOME (its update
//!    FUN_00034690) goes there at the pace of the trigger's `locomotion` (`home_motion`), done
//!    within the point's own h_e6564e8a when it has one, else 2 m (FUN_00026d80). The opening's
//!    wall sentry, TUTORIAL_terrorist_5, walks 10.5 m west along the walkway to wpoint1493
//!    (radius 1) through the opening's first shot (take03).
//!  - **The base goal**: what its personality's EVT_BASE entry draws (FUN_00054c40) when nothing
//!    else is on. GOAL_PATROL for h_e176c6ce, the first patrol's two (TUTORIAL_terrorist_2 and
//!    _18) and the base's TUTORIAL_terrorist_6; GOAL_IDLE_VARY for the Red Hand's other
//!    personalities, which stand where they are here (its pauses, cautious looks, crouches and
//!    random walks home are #26's); GOAL_WINGMAN_FOLLOW for a squad-type character on the player's
//!    side (the V-squad's Confed marines, through their wingman personality h_ebe6044c): the squad
//!    AI's follow (play.rs `squad_ai`), from a formation place after the squad's own. A goal whose
//!    start refuses (a patrol with no route) is struck out and the draw repeated (FUN_00054bc0).
//!  - **Patrol** (GOAL_PATROL: start FUN_0002af70, update FUN_0002b490, the walk FUN_00026d80, a
//!    point reached FUN_00026940): the route is the chain of the patrol point nearest the
//!    character (3D, FUN_000394b0): the first head (`<patrol-first-point>`) of that point's iD,
//!    then its forward links until there are none or they come back to the head; it starts at the
//!    nearest point's place on it (the head when that isn't on it) and refuses a route under two
//!    points. It walks there (FUN_0004d970 mode 2: the stick at half, a full walk by
//!    FUN_0012c6c0, the relaxed walk clip, the type's turn-rate), each point reached within 2 m
//!    across (FUN_0002b490 passes 2.0; a point's own h_e6564e8a above 0 instead), then the next;
//!    at a point with no forward or no backward link (an end of the chain) the route is turned
//!    round (FUN_00023320 -> FUN_00038d30) and it walks back. A chain whose last point links on to
//!    the head is walked round and round (FUN_00012650); the tutorial's two are open, walked to
//!    and fro. A point's trigger-object would get an event as it's reached (none in the data the
//!    demo plays: logged).
//!  - **Sight** (the sensor FUN_0004d140 with its fov test 0x3d300, the scan FUN_00044140, the
//!    contact clock FUN_00054950; todo/197-ai-sight/answer.md): a hostile is seen within its
//!    sight distance (the placement's `vision` when its h_11cddd9f says so, else its type's
//!    vision-range: 75 m for the Red Hand, 80 the marines) x 1.15 once it's a contact
//!    (FUN_000481c0) x its visibility, within the placement's field-of-view of the way it looks
//!    (60 degrees either way for the Red Hand, 90 the marines: its aim while it fires, else its
//!    facing), and with nothing of the world between its eyes and the chest (the squad's ray; the
//!    world's pair table casts its own between the two characters' FUN_0004cea0 points,
//!    FUN_00057a30, not matched here: #244). The gun's range plays no part in it (`Sensor`). A
//!    new sighting becomes a contact; a contact unseen 15 s is forgotten (FUN_00054950), and one
//!    whose stealth visibility is 0.2 or under at once (FUN_00044140). The target is the contact
//!    it fires at: kept until unseen 3.75 s (EVT_TGT_LOST_SIGHT, FUN_0002e330) or 10 s
//!    (FUN_0004a780), then the nearest one seen. The squad's AI and the test map's NPCs see by the
//!    same rule (`glance`): their contacts are forgotten as the brain's, one who hits them is a
//!    contact at once (the stand-in for a wingman's GOAL_INVESTIGATE_GUARD), the squad tells
//!    each other of its targets (`broadcast`, FUN_00045090), and with nothing to fire at they
//!    turn to the last place of one seen in the last 3.75 s, gun up (the goals after it: #26).
//!  - **Alert**: with a contact known it stops its walk and fights (its personality's
//!    EVT_ENEMY_SEEN draws an attack goal: GOAL_DEBUG1 for `default`'s line, ATTACK_GUARD /
//!    ATTACK_COVER for the others): it faces the target and fires at it while it's seen and
//!    within the gun's reach (play.rs `ai_fire`), the gun raised; unseen, it faces the target's
//!    last place (the turn toward a known contact, FUN_0002f960). As the game's, it fires through
//!    one of its own side in the line, who takes the shot (the line-of-sight query leaves
//!    characters out, FUN_00057730, and the attack check tests none, FUN_00024f80): a terrorist
//!    behind its buddy keeps firing into it until one of them moves. Its moves while it fights
//!    (pursuit, strafes, cover, and the game's EVT_SHOT_BLOCKED_BY_FRIEND reposition: the attack
//!    goals' draws) are #26's. Once it knows no contact it goes back to its base goal (a patrol
//!    from the point nearest it), home first if a shot sent it there.
//!  - **Shot, or shot at, while it knows no enemy**: EVT_DAMAGED_NO_ENEMIES (a hit, 0x3e756) or
//!    EVT_SHOT_AT_NO_ENEMIES (a shot passing close) posts, among others, EVT_USER1, whose
//!    personality entry's quick goal h_f765fc45 adds the shooter to its contacts ("QUICK GOAL!!!
//!    QG ADD OBJ TO CONTACTS being set here", the data's note on every Red Hand line), and
//!    EVT_RETURN_HOME. So a shot terrorist turns on its shooter, and after the fight goes home.
//!    (A brain that knows an enemy registers the one who hits it itself, at 0x3ea02; a near miss
//!    tells it nothing then.) The player's shots and the AI's alike; a hit from its own side
//!    posts none of it (0x3e8fc). Its hit reaction is play_enemies.rs's.
//!  - **Aim** (FUN_00024f80's FUN_00024c00, with FUN_00024ae0): each time it fires at a character
//!    the aim is on the chest with the probability of its accuracy, else off it by `up` turned
//!    about the line of fire by up to 45 degrees either way, 1 m long (1.2 m when it points up
//!    more than 0.7, as for a level shot it always does: a miss goes over the head), shrunk by the
//!    distance under 10 m. The accuracy is the placement's h_072837bb (with h_ec5337fb) or the
//!    type's h_12a04758, in percent, plus 0.05 a difficulty step (halved above 0), held within
//!    half and one and a half of itself and at most 1: the Red Hand's 25 is 0.125 on STANDARD. A
//!    character of a squad a player controls takes no difficulty step: the marines' 60 is 0.6.
//!    The gun's own spread takes the placement's bonus too (play_shots.rs).
//!
//!  - **The command mode** (`Command`, #318): every character's brain has one (brain+0x58, the
//!    PCMD_ enum, name table 0x3bd210), which picks its personality from its type's `<ai>` and
//!    what its base goal does. The squad starts in FOLLOW with combat and cover allowed (the
//!    constructor 0x16ff0); the Squad Command menu's orders and the hand-over change it
//!    (play_orders.rs, which runs what each mode does for the squad's AI).
//!
//! Test hooks: `BF_BRAIN_LOG=1` (each brain at its spawn, its goals, routes and points reached,
//! its contacts found, lost and forgotten, who shot at it); `BF_TEST_AIM_HOSTILE=<s>` (from that
//! time the controlled character aims at the nearest hostile it can see and fires: the takes' aim
//! helper, to stage a fight).

use super::*;
use bf_viewer::bf::character_type::{CharacterType, Placement, PlacementAi};
use bf_viewer::bf::level::PatrolPoint;
use bf_viewer::bf::personality;
use bf_viewer::bf::triggers::{Class, Trigger};

/// EVT_BASE (3, the EVT_ name table 0x3bdb9c): the event every brain has, whose personality
/// entry is its base goal.
const EVT_BASE: i64 = 3;
/// The base goals this module runs (the GOAL_ name table 0x3bcbb4): GOAL_IDLE_VARY (2),
/// GOAL_PATROL (4), GOAL_WINGMAN_FOLLOW (75).
const GOAL_IDLE_VARY: i64 = 2;
const GOAL_PATROL: i64 = 4;
const GOAL_WINGMAN_FOLLOW: i64 = 75;
/// A contact is seen this much farther (FUN_000481c0: 1.15, 0x3f933333, when the brain has it in
/// its contact map already).
const ENGAGED: f32 = 1.15;
/// The target is given up after this long unseen (s): EVT_TGT_LOST_SIGHT in the attack goals
/// (FUN_0002e330, 3.75 s with no clear ray), and its entry's unseen time over 10 s (FUN_0004a780).
const LOST_SIGHT: f32 = 3.75;
/// A contact alive and unseen this long (s) is forgotten (FUN_00054950 returns 0 at 15 s).
pub(crate) const FORGET: f32 = 15.0;
/// A contact whose stealth visibility (its char+0x15c) is this or under is forgotten at once
/// (FUN_00044140, 0x3a4f90).
const STEALTH_FORGET: f32 = 0.2;
/// A walk's point is reached within this (m, across: FUN_00026d80 tests x and z): the 2.0
/// (0x40000000) GOAL_PATROL's update (FUN_0002b490) and GOAL_RETURN_HOME's (FUN_00034690) pass.
const ARRIVE: f32 = 2.0;
/// The h_131de144 point is the home only within this of the character (m; FUN_0014f7a0 tests 25,
/// its square), else the game takes a nav point near it.
const HOME_NEAR: f32 = 5.0;
/// GOAL_RETURN_HOME's motion from its +0x94, the spawn-trigger's `locomotion` (h_f27d26c9):
/// FUN_00034690 asks FUN_0004d970 for that speed mode when it's over 2, else for mode 2, and
/// FUN_0004d970 takes the goal's own +0x94 instead whenever it isn't 0. Mode 1 scales the stick
/// to 0.01 (no step: it stays where it is, None), 2 halves it (a walk), 3 a run, 4 the dash
/// (FUN_00047a40(1); a run with brain+0x3c4 bit 10, not read).
fn home_motion(locomotion: i64) -> Option<i64> {
    match locomotion {
        1 => None,
        3 => Some(enemies::MT_RUN),
        g if g >= 4 => Some(enemies::MT_DASH),
        _ => Some(enemies::MT_WALK),
    }
}
/// FUN_00024c00: a miss turns `up` about the line of fire by up to this either way (degrees:
/// FUN_00023900(-0.785, 0.785) x 57.2958), is MISS_LONG m long, MISS_HIGH when its y is over
/// MISS_UP, and shrinks by the distance under MISS_NEAR m (x d / 10).
const MISS_ANGLE: f32 = 45.0;
const MISS_LONG: f32 = 1.0;
const MISS_HIGH: f32 = 1.2;
const MISS_UP: f32 = 0.7;
const MISS_NEAR: f32 = 10.0;
/// FUN_00024ae0: the accuracy moves 0.05 a difficulty step (halved above 0), within half and one
/// and a half of itself, at most 1.
const ACCURACY_STEP: f32 = 0.05;
/// A walk is given up on (its point taken as reached) when the walker hasn't come this much closer
/// (m) in STALL_TIME s: a guess, as play_script.rs's walk does, for the brain's route round what's
/// in the way. The game routes over the level's world points (each `<world-point>` lists its
/// neighbours in h_e3439383; a waypoint-trigger, h_10ecb459, marks one valid or not); its route
/// search isn't traced, and the demo walks straight lines.
const STALL_PROGRESS: f32 = 0.1;
const STALL_TIME: f32 = 1.0;

/// A brain's command mode (brain+0x58: the PCMD_ name table 0x3bd210, 12-byte records after the
/// GOAL_ table; -1 INVALID, 6 BASE_MAX / IDLE and 7 SCRIPT_GOAL aren't kept here: the trainer's
/// freeze and the scripts' own goal). Which personality of the type's `<ai>` the brain runs is
/// the mode's (`TypeAi::personalities` in this order). The Squad Command menu's orders
/// (FUN_000c1280): Stand Ground GUARD, Fire At Will WINGMAN, Cover Me FOLLOW, Move To MOVE_OUT.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Mode {
    Freewill,
    /// the squad's start (the brain's constructor, 0x17734)
    #[default]
    Follow,
    Formation,
    Guard,
    Wingman,
    MoveOut,
}

impl Mode {
    /// The mode of PCMD_ number `n`.
    pub fn from_index(n: usize) -> Option<Mode> {
        [Mode::Freewill, Mode::Follow, Mode::Formation, Mode::Guard, Mode::Wingman, Mode::MoveOut].get(n).copied()
    }

    /// Its PCMD_ number: brain+0x58's value, and the place of its personality in the type's list.
    pub fn index(self) -> usize {
        self as usize
    }

    /// Its name in the table (the takes' watch logs print the number).
    pub fn name(self) -> &'static str {
        ["FREEWILL", "FOLLOW", "FORMATION", "GUARD", "WINGMAN", "MOVE_OUT"][self.index()]
    }
}

/// The allow flags (brain+0x464): bit 0 allow_combat, which the attack check needs to fire
/// (FUN_00024f80 refuses at 0x250c5 without it, unless the current goal is flagged, FUN_00022e20:
/// none of the squad's), and bit 1 allow_cover (GOAL_ATTACK_COVER's and the cover goals' starts).
pub const ALLOW_COMBAT: u8 = 1;
pub const ALLOW_COVER: u8 = 2;
/// How long an event's grant of combat lasts (s): the quick goal h_f0233972 starts the timer at
/// brain+0x468 with 10.0 (0x41200000 at 0x3a74b: FUN_000124f0 sets its time left +0x46c and its
/// length +0x474, and +0x470 bit 0, running), and the attack check starts it again with 10.0 at
/// each shot while it runs (0x254ae-0x254cf: its time left over 0, 0x3a4f88). Take12: granted at
/// 14.41 s, Tex's burst to 15.6 s, cleared at 29.06-29.16 s (the watch log's game clock: 11.6 s
/// after the grant).
pub const GRANT_TIME: f32 = 10.0;

/// A brain's command state (#318): its mode (brain+0x58), allow flags (+0x464) and their grant's
/// timer (+0x468), and the Move To point (+0x8ac). Every character carries one; the squad's AI
/// runs by it (play_orders.rs), the level characters' brains don't read it yet (they keep their
/// own personality, `Brain`). Not touched while a player controls the character: its brain idles
/// and keeps its mode for when control passes on (FUN_00018230 keeps it and sets the allow flags
/// by it again, `released`; FUN_000c8e80 resets only MOVE_OUT, play_orders.rs).
#[derive(Clone, Debug)]
pub struct Command {
    /// the mode
    pub mode: Mode,
    /// the allow flags (ALLOW_COMBAT, ALLOW_COVER)
    pub allow: u8,
    /// the grant's time left (s), None while its timer is stopped
    pub grant: Option<f32>,
    /// MOVE_OUT's point (brain+0x8ac; the demo's: the order's x, z on the floor)
    pub point: Option<Vec3>,
    /// the events posted to it since its last update (bit n: EVT_ n), its share of the brain's
    /// event queue (+0x404): play_orders.rs `update` takes them through its personality
    pub events: u64,
    /// time in its mode (s): GUARD's kneel waits on it (play_orders.rs)
    pub since: f32,
    /// MOVE_OUT's walk: the nearest it has come to the point (m) and how long it hasn't come
    /// closer (the demo's stall rule, play_orders.rs `move_out`)
    pub best: f32,
    pub stalled: f32,
    /// the character's rounds fired (`Player::shots_fired`) when last looked at: a change is a
    /// round fired since (the demo's bookkeeping for play_orders.rs `mate_firing`)
    pub rounds: u32,
}

impl Default for Command {
    /// A fresh brain's (the constructor FUN_00016ff0, 0x16ff0-0x17734): mode FOLLOW for a squad
    /// type (6 first, then 1 when the type's +0x1a8 bit 0 is set: the four and the marines; others
    /// get FREEWILL, but only the squad runs by this), and allow_combat and allow_cover set by the
    /// constructor's own `| 3` at its start, not by FUN_00018000 (which would clear them for
    /// FOLLOW): every take's fresh e34 start reads mode 1, allow a3, and take11's members fight at
    /// once from it. The grant's timer stopped.
    fn default() -> Self {
        Command { mode: Mode::Follow, allow: ALLOW_COMBAT | ALLOW_COVER, grant: None, point: None, events: 0, since: 0.0, best: f32::MAX, stalled: 0.0, rounds: 0 }
    }
}

impl Command {
    /// The mode change (FUN_00018040, inlined at its other sites; mode in EAX): skipped when the
    /// brain is in `mode` already, but for MOVE_OUT, which takes the new point. FREEWILL becomes
    /// WINGMAN for a squad brain (+0x3c4 bit 8, every brain this runs for); WINGMAN stops the
    /// grant's timer (+0x46c / +0x474 zeroed, +0x470 = (& 0xfa) | 2); MOVE_OUT stores the point.
    /// Then the allow flags by the new mode (FUN_00018000), and the goal stack and event queues
    /// emptied and the base goal drawn again (FUN_00039f00, FUN_00055b50, FUN_00015fe0,
    /// FUN_0001a4e0: here its walk and pause start over, play_orders.rs `order`). True when it
    /// changed.
    pub fn set_mode(&mut self, mode: Mode, point: Option<Vec3>) -> bool {
        if self.mode == mode && mode != Mode::MoveOut {
            return false;
        }
        let mode = if mode == Mode::Freewill { Mode::Wingman } else { mode };
        if mode == Mode::Wingman {
            self.grant = None;
        }
        if mode == Mode::MoveOut {
            self.point = point;
        }
        self.mode = mode;
        self.allow_by_mode();
        self.events = 0;
        self.since = 0.0;
        self.best = f32::MAX;
        self.stalled = 0.0;
        true
    }

    /// The allow flags by the mode (FUN_00018000): FREEWILL and WINGMAN set allow_combat and
    /// allow_cover (and clear +0x3c4 bit 0); FOLLOW, GUARD and MOVE_OUT clear them; FORMATION
    /// leaves them.
    fn allow_by_mode(&mut self) {
        match self.mode {
            Mode::Freewill | Mode::Wingman => self.allow |= ALLOW_COMBAT | ALLOW_COVER,
            Mode::Follow | Mode::Guard | Mode::MoveOut => self.allow &= !(ALLOW_COMBAT | ALLOW_COVER),
            Mode::Formation => {}
        }
    }

    /// Control passing on from its character (FUN_0008a720 -> FUN_00018230): the mode kept, and for
    /// a squad brain the allow flags set by it again (FUN_00018000 at FUN_00018230's end): one left
    /// in FOLLOW, GUARD or MOVE_OUT holds fire until an event allows it. #33 take02: Tex's a3 went
    /// a0 at the hand-over to Hawk (2.96 s), Hawk's a3 a0 when she was left in turn (6.47 s). The
    /// events posted while it was controlled are dropped with it (its brain idled: the last
    /// frame's would otherwise grant combat a frame after the reset).
    pub fn released(&mut self) {
        self.allow_by_mode();
        self.events = 0;
    }

    /// The quick goal h_f0233972 (FUN_0003a4a0 at 0x3a744): allow_combat set and the grant's timer
    /// started at GRANT_TIME.
    pub fn grant_combat(&mut self) {
        self.allow |= ALLOW_COMBAT;
        self.grant = Some(GRANT_TIME);
    }

    /// The quick goal h_ea874132 (0x3a64e): allow_cover set (no timer).
    pub fn allow_cover(&mut self) {
        self.allow |= ALLOW_COVER;
    }

    /// Its own shot (the attack check's tail, 0x254ae): a running grant starts again at GRANT_TIME.
    pub fn fired(&mut self) {
        if self.grant.is_some_and(|t| t > 0.0) {
            self.grant = Some(GRANT_TIME);
        }
    }

    /// Whether it may fire (allow_combat, FUN_00024f80's test at 0x250c5).
    pub fn may_fire(&self) -> bool {
        self.allow & ALLOW_COMBAT != 0
    }

    /// Queue event `event` (EVT_ n, under 64) for its next update.
    pub fn post(&mut self, event: i64) {
        if (0..64).contains(&event) {
            self.events |= 1 << event;
        }
    }

    /// One frame (`dt` s) of its clocks: the time in its mode, and the grant's timer, which run
    /// out clears allow_combat outside WINGMAN (the AI update FUN_0001afe0, 0x1b1ff-0x1b235: a
    /// squad brain not in mode 4 whose +0x470 has bits 0 and 1, running and run out). True when
    /// the grant ran out this frame.
    pub fn tick(&mut self, dt: f32) -> bool {
        self.since += dt;
        let Some(t) = self.grant else { return false };
        let t = t - dt;
        if t > 0.0 {
            self.grant = Some(t);
            return false;
        }
        self.grant = None;
        if self.mode != Mode::Wingman {
            self.allow &= !ALLOW_COMBAT;
        }
        true
    }
}

/// The brains run inside play.rs's `update_player` (`sense`, `think`, `attacked`, `held`); the
/// plugin adds the aim helper hook, between the frame's input and the characters' step.
pub fn plugin(app: &mut App) {
    // (after the other input hooks, the free camera's input gate before them: it wins over
    // BF_TEST_TRIGGER's fire, and drives the character under the free camera too)
    app.add_systems(Update, aim_hostile.after(read_input).after(ammo::test_trigger).before(update_player).run_if(in_state(AppState::Playing)));
}

/// A level character's brain (`Player::brain`): see the module notes.
pub struct Brain {
    /// its name for the log (the spawn's copy name)
    label: String,
    /// its personality: the placement's wingman one for a follower, else its freewill one
    personality: u32,
    /// its home (brain+0x1ec) and the radius it's reached within (the point's own, else ARRIVE),
    /// whether it heads there first (an EVT_RETURN_HOME waiting: the spawn's home-point, or a
    /// shot while it knew no enemy), and how (`home_motion`: None stays)
    home: Option<Vec3>,
    home_radius: f32,
    go_home: bool,
    home_mt: Option<i64>,
    /// the base goal drawn from EVT_BASE (None: not drawn yet), and the goals refused
    base: Option<i64>,
    refused: Vec<i64>,
    /// the patrol under way: its points (where, the arrival radius, the point's name, whether it
    /// ends the chain) and the one it walks to
    route: Vec<RoutePoint>,
    at: usize,
    /// a squad-type character on the player's side: follows the leader
    follows: bool,
    /// its accuracy in percent (FUN_00024ae0's before the difficulty), and its gun's spread bonus
    /// (FUN_00222b30)
    accuracy: f32,
    pub gun_bonus: f32,
    /// the contact it fires at (the hostiles it knows are its character's, `Player::sensor`)
    target: Option<Who>,
    /// who shot at it since the last frame (registered by the next `sense`)
    attacked: Option<Who>,
    /// its walk's progress: the nearest it has come to the point (m) and how long it hasn't
    /// come closer (s)
    best: f32,
    stalled: f32,
}

/// One point of a patrol route (FUN_00038320's 0x18-byte entry: the position and the point).
#[derive(Clone, Copy, Debug)]
struct RoutePoint {
    at: Vec3,
    radius: f32,
    name: u32,
    /// it has no forward or no backward link: the chain's end (FUN_00026940)
    end: bool,
    /// its trigger-object, 0 none
    trigger: u32,
}

/// A contact (an entry of the brain's map brain+0x358: kind 1, a living enemy character): who,
/// how long it's been unseen (+0x18), where it was last seen (+0x3c).
#[derive(Clone, Copy, Debug)]
struct Contact {
    who: Who,
    unseen: f32,
    last: Vec3,
}

/// An AI's sensor (`Player::sensor`: the object at char+0x1b4, vtable 0x39dce8, which holds the
/// placement's `<ai>` record) and the hostiles it knows (a brain's contact map brain+0x358, kind 1;
/// the squad's AI and the test map's NPCs, which have no brain, keep theirs here too). See the
/// module notes, "Sight", and todo/197-ai-sight/answer.md.
#[derive(Clone, Debug, Default)]
pub struct Sensor {
    /// its sight distance (m; the sensor's +0xa8, 0x10fb90): the placement's `vision` when its
    /// h_11cddd9f is set, else the type's vision-range (75 for e40's Shrikes and the Red Hand, 80
    /// for Brutus, Hawk, Tex and the marines, 105 for Flint); 0 without a type, when the held
    /// gun's range stands in (`reach`: the demo's own)
    pub vision: f32,
    /// its field of view (degrees either way of the way it looks; the fov test 0x3d300): the
    /// placement's field-of-view (char+0x1e0, h_1b2327a0; 60 on the Red Hand, 90 on the marines),
    /// the type's half-fov for a character with no placement (the squad on a map that doesn't
    /// place it, the test map's NPCs of a type: 90 for the squad, 60 or 90 for the rest; the
    /// lead's call for #197, as the game's characters all come from a placement); 0 without a
    /// type: all round (the demo's)
    pub fov: f32,
    /// the hostiles it knows
    contacts: Vec<Contact>,
    /// who hit it since the last frame, for an AI with no brain (`attacked`; registered by the
    /// next `glance`, as a brain's `sense` does its own)
    shot_by: Option<Who>,
    /// the squad broadcast's timer (brain+0x158, `broadcast`): s to its next look
    broadcast: f32,
}

impl Sensor {
    /// Its sight distance and field of view for a character of type `ty` from a placement whose
    /// `<ai>` is `ai` (None: from none), its contacts kept. A spawn's `spawned` sets it, and
    /// play.rs `spawn_unit` again as the model loads (`enemies::State::ai` keeps the placement's).
    pub fn set(&mut self, ty: Option<&CharacterType>, ai: Option<&PlacementAi>) {
        self.vision = ai.filter(|a| a.vision_own).map(|a| a.vision).or(ty.map(|t| t.ai.vision_range)).unwrap_or(0.0);
        self.fov = ai.map(|a| a.field_of_view).or(ty.map(|t| t.ai.half_fov)).unwrap_or(0.0);
    }

    /// How far (m) it sees a fully visible character it doesn't know yet: its sight distance, or
    /// `range` (the held gun's) without one.
    pub fn reach(&self, range: f32) -> f32 {
        if self.vision > 0.0 { self.vision } else { range }
    }

    /// Whether it knows `who` (also the radar's test for a squadmate's knowledge, FUN_000ae470:
    /// play_hud.rs).
    pub fn knows(&self, who: Who) -> bool {
        self.contacts.iter().any(|c| c.who == who)
    }

    /// How many hostiles it knows: the chatter's stand-in for the brain's count of the enemies it
    /// fights (FUN_0001b460, play_chatter.rs `fighting`). A contact killed stays until the next
    /// scan drops it; the brain's dead branch (FUN_00044140 -> FUN_00043470) is read as doing the
    /// same, inferred.
    pub fn known(&self) -> usize {
        self.contacts.len()
    }

    /// The contact `who`, if it knows them.
    fn contact(&self, who: Who) -> Option<&Contact> {
        self.contacts.iter().find(|c| c.who == who)
    }

    /// `who` becomes a contact, seen last at `at` (FUN_00039880: a new sighting, or a shooter
    /// registered); false when it knew them already.
    fn remember(&mut self, who: Who, at: Vec3) -> bool {
        if self.knows(who) {
            return false;
        }
        self.contacts.push(Contact { who, unseen: 0.0, last: at });
        true
    }

    /// A squadmate's broadcast of `who`, last at `at` (EVT_ENEMY_BROADCAST, `broadcast`): a
    /// contact if it wasn't one (the wingman personality's quick goal h_f765fc45, "ADD OBJ TO
    /// CONTACTS"), its last place and unseen time renewed if it was (the demo's stand-in for the
    /// GOAL_WINGMAN_PURSUE the event draws, which heads for it: #26). True when it's new.
    fn told(&mut self, who: Who, at: Vec3) -> bool {
        match self.contacts.iter_mut().find(|c| c.who == who) {
            Some(c) => {
                (c.last, c.unseen) = (at, 0.0);
                false
            }
            None => self.remember(who, at),
        }
    }

    /// Forget every contact: the controlled character's (a player sees for it; it starts afresh
    /// when control passes on).
    pub fn clear(&mut self) {
        self.contacts.clear();
        self.shot_by = None;
    }

    /// Where it turns to with nothing to fire at (`glance`, `think`'s follower): the last place
    /// of the nearest contact seen within the last LOST_SIGHT s from `eye` (the brain's turn
    /// toward a known contact, FUN_0002f960, while the target lives: EVT_TGT_LOST_SIGHT gives it
    /// up after LOST_SIGHT, FUN_0002e330). One shot from behind, or gone behind cover a moment,
    /// is faced so.
    pub fn toward(&self, eye: Vec3) -> Option<Vec3> {
        self.contacts.iter().filter(|c| c.unseen < LOST_SIGHT).map(|c| c.last).min_by(|a, b| a.distance(eye).total_cmp(&b.distance(eye)))
    }

    /// The hostiles `m` (unit `me` of `sight`, whose sensor this is) sees now, their places in
    /// `sight` and distances: FUN_00043000's "in reach, not refused, ray clear". In reach is
    /// FUN_0004d140's: under its sight distance x FUN_000481c0's factor (ENGAGED for a contact,
    /// else 1; its x3 with the gun's weapon-data +0x16c is left out: the field isn't identified) x
    /// the target's visibility. Then the fov test (0x3d300) from its eyes, along its aim while it
    /// fires (its aim record char+0x3e8) or else its facing (vtable +0x2c). Then nothing of the
    /// world (or the test floor's pillars) between its eyes and the chest. The gun's range plays
    /// no part.
    pub fn scan(&self, m: &Player, me: usize, sight: &[Seen]) -> Vec<(usize, f32)> {
        let eye = m.position + Vec3::Y * (GROUND + m.height + AI_EYE);
        let facing = Vec3::new(-m.yaw.sin(), 0.0, -m.yaw.cos());
        let look = m.ai_target.map(|t| t - eye).filter(|v| v.length_squared() > 1e-6).map_or(facing, |v| v.normalize());
        let reach = self.reach(ai_range(m));
        let mut seen = vec![];
        for (i, s) in sight.iter().enumerate() {
            if i == me || !s.target || s.team == m.team {
                continue;
            }
            let d = eye.distance(s.chest);
            if d >= reach * if self.knows(s.who) { ENGAGED } else { 1.0 } * s.visibility {
                continue;
            }
            let to = (s.chest - eye) / d.max(1e-3);
            if self.fov > 0.0 && look.dot(to).clamp(-1.0, 1.0).acos().to_degrees() > self.fov {
                continue;
            }
            if ray_hit(eye, to, d - 0.3).is_none() {
                seen.push((i, d));
            }
        }
        seen
    }

    /// The contacts' clocks for this frame's sightings `seen` (FUN_00054950): one seen has its
    /// unseen time and last place renewed, the rest count up `dt`, and the new sightings become
    /// contacts (the scan's FUN_00039880). Then, as the scan's tail does after them, the forgotten
    /// ones are dropped and returned with why: unseen `forget` s ("unseen"), dead or out of the
    /// world or no longer of another side than `team` ("gone": the scan's dead list,
    /// EVT_TGT_KILLED), or stealthed to STEALTH_FORGET or under ("stealthed", FUN_00044140 at
    /// 0x443f1, every step: so a stealthed Hawk never keeps the 1.15). One knocked down stays
    /// (unseen meanwhile). The new contacts that stay are returned with their distances.
    fn update(&mut self, seen: &[(usize, f32)], sight: &[Seen], team: u8, dt: f32, forget: f32) -> (Vec<(Who, &'static str)>, Vec<(Who, f32)>) {
        for c in self.contacts.iter_mut() {
            match seen.iter().find(|(i, _)| sight[*i].who == c.who) {
                Some(&(i, _)) => (c.unseen, c.last) = (0.0, sight[i].chest),
                None => c.unseen += dt,
            }
        }
        let mut found = vec![];
        for &(i, d) in seen {
            if self.remember(sight[i].who, sight[i].chest) {
                found.push((sight[i].who, d));
            }
        }
        let mut forgot = vec![];
        self.contacts.retain(|c| {
            let s = sight.iter().find(|s| s.who == c.who);
            let why = if c.unseen >= forget {
                "unseen"
            } else if !s.is_some_and(|s| s.alive && s.team != team) {
                "gone"
            } else if s.is_some_and(|s| s.visibility <= STEALTH_FORGET) {
                "stealthed"
            } else {
                return true;
            };
            // (one found this step and forgotten at once isn't news)
            match found.iter().position(|f: &(Who, f32)| f.0 == c.who) {
                Some(k) => _ = found.remove(k),
                None => forgot.push((c.who, why)),
            }
            false
        });
        (forgot, found)
    }
}

/// Whether `u`'s brain knows an enemy (brain+0x3c4 bit 7 clear; false without a brain):
/// play_enemies.rs's damage path posts EVT_DAMAGE_LIGHT / HEAVY then, else
/// EVT_DAMAGED_NO_ENEMIES.
pub fn knows_enemy(u: &Player) -> bool {
    u.brain.is_some() && !u.sensor.contacts.is_empty()
}

/// The squad member a hostile brain has as its target is told so (#318): the target pick
/// (FUN_0004a860, 0x4aadd-0x4aafa) posts EVT_ENEMY_SEEN with the target to its own brain whenever
/// none is queued, and the message handler's case for it (0x3e45a-0x3e49b) posts
/// EVT_SQUAD_IMA_TARGET (43) to that target's brain when it's a character of a squad type
/// (type+0x1a8 bit 0). The follow, guard and moveout personalities grant combat on 43, so a
/// member an enemy has picked out fights back (play_orders.rs `update`) for as long as it's the
/// target. Posted every frame here (its queue keeps one of each), after the frame's sense steps
/// (`sense`'s target); the controlled character's is dropped (its brain idles). Not by one a
/// script holds (`held`: no brain meanwhile, char+0x834 bit 0), nor by the test map's NPCs
/// (BF_TEST_NPC: no brain at all), only by the level's characters.
pub fn tell_targets(player: &mut Player, squad: &mut [Player]) {
    let targets: Vec<Who> = squad.iter().filter(|u| !u.dead && !u.absent && !u.held && u.script_walk.is_empty()).filter_map(target).collect();
    for who in targets {
        let u = if player.who() == who { Some(&mut *player) } else { squad.iter_mut().find(|u| u.who() == who) };
        if let Some(u) = u.filter(|u| u.in_squad() && u.brain.is_none()) {
            u.command.post(personality::EVT_SQUAD_IMA_TARGET);
        }
    }
}

/// The nearest of `seen` (places in `sight` and distances) within `range` (m, the held gun's: the
/// shot's own test; the attack check doesn't test it, FUN_00024f80).
pub fn nearest_within(seen: &[(usize, f32)], range: f32) -> Option<(usize, f32)> {
    seen.iter().copied().filter(|&(_, d)| d < range).min_by(|a, b| a.1.total_cmp(&b.1))
}

/// A shooter at least this visible is registered as a contact by its hit, whatever the range
/// (EVT_DAMAGED at 0x3ea02: its visibility, slot 0, over 0.1, 0x3a4ff0); a fainter one only as
/// FUN_0004d140 lets it (left out: a stealthed Hawk's shot shows her at 1 anyway).
const SHOOTER_SEEN: f32 = 0.1;

/// One frame of the sensor of an AI with no brain (the squad's AI, play.rs `squad_ai`, and the
/// test map's NPCs, play_testtools.rs `npc_ai`; `m` being unit `me` of `sight`). First the one who
/// hit it since the last frame becomes a contact, at any range, if it's at least SHOOTER_SEEN
/// visible. That is the game's registration (0x3ea02) only for a brain that knows an enemy
/// already; a wingman that knows none gets EVT_DAMAGED_NO_ENEMIES (30), which its personality
/// h_ea20a17d maps to GOAL_INVESTIGATE_GUARD (13) and no registration. The demo has no such
/// goal: registering the shooter and turning to it (`toward`) is its stand-in (#26's ground).
/// Then who it sees
/// (`Sensor::scan`, the brain's rule), and its contacts' clocks, forgotten as the brain's are
/// (`Sensor::update`: unseen FORGET s). Returns what it fires at, the nearest contact it sees
/// within the held gun's reach (its place in `sight` and distance; one just forgotten,
/// stealthed, isn't: the game's fire needs the target's entry, FUN_00024f80), and with none,
/// where it turns to (`Sensor::toward`: a contact unseen under LOST_SIGHT s, the target's
/// EVT_TGT_LOST_SIGHT; what the goals do after it is #26's). BF_SIGHT_LOG logs its contacts
/// found and forgotten, at `now` (the log's clock, the controlled character's).
pub fn glance(m: &mut Player, me: usize, sight: &[Seen], dt: f32, now: f32) -> (Option<(usize, f32)>, Option<Vec3>) {
    let log = std::env::var("BF_SIGHT_LOG").is_ok();
    let team = m.team;
    if let Some(who) = m.sensor.shot_by.take() {
        if let Some(s) = sight.iter().find(|s| s.who == who && s.target && s.team != team && s.visibility > SHOOTER_SEEN) {
            if m.sensor.remember(who, s.chest) && log {
                println!("t={now:.2} {}: shot by {who:?}: a contact", unit_name(m));
            }
        }
    }
    let seen = m.sensor.scan(m, me, sight);
    let (forgot, found) = m.sensor.update(&seen, sight, team, dt, FORGET);
    if log {
        for (who, d) in found {
            println!("t={now:.2} {}: a contact: {who:?} at {d:.1} m", unit_name(m));
        }
        for (who, why) in forgot {
            println!("t={now:.2} {}: forgets {who:?} ({})", unit_name(m), if why == "unseen" { "unseen 15 s" } else { why });
        }
    }
    let known: Vec<(usize, f32)> = seen.into_iter().filter(|(i, _)| m.sensor.knows(sight[*i].who)).collect();
    let target = nearest_within(&known, ai_range(m));
    let eye = m.position + Vec3::Y * (GROUND + m.height + AI_EYE);
    (target, if target.is_none() { m.sensor.toward(eye) } else { None })
}

/// Whether `u` has a squad record (char+0x59c): the squad's own members, and a level character
/// following the leader (the marines, `Brain::follows`).
fn in_squad_record(u: &Player) -> bool {
    u.in_squad() || u.brain.as_ref().is_some_and(|b| b.follows)
}

/// The contact `u`'s brain fires at (FUN_0004ab70's pick, `Brain::target`), if it has a brain.
pub fn target(u: &Player) -> Option<Who> {
    u.brain.as_ref().and_then(|b| b.target)
}

/// How often (s) a squad member holding a target tells the others of it, and looks again
/// without one: FUN_00045090 sets brain+0x158 to 2.0 (0x40000000) after a broadcast, 1.0
/// (0x3f800000) after a look with no target, and looks once it's run out (its countdown by the
/// frame's time: inferred, the writer isn't read).
const BROADCAST_EVERY: f32 = 2.0;
const BROADCAST_LOOK: f32 = 1.0;

/// The squad's broadcast of its targets (FUN_00045090, at the tail of every brain's entry walk
/// FUN_000447b0, after the sense step): a member with a squad record whose target pick
/// (FUN_0004ab70) gives one tells every other living member (FUN_00045250 ->
/// FUN_0004dff0: EVT_ENEMY_BROADCAST, 39), which the wingman personality h_ea20a17d maps to
/// GOAL_WINGMAN_PURSUE and the quick goal h_f765fc45 "ADD OBJ TO CONTACTS": `Sensor::told`, so
/// they turn to it (`toward`) and fire once they see it. Its timer: BROADCAST_EVERY while it
/// holds one, BROADCAST_LOOK without. `held`: each unit's target this frame, by its place in
/// `sight` (0 the controlled character, whose "target" is the nearest hostile it sees, as the
/// game's UI character sends these on ENEMY_SEEN; k the squad's k - 1). The controlled character
/// isn't told (a player sees for it). The pursuit itself is #26's. BF_SIGHT_LOG logs each one, at
/// `now`.
pub fn broadcast(player: &mut Player, squad: &mut [Player], held: &[(usize, Option<Who>)], sight: &[Seen], dt: f32, now: f32) {
    let mut posts = vec![];
    for &(k, target) in held {
        let u = if k == 0 { &mut *player } else { &mut squad[k - 1] };
        if !in_squad_record(u) || u.dead {
            continue;
        }
        u.sensor.broadcast -= dt;
        if u.sensor.broadcast > 0.0 {
            continue;
        }
        match target.and_then(|w| sight.iter().find(|s| s.who == w)) {
            Some(s) => {
                posts.push((k, s.who, s.chest));
                u.sensor.broadcast = BROADCAST_EVERY;
            }
            None => u.sensor.broadcast = BROADCAST_LOOK,
        }
    }
    let log = std::env::var("BF_SIGHT_LOG").is_ok();
    for (k, who, at) in posts {
        let from = unit_name(if k == 0 { &*player } else { &squad[k - 1] }).to_string();
        for (j, u) in squad.iter_mut().enumerate() {
            if j + 1 == k || u.dead || u.absent || !in_squad_record(u) {
                continue;
            }
            if u.sensor.told(who, at) && log {
                println!("t={now:.2} {}: told of {who:?} by {from}: a contact", unit_name(u));
            }
        }
    }
}

impl Brain {
    /// Whether it follows the leader (a squad-type character on the player's side): it takes a
    /// formation place (play.rs `update_player`).
    pub fn follows(&self) -> bool {
        self.follows
    }
}

/// Whether BF_BRAIN_LOG is set.
fn log_on() -> bool {
    std::env::var("BF_BRAIN_LOG").is_ok()
}

/// A spawn-trigger's copy `m` gets its brain (play_triggers.rs `spawn_copy`), from the trigger
/// `t` (its home-point h_1b747c8d, its AI's world point h_131de144, its `locomotion`), the level's
/// points, the copy's type and placement, where its feet are and the player's side. `label` names
/// it in the log.
#[allow(clippy::too_many_arguments)]
pub fn spawned(m: &mut Player, t: &Trigger, level: &Level, ty: &CharacterType, placement: Option<&Placement>, feet: Vec3, player_team: u8,
               label: String) {
    // (a point: where, and its own arrival radius)
    let point = |n: u32| (n != 0).then(|| level.objects.iter().find(|o| o.name == n)).flatten().map(|o| (o.transform.w_axis.truncate(), o.radius));
    let attr = |k: u32| t.element.attr(k).and_then(|v| v.as_hash()).filter(|&n| n != bf_viewer::bf::hash::h("")).unwrap_or(0);
    let home_point = point(attr(H_HOME_POINT));
    let ai_point = match &t.class { Class::Spawn { ai_point, .. } => point(*ai_point), _ => None };
    let locomotion = t.element.attr(H_LOCOMOTION).and_then(|v| v.as_i64()).unwrap_or(0);
    // FUN_00018510: a home-point is the home, and the brain is sent there; without one, the world
    // point FUN_00017b60 was given, kept within HOME_NEAR of the copy (FUN_0014f7a0)
    let home_at = home_point.or(ai_point.filter(|p| p.0.xz().distance(feet.xz()) <= HOME_NEAR));
    let home = home_at.map(|p| p.0).or(Some(feet));
    // the walk home arrives within the point's own radius when it has one (FUN_00026d80: over 0,
    // with GOAL_RETURN_HOME's 2.0 passed), else ARRIVE
    let home_radius = home_at.map(|p| p.1).filter(|&r| r > 0.0).unwrap_or(ARRIVE);
    let follows = ty.player_type && m.team == player_team;
    let ai = placement.map(|p| &p.ai);
    // (its freewill one: the placement's, else its type's, as play_enemies.rs `spawned` takes it)
    let personality = if follows { ai.map(|a| a.personalities[4]).filter(|&p| p != 0).unwrap_or(ty.ai.personalities[4]) } else {
        Some(m.enemy.personality).filter(|&p| p != 0).unwrap_or(ty.ai.personalities[0])
    };
    // the sensor's sight distance (0x10fb90) and field of view (0x3d300: the placement's), the
    // placement's `<ai>` kept for the model's load (play.rs `spawn_unit` sets them again)
    m.enemy.ai = ai.cloned();
    m.sensor.set(Some(ty), ai);
    let (vision, fov) = (m.sensor.vision, m.sensor.fov);
    // FUN_00024ae0's accuracy and FUN_00222b30's bonus: the placement's h_072837bb with its
    // h_ec5337fb, else the type's h_12a04758 and -1
    let bonus = placement.and_then(|p| p.accuracy_bonus);
    let accuracy = bonus.filter(|&b| b >= 0).map_or(ty.ai.accuracy, |b| b as f32);
    let gun_bonus = bonus.map_or(shots::ACCURACY_BONUS, |b| b as f32);
    let b = Brain {
        label, personality, home, home_radius, go_home: home_point.is_some(), home_mt: home_motion(locomotion), base: None, refused: vec![], route: vec![], at: 0,
        follows, accuracy, gun_bonus, target: None, attacked: None, best: f32::MAX, stalled: 0.0,
    };
    if log_on() {
        println!("t={:.2} brain {}: home {}{} (locomotion {locomotion}: {}), sight {vision} m, field of view {fov} deg, accuracy {accuracy}%, gun bonus {gun_bonus}, \
                  personality h_{personality:08x}{}", m.sim_time, b.label,
                 home.map_or("none".to_string(), |h| format!("({:.1}, {:.1}, {:.1}) within {home_radius} m", h.x, h.y, h.z)),
                 if home_point.is_some() { " from its home-point: goes there first" } else { "" },
                 match b.home_mt { None => "stays", Some(enemies::MT_RUN) => "runs", Some(enemies::MT_DASH) => "dashes", _ => "walks" },
                 if follows { ", follows the leader" } else { "" });
    }
    m.brain = Some(Box::new(b));
}

/// A spawn-trigger's home-point (h_1b747c8d, +0x80, FUN_002096e0 -> FUN_00018510) and its
/// `locomotion` (h_f27d26c9, named in xmb_names.txt; +0x84, FUN_00018510's 5th argument, which
/// stores it as GOAL_RETURN_HOME's +0x94 at 0x18750: its speed mode).
const H_HOME_POINT: u32 = 0x1B74_7C8D;
const H_LOCOMOTION: u32 = 0xF27D_26C9;

/// `u` was hit by `by` (`hit`), or a shot of `by`'s passed close, the player's or an AI's (play.rs
/// `update_player`). A hit always tells it the shooter: EVT_DAMAGED_NO_ENEMIES (30) with no enemy
/// known, whose EVT_USER1 quick goal adds it, else the damage handler's own registration
/// (0x3ea02). A near miss only with no enemy known (EVT_SHOT_AT_NO_ENEMIES, 87, the same
/// EVT_USER1). With no enemy known before, it goes home after the fight (EVT_RETURN_HOME,
/// posted with them). Its next sense makes the shooter a contact. An AI with no brain (the
/// squad's, the test map's NPCs) learns its shooter from a hit only, EVT_DAMAGED's 0x3ea02 path
/// (`glance`): a near miss tells it nothing. The squad's AI gets the hit as EVT_DAMAGED and the
/// near miss as EVT_SHOT_AT for its command mode's personality (`Command::post`; #318: the
/// follow, guard and moveout personalities grant combat on them, play_orders.rs `update`).
pub fn attacked(u: &mut Player, by: Who, hit: bool) {
    if u.dead {
        return;
    }
    let me = u.who();
    let knows = !u.sensor.contacts.is_empty();
    if u.brain.is_none() {
        if by != me && u.in_squad() {
            u.command.post(if hit { personality::EVT_DAMAGED } else { personality::EVT_SHOT_AT });
        }
        if hit && by != me {
            u.sensor.shot_by = Some(by);
        }
        return;
    }
    let Some(b) = u.brain.as_mut() else { return };
    if by == me || (!hit && knows) {
        return;
    }
    if !knows && b.home.is_some() && !b.follows {
        b.go_home = true;
    }
    b.attacked = Some(by);
}

/// One frame of `m`'s sensor (the brain's sense step FUN_00044480, `m` being unit `me` of `sight`):
/// the hostiles it sees, its contacts and their clocks, its target. The target's place in `sight`
/// and distance when it's seen now and within the gun's reach (its fire may go), as `glance`
/// gives it for the squad.
pub fn sense(m: &mut Player, me: usize, sight: &[Seen], dt: f32) -> Option<(usize, f32)> {
    let mut b = m.brain.take()?;
    let eye = m.position + Vec3::Y * (GROUND + m.height + AI_EYE);
    let log = log_on();
    let team = m.team;
    // the shooter it was told of: a contact now (not one on its own side)
    if let Some(who) = b.attacked.take() {
        if let Some(s) = sight.iter().find(|s| s.who == who && s.target && s.team != team) {
            if m.sensor.remember(who, s.chest) && log {
                println!("t={:.2} brain {}: shot at by {who:?} {:.1} m off: a contact", m.sim_time, b.label, eye.distance(s.chest));
            }
        }
    }
    // who it sees (FUN_00043000: in reach, in the field of view, the ray clear), the contacts'
    // clocks (forgotten unseen FORGET s, gone with the character, at once when stealthed) and the
    // new sightings (`Sensor`)
    let seen = m.sensor.scan(m, me, sight);
    let before = m.sensor.contacts.len();
    let (forgot, found) = m.sensor.update(&seen, sight, team, dt, FORGET);
    if log {
        for (who, why) in forgot {
            println!("t={:.2} brain {}: forgets {who:?} ({})", m.sim_time, b.label, if why == "unseen" { "unseen 15 s" } else { why });
        }
        let known = m.sensor.contacts.len() - found.len();
        for (k, (who, d)) in found.iter().enumerate() {
            println!("t={:.2} brain {}: sees {who:?} at {d:.1} m: a contact ({} known)", m.sim_time, b.label, known + k + 1);
        }
    }
    // the target: kept while it's been seen in the last LOST_SIGHT s, else the nearest seen
    let kept = b.target.and_then(|w| m.sensor.contact(w)).is_some_and(|c| c.unseen < LOST_SIGHT);
    if !kept {
        let was = b.target;
        b.target = seen.iter().filter(|(i, _)| m.sensor.knows(sight[*i].who)).min_by(|x, y| x.1.total_cmp(&y.1))
            .map(|&(i, _)| sight[i].who)
            .or(m.sensor.contacts.iter().min_by(|x, y| x.unseen.total_cmp(&y.unseen)).map(|c| c.who));
        if log && was != b.target {
            println!("t={:.2} brain {}: target {:?}{}", m.sim_time, b.label, b.target,
                     if was.is_some() && m.sensor.contacts.len() == before { " (lost sight of the last)" } else { "" });
        }
    }
    // what it fires at: the target, seen now, within the gun's reach (the shot's own; the attack
    // check doesn't test it, FUN_00024f80)
    let range = ai_range(m);
    // (nobody of its own side in the line stops it: the game's ray leaves characters out,
    // FUN_00057730, and the attack check tests none, FUN_00024f80 / FUN_0004dc10; a friend in the
    // way takes the shot, play.rs `update_player`)
    let out = b.target.and_then(|w| seen.iter().find(|(i, _)| sight[*i].who == w)).copied().filter(|&(_, d)| d < range);
    m.brain = Some(b);
    out
}

/// One frame of `m`'s brain, in place of the squad's AI or the test map NPC's (see the module
/// notes). `leader`, `slot` and `heading`: the squad AI's, for a follower; `target`: the chest it
/// may fire at (`sense`); `patrols`: the level's patrol points; `n`: the difficulty.
#[allow(clippy::too_many_arguments)]
pub fn think(m: &mut Player, leader: &Player, slot: usize, heading: f32, dt: f32, target: Option<Vec3>, patrols: &[PatrolPoint], game: &Game, n: i64) {
    let Some(mut b) = m.brain.take() else { return };
    let log = log_on();
    let accuracy = accuracy_at(b.accuracy, n, b.follows);
    // a follower: the squad's wingman follow, firing at what it sees
    if b.follows {
        if b.base.is_none() {
            b.base = Some(GOAL_WINGMAN_FOLLOW);
            if log {
                println!("t={:.2} brain {}: follows {} (GOAL_WINGMAN_FOLLOW), formation place {slot}", m.sim_time, b.label, unit_name(leader));
            }
        }
        stop_walk(m);
        // (with nothing to fire at, turned to a contact's last place, as the alert below)
        let eye = m.position + Vec3::Y * (GROUND + m.height + AI_EYE);
        let toward = if target.is_none() { m.sensor.toward(eye) } else { None };
        // (its idle pauses: its wingman personality's EVT_IDLE_VARY_DATA draw)
        let pauses = game.personality_entry(b.personality, personality::EVT_IDLE_VARY_DATA).map(|e| e.pauses()).unwrap_or_default();
        squad_ai(m, leader, slot, heading, dt, target, toward, &pauses);
        aim(m, target, accuracy);
        m.brain = Some(b);
        return;
    }
    let mut npc = m.npc.unwrap_or(testtools::Npc { id: 0, fight: true, face: m.yaw });
    // alert: it fights where it stands (#26 moves it), facing the target, or its last place
    let known = m.sensor.contacts.len();
    if known > 0 {
        if b.walking(m) {
            stop_walk(m);
            if log {
                println!("t={:.2} brain {}: stops: {known} contact{}", m.sim_time, b.label, if known == 1 { "" } else { "s" });
            }
        }
        // (the patrol is started again, from the point nearest it, once it's over)
        b.route.clear();
        let toward = target.or(b.target.and_then(|w| m.sensor.contact(w)).map(|c| c.last));
        if let Some(p) = toward {
            let to = p - m.position;
            npc.face = (-to.x).atan2(-to.z);
        }
        m.npc = Some(npc);
        testtools::npc_ai(m, target, None, dt);
        // the gun stays raised while it knows the enemy (the attack goal's ready pose)
        m.aim = true;
        aim(m, target, accuracy);
        m.brain = Some(b);
        return;
    }
    // calm: standing, firing at nothing, facing the way it is
    npc.face = m.yaw;
    m.npc = Some(npc);
    testtools::npc_ai(m, None, None, dt);
    let here = m.position + Vec3::Y * GROUND;
    // home first (EVT_RETURN_HOME, pri 1.5 over EVT_BASE's 1)
    if b.go_home {
        let home = b.home.filter(|h| h.xz().distance(here.xz()) > b.home_radius);
        let stuck = home.is_some_and(|h| b.stuck(m, here, h, dt));
        match (home, b.home_mt) {
            (Some(h), Some(mt)) if !stuck => b.walk(m, h, mt),
            _ => {
                b.go_home = false;
                stop_walk(m);
                b.best = f32::MAX;
                if log {
                    match home {
                        Some(h) => println!("t={:.2} brain {}: {} {:.1} m short of home, at ({:.1}, {:.1}): stays there", m.sim_time, b.label,
                                            if b.home_mt.is_none() { "creeps (locomotion 1)" } else { "stalls" }, h.xz().distance(here.xz()), here.x, here.z),
                        None => println!("t={:.2} brain {}: home at ({:.1}, {:.1})", m.sim_time, b.label, here.x, here.z),
                    }
                }
            }
        }
        m.brain = Some(b);
        return;
    }
    // the base goal: drawn once (FUN_00054c40), a refused one struck out (FUN_00054bc0)
    if b.base.is_none() {
        let entry = game.personality_entry(b.personality, EVT_BASE);
        let u = m.random(1_000_000) as f32 / 1_000_000.0;
        b.base = entry.and_then(|e| e.draw(u, &b.refused)).or(Some(GOAL_IDLE_VARY));
        if log {
            println!("t={:.2} brain {}: base goal {} (personality h_{:08x}'s EVT_BASE: {:?}, refused {:?})", m.sim_time, b.label, goal_name(b.base),
                     b.personality, entry.map(|e| &e.goals), b.refused);
        }
    }
    if b.base == Some(GOAL_PATROL) {
        if b.route.is_empty() && !b.start_patrol(here, patrols, m.sim_time) {
            // refused: struck out, the draw again next frame
            b.refused.push(GOAL_PATROL);
            b.base = None;
            m.brain = Some(b);
            return;
        }
        b.patrol(m, here, dt);
    } else if b.walking(m) {
        stop_walk(m);
    }
    m.brain = Some(b);
}

impl Brain {
    /// Whether its walk is under way (the drive it set: play_enemies.rs).
    fn walking(&self, m: &Player) -> bool {
        m.enemy.drive.is_some()
    }

    /// Walk toward `to` in motion type `mt` (the drive, play_enemies.rs: the type's speed for it,
    /// the relaxed clip, turning at the type's turn-rate), or keep walking there. Not while a hit
    /// reaction is due or plays: it walks on after.
    fn walk(&mut self, m: &mut Player, to: Vec3, mt: i64) {
        if let Some(d) = m.enemy.drive.as_mut().filter(|d| d.mt == mt) {
            d.goal = Some(to.xz());
        } else if !enemies::reacting(m) {
            enemies::drive(m, mt, Some(to.xz()), false);
        }
    }

    /// Whether `m`'s walk toward `to` has stalled (STALL_PROGRESS in STALL_TIME s), counting; not
    /// while a hit reaction is due or plays (the walk waits for it: `walk`).
    fn stuck(&mut self, m: &Player, here: Vec3, to: Vec3, dt: f32) -> bool {
        let d = here.xz().distance(to.xz());
        if d < self.best - STALL_PROGRESS {
            (self.best, self.stalled) = (d, 0.0);
        } else if !enemies::reacting(m) {
            self.stalled += dt;
        }
        self.stalled >= STALL_TIME
    }

    /// GOAL_PATROL's start (FUN_0002af70): the route from the patrol point nearest `here`, or
    /// false (refused) without one of two points or more.
    fn start_patrol(&mut self, here: Vec3, patrols: &[PatrolPoint], now: f32) -> bool {
        let Some(nearest) = patrols.iter().min_by(|a, b| a.at.distance(here).total_cmp(&b.at.distance(here))) else { return false };
        let Some(head) = patrols.iter().find(|p| p.head && p.group == nearest.group) else { return false };
        let point = |p: &PatrolPoint| RoutePoint { at: p.at, radius: if p.radius > 0.0 { p.radius } else { ARRIVE }, name: p.name,
                                                   end: p.forward == 0 || p.backward == 0, trigger: p.trigger };
        let mut route = vec![point(head)];
        let mut next = head.forward;
        while let Some(p) = patrols.iter().find(|p| p.name == next && next != 0 && p.name != head.name) {
            if route.len() > patrols.len() {
                break;
            }
            route.push(point(p));
            next = p.forward;
        }
        if route.len() < 2 {
            return false;
        }
        self.at = route.iter().position(|r| r.name == nearest.name).unwrap_or(0);
        self.route = route;
        self.best = f32::MAX;
        if log_on() {
            println!("t={now:.2} brain {}: patrols iD {} ({} points, from point {}, ({:.1}, {:.1}), nearest of {} at {:.1} m)", self.label, head.group,
                     self.route.len(), self.at, self.route[self.at].at.x, self.route[self.at].at.z, patrols.len(), nearest.at.distance(here));
        }
        true
    }

    /// GOAL_PATROL's update (FUN_0002b490 with FUN_00026d80 and FUN_00026940): walk to the route's
    /// point; reached (or stalled), the next, the route turned round at an end of the chain.
    fn patrol(&mut self, m: &mut Player, here: Vec3, dt: f32) {
        let p = self.route[self.at];
        let reached = here.xz().distance(p.at.xz()) < p.radius;
        if reached || self.stuck(m, here, p.at, dt) {
            let log = log_on();
            if log {
                println!("t={:.2} brain {}: {} patrol point {} ({:.1}, {:.1}){}", m.sim_time, self.label, if reached { "reaches" } else { "stalls short of" },
                         self.at, p.at.x, p.at.z, if p.trigger != 0 { format!(": its trigger-object h_{:08x} not sent (#227)", p.trigger) } else { String::new() });
            }
            // the chain's end: turned round, walked back (FUN_00026940 -> FUN_00023320); a loop's
            // end goes on to its start (FUN_00012650)
            if self.at + 1 >= self.route.len() {
                if p.end {
                    self.route.reverse();
                }
                self.at = 0;
            }
            self.at = (self.at + 1).min(self.route.len() - 1);
            self.best = f32::MAX;
            self.stalled = 0.0;
        }
        let to = self.route[self.at].at;
        self.walk(m, to, enemies::MT_WALK);
    }
}

/// FUN_00024ae0: the accuracy (0-1) from its percent `a` at difficulty `n`: + 0.05 n (halved above
/// 0), within half and one and a half of a / 100, at most 1. A character of a squad a player
/// controls (`players`: the world's +0xbec object's vtable +0x7c, 0x8d9f0, counts the players
/// whose squad it is) takes a / 100 as it is: the marines following Tex.
fn accuracy_at(a: f32, n: i64, players: bool) -> f32 {
    let a = a * 0.01;
    if players {
        return a;
    }
    let d = n as f32 * ACCURACY_STEP;
    let d = if d > 0.0 { d * 0.5 } else { d };
    (a + d).max(a * 0.5).min(a * 1.5).min(1.0)
}

/// FUN_00024c00: the aim of the shots it fires at `target` (a chest) this frame, `ai_fire` having
/// faced it: on the chest with the probability `accuracy`, else off it (see the module notes).
/// The gun and its shots go to it (`Player::ai_target`).
fn aim(m: &mut Player, target: Option<Vec3>, accuracy: f32) {
    let Some(chest) = target else { return };
    let u = m.random(1_000_000) as f32 / 1_000_000.0;
    if accuracy > u {
        return;
    }
    let eye = m.position + Vec3::Y * (GROUND + m.height + AI_EYE);
    let line = chest - eye;
    let d = line.length();
    if d < 1e-3 {
        return;
    }
    let turn = (m.random(1_000_000) as f32 / 1_000_000.0 * 2.0 - 1.0) * MISS_ANGLE.to_radians();
    let off = Quat::from_axis_angle(line / d, turn) * Vec3::Y;
    let long = if off.y > MISS_UP { MISS_HIGH } else { MISS_LONG };
    let near = if d < MISS_NEAR { d / MISS_NEAR } else { 1.0 };
    m.ai_target = Some(chest + off * long * near);
}

/// `u` is held by a script (119, its start, a 105 walk: play_script.rs): no brain meanwhile
/// (char+0x834 bit 0), so the walk it had going stops; it takes its goal up again once let go.
pub fn held(u: &mut Player) {
    if u.brain.is_some() {
        stop_walk(u);
    }
}

/// Its walk stops: the drive let go and its walking motion with it (a hit reaction playing stays),
/// the body sliding to a stop (play_enemies.rs `glide`), as the drive does at its goal.
fn stop_walk(m: &mut Player) {
    m.enemy.drive = None;
    if m.enemy.motion.is_some_and(|p| !p.once) {
        m.enemy.motion = None;
    }
}

/// A base goal's name for the log.
fn goal_name(g: Option<i64>) -> &'static str {
    match g {
        Some(GOAL_IDLE_VARY) => "GOAL_IDLE_VARY (stands)",
        Some(GOAL_PATROL) => "GOAL_PATROL",
        Some(GOAL_WINGMAN_FOLLOW) => "GOAL_WINGMAN_FOLLOW",
        Some(1) => "GOAL_IDLE (stands)",
        Some(_) => "a goal the demo doesn't run (stands)",
        None => "none",
    }
}

/// BF_TEST_AIM_HOSTILE=<s>: from that time the controlled character's camera turns onto the
/// nearest living hostile it can see (within 120 m, nothing of the world in between: its chest,
/// else its head over cover) and it aims and fires at it: the takes' aim helper
/// (todo/28-tutorial/notes.md, "Aim helper"), to stage a fight. Run before the character's step
/// (play.rs `update_player`), after the test hooks' input. With a gun that has a scope its aim is
/// the scope's first level, as the other hooks' aim (play.rs `scope_input`, which ran before this
/// and made the aim whether the character is in the scope); `scoped` says the hook went in, so it
/// comes out once no hostile is left in sight, as the aim it held fell before the levels.
fn aim_hostile(mut player: ResMut<Player>, squad: Res<Squad>, mut scoped: Local<bool>) {
    let Some(from) = std::env::var("BF_TEST_AIM_HOSTILE").ok().and_then(|v| v.parse::<f32>().ok()) else { return };
    let player = &mut *player;
    if player.sim_time < from || player.dead || player.absent || script::blocks_input(player) {
        return;
    }
    let eye = player.position + Vec3::Y * (GROUND + player.height + AI_EYE);
    let clear = |p: Vec3| eye.distance(p) < AIM_HELPER_REACH && ray_hit(eye, (p - eye).normalize_or_zero(), eye.distance(p) - 0.3).is_none();
    let Some(chest) = squad.0.iter().filter(|u| !u.dead && !u.absent && u.team != player.team && u.loaded.is_some())
        .filter_map(|u| [AI_AIM_HEIGHT, AI_EYE].map(|h| u.position + Vec3::Y * (GROUND + u.height + h)).into_iter().find(|&p| clear(p)))
        .min_by(|a, b| eye.distance(*a).total_cmp(&eye.distance(*b))) else {
        player.fire = false;
        if std::mem::take(&mut *scoped) && player.scope_level > 0 {
            player.leave_scope();
            player.aim = false;
        }
        return;
    };
    // turn the camera until the crosshair's ray (play.rs `aim_ray`) passes through the chest: a
    // few steps of the angle between them
    for _ in 0..4 {
        let (cam, dir) = aim_ray(player);
        let want = (chest - cam).normalize_or_zero();
        let yaw = |v: Vec3| (-v.x).atan2(-v.z);
        let pitch = |v: Vec3| v.y.clamp(-1.0, 1.0).asin();
        player.cam_yaw = wrap_angle(player.cam_yaw + wrap_angle(yaw(want) - yaw(dir)));
        player.cam_pitch = (player.cam_pitch + pitch(want) - pitch(dir)).clamp(-camera::PITCH_LIMIT, camera::PITCH_LIMIT);
    }
    player.aim = true;
    player.fire = true;
    if player.scope_level == 0 && scope_levels(player) > 0 {
        player.scope_level = 1;
        *scoped = true;
    }
}

/// How far the aim helper looks for a hostile (m): beyond any of the tutorial's fights.
const AIM_HELPER_REACH: f32 = 120.0;
