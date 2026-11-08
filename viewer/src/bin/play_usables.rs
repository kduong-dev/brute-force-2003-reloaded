//! Usable objects (#262, on the wall panels of #125 and #220): the placements with
//! reticule-action 1 (RACT_USE) that the player uses with X, the gate panels (`world-button-object`s)
//! and the campaign's 37 consoles (`game-object`s in 14 zones: e34's two, e23's three, e16's eight,
//! m14's eight ...). Written from default.xbe's code (the addresses are in the comments and in
//! viewer/docs/code-map/README.md) and checked against todo/262-consoles (xemu, e34's two
//! consoles).
//!
//!  - **The target** (the HUD's use tracker, the one a lying gun is found by, play_weapon_pickups.rs;
//!    FUN_000bf1e0 and FUN_000bf2f0): an object whose reticule-action is 1 is a candidate when
//!    the character is on its front side (FUN_000bf1e0: its model's hardpoints h_ee389721 and
//!    h_10f81d34 give it a facing, the horizontal way from the button to the use point,
//!    FUN_0015a580; the character must be on the side it points to), the use point is at most
//!    HEIGHT above or below the character's position and REACH across (the target it has at KEEP
//!    of its distance), and the way to the button lies within 60 degrees of the aim (the dot of
//!    the aim's heading with it, at least FRONT), with a clear line from the use point to the
//!    character (FUN_000de330, the object itself in its skip list). The use point is hardpoint
//!    h_ee389721 (FUN_0015a500), the object's position when the model has none; the ring's point
//!    and the aim's the button, h_10f81d34 (FUN_0015a480). The one most in front wins,
//!    but a lying gun more in front takes the frame (`weapon_pickups::use_target`). e34's console
//!    h_09da26aa: its use point is 1.03 m in front of its origin, so the prompt shows from 3.0 m
//!    off, as take01 has it (3.2 m from the origin at 1.7 s, none at 3.9 m).
//!  - **The prompt**: "Hold [X] to activate %s." (FUN_000aa600's case 1) with the object's display
//!    name, its type's h_1dd3a49d (`ObjectType::display`): "console" on e34's, e23's and e16's
//!    h_08e82f12 / h_070486ec / h_13d18d32, "Shrike console" on m09's and m14's h_efa01b1a (not
//!    seen in the game: the xemu take didn't get a prompt there), "panel" on the wall panels and
//!    e16's h_036f525a, "control pod" on m02_b's and m04_c's, "Meteor Control Center" on m14_c's
//!    and m14_d's. The ring (blue) sits on the button.
//!  - **The hold**: X held HOLD (0.3 s, the player controller's +0x50, 0x132f0c) on a console,
//!    once for each press (armed by the press, disarmed by the release and by the use). A tap of
//!    0.2 s does nothing (take03). The wall panels keep their 0.5 s (`PANEL_HOLD`).
//!  - **The use** (FUN_000a9710 -> the object's vtable +0x140, 0x1581f0: for a reticule-action 1 it
//!    calls +0x138 at once, no animation, no wait): a panel (0x163060) sends its own signal and data
//!    to the triggers listening to it, a console (FUN_00158140: FUN_00157fe0 plays the type's use sound and
//!    effect, h_153f90c9 and h_1f8b39ec, none on any of these types) USE_OBJECT with data -1, the
//!    user as the subject (FUN_001fe3d0). The level's routers, sounds, timers and anim-triggers
//!    take it from there (play_triggers.rs): e34's console h_09da26aa sends router h_14eeda94, which
//!    plays the gate's sound, one leaf's clip at once and a 5 s timer that starts the other
//!    leaf's, with a voice line whose first beeps the take hears 0.4 s after the press.
//!  - **Once**: a console is used once because the level's property trigger h_14cccfaa on its
//!    USE_OBJECT sets its reticule-action to 0 (0x20b730, `Triggers::property`; applied here): the
//!    prompt never comes back and a second hold does nothing (take05, three holds). A trigger's
//!    router can feed it from the other console too: using either of e34's kills both. The ones
//!    with no such trigger on their use (m14's, m09's, three of e16's ...) stay usable, as the code
//!    leaves them (nothing in FUN_00158140 counts a use): a second use sends USE_OBJECT again, and what
//!    its listeners do with one is theirs (the data's entries behind it are once: the score and anim
//!    triggers). The wall panels are one use too, by the demo's own rule (#125, as before the
//!    tracker: used, or the doors their signal reaches unchanged already opening, and no prompt).
//!
//! Test hooks: BF_TEST_USE=<s>[,<hold s>][;<s>[,<hold s>]...] holds use (E) from each time for its
//! hold, a second without one (play.rs); BF_DOOR_LOG=1 lists the panels and consoles with their
//! button, use point and facing, and logs each use target, use and property trigger's change of
//! an object's reticule-action; BF_TRIGGER_LOG=1 shows the USE_OBJECT and what it reaches.

use super::*;
use super::weapon_pickups::{HEIGHT, HOLD, KEEP, REACH};
use bf_viewer::bf::level::{Button, Level};
use bf_viewer::bf::weapon::WeaponModel;

/// The button hardpoint, h_10f81d34 (FUN_0015a480's: the ring's point and the point whose way
/// the aim must follow; FUN_0015a580's second): on the wall panels' archetype h_ee4c83c9 (the
/// green glow quad) and every console's.
const BUTTON: u32 = 0x10F8_1D34;
/// The use hardpoint, h_ee389721 (FUN_0015a500's: the point the reach and the line of sight are
/// measured to; FUN_0015a580's first): the panels' 0.55 m in front of the button, e34's console's
/// 0.53 m.
const USE_POINT: u32 = 0xEE38_9721;
/// How far in front of the aim the way to a use target's button lies: the dot of the aim's
/// heading with it at least this (FUN_000bf2f0: 0.5 for every target but a pickup).
const FRONT: f32 = 0.5;
/// How long X is held on a wall panel (s): the demo's own 0.5, left as it was before the tracker
/// (#125; hints.md): the code's hold is `HOLD` for every target, but take11's panel (the tutorial's
/// gate) sounds 0.8 s after the press where the consoles' takes have the chirps 0.4 s after it, and
/// the demo's panels are left to that take until it's explained.
const PANEL_HOLD: f32 = 0.5;

/// A usable object as the level places it (`Level::buttons`): see the module notes.
pub(super) struct Usable {
    /// its name, what the level's triggers call it
    name: u32,
    /// a wall panel (world-button-object), else a console (a game-object)
    panel: bool,
    /// what the prompt calls it ("console"): its type's display name
    label: String,
    /// the event its use sends: a panel's own signal and data, a console's USE_OBJECT and -1
    signal: i64,
    data: i64,
    /// the button (FUN_0015a480: hardpoint h_10f81d34, the position without one), world: the ring's
    /// point and the aim's
    button: Vec3,
    /// the use point (FUN_0015a500: h_ee389721, the position without one), world: the reach's and
    /// the sight line's end
    reach: Vec3,
    /// the way it faces (FUN_0015a580: the horizontal way from the button to the use point), unit;
    /// none without both hardpoints
    facing: Option<Vec3>,
    /// its reticule-action now (RACT_): 1 as the level places it; a property trigger sets it
    /// (`Triggers::property`), a 0 takes it out of the use candidates
    action: i64,
    /// the doors its signal reaches unchanged (`Level::button_opens`: indices into `Doors::list`),
    /// for a panel's prompt
    opens: Vec<usize>,
    /// a panel has been used (the demo's one use for a panel, see the module notes)
    used: bool,
    /// how long X is held on it to use it (s): `HOLD` for a console, `PANEL_HOLD` for a panel
    hold: f32,
}

impl Usable {
    /// The usable made from a level's placement: its model's hardpoints in the world, its display
    /// name, the doors it opens; BF_DOOR_LOG lists it.
    fn new(b: &Button, level: &Level, game: &Game, doors: &[DoorState], log: bool) -> Usable {
        let model = game.object_meshes.get(&b.kind).and_then(|&a| WeaponModel::load(game, a).ok());
        let point = |k: u32| model.as_ref().and_then(|m| m.hardpoints.get(&k)).map(|h| b.transform.transform_point3(h.point));
        let origin = b.transform.w_axis.truncate();
        let (button, reach) = (point(BUTTON), point(USE_POINT));
        let facing = button.zip(reach).and_then(|(a, u)| Vec3::new(u.x - a.x, 0.0, u.z - a.z).try_normalize());
        let label = game.object_types.get(&b.kind).map(|t| t.display).filter(|&d| d != 0).and_then(|d| game.strings.get(&d)).cloned()
            .unwrap_or_else(|| if b.console { "console" } else { "panel" }.to_string());
        let reached = level.button_opens(b);
        let opens = doors.iter().enumerate().filter(|(_, d)| reached.iter().any(|x| x.0 == d.door.name)).map(|(i, _)| i).collect();
        let u = Usable { name: b.name, panel: !b.console, label, signal: b.signal, data: b.data, button: button.unwrap_or(origin),
                         reach: reach.unwrap_or(origin), facing, action: 1, opens, used: false, hold: if b.console { HOLD } else { PANEL_HOLD } };
        if log {
            let what = if u.panel { "panel".to_string() } else { format!("console ({:?}, type h_{:08x})", u.label, b.kind) };
            println!("{what} h_{:08x} button at {:.1} use point {:.1} facing {} signal {}: opens {:x?}", u.name, u.button, u.reach,
                     u.facing.map_or("none".to_string(), |f| format!("{f:.2}")), u.signal, reached);
        }
        u
    }

    /// Whether the character at `at` (its position, the hips) with the aim heading `look` (across,
    /// unit) has it in reach and in front, and how far in front (the dot of the heading with the
    /// way to the button; the tracker keeps the one with the most): FUN_000bf1e0's facing test and
    /// FUN_000bf2f0's height, reach (x KEEP for the target it has, `held`) and aim tests, without
    /// the sight line (`sees`).
    fn in_front(&self, at: Vec3, look: Vec2, held: bool) -> Option<f32> {
        // (the character must be on the side the object faces: FUN_000bf1e0, the dot with the
        // way from the button to the character not above 0 out)
        if self.facing.is_some_and(|f| (at - self.button).dot(f) <= 0.0) {
            return None;
        }
        let across = Vec2::new(self.reach.x - at.x, self.reach.z - at.z).length() * if held { KEEP } else { 1.0 };
        if (self.reach.y - at.y).abs() > HEIGHT || across > REACH {
            return None;
        }
        let to = Vec2::new(self.button.x - at.x, self.button.z - at.z);
        let dot = if to.length() > 1e-4 { to.normalize().dot(look) } else { 1.0 };
        (dot >= FRONT).then_some(dot)
    }

    /// Whether the line between the use point and the character at `at` is clear: FUN_000bf2f0 calls
    /// FUN_000de330 with FUN_0015a500's point as the start (ECX) and the character's position as the
    /// end (EAX), the candidate in the filter's skip list, and it is clear when nothing is met short
    /// of the end. The demo's ray is the level's collision alone, which has no console in it.
    fn sees(&self, at: Vec3) -> bool {
        let to = at - self.reach;
        ray_hit(self.reach, to.normalize_or_zero(), to.length()).is_none()
    }

    /// Whether it's offered at all: its reticule-action is 1, and a panel not used and not
    /// already opening what it opens (the demo's rule for the panels).
    fn offered(&self, doors: Option<&Doors>) -> bool {
        self.action == 1
            && !(self.panel && (self.used || (!self.opens.is_empty() && doors.is_some_and(|d| self.opens.iter().all(|&i| d.list[i].opening())))))
    }
}

/// The level's usable objects and the use key's hold on the one in reach (`UsePanel` has the
/// prompt, the ring and the target they draw): made by play.rs `setup`, none on the test floor.
#[derive(Resource, Default)]
pub(super) struct Usables {
    list: Vec<Usable>,
    /// X is armed: pressed, and not yet released or used (the player controller's +0xa0 bit 4)
    armed: bool,
    /// X was down last frame
    was_down: bool,
    /// how long X has been held on the target (s)
    held: f32,
    /// the target it had last frame (its place in `list`): kept to KEEP of its distance
    target: Option<usize>,
    /// BF_DOOR_LOG is set
    log: bool,
}

impl Usables {
    /// The level's usable objects, in the file's order (`Level::buttons`), listed by BF_DOOR_LOG;
    /// `doors` are the level's, for the doors each one's signal reaches.
    pub(super) fn new(level: &Level, game: &Game, doors: &[DoorState]) -> Usables {
        let log = std::env::var("BF_DOOR_LOG").is_ok();
        Usables { list: level.buttons.iter().map(|b| Usable::new(b, level, game, doors, log)).collect(), log, ..default() }
    }

    /// The candidate the tracker keeps for a character at `at` (its position, the hips) with the aim
    /// heading `look`: of those offered (`Usable::offered`) in reach and in front (`Usable::in_front`,
    /// the target it has at KEEP) with a clear line (`sees`), the one most in front (FUN_000bf2f0's
    /// caller 0xbf110 keeps the lowest 1 - dot): its place in `list` and its dot. The first of two
    /// as far in front wins.
    fn pick(&self, at: Vec3, look: Vec2, doors: Option<&Doors>, sees: impl Fn(&Usable) -> bool) -> Option<(usize, f32)> {
        let mut best: Option<(usize, f32)> = None;
        for (i, x) in self.list.iter().enumerate() {
            if !x.offered(doors) {
                continue;
            }
            let Some(dot) = x.in_front(at, look, self.target == Some(i)) else { continue };
            if best.is_some_and(|b| b.1 >= dot) || !sees(x) {
                continue;
            }
            best = Some((i, dot));
        }
        best
    }
}

/// The use tracker's frame for the panels and consoles, then the hold: the target (and with it the
/// prompt, the ring's point and the ring's colour) before the lying guns' (`use_target`, which a
/// gun more in front takes the frame from), the hold after it.
pub fn plugin(app: &mut App) {
    app.add_systems(Update, target.after(doors).before(super::weapon_pickups::use_target).before(update_player).run_if(in_state(AppState::Playing)))
        .add_systems(Update, activate.after(super::weapon_pickups::use_target).before(update_player).run_if(in_state(AppState::Playing)));
}

/// Each frame after `doors` (which clears the prompt): the property triggers' OPERATEs since last
/// frame taken (an object whose reticule-action they set to 0 is no longer a candidate), then the
/// candidate most in front of the controlled character (see the module notes) is its target, with
/// the prompt "Hold [X] to activate %s." and the ring on its button. Not scoped, in a cutscene,
/// dead, or while a gun's pickup is under way (FUN_000a9910: the character using something).
fn target(usables: Option<ResMut<Usables>>, mut tr: ResMut<triggers::Triggers>, player: Res<Player>, doors: Option<Res<Doors>>,
          mut use_panel: ResMut<UsePanel>) {
    let Some(mut usables) = usables else { return };
    let u = &mut *usables;
    for r in tr.take_reticules() {
        if let Some(x) = u.list.iter_mut().find(|x| x.name == r.object) {
            if u.log {
                println!("t {:.2}: {} h_{:08x}'s reticule-action {} -> {} (property trigger h_{:08x})", player.sim_time, if x.panel { "panel" } else { "console" }, x.name,
                         x.action, r.action, r.trigger);
            }
            x.action = r.action;
        }
    }
    let offered = !player.dead && player.loaded.is_some() && !script::blocks_input(&player) && player.scope < 0.5
        && player.gun_use.as_ref().is_none_or(|g| !g.using());
    let (at, look) = (player.position, Vec2::new(-player.cam_yaw.sin(), -player.cam_yaw.cos()));
    let best = if offered { u.pick(at, look, doors.as_deref(), |x| x.sees(at)) } else { None };
    if u.log && best.map(|b| b.0) != u.target {
        match best {
            Some((i, dot)) => println!("t {:.2}: use target {} h_{:08x} ({:?}) at {:.2}, dot {dot:.2}", player.sim_time, if u.list[i].panel { "panel" } else { "console" },
                                       u.list[i].name, u.list[i].label, u.list[i].button),
            None => println!("t {:.2}: no use target", player.sim_time),
        }
    }
    u.target = best.map(|b| b.0);
    if let Some((i, _)) = best {
        let x = &u.list[i];
        use_panel.prompt = Some(Prompt::Activate(x.label.clone()));
        use_panel.target = Some(x.button);
        use_panel.target_id = i as u64 + 1;
        use_panel.ring.colour = RingColour::Usable;
    }
}

/// Each frame after the guns' target: X's press arms the hold, its release or a use disarms it;
/// held HOLD s on a target that's still a usable's (a gun more in front took the frame, else) the
/// object is used: a panel's signal and data, or a console's USE_OBJECT and -1, go to the
/// triggers listening to it, the character as the subject (play_triggers.rs `run` delivers them
/// next frame). A panel is then used up for the demo's rule.
fn activate(usables: Option<ResMut<Usables>>, player: Res<Player>, time: Res<Time>, mut use_panel: ResMut<UsePanel>, mut events: ResMut<triggers::TriggerEvents>) {
    let Some(mut usables) = usables else { return };
    let u = &mut *usables;
    // (X held while the skill says the character isn't free to act, FUN_00117210 through its +0x14:
    // the use update that FUN_00120290 runs asks it at 0x1207b9, so the Wrath of Garner on, as
    // Flint's targeting, lets no hold arm or run: `skills::lets_act`, #208)
    let down = player.use_held && super::skills::lets_act(&player);
    if down && !u.was_down {
        u.armed = true;
        u.held = 0.0;
    }
    if !down {
        u.armed = false;
        u.held = 0.0;
    }
    u.was_down = down;
    // (the target id is a panel's or console's place + 1; a gun's is its entity's bits)
    let id = use_panel.target_id;
    let Some(i) = (id >= 1 && id <= u.list.len() as u64).then(|| id as usize - 1) else {
        u.held = 0.0;
        return;
    };
    if !u.armed {
        return;
    }
    u.held += frame_dt(&time);
    use_panel.held = u.held;
    if u.held < u.list[i].hold {
        return;
    }
    (u.armed, u.held, use_panel.held) = (false, 0.0, 0.0);
    let x = &mut u.list[i];
    // its event to its listeners (0x163060 for a panel: its signal and data; FUN_00158140 for a
    // console: USE_OBJECT, -1), the user as the subject
    events.0.push(triggers::ObjectEvent { from: x.name, signal: x.signal, subject: player.trigger_name, data: x.data });
    if x.panel {
        x.used = true;
    }
    if u.log {
        println!("t {:.2}: {} h_{:08x} ({:?}) used: {} {} to its listeners", player.sim_time, if x.panel { "panel" } else { "console" }, x.name, x.label,
                 bf_viewer::bf::triggers::sig::name(x.signal), if x.data == -1 { String::new() } else { format!("[data {}]", x.data) });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// e34's console h_09da26aa as the level places it (decompiled/xml/e34/levels-e34.xml): its
    /// transform, and its archetype h_08e82f12's hardpoints (objects-e34.xml): the button at
    /// (-0.497, 1.223, 0), the use point at (-1.027, 1.252, 0.0002), in front of it (its -x).
    fn e34_console() -> Usable {
        let m = bf_viewer::bf::level::transform(&[-0.409923, 0.0, -0.912121, 0.0, 1.0, 0.0, 0.912121, 0.0, -0.409923, -34.49934, 33.61102, 28.17535]).unwrap();
        let (button, reach) = (m.transform_point3(Vec3::new(-0.497337, 1.223195, 0.0)), m.transform_point3(Vec3::new(-1.027083, 1.252068, 0.000182)));
        Usable { name: 1, panel: false, label: "console".into(), signal: 14, data: -1, button, reach,
                 facing: Vec3::new(reach.x - button.x, 0.0, reach.z - button.z).try_normalize(), action: 1, opens: vec![], used: false, hold: HOLD }
    }

    /// The aim heading of a camera yaw (play.rs `doors`' own: -sin, -cos across).
    fn look(yaw: f32) -> Vec2 {
        Vec2::new(-yaw.sin(), -yaw.cos())
    }

    /// take01 and take04: Tex walks up to e34's console from (-33.3, 23.7) toward it (bearing 105
    /// degrees: the camera looks +z, yaw pi), the prompt shows from video 1.70 s at (-33.68,
    /// 25.18), 3.2 m from the console's origin, and not at z 24.4 (3.9 m). The use point (the
    /// hardpoint 1.03 m in front of the origin) is 2.1 m and 2.84 m away across: REACH 2.5 splits
    /// them. The hips are 1.05 m over the floor (the demo's `GROUND`).
    #[test]
    fn the_prompt_shows_from_three_metres_off() {
        let c = e34_console();
        let hips = |x: f32, z: f32| Vec3::new(x, 33.62 + 1.05, z);
        let aim = look(std::f32::consts::PI);
        assert!(c.in_front(hips(-33.68, 25.18), aim, false).is_some());
        assert!(c.in_front(hips(-33.30, 24.40), aim, false).is_none());
        // the target it has is kept a while longer (KEEP: out to 2.5 / 0.7 = 3.6 m across)
        assert!(c.in_front(hips(-33.30, 24.40), aim, true).is_some());
        // 2 m from it (take05: the whole time Tex stands there, z 26.9-28.0)
        assert!(c.in_front(hips(-33.28, 26.9), aim, false).is_some());
    }

    /// The object faces its -x: from behind it (the other side of the console) it's no target,
    /// nor with the aim turned away (the way to the button under 60 degrees off the heading) or
    /// from a floor 2 m up.
    #[test]
    fn it_is_used_from_in_front_looking_at_it() {
        let c = e34_console();
        let aim = look(std::f32::consts::PI);
        // (behind: through the console to z 30.5)
        assert!(c.in_front(Vec3::new(-35.0, 34.7, 30.5), look(0.0), false).is_none());
        // looking 90 degrees to the side
        assert!(c.in_front(Vec3::new(-33.68, 34.67, 25.6), look(std::f32::consts::FRAC_PI_2), false).is_none());
        // 1 m off the heading is 25 degrees: still a target
        assert!(c.in_front(Vec3::new(-33.0, 34.67, 25.6), aim, false).is_some());
        // on a floor 2 m up
        assert!(c.in_front(Vec3::new(-33.68, 36.7, 25.6), aim, false).is_none());
    }

    /// Of two usables in reach the one more in front is the target (the tracker keeps the lowest
    /// 1 - dot), a dead one (reticule-action 0) is passed over, a blocked line passes one over,
    /// and the target it has is kept to 3.6 m across.
    #[test]
    fn the_one_most_in_front_wins() {
        let a = e34_console();
        // (the same console 1.5 m along x: the character stands 2 m in front of both)
        let mut b = e34_console();
        (b.name, b.button.x, b.reach.x) = (2, b.button.x + 1.5, b.reach.x + 1.5);
        let list = vec![a, b];
        let mut usables = Usables { list, ..default() };
        let at = Vec3::new(-33.2, 34.67, 25.6);
        let towards = |u: &Usable| Vec2::new(u.button.x - at.x, u.button.z - at.z).normalize();
        let (aim_a, aim_b) = (towards(&usables.list[0]), towards(&usables.list[1]));
        assert_eq!(usables.pick(at, aim_a, None, |_| true).map(|p| p.0), Some(0));
        assert_eq!(usables.pick(at, aim_b, None, |_| true).map(|p| p.0), Some(1));
        // a dead one is passed over, a blocked one too
        usables.list[0].action = 0;
        assert_eq!(usables.pick(at, aim_a, None, |_| true).map(|p| p.0), Some(1));
        usables.list[0].action = 1;
        assert_eq!(usables.pick(at, aim_a, None, |u| u.name != 1).map(|p| p.0), Some(1));
        // nothing in front of an aim turned away
        assert_eq!(usables.pick(at, -aim_a, None, |_| true), None);
    }

    /// A console whose reticule-action a property trigger set to 0 isn't offered; a panel used, or
    /// whose doors are all opening, isn't either (the demo's rule).
    #[test]
    fn a_console_is_offered_until_its_reticule_action_goes() {
        let mut c = e34_console();
        assert!(c.offered(None));
        c.action = 0;
        assert!(!c.offered(None));
        let mut p = e34_console();
        p.panel = true;
        assert!(p.offered(None));
        p.used = true;
        assert!(!p.offered(None));
    }
}
