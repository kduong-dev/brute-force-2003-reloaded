//! Hit and death effects, from the game's data:
//!
//!  - blood: a bullet hitting a squadmate hits world material 21 (WMAT_FLESH_HUMAN). Its
//!    world-materials entry gives, per ammo type, a hit effect (ballistic: wmat_fleshit_standard,
//!    a burst of sparks) and four hit sounds, and one debris effect for every hit: bundle
//!    h_f304f9fb = blood_puff (blood_mist: smokecard.tga puffs), bloodsplat_s (diffuse_big.tga
//!    droplets in a jet, under a weak gravity field) and giblet (a blood.tga spatter). The numbers
//!    below are those nodes' parameters in common/effects-common.ale (`ale_tool.py node`).
//!  - decals: each character names two ground decals: one per hit (h_14410af1: 0.5 x 0.25 m,
//!    RGBA 128,34,34,220) and one under the body (h_16d327fd: 1 x 1 m, 148,44,44,220): white
//!    splat textures tinted by that colour (objecttypes `<decal>`, see `DecalDef`).
//!  - Shrike flesh (23, #129): its own green debris bundle (blood_puff#1, bloodsplat_s#1,
//!    giblet#1) and the game's decal rule (FUN_00124a70: `game_decals`), the first list at the
//!    body and the second along the shot, multiplied into what's under them. The other fleshes
//!    keep the demo's reading (a splat past each hit, splats and a pool at death) until #150.
//!  - a liquid's hits (#152, play_liquids.rs) on any flesh: the game's decal rule alone (the
//!    second list straight up, the record's +0x24), no blood, no hit effect or sound: the liquid
//!    deals no weapon's impact. A death by one leaves out the demo's death spray and splats.
//!  - the memory chip a dead squadmate drops (what this module used to draw as "the DNA", with
//!    the DNA canister's powerup_pill effect) is a mesh pickup now: play_dna.rs.
//!
//! The game draws an ALE appearance's particles as quads with the whole texture on them, not as
//! point sprites (medium confidence: read from FUN_002a1a50, which indexes a table at
//! DAT_00443658 + n * 0x48); so do these.
//!
//! The parameters read as Freelancer's ALE ones (Brute Force hashes their names): an emitter
//! emits `rate` particles/s over its own time, each living `life` s, leaving at `speed` m/s at an
//! angle within `spread` degrees of the emitter's axis (the hit's normal); appearance keys run
//! over a particle's life (0-1). Some curves have items for several values of the effect's user
//! parameter ("sparam": e.g. 26 droplets at the low end, 74 at the top); what the game passes for
//! a hit isn't known, so each hit picks one at random. A particle's size is its full width.

use super::*;
use bf_viewer::bf::audio::Surface;
use bf_viewer::bf::character::{Address, DecalDef};
use bf_viewer::bf::texture::Mips;

/// keys (time, value)
type Keys = &'static [(f32, f32)];
/// keys per sparam value
type Family = &'static [(f32, Keys)];

const LINEAR: u8 = 1;
const EASE_IN: u8 = 2;
const EASE_IN_OUT: u8 = 4;

/// The world material most characters are made of (WMAT_FLESH_HUMAN): the squad's but Flint's,
/// the humans' and the ferals' (their parts' collision, `Game::flesh`; `Loaded::flesh`).
pub(super) const FLESH: i64 = 21;
/// Flint's world material, read from her parts' collision (#128): WMAT_FLESH_SYNTHETIC (24 in the
/// XBE's WMAT_ name table, counted from WMAT_DEFAULT = 0), whose debris (h_1f7ca034: blood_puff_b,
/// bloodsplat_b, giblet_b) and decals (h_02d017b7, h_0d7a5245: RGBA 50,50,50) are grey. Kingman's
/// is too. Shrike flesh (23) has its own bundle (`SHRIKE`); every other flesh (22 MUTANT,
/// 25 SEER, 26 FIREHOUND) still gets FLESH_HUMAN's hit effects and bundle here: their own are the
/// species tickets' (#130-#133).
const SYNTHETIC: i64 = 24;
/// Shrike flesh (WMAT_FLESH_SHRIKE, 23: the Shrike Soldier, Heavy and Hound, the Hunter Lord, and
/// as stored the Supermutant; `Game::flesh`). Its world-materials entry is FLESH_HUMAN's (the same
/// hit effects and sounds per ammo type) but for the debris bundle h_14559f7c: blood_puff#1,
/// bloodsplat_s#1 and giblet#1, green (#129; spec §6.1). Its blood decals follow the game's rule
/// (`game_decals`): the product owner's scope for #129 is the Shrikes, the rest is #150's.
const SHRIKE: i64 = 23;
/// The game's blood decals (`game_decals`, FUN_00124af0 / FUN_00124f60): the first list's decal
/// goes down when the hit's amount is at least this share of the max hit points (0x3a4f94) or at
/// least the hit points left; the second list's while it lives at this share (0.1); the second's
/// ray along the shot reaches this far (m, 0x40800000); the first's spot moves up to this far (m)
/// at random from the middle of the body and the hit point (the takes' logged offsets 0.2-0.25 m).
const DECAL_HIT_SHARE: f32 = 0.05;
const DECAL_BEHIND_SHARE: f32 = 0.1;
const DECAL_BEHIND_REACH: f32 = 4.0;
const DECAL_JITTER: f32 = 0.3;
/// Particle colour / alpha steps (one shared material each).
const STEPS: usize = 32;
/// Streaked particles (motion blur) are drawn this many seconds of motion long.
const STREAK_TIME: f32 = 0.035;
const MAX_PARTICLES: usize = 2500;
const MAX_DECALS: usize = 80;
/// A decal's width / height read as half sizes: the capture's hit splats are about 1 m across
/// (h_14410af1 says 0.5 x 0.25), landing 1-3 m past the one hit, along the shot. The code agrees
/// (#129, answers.md Q4): FUN_00217920 builds a decal's UVs as 0.5 + d x 0.5 / extent, so it
/// spans 2 x width x scale by 2 x height x scale (FUN_00214710 scales them).
const DECAL_SCALE: f32 = 2.0;
const SPLAT_PAST: (f32, f32) = (1.0, 2.5);
/// How far inside the hit cylinder (radius 0.4 m) a body's skin is, and how far out of its middle.
const SKIN_IN: f32 = 0.25;
const SKIN_OUT: f32 = 0.15;
/// The pool goes under the body this long after death (it has stopped sliding by then).
const POOL_DELAY: f32 = 2.0;
/// A death splashes blood round where the body falls: the hit's blood effects sprayed up from
/// DEATH_SPRAY_UP m, and DEATH_SPLATS of the character's hit splats on the ground within
/// DEATH_SPLAT_REACH m. The demo's choice: the game's own death blood isn't known.
const DEATH_SPRAY_UP: f32 = 0.9;
const DEATH_SPLATS: usize = 5;
const DEATH_SPLAT_REACH: f32 = 1.4;
/// A light's lumens per unit of (alpha x size x colour) of its live particles (tuned to the
/// capture's brightening around the old DNA effect's light_powerup_pill, which is gone; the hit
/// lights keep it).
const LIGHT_LUMENS: f32 = 30_000.0;
const MAX_LIGHTS: usize = 16;

/// An emitter node (.emt).
struct Emit {
    /// the node's lifespan (s); INFINITY: until removed
    time: f32,
    rate: Family,
    life: Keys,
    speed: Family,
    /// edge of the box particles start in (m)
    size: f32,
    /// angle from the axis (degrees) over emitter time: min, max
    spread_min: Keys,
    spread_max: Keys,
    /// a gravity field with the emitter (m/s^2)
    gravity: f32,
}

/// An appearance node (.app): a camera-facing textured quad.
struct Look {
    texture: u32,
    additive: bool,
    color: &'static [(f32, [f32; 3])],
    alpha: Keys,
    size: Family,
    size_ease: u8,
    /// height / width
    aspect: f32,
    /// width / size over life (thin streaks)
    width: Keys,
    /// drawn along its motion (the node's motion-blur flag)
    streak: bool,
    /// roll over life (rad), from a random start
    spin: Keys,
}

struct Effect {
    id: usize,
    emit: Emit,
    look: Look,
}

/// A "light_" effect: its particles are a point light's, not sprites: `size` is the light's
/// reach (m), colour x alpha its strength. `rate` 0: one particle, at the start. `back`: how far
/// back along the shot from the hit it stands (m): its emitter's offset (TRANSFORM tz) in the
/// frame of its effect type's light-effect, alignment 0 (the shot's way, its +z back toward the
/// shooter; FUN_0001d0c0's frame, play.rs `hit_frame`).
struct Glow {
    rate: f32,
    life: f32,
    color: &'static [(f32, [f32; 3])],
    alpha: f32,
    size: Keys,
    back: f32,
}

/// light_hit_cutter (wmat_fleshit_cutter's light-effect h_16d39d8b): a pale blue flash, its
/// emitter light_hit_cutter.emt 2.70 m back along the shot (tz 2.6992). The demo had it 0.3 m
/// off the body, which washed a struck Shrike cyan-white from head to foot for 6 frames where
/// #65 take03 shows a green blotch on its chest (#287: with both cutter lights left out the
/// demo's hit is that blotch; at 2.70 m, as the data places it, the light reaches the body
/// faintly)
static CUTTER_LIGHT: Glow = Glow {
    rate: 0.0, life: 0.3866, color: &[(0.0, [0.545, 0.796, 0.871]), (1.0, [0.0, 0.0, 0.0])], alpha: 0.583,
    size: &[(0.0, 1.28), (0.434, 4.04)], back: 2.6992,
};

/// light_hit_laser (wmat_fleshit_laser's light): an orange flash, its emitter light_hit_laser.emt
/// 2.67 m back along the shot (tz 2.6659)
static LASER_LIGHT: Glow = Glow {
    rate: 0.0, life: 0.3866, color: &[(0.0, [0.886, 0.294, 0.0941]), (1.0, [0.0, 0.0, 0.0])], alpha: 0.583,
    size: &[(0.0, 1.01), (1.0, 3.98)], back: 2.6659,
};

/// blood_puff: blood_mist.emt / blood_mist.app
static BLOOD_MIST: Effect = Effect {
    id: 0,
    emit: Emit {
        time: 0.19,
        rate: &[(0.0135, &[(-0.0145, 58.43), (0.1596, 50.05), (0.177, 0.0)]),
                (1.0, &[(-0.0084, 78.2), (0.115, 75.5), (0.179, 0.0)])],
        life: &[(0.0162, 0.469), (0.0484, 0.31), (0.103, 0.474), (0.129, 0.334)],
        speed: &[(0.0, &[(0.0, 0.5543)])],
        size: 0.2412,
        spread_min: &[(0.0, 28.48)],
        spread_max: &[(0.0, 90.0)],
        gravity: 0.0,
    },
    look: Look {
        texture: 0xEA18_1967,   // smokecard.tga
        additive: false,
        color: &[(0.0, [0.871, 0.141, 0.106]), (0.466, [0.592, 0.125, 0.125])],
        alpha: &[(0.0, 0.893), (1.0, 0.0)],
        size: &[(0.0, &[(0.0, 0.021), (0.0524, 0.335), (1.0, 1.144)]),
                (1.0, &[(0.0, 0.0385), (0.0536, 0.438), (1.0, 1.6)])],
        size_ease: LINEAR,
        aspect: 1.0,
        width: &[(0.0, 1.0)],
        streak: false,
        spin: &[(0.0, 0.0)],
    },
};

/// bloodsplat_s: bloodsplat_s.emt (a cone) / bloodsplat_s.app / bloodsplat_s_gravfld. Late
/// droplets leave backwards (the speed curve goes negative): out of the far side.
static BLOOD_DROPS: Effect = Effect {
    id: 1,
    emit: Emit {
        time: 0.3,
        rate: &[(0.189, &[(0.0005, 258.0), (0.199, 0.0)]),
                (0.73, &[(-0.0067, 328.0), (0.206, 0.0)]),
                (1.0, &[(-0.0127, 324.0), (0.2, 199.0), (0.311, 196.0), (0.313, 0.0)])],
        life: &[(0.00527, 0.959), (0.0462, 0.417)],
        speed: &[(0.0, &[(-0.00167, 4.79), (0.0611, 2.44), (0.0795, -4.31)]),
                 (1.0, &[(0.00207, 8.19), (0.0611, 5.97), (0.0846, -6.48)])],
        size: 0.046,
        spread_min: &[(0.0, 0.0)],
        spread_max: &[(0.00737, 0.0), (0.197, 9.07)],
        gravity: 2.12,
    },
    look: Look {
        texture: 0x1500_CF27,   // diffuse_big.tga
        additive: false,
        color: &[(0.0172, [0.792, 0.0, 0.0]), (0.408, [0.467, 0.0, 0.0])],
        alpha: &[(0.0, 0.0), (0.0641, 0.606), (1.0, 0.0)],
        size: &[(0.0, &[(0.0, 0.156), (0.0727, 0.0673), (1.0, 0.0)])],
        size_ease: LINEAR,
        aspect: 1.0,
        width: &[(0.0, 1.0)],
        streak: true,
        spin: &[(0.0, 0.0)],
    },
};

/// giblet: giblet.emt / giblet.app (one to five spatters)
static GIBLET: Effect = Effect {
    id: 2,
    emit: Emit {
        time: 0.2,
        rate: &[(0.0, &[(0.0199, 5.91), (0.165, 4.57), (0.1651, 0.0)]),
                (0.784, &[(-0.001, 22.0), (0.16, 19.5), (0.166, 0.0)]),
                (1.0, &[(0.009, 35.8), (0.154, 33.5), (0.185, 0.0)])],
        life: &[(0.00299, 0.241), (0.0898, 0.159)],
        speed: &[(0.0, &[(0.0123, 1.71), (0.101, 1.69), (0.173, -1.25), (0.261, -1.39)])],
        size: 0.1094,
        spread_min: &[(0.0, 0.0)],
        spread_max: &[(0.0, 10.57)],
        gravity: 0.0,
    },
    look: Look {
        texture: 0xFBD5_61C9,   // blood.tga
        additive: false,
        color: &[(0.0, [0.839, 0.188, 0.188])],
        alpha: &[(0.0, 0.726), (1.0, 0.0)],
        size: &[(0.0, &[(0.0, 0.133), (1.0, 0.953)])],
        size_ease: LINEAR,
        aspect: 1.0,
        width: &[(0.0, 1.0)],
        streak: false,
        spin: &[(0.009, -0.0009), (1.0, 0.156)],
    },
};

/// wmat_fleshit_standard (ballistic ammo): fleshit_standard.emt / .app / _grav
static FLESH_SPARKS: Effect = Effect {
    id: 3,
    emit: Emit {
        time: 0.1,
        rate: &[(0.0, &[(0.0135, 351.0), (0.069, 0.0)])],
        life: &[(0.00292, 0.578), (0.0219, 0.398)],
        speed: &[(0.0, &[(0.00105, 7.83), (0.0257, 10.3), (0.0453, 3.52)])],
        size: 0.03,
        spread_min: &[(0.0, 0.0)],
        spread_max: &[(0.0, 77.48)],
        gravity: 12.44,
    },
    look: Look {
        texture: 0xFEA4_C6FA,   // spark.tga
        additive: true,
        color: &[(0.0172, [1.0, 1.0, 1.0]), (0.132, [1.0, 0.969, 0.835]), (0.408, [0.953, 0.553, 0.361])],
        alpha: &[(0.0, 0.774), (0.531, 0.749), (1.0, 0.0)],
        size: &[(0.0, &[(0.0, 0.269), (0.0802, 0.119), (1.0, 0.0655)])],
        size_ease: EASE_IN_OUT,
        aspect: 1.75,
        width: &[(0.0, 1.0)],
        streak: true,
        spin: &[(0.0, 0.0)],
    },
};

/// wmat_fleshit_cutter (ammo type 7, Brutus's): fleshit_cutter.emt / .app / .fld: long thin
/// streaks
static FLESH_CUTTER: Effect = Effect {
    id: 4,
    emit: Emit {
        time: 0.2,
        rate: &[(0.0, &[(0.00665, 594.0), (0.069, 0.0)])],
        life: &[(0.00413, 0.857), (0.0146, 0.388), (0.023, 0.777)],
        speed: &[(0.0, &[(0.00217, 8.22), (0.00482, 6.51), (0.00951, 8.87), (0.015, 3.29), (0.0181, 7.4)])],
        size: 0.007,
        spread_min: &[(0.0, 0.0)],
        spread_max: &[(0.0, 90.0)],
        gravity: 13.37,
    },
    look: Look {
        texture: 0xFEA4_C6FA,   // spark.tga
        additive: true,
        color: &[(0.0172, [0.91, 0.89, 0.549]), (0.167, [1.0, 0.863, 0.31])],
        alpha: &[(0.0129, 0.958), (1.0, 0.0)],
        size: &[(0.0, &[(0.0129, 0.995), (0.0965, 0.643), (0.26, 0.61)])],
        size_ease: EASE_IN_OUT,
        aspect: 1.0,
        width: &[(0.0, 0.353), (0.891, 0.132)],
        streak: true,
        spin: &[(0.0, 0.0)],
    },
};

/// wmat_fleshit_laser (ammo type 11, Tex's): fleshit_laser.emt (a cone opening up to 90
/// degrees) / .app / _gravfld: flickering embers
static FLESH_LASER: Effect = Effect {
    id: 5,
    emit: Emit {
        time: 0.2,
        rate: &[(0.0, &[(0.00238, 130.0), (0.155, 129.0), (0.164, 0.0)]),
                (0.973, &[(0.0132, 162.0), (0.155, 129.0), (0.181, 0.0)])],
        life: &[(-0.00439, 0.413), (0.0874, 0.554), (0.116, 0.226), (0.198, 0.274)],
        speed: &[(0.0, &[(0.00509, 0.65), (0.0447, 1.68), (0.0634, 0.59)])],
        size: 0.06,
        spread_min: &[(0.00804, 13.7), (0.284, 90.0)],
        spread_max: &[(-0.00133, 16.4), (0.188, 90.0)],
        gravity: 0.271,
    },
    look: Look {
        texture: 0x1E87_F03A,   // small-flare.tga
        additive: true,
        color: &[(0.0, [0.961, 0.859, 0.647]), (0.172, [0.957, 0.424, 0.0667]), (0.954, [0.592, 0.114, 0.102])],
        alpha: &[(0.0, 0.0), (0.131, 0.838), (0.244, 0.415), (0.36, 0.689), (0.502, 0.415), (0.74, 0.516), (1.0, 0.0)],
        size: &[(0.0, &[(0.0, 0.48), (0.0832, 0.118), (1.0, 0.0)])],
        size_ease: LINEAR,
        aspect: 1.0,
        width: &[(0.0, 1.0)],
        streak: true,
        spin: &[(0.0, 0.0)],
    },
};

/// blood_puff_b (synthetic flesh): blood_mist_b.emt / blood_mist_b.app. Grey, slower and
/// fewer than blood_mist.
static BLOOD_MIST_B: Effect = Effect {
    id: 6,
    emit: Emit {
        time: 0.21,
        rate: &[(0.0135, &[(-0.0088, 49.92), (0.1939, 42.1), (0.2056, 0.0)]),
                (1.0, &[(-0.0027, 69.7), (0.1491, 67.54), (0.2076, 0.0)])],
        life: &[(0.0158, 0.546), (0.0802, 0.333), (0.122, 0.503), (0.161, 0.357)],
        speed: &[(0.0, &[(0.0, 0.2268)])],
        size: 0.2412,
        spread_min: &[(0.0, 28.48)],
        spread_max: &[(0.0, 90.0)],
        gravity: 0.0,
    },
    look: Look {
        texture: 0xEA18_1967,   // smokecard.tga
        additive: false,
        color: &[(0.0115, [0.51, 0.51, 0.51]), (0.684, [0.259, 0.278, 0.31])],
        alpha: &[(0.0, 0.809), (1.0, 0.0)],
        size: &[(0.0, &[(0.0, 0.021), (0.0524, 0.3285), (1.0, 1.094)]),
                (1.0, &[(0.0, 0.0385), (0.0536, 0.3723), (1.0, 1.465)])],
        size_ease: LINEAR,
        aspect: 1.0,
        width: &[(0.0, 1.0)],
        streak: false,
        spin: &[(0.0, 0.0)],
    },
};

/// bloodsplat_b (synthetic flesh): bloodsplat_b.emt / bloodsplat_b.app /
/// bloodsplat_s_gravfld_b. Grey-blue droplets, slower and under less gravity than bloodsplat_s.
static BLOOD_DROPS_B: Effect = Effect {
    id: 7,
    emit: Emit {
        time: 0.2,
        rate: &[(0.189, &[(0.0005, 258.1), (0.0647, 152.5), (0.1032, 187.1), (0.1513, 0.0)]),
                (1.0, &[(0.0066, 289.8), (0.0583, 169.5), (0.1434, 199.1), (0.1457, 0.0)])],
        life: &[(0.00352, 0.984), (0.0462, 0.417)],
        speed: &[(0.0, &[(-0.00167, 3.21), (0.0744, 3.0), (0.1072, -2.9)]),
                 (1.0, &[(0.00207, 4.98), (0.0744, 4.92), (0.1124, -5.07)])],
        size: 0.046,
        spread_min: &[(0.0, 0.0)],
        spread_max: &[(0.00737, 0.0), (0.1184, 12.32)],
        gravity: 1.445,
    },
    look: Look {
        texture: 0x1500_CF27,   // diffuse_big.tga
        additive: false,
        color: &[(0.0172, [0.722, 0.745, 0.788]), (0.322, [0.239, 0.255, 0.306])],
        alpha: &[(0.0, 0.0), (0.0641, 0.606), (1.0, 0.0)],
        size: &[(0.0, &[(0.0, 0.156), (0.0727, 0.0673), (1.0, 0.0)])],
        size_ease: LINEAR,
        aspect: 1.0,
        width: &[(0.0, 1.0)],
        streak: true,
        spin: &[(0.0, 0.0)],
    },
};

/// giblet_b (synthetic flesh): giblet_b.emt / giblet_b.app. Dark blue-grey spatters, twice
/// giblet's emitter size.
static GIBLET_B: Effect = Effect {
    id: 8,
    emit: Emit {
        time: 0.22,
        rate: &[(0.0, &[(0.0199, 5.91), (0.2021, 5.95), (0.2021, 0.0)]),
                (1.0, &[(0.0075, 23.63), (0.1897, 22.78), (0.2221, 0.0)])],
        life: &[(0.00299, 0.224), (0.0962, 0.105)],
        speed: &[(0.0, &[(0.0059, 1.32), (0.0942, 1.3), (0.173, -1.25), (0.261, -1.39)])],
        size: 0.2166,
        spread_min: &[(0.0, 0.0)],
        spread_max: &[(0.0, 10.57)],
        gravity: 0.0,
    },
    look: Look {
        texture: 0xFBD5_61C9,   // blood.tga
        additive: false,
        color: &[(0.0, [0.314, 0.341, 0.404])],
        alpha: &[(0.0, 0.678), (1.0, 0.0)],
        size: &[(0.0, &[(0.0002, 0.133), (1.0, 0.953)])],
        size_ease: LINEAR,
        aspect: 1.0,
        width: &[(0.0, 1.0)],
        streak: false,
        spin: &[(0.009, -0.0009)],
    },
};

/// blood_puff#1 (Shrike flesh, #129): blood_mist.emt#1, the very node blood_mist_b.emt is (every
/// parameter the same), and blood_mist.app#1: green, starting at 0.29, 0.81, 0.16 (common.ale,
/// `ale_tool.py node`).
static BLOOD_MIST_1: Effect = Effect {
    id: 9,
    emit: Emit {
        time: 0.21,
        rate: &[(0.0135, &[(-0.0088, 49.92), (0.1939, 42.1), (0.2056, 0.0)]),
                (1.0, &[(-0.0027, 69.7), (0.1491, 67.54), (0.2076, 0.0)])],
        life: &[(0.0158, 0.546), (0.0802, 0.333), (0.122, 0.503), (0.161, 0.357)],
        speed: &[(0.0, &[(0.0, 0.2268)])],
        size: 0.2412,
        spread_min: &[(0.0, 28.48)],
        spread_max: &[(0.0, 90.0)],
        gravity: 0.0,
    },
    look: Look {
        texture: 0xEA18_1967,   // smokecard.tga
        additive: false,
        color: &[(0.0, [0.290, 0.812, 0.161]), (1.0, [0.306, 0.502, 0.290])],
        alpha: &[(0.0, 0.678), (1.0, 0.0)],
        size: &[(0.0, &[(0.0, 0.021), (0.0524, 0.3285), (1.0, 1.2777)]),
                (1.0, &[(0.0, 0.0385), (0.0536, 0.3723), (1.0, 1.6487)])],
        size_ease: LINEAR,
        aspect: 1.0,
        width: &[(0.0, 1.0)],
        streak: false,
        spin: &[(0.0, 0.0)],
    },
};

/// bloodsplat_s#1 (Shrike flesh): bloodsplat_s.emt#1 (a longer, steadier jet than bloodsplat_s's,
/// 0.4 s), bloodsplat_s.app#1 (green, 0.17, 0.79, 0.15) and its field bloodsplat_s_gravfld_y.
static BLOOD_DROPS_1: Effect = Effect {
    id: 10,
    emit: Emit {
        time: 0.4,
        rate: &[(0.189, &[(0.0005, 258.08), (0.0647, 152.51), (0.1032, 187.13), (0.1513, 0.0)]),
                (0.703, &[(0.0035, 257.44), (0.1259, 0.0)]),
                (1.0, &[(-0.0042, 323.77), (0.0637, 225.09), (0.3558, 193.33), (0.3581, 0.0)])],
        life: &[(0.00419, 0.936), (0.0462, 0.417)],
        speed: &[(0.0, &[(0.0235, 3.907), (0.1059, 3.798), (0.1135, -2.803)]),
                 (1.0, &[(0.0272, 6.777), (0.1059, 6.809), (0.1124, -5.068)])],
        size: 0.046,
        spread_min: &[(0.0, 0.0)],
        spread_max: &[(0.00737, 0.0), (0.2664, 8.651)],
        gravity: 2.312,
    },
    look: Look {
        texture: 0x1500_CF27,   // diffuse_big.tga
        additive: false,
        color: &[(0.0172, [0.173, 0.788, 0.153])],
        alpha: &[(0.0, 0.0), (0.0641, 0.606), (1.0, 0.0)],
        size: &[(0.0, &[(0.0, 0.156), (0.0727, 0.0673), (1.0, 0.0)])],
        size_ease: LINEAR,
        aspect: 1.0,
        width: &[(0.0, 1.0)],
        streak: true,
        spin: &[(0.0, 0.0)],
    },
};

/// giblet#1 (Shrike flesh): giblet.emt#1 (fewer spatters at the top of the user parameter than
/// giblet's, 0.25 s) and giblet.app#1 (green, 0.31, 0.65, 0.25).
static GIBLET_1: Effect = Effect {
    id: 11,
    emit: Emit {
        time: 0.25,
        rate: &[(0.0, &[(0.0199, 5.911), (0.2021, 5.946), (0.2021, 0.0)]),
                (1.0, &[(-0.0046, 21.01), (0.1776, 20.16), (0.2221, 0.0)])],
        life: &[(-0.00345, 0.236), (0.0962, 0.165)],
        speed: &[(0.0, &[(0.0059, 1.325), (0.0942, 1.304), (0.1731, -1.245), (0.2607, -1.391)])],
        size: 0.1094,
        spread_min: &[(0.0, 0.0)],
        spread_max: &[(0.0, 10.57)],
        gravity: 0.0,
    },
    look: Look {
        texture: 0xFBD5_61C9,   // blood.tga
        additive: false,
        color: &[(0.0057, [0.310, 0.655, 0.247])],
        alpha: &[(0.0, 0.857), (1.0, 0.0)],
        size: &[(0.0, &[(0.00015, 0.133), (1.0, 0.953)])],
        size_ease: LINEAR,
        aspect: 1.0,
        width: &[(0.0, 1.0)],
        streak: false,
        spin: &[(0.0091, -0.0009)],
    },
};

static EFFECTS: [&Effect; 12] = [&BLOOD_MIST, &BLOOD_DROPS, &GIBLET, &FLESH_SPARKS, &FLESH_CUTTER, &FLESH_LASER,
                                 &BLOOD_MIST_B, &BLOOD_DROPS_B, &GIBLET_B, &BLOOD_MIST_1, &BLOOD_DROPS_1, &GIBLET_1];

/// The debris bundle every flesh hit gets (h_f304f9fb).
static BLOOD: [&Effect; 3] = [&BLOOD_DROPS, &GIBLET, &BLOOD_MIST];
/// Synthetic flesh's debris bundle (h_1f7ca034).
static BLOOD_B: [&Effect; 3] = [&BLOOD_DROPS_B, &GIBLET_B, &BLOOD_MIST_B];
/// Shrike flesh's debris bundle (h_14559f7c).
static BLOOD_1: [&Effect; 3] = [&BLOOD_DROPS_1, &GIBLET_1, &BLOOD_MIST_1];

/// A flesh material's hit effect for an ammo type: the squad's three (ballistic, cutter, laser)
/// and their lights, made by hand here. Any other row's effect type (the sonic h_177a62d8, the
/// particle h_ed6363bd, the bio h_f8d4e76d, the rail's, the energy's) is played from the data
/// instead (`spawn_fx`, play.rs `hit_effect_at`).
fn hit_effect(surface: Option<&Surface>, ammo: i64) -> (Option<&'static Effect>, Option<&'static Glow>) {
    match surface.and_then(|s| s.hit_effect.get(&ammo)) {
        Some(0x1D25_07AB) => (Some(&FLESH_SPARKS), None),       // wmat_fleshit_standard
        Some(0xF72C_8C11) => (Some(&FLESH_CUTTER), Some(&CUTTER_LIGHT)),
        Some(0xFCBB_F2B9) => (Some(&FLESH_LASER), Some(&LASER_LIGHT)),
        _ => (None, None),
    }
}

/// The user parameter of a shot's flesh hit effect played from the data (FUN_0022cd80,
/// play.rs `hit_sp`): from the hit record's damage (`Bleed::record`), 1.0 for a killing hit; 0
/// for a hit without a record. (The hand-made effects keep their random pick.)
fn flesh_sp(b: &Bleed) -> f32 {
    b.record.map_or(0.0, |r| super::hit_sp(r, b.hp_left <= 0.0))
}

fn ease(kind: u8, f: f32) -> f32 {
    match kind {
        EASE_IN => f * f,
        3 => 1.0 - (1.0 - f) * (1.0 - f),
        EASE_IN_OUT => f * f * (3.0 - 2.0 * f),
        _ => f,
    }
}

/// A key track at `t`: eased between keys, held past the ends.
fn at(keys: Keys, kind: u8, t: f32) -> f32 {
    let Some(&(t0, v0)) = keys.first() else { return 0.0 };
    if t <= t0 {
        return v0;
    }
    for w in keys.windows(2) {
        let ((a, va), (b, vb)) = (w[0], w[1]);
        if t <= b {
            let f = if b > a { (t - a) / (b - a) } else { 1.0 };
            return va + (vb - va) * ease(kind, f);
        }
    }
    keys[keys.len() - 1].1
}

/// A family of tracks at sparam `sp`: the two items around it, blended.
fn family(f: Family, kind: u8, sp: f32, t: f32) -> f32 {
    let i = f.iter().position(|(s, _)| *s >= sp).unwrap_or(f.len() - 1);
    if i == 0 || f[i].0 <= sp {
        return at(f[i].1, kind, t);
    }
    let ((s0, k0), (s1, k1)) = (f[i - 1], f[i]);
    let w = (sp - s0) / (s1 - s0);
    at(k0, kind, t) * (1.0 - w) + at(k1, kind, t) * w
}

/// A colour track at `t` (eased in-out between keys, held past the ends).
fn color_at(keys: &[(f32, [f32; 3])], t: f32) -> [f32; 3] {
    let i = keys.iter().position(|(k, _)| *k >= t).unwrap_or(keys.len() - 1);
    if i == 0 || keys[i].0 <= t {
        return keys[i].1;
    }
    let ((a, ca), (b, cb)) = (keys[i - 1], keys[i]);
    let f = ease(EASE_IN_OUT, (t - a) / (b - a));
    [0, 1, 2].map(|c| ca[c] + (cb[c] - ca[c]) * f)
}

#[derive(Resource)]
struct FxAssets {
    quad: Handle<Mesh>,
    floor: Handle<Mesh>,
    /// per effect: its look's material at each step of a particle's life
    steps: Vec<Vec<Handle<StandardMaterial>>>,
    decal_textures: HashMap<u32, Handle<Image>>,
    /// the game's decal colour per (decal texture, tint) of the character types' decals
    /// (`decal_factor`, `spawn_game_decal`)
    decal_factors: HashMap<(u32, [u8; 3]), Handle<Image>>,
    flesh: Option<Surface>,
    /// SYNTHETIC's entry: Flint's hit sounds and effects
    synthetic: Option<Surface>,
    /// SHRIKE's entry (the Shrikes' hit sounds and effects: FLESH_HUMAN's, #129)
    shrike: Option<Surface>,
}

#[derive(Resource)]
struct FxState {
    rng: u32,
    particles: usize,
    /// hit lights alive
    lights: usize,
    decals: std::collections::VecDeque<Entity>,
    /// the bullet holes alive, oldest first (see `MAX_HOLES`)
    holes: std::collections::VecDeque<Entity>,
}

impl FxState {
    fn random(&mut self) -> f32 {
        self.rng ^= self.rng << 13;
        self.rng ^= self.rng >> 17;
        self.rng ^= self.rng << 5;
        (self.rng >> 8) as f32 / (1u32 << 24) as f32
    }
}

#[derive(Component)]
struct Emitter {
    fx: &'static Effect,
    at: Vec3,
    axis: Vec3,
    t: f32,
    acc: f32,
    sp: f32,
}

#[derive(Component)]
struct Particle {
    fx: &'static Effect,
    age: f32,
    life: f32,
    vel: Vec3,
    sp: f32,
    roll: f32,
    step: usize,
}

/// A decal on the ground or a wall: its age, its def (life, fade, tint) and material.
#[derive(Component)]
struct Decal {
    age: f32,
    def: DecalDef,
    material: Handle<StandardMaterial>,
    /// drawn as the game's decal pass draws it (`spawn_game_decal`: multiplied in, out of Bevy's
    /// fog): it fades to no effect with the fog at it instead (`animate_fx`)
    multiply: bool,
}

/// A point light run by a light effect: its particles' ages.
#[derive(Component)]
struct GlowLight {
    def: &'static Glow,
    ages: Vec<f32>,
    acc: f32,
}

/// Ground decals other modules ask for this frame: (decal definition name, where, scale on the
/// data's width / height). A grenade's blast leaves its bullet's decal (the Frag's scorch
/// h_ff1b711e) this way.
#[derive(Resource, Default)]
pub struct DecalRequests(pub Vec<(u32, Vec3, f32)>);

/// Bullet holes and scorches other modules ask for this frame (play_shots.rs): (the bullet's
/// h_06a27365 decal, where the shot met the surface, the surface's normal facing the shooter).
#[derive(Resource, Default)]
pub struct HoleRequests(pub Vec<(u32, Vec3, Vec3)>);

/// How many bullet holes stay at once; the oldest goes first. A guess: the game's own limit
/// isn't known (a Minigun clip is 80 rounds; the holes' data life is 25 s).
const MAX_HOLES: usize = 160;
/// A bullet hole's width / height read as full sizes (h_1c8c53ea, h_0e8a266c ... 0.3 x 0.3 m):
/// three MK holes at 10 m make a ~6-unit mark in 63/take01 at 9.3 s, about what a 0.3 m quad
/// with the hole textures' alpha gives. The blood decals read theirs as half sizes.
const HOLE_SCALE: f32 = 1.0;

/// The hit effects' systems (`plugin`): what hits this frame (a bolt's, play_bolts.rs) is made
/// before them, so its blood, hole and decals show on its frame.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct FxSystems;

pub fn plugin(app: &mut App) {
    app.init_resource::<DecalRequests>()
        .init_resource::<HoleRequests>()
        .add_systems(OnEnter(AppState::Playing), setup_fx.after(snapshot_entities))
        .add_systems(Update, (spawn_fx, requested_decals, requested_holes, emit, animate_fx).chain().in_set(FxSystems).after(update_player).before(play_sounds)
            .run_if(in_state(AppState::Playing)));
}

/// The decals asked for (`DecalRequests`), their textures loaded the first time, at the scale
/// asked for, darkening as the console's blend does (see `spawn_decal_alpha`).
fn requested_decals(mut commands: Commands, mut game: ResMut<GameData>, fx: Option<ResMut<FxAssets>>, mut state: ResMut<FxState>,
                    mut requests: ResMut<DecalRequests>, mut images: ResMut<Assets<Image>>,
                    mut materials: ResMut<Assets<StandardMaterial>>) {
    let Some(mut fx) = fx else { return };
    for (name, at, scale) in std::mem::take(&mut requests.0) {
        let Some(def) = game.0.decals.get(&name).cloned() else { continue };
        if std::env::var("BF_GRENADE_LOG").is_ok() {
            println!("decal h_{name:08x} at {at:.2}: {def:?}, textures loaded {:?}",
                     def.textures.iter().map(|t| fx.decal_textures.contains_key(t) || game.0.texture_rgba(*t).is_some()).collect::<Vec<_>>());
        }
        for &id in &def.textures {
            if !fx.decal_textures.contains_key(&id) {
                if let Some(mips) = game.0.texture_mips(id) {
                    fx.decal_textures.insert(id, images.add(rgba(&mips, false)));
                }
            }
        }
        spawn_decal_alpha(&mut commands, &mut state, &fx, &mut materials, &def, at, scale, true);
    }
}

/// The bullet holes asked for (`HoleRequests`): the decal laid on the surface the shot met,
/// turned at random within its data rotation, fading over its data life as the ground decals
/// do, darkening as the console's blend does (see `spawn_decal_alpha`).
fn requested_holes(mut commands: Commands, mut game: ResMut<GameData>, fx: Option<ResMut<FxAssets>>, mut state: ResMut<FxState>,
                   mut requests: ResMut<HoleRequests>, mut images: ResMut<Assets<Image>>,
                   mut materials: ResMut<Assets<StandardMaterial>>) {
    let Some(mut fx) = fx else { return };
    for (name, at, normal) in std::mem::take(&mut requests.0) {
        let Some(def) = game.0.decals.get(&name).cloned() else { continue };
        if def.textures.is_empty() {
            continue;
        }
        // (BF_SHOT_LOG: its data life and fade, h_199870ec / h_fe4e1d82: 25 / 5 s for most, the
        // bio guns' green h_1ce6db03 60 / 5)
        if std::env::var("BF_SHOT_LOG").is_ok() {
            println!("hole h_{name:08x} at {at:.2}: life {} s, fading over its last {} s", def.life, def.fade);
        }
        let texture = def.textures[(state.random() * def.textures.len() as f32) as usize % def.textures.len()];
        if !fx.decal_textures.contains_key(&texture) {
            if let Some(mips) = game.0.texture_mips(texture) {
                fx.decal_textures.insert(texture, images.add(rgba(&mips, false)));
            }
        }
        let [r, g, b, a] = def.color;
        let a = 1.0 - (1.0 - a).powf(2.2);
        let material = materials.add(StandardMaterial {
            base_color: Color::srgba(r, g, b, a), base_color_texture: fx.decal_textures.get(&texture).cloned(),
            alpha_mode: AlphaMode::Blend, perceptual_roughness: 1.0, reflectance: 0.0, depth_bias: 40.0, ..default()
        });
        let turn = (state.random() * 2.0 - 1.0) * def.rotation.to_radians();
        let normal = normal.normalize_or(Vec3::Y);
        let e = commands.spawn((Mesh3d(fx.floor.clone()), MeshMaterial3d(material.clone()), NotShadowCaster,
                                Transform::from_translation(at + normal * (0.01 + 0.004 * state.random()))
                                    .with_rotation(Quat::from_rotation_arc(Vec3::Y, normal) * Quat::from_rotation_y(turn))
                                    .with_scale(Vec3::new(def.width * HOLE_SCALE, 1.0, def.height * HOLE_SCALE)),
                                Decal { age: 0.0, def: def.clone(), material, multiply: false })).id();
        state.holes.push_back(e);
        while state.holes.len() > MAX_HOLES {
            if let Some(old) = state.holes.pop_front() {
                commands.entity(old).despawn();
            }
        }
    }
}

/// A sprite's or a decal's texture with the mip levels its archive ships (`scene::mip_image`),
/// clamped at its edges as before. `linear`: the texels as they are (for sprites added to the
/// picture, see `setup_fx`).
fn rgba(mips: &Mips, linear: bool) -> Image {
    use bevy::render::render_resource::TextureFormat;
    bf_viewer::scene::mip_image(mips, if linear { TextureFormat::Rgba8Unorm } else { TextureFormat::Rgba8UnormSrgb }, [Address::Clamp; 2])
}

fn setup_fx(mut commands: Commands, mut game: ResMut<GameData>, mut meshes: ResMut<Assets<Mesh>>,
            mut images: ResMut<Assets<Image>>, mut materials: ResMut<Assets<StandardMaterial>>) {
    // the blood decals' textures: the squad's, and every loaded character type's (the same white
    // splats, tinted: the Shrikes' green, the Seers' purple)
    let mut decal_ids: Vec<u32> = game.0.character_decals.values().flat_map(|&(a, b)| [a, b])
        .chain(game.0.character_types.values().flat_map(|t| t.decals.iter().flatten().copied()))
        .flat_map(|name| game.0.decals.get(&name).map(|d| d.textures.clone()).unwrap_or_default()).collect();
    decal_ids.sort();
    decal_ids.dedup();
    let flesh = game.0.surfaces.iter().find(|s| s.id == FLESH).cloned();
    let synthetic = game.0.surfaces.iter().find(|s| s.id == SYNTHETIC).cloned();
    let shrike = game.0.surfaces.iter().find(|s| s.id == SHRIKE).cloned();
    let mut tex = |id: u32, linear: bool| game.0.texture_mips(id).map(|mips| images.add(rgba(&mips, linear)));
    let mut steps = vec![];
    for fx in EFFECTS {
        // The Xbox adds sprites to the picture's stored (gamma) values; Bevy adds light. Taking
        // an additive sprite's texels and colour as linear amounts adds about what the console
        // did on a mid-grey picture (as sRGB they come out a third as bright).
        let additive = fx.look.additive;
        let texture = tex(fx.look.texture, additive);
        if texture.is_none() {
            warn!("effect texture h_{:08x} missing", fx.look.texture);
        }
        steps.push((0..STEPS).map(|i| {
            let k = (i as f32 + 0.5) / STEPS as f32;
            let [r, g, b] = color_at(fx.look.color, k);
            let a = at(fx.look.alpha, EASE_IN_OUT, k);
            materials.add(StandardMaterial {
                base_color: if additive { Color::linear_rgba(r, g, b, a) } else { Color::srgba(r, g, b, a) },
                base_color_texture: texture.clone(), unlit: true, double_sided: true, cull_mode: None,
                alpha_mode: if fx.look.additive { AlphaMode::Add } else { AlphaMode::Blend }, ..default()
            })
        }).collect());
    }
    let mut decal_textures = HashMap::new();
    for id in decal_ids {
        if let (std::collections::hash_map::Entry::Vacant(e), Some(t)) = (decal_textures.entry(id), tex(id, false)) {
            e.insert(t);
        }
    }
    // the game's decal colour of every character type's decal textures under their tints (the
    // game's rule draws Shrike flesh's: `spawn_game_decal`)
    let mut decal_factors = HashMap::new();
    let defs: Vec<DecalDef> = game.0.character_types.values().flat_map(|t| t.decals.iter().flatten().copied())
        .filter_map(|name| game.0.decals.get(&name).cloned()).collect();
    for def in defs {
        let tint = tint_key(&def);
        for &id in &def.textures {
            if decal_factors.contains_key(&(id, tint)) {
                continue;
            }
            if let Some(mut mips) = game.0.texture_mips(id) {
                mips.map_levels(|_, _, px| decal_factor(px, tint));
                decal_factors.insert((id, tint), images.add(rgba(&mips, false)));
            }
        }
    }
    if let Some(f) = &flesh {
        info!("flesh (world material {FLESH}): hit effects {:x?}, debris effect {:x?}", f.hit_effect, f.debris_effect);
    }
    commands.insert_resource(FxAssets {
        quad: meshes.add(Rectangle::new(1.0, 1.0)),
        floor: meshes.add(Plane3d::default().mesh().size(1.0, 1.0)),
        steps, decal_textures, decal_factors, flesh, synthetic, shrike,
    });
    commands.insert_resource(FxState { rng: 0x9E37_79B9, particles: 0, lights: 0, decals: Default::default(), holes: Default::default() });
}

fn spawn_light(commands: &mut Commands, def: &'static Glow, at: Vec3) {
    commands.spawn((PointLight { intensity: 0.0, range: 1.0, shadows_enabled: false, ..default() },
                    Transform::from_translation(at),
                    GlowLight { def, ages: if def.rate > 0.0 { vec![] } else { vec![0.0] }, acc: 0.0 }));
}

fn spawn_emitter(commands: &mut Commands, state: &mut FxState, fx: &'static Effect, at: Vec3, axis: Vec3, sp: f32) {
    let acc = state.random();
    commands.spawn(Emitter { fx, at, axis: axis.normalize_or(Vec3::Y), t: 0.0, acc, sp });
}

#[allow(clippy::too_many_arguments)]
fn spawn_decal(commands: &mut Commands, state: &mut FxState, fx: &FxAssets, materials: &mut Assets<StandardMaterial>,
               def: &DecalDef, at: Vec3, scale: f32) {
    spawn_decal_alpha(commands, state, fx, materials, def, at, scale, false);
}

/// `spawn_decal`; `console`: the alpha made to darken as the console's blend does. The Xbox
/// blends a decal into the picture's stored (sRGB) values; Bevy blends in linear light, so a
/// dark decal at alpha a leaves (1 - a)^(1/2.2) of the ground's stored value, not 1 - a. Alpha
/// 1 - (1 - a)^2.2 gives the console's result in the decal's middle. (The Frag's scorch, RGBA
/// 10 10 10 200: 0.47 of the ground in the middle at the data's alpha, the recording's ~0.2-0.3;
/// the blood decals were fitted to captures with the plain alpha and keep it.)
#[allow(clippy::too_many_arguments)]
fn spawn_decal_alpha(commands: &mut Commands, state: &mut FxState, fx: &FxAssets, materials: &mut Assets<StandardMaterial>,
                     def: &DecalDef, at: Vec3, scale: f32, console: bool) {
    if def.textures.is_empty() {
        return;
    }
    let texture = def.textures[(state.random() * def.textures.len() as f32) as usize % def.textures.len()];
    let [r, g, b, a] = def.color;
    let a = if console { 1.0 - (1.0 - a).powf(2.2) } else { a };
    let material = materials.add(StandardMaterial {
        base_color: Color::srgba(r, g, b, a), base_color_texture: fx.decal_textures.get(&texture).cloned(),
        alpha_mode: AlphaMode::Blend, perceptual_roughness: 1.0, reflectance: 0.0, depth_bias: 40.0, ..default()
    });
    let turn = (state.random() * 2.0 - 1.0) * def.rotation.to_radians();
    let y = floor_y(at.x, at.z, at.y + 0.5, world::Body::Thing) + 0.01 + 0.004 * state.random();
    // laid along the ground's slope under its middle (a big scorch on a hillside would otherwise
    // sink into the slope)
    let normal = world::arena().and_then(|a| a.floor_below(at.x, at.z, at.y + 0.5, world::Body::Thing)).map(|f| f.1)
        .filter(|n| n.y > 0.5).unwrap_or(Vec3::Y);
    let e = commands.spawn((Mesh3d(fx.floor.clone()), MeshMaterial3d(material.clone()), NotShadowCaster,
                            Transform::from_translation(Vec3::new(at.x, y, at.z))
                                .with_rotation(Quat::from_rotation_arc(Vec3::Y, normal.normalize()) * Quat::from_rotation_y(turn))
                                .with_scale(Vec3::new(def.width * scale, 1.0, def.height * scale)),
                            Decal { age: 0.0, def: def.clone(), material, multiply: false })).id();
    state.decals.push_back(e);
    while state.decals.len() > MAX_DECALS {
        if let Some(old) = state.decals.pop_front() {
            commands.entity(old).despawn();
        }
    }
}

/// The game's blood decals for one hit on `u` (FUN_00124a70, read in the code map; the takes'
/// breakpoints on the two FUN_00213e60 calls confirmed it, spec §6.2), `skin` the hit point at
/// the skin. a = the amount over the type's damage-multiplier (`Bleed::a`); nothing for a <= 0
/// (take 38: the hits the shield took whole laid none).
///  - FUN_00124af0, the first list (h_fb4bfc68, one at random): when a is at least DECAL_HIT_SHARE
///    of the max hit points or at least the hit points left; on the floor at the middle of the
///    body and the hit point, moved up to DECAL_JITTER m at random, turned at random; scale
///    min(1 + 0.05a, 2), doubled when a is more than the hit points left (every kill; on the
///    62-HP Soldier any hit over 31), alpha 0.5 + 0.5 min(0.05a, 1).
///  - FUN_00124f60, the second list (h_efc2a874): while the victim lives and a is at least
///    DECAL_BEHIND_SHARE of the max: a ray from the hit point along the shot, DECAL_BEHIND_REACH
///    m; on the first surface it meets (take 23: the gate's panel 2.8-3.3 m behind), facing back
///    along it; scale min(0.5 + 0.05a, 1), alpha 0.8 + 0.2 min(0.05a, 1). Nothing if it meets
///    nothing (take 14's first hit: the panel 4.03 m on). The shot's way is taken level (the
///    hit's push: the demo's shots are near level at these ranges); a liquid's hit's is straight
///    up (its record's +0x24, 0x221780: a ceiling within 4 m, or nothing).
/// The decal is multiplied into what's under it (`spawn_game_decal`).
#[allow(clippy::too_many_arguments)]
fn game_decals(commands: &mut Commands, state: &mut FxState, fx: &FxAssets, materials: &mut Assets<StandardMaterial>, game: &Game,
               u: &Player, b: &Bleed, skin: Vec3, lists: [&[u32]; 2]) {
    let a = b.a;
    if a <= 0.0 || b.shielded {
        return;
    }
    let max = u.max_health.max(1.0);
    let log = std::env::var("BF_COMBAT_LOG").is_ok();
    let k = (0.05 * a).min(1.0);
    if a >= DECAL_HIT_SHARE * max || a >= b.hp_left {
        let name = pick(lists[0], state);
        if let Some(def) = game.decals.get(&name) {
            let scale = (1.0 + 0.05 * a).min(2.0) * if a > b.hp_left { 2.0 } else { 1.0 };
            let alpha = 0.5 + 0.5 * k;
            let mid = (u.position + skin) * 0.5;
            let (turn, r) = (state.random() * std::f32::consts::TAU, DECAL_JITTER * state.random());
            let spot = Vec3::new(mid.x + r * turn.cos(), u.position.y + GROUND, mid.z + r * turn.sin());
            if log {
                println!("  decal 1 h_{name:08x} at {spot:.2}: a {a:.1}, scale {scale:.2}, alpha {alpha:.2}");
            }
            spawn_game_decal(commands, state, fx, materials, def, spot, None, scale, alpha);
        }
    }
    let way = if b.burn { Vec3::Y } else { Vec3::new(b.dir.x, 0.0, b.dir.z).normalize_or_zero() };
    if b.hp_left > 0.0 && a >= DECAL_BEHIND_SHARE * max && way != Vec3::ZERO {
        let name = pick(lists[1], state);
        let met = shots::hit_normal(skin, way, DECAL_BEHIND_REACH);
        if let (Some(def), Some((t, normal))) = (game.decals.get(&name), met) {
            let scale = (0.5 + 0.05 * a).min(1.0);
            let alpha = 0.8 + 0.2 * k;
            let at = skin + way * t;
            if log {
                println!("  decal 2 h_{name:08x} at {at:.2} ({t:.2} m behind) normal {normal:.2}: scale {scale:.2}, alpha {alpha:.2}");
            }
            spawn_game_decal(commands, state, fx, materials, def, at, Some(normal), scale, alpha);
        } else if log {
            println!("  decal 2: nothing within {DECAL_BEHIND_REACH} m behind");
        }
    }
}

/// A decal of `game_decals`: `def` at `at`, on the floor there (`normal` None: projected down,
/// along the slope) or on the surface with that normal, `scale` on its width and height read as
/// half sizes (`DECAL_SCALE`: FUN_00217920 builds the UVs as 0.5 + d x 0.5 / extent, so a decal
/// spans 2 x width x scale; a kill's list-1 decal is 4 x 2 m, its visible patch ~1.5 m by the
/// texture's alpha; answers.md Q4), turned at random within its rotation, fading over its life as
/// the others do. Drawn as the game's decal pass draws it (FUN_002175e0: SRCBLEND ZERO, DESTBLEND
/// SRCCOLOR, pixel shader 25; answers.md Q4): what's under it times
/// lerp(1, min(1, tint + 1 - the texture's alpha), a), a = the tint's alpha x `alpha` x the fade;
/// the texture's colour unused. That's `decal_factor`'s texture (the min(..) per texel) through
/// Bevy's `AlphaMode::Multiply` (dst x (1 - a + a x colour)), unlit; the console multiplies stored
/// (gamma) values, Bevy linear ones, the same where a is 1. Take 20's kill at 18.0 s: a floor of
/// 35, 28, 36 to 15, 23, 17 (this gives 19, 22, 20). The defs' h_071e88fb / h_17dd78a4 (132 on the
/// Shrikes') are stencil references, of no use in the normal view (answers.md Q4). The pass's
/// shader lerps the factor to 1 by the fog, so a decal fades to no effect in the fog: Bevy's fog
/// is off on it (fogged, the factor's white would take the fog's colour and the whole quad show)
/// and `animate_fx` scales its alpha by 1 - the fog at it instead.
#[allow(clippy::too_many_arguments)]
fn spawn_game_decal(commands: &mut Commands, state: &mut FxState, fx: &FxAssets, materials: &mut Assets<StandardMaterial>,
                    def: &DecalDef, at: Vec3, normal: Option<Vec3>, scale: f32, alpha: f32) {
    if def.textures.is_empty() {
        return;
    }
    let texture = def.textures[(state.random() * def.textures.len() as f32) as usize % def.textures.len()];
    let mut def = def.clone();
    def.color[3] = (def.color[3] * alpha).clamp(0.0, 1.0);
    let a = def.color[3];
    let material = materials.add(StandardMaterial {
        base_color: Color::srgba(1.0, 1.0, 1.0, a), base_color_texture: fx.decal_factors.get(&(texture, tint_key(&def))).cloned(),
        alpha_mode: AlphaMode::Multiply, unlit: true, fog_enabled: false, depth_bias: 40.0, ..default()
    });
    let turn = (state.random() * 2.0 - 1.0) * def.rotation.to_radians();
    let lift = 0.01 + 0.004 * state.random();
    let (at, normal) = match normal {
        Some(n) => (at + n.normalize_or(Vec3::Y) * lift, n.normalize_or(Vec3::Y)),
        None => {
            let y = floor_y(at.x, at.z, at.y + 0.5, world::Body::Thing) + lift;
            let n = world::arena().and_then(|a| a.floor_below(at.x, at.z, at.y + 0.5, world::Body::Thing)).map(|f| f.1).filter(|n| n.y > 0.5).unwrap_or(Vec3::Y);
            (Vec3::new(at.x, y, at.z), n.normalize())
        }
    };
    let size = Vec3::new(def.width * scale * DECAL_SCALE, 1.0, def.height * scale * DECAL_SCALE);
    let e = commands.spawn((Mesh3d(fx.floor.clone()), MeshMaterial3d(material.clone()), NotShadowCaster,
                            Transform::from_translation(at).with_rotation(Quat::from_rotation_arc(Vec3::Y, normal) * Quat::from_rotation_y(turn))
                                .with_scale(size),
                            Decal { age: 0.0, def, material, multiply: true })).id();
    state.decals.push_back(e);
    while state.decals.len() > MAX_DECALS {
        if let Some(old) = state.decals.pop_front() {
            commands.entity(old).despawn();
        }
    }
}

/// A decal def's tint as bytes (its `h_1c2f0cfb` RGB), the key of its factor textures.
fn tint_key(def: &DecalDef) -> [u8; 3] {
    [0, 1, 2].map(|k| (def.color[k].clamp(0.0, 1.0) * 255.0).round() as u8)
}

/// The colour the game's decal pass multiplies the picture by where a decal's alpha is 1, per
/// texel of decal texture `rgba` under `tint` (pixel shader 25's first combiner stage,
/// answers.md Q4): min(1, tint + 1 - the texel's alpha), its colour unused; alpha 1 (the decal's
/// own alpha is the material's).
fn decal_factor(rgba: &[u8], tint: [u8; 3]) -> Vec<u8> {
    rgba.chunks_exact(4).flat_map(|p| {
        let open = 255 - p[3] as u16;
        [(tint[0] as u16 + open).min(255) as u8, (tint[1] as u16 + open).min(255) as u8, (tint[2] as u16 + open).min(255) as u8, 255]
    }).collect()
}

/// A character's blood decals (`<decal>` names): its type's two lists (#128: the hit's
/// h_fb4bfc68, up to three, and the pool's h_efc2a874), none without a type. The squad's lists
/// hold one each, the ones their names gave before (BF_DUMP_TYPES compares them); the Shrikes'
/// hit list two.
fn blood_decals<'a>(game: &'a Game, u: &Player) -> [&'a [u32]; 2] {
    game.character_type(u.kind).map_or([&[], &[]], |t| [&t.decals[0], &t.decals[1]])
}

/// One decal of a list: at random, as the game picks one per hit (FUN_00124af0: floor(U x
/// count)); a list of one takes no draw (the effects' random stream stays as it was). 0 for none.
fn pick(list: &[u32], state: &mut FxState) -> u32 {
    match list.len() {
        0 => 0,
        1 => list[0],
        n => list[((state.random() * n as f32) as usize).min(n - 1)],
    }
}

/// New hits (blood, sparks, the hit sound, a decal) and, for the dead, the pool.
#[allow(clippy::too_many_arguments)]
fn spawn_fx(
    mut commands: Commands,
    time: Res<Time>,
    mut player: ResMut<Player>,
    mut squad: ResMut<Squad>,
    mut game: ResMut<GameData>,
    fx: Option<Res<FxAssets>>,
    mut state: ResMut<FxState>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
    ale: Option<ResMut<bf_viewer::ale_fx::AleAssets>>,
) {
    let Some(fx) = fx else { return };
    let dt = frame_dt(&time);
    // the flesh hit effects played from the data, once the characters are done with (effect
    // type, where, the shot's way, the struck surface's normal, user parameter, seed)
    let mut from_data: Vec<(u32, Vec3, Vec3, Vec3, f32, u32)> = vec![];
    for u in std::iter::once(&mut *player).chain(squad.0.iter_mut()) {
        let [hit_decals, pool_decals] = blood_decals(&game.0, u);
        // what they're made of (their parts' world material): its hit sounds and effects, and
        // its debris (grey for Flint, green for the Shrikes)
        let flesh = u.loaded.as_ref().map_or(FLESH, |l| l.flesh);
        let (surface, blood) = match flesh {
            SYNTHETIC => (fx.synthetic.as_ref(), &BLOOD_B),
            SHRIKE => (fx.shrike.as_ref().or(fx.flesh.as_ref()), &BLOOD_1),
            _ => (fx.flesh.as_ref(), &BLOOD),
        };
        // the game's decal rule (FUN_00124a70), for Shrike flesh (#129; the rest is #150's)
        let game_rule = flesh == SHRIKE;
        // (killed by a liquid's hit this frame: `burn` bleeds)
        let mut burned = false;
        for b in std::mem::take(&mut u.blood) {
            // a liquid's hit (play.rs `burn`): FUN_00124a70's decals alone, on any flesh
            if b.burn {
                burned |= b.hp_left <= 0.0;
                game_decals(&mut commands, &mut state, &fx, &mut materials, &game.0, u, &b, b.at, [hit_decals, pool_decals]);
                continue;
            }
            let (at, dir, ammo) = (b.at, b.dir, b.ammo);
            // the effect's axis: the hit's normal (back toward the shooter)
            let back = -Vec3::new(dir.x, 0.0, dir.z).normalize_or(Vec3::Z);
            // at the skin: shots stop on the hit cylinder, which is wider than a body; blasts hit
            // the middle
            let at = at + back * if ammo >= 0 { -SKIN_IN } else { SKIN_OUT };
            let sp = state.random();
            // (a hit a stamina shield took whole: no blood, play_enemies.rs)
            if !b.shielded {
                for e in blood {
                    spawn_emitter(&mut commands, &mut state, e, at, back, sp);
                }
            }
            let (effect, light) = hit_effect(surface, ammo);
            if let Some(e) = effect {
                spawn_emitter(&mut commands, &mut state, e, at, back, sp);
            }
            if let Some(g) = light {
                if state.lights < MAX_LIGHTS {
                    state.lights += 1;
                    // (where its effect type's light-effect puts it: `Glow::back` along the
                    // shot's way back toward the shooter, alignment 0)
                    spawn_light(&mut commands, g, at - dir.normalize_or(-back) * g.back);
                }
            }
            // a row's effect type the demo hasn't made by hand: from the data, with its light
            // (the sonic, particle and bio flesh hits, #57 #70 #47), aligned by its type: the
            // shot's way (0), or the body's surface there (2: out from the hit cylinder's axis,
            // the demo's stand-in for the struck part's collision)
            let row = surface.and_then(|s| s.hit_effect.get(&ammo)).copied();
            if let (None, Some(kind)) = (effect, row.filter(|&k| k != 0)) {
                let normal = Vec3::new(b.at.x - u.position.x, 0.0, b.at.z - u.position.z).normalize_or(back);
                from_data.push((kind, at, -back, normal, flesh_sp(&b), state.rng));
            }
            if let Some(sounds) = surface.and_then(|s| s.hit_sounds.get(&ammo)) {
                let sounds: Vec<u32> = sounds.iter().copied().filter(|&id| game.0.sounds.has(id)).collect();
                if !sounds.is_empty() {
                    let k = u.random(sounds.len());
                    u.sound_queue.push((sounds[k], 1.0));
                }
            }
            if game_rule {
                game_decals(&mut commands, &mut state, &fx, &mut materials, &game.0, u, &b, at, [hit_decals, pool_decals]);
            } else if let Some(def) = game.0.decals.get(&pick(hit_decals, &mut state)) {
                // a splat on the ground past the hit
                let past = at - back * (SPLAT_PAST.0 + (SPLAT_PAST.1 - SPLAT_PAST.0) * state.random());
                spawn_decal(&mut commands, &mut state, &fx, &mut materials, def, past, DECAL_SCALE);
            }
        }
        if !u.dead {
            u.dead_for = 0.0;
            continue;
        }
        // (the game's rule has nothing more at a death: the killing hit's own decal, doubled, and
        // its blood bundle are all, takes 17-22)
        if game_rule {
            u.dead_for += dt;
            continue;
        }
        // the moment they die: blood splashed round where they fall, a spray and splats on the
        // ground (DEATH_SPLATS, within DEATH_SPLAT_REACH m); not for a death by a liquid (#146's
        // take01: none as Brutus dies in the lava)
        if u.dead_for == 0.0 && !burned {
            let at = u.position + Vec3::Y * (GROUND + DEATH_SPRAY_UP);
            let sp = state.random();
            for e in blood {
                spawn_emitter(&mut commands, &mut state, e, at, Vec3::Y, sp);
            }
            for _ in 0..DEATH_SPLATS {
                if let Some(def) = game.0.decals.get(&pick(hit_decals, &mut state)) {
                    let (a, r) = (state.random() * std::f32::consts::TAU, DEATH_SPLAT_REACH * state.random().sqrt());
                    let spot = Vec3::new(u.position.x + r * a.cos(), u.position.y + GROUND, u.position.z + r * a.sin());
                    spawn_decal(&mut commands, &mut state, &fx, &mut materials, def, spot, DECAL_SCALE);
                }
            }
        }
        u.dead_for += dt;
        // (the memory chip beside them: play_dna.rs, from dead_for)
        // the pool under the body, once it's down and still
        if !u.pool_done && u.thud && u.dead_for >= POOL_DELAY {
            let pool_decal = pick(pool_decals, &mut state);
            if let (Some(body), Some(def)) = (u.body_at, game.0.decals.get(&pool_decal)) {
                u.pool_done = true;
                if std::env::var("BF_COMBAT_LOG").is_ok() {
                    println!("pool h_{pool_decal:08x} under {} at {body:.2}: {def:?}", unit_name(u));
                }
                spawn_decal(&mut commands, &mut state, &fx, &mut materials, def, body, DECAL_SCALE);
            }
        }
    }
    if let Some(mut ale) = ale {
        for (kind, at, way, normal, sp, seed) in from_data {
            if std::env::var("BF_COMBAT_LOG").is_ok() {
                println!("  flesh hit effect h_{kind:08x} at {at:.2} sp {sp:.2} light h_{:08x}", game.0.effect_type_defs.get(&kind).map_or(0, |t| t.light));
            }
            super::hit_effect_at(&mut commands, &mut game.0, &mut ale, &mut images, &mut materials, kind, at, way, normal, sp, seed);
        }
    }
}

/// Emitters make particles over their time.
fn emit(mut commands: Commands, time: Res<Time>, fx: Option<Res<FxAssets>>, mut state: ResMut<FxState>,
        mut emitters: Query<(Entity, &mut Emitter)>) {
    let Some(fx) = fx else { return };
    let dt = frame_dt(&time);
    for (e, mut em) in &mut emitters {
        let d = &em.fx.emit;
        if em.t >= d.time {
            commands.entity(e).despawn();
            continue;
        }
        em.acc += family(d.rate, LINEAR, em.sp, em.t).max(0.0) * dt;
        while em.acc >= 1.0 {
            em.acc -= 1.0;
            if state.particles >= MAX_PARTICLES {
                continue;
            }
            let (t, sp, axis) = (em.t, em.sp, em.axis);
            let life = at(d.life, LINEAR, t).max(0.02);
            let speed = family(d.speed, LINEAR, sp, t);
            let (s0, s1) = (at(d.spread_min, LINEAR, t), at(d.spread_max, LINEAR, t));
            let (lo, hi) = (s0.min(s1), s0.max(s1));
            let theta = (lo + (hi - lo) * state.random()).to_radians();
            let phi = state.random() * std::f32::consts::TAU;
            let (u, v) = axis.any_orthonormal_pair();
            let dir = axis * theta.cos() + (u * phi.cos() + v * phi.sin()) * theta.sin();
            let box_at = Vec3::new(state.random() - 0.5, state.random() - 0.5, state.random() - 0.5) * d.size;
            let roll = state.random() * std::f32::consts::TAU;
            state.particles += 1;
            commands.spawn((Mesh3d(fx.quad.clone()), MeshMaterial3d(fx.steps[em.fx.id][0].clone()), NotShadowCaster,
                            Transform::from_translation(em.at + box_at).with_scale(Vec3::splat(0.001)),
                            Particle { fx: em.fx, age: 0.0, life, vel: dir * speed, sp, roll, step: 0 }));
        }
        em.t += dt;
    }
}

/// How much of the view's colour fog `fog` lays over a point `distance` m from the camera (0 none,
/// 1 all of it), as Bevy's fog shader works it out for each falloff (the demo's levels use the
/// linear one: their start- and end-distance).
fn fog_at(fog: &DistanceFog, distance: f32) -> f32 {
    match fog.falloff {
        FogFalloff::Linear { start, end } => ((distance - start) / (end - start).max(1e-3)).clamp(0.0, 1.0),
        FogFalloff::Exponential { density } => 1.0 - (-density * distance).exp(),
        FogFalloff::ExponentialSquared { density } => 1.0 - (-(density * distance).powi(2)).exp(),
        FogFalloff::Atmospheric { extinction, .. } => 1.0 - (-extinction * distance).exp().max_element(),
    }
}

/// Particles move, grow, change colour and face the camera; decals fade at the end of their
/// time (the game's multiply ones also with the fog); a light effect's light follows its
/// particles.
#[allow(clippy::too_many_arguments)]
fn animate_fx(
    mut commands: Commands,
    time: Res<Time>,
    fx: Option<Res<FxAssets>>,
    mut state: ResMut<FxState>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    camera: Query<(&Transform, Option<&DistanceFog>), (With<MainCamera>, Without<Particle>)>,
    mut particles: Query<(Entity, &mut Particle, &mut Transform, &mut MeshMaterial3d<StandardMaterial>)>,
    mut decals: Query<(Entity, &mut Decal, &Transform), Without<Particle>>,
    mut lights: Query<(Entity, &mut GlowLight, &mut PointLight)>,
) {
    let Some(fx) = fx else { return };
    let dt = frame_dt(&time);
    let view = camera.single().ok();
    let facing = view.map(|(c, _)| c.rotation).unwrap_or_default();
    let to_camera = facing.inverse();
    for (e, mut p, mut tr, mut mat) in &mut particles {
        p.age += dt;
        if p.age >= p.life {
            commands.entity(e).despawn();
            state.particles = state.particles.saturating_sub(1);
            continue;
        }
        let look = &p.fx.look;
        let k = p.age / p.life;
        p.vel.y -= p.fx.emit.gravity * dt;
        tr.translation += p.vel * dt;
        let size = family(look.size, look.size_ease, p.sp, k).max(0.0);
        let width = (size * at(look.width, EASE_IN_OUT, k)).max(0.001);
        if look.streak {
            let v = to_camera * p.vel;
            let turn = v.y.atan2(v.x) - std::f32::consts::FRAC_PI_2;
            tr.rotation = facing * Quat::from_rotation_z(turn);
            tr.scale = Vec3::new(width, (size * look.aspect + v.truncate().length() * STREAK_TIME).max(0.001), 1.0);
        } else {
            tr.rotation = facing * Quat::from_rotation_z(p.roll + at(look.spin, LINEAR, k));
            tr.scale = Vec3::new(width, (size * look.aspect).max(0.001), 1.0);
        }
        let step = ((k * STEPS as f32) as usize).min(STEPS - 1);
        if step != p.step {
            p.step = step;
            mat.0 = fx.steps[p.fx.id][step].clone();
        }
    }
    for (e, mut d, tr) in &mut decals {
        d.age += dt;
        let left = d.def.life - d.age;
        if left <= 0.0 {
            commands.entity(e).despawn();
            state.decals.retain(|&x| x != e);
            state.holes.retain(|&x| x != e);
            continue;
        }
        let fade = (left / d.def.fade.max(0.01)).min(1.0);
        if d.multiply {
            // the game's decal pass: to no effect with the fog at it (`spawn_game_decal`)
            let fog = view.and_then(|(c, f)| f.map(|f| fog_at(f, c.translation.distance(tr.translation)))).unwrap_or(0.0);
            let want = d.def.color[3] * fade * (1.0 - fog);
            if materials.get(&d.material).is_some_and(|m| (m.base_color.alpha() - want).abs() > 1e-3) {
                if let Some(m) = materials.get_mut(&d.material) {
                    m.base_color.set_alpha(want);
                }
            }
        } else if left < d.def.fade {
            if let Some(m) = materials.get_mut(&d.material) {
                m.base_color.set_alpha(d.def.color[3] * fade);
            }
        }
    }
    for (e, mut l, mut light) in &mut lights {
        let def = l.def;
        l.acc += def.rate * dt;
        while l.acc >= 1.0 {
            l.acc -= 1.0;
            l.ages.push(0.0);
        }
        for a in l.ages.iter_mut() {
            *a += dt;
        }
        l.ages.retain(|&a| a < def.life);
        if def.rate <= 0.0 && l.ages.is_empty() {
            commands.entity(e).despawn();
            state.lights = state.lights.saturating_sub(1);
            continue;
        }
        // the particles' colour x alpha x reach, summed: the light's colour and strength
        let (mut sum, mut range) = (Vec3::ZERO, 0.5f32);
        for &a in &l.ages {
            let k = a / def.life;
            let size = at(def.size, EASE_IN_OUT, k);
            sum += Vec3::from(color_at(def.color, k)) * def.alpha * size;
            range = range.max(size);
        }
        let peak = sum.max_element();
        light.intensity = LIGHT_LUMENS * peak * bf_viewer::level_scene::POINT_LIGHT_SCALE;
        if peak > 1e-4 {
            light.color = Color::srgb(sum.x / peak, sum.y / peak, sum.z / peak);
        }
        light.range = range;
    }
}
