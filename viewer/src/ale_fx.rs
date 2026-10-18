//! A runtime for the game's ALE particle effects (`Game::effects`, common/effects-common.ale),
//! driven by the nodes' own parameters: the power-ups' spinning icons, the slide puff.
//!
//! Each effect pairs emitters with appearances (see `bf::ale`):
//!  emitter     lifespan (s; huge = forever), transform (offset and turn of its frame), initial
//!              particles, rate (/s), particle life (s), speed (m/s); its shape by class:
//!              cube (width / height / depth, spread about +y), cone (radius and spread about
//!              +y), sphere (radius, outward)
//!  appearance  colour, alpha, size, width factor, aspect and rotation over a particle's life;
//!              a texture, its blend (D3DBLEND src, dest: a dest of ONE added to the picture,
//!              5,2 by its alpha, 2,2 and 3,2 whatever their alpha, 3,2 by its own colour; the
//!              rest blended by their alpha, 5,6), and "perp": the
//!              quad lies flat in the emitter's frame (normal +y), turned by the appearance's
//!              transform at the effect's time it was born (the icons' particles fan out round
//!              the turn); otherwise it faces the camera. The game turns quads by the
//!              appearance's facing type instead (see FACING); facing 3 is drawn the game's way
//!              in the effects compiled with `Compiled::facing`.
//! Curves with keys run on the emitter's time in seconds (flagged ones repeat); per-particle
//! floats run over the particle's life (0-1). An emitter flagged "attached" (h_e2999ffd: the
//! projectiles' bolts) keeps its particles in its own frame, so they travel with it; one with no
//! rate and no initial count fires a single particle when it starts (the guns' tracer).
//! Motion-blurred appearances (h_fe7c2071) and stretched ones turn along their particle's
//! motion. Two appearances are drawn: the basic one (class h_111035aa, a quad per particle)
//! and the beam (class h_1f55f13e: a ribbon through the emitter's live particles in the order
//! they were born, facing the camera; its own colour h_048767e8, alpha h_04fc9016 and width
//! h_fba203b8 over a particle's life, texture h_1ba23359, blend h_1506eb6c): the cutter's
//! trail, the sniper's beam.
//!
//! A "light_" effect (an effect type's light-effect: light_explosion, light_phosphor ...) lights
//! the scene instead of drawing: each of its particles is a point light whose reach is the
//! appearance's size (m) and whose strength is its colour x alpha x size, as the hand-made DNA
//! light in play_fx.rs reads light_powerup_pill. (The effect types with a light-effect also
//! list a light object type in h_f8c6a8b4: h_09020c30, a point light of range 10, for the
//! Therm Sweeper's muzzle and explosion. That the light effect drives it is an inference from
//! the data; the code isn't traced.)

use std::collections::HashMap;
use std::sync::Arc;

use bevy::{
    asset::RenderAssetUsages,
    math::Affine2,
    pbr::NotShadowCaster,
    prelude::*,
    render::render_resource::TextureFormat,
};

use crate::bf::ale::Node;
use crate::bf::character::{Address, Game};
use crate::bf::hash::h;
use crate::scene::mip_image;

// emitter parameters
const LIFESPAN: u32 = 0xF27F_DE7D;
const TRANSFORM: u32 = 0xE13A_59A1;
const INITIAL: u32 = 0x0F9A_9D52;
/// For an emitter with no rate and no initial count, read at its start as a burst:
/// exp-lrg-flash's 4.1 (the recording's first flash frame is a wide haze, not one flare). Only
/// for the grenades' effects (`Compiled::recorded`): elsewhere it's a curve over the emitter's
/// time on ~40 emitters (laser hit rings, tracers ...), more likely an emit count; those keep
/// one particle. (It is Freelancer's Emitter_EmitCount: its hash is the game's hash of that
/// name. Keyed over the emitter's time on smoke-grenade-flsh - 9 keys of 10.6-13.7 over
/// 0.09-0.98 s - where the Gas recording's flash flickers for ~1 s; the demo's is the one
/// burst. How the keys emit isn't settled: exp-lrg-flash is keyed too; sonic_grenade's are
/// read as a rate after its burst, EMIT_COUNT_RATE.)
const BURST: u32 = 0xE722_1F95;
/// Emitters whose emit count (BURST) runs over their life as well, as particles per second, on
/// top of the opening burst: the Sonic's sonic_grenade.emt (no rate, no initial count, emit
/// count keyed 27.6 -> 38.3 over 0.007-0.297 s, lifespan 0.35 s: 28 particles at once, then
/// ~11 more). Checked against the Sonic recording (sonic/hi 449-481): its dome is whole and
/// smooth from the first frames, so it starts with many particles (with one particle it was a
/// few flat sheets); its spread opens from 28-74 to 90 degrees by 0.25 s and the recording's
/// flat ring shows from ~0.27 s, so particles keep coming after the start (with the burst
/// alone there is no ring). Read per 30 fps game frame instead (~300 particles), the dome was a
/// solid white blob. An inference, for this emitter only: the Frag's exp-lrg-flash and the
/// Gas's smoke-grenade-flsh keep the one burst they were checked with.
///
/// Its air field sonic_grenade_air.fld (a wind of 0.004 m/s, so a drag: Approach 0.80 falling
/// to 0.15 by 0.43 s) stays off. Applied 6 times a second (a fit to the reference agent's early
/// half-widths) it held the particles in a ball: 18-26% of the dome's pixels fully white
/// (recorded 0-2%) and the ring 5.3 m out at +0.6 s where the recording's is past 6.3-9.5 m.
/// Off, the dome is a see-through shell with a bright rim (0-8% white, sdm_e34 captures
/// scratchpad s81/N0), seen from above a ring with a dark middle, and it spreads faster - wider
/// than recorded over its first ~0.2 s, where the quads' own size (0 -> 5.5 m over a quarter of
/// their life) sets the width.
const EMIT_COUNT_RATE: [&str; 1] = ["sonic_grenade.emt"];
/// Emitter_Frequency (the emitter node's +0x30): particles per second.
const RATE: u32 = 0x023C_350C;
/// Emitter_LODCurve (the emitter node's +0x48): a float animation over the effect's distance
/// LOD (0-1) that scales each batch's count before it's cut to a whole number (FUN_00065800 at
/// 0x659ed-0x65a12). Read for the effects compiled with `Compiled::counted`.
const EMIT_LOD: u32 = 0xFE07_7E05;
/// Appearance_LODCurve (the appearance's +0x3c): the particle's size is Scale x this at the
/// effect's LOD (the appearance's bounds code at 0x2a12f0 reads both). 1 on the Therm Sweeper's
/// effects. Read for the effects compiled with `Compiled::counted`.
const APP_LOD: u32 = 0x0C53_B008;
/// An effect object's distance LOD (the render cull FUN_000e7850 -> FUN_00156050): d = tan(FOV
/// / 2) x the camera's distance past the effect type's bounding sphere (FUN_00219ed0: centre
/// h_e99750a9 in the effect's frame, radius h_1ab09530); LOD 1 up to LOD_NEAR, falling to 0 at
/// LOD_FAR. This is tan(35 degrees): the game camera's 70-degree field of view (its +0xc8, set
/// at 0x6bc09), whatever the demo's view is.
const LOD_TAN: f32 = 0.700_207_5;
/// The effect LOD's near distance (m): 10.0 at 0x3a4fdc (0x2194d0), x (2 - w) / 2 for the
/// world's load w (0x3bc720, which FUN_000d9090 writes: 0 in multiplayer, varying in the
/// campaign) and x 1 for one view (world+0xc74). The demo takes w = 0 everywhere, its
/// assumption: the factor is 1.
const LOD_NEAR: f32 = 10.0;
/// The far one (m): 60.0 at 0x3a2220 (0x2194e0), x the same.
const LOD_FAR: f32 = 60.0;
/// The game's frame (s): it runs at 30 a second (#73 spec: each game frame shows on two video
/// frames; play_shots' TICK). A counted effect's emitters update once a game frame, whatever
/// the demo's frame rate (`game_count`): FUN_00065800 cuts each update's count to a whole
/// number, so stepped once a render frame the count changed with the frame rate (exp-rocket-fire
/// made 1 particle at 15 and 60 fps where 30 gives 2, the shrapnel 47 to 29 from 15 to 144 fps).
const GAME_FRAME: f32 = 1.0 / 30.0;
const LIFE: u32 = 0x0A63_5880;
const SPEED: u32 = 0x0AB1_80C5;
const CUBE: [u32; 3] = [0xFD22_A11C, 0xEFB0_F8CF, 0x14B7_4EB2];
const CUBE_SPREAD: [u32; 2] = [0xE282_006B, 0x08C7_217F];
const CONE_RADIUS: [u32; 2] = [0x1C73_D16B, 0xF636_F07F];
const CONE_SPREAD: [u32; 2] = [0x100E_7F55, 0xFA4B_5E41];
const SPHERE_RADIUS: [u32; 2] = [0xF231_FF60, 0x1874_DE74];
const CLASS_CONE: u32 = 0xE1F4_0411;
const CLASS_SPHERE: u32 = 0xF940_F7F3;
const ATTACHED: u32 = 0xE299_9FFD;
const MOTION_BLUR: u32 = 0xFE7C_2071;
/// The basic particle appearance (billboard / perp quad).
const CLASS_APPEARANCE: u32 = 0x1110_35AA;
/// The beam appearance (a ribbon through the particles) and its parameters.
const CLASS_BEAM: u32 = 0x1F55_F13E;
const BEAM_COLOR: u32 = 0x0487_67E8;
const BEAM_ALPHA: u32 = 0x04FC_9016;
const BEAM_WIDTH: u32 = 0xFBA2_03B8;
const BEAM_TEXTURE: u32 = 0x1BA2_3359;
const BEAM_BLEND: u32 = 0x1506_EB6C;
// appearance parameters
const COLOR: u32 = 0xF89C_93A4;
const ALPHA: u32 = 0xF8E7_645A;
const SIZE: u32 = 0x00FA_C601;
const WIDTH: u32 = 0x04B5_FC1B;
const ASPECT: u32 = 0x0DB2_CC8D;
const ROTATE: u32 = 0xF24D_7541;
const PERP: u32 = 0x0D64_5074;
const TEXTURE: u32 = 0xF736_F94E;
const BLEND: u32 = 0x1DAE_A8C0;
/// An animated texture's frame over a particle's life (0-1 of its frames). Read from the data's
/// values: a ramp (Exp5 on exp-lrg-add 0.61 -> 1.0, the fire puffs 0.21 -> 0.98, campfires 0 ->
/// 1) runs the flipbook over the particle's life; a constant above 0 (the muzzle flashes'
/// 0.60, exp-lrg-fire's 0.70) holds that one frame; a constant 0 (the shield icon's arcb)
/// leaves the texture playing at its own rate. An inference (Freelancer's TexFrame reads the
/// same way), applied to the grenades' effects only (`Compiled::recorded`): played at 30 fps
/// and wrapped, exp-lrg-add went back to its bright first frames half a second into the Frag's
/// blast, which the recording doesn't show. Other effects keep the 30 fps loop.
const TEX_FRAME: u32 = 0x1865_7E4A;
/// The pair an appearance without one (h_1daea8c0) is drawn with: SRCALPHA, ONE, the most common
/// (the demo's default; the appearance constructor's isn't read).
const BLEND_ADD: (u32, u32) = (5, 2);
/// The blend pair's numbers are the PC's D3DBLEND (src, dest): 2 ONE, 3 SRCCOLOR, 5 SRCALPHA, 6
/// INVSRCALPHA (0x2a6f00 turns them into the console's through the table at 0x4036b4: 1, 0x300,
/// 0x302, 0x303). 5,2 is on 518 appearances, 5,6 on 207. A dest of ONE adds the particle to the
/// picture whatever its source factor: 3,2 on 33 (the lava's eruptions slava-erupt*, its wading
/// rings and splashes env_sl_*, the teleport and ion beams) and 2,2 on 13 (slava-fuzz, the laser
/// hits). Drawn as alpha blends before #228, they were dark smudges (the tutorial's lava bursts,
/// slava-erupt-jet, came out as brown balls where take08 shows a bright fire column).
const D3DBLEND_ONE: u32 = 2;
/// D3DBLEND_SRCCOLOR: the particle's colour is its own factor (see `Source`).
const D3DBLEND_SRCCOLOR: u32 = 3;

/// Whether an appearance's blend pair adds it to the picture (its dest factor is ONE).
fn adds(blend: (u32, u32)) -> bool {
    blend.1 == D3DBLEND_ONE
}

/// What an added appearance's source factor weighs its colour by. The console draws a particle as
/// the texel x its vertex colour, with the texel's alpha x the vertex alpha beside it (pixel shader
/// 0x3b89e0, which FUN_0029f5d0 sets for any pair whose dest isn't INVSRCALPHA, then the pair as
/// it's stored); the vertex colour is the appearance's colour and the vertex alpha its alpha curve
/// (x the angle fade), never one folded into the other (FUN_002a1a50 packs them side by side, the
/// particle vertex shader 0x3b8bf8 passes the colour through). So the alpha curve fades a SRCALPHA
/// particle only: with ONE the particle adds texel x colour whatever its alpha, with SRCCOLOR
/// (texel x colour) squared. take08's lava bursts (slava-erupt-jet, 3,2, alpha 0 at birth and at
/// most 0.59) show a bright fire column from their first frames, as that reading gives.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
enum Source {
    /// SRCALPHA (5): weighed by its alpha (the alpha curve x the texel's); the blended pairs too
    Alpha,
    /// ONE (2): its colour as it is
    One,
    /// SRCCOLOR (3): its colour by itself
    Colour,
}

impl Source {
    /// The source factor of an added appearance's blend pair (`adds`); any other than ONE and
    /// SRCCOLOR is drawn by its alpha.
    fn of(blend: (u32, u32)) -> Source {
        match blend.0 {
            D3DBLEND_ONE => Source::One,
            D3DBLEND_SRCCOLOR => Source::Colour,
            _ => Source::Alpha,
        }
    }
}
/// The effect appearance (class h_0ec77ea0): each particle of its emitter carries an effect of
/// its own, named by h_0ec7a290 (stun_grenade_master's node "stun_grenade_master" -> the
/// effect "stun_grenade": the Energy grenade's bolts). Only for the grenades' effects
/// (`Compiled::recorded`); the library has other effect appearances, unchecked.
const CLASS_SPAWNER: u32 = 0x0EC7_7EA0;
const SPAWN_EFFECT: u32 = 0x0EC7_A290;
/// The missing emitters given a stand-in (`missing_emitter`): stun_grenade_master's
/// h_ed10c55f (stun_grenade_init.app) and h_f48dc74d (stun_grenade_init#1.app).
const STAND_IN_EMITTERS: [u32; 2] = [0xED10_C55F, 0xF48D_C74D];

/// Perp quads lie flat in the emitter's frame, i.e. face along the emitter's axis (the laser
/// hits' rings, a bolt's cross-section). From a sphere emitter throwing its particles out at
/// this speed (m/s) or more (and a cone emitter in PERP_OUT_CONES), a perp quad faces out along
/// its own direction instead: the Frag's exp-fire-add (0.48 m/s, radius 1 -> 3.3 m) is a round
/// shell as in the recording (frag/a 0700-0730), not a stack of flat discs seen edge-on. The
/// power-ups' icons (sphere emitters at 0.02-0.16 m/s, verified flat) keep the emitter's
/// frame. The threshold is the demo's: the data has no flag for it. Only for the grenades'
/// effects (`Compiled::recorded`).
const PERP_RADIAL_SPEED: f32 = 0.3;
/// Cone emitters whose perp quads face out along their direction too (see PERP_RADIAL_SPEED):
/// the Sonic's sonic_grenade.emt (10-17 m/s, spread 28-74 opening to 90 degrees by 0.25 s). So
/// it's a dome with a bright rim, as recorded (sonic/hi 451-465), then a low wall round the
/// ground as its spread reaches 90 (the recording's ring from ~0.27 s); lying flat it was a
/// flat streak. Only this one, checked against the Sonic recording: the Energy's stun_hit_s
/// has a perp cone too and stays as it was checked.
const PERP_OUT_CONES: [&str; 1] = ["sonic_grenade.emt"];
/// GeneralApp_FacingType (the appearance's +0x44; names of the "GeneralApp_" parameters at
/// 0x403a58-0x403bc0, their table from 0x403904): how the game turns a particle's quad. The
/// appearance's draw (0x2a1de6) hands the particle vertex shader (0x3b8bf8) a one-hot selector
/// for it (the table at 0x390ce8, shader constant 45), and the shader builds the quad from:
/// 0, the particle's own transform; 1, the camera (a billboard); 2, the plane square to the
/// particle's motion; 3, a quad stretched along its motion (height along it, ScaleY) and turned
/// about it to face the camera, Freelancer's "rect" appearance (the .rect sparks are all 3).
/// The motion is the particle's velocity, normalised each frame while it moves (0x2a17b0), and
/// +z of the particle's frame for one born still (0x2a1768). The flag the demo calls PERP
/// (0d645074) is GeneralApp_AngleFadeOut: the shader fades out a quad seen within 20 degrees of
/// edge-on, to nothing at 5 (cosines 0.342 and 0.087, set at 0x2a1c9d), and doesn't turn it
/// (that fade isn't drawn: the demo's materials are shared per life step). The demo's perp rules
/// (flat across the emitter's y, PERP_RADIAL_SPEED) stand in for these elsewhere; facing 3 is
/// drawn as the shader does only for the effects compiled with `Compiled::facing` (see `Pair::axial`).
const FACING: u32 = 0xE354_2300;
/// GeneralApp_ShapeType (the appearance's +0x48): which of the game's particle shapes the quad
/// is. The table at 0x402a4c lists them (FUN_002b1bb0 registers each with FUN_002b1bf0, into
/// 0x443658; the appearance's draw picks its own at 0x2a1a7e), each a vertex list scaled by the
/// particle's Scale x ScaleX / ScaleY: 1, a triangle round a circle of radius 1; 2, a quad from -1
/// to +1 (twice its Scale across: 411 of the library's appearances); 3, a quad standing on the
/// particle, x -0.5..0.5 and y 0..1 (89); 4, a 12-point shape; 5, a quad from -0.5 to 0.5 (its
/// Scale across: 284, the muzzle flashes and the hit dust among them). The demo draws every quad
/// as a 5; the effects compiled with `Compiled::shapes` draw 2 twice as big and 3 standing on its
/// particle (1 and 4 stay quads).
const SHAPE: u32 = 0x17D4_5550;
/// Air fields, linked to an appearance by the effect's pair list (appearance -> field, as
/// Freelancer's ALE links them; FxAirField, AirField_Magnitude and AirField_Approach are the
/// game's hashes of Freelancer's names). One pulls its particles' velocity toward a wind of
/// Magnitude m/s along the field's +y (turned by its transform's rotation over the effect's
/// time), Approach of the way once each 1/APPROACH_FPS s.
///
/// Run only for the fields in AIR_FIELDS, the ones checked against a recording; every other
/// field (the Frag's exp-lrg-air, the gravity fields FxGravityField h_e644c021 of the
/// shrapnels and exp-lrg-dirt, turbulence fields FxTurbulenceField h_0b72ea10) is left off
/// until checked against its own capture (#101); the Sonic's sonic_grenade_air.fld was checked
/// and left off (see EMIT_COUNT_RATE).
/// Not done: a field node's own life (h_f27fde7d: gas-grenade.fld's 12 s) is ignored, the field
/// runs as long as its particles do; the wind is in the world's frame (the blasts are placed
/// upright, and a trail's spinning grenade would spin it), while an attached particle's
/// velocity is in its effect's frame - no attached particle has an allowed field yet.
const CLASS_AIR_FIELD: u32 = 0xE625_323F;
const AIR_MAGNITUDE: u32 = 0xE5E3_524C;
const AIR_APPROACH: u32 = 0x1042_3CEB;
/// The air fields that run (node names): the Gas cloud's two (gas-grenade.fld: a wind of about
/// +-1 m/s swinging round; gas-grenade2.fld: 0.26 m/s up; checked against the Gas recording)
/// and grenade_trail_rise (0.8 m/s up: the tester saw the trail rise the right way, thinner and
/// more vertical than the recordings' - left on, a partial match); the Therm Sweeper's trail's
/// mtrail-rise (0.3 m/s up, Approach 1) and mtrail-rise#1 (1.0 m/s up, Approach 1.66, taken as
/// 1): they hold the puffs where they're born, slowly rising, instead of their cones' 0.9 and
/// 14.5 m/s back along the missile's way, as take21's side view shows them (a cloud round the
/// missile from 0.1 s, then a line of puffs it leaves behind; flying back they passed the holder).
const AIR_FIELDS: [&str; 5] = ["gas-grenade.fld#1.fld", "gas-grenade2.fld#1.fld", "grenade_trail_rise", "mtrail-rise", "mtrail-rise#1"];
/// How often an air field's Approach is applied (per second): once per 30 fps game frame. Not
/// in the data: fitted to the Gas recording, whose cloud is ~6 m wide and ~4 m tall and swallows
/// Tex although gas-grenade_Cone.emt#1.emt throws its puffs out at 4-6.5 m/s for 2.3-4.4 s.
/// Captured from 12 m (scratchpad gas76/fps*): at 60 every puff stopped within ~0.3 m, a
/// 6.5 x 4 m cloud of billboards all centred in Tex, who stood out in front of it; at 30 it's
/// ~6.5 x 5 m and veils him as the recording does; at 15, ~9 x 6 m; at 8, ~13 m. An inference.
const APPROACH_FPS: f32 = 30.0;
/// An upright streak: a camera-facing appearance (not perp, not motion-blurred) whose width
/// factor (WIDTH) stays under this part of its smallest height factor (ASPECT, counted as 1.0 at
/// most) all its life, with a constant Rotate. It is drawn with no random roll - roll 0 is
/// upright on the screen, not in the world: it faces the camera, so under a steep camera pitch
/// it lies back with the view - turned only by its Rotate, and rides its appearance
/// transform's offset over its life (`Pair::rise`). The one
/// grenade appearance it picks out is phosphor_grenade_init_spike.app (width 0.05 -> 0.30 of a
/// size that peaks at 5.9 m): the Light recording's tall thin vertical beam (lg 0413-0487,
/// 1181-1289), where the random roll drew a fan of rays. The threshold is the demo's (the data
/// has no flag for it); only for the grenades' effects (`Compiled::recorded`).
const UPRIGHT_WIDTH: f32 = 0.5;
/// A spark: a perp appearance born longer than this (ASPECT at birth) on a cone emitter that
/// throws its particles out at PERP_RADIAL_SPEED or more. It is drawn as a camera-facing streak
/// along its motion (as a motion-blurred one is), not as a quad lying flat in the emitter's
/// frame. The one grenade appearance it picks out is phosphor_grenade-shrap.rect.app (spark.tga,
/// aspect 2.2 -> 0.25, ~100/s at 4-6 m/s within 49 degrees of up): the Light recording's
/// starburst of rays fanning up from the flare at ignition and the sparks round its base after
/// (lg 0397-0411, 1073-1085, 1200), which lying flat drew as short horizontal lines. The Frag's,
/// Gas's, Sonic's and Sentry's perp appearances are born square or squat (aspect 0.04-1.01) and
/// stay perp quads (the Sonic's facing out, PERP_OUT_CONES). The threshold is the demo's; only
/// for the grenades' effects (`Compiled::recorded`).
const SPARK_ASPECT: f32 = 1.5;
/// Camera-facing quads that stand vertical - turned about the world's vertical to face the
/// camera, from their first frame (no random roll) - instead of turning along their motion
/// once stretched: the Sonic's sonic_grenade_flash.app (godray.tga, aspect 0.5 -> 15.8: shafts
/// up to ~17 m tall, thrown out at 54-83 degrees from the vertical). Along their motion they
/// lay nearly flat, a white glare
/// across the ground; the recording's shafts stand up from the blast (sonic/hi 469-497). The
/// recording's lean out by up to ~30 degrees and there are 2-4 at a time, where this draws the
/// data's ~45 a second straight up: a partial match. Only this appearance, checked against the
/// Sonic recording (the Energy's stun_hit_s godrays are left as they were).
const VERTICAL_STREAKS: [&str; 1] = ["sonic_grenade_flash.app"];
/// Materials per appearance: its colour and alpha at this many points of a particle's life.
const STEPS: usize = 12;
/// Particles alive at once, over all effects.
const MAX_PARTICLES: usize = 3000;
/// A light particle's lumens per unit of colour x alpha x size: the tuning play_fx.rs uses for
/// the DNA's light_powerup_pill (fitted to the capture's brightening round the DNA).
pub const LIGHT_LUMENS: f32 = 30_000.0;
/// Point lights' strength under the console look (see level_scene::POINT_LIGHT_SCALE).
const LIGHT_SCALE: f32 = crate::level_scene::POINT_LIGHT_SCALE;

/// One emitter and the appearance its particles take.
pub struct Pair {
    emitter: Node,
    app: Node,
    /// materials by life step, then by flipbook frame (one frame for a still texture)
    steps: Vec<Vec<Handle<StandardMaterial>>>,
    /// an animated texture's frames per second
    fps: f32,
    /// how the flipbook's frame is picked (see TEX_FRAME)
    frame: FrameMode,
    perp: bool,
    /// the game's shape of its quad (see SHAPE): 2 or 3 where the effect is compiled with
    /// `Compiled::shapes`, else 5 (the demo's quad)
    shape: u32,
    /// a facing-3 appearance drawn as the game's shader does (see FACING): stretched along the
    /// particle's motion and turned about it to face the camera. Its perp (AngleFadeOut)
    /// appearances in the effects compiled with `Compiled::facing` only; every other quad keeps
    /// the demo's rules
    axial: bool,
    /// particles stay in the emitter's frame (move with it)
    attached: bool,
    /// turned along their motion
    streak: bool,
    /// an upright streak (see UPRIGHT_WIDTH): no random roll, and it rises by its appearance
    /// transform's offset over its age in seconds, up to this time (the offset's last key: the
    /// spike's 0 -> 2.26 m up over 0.92 s). The offset read over the particle's age rather than
    /// the effect's time is an inference: over the effect's time (repeating every 0.92 s) the
    /// whole beam would rise and drop back once every 0.92 s, while the recording's beam top
    /// flickers by ~25% about every 0.3 s with no slow cycle (the Light recording, frames
    /// 1181-1289: the second flare's beam top under a still camera)
    rise: Option<f32>,
    /// a beam appearance: the ribbon's material (vertex colours carry colour x alpha)
    beam: Option<Handle<StandardMaterial>>,
    /// a "light_" effect's pair: its particles are point lights
    light: bool,
    /// the fields its appearance is linked to (see CLASS_AIR_FIELD)
    fields: Vec<Node>,
    /// stretched quads drawn vertical (VERTICAL_STREAKS)
    vertical: bool,
    /// an effect appearance's pair (CLASS_SPAWNER): the effect each particle carries
    child: Option<Arc<Compiled>>,
}

/// How a pair's flipbook frame is picked (see TEX_FRAME).
#[derive(Clone, Copy, PartialEq)]
enum FrameMode {
    /// at the texture's own rate, round and round
    Play,
    /// this part of the frames (0-1), all its life
    Hold(f32),
    /// along the appearance's TEX_FRAME curve over the particle's life
    Life,
}

impl FrameMode {
    fn of(app: &Node) -> Self {
        match app.params.get(&TEX_FRAME) {
            Some(crate::bf::ale::Value::Floats(items)) => match items.first() {
                Some((_, _, keys)) if keys.len() > 1 => FrameMode::Life,
                Some((_, _, keys)) => match keys.first() {
                    Some(&(_, v)) if v > 0.0 => FrameMode::Hold(v),
                    _ => FrameMode::Play,
                },
                None => FrameMode::Play,
            },
            _ => FrameMode::Play,
        }
    }
}

/// An effect ready to run.
pub struct Compiled {
    pub name: String,
    pairs: Vec<Pair>,
    /// Compiled with the rules first checked against the Frag recording (the grenades' blast and
    /// trail effects, `load_recorded`; and, with the game's facing, the Therm Sweeper's trail,
    /// explosion and launch dust and the effects triggers make, `load_recorded_facing`, the
    /// latter checked against the tutorial's lava bursts, take08 of #228): the flipbook frame curve
    /// (TEX_FRAME), the opening burst (BURST) and outward-facing perp quads
    /// (PERP_RADIAL_SPEED). Every other effect plays as before them; they may hold for those
    /// too, once checked against their own captures.
    pub recorded: bool,
    /// Compiled with the game's facing 3 (see FACING, `Pair::axial`): the muzzle effects, which
    /// play in the muzzle hardpoint's frame (play_shots.rs; compared with main's, not with each
    /// gun's take), the Therm Sweeper's trail, explosion and launch dust (its muzzle
    /// effect; play_missile.rs, play_shots.rs; checked against its takes) and the effects
    /// triggers make (play_triggers.rs; the tutorial's lava bursts, take08 of #228). Other
    /// effects keep the demo's perp rules until checked against their own captures.
    pub facing: bool,
    /// Drawn with the game's particle shapes (see SHAPE): the effects compiled with both the
    /// recorded rules and the game's facing, the Therm Sweeper's trail, explosion and launch
    /// dust and the effects triggers make (`load_recorded_facing`), checked against their takes
    /// (the triggers' with the tutorial's lava bursts, #228). Every other effect keeps the
    /// demo's quad, half the size of the game's for its shape-2 appearances.
    pub shapes: bool,
    /// Emitted by the game's count, once a game frame (FUN_00065800, `game_count`,
    /// GAME_FRAME), and drawn at the effect's distance LOD (see LOD_TAN, EMIT_LOD, APP_LOD):
    /// the same effects, the Therm Sweeper's trail, explosion and launch dust and the triggers'
    /// effects. Every other
    /// effect keeps the demo's count (a rate's fraction carried over each render frame, an emit
    /// count's burst at the start) and no LOD, until checked against its own captures.
    pub counted: bool,
}

impl Compiled {
    /// The materials its particles draw with (the first of each pair's), for `warm_up`.
    pub fn materials(&self) -> Vec<Handle<StandardMaterial>> {
        self.pairs.iter().flat_map(|p| {
            let own = p.beam.clone().or_else(|| p.steps.first().and_then(|s| s.first()).cloned());
            own.into_iter().chain(p.child.iter().flat_map(|c| c.materials()))
        }).collect()
    }

    /// The effects its particles carry (its effect appearances' effects, CLASS_SPAWNER).
    pub fn children(&self) -> Vec<Arc<Compiled>> {
        self.pairs.iter().filter_map(|p| p.child.clone()).collect()
    }

    /// How long a one-shot run of the effect lasts: its longest emitter plus its longest-lived
    /// particles (finite emitters only; a few seconds otherwise).
    pub fn duration(&self) -> f32 {
        self.pairs.iter().map(|p| {
            let span = p.emitter.float(LIFESPAN).unwrap_or(1.0);
            let span = if span > 100.0 { 3.0 } else { span };
            span + p.emitter.curve(LIFE, 0.0, 0.0).unwrap_or(1.0).max(0.0)
        }).fold(0.0, f32::max)
    }
}

/// The quad and the effects compiled so far (by name hash).
#[derive(Resource)]
pub struct AleAssets {
    quad: Handle<Mesh>,
    /// a unit quad standing on its middle-bottom (y 0..1): the game's shape 3 (see SHAPE)
    quad_base: Handle<Mesh>,
    /// by (name hash, `recorded`, `facing`)
    effects: HashMap<(u32, bool, bool), Option<Arc<Compiled>>>,
    /// by (texture name hash, linear, the source factor: see `texture`)
    textures: HashMap<(u32, bool, Source), Option<Handle<Image>>>,
}

/// Seconds the effects advance this frame, if the app steps time itself (fixed-step captures);
/// otherwise real time.
#[derive(Resource)]
pub struct AleClock(pub f32);

/// A running effect at its entity's place. Inactive, it makes no new particles (those out
/// finish their lives).
#[derive(Component)]
pub struct AleEffect {
    pub fx: Arc<Compiled>,
    pub active: bool,
    /// the user parameter (0-1) that picks between a curve's variants
    pub sp: f32,
    /// run each emitter once (its lifespan), not over and over
    pub once: bool,
    placed: bool,
    /// where the effect was last frame (new particles spread from there)
    prev: Option<(Vec3, Quat)>,
    /// per pair: the beam ribbon drawing this effect's particles (beam pairs)
    strips: Vec<Option<Entity>>,
    t: f32,
    acc: Vec<f32>,
    started: bool,
    rng: u32,
    /// leave out perp quads lying along the effect's axis (not facing along it): a muzzle
    /// flash seen down its own axis from the holder's scope (play_shots.rs; the demo's rule)
    pub hide_flat: bool,
    /// its effect type's bounding sphere (centre in the effect's frame, radius), from which its
    /// distance LOD is worked out each frame (see LOD_TAN); None: LOD 1
    pub lod_sphere: Option<(Vec3, f32)>,
    /// its distance LOD now (1 near, 0 far)
    lod: f32,
    /// per pair: the emitter's time of its last batch (`game_count`)
    last: Vec<f32>,
    /// the game frames its counted emitters have run (GAME_FRAME)
    ticks: u32,
}

/// One batch of an emitter's new particles in a frame (`emit`): the demo's one a render frame,
/// or a counted effect's one a game frame (GAME_FRAME).
struct Batch {
    /// how many
    n: usize,
    /// the emitter's time they're born at, where its curves are read
    t: f32,
    /// how far back over the emitter's time (its animated offset) they spread, the newest at `t`
    span: f32,
    /// the shares of the effect's move this render frame (from its last place to this one) the
    /// oldest is born before and the newest at
    from: f32,
    to: f32,
    /// their age at the render frame's start (`animate` adds the frame's time)
    age: f32,
    /// the effect's time they're born at
    born: f32,
    /// the demo's count: the rate's fraction carried in (a steady light spaces its particles by
    /// it, `steady_rate`)
    carried: Option<f32>,
}

#[derive(Component)]
struct AleParticle {
    fx: Arc<Compiled>,
    pair: usize,
    /// its effect's distance LOD when it was born (APP_LOD)
    lod: f32,
    age: f32,
    life: f32,
    vel: Vec3,
    sp: f32,
    /// the effect's time when it was born (fields' curves run on the effect's time)
    born: f32,
    roll: f32,
    step: usize,
    /// perp quads: the plane they lie in (the effect's and the emitter's turn, then the
    /// appearance's turn at the effect's time when the particle was born: born over the
    /// appearance's spin, they fan out); for attached particles, in the effect's frame
    frame: Quat,
    /// attached particles: the effect they move with, and their place and motion in its frame
    owner: Option<Entity>,
    local: Vec3,
    /// the effect's turn last seen
    turn: Quat,
    /// the effect that made it, and the order it was born in (beam ribbons join particles so)
    source: Entity,
    seq: u64,
}

/// A beam ribbon: the live particles of one beam pair of one effect, joined.
#[derive(Component)]
struct BeamStrip {
    source: Entity,
    pair: usize,
    fx: Arc<Compiled>,
    mesh: Handle<Mesh>,
    age: f32,
}

#[derive(Resource, Default)]
struct Seq(u64);

#[derive(Resource, Default)]
struct Count(usize);

pub fn plugin(app: &mut App) {
    app.init_resource::<Count>().init_resource::<Seq>()
        .add_systems(PostUpdate, (emit, animate, beams).chain().before(TransformSystem::TransformPropagate))
        .add_systems(Update, end_warm_up);
}

/// A quad drawn (out of sight) for its material's render pipeline to be built before an effect
/// first needs it; gone after this many frames.
#[derive(Component)]
struct WarmUp(u32);
/// Frames the warm-up quads stay (the captures' 60 warm-up frames, and some).
const WARM_UP_FRAMES: u32 = 90;

/// Draw each of an effect's materials once, far below the ground and never culled, so their
/// render pipelines are built while the map loads. Bevy builds a pipeline the first time
/// something needs it, on other threads, and draws nothing with it until it's ready: a blast's
/// first additive layers could go missing for its first half second when the machine was
/// busy (the reference agent's run beside two other instances).
pub fn warm_up(commands: &mut Commands, assets: &AleAssets, fx: &Compiled) {
    for m in fx.materials() {
        commands.spawn((Mesh3d(assets.quad.clone()), MeshMaterial3d(m), NotShadowCaster, bevy::render::view::NoFrustumCulling,
                        Transform::from_xyz(0.0, -500.0, 0.0).with_scale(Vec3::splat(0.01)), Visibility::default(), WarmUp(WARM_UP_FRAMES)));
    }
}

fn end_warm_up(mut commands: Commands, mut quads: Query<(Entity, &mut WarmUp)>) {
    for (e, mut w) in &mut quads {
        w.0 = w.0.saturating_sub(1);
        if w.0 == 0 {
            commands.entity(e).despawn();
        }
    }
}

impl AleAssets {
    pub fn new(meshes: &mut Assets<Mesh>) -> Self {
        AleAssets { quad: meshes.add(Rectangle::new(1.0, 1.0)), quad_base: meshes.add(Mesh::from(Rectangle::new(1.0, 1.0)).translated_by(Vec3::Y * 0.5)),
                    effects: HashMap::new(), textures: HashMap::new() }
    }

    /// An effect compiled already (`load`).
    pub fn cached(&mut self, effect: u32) -> Option<Arc<Compiled>> {
        self.effects.get(&(effect, false, false)).cloned().flatten()
    }

    /// An effect compiled already with the recorded rules (`load_recorded`).
    pub fn cached_recorded(&mut self, effect: u32) -> Option<Arc<Compiled>> {
        self.effects.get(&(effect, true, false)).cloned().flatten()
    }

    /// An effect compiled already with the recorded rules and the game's facing
    /// (`load_recorded_facing`).
    pub fn cached_recorded_facing(&mut self, effect: u32) -> Option<Arc<Compiled>> {
        self.effects.get(&(effect, true, true)).cloned().flatten()
    }

    /// Compile an effect of the library by name hash (cached); None if the library lacks it.
    pub fn load(&mut self, game: &mut Game, images: &mut Assets<Image>, materials: &mut Assets<StandardMaterial>,
                effect: u32) -> Option<Arc<Compiled>> {
        self.load_as(game, images, materials, effect, false, false)
    }

    /// `load`, with the rules checked against the Frag recording only (see `Compiled::recorded`):
    /// for the grenades' blast and trail effects.
    pub fn load_recorded(&mut self, game: &mut Game, images: &mut Assets<Image>, materials: &mut Assets<StandardMaterial>,
                         effect: u32) -> Option<Arc<Compiled>> {
        self.load_as(game, images, materials, effect, true, false)
    }

    /// `load`, with the game's facing 3 (see `Compiled::facing`): for the muzzle effects.
    pub fn load_facing(&mut self, game: &mut Game, images: &mut Assets<Image>, materials: &mut Assets<StandardMaterial>,
                       effect: u32) -> Option<Arc<Compiled>> {
        self.load_as(game, images, materials, effect, false, true)
    }

    /// `load_recorded`, with the game's facing 3 (see `Compiled::facing`): for the Therm
    /// Sweeper's trail, explosion and launch dust (its muzzle effect, play_shots.rs) and the
    /// effects triggers make (play_triggers.rs).
    pub fn load_recorded_facing(&mut self, game: &mut Game, images: &mut Assets<Image>, materials: &mut Assets<StandardMaterial>,
                                effect: u32) -> Option<Arc<Compiled>> {
        self.load_as(game, images, materials, effect, true, true)
    }

    fn load_as(&mut self, game: &mut Game, images: &mut Assets<Image>, materials: &mut Assets<StandardMaterial>,
               effect: u32, recorded: bool, facing: bool) -> Option<Arc<Compiled>> {
        if let Some(c) = self.effects.get(&(effect, recorded, facing)) {
            return c.clone();
        }
        // (an effect that carries itself, through its effect appearances, carries nothing)
        self.effects.insert((effect, recorded, facing), None);
        let compiled = self.compile(game, images, materials, effect, recorded, facing).map(Arc::new);
        if compiled.is_none() {
            warn!("ALE effect h_{effect:08x}: not in the library, or nothing it draws is supported");
        }
        self.effects.insert((effect, recorded, facing), compiled.clone());
        compiled
    }

    fn compile(&mut self, game: &mut Game, images: &mut Assets<Image>, materials: &mut Assets<StandardMaterial>,
               effect: u32, recorded: bool, facing: bool) -> Option<Compiled> {
        // the game's particle shapes (see SHAPE, `Compiled::shapes`)
        let shapes = recorded && facing;
        let e = game.effects.effects.get(&effect)?.clone();
        let node = |game: &Game, i: u32| e.refs.iter().find(|r| r.3 == i).and_then(|r| game.effects.nodes.get(&r.1)).cloned();
        let node_name = |i: u32| e.refs.iter().find(|r| r.3 == i).map_or(0, |r| r.1);
        let mut pairs = vec![];
        for &(em, ap) in &e.pairs {
            let Some(app) = node(game, ap) else { continue };
            let emitter = match node(game, em) {
                Some(n) => n,
                None if recorded && STAND_IN_EMITTERS.contains(&node_name(em)) => missing_emitter(&app),
                None => continue,
            };
            if app.class == CLASS_SPAWNER {
                if !recorded {
                    continue;
                }
                let child = app.string(SPAWN_EFFECT).filter(|n| !n.is_empty())
                    .and_then(|n| self.load_as(game, images, materials, h(n), recorded, facing));
                if let Some(child) = child {
                    pairs.push(Pair { perp: false, axial: false, shape: 5, attached: emitter.flag(ATTACHED), streak: false, emitter, app, steps: vec![], fps: 0.0,
                                      frame: FrameMode::Play, beam: None, light: false, child: Some(child), fields: vec![], rise: None, vertical: false });
                }
                continue;
            }
            if app.class == CLASS_BEAM {
                let additive = adds(app.pair(BEAM_BLEND).unwrap_or(BLEND_ADD));
                let name = app.string(BEAM_TEXTURE).map(h).unwrap_or(0);
                // an animated texture (the Energy's bolts: "ARCb", a 4 x 4 sheet of arcs running
                // down its cells at 30 fps): one material per frame, the ribbon stepping through
                // them over its life (`beams`). Grenade effects only; elsewhere a beam's
                // flipbook name finds no texture, as before
                let book = if recorded { game.flipbooks.get(&name).cloned().filter(|b| !b.frames.is_empty()) } else { None };
                // (a ribbon's colour carries its alpha whatever its pair: the beam's own draw
                // isn't read, so `Source` isn't applied to it)
                let texture = self.texture(game, images, book.as_ref().map_or(name, |b| b.texture), additive, Source::Alpha);
                let frames: Vec<Affine2> = match &book {
                    Some(b) => b.frames.iter()
                        .map(|&[u0, v0, u1, v1]| Affine2::from_scale_angle_translation(Vec2::new(u1 - u0, v1 - v0), 0.0, Vec2::new(u0, v0))).collect(),
                    None => vec![Affine2::IDENTITY],
                };
                let mats: Vec<Handle<StandardMaterial>> = frames.into_iter().map(|uv| materials.add(StandardMaterial {
                    base_color: Color::WHITE, base_color_texture: texture.clone(), uv_transform: uv, unlit: true, double_sided: true, cull_mode: None,
                    fog_enabled: false, alpha_mode: if additive { AlphaMode::Add } else { AlphaMode::Blend }, ..default()
                })).collect();
                let material = mats[0].clone();
                let fps = book.map_or(0.0, |b| b.fps);
                pairs.push(Pair { perp: false, axial: false, shape: 5, attached: emitter.flag(ATTACHED), streak: false, emitter, app, steps: vec![mats], fps,
                                  frame: FrameMode::Play, beam: Some(material), light: false, child: None, fields: vec![], rise: None, vertical: false });
                continue;
            }
            if app.class != CLASS_APPEARANCE {
                continue;
            }
            if e.name.to_ascii_lowercase().starts_with("light_") {
                pairs.push(Pair { perp: false, axial: false, shape: 5, attached: emitter.flag(ATTACHED), streak: false, emitter, app, steps: vec![], fps: 0.0,
                                  frame: FrameMode::Play, beam: None, light: true, child: None, fields: vec![], rise: None, vertical: false });
                continue;
            }
            let blend = app.pair(BLEND).unwrap_or(BLEND_ADD);
            let additive = adds(blend);
            // an added particle's source factor (a blended one is drawn by its alpha)
            let source = if additive { Source::of(blend) } else { Source::Alpha };
            // a texture, or an animated one ("arcb": a 4 x 4 sheet at 30 fps)
            let name = app.string(TEXTURE).map(h).unwrap_or(0);
            let book = game.flipbooks.get(&name).cloned();
            let texture = self.texture(game, images, book.as_ref().map_or(name, |b| b.texture), additive, source);
            if texture.is_none() {
                warn!("{}: texture {:?} missing", e.name, app.string(TEXTURE));
            }
            let frames: Vec<Affine2> = match &book {
                Some(b) if !b.frames.is_empty() => b.frames.iter()
                    .map(|&[u0, v0, u1, v1]| Affine2::from_scale_angle_translation(Vec2::new(u1 - u0, v1 - v0), 0.0, Vec2::new(u0, v0))).collect(),
                _ => vec![Affine2::IDENTITY],
            };
            // The Xbox adds sprites to the picture's stored (gamma) values; taking an added
            // sprite's texels and colour as linear amounts adds about what the console did
            // (see play_fx.rs).
            let steps = (0..STEPS).map(|i| {
                let k = (i as f32 + 0.5) / STEPS as f32;
                // (SRCCOLOR: the colour squared, the texture's texels too; ONE and SRCCOLOR don't
                // read the alpha)
                let [r, g, b] = app.color(COLOR, 0.0, k).unwrap_or([1.0; 3]).map(|c| if source == Source::Colour { c * c } else { c });
                let a = if source == Source::Alpha { app.floats(ALPHA, 0.0, k).unwrap_or(1.0).clamp(0.0, 1.0) } else { 1.0 };
                frames.iter().map(|&uv| materials.add(StandardMaterial {
                    base_color: if additive { Color::linear_rgba(r, g, b, a) } else { Color::srgba(r, g, b, a) },
                    base_color_texture: texture.clone(), uv_transform: uv, unlit: true, double_sided: true, cull_mode: None,
                    fog_enabled: false, alpha_mode: if additive { AlphaMode::Add } else { AlphaMode::Blend }, ..default()
                })).collect()
            }).collect();
            let streak = app.flag(MOTION_BLUR);
            let frame = if recorded { FrameMode::of(&app) } else { FrameMode::Play };
            // the air fields linked to this appearance (pairs appearance -> field) that run (see
            // AIR_FIELDS; the effects compiled with the recorded rules: the grenades' and the
            // Therm Sweeper's trail)
            let fields = e.pairs.iter().filter(|p| recorded && p.0 == ap).filter_map(|p| node(game, p.1))
                .filter(|n| n.class == CLASS_AIR_FIELD && AIR_FIELDS.contains(&n.name.as_str())).collect();
            let rise = (recorded && !app.flag(PERP) && !streak && upright(&app)).then(|| offset_end(&app));
            // a spark (see SPARK_ASPECT): drawn as a streak along its motion, not a perp quad
            let spark = recorded && app.flag(PERP) && emitter.class == CLASS_CONE
                && emitter.curve(SPEED, 0.0, 0.0).unwrap_or(0.0) >= PERP_RADIAL_SPEED && app.floats(ASPECT, 0.0, 0.0).unwrap_or(1.0) > SPARK_ASPECT;
            // the game's facing 3, where the effect is compiled with it (see FACING; stored as a
            // u32, parameter type 4)
            let axial = facing && app.flag(PERP) && matches!(app.params.get(&FACING), Some(crate::bf::ale::Value::U32(3)));
            let (perp, streak) = if axial { (false, false) } else if spark { (false, true) } else { (app.flag(PERP), streak) };
            let vertical = recorded && VERTICAL_STREAKS.contains(&app.name.as_str());
            let shape = match app.params.get(&SHAPE) {
                Some(crate::bf::ale::Value::U32(k @ (2 | 3))) if shapes => *k,
                _ => 5,
            };
            pairs.push(Pair { perp, axial, shape, attached: emitter.flag(ATTACHED), streak, emitter, app, steps, fps: book.map_or(0.0, |b| b.fps), frame,
                              beam: None, light: false, fields, child: None, rise, vertical });
        }
        (!pairs.is_empty()).then_some(Compiled { name: e.name, pairs, recorded, facing, shapes, counted: shapes })
    }

    /// A particle texture: as sRGB for a blended appearance, as linear amounts for an added one
    /// (`linear`). For one added by ONE or SRCCOLOR (`source`) its alpha is made opaque, as the
    /// blend doesn't read it, and for SRCCOLOR its colour squared. Cached by all three. With the
    /// mip levels its archive ships (most particle textures in common.tgz have one), clamped at
    /// its edges as before.
    fn texture(&mut self, game: &mut Game, images: &mut Assets<Image>, id: u32, linear: bool, source: Source) -> Option<Handle<Image>> {
        self.textures.entry((id, linear, source)).or_insert_with(|| {
            let mut mips = game.texture_mips(id)?;
            if source != Source::Alpha {
                mips.map_texels(|p| {
                    if source == Source::Colour {
                        for c in &mut p[..3] {
                            *c = ((*c as u32 * *c as u32 + 127) / 255) as u8;
                        }
                    }
                    p[3] = 255;
                });
            }
            let format = if linear { TextureFormat::Rgba8Unorm } else { TextureFormat::Rgba8UnormSrgb };
            Some(images.add(mip_image(&mips, format, [Address::Clamp; 2])))
        }).clone()
    }
}

impl AleEffect {
    pub fn new(fx: Arc<Compiled>, sp: f32, seed: u32) -> Self {
        let n = fx.pairs.len();
        AleEffect { fx, active: true, sp, once: false, placed: false, prev: None, strips: vec![None; n], t: 0.0, acc: vec![0.0; n], started: false, rng: seed | 1,
                    hide_flat: false, lod_sphere: None, lod: 1.0, last: vec![0.0; n], ticks: 0 }
    }

    /// With its effect type's bounding sphere (`EffectType::bounds`), so its distance LOD is
    /// worked out from the camera (see LOD_TAN): it matters to the effects compiled with
    /// `Compiled::counted`.
    pub fn with_lod_sphere(mut self, sphere: Option<(Vec3, f32)>) -> Self {
        self.lod_sphere = sphere;
        self
    }

    /// Run the effect once (a hit, a shot's tracer): each emitter for its lifespan.
    pub fn once(fx: Arc<Compiled>, sp: f32, seed: u32) -> Self {
        AleEffect { once: true, ..Self::new(fx, sp, seed) }
    }

    fn random(&mut self) -> f32 {
        self.rng ^= self.rng << 13;
        self.rng ^= self.rng >> 17;
        self.rng ^= self.rng << 5;
        (self.rng >> 8) as f32 / (1u32 << 24) as f32
    }
}

/// A stand-in for an emitter the node library doesn't have (stun_grenade_master's h_ed10c55f
/// and h_f48dc74d, the emitters of stun_grenade_init.app and stun_grenade_init#1.app): one
/// particle at the effect's middle when it starts, living the appearance's own lifespan
/// (0.6 s, 0.45 s). A guess: the recording's cyan flash and ground wash, at the middle,
/// strongest at +0.17-0.28 s and gone by ~0.45 s, fit it. Only those two (STAND_IN_EMITTERS):
/// other grenade effects miss emitters too (exp-lrg-dirt's h_f6c66e90 in the Frag's and the
/// Roller's blast, exp-mine-dirt's h_ecb22a58 in the Sentry's), and drew nothing before; a
/// stand-in there would add a layer the Frag recording wasn't checked with.
fn missing_emitter(app: &Node) -> Node {
    use crate::bf::ale::{Curve, Value};
    let life = app.float(LIFESPAN).unwrap_or(1.0);
    Node { class: 0, name: format!("{} (its emitter is missing)", app.name), params: HashMap::from([
        (LIFESPAN, Value::Float(life)),
        (LIFE, Value::Curve(Curve(vec![(0.0, life, 0, vec![])]))),
    ]) }
}

/// Every key value of a float animation (all its sparam items).
fn float_keys(app: &Node, p: u32) -> Vec<f32> {
    match app.params.get(&p) {
        Some(crate::bf::ale::Value::Floats(items)) => items.iter().flat_map(|(_, _, keys)| keys.iter().map(|k| k.1)).collect(),
        _ => vec![],
    }
}

/// An upright streak (see UPRIGHT_WIDTH): its width factor under UPRIGHT_WIDTH of its smallest
/// height factor, that capped at 1.0, all its life, and a constant Rotate.
fn upright(app: &Node) -> bool {
    let (width, height, rotate) = (float_keys(app, WIDTH), float_keys(app, ASPECT), float_keys(app, ROTATE));
    let tall = height.iter().copied().fold(f32::MAX, f32::min).min(1.0);
    !width.is_empty() && width.iter().all(|&w| w < UPRIGHT_WIDTH * tall) && rotate.windows(2).all(|r| r[0] == r[1])
}

/// When an appearance transform's offset stops (the last key of its three translation curves,
/// s); read no later, so the curves' repeat flag doesn't take it back to the start.
fn offset_end(app: &Node) -> f32 {
    match app.params.get(&TRANSFORM) {
        Some(crate::bf::ale::Value::Transform(c)) => c.iter().take(3).flat_map(|c| c.0.iter().filter_map(|i| i.3.last().map(|k| k.0)))
            .fold(0.0, f32::max),
        _ => 0.0,
    }
}

/// A steady light: a "light_" effect's pair (grenade effects only, `Compiled::recorded`) whose
/// emitter has a constant rate (no keys) at which its particles overlap two deep or more: its
/// rate (/s). Each particle's light then rises over its first 1/rate s and falls over its last,
/// scaled by rate x life / (rate x life - 1), so the evenly spaced particles' sum stays constant
/// at the mean of the plain particles'. The one it picks out is light_phosphor (5.99/s, 0.514 s:
/// ~3 at once): as plain particles, the number lit stepped between 3 and 4 and the Light's ground
/// light pulsed by 3-11% about three times a second, where the recording's is steady within
/// 1/255 (the tester's measurement). The Frag's and the Sentry's light_explosion (one particle)
/// and the Sonic's light_sonic_grenade (a keyed rate) aren't steady and are lit as before. How
/// the console summed its lights isn't known: this keeps the data's mean.
fn steady_rate(fx: &Compiled, pair: &Pair) -> Option<f32> {
    if !fx.recorded || !pair.light {
        return None;
    }
    let constant = match pair.emitter.params.get(&RATE) {
        Some(crate::bf::ale::Value::Curve(c)) => c.0.len() == 1 && c.0[0].3.is_empty(),
        _ => false,
    };
    let rate = pair.emitter.curve(RATE, 0.0, 0.0).unwrap_or(0.0);
    let life = pair.emitter.curve(LIFE, 0.0, 0.0).unwrap_or(0.0);
    (constant && rate > 0.0 && rate * life >= 2.0).then_some(rate)
}

fn euler(deg: [f32; 3]) -> Quat {
    Quat::from_euler(EulerRot::XYZ, deg[0].to_radians(), deg[1].to_radians(), deg[2].to_radians())
}

/// A direction within `lo`..`hi` degrees of +y.
fn spread(fx: &mut AleEffect, lo: f32, hi: f32) -> Vec3 {
    let (lo, hi) = (lo.min(hi), lo.max(hi));
    let theta = (lo + (hi - lo) * fx.random()).to_radians();
    let phi = fx.random() * std::f32::consts::TAU;
    Vec3::new(theta.sin() * phi.cos(), theta.cos(), theta.sin() * phi.sin())
}

/// FUN_00065800, the game's count of an emitter's new particles for the game frame that ends at
/// the effect's time `at` (GAME_FRAME; the emitter's time from its last batch to then): the
/// whole number of its frequency's particles over that time (the frequency read at its end),
/// its initial particles on its first update (the flag the emitter's constructor FUN_00065640
/// sets), plus each emit count key that falls in it (cut to a whole number each time), then x
/// its LOD curve at the effect's LOD (EMIT_LOD), cut again. The fractions are thrown away: the
/// time keeps piling up only while nothing comes and the frequency isn't 0. So a low-rate
/// emitter makes far fewer particles than its rate (the launch dust's 18 a second for 0.43 s
/// give one card), and exp-rocket-flash's keys 3.59 at 0.076 s and 2.26 at 0.16 s give 2 and
/// 1. For the effects compiled with `Compiled::counted`. Returns the count and the emitter's
/// time it's born at.
fn game_count(fx: &mut AleEffect, i: usize, em: &Node, lifespan: f32, at: f32, fresh: bool) -> (usize, f32) {
    // the emitter's own time: once, up to its lifespan; else it starts its lifespan again, its
    // last batch's time going back to 0. The demo's choice: the game's loop of an emitter's time
    // isn't traced (it may stop it at its lifespan instead). The trail's 4 s emitters are the
    // only counted ones that loop, on a child's flight past 4 s: their rates start over at 0 for
    // 0.07-0.09 s, so the trail has a ~0.2 s gap there (nothing comes in the game frame of the
    // restart either, its rate being read at the new time)
    let t1 = if fx.once { at.min(lifespan) } else { at % lifespan };
    if t1 < fx.last[i] {
        fx.last[i] = 0.0;
    }
    let t0 = fx.last[i];
    let sp = fx.sp;
    let freq = em.curve(RATE, sp, t1).unwrap_or(0.0).max(0.0);
    let mut n = ((t1 - t0) * freq).max(0.0).trunc();
    if fresh {
        n += em.int(INITIAL).unwrap_or(0).max(0) as f32;
    }
    for (kt, kv) in em.curve_keys(BURST, sp) {
        if kt >= t0 && kt <= t1 {
            n = (n + kv).trunc();
        }
    }
    let n = (em.floats(EMIT_LOD, sp, fx.lod).unwrap_or(1.0) * n).trunc().max(0.0);
    if n > 0.0 || freq == 0.0 {
        fx.last[i] = t1;
    }
    (n as usize, t1)
}

fn clock(time: &Time, fixed: Option<Res<AleClock>>) -> f32 {
    fixed.map_or(time.delta_secs().min(0.1), |c| c.0)
}

#[allow(clippy::too_many_arguments)]
fn emit(mut commands: Commands, time: Res<Time>, fixed: Option<Res<AleClock>>, assets: Option<Res<AleAssets>>,
        mut count: ResMut<Count>, mut seq: ResMut<Seq>, mut meshes: ResMut<Assets<Mesh>>,
        mut effects: Query<(Entity, &mut AleEffect, &mut GlobalTransform, &Transform, Option<&ChildOf>)>,
        parents: Query<(Ref<GlobalTransform>, &Transform, Has<ChildOf>), Without<AleEffect>>,
        camera: Query<&GlobalTransform, (With<Camera3d>, Without<AleEffect>)>) {
    let Some(assets) = assets else { return };
    let dt = clock(&time, fixed);
    let eye = camera.iter().next().map(|c| c.translation());
    for (fx_entity, mut fx, mut at, local, parent) in &mut effects {
        let owner = fx_entity;
        // a new effect's world placement, set now (transforms propagate after this runs): one
        // without a parent is where its own transform says (a blast starts on the frame it goes
        // off), one with a parent at its own transform from the parent's last placement (a
        // muzzle effect and its light start on the shot's frame, a frame late before; the
        // parent's moved by a frame at most), so its attached particles and beam ribbons
        // (placed from it in `animate`) aren't drawn at the origin for that frame. A parent not
        // placed yet (made since this last ran: a missile and its trail, an effect particle's
        // carrier) makes it wait a frame, as every parented effect did before: a missile moves
        // from the frame after it's made, so its trail's time keeps with its flight
        if !fx.placed {
            match parent.map(|p| parents.get(p.parent())) {
                None => *at = GlobalTransform::from(*local),
                Some(Ok((g, _, _))) if !g.is_added() => *at = g.mul_transform(*local),
                Some(_) => {
                    fx.placed = true;
                    continue;
                }
            }
        }
        // a counted effect on a parent at the top (a missile's trail): from the parent's own
        // transform this frame, not its placement from the last frame's propagation, so the
        // particles of each game frame (GAME_FRAME) are born along the missile's move up to
        // where it is then; they were born a frame behind it (2 m at 15 fps). The game's world
        // update runs the physics step, which moves the missile (FUN_00169e20 at 0xd9ad4),
        // before the objects' ticks (FUN_000d9840 at 0xd9d0b), where the trail's (0x21b7a0)
        // follows the missile (FUN_0021a700) and births along its move since the last frame
        if let Some(Ok((_, t, false))) = parent.map(|p| parents.get(p.parent())) {
            if fx.fx.counted {
                *at = GlobalTransform::from(*t).mul_transform(*local);
            }
        }
        let place = at.compute_transform();
        fx.placed = true;
        // its distance LOD (see LOD_TAN)
        if let (Some((centre, radius)), Some(eye)) = (fx.lod_sphere, eye) {
            let d = LOD_TAN * ((eye - place.transform_point(centre)).length() - radius).max(0.0);
            fx.lod = ((LOD_FAR - d) / (LOD_FAR - LOD_NEAR)).clamp(0.0, 1.0);
        }
        if !fx.active {
            fx.started = false;
            continue;
        }
        let fresh = !fx.started;
        fx.started = true;
        let compiled = fx.fx.clone();
        // a counted effect's game frames this render frame: those its time reaches (GAME_FRAME)
        let mut ticks = vec![];
        if compiled.counted {
            // (1e-5 s of slack: the effect's time is a float sum of frame times, which can land
            // a hair short of a game frame due at this frame's end and put it off a frame)
            while (fx.ticks + 1) as f32 * GAME_FRAME <= fx.t + dt + 1e-5 {
                fx.ticks += 1;
                ticks.push(fx.ticks);
            }
        }
        let (from, from_turn) = fx.prev.unwrap_or((place.translation, place.rotation));
        let start = fx.t;
        // the share of this render frame's move (from the last place to this one) at the
        // effect's time `s`
        let share = |s: f32| ((s - start) / dt.max(1e-6)).clamp(0.0, 1.0);
        for (i, pair) in compiled.pairs.iter().enumerate() {
            let em = &pair.emitter;
            let lifespan = em.float(LIFESPAN).unwrap_or(1.0).max(0.01);
            let sp = fx.sp;
            let mut batches = vec![];
            if compiled.counted {
                // the game's count, a batch a game frame, born at its end and spread over its
                // part of the move
                for &k in &ticks {
                    let at = k as f32 * GAME_FRAME;
                    // (run once: nothing after the update that reached its lifespan)
                    if fx.once && at - GAME_FRAME >= lifespan {
                        break;
                    }
                    let (n, t) = game_count(&mut fx, i, em, lifespan, at, k == 1);
                    batches.push(Batch { n, t, span: GAME_FRAME, from: share(at - GAME_FRAME), to: share(at), age: start - at, born: at, carried: None });
                }
            } else {
                if fx.once && fx.t >= lifespan {
                    continue;
                }
                // a finite emitter starts over while the effect stays on
                let t = fx.t % lifespan;
                let mut rate = em.curve(RATE, sp, t).unwrap_or(0.0).max(0.0);
                let initial = em.int(INITIAL).unwrap_or(0).max(0) as usize;
                // no rate and no initial count: its burst when it starts (exp-lrg-flash's 4),
                // else one particle (a gun's tracer); an EMIT_COUNT_RATE emitter goes on emitting
                // its emit count per second after the burst
                let burst = if compiled.recorded { em.curve(BURST, sp, 0.0).unwrap_or(0.0).round().max(1.0) as usize } else { 1 };
                let over_time = compiled.recorded && initial == 0 && rate == 0.0 && EMIT_COUNT_RATE.contains(&em.name.as_str());
                if over_time {
                    rate = em.curve(BURST, sp, t).unwrap_or(0.0).max(0.0);
                }
                let mut n = if fresh { if over_time || (initial == 0 && rate == 0.0) { burst } else { initial } } else { 0 };
                let carried = fx.acc[i];
                fx.acc[i] += rate * dt;
                n += fx.acc[i] as usize;
                fx.acc[i] = fx.acc[i].fract();
                batches.push(Batch { n, t, span: dt, from: 0.0, to: 1.0, age: 0.0, born: fx.t, carried: Some(carried) });
            }
            for b in batches {
                let n = b.n;
                // a beam pair's ribbon, made with its first particles
                if let (true, Some(material), None) = (n > 0, &pair.beam, fx.strips[i]) {
                    let mesh = meshes.add(Mesh::new(bevy::render::mesh::PrimitiveTopology::TriangleList, RenderAssetUsages::default()));
                    // (never culled: its bounds would be those of its first few points, its mesh
                    // being rebuilt in the world every frame)
                    fx.strips[i] = Some(commands.spawn((Mesh3d(mesh.clone()), MeshMaterial3d(material.clone()), NotShadowCaster,
                        bevy::render::view::NoFrustumCulling, Transform::default(), Visibility::default(),
                        BeamStrip { source: owner, pair: i, fx: compiled.clone(), mesh, age: 0.0 })).id());
                }
                if n > 0 && std::env::var("BF_ALE_LOG").is_ok() {
                    println!("{} pair {i}: t {:.3} dt {dt:.4} emits {n} (alive {}) at {:.2}", compiled.name, b.born, count.0, place.translation);
                }
                for j in 0..n {
                    if count.0 >= MAX_PARTICLES {
                        break;
                    }
                    // spread over the batch: the emitter's own animated offset and the effect's
                    // movement over it (a fast trail or a beam drawn down the line)
                    let g = (j + 1) as f32 / n as f32;
                    let f = b.from + (b.to - b.from) * g;
                    let tj = (b.t - b.span * (1.0 - g)).max(0.0);
                    let t = b.t;
                    let (at, rot) = (from.lerp(place.translation, f), from_turn.slerp(place.rotation, f));
                    let [offset, turn, _] = em.transform(TRANSFORM, tj);
                    let frame = rot * euler(turn);
                    // (a grenade effect's offsets are scaled by its entity's scale: play_energy.rs
                    // fits a bolt's reach to the body it strikes so)
                    let offset = if compiled.recorded { Vec3::from(offset) * place.scale } else { Vec3::from(offset) };
                    let origin = at + rot * offset;
                    let speed = em.curve(SPEED, sp, t).unwrap_or(0.0);
                    let c = |p: u32| em.curve(p, sp, t).unwrap_or(0.0);
                    let (local, dir) = match em.class {
                        CLASS_SPHERE => {
                            let d = spread(&mut fx, 0.0, 180.0);
                            let (r0, r1) = (c(SPHERE_RADIUS[0]), c(SPHERE_RADIUS[1]));
                            (d * (r0 + (r1 - r0) * fx.random()), d)
                        }
                        CLASS_CONE => {
                            let d = spread(&mut fx, c(CONE_SPREAD[0]), c(CONE_SPREAD[1]));
                            let (r0, r1) = (c(CONE_RADIUS[0]), c(CONE_RADIUS[1]));
                            let flat = Vec3::new(d.x, 0.0, d.z).normalize_or_zero();
                            (flat * (r0 + (r1 - r0) * fx.random()), d)
                        }
                        _ => {
                            let size = Vec3::new(c(CUBE[0]), c(CUBE[1]), c(CUBE[2]));
                            let p = Vec3::new(fx.random() - 0.5, fx.random() - 0.5, fx.random() - 0.5) * size;
                            (p, spread(&mut fx, c(CUBE_SPREAD[0]), c(CUBE_SPREAD[1])))
                        }
                    };
                    let life = em.curve(LIFE, sp, t).unwrap_or(1.0).max(0.02);
                    // a particle carrying an effect lives no longer than that effect runs once
                    // (stun_grenade_master's 7.49 s particles carry 0.8 s bolts): it would carry an
                    // effect with nothing left to draw
                    let life = pair.child.as_ref().map_or(life, |c| life.min(c.duration()));
                    // (a facing-3 quad turns only by its Rotate, see FACING)
                    let roll = if pair.perp || pair.axial || pair.rise.is_some() { 0.0 } else { fx.random() * std::f32::consts::TAU };
                    let mut lie = frame * euler(pair.app.transform(TRANSFORM, b.born)[1]) * Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2);
                    if compiled.recorded && pair.perp && (em.class == CLASS_SPHERE || (em.class == CLASS_CONE && PERP_OUT_CONES.contains(&em.name.as_str())))
                        && speed >= PERP_RADIAL_SPEED {
                        // a perp quad from a sphere emitter (or a PERP_OUT_CONES cone) that
                        // throws its particles out: facing out along its direction from the
                        // middle (see PERP_RADIAL_SPEED)
                        lie = Quat::from_rotation_arc(Vec3::Z, (frame * dir).normalize_or(Vec3::Y));
                    }
                    if fx.hide_flat && pair.perp && (lie * Vec3::Z).dot(rot * Vec3::Y).abs() < 0.5 {
                        continue;
                    }
                    count.0 += 1;
                    // attached: kept in the effect's frame (offset and turn without the
                    // effect's own)
                    let (owner, local, vel, lie) = if pair.attached {
                        let inv = place.rotation.inverse();
                        (Some(owner), inv * (origin - place.translation) + inv * frame * local, inv * frame * dir * speed, inv * lie)
                    } else {
                        (None, Vec3::ZERO, frame * dir * speed, lie)
                    };
                    seq.0 += 1;
                    // a steady light's particles start at the moment within the frame the rate says
                    // they're due, so they come at even spacing (see `steady_rate`); a game frame's
                    // batch is born at its end
                    let age = match (steady_rate(&compiled, pair), b.carried) {
                        (Some(rate), Some(carried)) => -((j + 1) as f32 - carried) / rate,
                        _ => b.age,
                    };
                    // a game frame's batch, born before this render frame's end, starts back by
                    // its age (negative) along its way: `animate` moves every particle by the
                    // whole frame, so it stands where its age since its birth puts it
                    let back = if b.carried.is_none() { vel * age } else { Vec3::ZERO };
                    let (local, spawn) = match owner {
                        Some(_) => (local + back, origin + frame * local),
                        None => (local, origin + frame * local + back),
                    };
                    let particle = AleParticle { fx: compiled.clone(), pair: i, lod: fx.lod, age, life, vel, sp, born: b.born, roll, step: 0, frame: lie, owner, local,
                                                 turn: place.rotation, source: owner.unwrap_or(fx_entity), seq: seq.0 };
                    let at = Transform::from_translation(spawn).with_scale(Vec3::splat(0.001));
                    if let Some(child) = &pair.child {
                        // an effect appearance: the particle carries its effect, turned to face
                        // (+z) the way the particle goes along the ground and kept upright. The
                        // turn is an inference: the Energy recording's bolts run out every way
                        // from the middle, and the bolt's own reach runs along its +z
                        // (stun_grenade.emt's offset sweeps 0 -> 5.8 m along z)
                        let going = particle.vel;
                        let flat = Vec3::new(going.x, 0.0, going.z).normalize_or(Vec3::Z);
                        let carrier = commands.spawn((Transform::from_translation(at.translation).with_rotation(Quat::from_rotation_arc(Vec3::Z, flat)),
                                                      Visibility::default(), particle)).id();
                        let seed = fx.rng ^ (seq.0 as u32).wrapping_mul(0x9E37_79B9);
                        commands.spawn((Transform::default(), Visibility::default(), AleEffect::once(child.clone(), sp, seed), ChildOf(carrier)));
                    } else if pair.beam.is_some() {
                        // drawn by the ribbon, not as a quad
                        commands.spawn((at, particle));
                    } else if pair.light {
                        commands.spawn((at.with_scale(Vec3::ONE), particle,
                                        PointLight { intensity: 0.0, range: 0.5, shadows_enabled: false, ..default() }));
                    } else {
                        let quad = if pair.shape == 3 { assets.quad_base.clone() } else { assets.quad.clone() };
                        commands.spawn((Mesh3d(quad), MeshMaterial3d(pair.steps[0][0].clone()), NotShadowCaster, at, particle));
                    }
                }
            }
        }
        fx.t += dt;
        fx.prev = Some((place.translation, place.rotation));
    }
}

#[allow(clippy::type_complexity)]
fn animate(mut commands: Commands, time: Res<Time>, fixed: Option<Res<AleClock>>, mut count: ResMut<Count>,
           camera: Query<&GlobalTransform, With<Camera3d>>, owners: Query<&GlobalTransform, (With<AleEffect>, Without<AleParticle>)>,
           mut particles: Query<(Entity, &mut AleParticle, &mut Transform, Option<&mut MeshMaterial3d<StandardMaterial>>, Option<&mut PointLight>)>) {
    let dt = clock(&time, fixed);
    let view = camera.iter().next().map(|c| c.compute_transform()).unwrap_or_default();
    let (facing, eye) = (view.rotation, view.translation);
    for (e, mut p, mut tr, mat, light) in &mut particles {
        p.age += dt;
        if p.age >= p.life {
            commands.entity(e).despawn();
            count.0 = count.0.saturating_sub(1);
            continue;
        }
        let k = p.age / p.life;
        let fx = p.fx.clone();
        let pair = &fx.pairs[p.pair];
        let app = &pair.app;
        // its air fields: toward each one's wind (in the world's frame, see CLASS_AIR_FIELD)
        for field in &pair.fields {
            let t = p.born + p.age;
            let wind = euler(field.transform(TRANSFORM, t)[1]) * Vec3::Y * field.curve(AIR_MAGNITUDE, p.sp, t).unwrap_or(0.0);
            let approach = field.curve(AIR_APPROACH, p.sp, t).unwrap_or(0.0).clamp(0.0, 1.0);
            let k = 1.0 - (1.0 - approach).powf(dt * APPROACH_FPS);
            let v = p.vel;
            p.vel = v + (wind - v) * k;
        }
        // attached particles ride on their effect (left where they are if it's gone)
        let mut turn = Quat::IDENTITY;
        match p.owner.map(|o| owners.get(o)) {
            Some(Ok(o)) => {
                let o = o.compute_transform();
                let v = p.vel;
                p.local += v * dt;
                tr.translation = o.translation + o.rotation * p.local;
                turn = o.rotation;
                p.turn = turn;
            }
            Some(Err(_)) => {
                // the effect is gone: go on in the world from its last frame
                p.owner = None;
                p.vel = p.turn * p.vel;
                p.frame = p.turn * p.frame;
                p.turn = Quat::IDENTITY;
            }
            None => tr.translation += p.vel * dt,
        }
        // an upright streak rises by its appearance's offset over its age (see `Pair::rise`; in
        // the world's frame: the blasts are placed upright)
        if let Some(end) = pair.rise {
            let at = |t: f32| Vec3::from(app.transform(TRANSFORM, t.clamp(0.0, end * 0.9999))[0]);
            tr.translation += at(p.age) - at(p.age - dt);
        }
        let moving = turn * p.vel;
        // the appearance's LOD curve at its effect's LOD (APP_LOD), on its size
        let lod_scale = if p.fx.counted { app.floats(APP_LOD, p.sp, p.lod).unwrap_or(1.0) } else { 1.0 };
        // a light's: reach = size, strength = colour x alpha x size
        if let Some(mut light) = light {
            let size = app.floats(SIZE, p.sp, k).unwrap_or(1.0).max(0.0) * lod_scale;
            let a = app.floats(ALPHA, p.sp, k).unwrap_or(1.0).clamp(0.0, 1.0);
            let c = Vec3::from(app.color(COLOR, p.sp, k).unwrap_or([1.0; 3]));
            let peak = c.max_element();
            // a steady light's particles fade in and out over one spacing (see `steady_rate`)
            let steady = match steady_rate(&fx, pair) {
                Some(rate) => {
                    let gap = 1.0 / rate;
                    let ramp = (p.age.max(0.0) / gap).min((p.life - p.age) / gap).clamp(0.0, 1.0);
                    ramp * rate * p.life / (rate * p.life - 1.0)
                }
                None => 1.0,
            };
            light.intensity = LIGHT_LUMENS * a * size * peak * LIGHT_SCALE * steady;
            light.range = size.max(0.1);
            if peak > 1e-4 {
                light.color = Color::srgb(c.x / peak, c.y / peak, c.z / peak);
            }
            continue;
        }
        let Some(mut mat) = mat else { continue };           // a beam's: the ribbon draws it
        let size = app.floats(SIZE, p.sp, k).unwrap_or(1.0).max(0.0) * lod_scale;
        let width = (size * app.floats(WIDTH, p.sp, k).unwrap_or(1.0)).max(0.001);
        let height = (size * app.floats(ASPECT, p.sp, k).unwrap_or(1.0)).max(0.001);
        let spin = Quat::from_rotation_z(p.roll + app.floats(ROTATE, p.sp, k).unwrap_or(0.0));
        let aspect = app.floats(ASPECT, p.sp, k).unwrap_or(1.0);
        tr.rotation = if pair.axial {
            // facing 3 (see FACING): its height along its motion (+z of its frame if it has
            // none: its effect's for an attached particle, the world's otherwise, a
            // simplification of the shader's particle frame), its width square to that and to
            // the eye, so it faces the camera as much as it can, then turned in that plane by
            // its Rotate (the vertex shader's two axes and sine / cosine, 0x3b8bf8)
            let along = if moving.length_squared() > 1e-12 { moving.normalize() } else if p.owner.is_some() { turn * Vec3::Z } else { Vec3::Z };
            let side = along.cross(eye - tr.translation);
            let side = if side.length_squared() > 1e-12 { side.normalize() } else { along.any_orthonormal_vector() };
            let (s, c) = app.floats(ROTATE, p.sp, k).unwrap_or(0.0).sin_cos();
            let (x, y) = (side * c - along * s, side * s + along * c);
            Quat::from_mat3(&Mat3::from_cols(x, y, x.cross(y)))
        } else if pair.perp {
            turn * p.frame * spin
        } else if pair.vertical && !pair.streak {
            // standing up, turned about the vertical to face the camera (VERTICAL_STREAKS)
            let d = facing * Vec3::Z;
            let flat = Vec3::new(d.x, 0.0, d.z).normalize_or(Vec3::Z);
            Quat::from_rotation_y(flat.x.atan2(flat.z))
        } else if (pair.streak || aspect > 1.5) && moving.length_squared() > 1.0 {
            // stretched along the motion as seen from the camera
            let v = facing.inverse() * moving;
            facing * Quat::from_rotation_z(v.y.atan2(v.x) - std::f32::consts::FRAC_PI_2)
        } else {
            facing * spin
        };
        // (the game's shape 2 spans -1..1 of its Scale, see SHAPE)
        let across = if pair.shape == 2 { 2.0 } else { 1.0 };
        tr.scale = Vec3::new(width * across, height * across, 1.0);
        let step = ((k * STEPS as f32) as usize).min(STEPS - 1);
        let frames = pair.steps[step].len();
        let frame = match pair.frame {
            FrameMode::Play => (p.age * pair.fps) as usize % frames,
            FrameMode::Hold(v) => ((v * frames as f32) as usize).min(frames - 1),
            FrameMode::Life => ((app.floats(TEX_FRAME, p.sp, k).unwrap_or(0.0).clamp(0.0, 1.0) * frames as f32) as usize).min(frames - 1),
        };
        if step * 1000 + frame != p.step {
            p.step = step * 1000 + frame;
            mat.0 = pair.steps[step][frame].clone();
        }
    }
}

/// Beam ribbons: each joins its effect's live particles of its pair, oldest to newest, as a
/// strip facing the camera, with the appearance's width and colour x alpha at each particle's
/// age. A ribbon goes when its particles have.
fn beams(mut commands: Commands, time: Res<Time>, fixed: Option<Res<AleClock>>, mut meshes: ResMut<Assets<Mesh>>,
         camera: Query<&GlobalTransform, With<Camera3d>>, particles: Query<(&AleParticle, &Transform)>,
         effects: Query<(), With<AleEffect>>, mut strips: Query<(Entity, &mut BeamStrip, &mut MeshMaterial3d<StandardMaterial>)>) {
    let dt = clock(&time, fixed);
    let eye = camera.iter().next().map(|c| c.translation()).unwrap_or_default();
    let mut points: HashMap<(Entity, usize), Vec<(u64, Vec3, f32, f32)>> = HashMap::new();
    for (p, tr) in &particles {
        if p.fx.pairs[p.pair].beam.is_some() {
            points.entry((p.source, p.pair)).or_default().push((p.seq, tr.translation, p.age / p.life, p.sp));
        }
    }
    for (e, mut strip, mut material) in &mut strips {
        strip.age += dt;
        // an animated beam texture steps through its frames at its own rate
        let pair = &strip.fx.pairs[strip.pair];
        if let Some(frames) = pair.steps.first().filter(|f| f.len() > 1) {
            let m = &frames[(strip.age * pair.fps) as usize % frames.len()];
            if material.0 != *m {
                material.0 = m.clone();
            }
        }
        let mut pts = points.remove(&(strip.source, strip.pair)).unwrap_or_default();
        // the ribbon goes when its effect has and its particles have died
        if pts.is_empty() && strip.age > 0.2 && effects.get(strip.source).is_err() {
            commands.entity(e).despawn();
            continue;
        }
        pts.sort_by_key(|p| p.0);
        // particles born together at one spot (several a frame from a still emitter) make one
        // point of the ribbon
        pts.dedup_by(|b, a| a.1.distance_squared(b.1) < 1e-4);
        let app = &strip.fx.pairs[strip.pair].app;
        let (mut pos, mut col, mut uv, mut idx) = (vec![], vec![], vec![], vec![]);
        if std::env::var("BF_ALE_LOG").is_ok() && !pts.is_empty() {
            let (f, l) = (pts[0], pts[pts.len() - 1]);
            println!("  ribbon {} pair {}: {} points {:.1} .. {:.1}, width {:.2} alpha {:.2}", strip.fx.name, strip.pair, pts.len(), f.1, l.1,
                     app.floats(BEAM_WIDTH, f.3, f.2).unwrap_or(-1.0), app.floats(BEAM_ALPHA, f.3, f.2).unwrap_or(-1.0));
        }
        let n = pts.len();
        for (i, &(_, p, k, sp)) in pts.iter().enumerate() {
            let prev = pts[i.saturating_sub(1)].1;
            let next = pts[(i + 1).min(n - 1)].1;
            let along = (next - prev).normalize_or_zero();
            let side = along.cross((eye - p).normalize_or_zero()).normalize_or_zero();
            let w = app.floats(BEAM_WIDTH, sp, k).unwrap_or(0.2).max(0.0) * 0.5;
            let [r, g, b] = app.color(BEAM_COLOR, sp, k).unwrap_or([1.0; 3]);
            let a = app.floats(BEAM_ALPHA, sp, k).unwrap_or(1.0).clamp(0.0, 1.0);
            let u = if n > 1 { i as f32 / (n - 1) as f32 } else { 0.0 };
            // the beam textures run along v (as the `tracer` sprite, stretched in y, shows)
            for (s, across) in [(-1.0, 0.0), (1.0, 1.0)] {
                pos.push((p + side * w * s).to_array());
                col.push([r, g, b, a]);
                uv.push([across, u]);
            }
            if i + 1 < n {
                let j = (i * 2) as u32;
                idx.extend_from_slice(&[j, j + 1, j + 2, j + 1, j + 3, j + 2]);
            }
        }
        if let Some(mesh) = meshes.get_mut(&strip.mesh) {
            mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, pos);
            mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, col);
            mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uv);
            mesh.insert_indices(bevy::render::mesh::Indices::U32(idx));
        }
    }
}

/// Start the idle effects of the level's placed objects (the power-ups' icons: object type ->
/// idle effect type -> its ALE effects). Returns how many started.
pub fn spawn_idle_effects(commands: &mut Commands, game: &mut Game, level: &crate::bf::level::Level, assets: &mut AleAssets,
                          images: &mut Assets<Image>, materials: &mut Assets<StandardMaterial>) -> usize {
    let mut n = 0;
    for (i, o) in level.objects.iter().enumerate() {
        let Some(list) = game.idle_effects.get(&o.kind).and_then(|t| game.effect_types.get(t)).cloned() else { continue };
        for effect in list {
            if let Some(fx) = assets.load(game, images, materials, effect) {
                n += 1;
                if std::env::var("BF_LEVEL_DUMP").is_ok() {
                    println!("idle effect {} at {:.2}", fx.name, o.transform.w_axis.truncate());
                }
                commands.spawn((Transform::from_matrix(o.transform), Visibility::default(),
                                AleEffect::new(fx, 0.0, 0x9E37_79B9 ^ (i as u32).wrapping_mul(0x85EB_CA6B)),
                                Name::new(format!("idle effect h_{effect:08x}"))));
            }
        }
    }
    n
}
