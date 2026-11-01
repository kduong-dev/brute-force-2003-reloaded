//! The squad's follow (#331; viewer/docs/squad.md "The follow", nav.md "The squad on routes"):
//! GOAL_WINGMAN_FOLLOW's update FUN_00020b10, the goal of the squad's two follow modes, FOLLOW
//! (Cover Me, the default) and WINGMAN (Fire At Will), in place of the demo's formation places.
//! Every function cited is in viewer/docs/code-map.md ("AI"); the reading is todo/343-first-walk-
//! line/leads.md §1-§2 and #331's own (the code map's rows for the states).
//!
//!  - **The leader** (FUN_00020b10's start, FUN_0005e540): the squad's formation record (the
//!    squad's +0x28) keeps a list per follow mode, WINGMAN's at +4 and FOLLOW's at +0x14
//!    (`Formation`). A member's leader is the controlled character but for the third entry, which
//!    follows the first (the tables 0x3ffc40 / 0x3ffc70: FOLLOW's entries follow 3, 3, 0, 3 being
//!    the controlled character and 0 the first entry's; WINGMAN's the same while the first entry
//!    has no route point). The lists start empty and fill as orders change the members' modes, so
//!    after one the squad runs as a column: #33 take06's Hawk, the third after Cover Me to Flint,
//!    Brutus and Hawk, waited for Flint to be 13 m off while Tex was 27 m off, trailed Flint by
//!    15-16 m without the boost Tex's distance would have given her, and stopped 8.8 m from him;
//!    take04's three, fresh, each followed Tex.
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
//! FOLLOW) into FOLLOW's list in that order at the start, where a level's list starts empty, to
//! stage the column an order leaves (#33 take06's Cover Me to Flint, Brutus and Hawk:
//! flint,brutus,hawk).

use bf_viewer::bf::nav::{self, Dest, Graph};
use brain::Mode;

use super::*;

/// The formation's bookkeeping runs before the characters' step, after the frame's input and its
/// orders (a mode change made later in the frame is seen at the next).
pub fn plugin(app: &mut App) {
    app.init_resource::<Formation>()
        .add_systems(Update, sync.after(read_input).before(update_player).run_if(in_state(AppState::Playing)));
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
/// Who each entry of a follow list follows (FUN_0005e310 over the tables 0x3ffc40 / 0x3ffc70, read
/// with FUN_0005e540): None the squad's controlled character (the tables' 3), Some(k) the list's
/// entry k. FOLLOW's 3, 3, 0; WINGMAN's the same while its first entry has no route point (with
/// one 3, 0, 3: never in the demo).
const LEADS: [Option<usize>; 3] = [None, None, Some(0)];

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
/// member was last added as. FUN_0005e8b0 adds a member by its mode: FOLLOW's list by distance to
/// the controlled character (FUN_0005e670, `insert`), WINGMAN's at its end (FUN_0005eab0; at its
/// front when the one entry is in state 12 or 14, never in the demo), each taking it out of the
/// other list first (FUN_0005e1e0 at 0x5e937); any other mode takes it out of both (FUN_0005e600).
/// The callers: the brain's set-up at the spawn (FUN_00017b60), every mode change (FUN_00018040's
/// end), the brain released by the player (FUN_00018230); a character the player takes leaves both
/// (FUN_00018150). At a level's start the lists are empty: the spawn's insert finds no controlled
/// character to measure from (FUN_0005e670 adds nothing without FUN_0005bca0's member; read, and
/// the takes agree: #33 take04's three, fresh from the start, each set off by Tex's own distance,
/// d0, Brutus 0.3 s after Flint and Hawk as he stood 4 m nearer, where take06's Hawk, after Cover
/// Me to the three in the squad list's order, the third entry, waited for Flint to be 13 m off
/// while Tex was 27 m off, trailed him by 15-16 m and stopped 8.8 m from him). So a member joins
/// at its first mode change or its release by the player (`Command::joined` over 0). The demo
/// drops the dead.
#[derive(Resource, Default, Debug)]
pub struct Formation {
    pub wingman: Vec<Who>,
    pub follow: Vec<Who>,
    /// each member's mode and mode change (`Command::joined`) when it was last added
    seen: HashMap<Who, (Mode, u64)>,
    /// BF_TEST_FORMATION applied
    staged: bool,
}

impl Formation {
    /// `who` out of both lists (FUN_0005e600).
    fn remove(&mut self, who: Who) {
        self.wingman.retain(|w| *w != who);
        self.follow.retain(|w| *w != who);
    }

    /// `who` added by `mode` (FUN_0005e8b0), `far` its distance (3D) to the controlled character
    /// and `dist` the others'.
    fn add(&mut self, who: Who, mode: Mode, far: f32, dist: &HashMap<Who, f32>) {
        self.remove(who);
        match mode {
            Mode::Follow => insert(&mut self.follow, who, far, dist),
            Mode::Wingman => self.wingman.push(who),
            _ => {}
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

    /// `who`'s leader in `mode` (FUN_0005e540): the entry `LEADS` names for its place in its list,
    /// None the controlled character (a member in no list, past the tables' three entries, or the
    /// tables' 3).
    pub fn leader_of(&self, who: Who, mode: Mode) -> Option<Who> {
        let list = self.list(mode)?;
        let i = list.iter().position(|w| *w == who)?;
        LEADS.get(i).copied().flatten().and_then(|k| list.get(k).copied()).filter(|&w| w != who)
    }

    /// Whether `who` is the first entry of its mode's list (FUN_0005ea10).
    pub fn first(&self, who: Who, mode: Mode) -> bool {
        self.list(mode).is_some_and(|l| l.first() == Some(&who))
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

/// The formation's bookkeeping each frame, before the squad's AI (see `Formation`): a member whose
/// mode or mode change differs from what it was added as is added again, by its mode, in the order
/// the changes were made (`Command::joined`; a fresh brain's 0 is no join); one no longer a member
/// (dead, out of the world, taken by the player) leaves both lists. Then each member's leader and
/// first-entry bit for its follow (`Follow::leader`, `Follow::first`). BF_TEST_FORMATION's order is
/// applied once. The claims the dead and the goals' restarts gave up lose their bits here, where
/// the graph is at hand.
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
    let me = player.position;
    let member = |u: &Player| (u.in_squad() || follower(u)) && !u.dead && !u.absent;
    let dist: HashMap<Who, f32> = squad.0.iter().filter(|u| member(u)).map(|u| (u.who(), u.position.distance(me))).collect();
    let f = &mut *formation;
    let mut changed = false;
    let gone: Vec<Who> = f.seen.keys().filter(|w| !dist.contains_key(w)).copied().collect();
    for w in gone {
        f.remove(w);
        f.seen.remove(&w);
        changed = true;
    }
    let mut adds: Vec<(u64, usize, Who, Mode)> = squad.0.iter().enumerate().filter(|(_, u)| member(u) && u.command.joined > 0)
        .filter(|(_, u)| f.seen.get(&u.who()) != Some(&(u.command.mode, u.command.joined)))
        .map(|(k, u)| (u.command.joined, k, u.who(), u.command.mode)).collect();
    adds.sort_by_key(|a| (a.0, a.1));
    for (stamp, _, who, mode) in adds {
        f.add(who, mode, dist[&who], &dist);
        f.seen.insert(who, (mode, stamp));
        changed = true;
    }
    if !f.staged && !dist.is_empty() {
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
    if changed && log_on() {
        let names = |l: &[Who]| l.iter().map(|w| squad.0.iter().find(|u| u.who() == *w).map_or("?".into(), unit_name)).collect::<Vec<_>>().join(", ");
        println!("t {:.2}: formation: FOLLOW [{}], WINGMAN [{}]", player.sim_time, names(&f.follow), names(&f.wingman));
    }
    for u in squad.0.iter_mut() {
        u.follow.leader = f.leader_of(u.who(), u.command.mode);
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
/// (FUN_000416a0), the point claimed (the last given up) and state 5; else state 2, the claim
/// given up. Either way the next state now.
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

/// State 4's filter on point `p` for member `me` at `at` (the wrapper 0x3a400c, FUN_00020200: a
/// point of kind 10, then the object 0x3a4008, 0x20410: within COVER_NEAR of the member, the
/// filter of FUN_00020170 (0x26240) and FUN_00020260):
///  - a cover point (kind 10) with its set-valid bit (+0x64 bit 5, `Node::cover`) and valid (+0x28
///    bit 2, 0x5e070), within COVER_NEAR of the member (3D, strictly);
///  - no other character of the squad has it for its own point (FUN_00045310 over the squad list,
///    the controlled one too; FUN_00020260's first test);
///  - of the AI members in FOLLOW that aren't walking to a cover point (state 5) or holding at one
///    (state 3 with their own point of kind 10), the member is the nearest to it (3D).
///
/// Not modelled: the leash (FUN_00023630 on brain+0x1ec, none on a squad member as far as is
/// read), the point's vtable +0x40 test, FUN_00048000 on brain+0x510 and the filter's exclusion
/// list (FUN_00162d60, empty here).
fn cover_ok(g: &Graph, p: u32, me: Who, at: Vec3, mates: &[Mate]) -> bool {
    let n = &g.nodes[p as usize];
    if n.kind != nav::kind::COVER || !n.cover || !g.valid(p) || n.at.distance_squared(at) >= COVER_NEAR * COVER_NEAR {
        return false;
    }
    if mates.iter().any(|x| x.who != me && x.point == Some(p)) {
        return false;
    }
    let mine = n.at.distance_squared(at);
    !mates.iter().any(|x| {
        x.who != me && x.follows && x.state != State::ToCover
            && !(x.state == State::Hold && x.point.is_some_and(|q| g.nodes[q as usize].kind == nav::kind::COVER))
            && n.at.distance_squared(x.at) < mine
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
    /// third the first; a member in no list, or past three, the controlled character.
    #[test]
    fn the_third_entry_follows_the_first() {
        let (a, b, c, d) = (Who::Squad(0), Who::Squad(1), Who::Squad(2), Who::Npc(1, 0));
        let f = Formation { follow: vec![a, b, c, d], wingman: vec![b, a], ..Default::default() };
        assert_eq!(f.leader_of(a, Mode::Follow), None);
        assert_eq!(f.leader_of(b, Mode::Follow), None);
        assert_eq!(f.leader_of(c, Mode::Follow), Some(a));
        assert_eq!(f.leader_of(d, Mode::Follow), None, "past the tables' three entries");
        assert_eq!(f.leader_of(a, Mode::Guard), None);
        assert!(f.first(b, Mode::Wingman) && !f.first(a, Mode::Wingman) && f.first(a, Mode::Follow));
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
