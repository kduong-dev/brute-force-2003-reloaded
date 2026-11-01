//! The squad's follow (#331; viewer/docs/squad.md "The follow", nav.md "The squad on routes"):
//! GOAL_WINGMAN_FOLLOW's update FUN_00020b10, the goal of the squad's two follow modes, FOLLOW
//! (Cover Me, the default) and WINGMAN (Fire At Will), in place of the demo's formation places.
//! Every function cited is in viewer/docs/code-map.md ("AI"); the reading is todo/343-first-walk-
//! line/leads.md §1-§2 and #331's own (the code map's rows for the states).
//!
//!  - **The leader** (FUN_00020b10's start, FUN_0005e540): the squad's formation record (the
//!    squad's +0x28) keeps a list per follow mode, WINGMAN's at +4 and FOLLOW's at +0x14
//!    (`Formation`). A member's leader is the controlled character but for the third entry of a
//!    list, which follows the first (the tables 0x3ffc40 / 0x3ffc70: entries 0 and 1 follow 3, the
//!    controlled character, entry 2 follows entry 0; WINGMAN's while its first entry has no route
//!    point), counted over both lists (a member left in WINGMAN's list by a Cover Me keeps its
//!    place there). The lists start empty at each level and fill as orders change the members'
//!    modes, so after one the squad runs as a column: #33 take06's Hawk, the third after Cover Me
//!    to Flint, Brutus and Hawk, waited for Flint to be 13 m off while Tex was 27 m off, trailed
//!    Flint by 15-16 m without the boost Tex's distance would have given her, and stopped 8.8 m
//!    from him; take04's three, fresh, each followed Tex.
//!  - **The five distances** (FUN_00020b10's stack +8..+0x18): 8, 10, 7, 7 and 4 m in WINGMAN, and
//!    1.4 times them in FOLLOW (11.2, 14, 9.8, 9.8, 5.6; 0x20d6c-0x20de3), measured to the leader
//!    in 3D (`distances`).
//!  - **The states** (brain+0x6ac, the switch at 0x20f0b, looped while a state asks to go on, at
//!    most 13 times): the goal starts in 2 (FUN_000208e0); 2 begins a hold (the pace level 1 for
//!    0.3 s) and goes to 3; 3 holds while the leader is within d0 (x1.5 at a cover point), else 0;
//!    0 chases a route to the leader's own point at the dash until he's within d2 and not running,
//!    when FOLLOW looks for a cover point (4) and WINGMAN holds (2), and at the route's end holds
//!    with him within d4; 4 finds a cover point near the leader and walks to it (5), else holds.
//!    The states 6-16 walk the squad's route points (the first entry's in WINGMAN: FUN_0001adf0's
//!    point ahead of the leader on his path), not done: what fills the leader's path record
//!    (brain+0x1ac, FUN_0005acb0 over the points' +0x38 / +0x44 link lists) and brain+0x718 /
//!    +0x71c (the route-point distances) aren't traced, so no member has a route point and the
//!    machine stays in 0-5, as FOLLOW's always does (FUN_00020b10 clears the point in mode 1).
//!    Their one line, state 9's 1ecaeb98, keeps a stand-in (`walk_line`).
//!  - **The speed record** (brain+0x8a0 / +0x89d): 1.25 and on every update, 1.0 and off in state 0
//!    within d1 and in state 5 (`Follow::record`; play_speed.rs `record_boost` hands it on).
//!
//! The routes, the AI update's route steps, the pace ramp and the avoidance are play_routes.rs's
//! and play_avoid.rs's; this module decides where the follow goes and when it stands. Without a
//! level's points (the test map's floor) state 0 walks straight at the leader.
//!
//! Test hooks: BF_FOLLOW_LOG=1 prints each state change and why, each cover point found and
//! claimed, and the formation's lists when they change; BF_TEST_FORMATION=<member>,<member>,...
//! puts those members (as `BF_TEST_ORDER` names them: brutus, flint, hawk, tex or a slot; in
//! FOLLOW) into FOLLOW's list in that order at each level's start, where the list starts empty,
//! to stage the column an order leaves (#33 take06's Cover Me to Flint, Brutus and Hawk:
//! flint,brutus,hawk).

use bf_viewer::bf::nav::{self, Dest, Graph};
use brain::Mode;

use super::*;

/// The formation's bookkeeping runs before the characters' step, after the frame's input and its
/// orders (a mode change made later in the frame is seen at the next); a level's start begins it
/// afresh (`reset`), as its squad.
pub fn plugin(app: &mut App) {
    app.init_resource::<Formation>()
        .add_systems(OnEnter(AppState::Playing), reset)
        .add_systems(Update, sync.after(read_input).before(update_player).run_if(in_state(AppState::Playing)));
}

/// A level's start: the formation record empty, as the squad's new brains find it (`Formation`:
/// the old brains' destructor, FUN_00017770 at 0x177c0, took each squad member out of both lists;
/// the squad itself is made anew, play.rs `setup`), and BF_TEST_FORMATION to be applied again.
/// Without it a fresh squad (`Command::joined` 0, so never added) kept the last level's lists.
/// BF_FOLLOW_LOG says what it dropped.
fn reset(mut formation: ResMut<Formation>) {
    if log_on() {
        println!("level start: the formation record afresh (it held {} in FOLLOW's list, {} in WINGMAN's)", formation.follow.len(), formation.wingman.len());
    }
    *formation = Formation::default();
}

/// WINGMAN's five distances (FUN_00020b10's stack, 0x20d6c-0x20d94: 0x41000000, 0x41200000,
/// 0x40e00000 twice, 0x40800000): d0 the hold, d1 the speed record's, d2 where the chase stops for
/// a leader who isn't running, d3 the re-plan, d4 the hold at a route's end.
const WINGMAN_D: [f32; 5] = [8.0, 10.0, 7.0, 7.0, 4.0];
/// FOLLOW's (mode 1, `cmp [holder+0x18], 1` at 0x20ddd): 0x41333333, 0x41600000, 0x411ccccd twice,
/// 0x40b33333, 1.4 times WINGMAN's.
const FOLLOW_D: [f32; 5] = [11.2, 14.0, 9.8, 9.8, 5.6];
/// The state loop's bound (FUN_00020b10: `iStack_50 < 0xd`).
const STEPS: usize = 13;
/// State 0's walk (FUN_00026d80 at 0x40000000, 0x22346): each entry reached within 2 m.
const CHASE_ARRIVE: f32 = nav::ARRIVE;
/// State 2 (FUN_000223c0): the pace level 1 with its timer at 0.3 s (0x3e99999a at +0x2c and
/// +0x34), which the ramp climbs back from.
const HOLD_STAND: f32 = 0.3;
/// State 3 (FUN_00022420): d0 x this (0x3a4fb4) while the member's own point is a cover point.
const COVER_HOLD: f32 = 1.5;
/// State 4's flood (FUN_000225d0 -> FUN_000594a0): from the leader's own point through links
/// within 12 m of it (FUN_00020170's 0x41400000 at the filter's +0x110).
const COVER_FLOOD: f32 = 12.0;
/// ...a point the filter takes lying within this of the member (3D, strictly: 0x20410 squares the
/// filter's 12 m x 0.5, 0x3a4f74).
const COVER_NEAR: f32 = 6.0;
/// State 5's walk (FUN_00022790): 1.0 m an entry (0x3f800000), 0.1 m for its last, the cover point
/// (0x3dcccccd; 0.25, 0x3e800000, for one whose +0x5c is over 61 degrees, read as its fov-angle:
/// not kept in the demo's points, so 0.1).
const COVER_ARRIVE: f32 = 1.0;
const COVER_LAST: f32 = 0.1;
/// The goal's start (FUN_000208e0): a member whose own point is a cover point within this of it
/// (xz, 0.25 its square at 0x3a5028) claims it.
const START_CLAIM: f32 = 0.5;
/// FUN_0005e670's insert into FOLLOW's list: before an entry only when this much nearer the
/// controlled character than it (and this much farther than the entry before).
const INSERT_GAP: f32 = 5.0;
/// FUN_00058740's test (0x3a5028): the member's facing and the way to its route's first entry
/// with a dot under this (the entry more than about 75.5 deg off the facing) start the walk.
const TURN_DOT: f32 = 0.25;
/// FUN_00058740: the time (s) the pace level is held at a walk (0x3fc00000 to the pace record's
/// +0x2c and +0x34), after which the ramp climbs a notch each 0.17 s (play_routes.rs `pace_ramp`).
const TURN_WALK: f32 = 1.5;
/// Whom each of WINGMAN's first three entries follows (FUN_0005e310 over 0x3ffc70: each 16-byte
/// entry's second dword, or its fourth while its third is 1 and entry 0 has a route point,
/// 0x5e3b7-0x5e3ee; no member has one in the demo): the number of an entry of the same list, 3 the
/// squad's controlled character (whose number it is, FUN_0005bca0; it's in neither list). Entry 2
/// follows entry 0 (with the route point it would follow the controlled character, and entry 1
/// entry 0).
const WINGMAN_LEADS: [usize; 3] = [3, 3, 0];
/// FOLLOW's (0x3ffc40's second dwords; no route-point variant): entry 2 follows entry 0.
const FOLLOW_LEADS: [usize; 3] = [3, 3, 0];
/// FUN_0005e220's answer for a character not in the list: its part of FUN_0005e310 is skipped.
const ABSENT: usize = 4;

/// The follow's state (brain+0x6ac): the formation family the demo runs. The numbers are the
/// game's; 1 (FUN_00021fa0, the walk to brain+0x738's object, whose writer isn't traced) and the
/// route-point family 6-16 aren't run (see the module notes).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum State {
    /// 0, FUN_00022130: the chase, a route to the leader's own point at the dash
    Chase,
    /// 2, FUN_000223c0: a hold begins (the goal starts here, FUN_000208e0)
    #[default]
    HoldBegin,
    /// 3, FUN_00022520 -> FUN_00022420: the hold
    Hold,
    /// 4, FUN_000225d0: FOLLOW's look for a cover point near the leader
    FindCover,
    /// 5, FUN_00022790: the walk to the cover point claimed
    ToCover,
}

impl State {
    /// Its number in the game's switch, for the log.
    fn number(self) -> u8 {
        match self {
            State::Chase => 0,
            State::HoldBegin => 2,
            State::Hold => 3,
            State::FindCover => 4,
            State::ToCover => 5,
        }
    }
}

/// A member's follow (the goal's fields on the brain), kept with the character through the
/// hand-over (`Player::follow`).
#[derive(Clone, Debug, Default)]
pub struct Follow {
    /// brain+0x6ac
    pub state: State,
    /// brain+0x6c0: the cover point it claimed (the point's +0x28 bit 0, `nav::CLAIMED`)
    pub claimed: Option<u32>,
    /// its leader this frame (`sync`, FUN_0005e540): None the controlled character
    pub leader: Option<Who>,
    /// the first entry of its mode's list (FUN_0005ea10's test, the tables' first byte)
    pub first: bool,
    /// the speed record as the states leave it (brain+0x8a0 while +0x89d is set; None off)
    pub record: Option<f32>,
    /// the goal has started (FUN_000208e0 ran: the start's claim done)
    started: bool,
    /// stepping out of the leader's line of fire (`squad_ai`'s lane step, the demo's)
    pub lane: bool,
    /// claims given up away from the graph, their bits cleared by the next `sync`
    unclaim: Vec<u32>,
}

impl Follow {
    /// The goal starts again (a mode change's new base goal, the brain released by the player, the
    /// player taking it): state 2 and its start to come, its claim given up (the goal's end,
    /// FUN_00020a10, as read: the claimed point is the goal's).
    pub fn restart(&mut self) {
        if let Some(p) = self.claimed.take() {
            self.unclaim.push(p);
        }
        self.state = State::HoldBegin;
        self.started = false;
        self.lane = false;
    }
}

/// The leader as the follow reads him (FUN_00020b10's leader record): who, where (his node's
/// position), his own world point (FUN_0011dfa0's place) and how he moves (`class`).
#[derive(Clone, Copy, Debug)]
pub struct Lead {
    pub who: Who,
    pub at: Vec3,
    pub point: Option<u32>,
    pub class: i32,
}

impl Lead {
    /// `u` as a leader.
    pub fn of(u: &Player) -> Lead {
        Lead { who: u.who(), at: u.position, point: u.router.own.point, class: class(u) }
    }
}

/// FUN_00020ad0, the leader's movement by his motion state char+0x1f8: 4 the dash (8), 3 a run
/// (FUN_00126a40's states 3, 0x1a, 0x1b, 0x1e, 0x1f, 0x22), 2 a walk (FUN_00126b40's 4, 0x1c,
/// 0x1d, 0x20, 0x21, 0x2f), else 1. The demo's motion is its move input and gait: the stick held,
/// sprinting 4, walking 2, else 3. #33 take04: Tex's motion read 8 from the stick's push to its
/// release, while he slowed against the rocks too.
pub fn class(u: &Player) -> i32 {
    if u.dead || u.move_input.length() <= 0.1 {
        1
    } else if u.sprint {
        4
    } else if u.walk {
        2
    } else {
        3
    }
}

/// A character as state 4's filter looks at it (FUN_00045310, FUN_00020260): who, where, its own
/// world point, whether it's an AI member in FOLLOW (FUN_00020260's mode 1 and brain+0x14 bit 1,
/// read as a brain the AI runs), and its follow state.
#[derive(Clone, Copy, Debug)]
pub struct Mate {
    pub who: Who,
    pub at: Vec3,
    pub point: Option<u32>,
    pub follows: bool,
    pub state: State,
}

/// The squad's characters as state 4 looks through them (the squad list, char+0x59c): the
/// controlled one and the AI squad and its followers, alive and in the world.
pub fn mates(player: &Player, squad: &[Player]) -> Vec<Mate> {
    std::iter::once(player).chain(squad.iter()).filter(|u| !u.dead && !u.absent && (u.in_squad() || follower(u))).map(|u| Mate {
        who: u.who(),
        at: u.position,
        point: u.router.own.point,
        follows: !u.controlled && u.command.mode == Mode::Follow,
        state: u.follow.state,
    }).collect()
}

/// A level character's brain that follows the leader (the tutorial's marines, play_brain.rs).
fn follower(u: &Player) -> bool {
    u.brain.as_ref().is_some_and(|b| b.follows())
}

/// The five distances in `mode` (FUN_00020b10): FOLLOW's, else WINGMAN's.
pub fn distances(mode: Mode) -> [f32; 5] {
    if mode == Mode::Follow { FOLLOW_D } else { WINGMAN_D }
}

/// The squad's formation record (the squad's +0x28): its two follow lists, in order, and what each
/// member was last added as. FUN_0005e8b0 adds a member by its mode (0x5e8fc-0x5e9fb), the two
/// follow modes not alike:
///  - WINGMAN: a member already in WINGMAN's list stays as it is (0x5e92d); else it's taken out of
///    FOLLOW's (FUN_0005e1e0 on the record + 0x10, 0x5e937) and put at WINGMAN's end (FUN_0005eab0;
///    at its front when the one entry is in state 12 or 14, never in the demo);
///  - FOLLOW: one not in FOLLOW's list goes in by its distance (FUN_0005e670, `insert`); WINGMAN's
///    list isn't touched, so a member that had Fire At Will keeps its place there;
///  - any other mode: out of both (FUN_0005e600 at 0x5e9fb).
///
/// The callers: the brain's set-up at the spawn (FUN_00017b60), every mode change (FUN_00018040's
/// end), the brain released by the player (FUN_00018230). Out of both besides (FUN_0005e600): a
/// character the player takes (FUN_00018150, 0x181ef), one that dies (FUN_00112830, read as the
/// death, -> FUN_0001a420 at 0x1a454: the demo drops the dead), the killed leader at
/// EVT_LEADER_KILLED (the brain's event case at 0x3e1db: not modelled) and a squad member's brain
/// destroyed (its destructor FUN_00017770, 0x177c0: so a new level's brains find the lists empty).
/// So after Fire At Will and then Cover Me to the three both lists hold them, and a Fire At Will
/// to one of them changes neither. At a level's start the lists are empty: the
/// spawn's insert finds no controlled character to measure from (FUN_0005e670 adds nothing without
/// FUN_0005bca0's member; read, and the takes agree: #33 take04's three, fresh from the start, each
/// set off by Tex's own distance, d0, Brutus 0.3 s after Flint and Hawk as he stood 4 m nearer,
/// where take06's Hawk, after Cover Me to the three in the squad list's order, the third entry,
/// waited for Flint to be 13 m off while Tex was 27 m off, trailed him by 15-16 m and stopped 8.8 m
/// from him). So a member joins at its first mode change or its release by the player
/// (`Command::joined` over 0), and the demo starts each level's record afresh (`reset`).
#[derive(Resource, Default, Debug)]
pub struct Formation {
    pub wingman: Vec<Who>,
    pub follow: Vec<Who>,
    /// each member's mode and mode change (`Command::joined`) when it was last added
    seen: HashMap<Who, (Mode, u64)>,
    /// BF_TEST_FORMATION applied (this level)
    staged: bool,
}

impl Formation {
    /// `who` out of both lists (FUN_0005e600).
    fn remove(&mut self, who: Who) {
        self.wingman.retain(|w| *w != who);
        self.follow.retain(|w| *w != who);
    }

    /// `who` added by `mode` (FUN_0005e8b0, see `Formation`), `far` its distance (3D) to the
    /// controlled character and `dist` the others'.
    fn add(&mut self, who: Who, mode: Mode, far: f32, dist: &HashMap<Who, f32>) {
        match mode {
            Mode::Wingman if !self.wingman.contains(&who) => {
                self.follow.retain(|w| *w != who);
                self.wingman.push(who);
            }
            Mode::Follow if !self.follow.contains(&who) => insert(&mut self.follow, who, far, dist),
            Mode::Wingman | Mode::Follow => {}
            _ => self.remove(who),
        }
    }

    /// The list of `mode` (FOLLOW's or WINGMAN's), none for another mode.
    fn list(&self, mode: Mode) -> Option<&Vec<Who>> {
        match mode {
            Mode::Follow => Some(&self.follow),
            Mode::Wingman => Some(&self.wingman),
            _ => None,
        }
    }

    /// Whether `who` follows `x` (FUN_0005e310's list for `x`): `x`'s number in WINGMAN's list (its
    /// index, FUN_0005e220; ABSENT skips the part), and `who` an entry of WINGMAN's first three whose
    /// WINGMAN_LEADS is that number; then the same over FOLLOW's list with FOLLOW_LEADS. Both lists
    /// count whatever `who`'s mode, so a WINGMAN entry 2 left by a Cover Me still follows WINGMAN's
    /// entry 0. (A fourth entry's number, 3, is the controlled character's, as read, so entries 0
    /// and 1 would follow it: the demo's lists never hold four.)
    fn follows(&self, who: Who, x: Who) -> bool {
        let part = |list: &[Who], leads: &[usize; 3]| {
            let id = list.iter().position(|w| *w == x).filter(|&i| i != ABSENT);
            id.is_some_and(|id| list.iter().take(leads.len()).enumerate().any(|(i, w)| *w == who && leads[i] == id))
        };
        part(&self.wingman, &WINGMAN_LEADS) || part(&self.follow, &FOLLOW_LEADS)
    }

    /// `who`'s leader in `mode` (FUN_0005e540, EBX the list of its mode: the record or the record +
    /// 0x10, FUN_00020b10 at 0x20b49 / 0x20b52): the first entry of that list, in its order and not
    /// `who`, that `who` follows (`follows`); None the controlled character (FUN_0005bca0), as for a
    /// member in no list or another mode.
    pub fn leader_of(&self, who: Who, mode: Mode) -> Option<Who> {
        self.list(mode)?.iter().copied().filter(|&x| x != who).find(|&x| self.follows(who, x))
    }

    /// Whether `who` is the first entry of its mode's list (FUN_0005ea10).
    pub fn first(&self, who: Who, mode: Mode) -> bool {
        self.list(mode).is_some_and(|l| l.first() == Some(&who))
    }

    /// One frame's bookkeeping (`sync`) over the AI squad `squad`, `me` the controlled character's
    /// position: a member whose mode or mode change differs from what it was added as is added
    /// again by its mode (`add`), in the order the changes were made (`Command::joined`; a fresh
    /// brain's 0 is no join); one no longer a member (dead, out of the world, taken by the player)
    /// leaves both lists. True when anything changed.
    fn update(&mut self, me: Vec3, squad: &[Player]) -> bool {
        let member = |u: &Player| (u.in_squad() || follower(u)) && !u.dead && !u.absent;
        let dist: HashMap<Who, f32> = squad.iter().filter(|u| member(u)).map(|u| (u.who(), u.position.distance(me))).collect();
        let mut changed = false;
        let gone: Vec<Who> = self.seen.keys().filter(|w| !dist.contains_key(w)).copied().collect();
        for w in gone {
            self.remove(w);
            self.seen.remove(&w);
            changed = true;
        }
        let mut adds: Vec<(u64, usize, Who, Mode)> = squad.iter().enumerate().filter(|(_, u)| member(u) && u.command.joined > 0)
            .filter(|(_, u)| self.seen.get(&u.who()) != Some(&(u.command.mode, u.command.joined)))
            .map(|(k, u)| (u.command.joined, k, u.who(), u.command.mode)).collect();
        adds.sort_by_key(|a| (a.0, a.1));
        for (stamp, _, who, mode) in adds {
            self.add(who, mode, dist[&who], &dist);
            self.seen.insert(who, (mode, stamp));
            changed = true;
        }
        changed
    }
}

/// FUN_0005e670: `who`, `far` m from the controlled character, into FOLLOW's `list`: before the
/// first entry it's more than INSERT_GAP nearer than, provided it's more than INSERT_GAP farther
/// than the entry before that (`dist`: each entry's distance, 3D), else at the end.
fn insert(list: &mut Vec<Who>, who: Who, far: f32, dist: &HashMap<Who, f32>) {
    let d = |w: &Who| dist.get(w).copied().unwrap_or(f32::MAX);
    for k in 0..list.len() {
        let here = d(&list[k]);
        let after_prev = k == 0 || d(&list[k - 1]) + INSERT_GAP < far;
        if far < here - INSERT_GAP && after_prev {
            list.insert(k, who);
            return;
        }
    }
    list.push(who);
}

/// BF_FOLLOW_LOG=1, read once.
fn log_on() -> bool {
    static ON: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ON.get_or_init(|| std::env::var("BF_FOLLOW_LOG").is_ok_and(|v| v != "0"))
}

/// The formation's bookkeeping each frame, before the squad's AI (`Formation::update`), then each
/// member's leader and first-entry bit for its follow (`Follow::leader`, `Follow::first`; a change
/// of leader logged under BF_FOLLOW_LOG). BF_TEST_FORMATION's order is applied once a level. The
/// claims the dead and the goals' restarts gave up lose their bits here, where the graph is at
/// hand.
fn sync(mut formation: ResMut<Formation>, mut player: ResMut<Player>, mut squad: ResMut<Squad>, mut world_points: ResMut<super::nav::Nav>) {
    for u in std::iter::once(&mut *player).chain(squad.0.iter_mut()) {
        if u.dead && u.follow.claimed.is_some() {
            u.follow.restart();
        }
        for p in std::mem::take(&mut u.follow.unclaim) {
            if let Some(n) = world_points.graph.as_mut().and_then(|g| g.nodes.get_mut(p as usize)) {
                n.state &= !nav::CLAIMED;
            }
        }
    }
    let f = &mut *formation;
    let mut changed = f.update(player.position, &squad.0);
    let members = squad.0.iter().any(|u| (u.in_squad() || follower(u)) && !u.dead && !u.absent);
    if !f.staged && members {
        f.staged = true;
        if let Some(order) = staged_order(&squad.0) {
            for who in order {
                if let Some(u) = squad.0.iter().find(|u| u.who() == who && u.command.mode == Mode::Follow) {
                    f.remove(who);
                    f.follow.push(who);
                    f.seen.insert(who, (Mode::Follow, u.command.joined));
                }
            }
            changed = true;
        }
    }
    let named: Vec<(Who, String)> = if log_on() { squad.0.iter().map(|u| (u.who(), unit_name(u))).collect() } else { vec![] };
    let name = |w: Who| named.iter().find(|n| n.0 == w).map_or("?", |n| n.1.as_str());
    if changed && log_on() {
        let names = |l: &[Who]| l.iter().map(|w| name(*w)).collect::<Vec<_>>().join(", ");
        println!("t {:.2}: formation: FOLLOW [{}], WINGMAN [{}]", player.sim_time, names(&f.follow), names(&f.wingman));
    }
    for u in squad.0.iter_mut() {
        let leader = f.leader_of(u.who(), u.command.mode);
        if log_on() && leader != u.follow.leader {
            println!("t {:.2}: {}: follows {} (the formation, {:?})", player.sim_time, unit_name(u), leader.map_or("the controlled character", name), u.command.mode);
        }
        u.follow.leader = leader;
        u.follow.first = f.first(u.who(), u.command.mode);
    }
}

/// BF_TEST_FORMATION's members, by `testsquad::slot_named` (the controlled character's slot left
/// out), as the squad's characters.
fn staged_order(squad: &[Player]) -> Option<Vec<Who>> {
    let v = std::env::var("BF_TEST_FORMATION").ok()?;
    let order = v.split(',').filter_map(|n| testsquad::slot_named(n.trim(), usize::MAX))
        .filter_map(|c| squad.iter().find(|u| u.character == c && u.in_squad()).map(|u| u.who())).collect();
    Some(order)
}

/// One frame of `m`'s follow (FUN_00020b10 in FOLLOW or WINGMAN, after the AI update's route
/// steps): its leader `lead` (the formation's, `sync`), the squad `mates` (state 4's filter) and the
/// level's points `graph` (the routes; without them the straight walk). The way it goes: toward a
/// steering target at the dash (FUN_0004d970 mode 4 from states 0 and 5, held to its pace level by
/// the caller's `routes::pace`), or standing. The speed record is left in `Follow::record`.
pub fn update(m: &mut Player, lead: &Lead, mates: &[Mate], graph: Option<&mut Graph>) -> routes::Way {
    let mode = m.command.mode;
    let d = distances(mode);
    let dist = m.position.distance(lead.at);
    let mut graph = graph;
    if !m.follow.started {
        start(m, graph.as_deref_mut());
    }
    walk_line(m);
    // (0x20ece-0x20edb: the speed record 1.25 and on, every update)
    m.follow.record = Some(speed::FOLLOW_BOOST);
    for _ in 0..STEPS {
        let was = m.follow.state;
        let next = match (was, graph.as_deref_mut()) {
            (State::Chase, Some(g)) => chase(m, g, lead, &d, dist, mode),
            (State::Chase, None) => chase_straight(m, lead, &d, dist, mode),
            (State::HoldBegin, g) => hold_begin(m, g),
            (State::Hold, g) => hold(m, g, &d, dist),
            (State::FindCover, Some(g)) => find_cover(m, g, lead, mates),
            (State::FindCover, None) => {
                set(m, State::HoldBegin, "no world points to look for cover on", dist);
                Next::On
            }
            (State::ToCover, Some(g)) => to_cover(m, g, dist),
            (State::ToCover, None) => {
                set(m, State::HoldBegin, "no world points", dist);
                Next::Done(routes::Way::Stand)
            }
        };
        match next {
            Next::On => continue,
            Next::Done(way) => return way,
        }
    }
    routes::Way::Stand
}

/// What a state did: asked for the next state to run now (the switch's `*param = 1`), or ended the
/// update with the way it goes.
enum Next {
    On,
    Done(routes::Way),
}

/// The goal's start (FUN_000208e0): state 2 (set already, `Follow::restart`), and a member whose
/// own point is a cover point within START_CLAIM of it claims it (+0x28 |= 1, brain+0x6c0).
fn start(m: &mut Player, g: Option<&mut Graph>) {
    m.follow.started = true;
    let (Some(g), Some(p)) = (g, m.router.own.point) else { return };
    let n = &g.nodes[p as usize];
    if n.kind == nav::kind::COVER && n.at.xz().distance(m.position.xz()) < START_CLAIM {
        claim(m, g, p);
    }
}

/// The member claims cover point `p` (brain+0x6c0 = p, +0x28 bit 0), giving up the one it had.
fn claim(m: &mut Player, g: &mut Graph, p: u32) {
    release(m, Some(g));
    g.nodes[p as usize].state |= nav::CLAIMED;
    m.follow.claimed = Some(p);
}

/// The claimed point given up (`and [point+0x28], 0xfffffffe`, brain+0x6c0 = 0).
fn release(m: &mut Player, g: Option<&mut Graph>) {
    if let (Some(p), Some(g)) = (m.follow.claimed.take(), g) {
        g.nodes[p as usize].state &= !nav::CLAIMED;
    }
}

/// The state change to `to`, logged with `why` and the leader's distance.
fn set(m: &mut Player, to: State, why: &str, dist: f32) {
    if log_on() && m.follow.state != to {
        println!("t {:.2}: {}: follow state {} -> {} ({why}; its leader {dist:.1} m off)", m.sim_time, unit_name(m), m.follow.state.number(), to.number());
    }
    m.follow.state = to;
}

/// State 9's line (FUN_00021b70 at 0x21c91-0x21cba: brain+0x3c4 bit 0 tested and set, 1ecaeb98
/// through the gate, `Command::start_walk`), a stand-in: the game says it walking to its route
/// point, which only the first entry of WINGMAN's list gets (FUN_0001adf0 -> FUN_0005ea10, the
/// tables' first byte), once per order (the bit, cleared by Fire At Will). The demo has no route
/// point (see the module notes), so the first entry says it at its first follow update after the
/// order, as the game's did 0.2-0.6 s after (#33 take05 / take06 Flint, the list's first after the
/// order to Flint, Brutus and Hawk in the squad list's order; #36 take12 Brutus, ordered alone);
/// the others say nothing (take05's Brutus said it 11 s later, when the list's first entry had
/// changed: not done).
fn walk_line(m: &mut Player) {
    if m.follow.first && m.command.start_walk() {
        if log_on() {
            println!("t {:.2}: {}: the first of WINGMAN's list: \"on my way\" (1ecaeb98, state 9's line; its route point the demo's stand-in)", m.sim_time, unit_name(m));
        }
        chatter::say(m, chatter::FOLLOW_WALK);
    }
}

/// State 0 (FUN_00022130) on a level's points, `d` the five distances, `dist` the leader's:
///  - within d1 (at it too) the speed record 1.0 and off;
///  - no route: a route to the leader's place (FUN_000416a0 with FUN_0011dfa0's places: to his own
///    point, or to his position when the member's own point is his), started (FUN_00025730);
///    none, state 2 (the next update);
///  - a route, the leader beyond d2: planned again when he's farther than d3 (3D) from its last
///    entry (its point's position, else its own: 0x22251-0x222d7);
///  - a route, the leader within d2 and not running (FUN_00020ad0 under 3): FOLLOW to state 4,
///    WINGMAN to state 2, the route reset (FUN_0004e8f0) and the next state now (with brain+0x738
///    set and a player's character there, state 1 or 4: not modelled);
///  - then the walk (FUN_00026d80 at 2 m) at the dash; at its end the route reset and, the leader
///    within d4, state 2, else state 0 again (a new route next update).
fn chase(m: &mut Player, g: &mut Graph, lead: &Lead, d: &[f32; 5], dist: f32, mode: Mode) -> Next {
    if dist <= d[1] {
        m.follow.record = None;
    }
    let mut why = (!m.router.walking()).then_some("the follow's chase: a route to its leader's point");
    if why.is_none() {
        if dist > d[2] {
            let end = m.router.walker.as_ref().and_then(|w| w.entry_at(g, w.entries.len() as i32 - 1));
            if end.is_some_and(|e| e.distance_squared(lead.at) > d[3] * d[3]) {
                why = Some("the follow's chase: its leader over d3 from its route's end");
            }
        } else if lead.class < 3 {
            let to = if mode == Mode::Follow { State::FindCover } else { State::HoldBegin };
            set(m, to, "within d2 of a leader who isn't running", dist);
            routes::stop(m, g);
            return Next::On;
        }
    }
    // (a new route replaces the old in brain+0x930, the corner point kept: `routes::plan`)
    if let Some(why) = why {
        if m.router.own.point == lead.point && in_burning(lead) {
            routes::stop(m, g);
            return Next::Done(routes::Way::Stand);
        }
        if routes::plan(m, g, Dest::Character { at: lead.at, point: lead.point }, why).is_err() {
            routes::stop(m, g);
            set(m, State::HoldBegin, "no route to its leader", dist);
            return Next::Done(routes::Way::Stand);
        }
    }
    if !m.router.walking() {
        return Next::Done(routes::Way::Stand);
    }
    let s = routes::step(m, g, CHASE_ARRIVE, CHASE_ARRIVE);
    if s.walking {
        let along = m.router.walker.as_ref().map_or(0.0, |w| w.remaining(g, m.position));
        return Next::Done(routes::Way::Go { target: s.target, along });
    }
    routes::stop(m, g);
    if dist < d[4] {
        set(m, State::HoldBegin, "at its route's end within d4", dist);
    }
    Next::Done(routes::Way::Stand)
}

/// Whether the leader stands in a liquid that burns (his feet in it: play_liquids.rs `burns_at`).
/// State 0's route to a leader whose own point is the member's is the one entry of his position
/// (FUN_00022d70), which would take the member in after him: the game's AI has no liquid code
/// (todo/146 leads §4.2), its routes keeping it out as no world point lies in a harmful liquid, but
/// a bare position does. The demo's member stands at the shore instead (as play.rs `squad_ai`'s
/// guard, `routes::shore`, holds a step into one): #335's review recipe, Tex running round in e13's
/// lava 11-14 m from his own point on the shore, had the three slide along the shore for good,
/// chasing his position.
fn in_burning(lead: &Lead) -> bool {
    let feet = Vec3::new(lead.at.x, lead.at.y + GROUND, lead.at.z);
    world::arena().is_some_and(|a| liquids::burns_at(&a, feet))
}

/// State 0 without a level's points (the test map's floor; the game's state 0 always has its
/// route): the leader's position walked to straight at the dash, the same tests on him (within d2
/// and not running: FOLLOW's state 4, which finds no point, or WINGMAN's 2), and within CHASE_ARRIVE
/// of him (the route's end) the hold within d4. The demo's.
fn chase_straight(m: &mut Player, lead: &Lead, d: &[f32; 5], dist: f32, mode: Mode) -> Next {
    if dist <= d[1] {
        m.follow.record = None;
    }
    if dist <= d[2] && lead.class < 3 {
        let to = if mode == Mode::Follow { State::FindCover } else { State::HoldBegin };
        set(m, to, "within d2 of a leader who isn't running", dist);
        return Next::On;
    }
    let across = m.position.xz().distance(lead.at.xz());
    if across <= CHASE_ARRIVE {
        if dist < d[4] {
            set(m, State::HoldBegin, "at its leader within d4", dist);
        }
        return Next::Done(routes::Way::Stand);
    }
    Next::Done(routes::Way::Go { target: lead.at, along: across })
}

/// State 2 (FUN_000223c0): the pace level 1 with its timer at HOLD_STAND, state 3 now (the holder's
/// vtable +0x18 and brain+0x740 = +0x738: not modelled).
fn hold_begin(m: &mut Player, g: Option<&mut Graph>) -> Next {
    if let Some(g) = g {
        routes::stop(m, g);
    }
    m.router.set_pace(1, HOLD_STAND);
    m.follow.state = State::Hold;
    Next::On
}

/// State 3 (FUN_00022520 -> FUN_00022420 with d0): the hold, while the leader is within d0 (x1.5
/// while the member's own point is a cover point: the d0 test reads the member's place, EAX from
/// FUN_000120f0 at 0x2242e). Beyond it (and FUN_00020a80 false: brain+0x738, none in the demo) state
/// 0 at the next update, the claimed point given up. (A leader near his brain+0x3c8 object, the
/// stack's +3, holds it whatever his distance: not modelled; nor the hold's facing at the claimed
/// point, FUN_00022520's brain+0x6d4 within 0.3 m.)
fn hold(m: &mut Player, g: Option<&mut Graph>, d: &[f32; 5], dist: f32) -> Next {
    let at_cover = g.as_deref().zip(m.router.own.point).is_some_and(|(g, p)| g.nodes[p as usize].kind == nav::kind::COVER);
    let d0 = if at_cover { d[0] * COVER_HOLD } else { d[0] };
    if d0 < dist {
        release(m, g);
        set(m, State::Chase, if at_cover { "its leader beyond 1.5 x d0 of its cover" } else { "its leader beyond d0" }, dist);
    }
    Next::Done(routes::Way::Stand)
}

/// State 4 (FUN_000225d0, FOLLOW's): the flood from the leader's own point within COVER_FLOOD
/// (FUN_000594a0) for the first point the filter takes (`cover_ok`); found and a route to it
/// (FUN_000416a0), the turn check on it (`turn_check`), the point claimed (the last given up) and
/// state 5; else state 2, the claim given up. Either way the next state now.
fn find_cover(m: &mut Player, g: &mut Graph, lead: &Lead, mates: &[Mate]) -> Next {
    let me = m.who();
    let at = m.position;
    let found = {
        let gr: &Graph = g;
        lead.point.and_then(|lp| gr.flood(lp, COVER_FLOOD, |p| cover_ok(gr, p, me, at, mates))).and_then(|path| path.last().copied())
    };
    let dist = at.distance(lead.at);
    if let Some(p) = found {
        if routes::plan(m, g, Dest::Point(p), "the follow: to a cover point near its leader").is_ok() {
            turn_check(m, g);
            claim(m, g, p);
            if log_on() {
                let n = &g.nodes[p as usize];
                println!("t {:.2}: {}: claims cover point h_{:08x} ({:.1}, {:.1}), {:.1} m off it", m.sim_time, unit_name(m), n.name, n.at.x, n.at.z, n.at.distance(at));
            }
            set(m, State::ToCover, "a cover point near its leader", dist);
            return Next::On;
        }
    }
    release(m, Some(g));
    set(m, State::HoldBegin, "no cover point near its leader", dist);
    Next::On
}

/// FUN_00058740, the turn check state 4 runs on the route it has just planned to its cover point
/// (0x226d0-0x226f3: EDI the pace record [brain+0x1a8], ESI the route's first entry, its point's
/// position, else its own): the member's facing (char+0x1b4's vtable +0x34, flat and unit,
/// FUN_0004c530) against the way to that entry (flat and unit, FUN_0004c480; none when the member
/// stands on it); a dot under TURN_DOT sets the pace level to a walk, 2, for TURN_WALK s
/// ([brain+0x1a8]+0 and its timer), so a member whose cover point lies behind it turns at a walk
/// before the ramp lifts it to a run. #33 take04's Brutus: the dash until the frame Tex stopped
/// (gt 7.21, his chase ending within d2 of a leader not running: states 0, 4 and 5 in one update),
/// then MT_WALK for 1.4-1.5 s, his distance to Tex closing 8.1 -> 6.8 m and opening to 8.0 m (the
/// point behind him), the run from gt 8.71 and there at 8.89. (States 1, 9 and 11 run it too: not
/// in the demo.)
fn turn_check(m: &mut Player, g: &Graph) {
    let Some(first) = m.router.walker.as_ref().and_then(|w| w.entry_at(g, 0)) else { return };
    let flat = |x: f32, z: f32| {
        let d = Vec2::new(x, z);
        if d.length_squared() < routes::FLAT_MIN { Vec2::ZERO } else { d.normalize() }
    };
    let dot = flat(-m.yaw.sin(), -m.yaw.cos()).dot(flat(first.x - m.position.x, first.z - m.position.z));
    if dot < TURN_DOT {
        m.router.set_pace(2, TURN_WALK);
        if log_on() {
            println!("t {:.2}: {}: its route's first entry {:.0} deg off its facing: walks for {TURN_WALK} s (FUN_00058740)", m.sim_time, unit_name(m), dot.clamp(-1.0, 1.0).acos().to_degrees());
        }
    }
}

/// State 4's filter on point `p` for member `me` at `at` (the wrapper 0x3a400c, FUN_00020200: a
/// point of kind 10, then the object 0x3a4008, 0x20410: within COVER_NEAR of the member, the
/// filter of FUN_00020170 (0x26240) and FUN_00020260):
///  - a cover point (kind 10) with its set-valid bit (+0x64 bit 5, `Node::cover`) and valid (+0x28
///    bit 2, 0x5e070), within COVER_NEAR of the member (3D, strictly);
///  - no other character of the squad has it for its own point (FUN_00045310 over the squad list,
///    the controlled one too; FUN_00020260's first test);
///  - of the AI members in FOLLOW that aren't walking to a cover point (state 5) or holding at one
///    (state 3 with their own point of kind 10), the member is the nearest to it (3D). FUN_00020260
///    walks the squad list keeping the first strictly nearer one, so of two at the same distance
///    the one first in the list has it: the demo's order is `mates`' (the squad record's own, the
///    order FUN_0005b9a0 joined them in, isn't read).
///
/// Not modelled: the leash (FUN_00023630 on brain+0x1ec, none on a squad member as far as is
/// read), the point's vtable +0x40 test, FUN_00048000 (the point in the brain's list at
/// brain+0x570) and the filter's exclusion list (FUN_00162d60, empty here).
fn cover_ok(g: &Graph, p: u32, me: Who, at: Vec3, mates: &[Mate]) -> bool {
    let n = &g.nodes[p as usize];
    if n.kind != nav::kind::COVER || !n.cover || !g.valid(p) || n.at.distance_squared(at) >= COVER_NEAR * COVER_NEAR {
        return false;
    }
    if mates.iter().any(|x| x.who != me && x.point == Some(p)) {
        return false;
    }
    let mine = n.at.distance_squared(at);
    let place = mates.iter().position(|x| x.who == me).unwrap_or(mates.len());
    !mates.iter().enumerate().any(|(k, x)| {
        x.who != me && x.follows && x.state != State::ToCover
            && !(x.state == State::Hold && x.point.is_some_and(|q| g.nodes[q as usize].kind == nav::kind::COVER))
            && {
                let d = n.at.distance_squared(x.at);
                d < mine || (d == mine && k < place)
            }
    })
}

/// State 5 (FUN_00022790): the speed record 1.0 and off; the route to the claimed point walked at
/// COVER_ARRIVE an entry and COVER_LAST for the cover point, its last, at the dash; at its end state
/// 2 now (brain+0x488 zeroed, the route reset); no route, state 2 (the next update). Not done: the
/// slow-down within 1.5 m of the point (FUN_0002fba0's speed record under 1, which only a walk
/// takes) and the facing the point gives within its own radius (its +0x64 bits 0-1 and its
/// orientation: brain+0x488 and +0x6d4, which the hold keeps).
fn to_cover(m: &mut Player, g: &mut Graph, dist: f32) -> Next {
    if !m.router.walking() {
        set(m, State::HoldBegin, "no route to its cover point", dist);
        return Next::Done(routes::Way::Stand);
    }
    m.follow.record = None;
    let s = routes::step(m, g, COVER_ARRIVE, COVER_LAST);
    if s.walking {
        let along = m.router.walker.as_ref().map_or(0.0, |w| w.remaining(g, m.position));
        return Next::Done(routes::Way::Go { target: s.target, along });
    }
    routes::stop(m, g);
    set(m, State::HoldBegin, "its route to its cover point ends", dist);
    Next::On
}

#[cfg(test)]
mod tests {
    use super::*;
    use bf_viewer::bf::level::WorldPoint;

    /// A world point named `name` of type `kind` at (x, 0, z) linking to `links`, valid.
    fn point(name: u32, kind: u32, x: f32, z: f32, links: &[u32]) -> WorldPoint {
        WorldPoint { name, kind, at: Vec3::new(x, 0.0, z), flags: 0xffff_4000, radius: -1.0, links: links.to_vec(), state: nav::VALID }
    }

    /// The plain world point's and the cover point's element types.
    const PLAIN: u32 = 0xF95A_FB93;
    const COVER: u32 = 0x126F_47A3;

    /// A member of the squad at `at` with its own point `point` in `mode`.
    fn member(c: usize, at: Vec3, point: Option<u32>, mode: Mode) -> Player {
        let mut m = Player::new(c);
        m.position = at;
        m.router.own.point = point;
        m.command.mode = mode;
        m
    }

    /// The leader standing at `at` on point `point`.
    fn leader(at: Vec3, point: Option<u32>) -> Lead {
        Lead { who: Who::Squad(3), at, point, class: 1 }
    }

    /// The five distances: WINGMAN's 8, 10, 7, 7, 4 and FOLLOW's 1.4 times them.
    #[test]
    fn the_five_distances() {
        assert_eq!(distances(Mode::Wingman), [8.0, 10.0, 7.0, 7.0, 4.0]);
        for (f, w) in distances(Mode::Follow).iter().zip(distances(Mode::Wingman)) {
            assert!((f - 1.4 * w).abs() < 1e-5);
        }
    }

    /// FUN_0005e670: a member goes before the first entry it's over 5 m nearer the controlled
    /// character than, when over 5 m farther than the entry before (the first entry has none);
    /// else at the end.
    #[test]
    fn follows_list_is_near_first_by_five_metres() {
        let (a, b, c, x) = (Who::Squad(0), Who::Squad(1), Who::Squad(2), Who::Npc(1, 0));
        let dist: HashMap<Who, f32> = [(a, 10.0), (b, 18.0), (c, 30.0)].into_iter().collect();
        let put = |far: f32| {
            let mut list = vec![a, b, c];
            insert(&mut list, x, far, &dist);
            list
        };
        assert_eq!(put(4.0), [x, a, b, c], "over 5 m nearer than the first");
        assert_eq!(put(24.0), [a, b, x, c], "over 5 m nearer than c and over 5 m farther than b");
        assert_eq!(put(20.0), [a, b, c, x], "not over 5 m farther than b");
        assert_eq!(put(9.0), [a, b, c, x], "not 5 m nearer than a, nor 5 m farther than it");
    }

    /// FUN_0005e540 with the tables: the first two entries follow the controlled character, the
    /// third the first; a member in no list, or in another mode, the controlled character. Both
    /// lists count (FUN_0005e310): a WINGMAN entry 2 follows WINGMAN's entry 0 in FOLLOW too, when
    /// that one is in FOLLOW's list and no entry before it there claims it. A fourth entry's number
    /// is the controlled character's 3, so entries 0 and 1 follow it (as read; the demo's lists hold
    /// three at most).
    #[test]
    fn the_third_entry_follows_the_first() {
        let (a, b, c, d) = (Who::Squad(0), Who::Squad(1), Who::Squad(2), Who::Npc(1, 0));
        let f = Formation { follow: vec![a, b, c], wingman: vec![b, a], ..Default::default() };
        assert_eq!(f.leader_of(a, Mode::Follow), None);
        assert_eq!(f.leader_of(b, Mode::Follow), None);
        assert_eq!(f.leader_of(c, Mode::Follow), Some(a));
        assert_eq!(f.leader_of(d, Mode::Follow), None, "in no list");
        assert_eq!(f.leader_of(a, Mode::Guard), None);
        assert!(f.first(b, Mode::Wingman) && !f.first(a, Mode::Wingman) && f.first(a, Mode::Follow));
        // a stale WINGMAN entry 2: FOLLOW's own order would make c its entry 0
        let f = Formation { follow: vec![c, b, a], wingman: vec![a, b, c], ..Default::default() };
        assert_eq!(f.leader_of(c, Mode::Follow), Some(a), "WINGMAN's entry 2 follows its entry 0");
        assert_eq!(f.leader_of(a, Mode::Follow), Some(c), "and FOLLOW's entry 2 its entry 0");
        let f = Formation { follow: vec![a, b, c, d], ..Default::default() };
        assert_eq!(f.leader_of(a, Mode::Follow), Some(d), "a fourth entry, number 3");
    }

    /// FUN_0005e8b0's two follow modes (0x5e908-0x5e9e8), through a frame's bookkeeping
    /// (`Formation::update`) with the orders in the squad list's slot order: Fire At Will to all puts
    /// Flint, Brutus and Hawk in WINGMAN's list and Flint, its first, says the walk line; Cover Me to
    /// all puts them in FOLLOW's and leaves WINGMAN's as it was; Fire At Will to Hawk alone then
    /// changes neither (she's in WINGMAN's list already): she's its third, so she follows Flint, and
    /// says no line (not its first); Flint, in FOLLOW, is FOLLOW's first and follows Tex. Stand Ground
    /// takes one out of both; a fresh squad (no mode change) is in neither.
    #[test]
    fn fire_at_will_cover_me_fire_at_will_to_hawk() {
        let at = |c: usize, x: f32| {
            let mut m = Player::new(c);
            m.position = Vec3::new(x, 0.0, 0.0);
            m
        };
        // (Brutus, Flint, Hawk as play.rs makes them, all within 5 m of each other's distance to Tex)
        let mut squad = vec![at(0, 4.0), at(1, 6.0), at(2, 7.0)];
        let (brutus, flint, hawk) = (squad[0].who(), squad[1].who(), squad[2].who());
        let mut f = Formation::default();
        assert!(!f.update(Vec3::ZERO, &squad), "a fresh squad joins no list");
        assert!(f.follow.is_empty() && f.wingman.is_empty());
        // (FUN_000c1280's order: the squad list's slots, Flint, Brutus, Hawk)
        let order = |squad: &mut [Player], who: &[usize], mode: Mode| {
            for &k in who {
                squad[k].command.set_mode(mode, None);
            }
        };
        order(&mut squad, &[1, 0, 2], Mode::Wingman);
        f.update(Vec3::ZERO, &squad);
        assert_eq!(f.wingman, [flint, brutus, hawk]);
        assert!(f.first(flint, Mode::Wingman) && squad[1].command.start_walk(), "Flint says the walk line");
        assert_eq!(f.leader_of(hawk, Mode::Wingman), Some(flint));
        order(&mut squad, &[1, 0, 2], Mode::Follow);
        f.update(Vec3::ZERO, &squad);
        assert_eq!(f.follow, [flint, brutus, hawk]);
        assert_eq!(f.wingman, [flint, brutus, hawk], "Cover Me leaves WINGMAN's list");
        order(&mut squad, &[2], Mode::Wingman);
        f.update(Vec3::ZERO, &squad);
        assert_eq!((&f.wingman[..], &f.follow[..]), (&[flint, brutus, hawk][..], &[flint, brutus, hawk][..]), "Fire At Will to Hawk changes neither");
        assert_eq!(f.leader_of(hawk, Mode::Wingman), Some(flint), "Hawk follows Flint");
        assert!(!f.first(hawk, Mode::Wingman) && squad[2].command.start_walk() && !squad[1].command.start_walk(),
                "Hawk isn't WINGMAN's first: the bit is clear but she's never asked; Flint, in FOLLOW, can't say it");
        assert!(f.first(flint, Mode::Follow) && f.leader_of(flint, Mode::Follow).is_none() && f.leader_of(brutus, Mode::Follow).is_none());
        order(&mut squad, &[0], Mode::Guard);
        f.update(Vec3::ZERO, &squad);
        assert_eq!((&f.wingman[..], &f.follow[..]), (&[flint, hawk][..], &[flint, hawk][..]), "Stand Ground: out of both");
        assert_eq!(f.leader_of(hawk, Mode::Wingman), None, "WINGMAN's second now: the controlled character");
    }

    /// The goal starts holding (state 2, then 3); the hold ends once the leader is beyond d0, and
    /// the chase plans a route to his own point and walks it at once.
    #[test]
    fn the_hold_ends_beyond_d0_and_the_chase_routes_to_the_leaders_point() {
        let pts = [point(1, PLAIN, 0.0, 0.0, &[2]), point(2, PLAIN, 10.0, 0.0, &[1, 3]), point(3, PLAIN, 20.0, 0.0, &[2])];
        let mut g = Graph::new(&pts);
        let mut m = member(1, Vec3::new(0.5, 0.0, 0.0), Some(0), Mode::Follow);
        // the leader 10 m off: within d0 (11.2), it holds
        let near = leader(Vec3::new(10.5, 0.0, 0.0), Some(1));
        assert_eq!(update(&mut m, &near, &[], Some(&mut g)), routes::Way::Stand);
        assert_eq!((m.follow.state, m.router.cap), (State::Hold, 1));
        // 20 m off: state 0 at the next update, a route to his point h 3, walked
        let far = leader(Vec3::new(20.5, 0.0, 0.0), Some(2));
        assert_eq!(update(&mut m, &far, &[], Some(&mut g)), routes::Way::Stand);
        assert_eq!(m.follow.state, State::Chase);
        assert!(matches!(update(&mut m, &far, &[], Some(&mut g)), routes::Way::Go { .. }));
        assert_eq!(m.router.walker.as_ref().and_then(|w| w.last()), Some(2), "the route ends at the leader's own point");
        // beyond d1 the speed record stays 1.25
        assert_eq!(m.follow.record, Some(speed::FOLLOW_BOOST));
    }

    /// In the chase, within d2 of a leader who isn't running, FOLLOW looks for cover and WINGMAN
    /// holds; a running leader is chased on, within d2 too, with the speed record off inside d1.
    #[test]
    fn the_chase_stops_within_d2_of_a_leader_who_isnt_running() {
        let pts = [point(1, PLAIN, 0.0, 0.0, &[2]), point(2, PLAIN, 10.0, 0.0, &[1, 3]), point(3, PLAIN, 20.0, 0.0, &[2])];
        for mode in [Mode::Follow, Mode::Wingman] {
            let mut g = Graph::new(&pts);
            let mut m = member(1, Vec3::new(0.5, 0.0, 0.0), Some(0), mode);
            m.follow.started = true;
            m.follow.state = State::Chase;
            let at = Vec3::new(20.5, 0.0, 0.0);
            assert!(matches!(update(&mut m, &leader(at, Some(2)), &[], Some(&mut g)), routes::Way::Go { .. }));
            // 6 m off (within both modes' d2) and running: on
            m.position = Vec3::new(14.5, 0.0, 0.0);
            let running = Lead { class: 3, ..leader(at, Some(2)) };
            assert!(matches!(update(&mut m, &running, &[], Some(&mut g)), routes::Way::Go { .. }));
            assert_eq!(m.follow.record, None, "inside d1");
            // standing: FOLLOW finds no cover point (none in the graph) and holds; WINGMAN holds
            assert_eq!(update(&mut m, &leader(at, Some(2)), &[], Some(&mut g)), routes::Way::Stand);
            assert_eq!(m.follow.state, State::Hold, "{mode:?}");
            assert!(!m.router.walking());
        }
    }

    /// State 4: the first cover point the leader's flood reaches that lies within 6 m of the member,
    /// isn't another's own point and has no nearer eligible member; walked to (state 5) and held.
    #[test]
    fn follow_takes_a_cover_point_near_it() {
        // the leader on 1; a cover point 3 m from the member (5, linked to its own point 2), one
        // 8.8 m off (4)
        let pts = [point(1, PLAIN, 0.0, 0.0, &[2, 4, 5]), point(2, PLAIN, -8.0, 0.0, &[1, 5]), point(4, COVER, -2.0, 6.0, &[1]), point(5, COVER, -8.0, 3.0, &[1, 2])];
        let mut g = Graph::new(&pts);
        let at = Vec3::new(-8.5, 0.0, 0.0);
        let mut m = member(1, at, Some(1), Mode::Follow);
        m.follow.started = true;
        m.follow.state = State::FindCover;
        let lead = leader(Vec3::ZERO, Some(0));
        // a nearer member in FOLLOW, chasing, takes it from it
        let rival = Mate { who: Who::Squad(2), at: Vec3::new(-8.0, 0.0, 2.0), point: Some(0), follows: true, state: State::Chase };
        assert_eq!(update(&mut m, &lead, &[rival], Some(&mut g)), routes::Way::Stand);
        assert_eq!((m.follow.state, m.follow.claimed), (State::Hold, None));
        // one walking to a cover point doesn't count
        m.follow.state = State::FindCover;
        let busy = Mate { state: State::ToCover, ..rival };
        assert!(matches!(update(&mut m, &lead, &[busy], Some(&mut g)), routes::Way::Go { .. }));
        assert_eq!((m.follow.state, m.follow.claimed), (State::ToCover, Some(3)));
        assert!(g.nodes[3].state & nav::CLAIMED != 0);
        // there: the hold, its claim kept; the leader beyond 1.5 x d0 of the cover ends it
        m.position = Vec3::new(-8.0, 0.0, 3.0);
        m.router.own.point = Some(3);
        assert_eq!(update(&mut m, &lead, &[], Some(&mut g)), routes::Way::Stand);
        assert_eq!(m.follow.state, State::Hold);
        let off = leader(Vec3::new(5.0, 0.0, 3.0), Some(0));
        assert_eq!(update(&mut m, &off, &[], Some(&mut g)), routes::Way::Stand);
        assert_eq!(m.follow.state, State::Hold, "13 m: within 1.5 x 11.2");
        let far = leader(Vec3::new(9.0, 0.0, 3.0), Some(0));
        update(&mut m, &far, &[], Some(&mut g));
        assert_eq!((m.follow.state, m.follow.claimed), (State::Chase, None));
        assert!(g.nodes[3].state & nav::CLAIMED == 0);
    }

    /// FUN_00058740 after state 4's plan: a member facing away from its route's first entry (yaw 0
    /// faces -z; its own point and the cover point lie +z of it, whichever the route starts with)
    /// walks, the pace level 2; one facing it keeps its level.
    #[test]
    fn the_turn_check_walks_a_member_facing_away() {
        for (yaw, cap) in [(0.0, 2), (std::f32::consts::PI, 1)] {
            let pts = [point(1, PLAIN, 0.0, 0.0, &[2, 5]), point(2, PLAIN, -8.0, 0.0, &[1, 5]), point(5, COVER, -8.0, 3.0, &[1, 2])];
            let mut g = Graph::new(&pts);
            let mut m = member(1, Vec3::new(-8.0, 0.0, -1.0), Some(1), Mode::Follow);
            m.yaw = yaw;
            m.follow.started = true;
            m.follow.state = State::FindCover;
            assert!(matches!(update(&mut m, &leader(Vec3::ZERO, Some(0)), &[], Some(&mut g)), routes::Way::Go { .. }));
            assert_eq!((m.follow.state, m.router.cap), (State::ToCover, cap), "yaw {yaw}");
        }
    }

    /// FUN_00020260's tie: of two eligible members at the same distance from a cover point, the
    /// first in the squad list has it.
    #[test]
    fn a_tie_goes_to_the_first_in_the_list() {
        let pts = [point(1, PLAIN, 0.0, 0.0, &[2, 5]), point(2, PLAIN, -8.0, 0.0, &[1, 5]), point(5, COVER, -8.0, 3.0, &[1, 2])];
        let g = Graph::new(&pts);
        let (a, b) = (Vec3::new(-9.0, 0.0, 3.0), Vec3::new(-7.0, 0.0, 3.0));
        let mate = |c: usize, at: Vec3| Mate { who: Who::Squad(c), at, point: Some(1), follows: true, state: State::Hold };
        let mates = [mate(0, a), mate(1, b)];
        assert!(cover_ok(&g, 2, Who::Squad(0), a, &mates));
        assert!(!cover_ok(&g, 2, Who::Squad(1), b, &mates));
    }

    /// Without a level's points the chase walks straight at the leader and holds within d2 of one
    /// standing (the test map's floor).
    #[test]
    fn the_straight_chase() {
        let mut m = member(1, Vec3::ZERO, None, Mode::Wingman);
        m.follow.started = true;
        m.follow.state = State::Chase;
        let lead = leader(Vec3::new(0.0, 0.0, -20.0), None);
        assert_eq!(update(&mut m, &lead, &[], None), routes::Way::Go { target: lead.at, along: 20.0 });
        m.position = Vec3::new(0.0, 0.0, -14.0);
        assert_eq!(update(&mut m, &lead, &[], None), routes::Way::Stand);
        assert_eq!(m.follow.state, State::Hold);
    }
}
