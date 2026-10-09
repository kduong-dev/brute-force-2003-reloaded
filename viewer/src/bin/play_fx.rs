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
//!  - DNA: beside a fallen squadmate, effect h_ee11d51f = powerup_pill (green small-flare.tga
//!    sprites growing out of a small cube emitter, added to the picture) + light_powerup_pill, a
//!    "light_" node: those drive a point light rather than sprites (the light is what brightens
//!    the ground around it in the capture). The capture shows the sprites as a soft square (the
//!    "green block"): the data's texture is the round flare, so the square is the emulator's way
//!    of drawing them (the explosion sprites in the captures are boxy too).
//!
//! The parameters read as Freelancer's ALE ones (Brute Force hashes their names): an emitter
//! emits `rate` particles/s over its own time, each living `life` s, leaving at `speed` m/s at an
//! angle within `spread` degrees of the emitter's axis (the hit's normal); appearance keys run
//! over a particle's life (0-1). Some curves have items for several values of the effect's user
//! parameter ("sparam": e.g. 26 droplets at the low end, 74 at the top); what the game passes for
//! a hit isn't known, so each hit picks one at random. A particle's size is its full width.

use super::*;
use bf_viewer::bf::audio::Surface;
use bf_viewer::bf::character::DecalDef;

/// keys (time, value)
type Keys = &'static [(f32, f32)];
/// keys per sparam value
type Family = &'static [(f32, Keys)];

const LINEAR: u8 = 1;
const EASE_IN: u8 = 2;
const EASE_IN_OUT: u8 = 4;

/// The world material characters are made of (WMAT_FLESH_HUMAN).
const FLESH: i64 = 21;
/// Flint's world material: WMAT_FLESH_SYNTHETIC (24 in the XBE's WMAT_ name table, counted from
/// WMAT_DEFAULT = 0). An inference: the data doesn't name a character's material, but Flint's
/// decals are grey (h_02d017b7, h_0d7a5245: RGBA 50,50,50) and 24 is the flesh whose debris
/// (h_1f7ca034: blood_puff_b, bloodsplat_b, giblet_b) is grey.
const SYNTHETIC: i64 = 24;
/// The character made of SYNTHETIC.
const SYNTHETIC_CHARACTER: &str = "flint";
/// Particle colour / alpha steps (one shared material each).
const STEPS: usize = 32;
/// Streaked particles (motion blur) are drawn this many seconds of motion long.
const STREAK_TIME: f32 = 0.035;
const MAX_PARTICLES: usize = 2500;
const MAX_DECALS: usize = 80;
/// A decal's width / height read as half sizes: the capture's hit splats are about 1 m across
/// (h_14410af1 says 0.5 x 0.25), landing 1-3 m past the one hit, along the shot.
const DECAL_SCALE: f32 = 2.0;
const SPLAT_PAST: (f32, f32) = (1.0, 2.5);
/// How far inside the hit cylinder (radius 0.4 m) a body's skin is, and how far out of its middle.
const SKIN_IN: f32 = 0.25;
const SKIN_OUT: f32 = 0.15;
/// The DNA appears this long after a death, this far beside where they fell and this high
/// (capture).
const DNA_DELAY: f32 = 0.2;
const DNA_SIDE: f32 = 0.9;
const DNA_HEIGHT: f32 = 0.55;
/// The pool goes under the body this long after death (it has stopped sliding by then).
const POOL_DELAY: f32 = 2.0;
/// A death splashes blood round where the body falls: the hit's blood effects sprayed up from
/// DEATH_SPRAY_UP m, and DEATH_SPLATS of the character's hit splats on the ground within
/// DEATH_SPLAT_REACH m. The demo's choice: the game's own death blood isn't known.
const DEATH_SPRAY_UP: f32 = 0.9;
const DEATH_SPLATS: usize = 5;
const DEATH_SPLAT_REACH: f32 = 1.4;
/// A light's lumens per unit of (alpha x size x colour) of its live particles (tuned to the
/// capture's brightening around the DNA).
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
/// reach (m), colour x alpha its strength. `rate` 0: one particle, at the start.
struct Glow {
    rate: f32,
    life: f32,
    color: &'static [(f32, [f32; 3])],
    alpha: f32,
    size: Keys,
}

/// light_powerup_pill (the DNA's): blue, reaching 0.4 -> 2 -> 0.4 m over each particle's life
static DNA_LIGHT: Glow = Glow {
    rate: 1.492, life: 2.424, color: &[(0.0, [0.227, 0.475, 0.773])], alpha: 0.583,
    size: &[(0.00322, 0.408), (0.463, 2.0), (0.994, 0.405)],
};

/// light_hit_cutter: a pale blue flash
static CUTTER_LIGHT: Glow = Glow {
    rate: 0.0, life: 0.3866, color: &[(0.0, [0.545, 0.796, 0.871]), (1.0, [0.0, 0.0, 0.0])], alpha: 0.583,
    size: &[(0.0, 1.28), (0.434, 4.04)],
};

/// light_hit_laser: an orange flash
static LASER_LIGHT: Glow = Glow {
    rate: 0.0, life: 0.3866, color: &[(0.0, [0.886, 0.294, 0.0941]), (1.0, [0.0, 0.0, 0.0])], alpha: 0.583,
    size: &[(0.0, 1.01), (1.0, 3.98)],
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
    id: 5,
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
    id: 6,
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

/// powerup_pill (DNA): powerup_Cube_pill.emt (runs until removed) / powerup_pill.app
static DNA_PILL: Effect = Effect {
    id: 4,
    emit: Emit {
        time: f32::INFINITY,
        rate: &[(0.0, &[(0.0, 7.356)])],
        life: &[(0.0, 1.479)],
        speed: &[(0.0, &[(0.0, 0.03713)])],
        size: 0.0893,
        spread_min: &[(0.0, 6.72)],
        spread_max: &[(0.0, 28.21)],
        gravity: 0.0,
    },
    look: Look {
        texture: 0x1E87_F03A,   // small-flare.tga
        additive: true,
        color: &[(0.0, [0.224, 0.525, 0.243])],
        alpha: &[(0.0, 0.0), (0.626, 0.467), (1.0, 0.00619)],
        size: &[(0.0, &[(0.0, 0.101), (1.0, 1.57)])],
        size_ease: EASE_IN,
        aspect: 1.0,
        width: &[(0.0, 1.0)],
        streak: false,
        spin: &[(0.0, 0.0)],
    },
};

/// blood_puff_b (synthetic flesh): blood_mist_b.emt / blood_mist_b.app. Grey, slower and
/// fewer than blood_mist.
static BLOOD_MIST_B: Effect = Effect {
    id: 7,
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
    id: 8,
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
    id: 9,
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

static EFFECTS: [&Effect; 10] = [&BLOOD_MIST, &BLOOD_DROPS, &GIBLET, &FLESH_SPARKS, &DNA_PILL, &FLESH_CUTTER, &FLESH_LASER,
                                 &BLOOD_MIST_B, &BLOOD_DROPS_B, &GIBLET_B];

/// The debris bundle every flesh hit gets (h_f304f9fb).
static BLOOD: [&Effect; 3] = [&BLOOD_DROPS, &GIBLET, &BLOOD_MIST];
/// Synthetic flesh's debris bundle (h_1f7ca034).
static BLOOD_B: [&Effect; 3] = [&BLOOD_DROPS_B, &GIBLET_B, &BLOOD_MIST_B];

/// A flesh material's hit effect for an ammo type: the squad's three (ballistic, cutter, laser)
/// and their lights.
fn hit_effect(surface: Option<&Surface>, ammo: i64) -> (Option<&'static Effect>, Option<&'static Glow>) {
    match surface.and_then(|s| s.hit_effect.get(&ammo)) {
        Some(0x1D25_07AB) => (Some(&FLESH_SPARKS), None),       // wmat_fleshit_standard
        Some(0xF72C_8C11) => (Some(&FLESH_CUTTER), Some(&CUTTER_LIGHT)),
        Some(0xFCBB_F2B9) => (Some(&FLESH_LASER), Some(&LASER_LIGHT)),
        _ => (None, None),
    }
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
    flesh: Option<Surface>,
    /// SYNTHETIC's entry: Flint's hit sounds and effects
    synthetic: Option<Surface>,
}

#[derive(Resource)]
struct FxState {
    rng: u32,
    particles: usize,
    /// hit lights alive
    lights: usize,
    decals: std::collections::VecDeque<Entity>,
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

#[derive(Component)]
struct Decal {
    age: f32,
    def: DecalDef,
    material: Handle<StandardMaterial>,
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

pub fn plugin(app: &mut App) {
    app.init_resource::<DecalRequests>()
        .add_systems(OnEnter(AppState::Playing), setup_fx.after(snapshot_entities))
        .add_systems(Update, (spawn_fx, requested_decals, emit, animate_fx).chain().after(update_player).before(play_sounds)
            .run_if(in_state(AppState::Playing)));
}

/// The decals asked for (`DecalRequests`), their textures loaded the first time, at the scale
/// asked for.
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
                if let Some((w, h, px)) = game.0.texture_rgba(id) {
                    fx.decal_textures.insert(id, images.add(rgba(w, h, px, false)));
                }
            }
        }
        spawn_decal(&mut commands, &mut state, &fx, &mut materials, &def, at, scale);
    }
}

/// `linear`: the texels as they are (for sprites added to the picture, see `setup_fx`).
fn rgba(w: u32, h: u32, px: Vec<u8>, linear: bool) -> Image {
    use bevy::render::render_resource::TextureFormat;
    Image::new(bevy::render::render_resource::Extent3d { width: w, height: h, depth_or_array_layers: 1 },
               bevy::render::render_resource::TextureDimension::D2, px,
               if linear { TextureFormat::Rgba8Unorm } else { TextureFormat::Rgba8UnormSrgb },
               bevy::asset::RenderAssetUsages::default())
}

fn setup_fx(mut commands: Commands, mut game: ResMut<GameData>, mut meshes: ResMut<Assets<Mesh>>,
            mut images: ResMut<Assets<Image>>, mut materials: ResMut<Assets<StandardMaterial>>) {
    // the squad's blood decals' textures
    let decal_ids: Vec<u32> = game.0.character_decals.values().flat_map(|&(a, b)| [a, b])
        .flat_map(|name| game.0.decals.get(&name).map(|d| d.textures.clone()).unwrap_or_default()).collect();
    let flesh = game.0.surfaces.iter().find(|s| s.id == FLESH).cloned();
    let synthetic = game.0.surfaces.iter().find(|s| s.id == SYNTHETIC).cloned();
    let mut tex = |id: u32, linear: bool| game.0.texture_rgba(id).map(|(w, h, px)| images.add(rgba(w, h, px, linear)));
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
    if let Some(f) = &flesh {
        info!("flesh (world material {FLESH}): hit effects {:x?}, debris effect {:x?}", f.hit_effect, f.debris_effect);
    }
    commands.insert_resource(FxAssets {
        quad: meshes.add(Rectangle::new(1.0, 1.0)),
        floor: meshes.add(Plane3d::default().mesh().size(1.0, 1.0)),
        steps, decal_textures, flesh, synthetic,
    });
    commands.insert_resource(FxState { rng: 0x9E37_79B9, particles: 0, lights: 0, decals: Default::default() });
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
    if def.textures.is_empty() {
        return;
    }
    let texture = def.textures[(state.random() * def.textures.len() as f32) as usize % def.textures.len()];
    let [r, g, b, a] = def.color;
    let material = materials.add(StandardMaterial {
        base_color: Color::srgba(r, g, b, a), base_color_texture: fx.decal_textures.get(&texture).cloned(),
        alpha_mode: AlphaMode::Blend, perceptual_roughness: 1.0, reflectance: 0.0, depth_bias: 40.0, ..default()
    });
    let turn = (state.random() * 2.0 - 1.0) * def.rotation.to_radians();
    let y = floor_y(at.x, at.z, at.y + 0.5) + 0.01 + 0.004 * state.random();
    // laid along the ground's slope under its middle (a big scorch on a hillside would otherwise
    // sink into the slope)
    let normal = world::arena().and_then(|a| a.floor_at(at.x, at.z, at.y + 0.5)).map(|f| f.1)
        .filter(|n| n.y > 0.5).unwrap_or(Vec3::Y);
    let e = commands.spawn((Mesh3d(fx.floor.clone()), MeshMaterial3d(material.clone()), NotShadowCaster,
                            Transform::from_translation(Vec3::new(at.x, y, at.z))
                                .with_rotation(Quat::from_rotation_arc(Vec3::Y, normal.normalize()) * Quat::from_rotation_y(turn))
                                .with_scale(Vec3::new(def.width * scale, 1.0, def.height * scale)),
                            Decal { age: 0.0, def: def.clone(), material })).id();
    state.decals.push_back(e);
    while state.decals.len() > MAX_DECALS {
        if let Some(old) = state.decals.pop_front() {
            commands.entity(old).despawn();
        }
    }
}

/// New hits (blood, sparks, the hit sound, a decal) and, for the dead, the DNA and the pool.
#[allow(clippy::too_many_arguments)]
fn spawn_fx(
    mut commands: Commands,
    time: Res<Time>,
    mut player: ResMut<Player>,
    mut squad: ResMut<Squad>,
    game: Res<GameData>,
    fx: Option<Res<FxAssets>>,
    mut state: ResMut<FxState>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let Some(fx) = fx else { return };
    let dt = frame_dt(&time);
    for u in std::iter::once(&mut *player).chain(squad.0.iter_mut()) {
        let (hit_decal, pool_decal) = game.0.character_decals.get(CHARACTERS[u.character]).copied().unwrap_or((0, 0));
        // what they're made of: its hit sounds and effects, and its debris (grey for Flint)
        let (surface, blood) = if CHARACTERS[u.character] == SYNTHETIC_CHARACTER {
            (fx.synthetic.as_ref(), &BLOOD_B)
        } else {
            (fx.flesh.as_ref(), &BLOOD)
        };
        for (at, dir, ammo) in std::mem::take(&mut u.blood) {
            // the effect's axis: the hit's normal (back toward the shooter)
            let back = -Vec3::new(dir.x, 0.0, dir.z).normalize_or(Vec3::Z);
            // at the skin: shots stop on the hit cylinder, which is wider than a body; blasts hit
            // the middle
            let at = at + back * if ammo >= 0 { -SKIN_IN } else { SKIN_OUT };
            let sp = state.random();
            for e in blood {
                spawn_emitter(&mut commands, &mut state, e, at, back, sp);
            }
            let (effect, light) = hit_effect(surface, ammo);
            if let Some(e) = effect {
                spawn_emitter(&mut commands, &mut state, e, at, back, sp);
            }
            if let Some(g) = light {
                if state.lights < MAX_LIGHTS {
                    state.lights += 1;
                    spawn_light(&mut commands, g, at + back * 0.3);
                }
            }
            if let Some(sounds) = surface.and_then(|s| s.hit_sounds.get(&ammo)) {
                let sounds: Vec<u32> = sounds.iter().copied().filter(|&id| game.0.sounds.has(id)).collect();
                if !sounds.is_empty() {
                    let k = u.random(sounds.len());
                    u.sound_queue.push((sounds[k], 1.0));
                }
            }
            // a splat on the ground past the hit
            if let Some(def) = game.0.decals.get(&hit_decal) {
                let past = at - back * (SPLAT_PAST.0 + (SPLAT_PAST.1 - SPLAT_PAST.0) * state.random());
                spawn_decal(&mut commands, &mut state, &fx, &mut materials, def, past, DECAL_SCALE);
            }
        }
        if !u.dead {
            u.dead_for = 0.0;
            continue;
        }
        // the moment they die: blood splashed round where they fall, a spray and splats on the
        // ground (DEATH_SPLATS, within DEATH_SPLAT_REACH m)
        if u.dead_for == 0.0 {
            let at = u.position + Vec3::Y * (GROUND + DEATH_SPRAY_UP);
            let sp = state.random();
            for e in blood {
                spawn_emitter(&mut commands, &mut state, e, at, Vec3::Y, sp);
            }
            if let Some(def) = game.0.decals.get(&hit_decal) {
                for _ in 0..DEATH_SPLATS {
                    let (a, r) = (state.random() * std::f32::consts::TAU, DEATH_SPLAT_REACH * state.random().sqrt());
                    let spot = Vec3::new(u.position.x + r * a.cos(), u.position.y + GROUND, u.position.z + r * a.sin());
                    spawn_decal(&mut commands, &mut state, &fx, &mut materials, def, spot, DECAL_SCALE);
                }
            }
        }
        u.dead_for += dt;
        // the DNA, beside where they fell
        if !u.dna_done && u.dead_for >= DNA_DELAY {
            u.dna_done = true;
            let a = state.random() * std::f32::consts::TAU;
            let (x, z) = (u.position.x + DNA_SIDE * a.cos(), u.position.z + DNA_SIDE * a.sin());
            let pos = Vec3::new(x, floor_y(x, z, u.position.y + GROUND + 1.0) + DNA_HEIGHT, z);
            spawn_emitter(&mut commands, &mut state, &DNA_PILL, pos, Vec3::Y, 0.0);
            spawn_light(&mut commands, &DNA_LIGHT, pos);
        }
        // the pool under the body, once it's down and still
        if !u.pool_done && u.thud && u.dead_for >= POOL_DELAY {
            if let (Some(body), Some(def)) = (u.body_at, game.0.decals.get(&pool_decal)) {
                u.pool_done = true;
                if std::env::var("BF_COMBAT_LOG").is_ok() {
                    println!("pool h_{pool_decal:08x} under {} at {body:.2}: {def:?}", CHARACTERS[u.character]);
                }
                spawn_decal(&mut commands, &mut state, &fx, &mut materials, def, body, DECAL_SCALE);
            }
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

/// Particles move, grow, change colour and face the camera; decals fade at the end of their
/// time; the DNA light follows its particles.
#[allow(clippy::too_many_arguments)]
fn animate_fx(
    mut commands: Commands,
    time: Res<Time>,
    fx: Option<Res<FxAssets>>,
    mut state: ResMut<FxState>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    camera: Query<&Transform, (With<MainCamera>, Without<Particle>)>,
    mut particles: Query<(Entity, &mut Particle, &mut Transform, &mut MeshMaterial3d<StandardMaterial>)>,
    mut decals: Query<(Entity, &mut Decal)>,
    mut lights: Query<(Entity, &mut GlowLight, &mut PointLight)>,
) {
    let Some(fx) = fx else { return };
    let dt = frame_dt(&time);
    let facing = camera.single().map(|c| c.rotation).unwrap_or_default();
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
    for (e, mut d) in &mut decals {
        d.age += dt;
        let left = d.def.life - d.age;
        if left <= 0.0 {
            commands.entity(e).despawn();
            state.decals.retain(|&x| x != e);
            continue;
        }
        if left < d.def.fade {
            if let Some(m) = materials.get_mut(&d.material) {
                m.base_color.set_alpha(d.def.color[3] * left / d.def.fade.max(0.01));
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
