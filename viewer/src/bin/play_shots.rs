//! The shot path of a gun, from default.xbe's weapon class (0x22b000-0x233000; the data each
//! function reads is `weapon::ShotData`):
//!
//!  - **Fire timing** (FUN_0022f2a0, the fire loop, run once a game frame by FUN_0022ee30 ->
//!    FUN_0022efe0 while the weapon's fire flag +0x224 bit 4 is set):
//!    a shot sets the cooldown (weapon+0x218) to 1 / rate (FUN_0022f0e0; set, not added, so
//!    the rest of the frame is dropped); it counts down by the frame's time (FUN_0022ec90) and
//!    the trigger fires again once it's <= 0. The demo runs this in 30 Hz game frames, so an
//!    interval is the rate's period rounded up to whole frames: MK 8/s -> 4 frames, 7.5/s;
//!    Minigun 15/s -> 2-3 frames (the float sum lands either side of 0, see
//!    `TICK_JITTER`), ~12/s. The takes
//!    measured 7.0-7.4 and 11.4-11.7 (the emulator ran at 27-29 fps). While the holder's
//!    +0x7a8 is set (the scope; medium confidence) the rate is h_019c314a (MK 6/s).
//!  - **Bursts and pellets**: each shot counts weapon+0x28c; below the burst count the next
//!    cooldown is the burst-delay instead (FUN_0022f0e0). With a delay of 0 the loop goes on in
//!    the same frame and only the last takes a round (FUN_0022fe00): the Bower's 6
//!    pellets a shell for one round. Each after the first is turned from the one before, in place,
//!    by uniform random angles within the burst-spread about the world's x axis and then its
//!    vertical (FUN_0022e1d0; `directions`), so the pellets walk away from the first.
//!  - **Accuracy** (the object at weapon+0x1d0: current +0xc = weapon+0x1dc, cap +0x10): the cap
//!    is the data's max, or its scoped cap while the holder's +0x7a8 is set (FUN_001241b0 ->
//!    FUN_00222ac0); each trigger cycle takes recoil-per-shot off (FUN_00222c10; halved, and the
//!    floor min + 15, while the holder crouches, +0x1f8 == 6); each frame the cooldown is out it
//!    gains recovery x dt (FUN_00222b30, doubled crouched). It's used once a frame, before the
//!    fire loop (FUN_0022ee30 -> FUN_0022e4c0): the aim from each muzzle (FUN_0022e5b0) is turned
//!    about the three world axes by uniform random angles within +-S degrees (FUN_002229b0),
//!    S = (100 - clamp(A + bonus, 0, 100)) x 0.03 (FUN_000c4e10). The bonus (accuracy object
//!    +8, written each frame by FUN_00222b30 from the holder) is -1 unless the holder's script
//!    property h_ec5337fb (char+0x498) is set, then its h_072837bb (char+0x49c, -100..100: the
//!    property handler at 0x11302c): the placement's, for a character a spawn-trigger made from
//!    one (#227: the tutorial's Red Hand +25, `Trigger::bonus`), -1 for everyone else.
//!  - **Tracers**: an instant ray (bullet-type 4, FUN_002317e0) spawns its flight effect only
//!    when the counter weapon+0x290 is 0, which then restarts at h_eeb9e75a: the first shot and
//!    every (N+1)th after (MK every 3rd, Minigun every 4th). Other bullet types always fly theirs.
//!  - **Muzzle effect** h_fd88830d at the muzzle hardpoint, set off with each shot
//!    (FUN_0022f170).
//!  - **Shells**: each trigger cycle adds h_fdc93b33 to weapon+0x208 (FUN_0022f2a0); each whole
//!    one throws a casing object (h_19b21bf5, the casing h_16415157) from the h_ea1abbbe
//!    hardpoint (FUN_0022eee0 -> FUN_00231b10): 5 cm back along the hardpoint's z, at 2-3 m/s
//!    along its z turned a quarter turn about the vertical, plus up to 0.5 m/s along each of the
//!    other two axes (turned +-15 degrees about z).
//!  - **Damage falloff**: an instant hit's damage is scaled by FUN_00223780(Damage h_fb124e6c,
//!    1 - distance / range) (FUN_00230040): the Bower's mode 2 is (1 - d/range)^2, 0.60 at 9 m.
//!  - **Two guns** (Tex's berserk, play_skills.rs): each weapon object runs its own fire loop
//!    (FUN_0022ee30), and the fire command sets the fire flag of the gun in hand and of the
//!    second hand's gun alike (CT_FIRE_WEAPON, 0x122614-0x122655). So both fire on the trigger,
//!    each at its own rate from its own clip, with its own cooldown, burst counter and accuracy
//!    (#35 take04: the LZR-50 every 0.29 s and the Jax-iC every 0.52 s, from the same frame).

use super::*;
use bf_viewer::bf::weapon::ShotData;

/// The game frame the fire loop runs in (s): the takes' rates are whole frames at ~30 Hz.
pub const TICK: f32 = 1.0 / 30.0;
/// How much a game frame's time varies, as a share of TICK (+-). Most rates' periods are a
/// whole number of frames (Minigun 1/15 = 2, Foley 1/3 = 10, LZR-50 1/3.75 = 8, Jax-iC 1/2 =
/// 15): there the cooldown lands on 0 and the frame time's own wobble decides whether the shot
/// goes on that frame or the next, about half the time each. The takes measured just that:
/// Minigun 0.086 s (2.5 frames: 0.083), Foley 0.352 (10.5: 0.350), LZR-50 0.284 (8.5: 0.283),
/// Jax-iC 0.52 (15.5: 0.517), while the MK's 3.75 frames is always 4 (0.142 measured, 0.133).
/// This is the demo's model of the frame clock's jitter, not anything in the code: small enough
/// not to move any period that isn't a tie. The takes ran in xemu at 27-29 fps, whose uneven
/// frames may be all there is to it; on hardware at a steady 30 fps the Minigun may well fire at
/// 15/s. Its random numbers are the frame clock's own (`FireState::clock_seed`): drawn from the
/// gun's stream between its shots' draws, a cycle of a fixed number of draws sampled the LCG at a
/// fixed stride, and at a power of two (the Foley's 16: a shot's 3 accuracy draws and 2 effect
/// seeds, then 11 frames) the frames of one cycle ran short or long together every cycle, so its
/// shots came 11 frames apart every time (#54's check: 2.73 a second against the takes' 2.79-2.93)
/// where the jitter splits 10 and 11.
const TICK_JITTER: f32 = 1e-4;
/// Mixed into the gun's seed for the frame clock's own (any odd constant: the demo's).
const CLOCK_SEED: u32 = 0x7C10_C4E5;
/// The game's degrees-to-radians factor in FUN_002229b0 and FUN_0022e1d0 (0x3a52e0).
const DEG: f32 = 0.017444;
/// FUN_000c4e10: degrees of spread per point of accuracy below 100.
const SPREAD_PER_POINT: f32 = 0.03;
/// The accuracy object's +8 bonus without the holder's script property (FUN_00222b30): every
/// holder's but a placement's that sets h_ec5337fb (`Trigger::bonus`).
pub const ACCURACY_BONUS: f32 = -1.0;
/// Crouched (holder +0x1f8 == 6): the floor of the recoil is min + this (FUN_00222c10).
const CROUCH_FLOOR: f32 = 15.0;
/// FUN_00231b10: a casing starts this far (m) back along the hardpoint's z, leaves at
/// SHELL_SPEED + up to SHELL_SPEED_MORE m/s along its z turned a quarter turn about the
/// vertical, with up to SHELL_JITTER m/s along each other axis, the frame turned up to
/// SHELL_ROLL degrees either way about z.
const SHELL_BACK: f32 = 0.05;
const SHELL_SPEED: f32 = 2.0;
const SHELL_SPEED_MORE: f32 = 1.0;
const SHELL_JITTER: f32 = 0.5;
const SHELL_ROLL: f32 = 15.0;
/// In the holder's own scope the muzzle effect is drawn this far (m) from the eye, at
/// SCOPE_FLASH_AT, its axis along the line of sight. The demo's rule: drawn at the muzzle, a few
/// tenths of a metre from the scoped eye and magnified, its sparks covered half the view in
/// streaks. No code was found that moves or hides it for the scope (the scope-in at 0x1223da and
/// FUN_001249b0 only set +0x7a8, the sway and the zoom sounds). The distance sets its size: the
/// game's is 124 x 144 units (see SCOPE_FLASH_AT); at 3 m ours measured 153 x 190, so 3.8 m.
/// Its star look is the ALE rendering's (#104), not the game's soft ball.
const SCOPE_FLASH_REACH: f32 = 3.8;
/// Where the scoped flash shows: its centre this many 640 x 480 units right of and below the
/// crosshair. Measured on 63/take01 at 28.77-28.85 s (the bright pixels that changed): centroid
/// (412-424, 231-249), extent x 344..468, y 163..307 (124 x 144 units), clear of the crosshair
/// (x 303-336): 95-104 right, level. The demo's rule: the demo's hidden gun sits further off the
/// eye than the game's. Set to [100, 0], ours measured centroid (107-110, -15..-21), 143-155 x
/// 97-115 (its emitters sit off the effect's middle), so the aim point is moved to land it on
/// the game's: centroid ~(100, 0).
const SCOPE_FLASH_AT: [f32; 2] = [92.0, 18.0];
/// How long a casing stays (s), how it falls (m/s^2) and bounces. Guesses: the casing is a
/// pooled physics object (FUN_002291d0) whose life and material aren't traced.
const SHELL_LIFE: f32 = 2.0;
const SHELL_GRAVITY: f32 = 9.81;
const SHELL_BOUNCE: f32 = 0.3;

/// The game's random numbers (FUN_00233b90: x * 0x19660d + 0x3c6ef35f, the top 23 bits as a
/// float in [0, 1)). (play_missile.rs draws its own from it too.)
pub(super) fn random(seed: &mut u32) -> f32 {
    *seed = seed.wrapping_mul(0x0019_660D).wrapping_add(0x3C6E_F35F);
    f32::from_bits((*seed >> 9) | 0x3F80_0000) - 1.0
}

/// One weapon object's own fire state (each gun keeps its own in the game).
#[derive(Clone, Default)]
struct WeaponFire {
    /// current accuracy (weapon+0x1dc); None until the gun is first used: it starts at its cap
    accuracy: Option<f32>,
    /// tracer counter (weapon+0x290)
    tracer: i64,
    /// casings owed (weapon+0x208)
    shells: f32,
}

/// The hands a character fires from: 0 the gun in hand (inventory +0x164), 1 the second hand's
/// (inventory +0x198: Tex's berserk, `Player::second`).
pub const HANDS: usize = 2;

/// One shot the fire loop let off this frame, for the shot builder (play.rs).
#[derive(Clone, Copy, Debug)]
pub struct Pellet {
    /// the hand whose gun let it off (see `HANDS`)
    pub hand: usize,
    /// the game frame it went in (the accuracy turn is once a frame)
    tick: u32,
    /// the accuracy spread then (degrees, FUN_000c4e10)
    spread: f32,
    /// a zero-delay burst's later shot: turned by the burst-spread from the one before
    walk: bool,
    /// it carries the bullet's flight effect (tracer interval)
    pub tracer: bool,
}

/// A character's fire loop state (kept on `Player`).
#[derive(Default)]
pub struct FireState {
    /// time not yet run in game frames
    clock: f32,
    tick: u32,
    /// shots into the current burst (weapon+0x28c), by hand
    burst: [i64; HANDS],
    weapons: Vec<WeaponFire>,
    /// the shots let off since the shot builder last took them
    pub pellets: Vec<Pellet>,
    /// muzzle effects and casings owed to the effects (`shot_fx`), by hand
    pub flashes: [u32; HANDS],
    pub shells: [u32; HANDS],
    /// empty trigger cycles since play_ammo.rs last took them (FUN_002327f0), by hand
    empty: [u32; HANDS],
    seed: u32,
    /// the frame clock's random numbers (`TICK_JITTER`), apart from the gun's stream
    clock_seed: u32,
    /// a trigger pull still owed its first shot, by hand (see `pull`), and the trigger last frame
    latched: [bool; HANDS],
    was_held: bool,
    /// seconds the gun stays up after a shot (see `READY_HOLD`)
    ready: f32,
}

/// After a shot the holder stays in its ready pose this long (s): FUN_00123ea0 raises the gun or
/// keeps it raised, holding the character's ready timer +0x7dc at 4 (0x3c05fc) or more through
/// FUN_00123f60 while it runs (and starting it otherwise).
/// Read as the time the gun stays raised (medium confidence): within it a pull fires without the
/// raise (play.rs `raise`: the motion set's in-time, #140), as the takes' taps 2.2 s apart
/// logged 0.07-0.11 s after the input, against 0.19-0.24 s from idle (both input-to-log times,
/// with the rig's delay and the log's lateness: todo/140 answer §2). #35 take17 shows it after Flint's targeting: off at 11.55 s, the rifle lowered at
/// 15.5-15.8 (play_skills.rs `Skill::raised`).
pub(super) const READY_HOLD: f32 = 4.0;

/// The cooldown a trigger cycle, a dry click or a switch sets: 1 / rate, or 1 / the scoped rate
/// h_019c314a while the holder's +0x7a8 is set (FUN_0022f0e0, FUN_002327f0, FUN_0022dc00 all
/// read it through vtable +0x15c).
pub fn period(def: &WeaponDef, scoped: bool) -> f32 {
    let rate = if scoped && def.shots.scoped_rate > 0.0 { def.shots.scoped_rate } else { def.rate };
    1.0 / rate.max(0.1)
}

/// What the trigger and the holder are doing this frame.
pub struct Trigger {
    /// the trigger for the fire loop (`pull`: held, or a tap still owed its shot)
    pub held: bool,
    /// the button down on a gun that's up but whose clip is empty (FUN_002327f0's case), or a pull
    /// still owed the other hand's shot (the request sets both guns' fire flags, `pull`)
    pub dry: bool,
    /// rounds in the clip
    pub clip: i64,
    /// the holder's +0x7a8 (the scope)
    pub scoped: bool,
    /// the holder's +0x1f8 == 6: read as crouched (MT_CROUCH; with 7, MT_CROUCH_WALK: the MT_
    /// table 0x3bfdac). A reading, not a name: the crouch-height reader this cited
    /// (0x14dd50 / 0x1098f0) is neither (FUN_0014dd50 applies a placement's inventory,
    /// code-map.md, #281)
    pub crouched: bool,
    /// the accuracy object's bonus from the holder (FUN_00222b30: its h_072837bb when its
    /// h_ec5337fb is set, else ACCURACY_BONUS)
    pub bonus: f32,
}

/// The spread (degrees either way, on each axis) at accuracy `a` with the holder's bonus
/// (FUN_000c4e10).
pub fn spread_at(a: f32, bonus: f32) -> f32 {
    (100.0 - (a + bonus).clamp(0.0, 100.0)) * SPREAD_PER_POINT
}

/// One gun for the fire loop this frame (`FireState::run`): the game's weapon object, which
/// keeps its own cooldown (+0x218).
pub struct Gun<'a> {
    /// the hand it's in (see `HANDS`)
    pub hand: usize,
    /// its index in the character's weapons, and its data
    pub weapon: usize,
    pub def: &'a WeaponDef,
    pub trigger: Trigger,
    /// its cooldown (weapon+0x218), run down and set by the loop: the gun in hand's is
    /// `Player::cooldown`, the second hand's `SecondGun::cooldown`
    pub cooldown: f32,
    /// the rounds the loop took from its clip this frame (out)
    pub rounds: i64,
}

impl FireState {
    fn weapon(&mut self, i: usize) -> &mut WeaponFire {
        if self.weapons.len() <= i {
            self.weapons.resize(i + 1, WeaponFire::default());
        }
        &mut self.weapons[i]
    }

    /// A reload starting (FUN_0022daf0), a dry click (FUN_002327f0) or a switch (FUN_0022dc00):
    /// each clears the burst counter weapon+0x28c of the gun in `hand`, so its next pull starts a
    /// fresh cycle.
    pub fn reset_burst(&mut self, hand: usize) {
        self.burst[hand] = 0;
    }

    /// The trigger for each hand's fire loop this frame (`can_fire`: whether that hand's gun can
    /// fire): held, or pressed earlier and still owed its first shot. The game's request outlives
    /// the button: the press sets the holder's +0x260 bit 3 (8), which sets the fire flag (+0x224
    /// bit 2) of the gun in hand and of the second hand's each frame (0x122614-0x122655) until an
    /// animation event clears it (0x0f8408d5, FUN_00117660 at 0x117b12, which clears both guns'
    /// +0x224 bit 1 too), so an 80 ms tap fires once the gun is raised (the takes logged it 0.19-0.24
    /// s after the input; the game's raise is 0.10-0.13 s, play.rs `raise`), both guns in Tex's
    /// berserk. The clip that clears it isn't placed; the demo keeps each hand's request until
    /// that gun's first shot, or until it can't fire.
    pub fn pull(&mut self, held: bool, can_fire: [bool; HANDS]) -> [bool; HANDS] {
        let pressed = held && !self.was_held;
        self.was_held = held;
        std::array::from_fn(|k| {
            self.latched[k] = (self.latched[k] || pressed) && can_fire[k];
            (held || self.latched[k]) && can_fire[k]
        })
    }

    /// Whether the fire loop of the gun in `hand` ran an empty cycle since the last call
    /// (play_ammo.rs: the reload or the dry click it leads to, FUN_002327f0 -> FUN_00120d40).
    pub fn take_empty(&mut self, hand: usize) -> bool {
        std::mem::take(&mut self.empty[hand]) > 0
    }

    /// Forget a pull still owed its shot (control passing to another character).
    pub fn drop_pull(&mut self) {
        self.latched = [false; HANDS];
        self.was_held = false;
    }

    /// Whether the gun is still up from a recent shot (`READY_HOLD`): no raise before the next.
    pub fn raised(&self) -> bool {
        self.ready > 0.0
    }

    /// The gun raised with the ready timer out (play.rs `raise`): FUN_00123f60 starts it at 4 +
    /// the in-time (FUN_0010fce0), so it counts as up from then, a shot or not, and a target taken
    /// up again within it raises nothing more.
    pub fn raise(&mut self, in_time: f32) {
        self.ready = self.ready.max(READY_HOLD + in_time);
    }

    /// FUN_00123ea0 asked for besides a shot (Flint's targeting, every frame while it's on,
    /// play_skills.rs; the right mouse's aim, play.rs, the demo's stand-in for it): the gun raised,
    /// its ready timer held at READY_HOLD or more.
    pub fn keep_raised(&mut self) {
        self.ready = self.ready.max(READY_HOLD);
    }

    /// The ready timer out at once (char+0x7dc to 0): what FUN_0011e000 does every frame of a
    /// motion that can't aim (aim="false", the motion's +0x250 bit 6: the crouch's transitions,
    /// the dash, a jump, a dodge, a dive, a slide, a throw) and with an overlay that can't (the
    /// reload's, inferred from #35 take29), FUN_00121670 at a weapon's draw and FUN_00115d50 at
    /// death (todo/126 answers.md Q7, Q11; play.rs `can_aim`). The gun counts as down, so the
    /// follow camera's aim shift runs out (play_camera.rs) and the next pull raises it again.
    pub fn lower(&mut self) {
        self.ready = 0.0;
    }

    /// The ready timer's time left (s; BF_CAMERA_LOG's t7dc).
    pub fn ready_left(&self) -> f32 {
        self.ready.max(0.0)
    }

    /// The current accuracy of weapon `i` (for logs and tests).
    pub fn accuracy(&self, i: usize) -> Option<f32> {
        self.weapons.get(i).and_then(|w| w.accuracy)
    }

    /// Runs the guns' fire loops over `dt` in whole game frames, the gun in hand first: each
    /// one's cooldown (weapon+0x218, kept by play.rs), bursts, accuracy, tracers and casings.
    /// Each gun's `rounds` gets the rounds taken from its clip; the shots themselves wait in
    /// `pellets`.
    pub fn run(&mut self, guns: &mut [Gun], seed: u32, dt: f32) {
        if self.seed == 0 {
            self.seed = seed | 1;
        }
        if self.clock_seed == 0 {
            self.clock_seed = (seed ^ CLOCK_SEED) | 1;
        }
        self.ready -= dt;
        self.clock += dt;
        while self.clock >= TICK {
            self.clock -= TICK;
            self.tick = self.tick.wrapping_add(1);
            // FUN_0022ec90 (with the frame time's wobble, TICK_JITTER: one frame time for both)
            let frame = TICK * (1.0 + (random(&mut self.clock_seed) * 2.0 - 1.0) * TICK_JITTER);
            for g in guns.iter_mut() {
                self.tick_gun(g, frame);
            }
        }
    }

    /// One game frame of one gun's fire loop (see `run`).
    fn tick_gun(&mut self, g: &mut Gun, frame: f32) {
        let (t, weapon, hand) = (&g.trigger, g.weapon, g.hand);
        let d: &ShotData = &g.def.shots;
        let [min, max, scoped_cap, recoil, recovery] = d.accuracy;
        let cap = if t.scoped { scoped_cap } else { max };
        let cycle = period(g.def, t.scoped);
        let cooldown = &mut g.cooldown;
        if *cooldown > 0.0 {
            *cooldown -= frame;
        }
        // FUN_00222ac0: held within [min, cap]
        let mut a = self.weapon(weapon).accuracy.unwrap_or(cap).max(min).min(cap);
        // FUN_0022e4c0: the aim's turn this frame, from the accuracy before the shots
        let spread = spread_at(a, t.bonus);
        // FUN_0022f2a0: an empty clip ends the loop (FUN_002327f0, play_ammo.rs). The clip
        // drops only after a shot for which FUN_0022fe00 returns 0: in a zero-delay burst
        // that's the last shot, once FUN_0022f0e0 has cleared the counter, so the clip stays
        // put through the burst and every shell is its full pellets, the last one included
        while t.held && *cooldown <= 0.0 && t.clip - g.rounds > 0 {
            // turned from the shot before: a zero-delay burst's shots after the first
            let walk = self.burst[hand] > 0 && d.burst_delay == 0.0;
            // FUN_0022f0e0 (the clip it tests hasn't dropped yet this cycle)
            self.burst[hand] += 1;
            if self.burst[hand] < d.burst_count && t.clip - g.rounds > 0 {
                *cooldown = d.burst_delay;
            } else {
                *cooldown = cycle;
                self.burst[hand] = 0;
            }
            // FUN_0022fe00: no round for a zero-delay burst's shots before its last
            if self.burst[hand] == 0 || d.burst_delay != 0.0 {
                g.rounds += 1;
            }
            self.latched[hand] = false;
            self.ready = READY_HOLD;
            // FUN_002317e0: instant rays carry the flight effect every (N+1)th shot
            let tracer = if d.bullet_type == 4 {
                let w = self.weapon(weapon);
                if w.tracer == 0 {
                    w.tracer = d.tracer_gap;
                    true
                } else {
                    w.tracer -= 1;
                    false
                }
            } else {
                true
            };
            self.pellets.push(Pellet { hand, tick: self.tick, spread, walk, tracer });
            self.flashes[hand] += 1;
            // the end of a trigger cycle: casings, recoil (FUN_0022f2a0 -> FUN_00222c10)
            if self.burst[hand] == 0 || d.burst_delay != 0.0 {
                let (lost, floor) = if t.crouched { ((recoil as i64 / 2) as f32, (min + CROUCH_FLOOR).min(cap)) } else { (recoil, min) };
                a = (a - lost).max(floor).min(cap);
                if d.shell_object != 0 && d.shell_hardpoint != 0 {
                    self.weapon(weapon).shells += d.shell_rate;
                }
            }
            if *cooldown > 0.0 {
                break;
            }
        }
        // FUN_002327f0: the trigger down on an empty clip once the cooldown is out sets the
        // cooldown as a shot does (1 / rate, scoped rate in the scope) and clears the burst;
        // the character then reloads or clicks (play_ammo.rs). In game frames, so dry clicks
        // keep the fire loop's rhythm
        if (t.dry || t.held) && *cooldown <= 0.0 && t.clip - g.rounds <= 0 {
            *cooldown = cycle;
            self.burst[hand] = 0;
            self.empty[hand] += 1;
        }
        // FUN_0022eee0: a casing for each whole one owed
        let w = self.weapon(weapon);
        let owed = w.shells.floor().max(0.0);
        w.shells -= owed;
        self.shells[hand] += owed as u32;
        // FUN_00222b30: back toward the cap while the cooldown is out
        if *cooldown <= 0.0 {
            a += recovery * frame * if t.crouched { 2.0 } else { 1.0 };
        }
        self.weapon(weapon).accuracy = Some(a.max(min).min(cap));
    }

    /// The directions of the shots of the gun in `hand` waiting, from its aim `base`: each game
    /// frame's turned once by the accuracy (FUN_002229b0, each weapon object's own turn,
    /// FUN_0022e4c0), a zero-delay burst's later shots each turned again from the one before by
    /// the burst-spread (FUN_0022e1d0, the pellet walk). The walk is in the world's frame: the
    /// aim d becomes Ry(a2) Rx(a1) d, a1 then a2 drawn uniform in +-the burst-spread about the
    /// world's x and then its vertical axis (the routine builds Rx(a1) with FUN_0013e800, turns it
    /// by Ry(a2) with FUN_0001fa20, multiplies the aim's frame by it, FUN_002334b0 as B x A, and
    /// takes the frame's -z, FUN_0001f9c0). So a heading along x keeps its pellets nearly level
    /// (the x turn spins the aim about itself) and one along z scatters them up and down as much as
    /// across: the Bower's take09 at -170 degrees, 10.3 degrees across and 0.8 up and down (the
    /// rule's 9.3 and 1.7), where a turn about the gun's own right gave 9.3 up and down at any
    /// heading (todo/49-bower-20/spec.md §4). The MNR-7's darts walk by the same rule. The other
    /// hand's shots stay waiting.
    pub fn directions(&mut self, hand: usize, base: Vec3, burst_spread: f32) -> Vec<(Pellet, Vec3)> {
        let (pellets, rest): (Vec<Pellet>, Vec<Pellet>) = std::mem::take(&mut self.pellets).into_iter().partition(|p| p.hand == hand);
        self.pellets = rest;
        let mut out = Vec::with_capacity(pellets.len());
        let (mut tick, mut dir) = (None, base);
        for p in pellets {
            if tick != Some(p.tick) {
                tick = Some(p.tick);
                let s = p.spread * DEG;
                let turn = |seed: &mut u32| (random(seed) * 2.0 - 1.0) * s;
                let (x, y, z) = (turn(&mut self.seed), turn(&mut self.seed), turn(&mut self.seed));
                dir = (Quat::from_rotation_z(z) * Quat::from_rotation_y(y) * Quat::from_rotation_x(x) * base).normalize();
            }
            if p.walk && burst_spread > 0.0 {
                // FUN_0022e1d0: a1 (the first draw) about the world's x, then a2 about its
                // vertical, applied to the world direction
                let s = burst_spread * DEG;
                let a1 = (random(&mut self.seed) * 2.0 - 1.0) * s;
                let a2 = (random(&mut self.seed) * 2.0 - 1.0) * s;
                dir = (Quat::from_rotation_y(a2) * Quat::from_rotation_x(a1) * dir).normalize();
            }
            out.push((p, dir));
        }
        out
    }
}

/// FUN_00223780: the damage factor at `x` = 1 - distance / range, by the Damage's falloff mode
/// (h_fb124e6c).
pub fn falloff(mode: i64, x: f32) -> f32 {
    let x = x.clamp(0.0, 1.0);
    match mode {
        1 => x,
        2 => x * x,
        3 if x < 0.5 => 2.0 * x,
        _ => 1.0,
    }
}

impl Shot {
    /// The share of the weapon's damage a hit `t` m along this shot deals (`falloff`, as
    /// FUN_00230040 applies it to an instant hit with the bullet's range).
    pub fn falloff_at(&self, t: f32) -> f32 {
        if self.range <= 0.0 { 1.0 } else { falloff(self.falloff, 1.0 - t / self.range) }
    }
}

/// Where a ray meets the world, and the surface's normal facing back along it (for the hole it
/// leaves): the map's collision, or on the flat test floor the ground, a pillar's side, a piece of
/// scenery the test tools put there (#185, its triangle), or (a placed object's box, `ray_hit`)
/// facing the shot.
pub fn hit_normal(origin: Vec3, dir: Vec3, max: f32) -> Option<(f32, Vec3)> {
    let arena = world::arena();
    if let Some(a) = arena.as_ref().filter(|a| !a.is_flat()) {
        return a.ray_normal(origin, dir, max);
    }
    let t = ray_hit(origin, dir, max)?;
    // (the flat floor's arena: its ground or a piece, when that's what the ray met first)
    if let Some(hit) = arena.and_then(|a| a.ray_normal(origin, dir, max)).filter(|(ta, _)| (ta - t).abs() < 1e-4) {
        return Some(hit);
    }
    let at = origin + dir * t;
    if (at.y - GROUND).abs() < 0.01 {
        return Some((t, Vec3::Y));
    }
    for c in pillars() {
        let d = (at - c) / Vec3::new(0.3, 1.5, 0.3);
        if d.abs().max_element() < 1.01 {
            let k = if d.x.abs() >= d.y.abs().max(d.z.abs()) { 0 } else if d.y.abs() >= d.z.abs() { 1 } else { 2 };
            let mut n = Vec3::ZERO;
            n[k] = d[k].signum();
            return Some((t, n));
        }
    }
    Some((t, -dir))
}

/// A casing thrown from a gun: it falls and bounces off the floor, then goes.
#[derive(Component)]
struct Shell {
    velocity: Vec3,
    spin: Vec3,
    life: f32,
}

/// The casing objects' meshes, by object type (loaded the first time one is thrown); None if it
/// has no model.
#[derive(Resource, Default)]
struct ShellModels(HashMap<u32, Option<Vec<(Handle<Mesh>, Handle<StandardMaterial>, Transform)>>>);

pub fn plugin(app: &mut App) {
    app.init_resource::<ShellModels>()
        .add_systems(Update, test_weapon.before(spawn_player).run_if(in_state(AppState::Playing)))
        .add_systems(Update, (shot_fx, move_shells).chain().after(update_weapons).before(projectiles)
            .run_if(in_state(AppState::Playing)));
}

/// The muzzle effect and the casings for the shots each character let off this frame.
#[allow(clippy::too_many_arguments)]
fn shot_fx(mut commands: Commands, mut player: ResMut<Player>, mut squad: ResMut<Squad>, mut game: ResMut<GameData>,
           ale: Option<ResMut<bf_viewer::ale_fx::AleAssets>>, mut models: ResMut<ShellModels>, globals: Query<&GlobalTransform>,
           mut meshes: ResMut<Assets<Mesh>>, mut materials: ResMut<Assets<StandardMaterial>>, mut images: ResMut<Assets<Image>>,
           mut bindposes: ResMut<Assets<bevy::render::mesh::skinning::SkinnedMeshInverseBindposes>>) {
    let Some(mut ale) = ale else { return };
    for (k, p) in std::iter::once(&mut *player).chain(squad.0.iter_mut()).enumerate() {
        // each hand's gun: the one in hand, and the second hand's (Tex's berserk)
        for hand in 0..HANDS {
            let (flashes, shells) = (std::mem::take(&mut p.fire_state.flashes[hand]), std::mem::take(&mut p.fire_state.shells[hand]));
            if flashes == 0 && shells == 0 {
                continue;
            }
            // the controlled character's eye, when the view is its scope (see SCOPE_FLASH_REACH)
            let eye = (k == 0 && hand == 0 && p.scope > 0.5).then(|| {
                let (eye, turn) = camera::view(p);
                (eye, turn * Vec3::NEG_Z, (fov_at(p.zoom) * 0.5).tan())
            });
            let gun = if hand == 0 { Some(p.weapon) } else { p.second.map(|s| s.weapon) };
            let Some(w) = gun.and_then(|g| p.loaded.as_ref().and_then(|l| l.weapons.get(g))) else { continue };
            let d = &w.def.shots;
            // the muzzle effect, once a frame however many shots (the Bower's pellets are one), in
            // the muzzle hardpoint's own frame: FUN_00232060 attaches the effect object at the
            // hardpoint (vtable +0xdc). Every weapon's muzzle hardpoint has the barrel along its
            // -z (unturned, or half a turn about y on the models built facing +z) and the flash
            // emitters are turned -90 degrees about x to fire along it (sminigun_muzzle.emt: the
            // MK's). One exception: the Psionic Blast's (ORIENTATION w x y z 0 0 0.707 0.707, half
            // a turn about (0, 1, 1)) puts its -z down the model's -y; its effect
            // (seer_fball_mzl_purple) isn't checked against a take. The Therm Sweeper's back-blast
            // dust (mtrail-dust:
            // jetdust_Cube.emt#1.emt, 4.3 m along +z and 2 m down) lands behind the launcher, where
            // turned +y along the barrel it hung 4.3 m above the holder
            if flashes > 0 && d.muzzle_effect != 0 {
                let at = Transform::from_translation(w.muzzle.point).with_rotation(w.muzzle.rot);
                // in the holder's own scope: SCOPE_FLASH_REACH from the eye toward the muzzle,
                // in the world (the demo's rule)
                let scoped = eye.map(|(eye, fwd, half)| {
                    // where the take shows it in the scope: SCOPE_FLASH_AT units right of and below
                    // the crosshair (on the 640 x 480 picture, at whatever the zoom), the gun's side
                    let right = fwd.cross(Vec3::Y).normalize_or(Vec3::X);
                    let up = right.cross(fwd);
                    let [x, y] = SCOPE_FLASH_AT.map(|u| u / 240.0 * half);
                    let toward = (fwd + right * x - up * y).normalize();
                    // its axis along the line of sight, as the game's barrel nearly is from the scope:
                    // the effect's flat flash quads, which lie along the barrel, then stay edge-on
                    // (along the real barrel, seen from above, they were bright ellipses under it).
                    // This turn (+y along the sight) is the one the unscoped flash had before the
                    // hardpoint frame; kept on purpose: the scoped flash was fitted to the take with
                    // it, and it draws as before pixel for pixel (#73's check against main)
                    Transform::from_translation(eye + toward * SCOPE_FLASH_REACH).with_rotation(Quat::from_rotation_arc(Vec3::Y, toward))
                });
                // its ALE effects, with the game's facing 3 (ale_fx's FACING: the sphere muzzles'
                // glows face the camera about their motion, where lying flat across the hardpoint's
                // y they were edge-on); in the scope, with the demo's perp rules, the fit to the take
                // that's kept on purpose (see SCOPE_FLASH_AT). And the effect type's light-effect, a
                // point light at the muzzle as ale_fx plays the blasts' lights; each kind of gun
                // names its own (light_muz_minigun h_08b5bbab on the ballistic guns, 0.2 s growing
                // 0.28 -> 2.66 m; light_muz_laser h_1a96e11e on the LZRs; light_muzzle h_e7e8d51b on
                // the Therm Sweeper and the Sweeper V; _bio, _cutter, _sonic, _particle, _rail,
                // _seer, _energy on the others; none on the iKhan-GPL). Not in the holder's own
                // scope, the demo's choice: from the gun under the eye it washed the whole scope view
                // orange for a frame (the scoped MK pitched down: +29 red on average), and no take
                // shows what the game's light does there, so the scope keeps the demo's old rule of
                // no light
                let kind = game.0.effect_type_defs.get(&d.muzzle_effect).cloned().unwrap_or_default();
                // (a missile launcher's, the Therm Sweeper's and the Sweeper V's launch dust, the
                // same h_002779c8: with its trail's and its explosion's rules, so the game's count
                // makes it one card, see ale_fx's `Compiled::counted`)
                let missile = super::missile::flies(&w.def);
                let fxs: Vec<_> = kind.effects.iter().filter_map(|&e| match scoped {
                    Some(_) => ale.load(&mut game.0, &mut images, &mut materials, e),
                    None if missile => ale.load_recorded_facing(&mut game.0, &mut images, &mut materials, e),
                    None => ale.load_facing(&mut game.0, &mut images, &mut materials, e),
                }).collect();
                let light = (kind.light != 0 && scoped.is_none()).then(|| ale.load(&mut game.0, &mut images, &mut materials, kind.light)).flatten();
                let mut drawn = 0;
                for (fx, is_light) in fxs.into_iter().map(|f| (f, false)).chain(light.map(|l| (l, true))) {
                    let life = fx.duration();
                    // (the light's seed is made from the last effect's: the gun's random stream,
                    // which the shots' spread and frame jitter share, runs as it did before the
                    // light was played)
                    let seed = if is_light { drawn ^ LIGHT_SEED } else { p.fire_state.next_seed() };
                    drawn = seed;
                    // (in the scope, without its flat flash quads: seen from the eye they lay as
                    // bright ellipses or lines under the star, which the take doesn't show; the
                    // demo's rule)
                    let mut run = bf_viewer::ale_fx::AleEffect::once(fx, 0.0, seed).with_lod_sphere(kind.bounds);
                    run.hide_flat = scoped.is_some();
                    let effect = (Visibility::default(), run, AleExpire(life));
                    match scoped {
                        Some(place) => commands.spawn((place, effect)),
                        None => commands.spawn((at, effect, ChildOf(w.entity))),
                    };
                }
            }
            let (Some(port), Ok(g)) = (w.shell, globals.get(w.entity)) else { continue };
            if shells == 0 {
                continue;
            }
            let parts = models.0.entry(d.shell_object).or_insert_with(|| {
                let arch = *game.0.object_meshes.get(&d.shell_object)?;
                let model = WeaponModel::load(&game.0, arch).ok()?;
                let mut assets = ModelAssets { meshes: &mut meshes, materials: &mut materials, images: &mut images, bindposes: &mut bindposes };
                Some(model.parts.iter().flat_map(|part| {
                    let place = Transform::from_translation(part.offset).with_rotation(part.rotation);
                    bf_viewer::scene::static_meshes(&mut game.0, &part.geosets, &mut assets, true).into_iter().map(move |(m, mat)| (m, mat, place))
                }).collect())
            }).clone();
            let Some(parts) = parts else { continue };
            let frame = g.compute_transform();
            let rot = frame.rotation * port.rot;
            let at = frame.transform_point(port.point);
            for _ in 0..shells {
                let s = &mut p.fire_state;
                let quarter = Quat::from_rotation_y(std::f32::consts::FRAC_PI_2);
                let roll = (random(&mut s.seed) * 2.0 * SHELL_ROLL - SHELL_ROLL).to_radians();
                let turned = quarter * rot * Quat::from_rotation_z(roll);
                let speed = SHELL_SPEED + random(&mut s.seed) * SHELL_SPEED_MORE;
                let velocity = -(turned * Vec3::Z) * speed + turned * Vec3::Y * (random(&mut s.seed) * SHELL_JITTER)
                    + turned * Vec3::X * (random(&mut s.seed) * SHELL_JITTER);
                let spin = Vec3::new(random(&mut s.seed) - 0.5, random(&mut s.seed) - 0.5, random(&mut s.seed) - 0.5) * 30.0;
                let root = commands.spawn((Transform::from_translation(at - rot * Vec3::Z * SHELL_BACK).with_rotation(rot),
                                           Visibility::default(), Shell { velocity, spin, life: SHELL_LIFE })).id();
                for (mesh, material, place) in &parts {
                    commands.spawn((Mesh3d(mesh.clone()), MeshMaterial3d(material.clone()), *place, NotShadowCaster, ChildOf(root)));
                }
            }
        }
    }
}

impl FireState {
    /// A seed for an effect's own random numbers.
    fn next_seed(&mut self) -> u32 {
        random(&mut self.seed);
        self.seed
    }
}

/// BF_TEST_WEAPON=<label or hex name> (e.g. "Bower 20", "RVG50 Minigun", 0c3db625): the
/// controlled character starts with that weapon in the slot BF_START_WEAPON picks (the first by
/// default), if the map's data (or the test map's, `-- --test`) defines it. For trying a gun
/// the character doesn't carry. A list, `<a>;<b>`, fills the slots in order from the first
/// instead (#35's Tex: `Confed LZR-50;Jax-iC`, the takes' loadout). On a level whose
/// spawn-triggers start the squad (e34, e40, the tutorial: #220) the character carries its
/// spawn's template weapons (`SquadSpawn::dress`, set before this runs), so the slot is set in
/// those too: without it the hook did nothing there.
fn test_weapon(mut done: Local<bool>, mut game: ResMut<GameData>, mut player: ResMut<Player>) {
    if std::mem::replace(&mut *done, true) {
        return;
    }
    let Ok(want) = std::env::var("BF_TEST_WEAPON") else { return };
    let names: Vec<&str> = want.split(';').map(str::trim).filter(|n| !n.is_empty()).collect();
    let first: usize = if names.len() > 1 { 0 } else { std::env::var("BF_START_WEAPON").ok().and_then(|v| v.parse().ok()).unwrap_or(0) };
    for (k, name) in names.into_iter().enumerate() {
        let hex = u32::from_str_radix(name.trim_start_matches("h_").trim_start_matches("0x"), 16).ok();
        let Some(key) = game.0.weapons.iter().find(|(k, d)| Some(**k) == hex || d.label.eq_ignore_ascii_case(name)).map(|(k, _)| *k) else {
            warn!("BF_TEST_WEAPON: no weapon {name}");
            continue;
        };
        let slot = first + k;
        let list = game.0.character_weapons.entry(CHARACTERS[player.character].to_string()).or_default();
        if slot < list.len() { list[slot] = key } else { list.push(key) }
        if let Some(list) = player.enemy.weapons.as_mut() {
            if slot < list.len() { list[slot] = key } else { list.push(key) }
        }
    }
}

/// Mixed into the last muzzle effect's seed for its light effect's (any odd constant: the
/// demo's), so the light draws no number of the gun's own random stream.
const LIGHT_SEED: u32 = 0x4C16_4855;

/// Casings fall, bounce off the floor losing most of their speed, and go after their life.
fn move_shells(mut commands: Commands, time: Res<Time>, mut shells: Query<(Entity, &mut Shell, &mut Transform)>) {
    let dt = frame_dt(&time);
    for (e, mut s, mut tr) in &mut shells {
        s.life -= dt;
        if s.life <= 0.0 {
            commands.entity(e).despawn();
            continue;
        }
        s.velocity.y -= SHELL_GRAVITY * dt;
        let next = tr.translation + s.velocity * dt;
        let floor = floor_y(next.x, next.z, tr.translation.y + 0.3) + 0.01;
        if next.y < floor && s.velocity.y < 0.0 {
            s.velocity = Vec3::new(s.velocity.x, -s.velocity.y, s.velocity.z) * SHELL_BOUNCE;
            s.spin *= SHELL_BOUNCE;
            tr.translation = Vec3::new(next.x, floor, next.z);
        } else {
            tr.translation = next;
        }
        let spin = s.spin * dt;
        tr.rotate(Quat::from_euler(EulerRot::XYZ, spin.x, spin.y, spin.z));
    }
}
