//! Deathmatch classes (#394): a character type made from a campaign type by the rule the squad
//! deathmatch types follow, for the characters the game's multiplayer has no type for.
//!
//! The shipped game's 20 deathmatch characters each have a type of their own in mp_common's
//! libraries (`dm`, `sdm_bf` ... `sdm_ferg`), and each differs from the campaign version of the
//! same character in the same few fields (the analyst's diff of all 20 against every campaign
//! copy, `todo/drafts/done/sdm-classes-kingman-monsters.md` section 1.3; `rule_gaps` and
//! `character_type_report` check it on the loaded data, the `dm` library's 20 types and the `sdm_*`
//! libraries' 20 alike):
//!
//!  - **hit points** by a tier of the role, 45 - 95, not a ratio of the campaign's;
//!  - **damage-multiplier** 1 (the Shrike Hound's species 1.8 aside) and the player flag
//!    h_e78efbb2 true (`CharacterType::player_type`, read by FUN_00190330: the stamina update
//!    FUN_0011a840 runs for it, the AI's speed factor 0.8 FUN_0012cb40 and the lobbed-gun flag
//!    FUN_00016ff0 go by it);
//!  - **no vulnerabilities and no shield** (15 of the 20 have none; the Hunter Lord, the Shrike
//!    Heavy and Soldier and the two hounds keep a species-defining factor, and the three Shrikes
//!    a stamina shield with max-stamina 28 - 45: the dump's "rule gaps");
//!  - **the HUD slot** h_072ab4af 1 - 4 (FUN_000c84e0: Tex 1 top, Flint 2 right, Brutus 3
//!    bottom, Hawk 4 left), each once in a squad, with **the slot's skull** (skull_tex, _flint,
//!    _brutus, _hawk) and **a portrait** of the character's own. The same h_072ab4af is the
//!    footstep enum: the type's set of footsteps is its slot's (play.rs `spawn_unit` takes
//!    `footstep_type` from the type, `step_player` plays `surface.footsteps[footstep_type]`), so a
//!    member steps with Tex's, Flint's, Brutus's or Hawk's, as the sdm types do;
//!  - **radar-range** 60 m or more, **kill reward** 0, **max-stamina** 100 and the skill block's
//!    numbers kept where the type has no skill of its own;
//!  - **speeds** Tex's 3 / 3.6 / 7.8 / 8.5 for a human on ms_militia (the Ferguson four), a
//!    species' own where it has its own clips;
//!  - **the camera**: a player camera of 2.9 - 4 m (walk, run, dash, ready and h_e42d50a5 equal,
//!    dead 3.5 5 0 or the Hunter Lord's 4 5 0, snipe 0.001 0 0), not the campaign enemies' 1.55 m at
//!    10 degrees: the common set `3.2 1 0` on nine of the 20 types (the Ferguson four, Flint, the
//!    Shrike Heavy and Soldier, the hounds), the species' own on the rest (Tex 3.9 at 0.9, Brutus
//!    and the ferals 3.4 at 7, Hawk 2.9 at 1.5, the Follower 4 at 8, the Priest 3.5 at 3, the
//!    Shadoon 4 at 6, the Hunter Lord 4 at 5). A class takes the common set, all eight of them (the
//!    two bosses too: the product owner chose the rule's camera over the Hunter Lord's);
//!  - **the gun** the campaign type carries, with its ammo item.
//!
//! A `Class` is that rule for one character, as a record: the campaign type it starts from
//! (`base`, as `Game::find_character_type` names it), the values that are the class's own
//! (hit points, face, speeds, ammo, and its home slot) and where each came from (`Origin`).
//! `Class::apply` makes the type, `Game::add_classes` adds the loaded ones to
//! `Game::character_types` under the class's key with the place "demo/<library>"
//! (`CLASS_ARCHIVE`), so the NPC and squad menus, `BF_TEST_NPC`, the HUD's portraits, the damage
//! and the dump read a class as they read any type. Everything else stays the base type's: model,
//! clips, face block, chatter, capsule, decals, death effect, AI block, weapons (#357's chatter
//! rule; the AI block is the open difference, below).
//!
//! **Squads and seats.** The slot is a property of the type (above), but the demo's own squads
//! seat a character per squad: the Filthy Mutant is at the left of the Mutant Horde (slot 4) and at
//! the bottom of the Kingman Posse (slot 3). A `Squad` lists four `Seat`s (a class, the slot it
//! takes there, and why). A class has one home squad, `Class::library`: its own key's type is that
//! squad's member. In another squad the same character is a type of its own with the squad's
//! library and the seat's slot (`Class::apply_seated`, the key `<class key>@<library>`: the game
//! does the same, each squad's library having its own type of every member, with the slot in it),
//! so its place on the radar, its skull and its footsteps all follow the seat.
//!
//! The classes of #394 are the eight characters of the demo's three own squads (the Terrorist
//! Cell, the Kingman Posse and the Mutant Horde, play_testsquad.rs); #395 adds the cut characters
//! the data has only pictures for. **Real** values are the data's (the base type's own, or the cut
//! roster's remains); **rule** values are the demo's by the rule above; **guess** values are the
//! demo's with no rule behind them. The rule and the guesses are demo-own.
//!
//! What a class does not copy from the sdm types, because the rule table doesn't list it: their
//! shared AI block (alignment 0, vision 80, half-fov 90 ...: the demo's squad AI is its own).

use super::character::{CameraOffsets, Game};
use super::character_type::CharacterType;
use super::hash::h;

/// The archive a class is "defined" in (`CharacterType::archive`): the demo, which makes it. A
/// class's place in the menus and the dump is "demo/<its library>", never the base's archive.
pub const CLASS_ARCHIVE: &str = "demo";

/// How far (m) a class's radar reaches round it: `<h_12501487 radar-range>` 60, as the humans',
/// ferals' and Seer Follower's of the 20 sdm types and the squad's (80 the Hunter Lord and Shrike
/// Heavy, 130 the shamans and the Priest, 150 the Shadoon, 180 the hounds). The product owner's
/// brief gave 60 for all eight. `SkillData::radar_range`, read by play_hud.rs `radar_reach`.
pub const RADAR_RANGE: f32 = 60.0;

/// The skull a dead member's place shows, by HUD slot 1 - 4 (`h_00c51907`, the parse record's
/// +0x180: FUN_00190330): skull_tex, skull_flint, skull_brutus, skull_hawk, the same on all 20
/// sdm types by their slot (`BF_DUMP_TYPES=all`'s rule gaps check it). Textures of common's
/// library, found by name hash: h_fe6fa2af, h_18343e47, h_19da7869, h_17bd5b5c.
pub const SKULLS: [u32; 4] = [h("skull_tex.tga"), h("skull_flint.tga"), h("skull_brutus.tga"), h("skull_hawk.tga")];

/// The band the hit points of the 20 sdm types fall in (45 the two hounds, 95 the Shadoon): a
/// class is a tier inside it. `rule_gaps` checks it.
pub const HITPOINT_BAND: (f32, f32) = (45.0, 95.0);

/// Tex's speeds (crouch walk, walk, run, dash: `<h_14708eb2 h_113d3e3c>` of the sdm Ferguson
/// four on ms_militia, 3 / 3.6 / 7.8 / 8.5, and Tex's own): what a human of the militia model
/// gets in the rule, the campaign's 1.18 / 3 / 5.5 / 6.8 replaced.
pub const MILITIA_SPEEDS: [f32; 4] = [3.0, 3.6, 7.8, 8.5];

/// The band the walk boom (`CameraOffsets::walk[0]`, m) of the 40 deathmatch types (the `dm`
/// library's 20 and the `sdm_*` ones' 20) lies in: 2.9 Hawk's, 4 the Hunter Lord's, Shadoon's and
/// Follower's. A class's camera is inside it (`rule_gaps`); the campaign enemies' is 1.55 m.
pub const CAMERA_BOOM_BAND: (f32, f32) = (2.9, 4.0);
/// The band the dead camera's first value (`CameraOffsets::dead[0]`) of the 40 deathmatch types
/// lies in: 3.5 on 39 of them (`3.5 5 0`), 4 on the Hunter Lord (`4 5 0`).
pub const CAMERA_DEAD_BAND: (f32, f32) = (3.5, 4.0);

/// A camera block as the deathmatch types carry it (`<h_0d41e5f1>`, FUN_0012eca0): `boom` m at
/// `elevation` degrees for the five live modes (walk, run, dash, ready and h_e42d50a5 are equal on
/// every one of the 40), `dead` m at 5 degrees for the death camera (`dead 5 0`), and the snipe
/// offset `0.001 0 0` they all share.
pub const fn camera(boom: f32, elevation: f32, dead: f32) -> CameraOffsets {
    let live = [boom, elevation, 0.0];
    CameraOffsets { walk: live, run: live, dash: live, ready: live, h_e42d50a5: live, dead: [dead, 5.0, 0.0], snipe: [0.001, 0.0, 0.0] }
}

/// The common camera: `3.2 1 0` alive, `3.5 5 0` dead. Nine of the 20 sdm types carry it (the
/// Ferguson four on ms_militia and ms_mctavish, Flint, the Shrike Heavy and Soldier, the Shrike and
/// Fire Hounds), as do the `dm` library's nine. What the rule gives **every** class (rule): the
/// three militia-model terrorists the same values as the sdm Militia's (the same model), the five
/// others (the two bosses and the three mutants) having no sdm type of their model. The product
/// owner chose it for the bosses, Edward Kingman and the Supermutant, over the sdm Hunter Lord's
/// camera (h_0959b0fc of sdm_shrike: `4 5 0` alive and `4 5 0` dead, the sdm boss of the nearest
/// body, capsule r 0.45 x 2.1 against their 0.5 x 2.0 and 0.5 x 1.9), the other candidate, which
/// the demo doesn't use.
pub const CAMERA_COMMON: CameraOffsets = camera(3.2, 1.0, 3.5);

/// The High ROF Ammo item (objecttypes `<inventory>` entry with function-type 14 and ammo-type
/// h_e5f51266 = 2, name h_02735ad0, stack-limit 600): the reserve of the Kman Auto and the
/// MK-ASLT, which the sdm Militia and Shrike Soldier carry 120 of.
pub const HIGH_ROF_AMMO: u32 = 0x0273_5AD0;
/// The Kman Auto (weapon-type 0, a pistol; ammo-type 2, clip 35): Edward Kingman's gun, whose archetype
/// name hash is `CharacterType::weapons`' 0xEAE7F2E3 on his campaign type (what `BF_TEST_GUN` prints
/// for it), and so his second gun too (#407, the product owner's request).
pub const KMAN_AUTO: u32 = 0xEAE7_F2E3;
/// The Bio Ammo item (ammo-type 8, name h_01c1965b, stack-limit 200): the MNR-7 Bio's. No sdm
/// type carries any.
pub const BIO_AMMO: u32 = 0x01C1_965B;

/// Where a value of a class came from.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Origin {
    /// the game's data: the base type's own value kept, or the cut roster's remains
    Real,
    /// the demo's, by the rule of the 20 sdm types (see the module notes)
    Rule,
    /// the demo's, with no rule behind it
    Guess,
}

impl Origin {
    /// "real", "rule" or "guess", for the logs.
    pub fn word(self) -> &'static str {
        match self {
            Origin::Real => "real",
            Origin::Rule => "rule",
            Origin::Guess => "guess",
        }
    }
}

/// A value of a class, where it came from and why: (the value's name, its origin, the reason).
pub type Why = (&'static str, Origin, &'static str);

/// A skill the demo gives a class in place of its base type's, set by the product owner and not
/// by the game's data (#407): `Class::apply_seated` writes these into the type's skill block (its
/// skill-set, cost, recovery, start and end sounds, and the type's `max-stamina` too) and keeps the
/// rest of the block (the radar range, the hazards and the hit sound).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SkillOverride {
    /// the `skill-set` (default.xbe's CSKILL_ table 0x3bd814)
    pub skill_set: i64,
    /// `stamina-cost`
    pub cost: f32,
    /// `stamina-recovery`
    pub recovery: f32,
    /// `max-stamina`: the stamina at spawn and its cap
    pub max_stamina: f32,
    /// the `start-skill` and `end-skill` sounds, 0 for none
    pub sounds: (u32, u32),
}

/// A gun the demo adds to a class's starting weapons (#407, the product owner's request), with the
/// ammo item its ammo-type draws on where the class doesn't carry one already.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ExtraGun {
    /// the weapon's archetype name hash, as `CharacterType::weapons` lists them
    pub weapon: u32,
    /// the ammo item (objecttypes `<inventory>`, function-type 14) and how many of it the class
    /// carries; None where `Class::ammo` has it (Kingman's second Kman Auto draws on his High ROF Ammo)
    pub ammo: Option<(u32, i64)>,
}

/// The guns the demo's five armed classes carry (#421), by the object type name of each weapon's
/// data entry: the `<base name>` of the entry named by its stringtable name, which is the key
/// `CharacterType::weapons` lists (the Kman Auto's h_19d816bc -> h_eae7f2e3 checks the reading).
/// Ion Beamer h_fb0a72d0 -> h_ea241276 (e16's objecttypes-e16.xml line 5872), Saryl-45
/// h_fe7d9105 -> h_1c8e42e0 (line 6050), Bower 20 h_fd5e544f -> h_fa9e169d (line 6005), Foley
/// 356 Tact h_173f0051 -> h_e93533d1 (e01's line 5178), Jax-iP h_f71e9371 -> h_fc8af8aa (e23's
/// line 6791).
pub const ION_BEAMER: u32 = 0xEA24_1276;
pub const SARYL_45: u32 = 0x1C8E_42E0;
pub const BOWER_20: u32 = 0xFA9E_169D;
pub const FOLEY_356: u32 = 0xE935_33D1;
pub const JAX_IP: u32 = 0xFC8A_F8AA;

/// Their ammo items, and what a class carries of each (guess: half the item's stack, about the
/// share the sdm types carry): the 11mm (h_12302d03, stack 50), Sonic h_ef0c4c1e (200), Particle
/// h_fc0d262e (200), Shotgun h_e5d6b39e (80) and Energy h_15c866a4 (200) Ammo
/// (`docs/pickups/ammo-boxes.md`).
pub const AMMO_11MM: u32 = 0x1230_2D03;
pub const SONIC_AMMO: u32 = 0xEF0C_4C1E;
pub const PARTICLE_AMMO: u32 = 0xFC0D_262E;
pub const SHOTGUN_AMMO: u32 = 0xE5D6_B39E;
pub const ENERGY_AMMO: u32 = 0x15C8_66A4;

/// A deathmatch class: one character made by the rule (see the module notes).
#[derive(Clone, Copy, Debug)]
pub struct Class {
    /// its name, hashed into its type's key: the cut roster's archetype name where the data
    /// keeps one (`mp_edward_kingman` h_1b39dea1, `mp_terrorist`, `mp_subversive_marine`
    /// h_1f330430, `mp_supermutant`: the unlock flags of default-global-gameflags.xml's
    /// `<h_1d604469>` group, which have no `<character>`), the demo's own for the rest. What
    /// `BF_TEST_NPC` and `Game::find_character_type` take.
    pub key: &'static str,
    /// the campaign type it starts from, as `Game::find_character_type` names it
    /// ("edward kingman@m03_c")
    pub base: &'static str,
    /// its home squad's library: the squad whose member its own key's type is (the place reads
    /// "demo/<library>"). In another squad it is a type of its own (`Class::apply_seated`)
    pub library: &'static str,
    /// its hit points: a tier of the sdm types' band (`HITPOINT_BAND`)
    pub hitpoints: f32,
    /// its home slot 1 - 4, `h_072ab4af` of its own key's type: the place round the radar it
    /// takes (play_testsquad.rs `seat`) and its skull (`SKULLS`). The same h_072ab4af is the
    /// footstep enum, so the member steps with its slot's set (Tex's for 1, Flint's 2, Brutus's
    /// 3, Hawk's 4: play.rs `spawn_unit` reads the type's `footstep_type`, `step_player` plays
    /// `surface.footsteps[footstep_type]`), as the sdm types do. Its seat in a squad
    /// (`Seat::slot`) may differ; that makes a type of its own with that slot
    pub slot: i64,
    /// its portrait: a face texture of common's library (32 x 32), None for the "?"
    pub face: Option<&'static str>,
    /// its speeds when they aren't the base's (crouch walk, walk, run, dash)
    pub speeds: Option<[f32; 4]>,
    /// the ammo item its gun uses and how many it carries, added to the base's items
    pub ammo: Option<(u32, i64)>,
    /// a second gun the demo adds to the base's weapons, with its ammo item (`ExtraGun`), None for
    /// the base's guns alone: Edward Kingman's only
    pub extra_gun: Option<ExtraGun>,
    /// the guns it carries in place of the base type's (`CharacterType::weapons`'s archetype name
    /// hashes), with the ammo item `ammo` for them, None for the base's own: the five classes #421
    /// arms
    pub guns: Option<&'static [u32]>,
    /// its camera block (`CharacterType::camera`: the follow camera's boom is its walk, the death
    /// camera's framing its dead): the deathmatch types' 2.9 - 4 m, not the campaign enemies' 1.55 m
    /// at 10 degrees. `CAMERA_COMMON` for every class, by the rule (the sdm Militia's values for the
    /// militia model, the common set for the rest)
    pub camera: CameraOffsets,
    /// a skill of the demo's choosing in place of the base type's (`SkillOverride`), None for the
    /// base's own skill block, kept: every class but the Red Hand and the Subversive (the Orbital
    /// Beam), Edward Kingman (the Berserker), #407, and the Terrorist (Gunthar Ghent's Cloak), #205
    pub skill: Option<SkillOverride>,
    /// where each of those values came from (`report` prints them); the slot's reason is the
    /// seat's (`Seat::why`)
    pub why: &'static [Why],
}

/// The columns of the rule every class gets, whatever its own values (`Class::apply`), with where
/// each came from. Printed after a class's own values by `Class::report`.
pub const RULE: [Why; 8] = [
    ("damage-multiplier", Origin::Real, "1: the campaign type's own (every campaign enemy's) and 19 of the 20 sdm types'"),
    ("player flag h_e78efbb2", Origin::Rule, "true on all 20 sdm types"),
    ("vulnerabilities", Origin::Rule, "none: the campaign type's stripped, as on 15 of the 20 sdm types (the Shrikes and hounds keep one)"),
    ("shield", Origin::Real, "none: no campaign enemy of the eight has one"),
    ("radar-range", Origin::Rule, "60 m: the sdm humans' and ferals' (the product owner's brief for all eight)"),
    ("kill reward", Origin::Rule, "0: all 20 sdm types"),
    ("skull", Origin::Rule, "the slot's, skull_tex / _flint / _brutus / _hawk for 1 - 4, on all 20 sdm types"),
    ("skill block", Origin::Real, "the base type's skill-set, stamina cost and recovery and max-stamina 100, kept"),
];

/// Edward Kingman, the boss of the Terrorist Cell (his home) and the Kingman Posse: the campaign
/// type of m03_c, Kingman's own level (311 hp), with:
///  - hit points 95 (rule: the boss tier, the Shadoon's; the product owner's choice);
///  - home slot 1, Tex's place at the top (rule: the boss at the top, as the Shadoon and the
///    Hunter Lord; the product owner's choice);
///  - face_kingman.tga (real: one of the cut faces the data keeps in common's library);
///  - Tex's Berserker, CSKILL_TEX 3, cost 6.5, recovery 1.2, the deathmatch Tex's (guess: demo-own,
///    the product owner's request, #407; the campaign type's CSKILL_KINGMAN 5, the briefcase of
///    #201, isn't built);
///  - speeds 0 / 1.41 / 4.53 / 6.2 kept (guess: no sdm type is of his model; the sdm bosses
///    run 5 - 5.5 and dash 7 - 7.2, the Hunter Lord and the Shadoon, who keeps the campaign's
///    walk and trims its run and dash from 6 / 7.5);
///  - a second Kman Auto (guess: demo-own, the product owner's request, #407; the campaign type
///    carries one), the same ammo item (the High ROF Ammo below);
///  - the Kman Auto (real), 5 Gas grenades (real: `h_e8c67904`), and 120 High ROF Ammo, the
///    Kman Auto's ammo-type 2 (rule: the campaign gun plus its ammo item; guess: 120, the sdm
///    Militia's and Shrike Soldier's count);
///  - the camera 3.2 m at 1 degree alive and 3.5 m dead (rule: the common set, `CAMERA_COMMON`; no
///    sdm type is of his model, and the product owner chose it over the sdm Hunter Lord's 4 m at
///    5), not the campaign type's 1.55 m at 10.
pub const KINGMAN: Class = Class {
    key: "mp_edward_kingman",
    base: "edward kingman@m03_c",
    library: "terrorist_cell",
    hitpoints: 95.0,
    slot: 1,
    face: Some("face_kingman.tga"),
    speeds: None,
    ammo: Some((HIGH_ROF_AMMO, 120)),
    extra_gun: Some(ExtraGun { weapon: KMAN_AUTO, ammo: None }),
    guns: None,
    camera: CAMERA_COMMON,
    skill: Some(BERSERKER_DEMO),
    why: &[
        ("name", Origin::Real, "mp_edward_kingman h_1b39dea1: the cut roster's unlock flag in default-global-gameflags.xml"),
        ("hitpoints", Origin::Rule, "95: the boss tier, the sdm Shadoon's; the product owner's choice"),
        ("face", Origin::Real, "face_kingman.tga h_1cc0787c: a cut face in common's textures"),
        ("skill", Origin::Guess, "Tex's Berserker, CSKILL_TEX 3, cost 6.5, recovery 1.2, his start and end sounds, max-stamina 100: demo-own, the product owner's request (#407), in place of the campaign type's CSKILL_KINGMAN 5 (cost 3, recovery 1.5: the briefcase, #201, isn't built). The cost and recovery are the deathmatch Tex's (objecttypes-dm.xml line 51, mp_common's `<base name=\"mp_tex\">`), the product owner's choice, not the campaign Tex's 2.2 / 1.8"),
        ("speeds", Origin::Guess, "the campaign type's 0 / 1.41 / 4.53 / 6.2 kept: no sdm type is his model; the sdm bosses run 5 - 5.5 and dash 7 - 7.2"),
        ("camera", Origin::Rule, "3.2 1 0 alive, dead 3.5 5 0: the common set, carried by 9 of the 20 sdm types; no sdm type is of his model. The product owner chose it over the sdm Hunter Lord's 4 5 0 (h_0959b0fc, the boss of the nearest body: capsule r 0.45 x 2.1 against his 0.5 x 2.0)"),
        ("ammo", Origin::Guess, "High ROF Ammo for the Kman Auto (rule); 120 as the sdm Militia's"),
        ("second gun", Origin::Guess, "a second Kman Auto h_eae7f2e3 in his second weapon slot (two of the same gun in the inventory), switched between as usual (no dual wield): demo-own, the product owner's request (#407). Against his data: his `<limit-weapon one=\"0 8\" two=\"8 8\">` (objecttypes-m03_c.xml line 71) has slot 1 take pistols (class 0) and slot 2 take nothing, so a second gun in slot 2 is the demo's, which `spawn_unit` allows as it doesn't check `slot_classes`; both draw on the one High ROF Ammo reserve (the `ammo` row). In berserk both are drawn, the second gun in the other hand (the left one at the mirror of the trigger hand's mount), with the arm without the gun in hand posed as the mirror of the other (the demo's own: only Tex's model has a SECOND_HAND and berserk clips; play.rs `build_weapon`, play_mirror_arm.rs). His model has no stow hardpoint either, so the demo gives him a holster on the side of each hip (#410: Flint's pistol holster h_f4ccf78c on the right, scaled to his thigh, set 0.10 m back and 0.12 m up onto his pelvis, and its mirror on the left; #409 had both guns on the left hip), each gun belonging to a side and its hand, the left one held and fired left-handed (his arms exchanged, play_mirror_arm.rs); the swap is a timed switch of the demo's (play_holster.rs, #409) that his hands do side to side, the hand in use holstering its gun and the other drawing from its own hip, and the berserk's second gun is fetched and put back by its own hand (play_reach.rs, #410); a pickup replaces the pistol in the hand in use (#410: his data's mode 0 sends every pickup to slot 1)"),
    ],
};

/// The Subversive, the Terrorist Cell's senior terrorist and the Kingman Posse's trooper: m03_c's
/// campaign type (36 hp; m08's 88), with hit points 70 (rule: the standard tier, Tex's), home slot 4,
/// Hawk's place at the left (guess: the product owner's seat, #400, which swapped the two cut
/// troopers' seats), face_subversive.tga (real: a cut
/// face), Tex's speeds (rule: the Ferguson four's, on the same ms_militia) and the Bower 20 with 40
/// Shotgun Ammo (guess: demo-own, the product owner's request of 2026-11-09, in place of the base's
/// Confed LZR-23, which has no ammo item). The Orbital Beam, cost 55, recovery 2.3 (guess: demo-own,
/// the product owner's request, #407; the campaign type has skill-set 0).
/// Its camera is 3.2 m at 1 degree alive, 3.5 m dead (rule: the sdm Militia's block h_1563301d, the same
/// ms_militia model and the common set, as their speeds), not the campaign type's 1.55 m at 10.
pub const SUBVERSIVE: Class = Class {
    key: "mp_subversive_marine",
    base: "subversive@m03_c",
    library: "terrorist_cell",
    hitpoints: 70.0,
    slot: 4,
    face: Some("face_subversive.tga"),
    speeds: Some(MILITIA_SPEEDS),
    ammo: Some((SHOTGUN_AMMO, 40)),
    extra_gun: None,
    guns: Some(&[BOWER_20]),
    camera: CAMERA_COMMON,
    skill: Some(ORBITAL_BEAM_DEMO),
    why: &[
        ("name", Origin::Real, "mp_subversive_marine h_1f330430: the cut roster's unlock flag"),
        ("gun", Origin::Guess, "the Bower 20 h_fa9e169d and 40 Shotgun Ammo h_e5d6b39e, in place of the base type's Confed LZR-23 (which has no ammo item): demo-own, the product owner's request of 2026-11-09"),
        ("hitpoints", Origin::Rule, "70: the standard tier, Tex's; the senior terrorist (36 hp to the Terrorist's 25)"),
        ("face", Origin::Real, "face_subversive.tga h_f7f1acff: a cut face in common's textures"),
        ("skill", Origin::Guess, "the Orbital Beam, CSKILL_ORBITAL_BEAM 9, cost 55, recovery 2.3, max-stamina 100: demo-own, the product owner's request (#407), the dm Confed Marine's skill. The game's Subversive has skill-set 0, no skill"),
        ("speeds", Origin::Rule, "Tex's 3 / 3.6 / 7.8 / 8.5: the Ferguson four's, on the same ms_militia"),
        ("camera", Origin::Rule, "3.2 1 0 alive, dead 3.5 5 0, copied from the sdm Militia h_1563301d: the same ms_militia model (the Ferguson four all carry it)"),
    ],
};

/// The name of the Red Hand's portrait, a texture the demo derives (#400), **not one of the data's**:
/// `Game::texture_rgba` makes it from `RED_HAND_FACE_SOURCE` (`red_hand_face`) when it is asked for,
/// and the HUD caches it as an image like any face. The name is the demo's own.
pub const RED_HAND_FACE: &str = "demo_face_red_hand.tga";

/// The cut face the Red Hand's portrait is derived from: the Terrorist's (h_0be444b1, 32 x 32, in
/// common's library; the Red Hand has the same dark helmet and the data has no face of its own).
pub const RED_HAND_FACE_SOURCE: &str = "face_terrorist.tga";

/// The rows of the Terrorist's face that are the helmet, counted from the top of the picture (a 32
/// row texture: its rim and dark dome, rows 5 - 7 and 8 - 16 of the picture, and the visor), kept as
/// the Terrorist has them. The rows below are the collar, shoulders and chest. Read off the face's
/// value and saturation grid: the helmet's lower edge is the near-black row 16, the body starts at 17.
const RED_HAND_HELMET_ROWS: usize = 17;

/// The Red Hand's armour colours by the value (brightness 0 - 1) of the Terrorist's body texel they
/// replace: (value, red, green, blue). Dark brown in the shadows, a red band for the shaded folds
/// (the model's red markings), then the tans, then a light tan for the highlights. The colours are the
/// dominant ones of the Red Hand's own skin texture h_e2aebd26 (e05's, the colour texture of its
/// material h_f442845f; the model's other textures h_1c7e6cf8, h_fb7fd9a7 and h_0b618930 are the
/// Terrorist's and the Militia's too): a k-means (k = 6) of its texels with saturation over 0.3 and
/// alpha over 40 (about 2,900) gave dark brown (30, 17, 8) and (42, 28, 14), maroon (86, 46, 28) and
/// (114, 61, 49) (the hue 0 - 20 degrees texels' mean), tan (140, 116, 79), (163, 143, 97) (the hue 25 -
/// 55 degrees, value over 0.5 texels' mean) and (171, 146, 104), light tan (206, 169, 129); and a
/// k-means of its fully opaque texels (57) added the strong red of the markings, (146, 24, 13) and
/// (150, 40, 28). The red band uses that strong red, (146, 38, 26). The stops' values (where each
/// colour sits on the source's shading) are a fit by eye, against the Red Hand model in the demo, so
/// that the shaded sides read red and the lit plate tan, as the model's red-and-tan armour does.
const RED_HAND_RAMP: [(f32, [f32; 3]); 7] = [
    (0.00, [30.0, 17.0, 8.0]),
    (0.10, [86.0, 46.0, 28.0]),
    (0.22, [146.0, 38.0, 26.0]),
    (0.32, [128.0, 48.0, 36.0]),
    (0.38, [163.0, 143.0, 97.0]),
    (0.55, [171.0, 146.0, 104.0]),
    (0.75, [206.0, 169.0, 129.0]),
];

/// The Terrorist's own skin, under his visor (#402): the opening of his helmet, a patch of warm mauve
/// and pink-brown texels at picture rows 16 - 20, columns 4 - 7: (74, 52, 57), (98, 76, 76), (123,
/// 100, 95) and the lit chin (148, 125, 115) at rows 19 - 20 (values 0.29 - 0.58), framed by the
/// visor's dark blue-black texels to its right (columns 8 - 11) and the black of the neck under it
/// (rows 20 - 21). Found by exporting `face_terrorist.tga` (h_0be444b1, `tools/tex_tool.py export` on
/// common's archive, flipped upright) and looking at the 32 x 32 picture enlarged. `red_hand_face`
/// took these texels for armour (rows 17 - 20 are below its helmet rows) and recoloured them to the
/// Red Hand's tan and red; it now keeps them as the Terrorist has them: rows 17 - 20 (row 16 is a
/// helmet row, kept already), columns 3 - 11. A fit by eye on the enlarged portrait.
const RED_HAND_SKIN_ROWS: std::ops::Range<usize> = 17..21;
/// The columns of `RED_HAND_SKIN_ROWS`' patch, counted from the picture's left; see that constant.
const RED_HAND_SKIN_COLUMNS: std::ops::Range<usize> = 3..12;

/// How much of the Terrorist's own texel `red_hand_face` keeps at `distance` texels (Chebyshev) outside
/// the patch of `RED_HAND_SKIN_ROWS` x `RED_HAND_SKIN_COLUMNS`: all of it inside, half in the ring of
/// texels round it (the other half the armour's colour, so the patch blends into the collar and hood
/// instead of ending on a box) and none beyond. A fit by eye on the enlarged portrait.
fn red_hand_skin_weight(distance: usize) -> f32 {
    match distance {
        0 => 1.0,
        1 => 0.5,
        _ => 0.0,
    }
}

/// `ramp`'s colour for a texel of value `v` (0 - 1): between its two nearest stops, and the end stop's
/// colour beyond the ends.
fn ramp_colour(ramp: &[(f32, [f32; 3])], v: f32) -> [f32; 3] {
    // (above the last stop the last colour holds)
    let k = ramp.iter().position(|s| s.0 >= v).unwrap_or(ramp.len() - 1).max(1);
    let (lo, hi) = (ramp[k - 1], ramp[k]);
    let t = ((v - lo.0) / (hi.0 - lo.0)).clamp(0.0, 1.0);
    [0, 1, 2].map(|j| lo.1[j] + (hi.1[j] - lo.1[j]) * t)
}

/// The Red Hand's portrait: the Terrorist's face texels (`RED_HAND_FACE_SOURCE`, RGBA8 as
/// `Game::texture_rgba` gives them, `width` texels a row, the bottom row first) with everything below
/// the helmet (`RED_HAND_HELMET_ROWS` rows from the top: the collar, shoulders and chest) recoloured
/// from the Red Hand's skin: each texel's value is looked up in `RED_HAND_RAMP`, so the shading is the
/// source's and the colours the Red Hand's tan and red. Except the Terrorist's own skin under his
/// visor (`RED_HAND_SKIN_ROWS` x `RED_HAND_SKIN_COLUMNS`, #402): those texels stay as the Terrorist
/// has them, and the ring of texels round the patch is mixed half and half with the armour's colour
/// (`red_hand_skin_weight`). The helmet's other texels, with its dark dome, grey rim and visor, and
/// every alpha (the transparent background) are kept. The demo's own derived image, not the data's;
/// the armour's colours are the Red Hand's skin texture's, the stops' values and the rows and columns
/// fits by eye.
pub fn red_hand_face(rgba: &[u8], width: usize) -> Vec<u8> {
    let w = width.max(1);
    let rows = rgba.len() / 4 / w;
    // (how many texels a coordinate lies outside a range)
    let outside = |x: usize, r: &std::ops::Range<usize>| if x < r.start { r.start - x } else if x >= r.end { x + 1 - r.end } else { 0 };
    rgba.chunks_exact(4).enumerate().flat_map(|(i, p)| {
        // (library order: row 0 is the picture's bottom)
        let from_top = rows - 1 - i / w;
        if from_top < RED_HAND_HELMET_ROWS {
            return [p[0], p[1], p[2], p[3]];
        }
        let v = p[0].max(p[1]).max(p[2]) as f32 / 255.0;
        let armour = ramp_colour(&RED_HAND_RAMP, v);
        let keep = if p[3] > 0 {
            red_hand_skin_weight(outside(from_top, &RED_HAND_SKIN_ROWS).max(outside(i % w, &RED_HAND_SKIN_COLUMNS)))
        } else {
            0.0
        };
        let c = |j: usize| (p[j] as f32 * keep + armour[j] * (1.0 - keep)).round() as u8;
        [c(0), c(1), c(2), p[3]]
    }).collect()
}

/// The Filthy Mutant's portrait, a texture the demo derives (#401), **not one of the data's**:
/// `Game::texture_rgba` makes it from `MUTANT_FACE_SOURCE` (`mutant_face`) when it is asked for.
/// The name is the demo's own.
pub const MUTANT_FILTHY_FACE: &str = "demo_face_mutant_filthy.tga";
/// The Molotov Mutant's portrait, derived the same way (#401); the name is the demo's own.
pub const MUTANT_MOLOTOV_FACE: &str = "demo_face_mutant_molotov.tga";
/// The Ugly Mutant's portrait (#401): the data's mutant face in its own colours with the black box
/// cleared and the art fitted; the name is the demo's own.
pub const MUTANT_UGLY_FACE: &str = "demo_face_mutant_ugly.tga";

/// The data's one mutant face (h_fc5f519f, 32 x 32, in common's library) the three mutant portraits
/// are derived from: 27 x 21 texels of art at the picture's top left (columns 0 - 26, picture rows
/// 0 - 20; rows 11 - 31 of the texels as the library keeps them, the bottom row first, like every
/// face's art) with 109 opaque pure-black texels beside the head (a 7 x 10 block at the art's right
/// and a corner at its left, which the radar shows as a hard black box), the rest transparent.
pub const MUTANT_FACE_SOURCE: &str = "face_mutant.tga";

/// The largest value (the brightest channel, 0 - 255) of a texel `mutant_face` takes for the black
/// box: the box is pure black (value 0) and the near-black shading round it starts at 8. 2 is a
/// guess that keeps the shading texels and clears the box (109 texels, the ticket's 107 - 120).
const MUTANT_BOX_BLACK: u8 = 2;

/// The box `mutant_face` fits the cleared art into, in texels of the 32 x 32 picture (#401): the art's
/// whole opaque bounding box is scaled down uniformly (aspect kept, never enlarged) to fit
/// `MUTANT_FIT_WIDTH` x `MUTANT_FIT_HEIGHT`, centred across the texture and `MUTANT_FIT_TOP` rows
/// below its top, so nothing touches the texture's border and no head is cut. Measured on the nine
/// other portraits (`tools/tex_tool.py export`, opaque bounds, the picture's rows from the top): their
/// art is 21 - 27 columns wide (mean 23.4: the Terrorist, the Subversive, Flint's, Hawk's and the
/// Supermutant's 21 - 22, Brutus's, Tex's and Kingman's 27) and 24 - 31 rows tall, starting at the
/// top row (Tex's at 0 - 1) except Kingman's at row 3, and ending at row 26 - 30. The width is the
/// mean rounded, the top Kingman's margin, the height the rows left under it down to row 26 (the
/// most common bottom); the box is a fit by eye on those.
const MUTANT_FIT_WIDTH: f32 = 24.0;
/// See `MUTANT_FIT_WIDTH`: the box's height, rows 3 - 26.
const MUTANT_FIT_HEIGHT: f32 = 24.0;
/// See `MUTANT_FIT_WIDTH`: the rows of clear texels above the fitted art, Kingman's margin.
const MUTANT_FIT_TOP: f32 = 3.0;

/// A skin's colours as the quantiles of its texels: eight colours, the mean of each eighth of the
/// texture's texels sorted by value (the brightest channel), from the darkest eighth to the lightest.
/// What `mutant_face` recolours a face with: a texel's rank among the face's own texels picks the
/// colour at the same rank here, so the face keeps its shading (its light and shadow) and takes the
/// skin's colours.
pub type SkinRamp = [[f32; 3]; 8];

/// The Filthy Mutant's skin: the texture h_1ac02ad3 (256 x 256 in e23, the colour texture of his
/// model's materials h_ed6ca9fc and h_000f0cc9: the head's and hands' skin, brown and pink, mean
/// (110, 70, 52)), eight value-quantile means of all its texels (alpha ignored: the model's material
/// draws it opaque). The Molotov Mutant has the same model and the same textures (e23's h_f0c5e50e
/// carries the compound h_18f5f8dc, materials h_09083c5f, h_ed6ca9fc, h_000f0cc9 and h_1b065d73, as
/// the Filthy's h_0b5f6e21), so the pair's different faces are the demo's choice, not the data's.
pub const FILTHY_SKIN: SkinRamp = [
    [34.0, 22.0, 14.0], [51.0, 33.0, 22.0], [67.0, 41.0, 26.0], [88.0, 51.0, 32.0],
    [111.0, 57.0, 34.0], [142.0, 82.0, 61.0], [186.0, 121.0, 98.0], [222.0, 160.0, 135.0],
];

/// The Molotov Mutant's skin (the demo's choice, see `FILTHY_SKIN`): the texture h_fe8edd48 (256 x 256
/// in e23, the colour texture of the same model's material h_09083c5f, the hair and fur: tan and
/// cream with orange-brown shadows, mean (159, 113, 85), against the blond and tan the model shows),
/// as `FILTHY_SKIN`.
pub const MOLOTOV_SKIN: SkinRamp = [
    [32.0, 1.0, 2.0], [59.0, 23.0, 14.0], [148.0, 96.0, 63.0], [178.0, 119.0, 79.0],
    [195.0, 135.0, 93.0], [209.0, 155.0, 114.0], [218.0, 170.0, 133.0], [236.0, 207.0, 184.0],
];

/// The portrait of a mutant (#401), the demo's own derived image, made from the data's mutant face
/// (`MUTANT_FACE_SOURCE`, RGBA8 as `Game::texture_rgba` gives it, `width` texels a row, the picture's
/// bottom row first) in three steps:
///  1. **the black box**: the opaque near-black texels (`MUTANT_BOX_BLACK`) joined to the art's edge
///     (the texture's border or a transparent texel) turn transparent, a flood fill over the
///     four-connected texels, so the head sits as the other portraits' do;
///  2. **the colours**, when `skin` is given: each texel keeps its place in the face's own shading
///     (its rank among the face's remaining opaque texels by value, ties sharing their mean) and takes
///     the skin's colour at that quantile (`SkinRamp`, interpolated between the eight);
///  3. **the fit**: the cleared art's whole opaque bounding box scaled down to fit the box of
///     `MUTANT_FIT_WIDTH` x `MUTANT_FIT_HEIGHT` (aspect kept), centred across the texture and
///     `MUTANT_FIT_TOP` rows from its top (bilinear on premultiplied colour, so the edge fades into the
///     clear padding): the whole head shows inside the portrait, as the other portraits' art sits.
pub fn mutant_face(rgba: &[u8], width: usize, skin: Option<&SkinRamp>) -> Vec<u8> {
    let w = width.max(1);
    let rows = rgba.len() / 4 / w;
    let mut px: Vec<[u8; 4]> = rgba.chunks_exact(4).map(|p| [p[0], p[1], p[2], p[3]]).collect();
    // 1. the box: flood from the black texels at the border of the art
    let black = |p: &[u8; 4]| p[3] > 0 && p[0].max(p[1]).max(p[2]) <= MUTANT_BOX_BLACK;
    let neighbours = |i: usize| {
        let (x, y) = (i % w, i / w);
        [(x > 0).then(|| i - 1), (x + 1 < w).then(|| i + 1), (y > 0).then(|| i - w), (y + 1 < rows).then(|| i + w)]
    };
    let mut stack: Vec<usize> = (0..px.len())
        .filter(|&i| black(&px[i]) && neighbours(i).iter().any(|n| n.is_none_or(|n| px[n][3] == 0)))
        .collect();
    while let Some(i) = stack.pop() {
        if !black(&px[i]) {
            continue;
        }
        px[i] = [0, 0, 0, 0];
        stack.extend(neighbours(i).into_iter().flatten().filter(|&n| black(&px[n])));
    }
    // 2. the colours, by rank among the opaque texels' values
    if let Some(skin) = skin {
        let mut values: Vec<u8> = px.iter().filter(|p| p[3] > 0).map(|p| p[0].max(p[1]).max(p[2])).collect();
        values.sort_unstable();
        let n = values.len().max(1) as f32;
        for p in px.iter_mut().filter(|p| p[3] > 0) {
            let v = p[0].max(p[1]).max(p[2]);
            let (below, upto) = (values.partition_point(|&x| x < v), values.partition_point(|&x| x <= v));
            let q = (below + upto) as f32 / 2.0 / n;
            // (the eight colours are the quantiles' centres, 1/16, 3/16 ... 15/16; the ends hold)
            let at = (q * 8.0 - 0.5).clamp(0.0, 7.0);
            let k = (at.floor() as usize).min(6);
            let t = at - k as f32;
            let c = |j: usize| (skin[k][j] + (skin[k + 1][j] - skin[k][j]) * t).round() as u8;
            *p = [c(0), c(1), c(2), p[3]];
        }
    }
    // 3. the fit: the opaque bounding box (the picture's rows from the top), scaled to the box and placed in it
    let from_top = |i: usize| rows - 1 - i / w;
    let (mut x_lo, mut x_hi, mut y_lo, mut y_hi) = (usize::MAX, 0, usize::MAX, 0);
    for (i, _) in px.iter().enumerate().filter(|(_, p)| p[3] > 0) {
        (x_lo, x_hi) = (x_lo.min(i % w), x_hi.max(i % w));
        (y_lo, y_hi) = (y_lo.min(from_top(i)), y_hi.max(from_top(i)));
    }
    if x_lo > x_hi {
        return px.iter().flatten().copied().collect();
    }
    let (bw, bh) = ((x_hi - x_lo + 1) as f32, (y_hi - y_lo + 1) as f32);
    let scale = (MUTANT_FIT_WIDTH / bw).min(MUTANT_FIT_HEIGHT / bh).min(1.0);
    // (the art's left edge lands so its box is centred across the texture)
    let left = (w as f32 - bw * scale) / 2.0;
    let at = |x: i64, y: i64| -> [f32; 4] {
        // (beyond the texture is clear)
        if x < 0 || y < 0 || x >= w as i64 || y >= rows as i64 {
            return [0.0; 4];
        }
        let p = px[y as usize * w + x as usize];
        let a = p[3] as f32 / 255.0;
        [p[0] as f32 * a, p[1] as f32 * a, p[2] as f32 * a, a]
    };
    let mut out = Vec::with_capacity(px.len() * 4);
    for y in 0..rows {
        for x in 0..w {
            let row_from_top = (rows - 1 - y) as f32;
            let u = x_lo as f32 + (x as f32 + 0.5 - left) / scale - 0.5;
            let v_top = y_lo as f32 + (row_from_top + 0.5 - MUTANT_FIT_TOP) / scale - 0.5;
            let v = rows as f32 - 1.0 - v_top;
            let (x0, y0) = (u.floor() as i64, v.floor() as i64);
            let (tx, ty) = (u - x0 as f32, v - y0 as f32);
            let mut s = [0f32; 4];
            for (dx, dy, wgt) in [(0, 0, (1.0 - tx) * (1.0 - ty)), (1, 0, tx * (1.0 - ty)), (0, 1, (1.0 - tx) * ty), (1, 1, tx * ty)] {
                let q = at(x0 + dx, y0 + dy);
                for j in 0..4 {
                    s[j] += q[j] * wgt;
                }
            }
            let a = s[3];
            let un = |c: f32| if a > 0.0 { (c / a).round().clamp(0.0, 255.0) as u8 } else { 0 };
            out.extend_from_slice(&[un(s[0]), un(s[1]), un(s[2]), (a * 255.0).round() as u8]);
        }
    }
    out
}

/// A portrait the demo derives instead of the data keeping it: its texture name, the texture it is
/// made from and how (`Game::texture_rgba` makes it when it is asked for, the HUD caches it as an
/// image like any face).
pub struct DerivedFace {
    /// the demo's own texture name (`RED_HAND_FACE` ...)
    pub name: &'static str,
    /// the data's texture it is made from
    pub source: &'static str,
    /// the image: the source's RGBA8 and its width in texels, the picture's bottom row first
    pub make: fn(&[u8], usize) -> Vec<u8>,
}

/// The portraits the demo derives (#400, #401): the Red Hand's from the Terrorist's, the three
/// mutants' from the shared mutant face.
pub const DERIVED_FACES: [DerivedFace; 4] = [
    DerivedFace { name: RED_HAND_FACE, source: RED_HAND_FACE_SOURCE, make: red_hand_face },
    DerivedFace { name: MUTANT_FILTHY_FACE, source: MUTANT_FACE_SOURCE, make: |p, w| mutant_face(p, w, Some(&FILTHY_SKIN)) },
    DerivedFace { name: MUTANT_MOLOTOV_FACE, source: MUTANT_FACE_SOURCE, make: |p, w| mutant_face(p, w, Some(&MOLOTOV_SKIN)) },
    DerivedFace { name: MUTANT_UGLY_FACE, source: MUTANT_FACE_SOURCE, make: |p, w| mutant_face(p, w, None) },
];

/// The derived portrait whose texture has the name hash `name`, if it is one.
pub fn derived_face(name: u32) -> Option<&'static DerivedFace> {
    DERIVED_FACES.iter().find(|d| h(d.name) == name)
}

/// The Orbital Beam the Red Hand carries (#407, demo-own: the product owner's request; the
/// Red Hand's campaign type has skill-set 0): CSKILL_ORBITAL_BEAM 9 with the dm Confed Marine's
/// cost 55 and recovery 2.3 (objecttypes-dm.xml, h_048b8580; `play_effect_skills.rs`
/// MARINE_COST / MARINE_RECOVERY) and max-stamina 100.
pub const ORBITAL_BEAM_DEMO: SkillOverride = SkillOverride { skill_set: 9, cost: 55.0, recovery: 2.3, max_stamina: 100.0, sounds: (0, 0) };

/// Tex's Berserker, CSKILL_TEX 3, as the deathmatch Tex carries it (#407, demo-own: the product
/// owner's request for Edward Kingman, in place of his CSKILL_KINGMAN 5, with the deathmatch
/// numbers by his choice): `mp_common/objecttypes-dm.xml` line 51, the combat-skill of the
/// character entry whose `<base name="mp_tex">`: `skill-set="3"`, stamina-cost h_105c13aa 6.5,
/// stamina-recovery h_f2a2beb9 1.2, `<sounds start-skill="h_ed7492f4" end-skill="h_e00d91fa">` (the
/// campaign Tex's, `<base name="tex">` h_fc310d72, has 2.2 / 1.8 and the same sounds) and max-stamina 100.
pub const BERSERKER_DEMO: SkillOverride = SkillOverride { skill_set: 3, cost: 6.5, recovery: 1.2, max_stamina: 100.0, sounds: (0xED74_92F4, 0xE00D_91FA) };

/// Gunthar Ghent's Cloak, CSKILL_CLOAK 8, as the deathmatch Gunthar Ghent carries it (#205, demo-own:
/// the product owner's request for the Terrorist, whose campaign type has skill-set 0):
/// `mp_common/objecttypes-dm.xml` line 415, the combat-skill of the character entry whose `<base
/// name="h_1dc1aa6d">` (the Ferguson Platoon's h_e714d929, objecttypes-sdm_ferg.xml line 235, has the
/// same): `skill-set="8"`, stamina-cost h_105c13aa 6, stamina-recovery h_f2a2beb9 1.5, `<sounds
/// start-skill="" end-skill="">` (none) and max-stamina 100.
pub const CLOAK_DEMO: SkillOverride = SkillOverride { skill_set: 8, cost: 6.0, recovery: 1.5, max_stamina: 100.0, sounds: (0, 0) };

/// The m03 Filthy Mutants' corpse eater, CSKILL_CORPSE_EATER 16, as the demo's Filthy and Ugly Mutant
/// classes carry it (#210, demo-own: the product owner's design, 2026-11-09; the Ugly Mutant's too,
/// 2026-11-09, as he shares the mutants' motion set and so the eat clips). e23's copy (the class's base
/// type) has skill-set 0; m03's carry 16 as the AI's set dressing, and the cost 3 and recovery 1.5
/// are theirs (`<combat-skill skill-set="16" h_105c13aa="3" h_f2a2beb9="1.5">`,
/// objecttypes-m03_a.xml's h_1662a599), max-stamina 100, no sounds (the type's are empty).
pub const CORPSE_EATER_DEMO: SkillOverride = SkillOverride { skill_set: 16, cost: 3.0, recovery: 1.5, max_stamina: 100.0, sounds: (0, 0) };

/// The Red Hand, the Terrorist Cell's heavy fighter: e05's campaign type (31 hp; e13's 128, the
/// tutorial's 23), with hit points 55 (rule: the light-armoured tier, the Shrike Heavy's), home
/// slot 3, Brutus's place at the bottom (rule: the heavy fighter's), Tex's speeds (rule), the
/// Saryl-45 with 100 Particle Ammo (guess: demo-own, the product owner's request of 2026-11-09, in
/// place of the base's Confed LZR-23) and the demo's own derived portrait `RED_HAND_FACE` (#400: the data has
/// no face for the Red Hand, so the ring shows the cut Terrorist's face with the armour below the helmet recoloured tan and red). The key
/// is the demo's own (guess: the data has no cut name for it).
/// Its camera is 3.2 m at 1 degree alive, 3.5 m dead (rule: the sdm Militia's block h_1563301d, the same
/// ms_militia model and the common set, as their speeds), not the campaign type's 1.55 m at 10.
pub const RED_HAND: Class = Class {
    key: "mp_red_hand",
    base: "red hand@e05",
    library: "terrorist_cell",
    hitpoints: 55.0,
    slot: 3,
    face: Some(RED_HAND_FACE),
    speeds: Some(MILITIA_SPEEDS),
    ammo: Some((PARTICLE_AMMO, 100)),
    extra_gun: None,
    guns: Some(&[SARYL_45]),
    camera: CAMERA_COMMON,
    skill: Some(ORBITAL_BEAM_DEMO),
    why: &[
        ("name", Origin::Guess, "mp_red_hand h_ebefd26c: the demo's, the data has no cut name for it"),
        ("gun", Origin::Guess, "the Saryl-45 h_1c8e42e0 and 100 Particle Ammo h_fc0d262e, in place of the base type's Confed LZR-23 (which has no ammo item): demo-own, the product owner's request of 2026-11-09"),
        ("hitpoints", Origin::Rule, "55: the light-armoured tier, the Shrike Heavy's"),
        ("face", Origin::Guess, "the demo's own derived portrait, not the data's (the data has none for the Red Hand): face_terrorist.tga h_0be444b1 with everything below the helmet recoloured to the Red Hand's own tan and red (its skin texture h_e2aebd26, `red_hand_face`, made when the texture is asked for), except the Terrorist's own skin under the visor, kept as it is (#402)"),
        ("skill", Origin::Guess, "the Orbital Beam, CSKILL_ORBITAL_BEAM 9, cost 55, recovery 2.3, max-stamina 100: demo-own, the product owner's request (#407), the dm Confed Marine's skill. The game's Red Hand has skill-set 0, no skill"),
        ("speeds", Origin::Rule, "Tex's 3 / 3.6 / 7.8 / 8.5: the Ferguson four's, on the same ms_militia"),
        ("camera", Origin::Rule, "3.2 1 0 alive, dead 3.5 5 0, copied from the sdm Militia h_1563301d: the same ms_militia model (the Ferguson four all carry it)"),
    ],
};

/// The Terrorist, the Terrorist Cell's grunt: m03_c's campaign type (25 hp, the lowest of the
/// three), with hit points 50 (rule: the grunt tier, Hawk's and the Shrike Soldier's), home slot 2,
/// Flint's place at the right (guess: the product owner's seat, #400, which swapped the two cut
/// troopers' seats), face_terrorist.tga (real: a cut face), Tex's
/// speeds (rule) and the Ion Beamer with 100 Energy Ammo (guess: demo-own, the product owner's
/// request of 2026-11-09, in place of the base's Confed LZR-23). Gunthar Ghent's Cloak, cost 6, recovery 1.5 (guess:
/// demo-own, the product owner's request, #205; the campaign type has skill-set 0).
/// Its camera is 3.2 m at 1 degree alive, 3.5 m dead (rule: the sdm Militia's block h_1563301d, the same
/// ms_militia model and the common set, as their speeds), not the campaign type's 1.55 m at 10.
pub const TERRORIST: Class = Class {
    key: "mp_terrorist",
    base: "terrorist@m03_c",
    library: "terrorist_cell",
    hitpoints: 50.0,
    slot: 2,
    face: Some("face_terrorist.tga"),
    speeds: Some(MILITIA_SPEEDS),
    ammo: Some((ENERGY_AMMO, 100)),
    extra_gun: None,
    guns: Some(&[ION_BEAMER]),
    camera: CAMERA_COMMON,
    skill: Some(CLOAK_DEMO),
    why: &[
        ("name", Origin::Real, "mp_terrorist h_03589fbb: the cut roster's unlock flag"),
        ("gun", Origin::Guess, "the Ion Beamer h_ea241276 and 100 Energy Ammo h_15c866a4, in place of the base type's Confed LZR-23 (which has no ammo item): demo-own, the product owner's request of 2026-11-09"),
        ("hitpoints", Origin::Rule, "50: the grunt tier, Hawk's and the Shrike Soldier's"),
        ("face", Origin::Real, "face_terrorist.tga h_0be444b1: a cut face in common's textures"),
        ("skill", Origin::Guess, "Gunthar Ghent's Cloak, CSKILL_CLOAK 8, cost 6, recovery 1.5, max-stamina 100, no sounds: demo-own, the product owner's request (#205), the dm Gunthar Ghent's skill. The game's Terrorist has skill-set 0, no skill"),
        ("speeds", Origin::Rule, "Tex's 3 / 3.6 / 7.8 / 8.5: the Ferguson four's, on the same ms_militia"),
        ("camera", Origin::Rule, "3.2 1 0 alive, dead 3.5 5 0, copied from the sdm Militia h_1563301d: the same ms_militia model (the Ferguson four all carry it)"),
    ],
};

/// The Supermutant, the Mutant Horde's boss (his home) and a Kingman Posse member: e23's
/// campaign type (669 hp), with hit points 95 (rule: the boss tier), home slot 2, Flint's place at
/// the right (guess: the product owner's seat, #401: his portrait fits a side slot; the rule's boss-at-
/// the-top seat went to the Molotov Mutant), face_supermutant.tga (real: a cut face), the skill CSKILL_BOSS_SUPERMUTANT 17,
/// cost 1, recovery 1.5 (real: the campaign type's; the ground pound, #212, built in
/// play_ground_pound.rs), the
/// MNR-7 Bio, 5 Energy grenades and its death effect h_e19a8348 (real), 60 Bio Ammo (rule: the
/// gun's ammo-type 8 item; guess: 60, 0.3 of its stack of 200, about the share the sdm types
/// carry) and speeds 0 / 2.3 / 5 / 7 kept (guess: the sdm bosses run 5 - 5.5 and dash 7 - 7.2,
/// the Hunter Lord and the Shadoon).
/// Its camera is the common 3.2 m at 1 degree alive, 3.5 m dead (rule: `CAMERA_COMMON`, no sdm type is of the mutants'
/// model; the product owner chose it over the sdm Hunter Lord's 4 m at 5), not the campaign type's 1.55 m at 10.
pub const SUPERMUTANT: Class = Class {
    key: "mp_supermutant",
    base: "supermutant@e23",
    library: "mutant_horde",
    hitpoints: 95.0,
    slot: 2,
    face: Some("face_supermutant.tga"),
    speeds: None,
    ammo: Some((BIO_AMMO, 60)),
    extra_gun: None,
    guns: None,
    camera: CAMERA_COMMON,
    skill: None,
    why: &[
        ("name", Origin::Real, "mp_supermutant h_16122b66: the cut roster's unlock flag"),
        ("hitpoints", Origin::Rule, "95: the boss tier"),
        ("face", Origin::Real, "face_supermutant.tga h_f05a2afc: a cut face in common's textures"),
        ("skill", Origin::Real, "CSKILL_BOSS_SUPERMUTANT 17, cost 1, recovery 1.5: the campaign type's (the ground pound, #212, built in play_ground_pound.rs)"),
        ("speeds", Origin::Guess, "the campaign type's 0 / 2.3 / 5 / 7 kept: the sdm bosses run 5 - 5.5 and dash 7 - 7.2"),
        ("camera", Origin::Rule, "3.2 1 0 alive, dead 3.5 5 0: the common set, carried by 9 of the 20 sdm types; no sdm type is of the mutants' model. The product owner chose it over the sdm Hunter Lord's 4 5 0 (h_0959b0fc, the boss of the nearest body: capsule r 0.45 x 2.1 against his 0.5 x 1.9)"),
        ("ammo", Origin::Guess, "Bio Ammo for the MNR-7 Bio (rule); 60, about the share of its stack the sdm types carry"),
    ],
};

/// The Molotov Mutant, the Mutant Horde's specialist: e23's campaign type (55 hp), with hit
/// points 55 (rule: the specialist tier, Flint's; a self-destruct type is spent on use), home
/// slot 1, Tex's place at the top (guess: the product owner's seat, #401; #394 had it at the right,
/// where #365 put it), the demo's own derived mutant face `MUTANT_MOLOTOV_FACE` (#401: the data has
/// one mutant face, `face_mutant.tga`, shared and boxed in black), the skill
/// CSKILL_MOLOTOV_MUTANT 20, cost 3, recovery 1.5 (real: the campaign type's, whose AI-only bottle and lunge the
/// demo gives a player by the product owner's design, #211: play_molotov_mutant.rs), the Confed LZR-10 and 2
/// Gas grenades (real), speeds kept
/// (guess, as the Supermutant's). The key is the demo's own (guess).
/// Its camera is the common 3.2 m at 1 degree alive, 3.5 m dead (rule: `CAMERA_COMMON`, no sdm type is of the mutants'
/// model), not the campaign type's 1.55 m at 10.
pub const MOLOTOV: Class = Class {
    key: "mp_mutant_molotov",
    base: "molotov mutant@e23",
    library: "mutant_horde",
    hitpoints: 55.0,
    slot: 1,
    face: Some(MUTANT_MOLOTOV_FACE),
    speeds: None,
    ammo: None,
    extra_gun: None,
    guns: None,
    camera: CAMERA_COMMON,
    skill: None,
    why: &[
        ("name", Origin::Guess, "mp_mutant_molotov h_02f7069a: the demo's, the cut roster names two of the three mutants at most"),
        ("hitpoints", Origin::Rule, "55: the specialist tier, Flint's"),
        ("face", Origin::Guess, "the demo's own derived portrait, not the data's (the data has one mutant face for all three, face_mutant.tga h_fc5f519f, with an opaque black box): that face with the box cleared, recoloured from the skin texture h_fe8edd48 of its model's material h_09083c5f (e23, the hair and fur, tan and cream; the Filthy's model and textures are the same, so the choice of this texture over his h_1ac02ad3 is the demo's, to tell the two apart), fitted to the slot (`mutant_face`, made when the texture is asked for)"),
        ("skill", Origin::Real, "CSKILL_MOLOTOV_MUTANT 20, cost 3, recovery 1.5: the campaign type's, kept (built as the product owner's design for a player, #211: a lit bottle held in his left hand, put away by a press, a lunge, 500 FLAME to himself and a cloud within 5 m of a hostile; the game's version is AI-only)"),
        ("speeds", Origin::Guess, "the campaign type's 0 / 2.3 / 5 / 7 kept, as the Supermutant's"),
        ("camera", Origin::Rule, "3.2 1 0 alive, dead 3.5 5 0: the common set, carried by 9 of the 20 sdm types (the Ferguson four, Flint, the Shrike Heavy and Soldier, the two hounds); no sdm type is of the mutants' model"),
    ],
};

/// The Ugly Mutant, the Mutant Horde's big one: e23's campaign type (55 hp, on the bigger
/// ms_mutant model), with hit points 75 (rule: Brutus's tier, the Supermutant's model), home slot
/// 3, Brutus's place at the bottom (rule), the data's mutant face with its black box cleared,
/// `MUTANT_UGLY_FACE` (the demo's own derived image, #401), the Foley 356 Tact with 25 11mm Ammo
/// (guess: demo-own, the product owner's request of 2026-11-09, in place of the base's Confed LZR-10) and
/// 2 of the grenade item h_fd437c4d (real: an item of its own, stack-limit 999, whose pickup is
/// the Frag's h_060f95f7; the analyst's draft calls them Frag), speeds kept (guess). The corpse eater, CSKILL_CORPSE_EATER 16 (`CORPSE_EATER_DEMO`, demo-own: the product owner's, #210),
/// cost 3, recovery 1.5 (the campaign type's skill-set is 0). The key is the demo's own (guess).
/// Its camera is the common 3.2 m at 1 degree alive, 3.5 m dead (rule: `CAMERA_COMMON`, no sdm type is of the mutants'
/// model), not the campaign type's 1.55 m at 10.
pub const UGLY: Class = Class {
    key: "mp_mutant_ugly",
    base: "ugly mutant@e23",
    library: "mutant_horde",
    hitpoints: 75.0,
    slot: 3,
    face: Some(MUTANT_UGLY_FACE),
    speeds: None,
    ammo: Some((AMMO_11MM, 25)),
    extra_gun: None,
    guns: Some(&[FOLEY_356]),
    camera: CAMERA_COMMON,
    skill: Some(CORPSE_EATER_DEMO),
    why: &[
        ("name", Origin::Guess, "mp_mutant_ugly h_0ca8ee04: the demo's"),
        ("gun", Origin::Guess, "the Foley 356 Tact h_e93533d1 and 25 11mm Ammo h_12302d03, in place of the base type's Confed LZR-10: demo-own, the product owner's request of 2026-11-09"),
        ("hitpoints", Origin::Rule, "75: Brutus's tier; the bigger ms_mutant model, the Supermutant's"),
        ("face", Origin::Guess, "the demo's own derived portrait: face_mutant.tga h_fc5f519f, the data's one mutant face, in its own colours but with its opaque black box cleared and the whole art fitted inside the slot (`mutant_face`, made when the texture is asked for)"),
        ("skill", Origin::Guess, "the corpse eater, CSKILL_CORPSE_EATER 16, cost 3, recovery 1.5, max-stamina 100: demo-own, the product owner's design (#210), given to this class too (2026-11-09), as he shares the mutants' motion set and so the eat clips. The cost and recovery are m03's Filthy Mutants' (objecttypes-m03_a.xml's h_1662a599); the class's own base type, e23's copy, has skill-set 0, and the game's eating is an AI set piece the level's script plays"),
        ("speeds", Origin::Guess, "the campaign type's 0 / 2.3 / 5 / 7 kept, as the Supermutant's"),
        ("camera", Origin::Rule, "3.2 1 0 alive, dead 3.5 5 0: the common set, carried by 9 of the 20 sdm types (the Ferguson four, Flint, the Shrike Heavy and Soldier, the two hounds); no sdm type is of the mutants' model"),
    ],
};

/// The Filthy Mutant, the Mutant Horde's common one (his home) and the Kingman Posse's mutant: e23's
/// campaign type (55 hp; m03's copies carry the corpse-eater skill, #210), with hit points 70
/// (rule: the standard tier, the ferals' 70 - 75), home slot 4, Hawk's place at the left (rule: the
/// lightest model, ms_mutant2), the demo's own derived mutant face `MUTANT_FILTHY_FACE` (#401), the Jax-iP with
/// 100 Sonic Ammo (guess: demo-own, the product owner's request of 2026-11-09, in place of the base's Confed LZR-10) and 2 Gas
/// grenades (real), speeds kept (guess). The corpse eater, CSKILL_CORPSE_EATER 16 (`CORPSE_EATER_DEMO`,
/// demo-own: the product owner's design, #210; e23's copy has skill-set 0), cost 3, recovery 1.5.
/// The key is the demo's own (guess).
/// Its camera is the common 3.2 m at 1 degree alive, 3.5 m dead (rule: `CAMERA_COMMON`, no sdm type is of the mutants'
/// model), not the campaign type's 1.55 m at 10.
pub const FILTHY: Class = Class {
    key: "mp_mutant_filthy",
    base: "filthy mutant@e23",
    library: "mutant_horde",
    hitpoints: 70.0,
    slot: 4,
    face: Some(MUTANT_FILTHY_FACE),
    speeds: None,
    ammo: Some((SONIC_AMMO, 100)),
    extra_gun: None,
    guns: Some(&[JAX_IP]),
    camera: CAMERA_COMMON,
    skill: Some(CORPSE_EATER_DEMO),
    why: &[
        ("name", Origin::Guess, "mp_mutant_filthy h_fa73c507: the demo's"),
        ("gun", Origin::Guess, "the Jax-iP h_fc8af8aa and 100 Sonic Ammo h_ef0c4c1e, in place of the base type's Confed LZR-10: demo-own, the product owner's request of 2026-11-09"),
        ("hitpoints", Origin::Rule, "70: the standard tier, the ferals' 70 - 75"),
        ("face", Origin::Guess, "the demo's own derived portrait, not the data's (the data has one mutant face for all three, face_mutant.tga h_fc5f519f, with an opaque black box): that face with the box cleared, recoloured from the skin texture h_1ac02ad3 of its model's materials h_ed6ca9fc and h_000f0cc9 (e23, the head and hands' skin; the Molotov Mutant's model and textures are the same), fitted to the slot (`mutant_face`, made when the texture is asked for)"),
        ("skill", Origin::Guess, "the corpse eater, CSKILL_CORPSE_EATER 16, cost 3, recovery 1.5, max-stamina 100: demo-own, the product owner's design (#210). The cost and recovery are m03's Filthy Mutants' (objecttypes-m03_a.xml's h_1662a599); the class's own base type, e23's copy, has skill-set 0, and the game's eating is an AI set piece the level's script plays"),
        ("speeds", Origin::Guess, "the campaign type's 0 / 2.3 / 5 / 7 kept, as the Supermutant's"),
        ("camera", Origin::Rule, "3.2 1 0 alive, dead 3.5 5 0: the common set, carried by 9 of the 20 sdm types (the Ferguson four, Flint, the Shrike Heavy and Soldier, the two hounds); no sdm type is of the mutants' model"),
    ],
};

/// Every class #394 makes (the Terrorist Cell's four, then the Mutant Horde's): the types of their
/// own keys, in the order the NPC menu's data lists them. `Game::add_classes` adds those whose base
/// type is loaded, and a type of its own for each seat in another squad's library.
pub const CLASSES: [&Class; 8] = [&KINGMAN, &SUBVERSIVE, &RED_HAND, &TERRORIST, &SUPERMUTANT, &MOLOTOV, &UGLY, &FILTHY];

/// A class's seat in a squad (`Squad::members`): the place round the radar it takes there, and
/// why. The slot is the squad's, not the class's: the same character may sit at the top of one
/// squad and at the right of another, so a seat that isn't the class's home is a type of its own
/// with that slot (the footsteps and the skull follow it), made by `Class::apply_seated`.
#[derive(Clone, Copy, Debug)]
pub struct Seat {
    /// who sits there
    pub class: &'static Class,
    /// its HUD slot 1 - 4 in this squad: 1 Tex's place at the top, 2 Flint's at the right, 3
    /// Brutus's at the bottom, 4 Hawk's at the left
    pub slot: i64,
    /// where the seat came from, with `why`
    pub origin: Origin,
    /// why it sits there
    pub why: &'static str,
}

/// One of the demo's own squads of classes: its name, its library (the place "demo/<library>" and
/// the name a hook may call it by), the names a hook may also use, and its four seats. The game has
/// none of these (the data keeps the name "Kingman Posse" and six cut characters, no `<squad>`).
#[derive(Clone, Copy, Debug)]
pub struct Squad {
    /// its name in the menu when the data's string table has none for it: the demo's
    pub name: &'static str,
    /// the string-table id of its name where the data has one ("Kingman Posse" is h_0f3ad7a9, which
    /// the table keeps between "Ferguson Platoon" and "Seer Coven" with no squad behind it)
    pub name_id: Option<u32>,
    /// the library its members' types carry (`CharacterType::library`, the place
    /// "demo/<library>"), and the name a hook may call it by
    pub library: &'static str,
    /// other names a hook may call it by: #365's hook words and names, and the words of its own
    pub aliases: &'static [&'static str],
    /// its four members, one to each HUD slot
    pub members: [Seat; 4],
}

/// The Terrorist Cell: Edward Kingman, a Terrorist, a Red Hand and a Subversive, #365's "Kingman's
/// squad" renamed by the product owner (2026-11-07; demo-own: neither the name nor the squad is in
/// the data). The roster is #365's; the boss at the top and the Red Hand at the bottom are the
/// rule's seats, as the real squads' (Tex's place at slot 1 holds the Shadoon, the Hunter Lord, the
/// Militia); the two troopers' seats, the Terrorist right and the Subversive left, are the product
/// owner's (#400), demo-own. Its members
/// are their classes' own types (`mp_edward_kingman` ...): this is their home squad.
pub const TERRORIST_CELL: Squad = Squad {
    name: "Terrorist Cell",
    name_id: None,
    library: "terrorist_cell",
    aliases: &["kingman", "kingman's squad", "terrorist cell", "cell"],
    members: [
        Seat { class: &KINGMAN, slot: 1, origin: Origin::Rule, why: "the boss at the top, Tex's place: the real squads' boss at slot 1; the product owner's choice" },
        Seat { class: &TERRORIST, slot: 2, origin: Origin::Guess, why: "Flint's place at the right: demo-own, the product owner's seat (#400 swapped the two troopers; #394 had the Subversive here)" },
        Seat { class: &RED_HAND, slot: 3, origin: Origin::Rule, why: "Brutus's place at the bottom, the heavy fighter's" },
        Seat { class: &SUBVERSIVE, slot: 4, origin: Origin::Guess, why: "Hawk's place at the left: demo-own, the product owner's seat (#400; #394 had the Terrorist here)" },
    ],
};

/// The real Kingman Posse: the name is the string table's (h_0f3ad7a9, between "Ferguson Platoon"
/// and "Seer Coven", the six squad names kept alphabetically), the roster the cut squad's group
/// picture h_19f9d1f0 of the front end's `splash_screen` library: a black-armoured trooper with a
/// rifle, the Supermutant, a crouching pink mutant (`mutant1.tga`'s, taken as the Filthy Mutant: a
/// guess) and Kingman in his grey suit. The trooper is the Subversive (the product owner, #400:
/// both cut select pictures, `terrorist.tga` and `subversive.tga`, show black-armoured troopers
/// that look alike in the group picture, so which one it is stays a guess). **The data
/// has no seats for it** (no `<squad>` entry, no mp types, no library): Kingman at the top follows
/// the real squads' boss-at-slot-1 rule, the rest are the demo's own, the product owner's. Its
/// members are types of their own in this library, `<class key>@kingman_posse`, whose slot, skull
/// and footsteps follow the seat (the Filthy Mutant is at slot 4 in the Mutant Horde, 3 here).
pub const KINGMAN_POSSE: Squad = Squad {
    name: "Kingman Posse",
    name_id: Some(0x0F3A_D7A9),
    library: "kingman_posse",
    aliases: &["posse", "kingman posse"],
    members: [
        Seat { class: &KINGMAN, slot: 1, origin: Origin::Rule, why: "the boss at the top, Tex's place: the real squads' boss-at-slot-1 rule (the data has no seats for the Posse)" },
        Seat { class: &SUPERMUTANT, slot: 2, origin: Origin::Guess, why: "the right, Flint's place: demo-own, the product owner's seat for the picture's Supermutant" },
        Seat { class: &FILTHY, slot: 3, origin: Origin::Guess, why: "the bottom, Brutus's place: demo-own; the picture's crouching mutant is mutant1.tga's pink bare-chested one, taken as the Filthy Mutant (a guess)" },
        Seat { class: &SUBVERSIVE, slot: 4, origin: Origin::Guess, why: "the left, Hawk's place: demo-own; the picture's black-armoured trooper is the Subversive (the product owner's reading, #400; #394 took him for the Terrorist: both cut troopers look alike in the picture, so the label is a guess)" },
    ],
};

/// The Mutant Horde: the Supermutant and the three mutant variants of the data (Molotov, Ugly and
/// Filthy), #365's "Monsters" renamed by the product owner (2026-11-07; demo-own, in the real
/// squads' faction + group-noun style; the briefings call them "Mutants"). The roster is #365's;
/// the seats are the product owner's (#401: the Molotov Mutant at the top, the Supermutant at the
/// right, whose portrait fits a side slot, the Ugly at the bottom and the Filthy at the left),
/// demo-own: the real squads' rule puts the boss at the top. Its members are their classes' own
/// types: this is their home squad.
pub const MUTANT_HORDE: Squad = Squad {
    name: "Mutant Horde",
    name_id: None,
    library: "mutant_horde",
    aliases: &["monsters"],
    members: [
        Seat { class: &MOLOTOV, slot: 1, origin: Origin::Guess, why: "Tex's place at the top: demo-own, the product owner's seat (#401; #394 had the Supermutant here, the boss at the top as the real squads)" },
        Seat { class: &SUPERMUTANT, slot: 2, origin: Origin::Guess, why: "Flint's place at the right: demo-own, the product owner's seat (#401: the Supermutant's portrait fits a side slot)" },
        Seat { class: &UGLY, slot: 3, origin: Origin::Rule, why: "Brutus's place at the bottom" },
        Seat { class: &FILTHY, slot: 4, origin: Origin::Rule, why: "Hawk's place at the left, the lightest model's (ms_mutant2)" },
    ],
};

/// The demo's squads of classes, in the order of the squad menu's rows after the real squads.
pub const SQUADS: [&Squad; 3] = [&TERRORIST_CELL, &KINGMAN_POSSE, &MUTANT_HORDE];

impl Squad {
    /// The key of the type of `seat`'s class in this squad: the class's own key where this is its
    /// home squad and slot, else the type of its own, `<class key>@<library>`.
    pub fn type_key(&self, seat: &Seat) -> u32 {
        seat.class.seated_key(self.library, seat.slot)
    }

    /// The name of that type, for the logs: the class's key or `<class key>@<library>`.
    pub fn type_name(&self, seat: &Seat) -> String {
        seat.class.seated_name(self.library, seat.slot)
    }
}

/// The squad and seat whose type has key `type_key` (a class's own type is its home squad's
/// member, the others the seats in other squads), if there is one.
pub fn member_of(type_key: u32) -> Option<(&'static Squad, &'static Seat)> {
    SQUADS.iter().copied().find_map(|s| s.members.iter().find(|m| s.type_key(m) == type_key).map(|m| (s, m)))
}

/// The class whose type has key `type_key`: its own key's, or one of its seats' in another squad.
pub fn class_of(type_key: u32) -> Option<&'static Class> {
    CLASSES.iter().copied().find(|c| c.type_key() == type_key).or_else(|| member_of(type_key).map(|(_, m)| m.class))
}

impl Class {
    /// Its own type's key: the hash of its name, what `Game::character_types` and the hooks know it
    /// by.
    pub fn type_key(&self) -> u32 {
        h(self.key)
    }

    /// The name of its type seated in the squad `library` at `slot`: its key at home (its
    /// `library` and `slot`), else `<key>@<library>`.
    pub fn seated_name(&self, library: &str, slot: i64) -> String {
        if library == self.library && slot == self.slot { self.key.to_string() } else { format!("{}@{library}", self.key) }
    }

    /// The key of that type (the hash of `seated_name`).
    pub fn seated_key(&self, library: &str, slot: i64) -> u32 {
        h(&self.seated_name(library, slot))
    }

    /// The class as a character type at home (`apply_seated` in its own library and slot).
    pub fn apply(&self, base: &CharacterType) -> CharacterType {
        self.apply_seated(base, self.library, self.slot)
    }

    /// The class as a character type seated in the squad `library` at `slot`: `base` with the
    /// rule's columns applied (damage-multiplier 1, the player flag, no vulnerabilities or shield,
    /// kill reward 0, radar-range `RADAR_RANGE`, the stamina update on) and the class's own values
    /// (hit points, the slot as `footstep_type`, which is the footstep set too, with the slot's
    /// skull, the face, speeds, the camera block, the ammo item, the second gun `extra_gun` and its ammo
    /// item, and the guns `guns` in place of the base type's), under its key at that seat (`seated_key`), in the archive `CLASS_ARCHIVE` and that
    /// library. The skill block is the base's with the stamina update on and, for a class with a
    /// `skill` override (#407), that skill-set, cost, recovery, sounds and max-stamina; everything
    /// else is the base's.
    pub fn apply_seated(&self, base: &CharacterType, library: &str, slot: i64) -> CharacterType {
        let mut t = base.clone();
        t.key = self.seated_key(library, slot);
        t.archive = CLASS_ARCHIVE.to_string();
        t.library = library.to_string();
        t.hitpoints = self.hitpoints;
        t.damage_multiplier = 1.0;
        t.player_type = true;
        t.vulnerabilities.clear();
        t.shield = None;
        t.award.amount = 0.0;
        t.footstep_type = slot;
        t.skull = usize::try_from(slot - 1).ok().and_then(|i| SKULLS.get(i)).copied().unwrap_or(0);
        t.portrait = self.face.map_or(0, h);
        t.camera = Some(self.camera);
        if let Some(speeds) = self.speeds {
            t.speeds = speeds;
        }
        if let Some(gun) = self.extra_gun {
            // (an inventory has two weapon slots: a class made again from a made type adds none)
            if t.weapons.len() < 2 {
                t.weapons.push(gun.weapon);
            }
            if let Some(ammo) = gun.ammo.filter(|a| !t.items.iter().any(|&(i, _)| i == a.0)) {
                t.items.push(ammo);
            }
        }
        if let Some(guns) = self.guns {
            t.weapons = guns.to_vec();
        }
        if let Some((item, count)) = self.ammo {
            if !t.items.iter().any(|&(i, _)| i == item) {
                t.items.push((item, count));
            }
        }
        let mut skill = t.skill.take().unwrap_or_default();
        if let Some(o) = self.skill {
            (skill.skill_set, skill.cost, skill.recovery, skill.max_stamina) = (o.skill_set, o.cost, o.recovery, o.max_stamina);
            (skill.start_sound, skill.end_sound) = o.sounds;
            t.max_stamina = o.max_stamina;
        }
        skill.radar_range = RADAR_RANGE;
        skill.stamina_update = true;
        t.skill = Some(skill);
        t
    }

    /// The log's lines for this class as made (`t` is `apply_seated`'s result, `seat` the seat it was
    /// made for, if it has one): its values on the first, then one line for each value of its own,
    /// for the seat's slot and for each column of the rule with where it came from.
    pub fn report(&self, t: &CharacterType, seat: Option<&Seat>) -> Vec<String> {
        let skill = t.skill.clone().unwrap_or_default();
        let items: Vec<String> = t.items.iter().map(|(i, n)| format!("{n} x h_{i:08x}")).collect();
        let camera = t.camera.map_or("none".to_string(), |c| format!("{} m at {} deg (dead {} m)", c.walk[0], c.walk[1], c.dead[0]));
        let mut out = vec![format!(
            "hp {}, slot {}, face {}, skull h_{:08x}, skill-set {} cost {} recovery {}, speeds {:?}, camera {camera}, items [{}], damage-multiplier {}, player flag {}, vulnerabilities {}, shield {}, radar-range {}, kill reward {}",
            t.hitpoints, t.footstep_type, if t.portrait == 0 { "none".to_string() } else { format!("h_{:08x}", t.portrait) }, t.skull,
            skill.skill_set, skill.cost, skill.recovery, t.speeds, items.join(", "), t.damage_multiplier, t.player_type, t.vulnerabilities.len(),
            t.shield.as_ref().map_or("none".to_string(), |s| format!("type {}", s.kind)), skill.radar_range, t.award.amount)];
        let seat_why = seat.map(|s| ("slot", s.origin, s.why));
        for (name, origin, why) in self.why.iter().copied().chain(seat_why).chain(RULE.iter().copied()) {
            out.push(format!("    {name}: {} ({why})", origin.word()));
        }
        out
    }
}

/// The columns of the rule a type does NOT satisfy (empty: it fits), for a class and for the 20
/// sdm types and the 20 `dm` ones alike: the player flag, damage-multiplier 1, no vulnerabilities, no
/// shield, kill reward 0, radar-range of `RADAR_RANGE` or more, a HUD slot 1 - 4 with that slot's
/// skull, hit points in `HITPOINT_BAND`, max-stamina 100, a camera whose walk boom and dead camera
/// are in `CAMERA_BOOM_BAND` and `CAMERA_DEAD_BAND`. The sdm types that deviate do so with a
/// species trait (the Shrike Hound's 1.8, the Shrikes' stamina shields and factors, the hounds'
/// factors), which `character_type_report` prints beside them.
pub fn rule_gaps(t: &CharacterType) -> Vec<&'static str> {
    let mut gaps = vec![];
    if !t.player_type {
        gaps.push("player flag");
    }
    if t.damage_multiplier != 1.0 {
        gaps.push("damage-multiplier");
    }
    if !t.vulnerabilities.is_empty() {
        gaps.push("vulnerabilities");
    }
    if t.shield.is_some() {
        gaps.push("shield");
    }
    if t.award.amount != 0.0 {
        gaps.push("kill reward");
    }
    if t.skill.as_ref().is_none_or(|s| s.radar_range < RADAR_RANGE) {
        gaps.push("radar-range");
    }
    match usize::try_from(t.footstep_type - 1).ok().and_then(|i| SKULLS.get(i)) {
        None => gaps.push("HUD slot"),
        Some(&skull) if t.skull != skull => gaps.push("skull"),
        Some(_) => {}
    }
    if !(HITPOINT_BAND.0..=HITPOINT_BAND.1).contains(&t.hitpoints) {
        gaps.push("hit points");
    }
    if t.max_stamina != 100.0 {
        gaps.push("max-stamina");
    }
    let (boom, dead) = (CAMERA_BOOM_BAND.0..=CAMERA_BOOM_BAND.1, CAMERA_DEAD_BAND.0..=CAMERA_DEAD_BAND.1);
    if !t.camera.is_some_and(|c| boom.contains(&c.walk[0]) && dead.contains(&c.dead[0])) {
        gaps.push("camera");
    }
    gaps
}

impl Game {
    /// Adds the deathmatch classes whose base type is loaded with its model, each as a type of its
    /// own after every type of the data (`Class::apply_seated`; a name lookup without an archive
    /// still finds the data's first): the class itself in its own key (`CLASSES`), and for each
    /// seat in another squad's library (`SQUADS`) the class seated there, `<key>@<library>`. One
    /// already added is left as it is, so a second call adds nothing. For each type its name and
    /// its key, or why there is none.
    pub fn add_classes(&mut self) -> Vec<(String, Result<u32, String>)> {
        self.add_classes_with(Game::has_model)
    }

    /// `add_classes` with the test of a base type's model given (`Game::has_model`: its compound
    /// archetype is loaded), so that the rest can be tried without the data's objects.
    fn add_classes_with(&mut self, has_model: impl Fn(&Game, &CharacterType) -> bool) -> Vec<(String, Result<u32, String>)> {
        // each type to make: the class, the library and slot it is seated in
        let mut want: Vec<(&'static Class, &'static str, i64)> = CLASSES.iter().map(|&c| (c, c.library, c.slot)).collect();
        for squad in SQUADS {
            for seat in &squad.members {
                if squad.type_key(seat) != seat.class.type_key() {
                    want.push((seat.class, squad.library, seat.slot));
                }
            }
        }
        let mut out = vec![];
        for (class, library, slot) in want {
            let (name, key) = (class.seated_name(library, slot), class.seated_key(library, slot));
            if let Some(t) = self.character_types.get(&key) {
                // (added already; a type of the data with the class's name would be in its way)
                let r = if t.is_class() { Ok(key) } else { Err(format!("a type of the data is named {name}")) };
                out.push((name, r));
                continue;
            }
            let made = match self.find_character_type(class.base).and_then(|k| self.character_types.get(&k)) {
                None => Err(format!("its base type {} isn't loaded", class.base)),
                Some(base) if !has_model(self, base) => Err(format!("its base type {} has no model loaded", class.base)),
                Some(base) => Ok(class.apply_seated(base, library, slot)),
            };
            out.push((name, made.map(|t| {
                self.add_character_type(t);
                key
            })));
        }
        out
    }

    /// The classes and the deathmatch types of the loaded data (mp_common's `dm` library and the
    /// squad deathmatch `sdm_*` ones) with the columns of the rule each doesn't satisfy
    /// (`rule_gaps`), one line each, for `character_type_report`: how the 40 shipped types fit the
    /// rule the classes are made by, and that the classes do.
    pub fn class_report(&self) -> Vec<String> {
        let mut out = vec![];
        for &k in &self.character_type_order {
            let t = &self.character_types[&k];
            let class = class_of(k).filter(|_| t.is_class());
            let deathmatch = t.archive == "mp_common" && (t.library == "dm" || t.library.starts_with("sdm_"));
            if class.is_none() && !deathmatch {
                continue;
            }
            let gaps = rule_gaps(t);
            out.push(format!("{} h_{k:08x} [{}] \"{}\"{}: hp {}, slot {}, damage-multiplier {}, vulnerabilities {}, shield {}, radar-range {}, kill reward {} | rule gaps: {}",
                if class.is_some() { "class" } else if t.library == "dm" { "dm type" } else { "sdm type" }, t.place(), self.type_label(k),
                class.map_or(String::new(), |c| format!(" (from {})", c.base)),
                t.hitpoints, t.footstep_type, t.damage_multiplier, t.vulnerabilities.len(),
                t.shield.as_ref().map_or("none".to_string(), |s| format!("type {}", s.kind)),
                t.skill.as_ref().map_or(0.0, |s| s.radar_range), t.award.amount,
                if gaps.is_empty() { "none".to_string() } else { gaps.join(", ") }));
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bf::character::SkillData;
    use crate::bf::character_type::{Award, Shield};

    /// The campaign enemies' camera block: `1.55 10 0` in all seven modes but the snipe's `0.001 0 0`
    /// (the eight bases').
    fn campaign_camera() -> CameraOffsets {
        let c = [1.55, 10.0, 0.0];
        CameraOffsets { walk: c, run: c, dash: c, ready: c, h_e42d50a5: c, dead: c, snipe: [0.001, 0.0, 0.0] }
    }

    /// A campaign enemy of the Kingman kind: 311 hp, vulnerabilities, a reward, the radar's 30 m,
    /// no portrait, the campaign's speeds and camera and a gun with 5 grenades. m03_c's Kingman has
    /// no shield (shield-type 0) and the multiplier 1: the energy shield and the 0.62 multiplier
    /// here are made up for the test, so as to see the rule clear them.
    fn campaign() -> CharacterType {
        CharacterType {
            key: 0xFFDC_629E,
            archive: "m03_c".into(),
            library: "m03_c".into(),
            label: 7,
            mesh: 11,
            motion_set: 12,
            chatter: "edward_kingman_chatter".into(),
            hitpoints: 311.0,
            vulnerabilities: vec![(3, 0.5), (5, 0.8), (11, 0.042)],
            shield: Some(Shield { kind: 1, ..Default::default() }),
            damage_multiplier: 0.62,
            max_stamina: 100.0,
            weapons: vec![0xEAE7_F2E3],
            items: vec![(0xE8C6_7904, 5)],
            speeds: [0.0, 1.40755, 4.532335, 6.2],
            camera: Some(campaign_camera()),
            footstep_type: 0,
            award: Award { amount: 3500.0, to_squad: true, kind: 2, ..Default::default() },
            skill: Some(SkillData { skill_set: 5, max_stamina: 100.0, cost: 3.0, recovery: 1.5, radar_range: 30.0, ..Default::default() }),
            ..Default::default()
        }
    }

    #[test]
    fn the_rule_makes_a_campaign_type_a_class() {
        let base = campaign();
        assert_eq!(rule_gaps(&base), vec!["player flag", "damage-multiplier", "vulnerabilities", "shield", "kill reward", "radar-range", "HUD slot", "hit points", "camera"]);
        let t = KINGMAN.apply(&base);
        assert!(rule_gaps(&t).is_empty(), "{:?}", rule_gaps(&t));
        // the class's own values
        assert_eq!((t.key, t.archive.as_str(), t.library.as_str(), t.place().as_str()), (h("mp_edward_kingman"), "demo", "terrorist_cell", "demo/terrorist_cell"));
        assert_eq!((t.hitpoints, t.footstep_type, t.skull, t.portrait), (95.0, 1, h("skull_tex.tga"), h("face_kingman.tga")));
        assert!(t.is_class());
        // the rule's columns
        assert_eq!((t.damage_multiplier, t.player_type, t.award.amount), (1.0, true, 0.0));
        assert!(t.vulnerabilities.is_empty() && t.shield.is_none());
        let skill = t.skill.as_ref().expect("a skill block");
        assert_eq!((skill.radar_range, skill.stamina_update), (60.0, true));
        // the skill is the demo's Berserker (#407); the model and the rest are the campaign type's, and the
        // gun is its gun with the demo's second one after it
        assert_eq!((skill.skill_set, skill.cost, skill.recovery, skill.max_stamina), (3, 6.5, 1.2, 100.0));
        assert_eq!((skill.start_sound, skill.end_sound), (0xED74_92F4, 0xE00D_91FA));
        assert_eq!(t.weapons, vec![base.weapons[0], KMAN_AUTO]);
        assert_eq!((t.label, t.mesh, t.motion_set, t.chatter.as_str(), t.speeds), (7, 11, 12, "edward_kingman_chatter", base.speeds));
        // the camera is the class's, the common set for Kingman as for every class: 3.2 m at 1 degree in the live modes, 3.5 dead
        let cam = t.camera.expect("a camera block");
        let live = [3.2, 1.0, 0.0];
        assert_eq!((cam.walk, cam.run, cam.dash, cam.ready, cam.h_e42d50a5), (live, live, live, live, live));
        assert_eq!((cam.dead, cam.snipe), ([3.5, 5.0, 0.0], [0.001, 0.0, 0.0]));
        // the ammo item joins the grenades
        assert_eq!(t.items, vec![(0xE8C6_7904, 5), (HIGH_ROF_AMMO, 120)]);
        // the base is untouched
        assert_eq!((base.hitpoints, base.player_type, base.is_class()), (311.0, false, false));
    }

    #[test]
    fn a_class_with_a_derived_face_or_speeds_of_its_own_says_so() {
        let t = RED_HAND.apply(&campaign());
        assert_eq!((t.portrait, t.speeds, t.footstep_type, t.skull), (h("demo_face_red_hand.tga"), MILITIA_SPEEDS, 3, h("skull_brutus.tga")));
        // (the Saryl-45's Particle Ammo joins the grenades)
        assert_eq!(t.items, vec![(0xE8C6_7904, 5), (PARTICLE_AMMO, 100)]);
        assert!(rule_gaps(&t).is_empty());
        // a class doesn't add its ammo item twice
        let twice = KINGMAN.apply(&KINGMAN.apply(&campaign()));
        assert_eq!(twice.items.iter().filter(|&&(i, _)| i == HIGH_ROF_AMMO).count(), 1);
    }

    #[test]
    fn the_red_hand_the_subversive_kingman_the_terrorist_and_the_mutants_carry_a_skill_of_the_demos_choosing() {
        // the Red Hand and the Subversive: the Orbital Beam (9), cost 55, recovery 2.3, stamina 100 (#407), the rest of the block kept
        for class in [&RED_HAND, &SUBVERSIVE] {
            let t = class.apply(&campaign());
            let skill = t.skill.as_ref().expect("a skill block");
            assert_eq!((skill.skill_set, skill.cost, skill.recovery, skill.max_stamina, t.max_stamina), (9, 55.0, 2.3, 100.0, 100.0), "{}", class.key);
            assert_eq!((skill.radar_range, skill.stamina_update), (60.0, true));
            assert!(class.why.iter().any(|w| w.0 == "skill" && w.1 == Origin::Guess && w.2.contains("demo-own")), "{}", class.key);
        }
        // Edward Kingman: Tex's Berserker (3), the dm Tex's cost 6.5, recovery 1.2, in place of CSKILL_KINGMAN 5
        assert_eq!(KINGMAN.skill, Some(BERSERKER_DEMO));
        assert!(KINGMAN.why.iter().any(|w| w.0 == "skill" && w.1 == Origin::Guess && w.2.contains("demo-own")));
        // the Terrorist: Gunthar Ghent's Cloak (8), the dm Gunthar's cost 6, recovery 1.5, no sounds (#205), the rest of the block kept
        assert_eq!(TERRORIST.skill, Some(CLOAK_DEMO));
        let t = TERRORIST.apply(&campaign());
        let skill = t.skill.as_ref().expect("a skill block");
        assert_eq!((skill.skill_set, skill.cost, skill.recovery, skill.max_stamina, t.max_stamina), (8, 6.0, 1.5, 100.0, 100.0));
        assert_eq!((skill.start_sound, skill.end_sound, skill.radar_range, skill.stamina_update), (0, 0, 60.0, true));
        assert!(TERRORIST.why.iter().any(|w| w.0 == "skill" && w.1 == Origin::Guess && w.2.contains("demo-own")));
        // the Filthy and Ugly Mutants: the corpse eater (16), the m03 Filthy Mutants' cost 3 and recovery 1.5, stamina 100 (#210; the Ugly Mutant's too, 2026-11-09, as he shares the mutants' motion set)
        for class in [&FILTHY, &UGLY] {
            assert_eq!(class.skill, Some(CORPSE_EATER_DEMO), "{}", class.key);
            let t = class.apply(&campaign());
            let skill = t.skill.as_ref().expect("a skill block");
            assert_eq!((skill.skill_set, skill.cost, skill.recovery, skill.max_stamina, t.max_stamina), (16, 3.0, 1.5, 100.0, 100.0), "{}", class.key);
            assert_eq!((skill.start_sound, skill.end_sound, skill.radar_range, skill.stamina_update), (0, 0, 60.0, true), "{}", class.key);
            assert!(class.why.iter().any(|w| w.0 == "skill" && w.1 == Origin::Guess && w.2.contains("demo-own")), "{}", class.key);
        }
        // every other class keeps its base's block
        for class in CLASSES.iter().filter(|c| !["mp_red_hand", "mp_subversive_marine", "mp_edward_kingman", "mp_terrorist", "mp_mutant_filthy", "mp_mutant_ugly"].contains(&c.key)) {
            assert_eq!(class.skill, None, "{}", class.key);
        }
        // only Kingman carries a second gun, a second Kman Auto, demo-own
        for class in CLASSES.iter().filter(|c| c.key != "mp_edward_kingman") {
            assert_eq!(class.extra_gun, None, "{}", class.key);
        }
        assert_eq!(KINGMAN.extra_gun.map(|g| g.weapon), Some(KMAN_AUTO));
        assert!(KINGMAN.why.iter().any(|w| w.0 == "second gun" && w.1 == Origin::Guess && w.2.contains("demo-own")));
        // (a class applied twice doesn't double it)
        let twice = KINGMAN.apply(&KINGMAN.apply(&campaign()));
        assert_eq!(twice.weapons, vec![KMAN_AUTO, KMAN_AUTO]);
    }

    #[test]
    fn a_seat_in_another_squad_is_a_type_of_its_own_with_that_slot_its_skull_and_its_footsteps() {
        // the Supermutant is at the right of the Mutant Horde (slot 2, his own type) and of the Kingman
        // Posse (slot 2): there a type of its own in the Posse's library, as the squad's library differs
        let (posse, seat) = (&KINGMAN_POSSE, &KINGMAN_POSSE.members[1]);
        assert_eq!(seat.class.key, "mp_supermutant");
        assert_eq!(posse.type_name(seat), "mp_supermutant@kingman_posse");
        assert_eq!(posse.type_key(seat), h("mp_supermutant@kingman_posse"));
        assert_ne!(posse.type_key(seat), SUPERMUTANT.type_key());
        let base = campaign();
        let at_home = SUPERMUTANT.apply(&base);
        let seated = SUPERMUTANT.apply_seated(&base, posse.library, seat.slot);
        assert_eq!((at_home.key, at_home.footstep_type, at_home.skull, at_home.place().as_str()), (h("mp_supermutant"), 2, h("skull_flint.tga"), "demo/mutant_horde"));
        assert_eq!((seated.key, seated.footstep_type, seated.skull, seated.place().as_str()), (h("mp_supermutant@kingman_posse"), 2, h("skull_flint.tga"), "demo/kingman_posse"));
        // everything but the seat is the class's: the same hit points, face and rule
        assert_eq!((seated.hitpoints, seated.portrait), (at_home.hitpoints, at_home.portrait));
        assert!(rule_gaps(&seated).is_empty() && rule_gaps(&at_home).is_empty());
        // a seat that is the class's home is the class's own type
        assert_eq!(MUTANT_HORDE.type_key(&MUTANT_HORDE.members[0]), MOLOTOV.type_key());
        assert_eq!(MUTANT_HORDE.type_key(&MUTANT_HORDE.members[1]), SUPERMUTANT.type_key());
        assert_eq!(MUTANT_HORDE.type_key(&MUTANT_HORDE.members[3]), FILTHY.type_key());
        // the Filthy Mutant seated at the bottom of the Posse (slot 3, his home is the left, 4): his own slot's skull and footsteps there
        let filthy = &KINGMAN_POSSE.members[2];
        assert_eq!((KINGMAN_POSSE.type_name(filthy), filthy.slot, FILTHY.slot), ("mp_mutant_filthy@kingman_posse".to_string(), 3, 4));
        let there = FILTHY.apply_seated(&base, "kingman_posse", filthy.slot);
        assert_eq!((there.footstep_type, there.skull, there.portrait), (3, h("skull_brutus.tga"), h(MUTANT_FILTHY_FACE)));
        assert_eq!(TERRORIST_CELL.type_key(&TERRORIST_CELL.members[1]), TERRORIST.type_key());
        assert_eq!(TERRORIST_CELL.type_key(&TERRORIST_CELL.members[3]), SUBVERSIVE.type_key());
        // the Subversive seated at the left of the Posse is a type of its own, at the slot of his home squad
        let trooper = &KINGMAN_POSSE.members[3];
        assert_eq!(KINGMAN_POSSE.type_name(trooper), "mp_subversive_marine@kingman_posse");
        assert_eq!((trooper.class.key, trooper.slot), ("mp_subversive_marine", SUBVERSIVE.slot));
        assert!(member_of(h("mp_terrorist@kingman_posse")).is_none());
        // and the squad and seat of any of the twelve types are found by their key
        let (squad, member) = member_of(h("mp_supermutant@kingman_posse")).expect("the Posse's Supermutant");
        assert_eq!((squad.library, member.slot, member.class.key), ("kingman_posse", 2, "mp_supermutant"));
        assert!(class_of(h("mp_supermutant@kingman_posse")).is_some_and(|c| c.key == "mp_supermutant"));
        assert!(member_of(h("mp_tex")).is_none());
    }

    #[test]
    fn the_red_hands_face_keeps_the_helmet_and_takes_the_armour_from_the_red_hands_skin() {
        // a column of 32 texels, the library's bottom row first: the picture's rows 31 .. 17 are the body (indexes 0 - 14), 16 .. 0 the helmet
        let texel = |v: u8, a: u8| [v, v, v, a];
        let src: Vec<u8> = (0..32usize).flat_map(|i| texel(if i == 3 { 20 } else if i == 6 { 55 } else if i == 9 { 190 } else { 110 }, if i == 11 { 0 } else { 255 })).collect();
        let out = red_hand_face(&src, 1);
        assert_eq!(out.len(), src.len());
        // the helmet's texels (picture rows 0 - 16, indexes 15 - 31) and every alpha are the face's own
        assert_eq!(out[15 * 4..], src[15 * 4..]);
        assert!((0..32).all(|i| out[i * 4 + 3] == src[i * 4 + 3]));
        // the body's are the Red Hand's: a shadow dark brown, a mid-dark tone red, the mid-tones tan, a highlight light tan
        let at = |i: usize| [out[i * 4], out[i * 4 + 1], out[i * 4 + 2]];
        assert!(at(3)[0] < 90 && at(3)[0] > at(3)[1], "{:?}", at(3));
        assert!(at(6)[0] > at(6)[1] + 40 && at(6)[0] > at(6)[2] + 50, "{:?}", at(6));
        assert!(at(0)[0] > 150 && at(0)[0] > at(0)[2] + 40 && at(0)[1] > 110, "{:?}", at(0));
        assert!(at(9)[0] > 200, "{:?}", at(9));
        // no body texel is grey any more
        assert!((0..15).all(|i| at(i)[0] > at(i)[2]));
        // the Terrorist's own skin under the visor (#402, the demo's own): a 32 x 32 face of mauve texels (74, 52, 57) with a clear texel at
        // (6, 18); the patch (picture rows 17 - 20, columns 3 - 11) stays as it is, the ring round it is half armour, the rest armour
        let skin = [74u8, 52, 57, 255];
        let face: Vec<u8> = (0..32usize * 32).flat_map(|i| if i % 32 == 6 && 31 - i / 32 == 18 { [74, 52, 57, 0] } else { skin }).collect();
        let out = red_hand_face(&face, 32);
        let px = |x: usize, from_top: usize| {
            let i = ((31 - from_top) * 32 + x) * 4;
            [out[i], out[i + 1], out[i + 2], out[i + 3]]
        };
        for x in 3..12 {
            for r in 17..21 {
                if (x, r) != (6, 18) {
                    assert_eq!(px(x, r), skin, "the patch's ({x}, {r}) is the Terrorist's own");
                }
            }
        }
        // (a clear texel stays clear: no skin shows through)
        assert_eq!(px(6, 18)[3], 0);
        // far from the patch the same texel is armour, and the ring round the patch is the mean of the two (to a rounding)
        let armour = px(20, 28);
        assert_ne!(armour[..3], skin[..3]);
        for (x, r) in [(2, 19), (12, 19), (6, 21), (2, 21), (12, 20)] {
            let c = px(x, r);
            assert!((0..3).all(|j| (c[j] as i32 * 2 - (skin[j] as i32 + armour[j] as i32)).abs() <= 2), "the ring's ({x}, {r}): {c:?}");
        }
        assert_eq!(px(0, 24)[..3], armour[..3]);
        // the helmet rows (16 and up) are the Terrorist's own, row 16 included
        assert!([(5, 16), (10, 3)].iter().all(|&(x, r)| px(x, r) == skin));
        // it is named for the demo and made from the Terrorist's face
        assert_eq!(RED_HAND_FACE_SOURCE, "face_terrorist.tga");
        assert_eq!(h(RED_HAND_FACE_SOURCE), 0x0BE4_44B1);
        assert!(RED_HAND.why.iter().any(|w| w.0 == "face" && w.1 == Origin::Guess && w.2.contains("demo's own") && w.2.contains("h_e2aebd26")));
    }

    /// A 32 x 32 face in the layout of the data's mutant face, the picture's bottom row first: 27
    /// columns and 21 rows of opaque art at the picture's top left (columns 0 - 26, rows 11 - 31; the
    /// left part value 60, the right part value 200, grey), a pure-black box at its right edge
    /// (columns 20 - 26, rows 22 - 31) and a single pure-black texel inside the head (5, 19) that the box
    /// doesn't reach, the rest clear.
    fn mutant_source() -> Vec<u8> {
        (0..32usize * 32).flat_map(|i| {
            let (x, y) = (i % 32, i / 32);
            let inside = x < 27 && y >= 11;
            let v = if (20..27).contains(&x) && y >= 22 || (x, y) == (5, 19) { 0 } else if x < 13 { 60 } else { 200 };
            [v, v, v, if inside { 255 } else { 0 }]
        }).collect()
    }

    /// The texel (x, y) of a 32-wide image, bottom row first.
    fn texel(img: &[u8], x: usize, y: usize) -> [u8; 4] {
        let i = (y * 32 + x) * 4;
        [img[i], img[i + 1], img[i + 2], img[i + 3]]
    }

    #[test]
    fn the_mutant_faces_lose_the_black_box_keep_the_black_inside_the_head_and_fit_whole_inside_the_slot() {
        let src = mutant_source();
        // the texture is the data's: no skin, the art in its own colours
        let ugly = mutant_face(&src, 32, None);
        assert_eq!(ugly.len(), src.len());
        // the art is 27 x 21 (the box cleared leaves its bounding box alone: the box is at the art's edge but the head's
        // texels reach column 26 below it), so it is scaled by 24 / 27 to 24 x 18.7: columns 4 - 27, rows 3 - 21 from the top
        // (buffer rows 28 down to 10)
        // the box is clear (its centre, the art's (23, picture row 4), lands at column 24.9, row 7 from the top, buffer row 24)
        assert_eq!(texel(&ugly, 24, 24)[3], 0);
        assert_eq!(texel(&ugly, 25, 24)[3], 0);
        // a black texel the box's flood doesn't reach is the face's own shadow and stays opaque (darker than its neighbours' 60)
        let inner = texel(&ugly, 8, 17);
        assert!(inner[3] > 200 && inner[0] < 55, "{inner:?}");
        // the grey of the art keeps its colours, left dark and right light
        assert_eq!((texel(&ugly, 6, 15)[0], texel(&ugly, 22, 15)[0]), (60, 200));
        // nothing is cut and nothing touches the texture's border: the art stays inside columns 3 - 28 and rows 2 - 23 from the top
        let opaque: Vec<(usize, usize)> = (0..32 * 32).filter(|&i| ugly[i * 4 + 3] > 0).map(|i| (i % 32, 31 - i / 32)).collect();
        assert!(!opaque.is_empty());
        assert!(opaque.iter().all(|&(x, top)| (3..=28).contains(&x) && (2..=23).contains(&top)), "{opaque:?}");
        // and it is centred across the texture and the box's height and width are used: the width is the box's 24 (columns 4 - 27)
        let (lo, hi) = (opaque.iter().map(|p| p.0).min().expect("a texel"), opaque.iter().map(|p| p.0).max().expect("a texel"));
        assert_eq!((lo + hi) as i64 - 31, 0, "{lo} {hi}");
        assert!(hi - lo >= 23);
        // the art is never enlarged: an art smaller than the box keeps its size (the fixture's, cut to 8 x 8)
        let small: Vec<u8> = (0..32usize * 32).flat_map(|i| { let (x, y) = (i % 32, i / 32); [90, 90, 90, if x < 8 && y >= 24 { 255 } else { 0 }] }).collect();
        let fitted = mutant_face(&small, 32, None);
        let n = (0..32 * 32).filter(|&i| fitted[i * 4 + 3] > 128).count();
        assert!((60..=64).contains(&n), "{n}");
    }

    #[test]
    fn a_mutant_face_with_a_skin_takes_the_skins_colours_by_the_faces_own_shading() {
        let src = mutant_source();
        let filthy = mutant_face(&src, 32, Some(&FILTHY_SKIN));
        let molotov = mutant_face(&src, 32, Some(&MOLOTOV_SKIN));
        // the same shading: the dark left half darker than the light right half in both, in the skin's reds and browns
        let (dark_f, light_f) = (texel(&filthy, 6, 15), texel(&filthy, 22, 15));
        let (dark_m, light_m) = (texel(&molotov, 6, 15), texel(&molotov, 22, 15));
        for t in [dark_f, light_f, dark_m, light_m] {
            assert!(t[0] > t[1] && t[1] > t[2] || t[0] > t[2], "{t:?}");
            assert_eq!(t[3], 255);
        }
        assert!(dark_f[0] < light_f[0] && dark_m[0] < light_m[0]);
        // the two skins differ: the Molotov's fur is the lighter and more orange tan, the Filthy's the darker brown
        assert!(light_m[1] > light_f[1] + 30 && dark_m[0] > dark_f[0], "{light_m:?} {light_f:?} {dark_m:?} {dark_f:?}");
        assert_ne!(light_f, light_m);
        // the alpha is the cleared art's, the box clear in both
        assert_eq!((texel(&filthy, 24, 24)[3], texel(&molotov, 24, 24)[3]), (0, 0));
        // each ramp is eight colours from dark to light, the data's quantile means
        for skin in [&FILTHY_SKIN, &MOLOTOV_SKIN] {
            assert!(skin.windows(2).all(|w| w[0][0] <= w[1][0]));
        }
        // the portraits are named for the demo, derived from the data's mutant face, and said to be the demo's own
        assert_eq!(h(MUTANT_FACE_SOURCE), 0xFC5F_519F);
        for d in &DERIVED_FACES[1..] {
            assert_eq!(d.source, "face_mutant.tga");
            assert!(d.name.starts_with("demo_face_mutant_"));
            assert!(derived_face(h(d.name)).is_some_and(|f| f.name == d.name));
        }
        assert_eq!(derived_face(h("demo_face_red_hand.tga")).map(|f| f.source), Some("face_terrorist.tga"));
        assert!(derived_face(h("face_mutant.tga")).is_none());
        for c in [&MOLOTOV, &FILTHY, &UGLY] {
            assert!(c.why.iter().any(|w| w.0 == "face" && w.1 == Origin::Guess && w.2.contains("demo's own")), "{}", c.key);
            assert!(c.face.is_some_and(|f| derived_face(h(f)).is_some()), "{}", c.key);
        }
        assert!(FILTHY.why.iter().any(|w| w.0 == "face" && w.2.contains("h_1ac02ad3")));
        assert!(MOLOTOV.why.iter().any(|w| w.0 == "face" && w.2.contains("h_fe8edd48")));
        // the Supermutant's face is still the data's own, and the Posse's mutant is the Filthy's
        assert_eq!(SUPERMUTANT.face, Some("face_supermutant.tga"));
    }

    #[test]
    fn a_type_with_no_skill_block_gets_one_for_the_radar() {
        let mut base = campaign();
        base.skill = None;
        let t = TERRORIST.apply(&base);
        assert_eq!(t.skill.as_ref().map(|s| (s.radar_range, s.stamina_update)), Some((60.0, true)));
    }

    #[test]
    fn the_names_are_the_datas_hashes() {
        // the cut roster's unlock flags of default-global-gameflags.xml, three of them written as
        // hashes there (recovered by hashing the archetype names)
        assert_eq!(KINGMAN.type_key(), 0x1B39_DEA1);
        assert_eq!(SUBVERSIVE.type_key(), 0x1F33_0430);
        // the cut faces of common's texture library, and the slots' skulls (the sdm types' h_00c51907)
        assert_eq!(h("face_kingman.tga"), 0x1CC0_787C);
        assert_eq!(h("face_subversive.tga"), 0xF7F1_ACFF);
        assert_eq!(h("face_terrorist.tga"), 0x0BE4_44B1);
        assert_eq!(h("face_supermutant.tga"), 0xF05A_2AFC);
        assert_eq!(h("face_mutant.tga"), 0xFC5F_519F);
        assert_eq!(SKULLS, [0xFE6F_A2AF, 0x1834_3E47, 0x19DA_7869, 0x17BD_5B5C]);
        // the item hashes of the ammo boxes (ammo-type 2 and 8)
        assert_eq!((HIGH_ROF_AMMO, BIO_AMMO), (0x0273_5AD0, 0x01C1_965B));
    }

    #[test]
    fn the_eight_classes_are_the_product_owners() {
        // (hit points by the owner's choice of 2026-11-07, the home seats by the rule: the boss at slot 1, but the Mutant
        // Horde's the owner's, #401: the Molotov Mutant at the top, the Supermutant at the right)
        let table: Vec<(&str, f32, i64, Option<&str>)> = CLASSES.iter().map(|c| (c.key, c.hitpoints, c.slot, c.face)).collect();
        assert_eq!(table, vec![
            ("mp_edward_kingman", 95.0, 1, Some("face_kingman.tga")),
            ("mp_subversive_marine", 70.0, 4, Some("face_subversive.tga")),
            ("mp_red_hand", 55.0, 3, Some("demo_face_red_hand.tga")),
            ("mp_terrorist", 50.0, 2, Some("face_terrorist.tga")),
            ("mp_supermutant", 95.0, 2, Some("face_supermutant.tga")),
            ("mp_mutant_molotov", 55.0, 1, Some("demo_face_mutant_molotov.tga")),
            ("mp_mutant_ugly", 75.0, 3, Some("demo_face_mutant_ugly.tga")),
            ("mp_mutant_filthy", 70.0, 4, Some("demo_face_mutant_filthy.tga")),
        ]);
        // every class tells where its values came from; its home slots are taken once in each home squad
        for library in ["terrorist_cell", "mutant_horde"] {
            let mut slots: Vec<i64> = CLASSES.iter().filter(|c| c.library == library).map(|c| c.slot).collect();
            slots.sort();
            assert_eq!(slots, vec![1, 2, 3, 4], "{library}");
        }
        let mut keys: Vec<u32> = CLASSES.iter().map(|c| c.type_key()).collect();
        keys.sort();
        keys.dedup();
        assert_eq!(keys.len(), CLASSES.len());
        for c in CLASSES {
            assert!(c.why.iter().any(|w| w.0 == "hitpoints") && c.why.iter().any(|w| w.0 == "face"), "{}", c.key);
            assert!(c.why.iter().all(|w| w.0 != "slot"), "{}: the slot's reason is the seat's", c.key);
            assert!(c.base.contains('@'), "{}", c.key);
            assert!(class_of(c.type_key()).is_some_and(|k| k.key == c.key));
        }
        assert!(class_of(h("mp_tex")).is_none());
        // the hit points are tiers of the sdm types' band
        assert!(CLASSES.iter().all(|c| (HITPOINT_BAND.0..=HITPOINT_BAND.1).contains(&c.hitpoints)));
    }

    #[test]
    fn the_three_squads_are_the_product_owners_rosters_and_seats() {
        let roster = |s: &Squad| s.members.iter().map(|m| (m.class.key, m.slot, m.class.hitpoints)).collect::<Vec<_>>();
        assert_eq!(roster(&TERRORIST_CELL), vec![("mp_edward_kingman", 1, 95.0), ("mp_terrorist", 2, 50.0), ("mp_red_hand", 3, 55.0), ("mp_subversive_marine", 4, 70.0)]);
        // the Posse's picture: Kingman top, the Supermutant right, the Filthy Mutant bottom, the Subversive left
        assert_eq!(roster(&KINGMAN_POSSE), vec![("mp_edward_kingman", 1, 95.0), ("mp_supermutant", 2, 95.0), ("mp_mutant_filthy", 3, 70.0), ("mp_subversive_marine", 4, 70.0)]);
        // the Horde's ring (#401, the owner's): the Molotov Mutant top, the Supermutant right, the Ugly bottom, the Filthy left
        assert_eq!(roster(&MUTANT_HORDE), vec![("mp_mutant_molotov", 1, 55.0), ("mp_supermutant", 2, 95.0), ("mp_mutant_ugly", 3, 75.0), ("mp_mutant_filthy", 4, 70.0)]);
        assert_eq!(MUTANT_HORDE.members.map(|m| m.origin), [Origin::Guess, Origin::Guess, Origin::Rule, Origin::Rule]);
        // the squad menu's order, the names, the data's string for the Posse's, the libraries and the aliases
        assert_eq!(SQUADS.map(|s| s.name), ["Terrorist Cell", "Kingman Posse", "Mutant Horde"]);
        assert_eq!(SQUADS.map(|s| s.name_id), [None, Some(0x0F3A_D7A9), None]);
        assert_eq!(SQUADS.map(|s| s.library), ["terrorist_cell", "kingman_posse", "mutant_horde"]);
        assert!(TERRORIST_CELL.aliases.contains(&"kingman") && TERRORIST_CELL.aliases.contains(&"kingman's squad"));
        assert!(TERRORIST_CELL.aliases.contains(&"terrorist cell") && TERRORIST_CELL.aliases.contains(&"cell"));
        assert!(KINGMAN_POSSE.aliases.contains(&"posse") && KINGMAN_POSSE.aliases.contains(&"kingman posse"));
        assert!(MUTANT_HORDE.aliases.contains(&"monsters"));
        for s in SQUADS {
            // each squad has the four slots once, and every seat says why
            let mut slots: Vec<i64> = s.members.iter().map(|m| m.slot).collect();
            slots.sort();
            assert_eq!(slots, vec![1, 2, 3, 4], "{}", s.name);
            assert!(s.members.iter().all(|m| !m.why.is_empty()), "{}", s.name);
        }
        // a class's home seat is its own: its slot in its home squad, whose library it carries
        for c in CLASSES {
            let home = SQUADS.iter().find(|s| s.library == c.library).expect("a home squad");
            assert!(home.members.iter().any(|m| m.class.key == c.key && m.slot == c.slot), "{} at home", c.key);
        }
        // twelve types, one for each seat, with keys of their own
        let mut keys: Vec<u32> = SQUADS.iter().flat_map(|s| s.members.iter().map(|m| s.type_key(m))).collect();
        assert_eq!(keys.len(), 12);
        keys.sort();
        keys.dedup();
        assert_eq!(keys.len(), 12);
        // the Posse's Kingman at the top follows the real squads' rule (the data has no seats for it), the rest are demo-own
        assert_eq!(KINGMAN_POSSE.members.map(|m| m.origin), [Origin::Rule, Origin::Guess, Origin::Guess, Origin::Guess]);
        assert!(KINGMAN_POSSE.members[0].why.contains("boss-at-slot-1") && KINGMAN_POSSE.members[0].why.contains("no seats"));
        assert!(KINGMAN_POSSE.members[2].why.contains("mutant1.tga") && KINGMAN_POSSE.members[2].why.contains("guess"));
    }

    #[test]
    fn every_class_has_the_common_sdm_camera_by_the_rule() {
        // the common set: 3.2 m at 1 degree alive, 3.5 dead, snipe 0.001, the five live modes one value (as on all 40
        // deathmatch types), inside the bands of the 40
        let c = CAMERA_COMMON;
        assert_eq!((c.walk, c.dead, c.snipe), ([3.2, 1.0, 0.0], [3.5, 5.0, 0.0], [0.001, 0.0, 0.0]));
        assert!([c.run, c.dash, c.ready, c.h_e42d50a5].iter().all(|m| *m == c.walk));
        assert!((CAMERA_BOOM_BAND.0..=CAMERA_BOOM_BAND.1).contains(&c.walk[0]) && (CAMERA_DEAD_BAND.0..=CAMERA_DEAD_BAND.1).contains(&c.dead[0]));
        // by class: all eight the common set, by the rule (the bosses too, the product owner's choice over the Hunter Lord's
        // 4 m at 5), each saying so in its why
        let table: Vec<(&str, f32, Origin)> = CLASSES.iter().map(|c| (c.key, c.camera.walk[0], c.why.iter().find(|w| w.0 == "camera").expect("a camera reason").1)).collect();
        assert_eq!(table, vec![
            ("mp_edward_kingman", 3.2, Origin::Rule),
            ("mp_subversive_marine", 3.2, Origin::Rule),
            ("mp_red_hand", 3.2, Origin::Rule),
            ("mp_terrorist", 3.2, Origin::Rule),
            ("mp_supermutant", 3.2, Origin::Rule),
            ("mp_mutant_molotov", 3.2, Origin::Rule),
            ("mp_mutant_ugly", 3.2, Origin::Rule),
            ("mp_mutant_filthy", 3.2, Origin::Rule),
        ]);
        assert!(CLASSES.iter().all(|k| k.camera.walk == c.walk && k.camera.dead == c.dead && k.camera.snipe == c.snipe));
        // the bosses' reason names the block they weren't given, the militia terrorists' the Militia they copy
        assert!(KINGMAN.why.iter().any(|w| w.0 == "camera" && w.2.contains("chose it over the sdm Hunter Lord")));
        assert!(SUPERMUTANT.why.iter().any(|w| w.0 == "camera" && w.2.contains("chose it over the sdm Hunter Lord")));
        assert!(TERRORIST.why.iter().any(|w| w.0 == "camera" && w.2.contains("Militia h_1563301d")));
        // a class made from a campaign type takes its camera, whatever the base's was, in every seat
        let base = campaign();
        for c in CLASSES {
            assert_eq!(c.apply(&base).camera.map(|k| k.walk), Some(c.camera.walk), "{}", c.key);
            assert_eq!(c.apply_seated(&base, "kingman_posse", 2).camera.map(|k| k.dead), Some(c.camera.dead), "{}", c.key);
        }
    }

    #[test]
    fn a_squad_deathmatch_type_fits_the_rule() {
        // the sdm Militia (Ferguson Platoon, Tex's place): 80 hp, the player flag, slot 1, no
        // factors, radar 60, kill reward 0, Tex's speeds
        let militia = CharacterType {
            hitpoints: 80.0,
            damage_multiplier: 1.0,
            player_type: true,
            footstep_type: 1,
            skull: h("skull_tex.tga"),
            portrait: 0x0B2E_FE60,
            max_stamina: 100.0,
            speeds: MILITIA_SPEEDS,
            camera: Some(CAMERA_COMMON),
            skill: Some(SkillData { skill_set: 6, max_stamina: 100.0, cost: 20.0, recovery: 1.0, radar_range: 60.0, stamina_update: true, ..Default::default() }),
            ..Default::default()
        };
        assert!(rule_gaps(&militia).is_empty());
        // a Shrike Hound's species: 1.8, factors and 180 m radar (the radar is above the rule's 60: no gap)
        let hound = CharacterType { damage_multiplier: 1.8, vulnerabilities: vec![(4, 0.1)], skill: Some(SkillData { radar_range: 180.0, ..Default::default() }), ..militia.clone() };
        assert_eq!(rule_gaps(&hound), vec!["damage-multiplier", "vulnerabilities"]);
        // a slot out of 1 - 4 and a skull that isn't the slot's
        assert!(rule_gaps(&CharacterType { footstep_type: 0, ..militia.clone() }).contains(&"HUD slot"));
        assert!(rule_gaps(&CharacterType { skull: h("skull_hawk.tga"), ..militia.clone() }).contains(&"skull"));
        // a camera under the sdm types' 2.9 m, or none, or a dead camera outside theirs
        assert_eq!(rule_gaps(&CharacterType { camera: Some(campaign_camera()), ..militia.clone() }), vec!["camera"]);
        assert_eq!(rule_gaps(&CharacterType { camera: None, ..militia.clone() }), vec!["camera"]);
        assert_eq!(rule_gaps(&CharacterType { camera: Some(CameraOffsets { dead: [1.55, 10.0, 0.0], ..CAMERA_COMMON }), ..militia }), vec!["camera"]);
    }

    #[test]
    fn the_report_says_where_each_value_came_from() {
        let t = KINGMAN.apply(&campaign());
        let lines = KINGMAN.report(&t, Some(&TERRORIST_CELL.members[0]));
        assert!(lines[0].starts_with("hp 95, slot 1, face h_1cc0787c, skull h_fe6fa2af, skill-set 3 cost 6.5 recovery 1.2"), "{}", lines[0]);
        assert!(lines[0].contains("radar-range 60") && lines[0].contains("damage-multiplier 1") && lines[0].contains("kill reward 0"), "{}", lines[0]);
        assert!(lines.iter().any(|l| l.starts_with("    hitpoints: rule (")));
        assert!(lines.iter().any(|l| l.starts_with("    slot: rule (the boss at the top")));
        assert!(lines.iter().any(|l| l.starts_with("    face: real (")));
        assert!(lines.iter().any(|l| l.starts_with("    speeds: guess (")));
        assert!(lines.iter().any(|l| l.starts_with("    radar-range: rule (")));
        // (one line a value of the class's, one for the seat, one for each column of the rule)
        assert_eq!(lines.len(), 1 + KINGMAN.why.len() + 1 + RULE.len());
        // a class with no seat reports none of the seat's
        assert_eq!(KINGMAN.report(&t, None).len(), 1 + KINGMAN.why.len() + RULE.len());
        // the Posse's seat for the Subversive is a guess, and says so
        let posse = KINGMAN_POSSE.members[3];
        let lines = SUBVERSIVE.report(&SUBVERSIVE.apply_seated(&campaign(), "kingman_posse", 4), Some(&posse));
        assert!(lines.iter().any(|l| l.starts_with("    slot: guess (the left, Hawk's place")), "{lines:?}");
    }

    /// A data-less game holding the campaign Kingman as m03_c defines it (label string 7 "Edward
    /// Kingman").
    fn game_with_kingman() -> Game {
        let mut game = Game::empty(crate::bf::bxml::SchemaSet::default());
        game.strings.insert(7, "Edward Kingman".into());
        game.add_character_type(campaign());
        game
    }

    #[test]
    fn the_classes_whose_bases_are_loaded_are_added_once_after_the_data() {
        let mut game = game_with_kingman();
        let made = game.add_classes_with(|_, _| true);
        // eight classes and the four seats in the Posse's library
        assert_eq!(made.len(), CLASSES.len() + 4);
        // only Kingman's base is loaded: his own type and his seat in the Posse are made, the rest say so
        let ok: Vec<&str> = made.iter().filter(|r| r.1.is_ok()).map(|r| r.0.as_str()).collect();
        assert_eq!(ok, vec!["mp_edward_kingman", "mp_edward_kingman@kingman_posse"]);
        assert!(made.iter().filter(|r| r.1.is_err()).all(|r| r.1.as_ref().is_err_and(|e| e.contains("isn't loaded"))));
        let t = game.character_type(h("mp_edward_kingman")).expect("added");
        assert!(t.is_class() && t.player_type && t.hitpoints == 95.0 && rule_gaps(t).is_empty());
        let there = game.character_type(h("mp_edward_kingman@kingman_posse")).expect("seated in the Posse");
        assert_eq!((there.place().as_str(), there.footstep_type, there.hitpoints), ("demo/kingman_posse", 1, 95.0));
        // they come after the data's types: a name without an archive finds the data's first, `@demo` the class
        assert_eq!(game.character_type_order, vec![0xFFDC_629E, h("mp_edward_kingman"), h("mp_edward_kingman@kingman_posse")]);
        assert_eq!(game.find_character_type("edward kingman"), Some(0xFFDC_629E));
        assert_eq!(game.find_character_type("edward kingman@m03_c"), Some(0xFFDC_629E));
        assert_eq!(game.find_character_type("edward kingman@demo"), Some(h("mp_edward_kingman")));
        assert_eq!(game.find_character_type("edward kingman@kingman_posse"), Some(h("mp_edward_kingman@kingman_posse")));
        assert_eq!(game.find_character_type("mp_edward_kingman"), Some(h("mp_edward_kingman")));
        assert_eq!(game.find_character_type("mp_edward_kingman@kingman_posse"), Some(h("mp_edward_kingman@kingman_posse")));
        assert_eq!(game.type_label(h("mp_edward_kingman")), "Edward Kingman");
        // a second call adds nothing and still reports them
        let again = game.add_classes_with(|_, _| true);
        assert_eq!(game.character_type_order.len(), 3);
        assert!(again.iter().any(|r| r.0 == "mp_edward_kingman" && r.1 == Ok(h("mp_edward_kingman"))));
        // the dump's report lists the classes (and no deathmatch type: none is loaded)
        let lines = game.class_report();
        assert_eq!(lines.len(), 2);
        assert!(lines[0].starts_with("class h_1b39dea1 [demo/terrorist_cell] \"Edward Kingman\" (from edward kingman@m03_c): hp 95, slot 1"), "{}", lines[0]);
        assert!(lines[1].starts_with("class h_") && lines[1].contains("[demo/kingman_posse]"), "{}", lines[1]);
        assert!(lines.iter().all(|l| l.ends_with("rule gaps: none")), "{lines:?}");
    }

    #[test]
    fn the_armed_classes_carry_their_gun_in_place_of_the_base_types() {
        for (class, gun, ammo, count) in [
            (&TERRORIST, ION_BEAMER, ENERGY_AMMO, 100),
            (&RED_HAND, SARYL_45, PARTICLE_AMMO, 100),
            (&SUBVERSIVE, BOWER_20, SHOTGUN_AMMO, 40),
            (&UGLY, FOLEY_356, AMMO_11MM, 25),
            (&FILTHY, JAX_IP, SONIC_AMMO, 100),
        ] {
            let base = campaign();
            let t = class.apply(&base);
            // the base's Kman Auto is gone, the class's gun in its place, with its ammo after the grenades
            assert_eq!(t.weapons, vec![gun], "{}", class.key);
            assert_eq!(t.items.last(), Some(&(ammo, count)), "{}", class.key);
            assert_eq!(class.guns, Some(&[gun][..]));
            // the base is untouched
            assert_eq!(base.weapons, vec![KMAN_AUTO]);
        }
    }

    #[test]
    fn the_report_compares_the_dm_library_with_the_rule_too() {
        // (#395's cut characters will be compared with the 20 types of mp_common's dm library as well)
        let mut game = Game::empty(crate::bf::bxml::SchemaSet::default());
        game.strings.insert(1, "Tex".into());
        let dm = CharacterType { key: 1, archive: "mp_common".into(), library: "dm".into(), label: 1, hitpoints: 70.0, damage_multiplier: 1.0, player_type: true,
                                 footstep_type: 1, skull: h("skull_tex.tga"), max_stamina: 100.0, camera: Some(camera(3.9, 0.9, 3.5)),
                                 skill: Some(SkillData { radar_range: 60.0, ..Default::default() }), ..Default::default() };
        let other = CharacterType { key: 2, archive: "mp_common".into(), library: "objects-dm".into(), ..dm.clone() };
        game.add_character_type(dm);
        game.add_character_type(other);
        let lines = game.class_report();
        assert_eq!(lines.len(), 1);
        assert!(lines[0].starts_with("dm type h_00000001 [mp_common/dm] \"Tex\": hp 70, slot 1") && lines[0].ends_with("rule gaps: none"), "{}", lines[0]);
    }

    #[test]
    fn a_base_without_its_model_or_a_type_in_the_way_is_skipped_with_a_reason() {
        let mut game = game_with_kingman();
        let made = game.add_classes_with(|_, _| false);
        let why = made.iter().find(|r| r.0 == "mp_edward_kingman").and_then(|r| r.1.clone().err()).expect("skipped");
        assert!(why.contains("no model") && why.contains("edward kingman@m03_c"), "{why}");
        assert_eq!(game.character_types.len(), 1);
        // a type of the data named like a class (none is, in the data) is not overwritten
        game.add_character_type(CharacterType { key: h("mp_terrorist"), archive: "e01".into(), ..Default::default() });
        let made = game.add_classes_with(|_, _| true);
        let terrorist = made.iter().find(|r| r.0 == "mp_terrorist").expect("listed");
        assert!(terrorist.1.as_ref().is_err_and(|e| e.contains("a type of the data is named")), "{:?}", terrorist.1);
        assert!(!game.character_type(h("mp_terrorist")).is_some_and(|t| t.is_class()));
    }
}
