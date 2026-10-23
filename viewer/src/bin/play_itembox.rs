//! The item box and the item list (#196): the game's B button (Tab: a tap steps to the next item,
//! a hold opens the list and pauses the game), the scroll of the open list, the Black button's
//! Medkit shortcut (B), the use button's refusals ("Inventory is empty", "Not ready", "No
//! Medkits"), NEW, and the HUD widget that draws the box and the list's four slots. Read from
//! default.xbe (each function is cited where it's used, with a row in viewer/docs/code-map.md)
//! and checked against the #196 takes (todo/196-inventory-hud: spec.md, notes.md).
//!
//!  - **The selection** is the inventory front's wanted pair (inv+0x1c0, `Player::item`; None is
//!    the game's (10, -1)) over the squad's one store in (category, index) order (play.rs
//!    `items`). A stack is listable while it holds any (a stack that reaches 0 loses its type)
//!    and its type's h_1852ab6c lists it (FUN_0014d680). Next and previous wrap round
//!    (FUN_0014d960 / FUN_0014d7c0: `next`, `prev`).
//!  - **Each frame of the world** (the front's FUN_0014db60, `settle`): a "none" selection becomes
//!    the first listable stack; the shown pair (+0x1b8, which "can use now" checks) follows the
//!    wanted unless a use is under way (+0x1c8 bit 0); a shown stack that ran out sets both to
//!    none, so the box is blank that frame and shows the first stack the next (take09 5.317 s).
//!  - **The controls** (the player controller 0x1332a0's item section, `item_controls`): B held
//!    counts its hold timer (ctrl+0x40) and keeps a grenade's charge at 0; let go under 0.25 s it
//!    steps to the next item; at 0.25 s the list opens (FUN_000ac080) and the game pauses until
//!    B is let go (FUN_000ac160). Black selects the 80-point Medkit (5, 0), uses it as if the use
//!    button were pressed, and restores the item before it on its release. The use button (G) is
//!    the item's: a grenade charges and is thrown on the release (play.rs `update_player`), the rest
//!    are used on the press; a B press or an item changed under it leaves it dead until it's let go
//!    (ctrl+0x68, `UseLock`).
//!  - **The widget** (0xabc60 -> FUN_000ac220, `widget`), on the HUD's clock (it runs through the
//!    list's pause, take02): the open level rises 2 a second while the list is open and falls 3
//!    after; the four list slots slide out by int(open x 70) units each and fade in as open^3; up
//!    are the next and the one after, left the previous and the one before; a slot repeating an
//!    earlier one's stack is hidden (box, left1, up1, left2, up2: FUN_000ac7f0). The open list
//!    scrolls with the stick (`ItemInput::stick`) past 0.5, a step at once and then every 0.25 s.
//!  - **NEW** (0xad0c0): a stack's NEW flag is set from its making and cleared the first time any
//!    slot shows it, which starts that slot's 2 s label: on while fract(2 x time left) >= 0.3, its
//!    alpha 255 x min(1, 2 x time left). The stack shown first at a level's start is cleared
//!    without a label (take27: its label ran out in the intro).
//!  - **The input** (`ItemInput`, the product owner's call D5): the buttons the item section reads,
//!    which the keyboard and mouse fill now (Tab B, G the left trigger, B Black, W/D and S/A the
//!    stick's next and previous, the wheel) and the pad will under #154.
//!
//! Test hooks: BF_TEST_ITEM_INPUT="<s>:<input>:<hold s>;..." holds an input from that time
//! (tab, lt, black, next, prev; on the hooks' clock, the sim time plus the time the list's pause
//! held it); BF_TEST_ITEM_LIST=1 holds Tab from the start; BF_TEST_NEXT_GRENADE=<s>[,<s>...] taps
//! Tab (0.1 s) at each time; BF_TEST_THROW=<s>[,<hold s>...] (with BF_TEST_GOTO) holds G;
//! BF_TEST_ITEM_USE=<s>[,<s>...] presses G for one frame at each time for an item that isn't a
//! grenade; BF_TEST_INV / BF_TEST_INV_NEW set the pool and its NEW flags at the start
//! (play_inventory.rs `stock`); BF_ITEM_LOG=1 logs the controls' events and the widget's state on
//! every change and every 0.1 s while the list is out.

use std::collections::HashSet;

use super::fonts::{record_text, Across, RecordText, BODY12};
use super::grenade::GrenadeKits;
use super::hud::{flipped, texture, texture_grey, HudImages, HudRoot};
use super::*;

// ---- the controls (the player controller 0x1332a0's item section) ----

/// B's hold that opens the list (s): ctrl+0x40 counts the frame's dt while B is held and opens
/// the list once it reaches 0x3a5028's 0.25 (0x134151); a release before it is a tap (0x1341c7).
/// take04's sweep: presses of 100-260 ms stepped, 300 and 400 ms opened the list.
const B_HOLD: f32 = 0.25;
/// The hold timer once the list is open (0x134195: 0x3e99999a, 0.3).
const LIST_TIMER: f32 = 0.3;
/// A tap's length for BF_TEST_NEXT_GRENADE (s): well under B_HOLD.
const TEST_TAP: f32 = 0.1;

// ---- the widget (FUN_000ab8d0, FUN_000ac220, FUN_000ac080, FUN_000ac370) ----

/// The open level's rates, a second of the HUD's clock: +0x70 rises 2 x dt while the list is open
/// and falls 3 x dt after (FUN_000ac220): out in 0.5 s, gone 0.33 s after the release (take02:
/// 0.054 -> 1.0 over 0.518 s of the app clock, 1.0 -> 0 in 0.34 s).
const OPEN_RATE: f32 = 2.0;
const CLOSE_RATE: f32 = 3.0;
/// The widget's hold timer (+0x74): FUN_000ac080 puts it to 0.5 each frame B holds the list
/// open; FUN_000ac220 counts it down (closing the list should it run out) and scrolls only while
/// it's at least SCROLL_FRESH (B held within the last 0.1 s) and the open level is past
/// SCROLL_OPEN (the first 0.375 s of the slide take no scroll).
const HOLD_REFRESH: f32 = 0.5;
const SCROLL_FRESH: f32 = 0.4;
const SCROLL_OPEN: f32 = 0.75;
/// The scroll (FUN_000ac370): the stick past DEAD_ZONE on its larger axis steps at once, then its
/// ramp (+0x78) runs RAMP_RATE a second and steps again each time it passes 1: every 0.25 s and a
/// frame (8 game frames at the game's 30 a second; take08's steps 0.302 s apart on the log at
/// the take's 0.906 speed, 0.27 s of game time).
const DEAD_ZONE: f32 = 0.5;
const RAMP_RATE: f32 = 4.0;
/// The list slots' step (units): +0x68 / +0x6c, 70.0 (FUN_000ab8d0; 0xabcbd on a full screen);
/// FUN_000ac500 puts slot k at -(k + 1) x int(open x 70). take25 9.5 s: the slots' counts 105 and
/// 210 px (70 and 140 units) from the box's.
const SLOT_STEP: f32 = 70.0;
/// The box's place: the HUD safe area's bottom-right corner less its size (0xabc60:
/// FUN_000b28e0 / FUN_000b2770, the safe area x 51-592, y 39-439 on a full screen), and its size
/// (FUN_000ab580: 64 x 64 on a full screen). take25 1.5 s: the frame's ink at x 529.3-591.3,
/// y 376-436.7 units.
const BOX_AT: Vec2 = Vec2::new(528.0, 375.0);
const SLOT_SIZE: f32 = 64.0;
/// A slot's panel texture (0xab614 in FUN_000ab580: common's 64 x 64 DXT3, a glass square with a
/// cut top-left corner), drawn white at 255 for the box, (30, 30, 30) at 153 for a list slot
/// (0xab63b / 0xab65c).
const PANEL: u32 = 0xE169_27FB;
const LIST_PANEL_GREY: f32 = 30.0 / 255.0;
const LIST_PANEL_ALPHA: f32 = 153.0 / 255.0;
/// A grenade type's icon if its definition has none (the Frag's h_e5ec3f1f), and the Medkit's
/// if the carried type has none (h_f647bbef, both Medkits' in the data).
const FRAG_ICON: u32 = 0xFE20_B919;
const MEDKIT_ICON: u32 = 0xF647_BBEF;
/// The texts' colours: FUN_000b87a0's bytes c0 d7 ff ff, (192, 215, 255), while the character can
/// use the item now; (150, 70, 70) otherwise (0xaca4a); NEW's (255, 160, 106) (FUN_000acfb0's
/// ff6aa0ff). The bytes go to the text's +0x38 (red), +0x3a (green) and +0x39 (blue): NEW's
/// orange and take25's (194, 210, 244) name agree. The slot's texts draw a shadow (their
/// +0x3b bit 4, as the message line's): black, a unit down and right (the static texts'
/// FUN_0010a420; its colour here isn't traced).
const BLUE: [u8; 3] = [192, 215, 255];
const RED: [u8; 3] = [150, 70, 70];
const NEW_ORANGE: [u8; 3] = [255, 160, 106];
/// Where a slot's texts sit (FUN_000ab740): the name centred, its top 64 - its height + 1; the
/// count's right edge 4 in from the slot's (COUNT_IN), its top 13 above the name's (COUNT_UP);
/// NEW at (10, 2) (FUN_000acfb0). A line of the 12-point record is its line height and one
/// (fonts.rs `Record::height`, 20).
const COUNT_IN: f32 = 4.0;
const COUNT_UP: f32 = 13.0;
const NEW_AT: Vec2 = Vec2::new(10.0, 2.0);
/// NEW's label time (s): 0xad0c0 puts +0x50 to 2.0 the frame it clears a stack's flag, counts it
/// down by the HUD's dt, and shows the label while fract(2 x time left) >= NEW_BLINK (0x3a50b0):
/// on 0.35 s, off 0.15 s, four times (take23 at 60 fps: 0.33 / 0.165 / 0.36 / 0.12 / 0.36 / 0.135
/// / 0.39 s).
const NEW_TIME: f32 = 2.0;
const NEW_BLINK: f32 = 0.3;
/// "NEW" (h_0ba41874, which FUN_000acfb0 sets at 0xad089-0xad09e).
const S_NEW: u32 = 0x0BA4_1874;
/// The widget's sounds (FUN_000bb220, the HUD entry's player: 2D, at their library volume): the
/// list opening (FUN_000ac080, once), the next and the previous item (FUN_000abfc0 / FUN_000ac020,
/// only when the pair changed), the list closing (FUN_000ac160). common/sounds-common.xml.
const SOUND_OPEN: u32 = 0x18FC_2DB8;
const SOUND_NEXT: u32 = 0xF8F9_3C5E;
const SOUND_PREV: u32 = 0x1AB2_FC23;
const SOUND_CLOSE: u32 = 0xEF4B_D74D;

// ---- the messages (the HUD entry's message line, its widget +0x114) ----

/// How long an item message shows (s): the controller's FUN_000bcfe0(entry, 3.0) at each of them
/// (0x1344fd, 0x13496a, 0x1349cc); the line fades over its last 0.5 s (its update 0xb33b0,
/// play_hud.rs). Restarted by each press (take16).
pub const MESSAGE_TIME: f32 = 3.0;
/// "Inventory is empty" (LT with nothing selected, 0x1349b2), "No Medkits" (Black with no (5, 0)
/// stack, 0x1344ef), "Not ready" (FUN_000abbb0: the shown pair isn't the wanted one).
const S_INVENTORY_EMPTY: u32 = 0x1393_EBEA;
const S_NO_MEDKITS: u32 = 0xF450_2B28;
const S_NOT_READY: u32 = 0xEC28_E39E;

pub fn plugin(app: &mut App) {
    app.init_resource::<ItemBox>()
        .init_resource::<ItemInput>()
        .add_systems(OnEnter(AppState::Playing), (
            (|mut commands: Commands| {
                commands.insert_resource(ItemBox::default());
                commands.insert_resource(ItemInput::default());
            }).after(setup),
            spawn_widget.after(snapshot_entities).in_set(super::hud::HudSetup),
        ))
        .add_systems(OnExit(AppState::Playing), |mut b: ResMut<ItemBox>| {
            b.opened = false;
            set_paused(PAUSE_ITEMS, false);
        })
        .add_systems(Update, (
            item_input.after(read_input).after(super::testtools::gate_input),
            item_controls.after(item_input).after(squad_control).before(update_player),
            settle.after(update_player).after(super::pickups::PickupSystems).after(super::grenade::GrenadeSystems)
                .after(super::inventory::use_other),
            widget.after(settle).before(play_sounds),
            draw.after(widget),
        ).run_if(in_state(AppState::Playing)));
}

/// The item section's buttons this frame (the product owner's call D5): what the player
/// controller's item section reads. The keyboard and mouse fill it now (`item_input`: Tab the B
/// button, G the left trigger, B the Black button, W/D and S/A the stick's next and previous,
/// the wheel's notches) and the test hooks; the pad fills the same under #154 (play_hints.rs
/// `BINDINGS`: East, LeftTrigger2, RightTrigger; the left stick and the d-pad scroll, take05 /
/// take07).
#[derive(Resource, Default, Clone, Copy, Debug)]
pub struct ItemInput {
    /// B (VB 12), the use button LT (VB 7) and Black (VB 63) held
    pub b: bool,
    pub lt: bool,
    pub black: bool,
    /// the stick the open list scrolls by (the pad device's +0x78 / +0x7c that FUN_000ac370 reads):
    /// x right, y down; right or up is the next item (take05). The keys fill x only: W or D +1,
    /// S or A -1
    pub stick: Vec2,
    /// the wheel's notches this frame, up + (the next item: the product owner's call D2)
    pub wheel: i32,
}

/// The use button's lock (ctrl+0x68): the stack the use button is on while it's held, or "another"
/// (-1: a B press, a refused use, the empty inventory). While it's another and not the selected
/// stack, the use button does nothing until it's let go (0x134689-0x1346b1: the charge zeroed, the
/// lock cleared once LT is up): take19's B tap during a charge left LT dead to its release.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
enum UseLock {
    #[default]
    Free,
    On(Item),
    Blocked,
}

/// One of the widget's five slots (FUN_000ab410; the box, then up 1, up 2, left 1, left 2: the
/// widget's +0x84 ... +0x94): its stack (+0x7c / +0x80; None the game's (10, -1)), whether it's
/// hidden (+0x13 bit 0: FUN_000ac5e0, FUN_000ac7f0; the box when it has none), and its NEW
/// label's time left and the stack it started for (0xad0c0's +0x50 / +0x4c).
#[derive(Clone, Copy, Debug, Default)]
struct Slot {
    pair: Option<Item>,
    hidden: bool,
    new_time: f32,
    new_of: Option<Item>,
}

/// The slots' order in FUN_000ac7f0's duplicate test: the box, left 1, up 1, left 2, up 2 (a
/// later slot showing an earlier one's stack is hidden). Indices into `ItemBox::slots`.
const DEDUPE_ORDER: [usize; 5] = [0, 3, 1, 4, 2];

/// The item box's state: the controller's (the hold timer, the list mode, the use lock, Black's
/// saved selection), the inventory front's shown pair, the store's NEW flags, the widget's (the
/// open level, the hold timer, the scroll ramp, the slots), and the test hooks' and log's.
/// A level's: reset as each map starts.
#[derive(Resource)]
pub struct ItemBox {
    /// B's hold timer (ctrl+0x40) and the list mode (ctrl+0xa0 bit 5: the controller's movement,
    /// look and item sections off, bits 0-2)
    b_timer: f32,
    list_mode: bool,
    /// the use button's lock (ctrl+0x68) and whether it was held last frame (a press is its
    /// first frame)
    lock: UseLock,
    lt_was: bool,
    /// Black's saved selection (ctrl+0x6c / +0x70; None the game's (10, -1)), whether its use has
    /// been made (ctrl+0xa1 bit 0) and whether Black was held last frame
    saved: Option<Item>,
    black_used: bool,
    black_was: bool,
    /// the front's shown pair (inv+0x1b8): what the box's "can use now" checks
    shown: Option<Item>,
    /// the stacks whose NEW flag (the stack's +0x18) is cleared; every other's is set
    seen: HashSet<Item>,
    /// the widget: the open level (+0x70), its hold timer (+0x74), the scroll's ramp (+0x78), the
    /// list open (+0x7c bit 0) and the game paused by it (+0x7c bit 2)
    open: f32,
    hold: f32,
    ramp: f32,
    opened: bool,
    paused_game: bool,
    slots: [Slot; 5],
    /// the controlled character last frame (a hand-over closes the list: FUN_000c8fa0)
    character: Option<usize>,
    /// the time the list's pause has held the world's clock (s): the hooks' clock is the sim time
    /// plus it
    paused_time: f32,
    /// BF_TEST_ITEM_INPUT's entries (read once), BF_TEST_ITEM_USE's last press
    tests: Option<Vec<TestInput>>,
    last_use_press: f32,
    /// BF_ITEM_LOG: on, the state last logged and when (the HUD's clock)
    log: bool,
    logged: String,
    log_clock: f32,
    log_at: f32,
}

impl Default for ItemBox {
    fn default() -> Self {
        ItemBox {
            b_timer: 0.0, list_mode: false, lock: UseLock::Free, lt_was: false, saved: None, black_used: false, black_was: false,
            shown: None, seen: HashSet::new(), open: 0.0, hold: 0.0, ramp: 0.0, opened: false, paused_game: false,
            slots: [Slot::default(); 5], character: None, paused_time: 0.0, tests: None, last_use_press: -1.0,
            log: std::env::var("BF_ITEM_LOG").is_ok(), logged: String::new(), log_clock: 0.0, log_at: 0.0,
        }
    }
}

impl ItemBox {
    /// The level's start: every stack NEW (the game builds them with the flag set, FUN_0014ac90)
    /// but `first`, the one the box shows first, whose label ran out unseen during the intro
    /// (take27 13.8 s: no label, its flag 0); or, with BF_TEST_INV_NEW=<name>,..., NEW only on
    /// those (`named` finds a name's stack).
    pub fn start(&mut self, all: &[Item], first: Option<Item>, named: impl Fn(&str) -> Option<Item>) {
        self.seen.clear();
        match std::env::var("BF_TEST_INV_NEW") {
            Ok(v) => {
                let new: Vec<Item> = v.split(',').filter_map(|n| named(n.trim())).collect();
                self.seen.extend(all.iter().copied().filter(|i| !new.contains(i)));
            }
            Err(_) => self.seen.extend(first),
        }
        self.shown = None;
    }
}

/// A BF_TEST_ITEM_INPUT entry (and BF_TEST_NEXT_GRENADE's taps): from when on the hooks' clock,
/// which input, for how long, and whether it has been down yet (held for at least a frame).
#[derive(Clone, Copy, Debug)]
struct TestInput {
    at: f32,
    what: TestKey,
    hold: f32,
    down: bool,
}

/// What a BF_TEST_ITEM_INPUT entry holds: Tab (B), G (LT), B (Black), the stick's next or
/// previous.
#[derive(Clone, Copy, Debug, PartialEq)]
enum TestKey {
    B,
    Lt,
    Black,
    Next,
    Prev,
}

/// BF_TEST_ITEM_INPUT's entries ("<s>:<input>:<hold s>;...") and BF_TEST_NEXT_GRENADE's taps.
fn read_tests() -> Vec<TestInput> {
    let mut out: Vec<TestInput> = std::env::var("BF_TEST_ITEM_INPUT").ok().iter().flat_map(|v| v.split(';').filter_map(|e| {
        let mut it = e.split(':').map(str::trim);
        let at = it.next()?.parse::<f32>().ok()?;
        let what = match it.next()?.to_ascii_lowercase().as_str() {
            "tab" | "b" => TestKey::B,
            "lt" | "g" => TestKey::Lt,
            "black" => TestKey::Black,
            "next" => TestKey::Next,
            "prev" => TestKey::Prev,
            _ => return None,
        };
        let hold = it.next().and_then(|h| h.parse::<f32>().ok()).unwrap_or(TEST_TAP);
        Some(TestInput { at, what, hold, down: false })
    }).collect::<Vec<_>>()).collect();
    // BF_TEST_NEXT_GRENADE=<s>[,<s>...]: a Tab tap at each time (the T key it pressed is gone: the
    // game steps every item with B, the product owner's call D3)
    if let Ok(v) = std::env::var("BF_TEST_NEXT_GRENADE") {
        out.extend(v.split(',').filter_map(|x| x.trim().parse::<f32>().ok()).map(|at| TestInput { at, what: TestKey::B, hold: TEST_TAP, down: false }));
    }
    out
}

/// The hooks' clock: the sim time, which stops under the list's pause, plus the time the pause
/// held it (so a hold's end comes while the world is stopped).
fn hook_time(p: &Player, b: &ItemBox) -> f32 {
    p.sim_time + b.paused_time
}

/// The item widget's clock step (s): the HUD's, which runs through the list's pause (take02: the
/// slide while the app clock is still): the real frame time, the capture's fixed step while
/// capturing (none in its warm-up), none while something else stops the game's clock (a menu over
/// the game: the HUD isn't updated).
fn hud_dt(time: &Time) -> f32 {
    if paused_by_other(PAUSE_ITEMS) {
        return 0.0;
    }
    if std::env::var("BF_CAPTURE").is_ok() {
        if WARMUP.load(std::sync::atomic::Ordering::Relaxed) > 0 { 0.0 } else { capture_step() }
    } else {
        time.delta_secs().min(0.1)
    }
}

/// Whether the keyboard drives the character (`read_input`'s own test): not under the autopilot
/// (BF_TEST_GOTO, BF_AUTOPILOT), whose hooks drive it instead.
fn autopilot_on() -> bool {
    test_goto().is_some() || std::env::var("BF_AUTOPILOT").is_ok()
}

/// The item section's buttons this frame (`ItemInput`): the keyboard's (not under another menu,
/// while a cutscene or a script holds the character, under the autopilot or in the free camera:
/// as `read_input`'s keys), then the hooks'. In a test session B is the test tools' (the NPC
/// tool's dummy / fight, play_testtools.rs), so the Black button there is the hooks' only.
#[allow(clippy::too_many_arguments)]
fn item_input(keys: Res<ButtonInput<KeyCode>>, scroll: Res<AccumulatedMouseScroll>, player: Res<Player>, mut input: ResMut<ItemInput>,
              mut boxed: ResMut<ItemBox>, tools: Option<Res<super::testtools::Tools>>, test: Option<Res<super::testmap::TestMap>>) {
    let b = &mut *boxed;
    let mut i = ItemInput::default();
    let live = !paused_by_other(PAUSE_ITEMS) && !script::blocks_input(&player) && !autopilot_on()
        && tools.as_ref().is_none_or(|t| t.freecam.is_none());
    if live {
        i.b = keys.pressed(KeyCode::Tab);
        i.lt = keys.pressed(KeyCode::KeyG);
        i.black = keys.pressed(KeyCode::KeyB) && test.is_none();
        let next = keys.any_pressed([KeyCode::KeyW, KeyCode::KeyD]);
        let prev = keys.any_pressed([KeyCode::KeyS, KeyCode::KeyA]);
        i.stick.x = next as i32 as f32 - prev as i32 as f32;
        if scroll.delta.y != 0.0 {
            i.wheel = scroll.delta.y.signum() as i32;
        }
    }
    let t = hook_time(&player, b);
    // BF_TEST_ITEM_INPUT and BF_TEST_NEXT_GRENADE
    let tests = b.tests.get_or_insert_with(read_tests);
    for e in tests.iter_mut() {
        if t >= e.at && (t < e.at + e.hold || !e.down) {
            e.down = true;
            match e.what {
                TestKey::B => i.b = true,
                TestKey::Lt => i.lt = true,
                TestKey::Black => i.black = true,
                TestKey::Next => i.stick.x = 1.0,
                TestKey::Prev => i.stick.x = -1.0,
            }
        }
    }
    // BF_TEST_ITEM_LIST=1: Tab held from the start (the list opens and the game pauses)
    if std::env::var("BF_TEST_ITEM_LIST").is_ok() {
        i.b = true;
    }
    // BF_TEST_THROW=<s>[,<hold s>][,<s>,<hold s>...]: with BF_TEST_GOTO, G held from s for hold s
    // (CHARGE_TIME by default: a full charge); BF_AUTOPILOT's own throw
    if test_goto().is_some() {
        if let Ok(v) = std::env::var("BF_TEST_THROW") {
            let v: Vec<f32> = v.split(',').filter_map(|x| x.trim().parse().ok()).collect();
            i.lt |= v.chunks(2).any(|c| (c[0]..c[0] + c.get(1).copied().unwrap_or(CHARGE_TIME)).contains(&t));
        }
    } else if std::env::var("BF_AUTOPILOT").is_ok() {
        i.lt |= AUTOPILOT_THROW.contains(&t);
    }
    // BF_TEST_ITEM_USE=<s>[,<s>...]: G for one frame at each time, for an item that isn't a
    // grenade (it owns the use for the run, as before #196)
    if let Ok(v) = std::env::var("BF_TEST_ITEM_USE") {
        let due = v.split(',').filter_map(|x| x.trim().parse::<f32>().ok()).filter(|&s| t >= s && b.last_use_press < s)
            .fold(None, |a: Option<f32>, s| Some(a.map_or(s, |a| a.max(s))));
        if let Some(s) = due {
            b.last_use_press = s;
            i.lt |= !matches!(player.item, Some(Item::Grenade(_)));
        }
    }
    *input = i;
}

/// The (category, index) of an item's stack (its type's group-type and h_e5f51266): a grenade
/// kit's, the Medkit's (5, 0), another item's (play_inventory.rs `item_info`).
pub fn key(game: &Game, kits: Option<&GrenadeKits>, item: Item) -> (i64, i64) {
    match item {
        Item::Grenade(k) => kits.and_then(|g| g.0.get(k)).map_or((2, k as i64), |g| (g.def.group_type, g.def.item_index)),
        Item::Medkit => (5, 0),
        Item::Other(kind) => super::inventory::item_info(game, kind).map_or((10, -1), |i| (i.group, i.index)),
    }
}

/// The listable stacks, in the store's (category, index) order: each with a count, as play.rs's
/// `items` lists them.
pub fn listable(p: &Player) -> Vec<Item> {
    items(p).into_iter().filter(|&i| item_count(p, i) > 0).collect()
}

/// The next listable stack after `from` (FUN_0014d960 with its wrap): the first after its place
/// in the order, else round to the first; from none, the first. None when nothing is listable.
pub fn next(p: &Player, from: Option<Item>) -> Option<Item> {
    let all = items(p);
    let at = from.and_then(|f| all.iter().position(|&i| i == f));
    let after = at.map_or(0, |a| a + 1);
    all[after.min(all.len())..].iter().chain(all.iter()).copied().find(|&i| item_count(p, i) > 0)
}

/// The listable stack before `from` (FUN_0014d7c0 with FUN_0014c300's list): the last before its
/// place, else round to the last; from none, the last.
pub fn prev(p: &Player, from: Option<Item>) -> Option<Item> {
    let all = items(p);
    let at = from.and_then(|f| all.iter().position(|&i| i == f)).unwrap_or(all.len());
    all[..at].iter().rev().chain(all.iter().rev()).copied().find(|&i| item_count(p, i) > 0)
}

/// The first listable stack (FUN_0014da70 from none): what a level starts on (AC-04: e34's Sonic
/// 4, e40's Frag 5, the tutorial's Frag 3) and what follows a stack that ran out.
pub fn first(p: &Player) -> Option<Item> {
    next(p, None)
}

/// The selection stepped forward or back (FUN_000abfc0 / FUN_000ac020 -> FUN_0014da70 /
/// FUN_0014d8f0), with the step's sound when the pair changed. Returns whether it did.
fn step(p: &mut Player, game: &Game, forward: bool) -> bool {
    let before = p.item;
    p.item = if forward { next(p, before) } else { prev(p, before) };
    let changed = p.item != before;
    if changed {
        hud_sound(p, game, if forward { SOUND_NEXT } else { SOUND_PREV });
    }
    changed
}

/// A HUD sound (FUN_000bb220): 2D, at its library entry's volume (h_eff0deab).
fn hud_sound(p: &mut Player, game: &Game, id: u32) {
    let volume = game.sounds.params(id).map_or(1.0, |s| s.volume);
    p.sound_queue.push((id, volume));
}

/// A message on the HUD's message line for MESSAGE_TIME (FUN_000bcfe0(entry, 3.0)): the string
/// table's text for `id`, else `fallback`.
fn message(status: &mut UsePanel, game: &Game, id: u32, fallback: &str) {
    let text = game.strings.get(&id).cloned().unwrap_or_else(|| fallback.to_string());
    status.message = Some((text, MESSAGE_TIME));
}

/// The list opens (FUN_000ac080, each frame B holds it): the widget's hold timer to 0.5; the
/// first time the open sound, the game paused (single player: the session's pause through the app,
/// 0x7a320) and the list flagged open.
fn open_list(b: &mut ItemBox, p: &mut Player, game: &Game) {
    b.hold = HOLD_REFRESH;
    if !b.opened {
        hud_sound(p, game, SOUND_OPEN);
        set_paused(PAUSE_ITEMS, true);
        b.paused_game = true;
        b.opened = true;
        if b.log {
            println!("item t={:.3}: the list opens (B held {:.3} s), the game paused", hook_time(p, b), b.b_timer.min(B_HOLD + 0.1));
        }
    }
}

/// The list closes (FUN_000ac160: B let go, a hand-over, the hold timer run out): the close sound,
/// the game unpaused if the list paused it, the hold timer 0. The slots then fade out (`widget`).
fn close_list(b: &mut ItemBox, p: &mut Player, game: &Game) {
    if b.opened {
        b.opened = false;
        hud_sound(p, game, SOUND_CLOSE);
        if b.paused_game {
            b.paused_game = false;
            set_paused(PAUSE_ITEMS, false);
        }
        b.hold = 0.0;
        if b.log {
            println!("item t={:.3}: the list closes on {}", hook_time(p, b), item_name_plain(p, b.slots[0].pair));
        }
    }
}

/// An item for the log: its stack's kind ("grenade 2", "Medkit", "h_11433446") or "none".
fn item_name_plain(_p: &Player, item: Option<Item>) -> String {
    match item {
        None => "none".into(),
        Some(Item::Grenade(k)) => format!("grenade {k}"),
        Some(Item::Medkit) => "Medkit".into(),
        Some(Item::Other(kind)) => format!("h_{kind:08x}"),
    }
}

/// The player controller's item section, each frame for the controlled character (0x1332a0: B at
/// 0x1340e1-0x13422f, Black at 0x134316-0x13460e, the use button at 0x13460e-0x134a30, the list
/// mode at 0x1350ff-0x1351a9). It decides `Player::throw_held` (a grenade's charge and throw,
/// play.rs `update_player`) and `Player::item_use` (the other items' use on the press:
/// play_pickups.rs `use_medkit`, play_inventory.rs `use_other`).
#[allow(clippy::too_many_arguments)]
fn item_controls(time: Res<Time>, input: Res<ItemInput>, mut player: ResMut<Player>, mut boxed: ResMut<ItemBox>,
                 mut status: ResMut<UsePanel>, game: Res<GameData>) {
    let dt = frame_dt(&time);
    let (p, b, g) = (&mut *player, &mut *boxed, &game.0);
    let i = *input;
    // a hand-over (a squad switch, the death camera's): the old character's list closes and its
    // Black shortcut ends (FUN_000c8fa0 closes the old entry's list; the selection itself goes with
    // control, play.rs `squad_control`)
    if b.character != Some(p.character) {
        if b.character.is_some() {
            close_list(b, p, g);
        }
        b.character = Some(p.character);
        b.list_mode = false;
        b.b_timer = 0.0;
        b.saved = None;
        b.black_used = false;
        b.lock = UseLock::Free;
    }
    // the section runs for a living character whose controls are on (0x1340e5 bit 2, 0x134101's
    // char+0x70 bit 0); a menu over the game takes the controls (a list open under it closes)
    let on = !p.dead && p.loaded.is_some() && !p.stand_in && !script::blocks_input(p) && !paused_by_other(PAUSE_ITEMS);
    if !on {
        if b.list_mode {
            close_list(b, p, g);
            b.list_mode = false;
        }
        b.b_timer = 0.0;
        (p.throw_held, p.item_use) = (false, false);
        b.lt_was = i.lt;
        b.black_was = i.black;
        return;
    }
    // ---- the list (0x1350ff): B held refreshes it (FUN_000ac080); let go, it closes, the
    // controller's sections come back and the hold timer is 0. Nothing is picked: the selection
    // is what the scroll left (take08)
    if b.list_mode {
        if i.b {
            open_list(b, p, g);
        } else {
            close_list(b, p, g);
            b.list_mode = false;
            b.b_timer = 0.0;
        }
        (p.throw_held, p.item_use) = (false, false);
        b.lt_was = i.lt;
        b.black_was = i.black;
        return;
    }
    // ---- B: held, its timer counts and a grenade's charge stays 0 (the use button's lock set:
    // take19's charge zeroed at the B press); at B_HOLD the list opens; let go before it, the next
    // item (take01's ten taps)
    if i.b {
        if b.b_timer >= 0.0 {
            b.b_timer += dt;
            if b.b_timer >= B_HOLD {
                open_list(b, p, g);
                b.list_mode = true;
                b.b_timer = LIST_TIMER;
            }
            p.charge = 0.0;
            b.lock = UseLock::Blocked;
        }
    } else {
        if b.b_timer > 0.0 && b.b_timer < B_HOLD {
            let was = p.item;
            let changed = step(p, g, true);
            p.charge = 0.0;
            b.lock = UseLock::Blocked;
            if b.log {
                println!("item t={:.3}: B tap ({:.3} s): {} -> {}{}", hook_time(p, b), b.b_timer, item_name_plain(p, was), item_name_plain(p, p.item),
                         if changed { "" } else { " (unchanged: no sound)" });
            }
        }
        b.b_timer = 0.0;
    }
    if b.list_mode {
        (p.throw_held, p.item_use) = (false, false);
        b.lt_was = i.lt;
        b.black_was = i.black;
        return;
    }
    // ---- Black (VB 63): the use button reads 0 this frame; a charge is dropped; with Tex's or
    // Flint's skill on, the skill's toggle instead (vtable +0x7c, #35); with no Medkit (5, 0) "No
    // Medkits"; else the Medkit selected (the item before it saved), then used as if the use
    // button were pressed (its value 1.0 for the frame); let go, the saved item back
    let mut lt = i.lt;
    let mut forced = false;
    if i.black {
        lt = false;
        if p.charge > 0.0 {
            p.charge = 0.0;
            if b.log {
                println!("item t={:.3}: Black: the charge dropped", hook_time(p, b));
            }
        } else if skills::ended_by_actions(p) {
            if !b.black_was && !p.skill.request && p.skill.toggling.is_none() {
                p.skill.request = true;
                if b.log {
                    println!("item t={:.3}: Black: the skill's toggle (its +0x7c)", hook_time(p, b));
                }
            }
        } else if p.medkits <= 0 {
            if !b.black_used {
                message(&mut status, g, S_NO_MEDKITS, "No Medkits");
                b.lock = UseLock::Blocked;
                p.charge = 0.0;
                if b.log && !b.black_was {
                    println!("item t={:.3}: Black: No Medkits (no (5, 0) stack)", hook_time(p, b));
                }
            }
        } else if p.item != Some(Item::Medkit) && b.saved.is_none() {
            // (from "none" nothing is saved, so nothing comes back)
            b.saved = p.item;
            p.item = Some(Item::Medkit);
            if b.log {
                println!("item t={:.3}: Black: Medkit selected ({} saved)", hook_time(p, b), item_name_plain(p, b.saved));
            }
        } else {
            // FUN_001210c0, the character can use it now: no use or other action under way
            let can = p.using.is_none() && p.throwing.is_none() && p.reloading.is_none() && p.switching.is_none() && p.second.is_none()
                && skills::lets_act(p);
            if (can || !b.black_used) && b.lock == UseLock::Free {
                forced = true;
                b.black_used = true;
            }
        }
    } else {
        if b.black_was {
            // (FUN_0014d730: back only if it's still listable)
            if let Some(s) = b.saved.take() {
                if item_count(p, s) > 0 {
                    p.item = Some(s);
                }
                if b.log {
                    println!("item t={:.3}: Black let go: {} back", hook_time(p, b), item_name_plain(p, p.item));
                }
            }
        }
        b.saved = None;
        b.black_used = false;
    }
    b.black_was = i.black;
    let press = (lt && !b.lt_was) || forced;
    b.lt_was = i.lt;
    // ---- the use button (0x13460e)
    let current = p.item;
    let held = lt || forced;
    let blocked = match b.lock {
        UseLock::Free => false,
        UseLock::On(x) => Some(x) != current,
        UseLock::Blocked => true,
    };
    if blocked {
        // another item's or a B press's lock: nothing until the button is let go
        if !held {
            b.lock = UseLock::Free;
        }
        p.charge = 0.0;
        (p.throw_held, p.item_use) = (false, false);
        return;
    }
    let Some(item) = current else {
        // nothing selected: "Inventory is empty" while it's held (shown again each frame, so
        // 3 s from the release; take21: on 0.2 s after the press, about 3 s)
        if held {
            message(&mut status, g, S_INVENTORY_EMPTY, "Inventory is empty");
            b.lock = UseLock::Blocked;
            if b.log && press {
                println!("item t={:.3}: use: Inventory is empty", hook_time(p, b));
            }
        }
        p.charge = 0.0;
        (p.throw_held, p.item_use) = (false, false);
        return;
    };
    // "can use now" (FUN_000abbb0): the shown pair must be the wanted one, else "Not ready" (the
    // shown pair lags while a use is under way: a B tap in a throw, take20; code only, not filmed)
    if b.shown != current {
        if press {
            message(&mut status, g, S_NOT_READY, "Not ready");
            b.lock = UseLock::Blocked;
            if b.log {
                println!("item t={:.3}: use: Not ready (shown {} != wanted {})", hook_time(p, b), item_name_plain(p, b.shown), item_name_plain(p, current));
            }
        }
        p.charge = 0.0;
        (p.throw_held, p.item_use) = (false, false);
        return;
    }
    if held {
        b.lock = UseLock::On(item);
    }
    let grenade = matches!(item, Item::Grenade(_));
    p.throw_held = held && grenade;
    p.item_use = press && !grenade;
    if b.log && press {
        println!("item t={:.3}: use {}{}", hook_time(p, b), item_name_plain(p, current), if forced { " (Black)" } else { "" });
    }
}

/// The inventory front's frame (FUN_0014db60, from the world's update: not under a pause): a
/// "none" selection becomes the first listable stack; the shown pair follows the wanted unless a
/// use is under way (+0x1c8 bit 0: FUN_00121170 sets it as the use starts; here the grenade not
/// yet out of the hand or the medkit's use clip); a shown stack that ran out sets both to none
/// (take09: (10, -1) at 5.219, the Frag wanted at 5.270, shown at 5.572 as the throw ended).
fn settle(mut player: ResMut<Player>, mut boxed: ResMut<ItemBox>) {
    if paused() {
        return;
    }
    let (p, b) = (&mut *player, &mut *boxed);
    if p.loaded.is_none() || p.stand_in {
        return;
    }
    if p.item.is_none() {
        p.item = first(p);
        if b.log && p.item.is_some() {
            println!("item t={:.3}: none -> {} (the first stack)", hook_time(p, b), item_name_plain(p, p.item));
        }
    }
    let in_use = p.throwing.as_ref().is_some_and(|t| !t.released) || p.using.is_some();
    if b.shown != p.item && !in_use {
        b.shown = p.item;
    }
    if let Some(s) = b.shown {
        if item_count(p, s) <= 0 {
            if b.log {
                println!("item t={:.3}: {} ran out: none", hook_time(p, b), item_name_plain(p, b.shown));
            }
            b.shown = None;
            p.item = None;
        }
    }
}

/// The item widget's frame (0xabc60 -> FUN_000ac220, on the HUD's clock): the hold timer, the open
/// level, the scroll while the list is out, the slots' stacks (the box from the wanted pair; the
/// list's around it while it shows; repeats hidden), then each slot's NEW label (0xad0c0).
#[allow(clippy::too_many_arguments)]
fn widget(time: Res<Time>, input: Res<ItemInput>, mut player: ResMut<Player>, mut boxed: ResMut<ItemBox>, game: Res<GameData>,
          kits: Option<Res<GrenadeKits>>) {
    let dt = hud_dt(&time);
    let (p, b, g) = (&mut *player, &mut *boxed, &game.0);
    if b.paused_game {
        b.paused_time += dt;
    }
    if b.hold != 0.0 {
        b.hold -= dt;
        if b.hold < 0.0 {
            close_list(b, p, g);
            b.hold = 0.0;
        }
    }
    if !b.opened {
        if b.open > 0.0 {
            b.open -= CLOSE_RATE * dt;
        }
    } else {
        if b.open <= SCROLL_OPEN || b.hold < SCROLL_FRESH {
            b.ramp = 0.0;
        } else {
            scroll(b, p, g, &input, dt);
        }
        if b.open < 1.0 {
            b.open += OPEN_RATE * dt;
        }
    }
    b.open = b.open.clamp(0.0, 1.0);
    // the box: the wanted pair, hidden at none (FUN_000ab860)
    let wanted = p.item;
    b.slots[0].pair = wanted;
    b.slots[0].hidden = wanted.is_none();
    // the list (FUN_000ac5e0, while it shows at all): up the next two, left the previous two; a slot
    // after a none is hidden
    if b.open > 0.0 {
        for (slots, forward) in [([1, 2], true), ([3, 4], false)] {
            let mut cur = wanted;
            let mut go = true;
            for s in slots {
                if go {
                    cur = if forward { next(p, cur) } else { prev(p, cur) };
                    go = cur.is_some();
                    b.slots[s].hidden = cur.is_none();
                } else {
                    b.slots[s].hidden = true;
                }
                b.slots[s].pair = cur;
            }
        }
        // FUN_000ac7f0: a slot showing an earlier one's stack (box, left 1, up 1, left 2, up 2) is
        // hidden: two stacks show left 1 only, three left 1 and up 1 (take25's slot logs)
        for at in (1..DEDUPE_ORDER.len()).rev() {
            let s = DEDUPE_ORDER[at];
            if b.slots[s].hidden {
                continue;
            }
            if DEDUPE_ORDER[..at].iter().any(|&e| b.slots[e].pair == b.slots[s].pair) {
                b.slots[s].hidden = true;
            }
        }
    }
    // NEW (0xad0c0, each slot): the first slot to show a stack whose flag is set clears it and runs
    // its label; a slot's label stops when its stack changes
    for s in 0..b.slots.len() {
        let stack = b.slots[s].pair.filter(|&i| item_count(p, i) > 0);
        if let Some(i) = stack {
            if b.seen.insert(i) {
                b.slots[s].new_time = NEW_TIME;
                b.slots[s].new_of = Some(i);
                if b.log {
                    println!("item t={:.3}: NEW on {} (slot {s})", hook_time(p, b), item_name_plain(p, Some(i)));
                }
            }
        }
        if b.slots[s].new_of != stack {
            b.slots[s].new_time = 0.0;
            b.slots[s].new_of = stack;
        }
        if b.slots[s].new_time != 0.0 {
            b.slots[s].new_time = (b.slots[s].new_time - dt).max(0.0);
        }
    }
    if b.log {
        log_state(b, p, g, kits.as_deref(), dt);
    }
}

/// The open list's scroll (FUN_000ac370): the stick past the dead zone on its larger axis, right
/// or up the next item, left or down the previous; a step the frame it's pushed, then another
/// each time the ramp passes 1 while it's held; the wheel's notches a step each (the demo's).
fn scroll(b: &mut ItemBox, p: &mut Player, game: &Game, input: &ItemInput, dt: f32) {
    let (x, y) = (input.stick.x, input.stick.y);
    let forward = (x > DEAD_ZONE && x > y.abs()) || (y < -DEAD_ZONE && y < -x.abs());
    let back = (x < -DEAD_ZONE && x < -y.abs()) || (y > DEAD_ZONE && x.abs() < y);
    if forward {
        if b.ramp <= 0.0 {
            step(p, game, true);
            b.ramp = 0.0;
        }
        b.ramp += RAMP_RATE * dt;
        if b.ramp > 1.0 {
            b.ramp = 0.0;
        }
    } else if back {
        if b.ramp >= 0.0 {
            step(p, game, false);
            b.ramp = 0.0;
        }
        b.ramp -= RAMP_RATE * dt;
        if b.ramp < -1.0 {
            b.ramp = 0.0;
        }
    } else {
        b.ramp = 0.0;
    }
    for _ in 0..input.wheel.unsigned_abs() {
        step(p, game, input.wheel > 0);
    }
}

/// BF_ITEM_LOG's state line: on every change, and every 0.1 s of the HUD's clock while the list
/// shows. The selection and the shown pair as (category, index), the open level, the timers, the
/// five slots (box, up 1, up 2, left 1, left 2; `h` hidden) and their NEW labels, the pause, the
/// lock, Black's saved item, the message and the counts.
fn log_state(b: &mut ItemBox, p: &Player, game: &Game, kits: Option<&GrenadeKits>, dt: f32) {
    b.log_clock += dt;
    let none = GrenadeKits(vec![]);
    let pair = |i: Option<Item>| i.map_or("10,-1".to_string(), |i| { let (c, x) = key(game, kits, i); format!("{c},{x}") });
    let name = |i: Option<Item>| i.map_or("-".to_string(), |i| super::inventory::item_label(game, kits.unwrap_or(&none), p, i));
    let slots: Vec<String> = b.slots.iter().map(|s| format!("{}{}", pair(s.pair), if s.hidden { "h" } else { "" })).collect();
    let counts: Vec<String> = listable(p).into_iter()
        .map(|i| format!("{} {}{}", name(Some(i)), item_count(p, i), if b.seen.contains(&i) { "" } else { " (NEW)" })).collect();
    let state = format!("sel {} {} | shown {} | list {}{} | slots {} | lock {:?} | black {} | counts {}",
                        pair(p.item), name(p.item), pair(b.shown), if b.list_mode { "on" } else { "off" }, if b.paused_game { ", paused" } else { "" },
                        slots.join(" "), b.lock, b.saved.map_or("-".to_string(), |s| name(Some(s))), counts.join(", "));
    // (the labels' times change every frame: whether each runs is the change)
    let running: String = b.slots.iter().map(|s| if s.new_time > 0.0 { '1' } else { '0' }).collect();
    let signature = format!("{state} {running}");
    let ticking = b.open > 0.0 && b.log_clock - b.log_at >= 0.1;
    if signature != b.logged || ticking {
        b.logged = signature;
        b.log_at = b.log_clock;
        let labels: Vec<String> = b.slots.iter().map(|s| if s.new_time > 0.0 { format!("{:.2}", s.new_time) } else { "-".into() }).collect();
        println!("item t={:.3} (sim {:.3}, hud {:.3}): o {:.3} ht {:.3} B {:.3} ramp {:.3} | {state} | NEW {}", hook_time(p, b), p.sim_time,
                 b.log_clock, b.open, b.hold, b.b_timer, b.ramp, labels.join(" "));
    }
}

/// One of a slot's drawn parts: its panel, its stack's icon, name, count and NEW label.
#[derive(Component, Clone, Copy, PartialEq)]
enum Part {
    /// the slot's frame (a node the size of the HUD's screen, moved to the slot's place), whose
    /// children are the slot's parts in slot units
    Frame(usize),
    Panel(usize),
    Icon(usize),
    Name(usize),
    Count(usize),
    New(usize),
}

/// The widget's nodes as a map starts (with the HUD, BF_NO_HUD none): for each slot a frame the
/// size of the HUD's 640 x 480 screen moved to the slot's place, with the panel and the icon
/// (64 x 64 at its corner: FUN_000ab7c0), the name, the count and NEW in the 12-point body font
/// (h_0c626fd5, FUN_000cd9c0(0xc626fd5, 12): play_fonts.rs BODY12), drawn in that order
/// (FUN_000ab460's children). The list slots first, so the box draws over them.
fn spawn_widget(mut commands: Commands, mut game: ResMut<GameData>, mut images: ResMut<Assets<Image>>, mut cache: ResMut<HudImages>) {
    if std::env::var("BF_NO_HUD").is_ok() {
        return;
    }
    let panel = texture(&mut game.0, &mut images, &mut cache, PANEL).unwrap_or_default();
    let new_word = game.0.strings.get(&S_NEW).cloned().unwrap_or("NEW".into());
    let window = commands.spawn((HudRoot, Name::new("item box"), Node { position_type: PositionType::Absolute, width: Val::Percent(100.0),
                                                                        height: Val::Percent(100.0), justify_content: JustifyContent::Center,
                                                                        ..default() })).id();
    let screen = commands.spawn((Node { height: Val::Percent(100.0), aspect_ratio: Some(4.0 / 3.0), ..default() }, ChildOf(window))).id();
    let shadow = Some(Color::BLACK);
    let rgb = |c: [u8; 3]| Color::srgb_u8(c[0], c[1], c[2]);
    let line = BODY12.height();
    for s in [4, 3, 2, 1, 0] {
        let frame = commands.spawn((Part::Frame(s), Visibility::Hidden, ChildOf(screen),
                                    Node { position_type: PositionType::Absolute, width: Val::Percent(100.0), height: Val::Percent(100.0),
                                           left: Val::Percent(BOX_AT.x / 6.4), top: Val::Percent(BOX_AT.y / 4.8), ..default() })).id();
        let tint = if s == 0 { Color::WHITE } else { Color::srgba(LIST_PANEL_GREY, LIST_PANEL_GREY, LIST_PANEL_GREY, LIST_PANEL_ALPHA) };
        commands.spawn((Part::Panel(s), ChildOf(frame), super::hud::at(0.0, 0.0, SLOT_SIZE, SLOT_SIZE), flipped(panel.clone()).with_color(tint)));
        commands.spawn((Part::Icon(s), ChildOf(frame), super::hud::at(0.0, 0.0, SLOT_SIZE, SLOT_SIZE), flipped(Handle::default()),
                        Visibility::Hidden));
        commands.spawn((Part::Name(s), ChildOf(frame), Visibility::Hidden,
                        record_text("", SLOT_SIZE / 2.0, SLOT_SIZE - line + 1.0, &BODY12, vec![rgb(BLUE)], shadow, Across::Centre)));
        commands.spawn((Part::Count(s), ChildOf(frame), Visibility::Hidden,
                        record_text("", SLOT_SIZE - COUNT_IN, SLOT_SIZE - line - COUNT_UP, &BODY12, vec![rgb(BLUE)], shadow, Across::Right)));
        commands.spawn((Part::New(s), ChildOf(frame), Visibility::Hidden,
                        record_text(&new_word, NEW_AT.x, NEW_AT.y, &BODY12, vec![rgb(NEW_ORANGE)], shadow, Across::Left)));
    }
}

/// The slot's place on the HUD's screen (FUN_000ac500): the box at BOX_AT; up slot k (0, 1) at
/// -(k + 1) x int(open x 70) units above it, left slot k as far left.
fn slot_at(s: usize, open: f32) -> Vec2 {
    let off = (open * SLOT_STEP).trunc();
    match s {
        1 | 2 => BOX_AT - Vec2::new(0.0, s as f32 * off),
        3 | 4 => BOX_AT - Vec2::new((s - 2) as f32 * off, 0.0),
        _ => BOX_AT,
    }
}

/// The widget drawn: each slot's frame at its place and shown or hidden, its alpha (the box at
/// full; a list slot open^3, FUN_000ac500), its stack's icon (its type's h_e5ec3f1f; grey while
/// the character can't use it now, FUN_000ace10's flag), name (the type's stringtable-name) and
/// count (from 2 up: 0xacbd0's `cmp word [item+0x1a], 1; jle`) in the blue or the red, and NEW
/// (0xad0c0's blink and fade).
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
fn draw(boxed: Res<ItemBox>, player: Res<Player>, mut game: ResMut<GameData>, kits: Option<Res<GrenadeKits>>, mut images: ResMut<Assets<Image>>,
        mut cache: ResMut<HudImages>,
        mut parts: Query<(&Part, &mut Visibility, Option<&mut Node>, Option<&mut ImageNode>, Option<&mut RecordText>)>) {
    let (b, p) = (&*boxed, &*player);
    let kits = kits.as_deref();
    let none = GrenadeKits(vec![]);
    let rgb = |c: [u8; 3]| Color::srgb_u8(c[0], c[1], c[2]);
    // (no box without a character: the editor's stand-in)
    let shown = |s: usize| p.loaded.is_some() && !p.stand_in && b.slots[s].pair.is_some() && !b.slots[s].hidden && (s == 0 || b.open > 0.0);
    let alpha = |s: usize| if s == 0 { 1.0 } else { b.open * b.open * b.open };
    for (part, mut vis, node, img, text) in &mut parts {
        let set_vis = |v: &mut Visibility, on: bool| { let want = if on { Visibility::Inherited } else { Visibility::Hidden }; if *v != want { *v = want; } };
        match *part {
            Part::Frame(s) => {
                set_vis(&mut vis, shown(s));
                let at = slot_at(s, b.open);
                if let Some(mut n) = node {
                    let (l, t) = (Val::Percent(at.x / 6.4), Val::Percent(at.y / 4.8));
                    if n.left != l || n.top != t {
                        (n.left, n.top) = (l, t);
                    }
                }
            }
            Part::Panel(s) => {
                if let Some(mut img) = img {
                    let base = if s == 0 { 1.0 } else { LIST_PANEL_ALPHA };
                    let a = base * alpha(s);
                    if (img.color.alpha() - a).abs() > 1e-4 {
                        img.color.set_alpha(a);
                    }
                }
            }
            Part::Icon(s) => {
                let Some(item) = b.slots[s].pair.filter(|_| shown(s)) else { set_vis(&mut vis, false); continue };
                let id = icon_of(&game.0, kits, p, item);
                let h = if item_usable(p, item) { texture(&mut game.0, &mut images, &mut cache, id) }
                    else { texture_grey(&mut game.0, &mut images, &mut cache, id) };
                set_vis(&mut vis, h.is_some());
                if let (Some(mut img), Some(h)) = (img, h) {
                    if img.image != h {
                        img.image = h;
                    }
                    let a = alpha(s);
                    if (img.color.alpha() - a).abs() > 1e-4 {
                        img.color.set_alpha(a);
                    }
                }
            }
            Part::Name(s) | Part::Count(s) => {
                let Some(item) = b.slots[s].pair.filter(|_| shown(s)) else { set_vis(&mut vis, false); continue };
                let count = item_count(p, item);
                let is_name = matches!(*part, Part::Name(_));
                let words = if is_name { super::inventory::item_label(&game.0, kits.unwrap_or(&none), p, item) } else { count.to_string() };
                set_vis(&mut vis, is_name || count >= 2);
                if let Some(mut t) = text {
                    let colour = rgb(if item_usable(p, item) { BLUE } else { RED });
                    if t.text != words {
                        t.text = words;
                    }
                    if t.colours.first() != Some(&colour) {
                        t.colours = vec![colour];
                        t.redraw();
                    }
                    let a = alpha(s);
                    if (t.alpha - a).abs() > 1e-4 {
                        t.alpha = a;
                    }
                }
            }
            Part::New(s) => {
                let time = b.slots[s].new_time;
                let on = shown(s) && time > 0.0 && (2.0 * time).fract() >= NEW_BLINK;
                set_vis(&mut vis, on);
                if let Some(mut t) = text {
                    let a = (2.0 * time).min(1.0) * alpha(s);
                    if on && (t.alpha - a).abs() > 1e-4 {
                        t.alpha = a;
                    }
                }
            }
        }
    }
}

/// An item's HUD icon (its type's h_e5ec3f1f): a grenade type's (the Frag's without one), the
/// carried Medkit's, another item's (the OrgSen's h_1b73c691, the Hat's h_06a6c5ad).
fn icon_of(game: &Game, kits: Option<&GrenadeKits>, p: &Player, item: Item) -> u32 {
    match item {
        Item::Grenade(k) => kits.and_then(|g| g.0.get(k)).map(|g| g.def.icon).filter(|&x| x != 0).unwrap_or(FRAG_ICON),
        Item::Medkit => game.items.get(&p.medkit_kind).map(|t| t.icon).filter(|&x| x != 0).unwrap_or(MEDKIT_ICON),
        Item::Other(kind) => game.items.get(&kind).map(|t| t.icon).filter(|&x| x != 0).unwrap_or(MEDKIT_ICON),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A player carrying the given grenade counts (kits 0..), Medkits and others (OrgSen-like
    /// stacks, listed).
    fn carrying(grenades: &[i64], medkits: i64, others: &[(u32, i64)]) -> Player {
        let mut p = Player::new(3);
        p.grenades = grenades.to_vec();
        p.medkits = medkits;
        p.others = others.iter().map(|&(kind, count)| super::super::inventory::Carried { kind, count, listed: true, use_type: 1, heals: false }).collect();
        p
    }

    /// take01's order and wrap: next steps through the listable stacks in order and round; a
    /// stack at 0 is skipped (AC-01, AC-02).
    #[test]
    fn next_wraps_and_skips_empty() {
        // Frag 5, Gas 5, Sonic 4, Energy 5, (Light 0), Sentry 3, Roller 4, Medkit 2, OrgSen 1
        let p = carrying(&[5, 5, 4, 5, 0, 3, 4], 2, &[(0x1143_3446, 1)]);
        let mut at = Some(Item::Grenade(2));
        let mut seen = vec![];
        for _ in 0..9 {
            at = next(&p, at);
            seen.push(at.unwrap());
        }
        assert_eq!(seen, vec![Item::Grenade(3), Item::Grenade(5), Item::Grenade(6), Item::Medkit, Item::Other(0x1143_3446),
                              Item::Grenade(0), Item::Grenade(1), Item::Grenade(2), Item::Grenade(3)]);
        assert_eq!(prev(&p, Some(Item::Grenade(0))), Some(Item::Other(0x1143_3446)));
        assert_eq!(prev(&p, Some(Item::Grenade(5))), Some(Item::Grenade(3)));
        assert_eq!(first(&p), Some(Item::Grenade(0)));
    }

    /// One stack: next and previous are itself (a tap changes nothing, no sound); none: none.
    #[test]
    fn one_or_none() {
        let p = carrying(&[0, 0, 4], 0, &[]);
        assert_eq!(next(&p, Some(Item::Grenade(2))), Some(Item::Grenade(2)));
        assert_eq!(prev(&p, Some(Item::Grenade(2))), Some(Item::Grenade(2)));
        let empty = carrying(&[0, 0], 0, &[]);
        assert_eq!(first(&empty), None);
        assert_eq!(next(&empty, None), None);
    }
}
