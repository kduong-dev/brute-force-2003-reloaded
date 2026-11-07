//! Deathmatch classes (#394): a character type made from a campaign type by the rule the squad
//! deathmatch types follow, for the characters the game's multiplayer has no type for.
//!
//! The shipped game's 20 deathmatch characters each have a type of their own in mp_common's
//! libraries (`dm`, `sdm_bf` ... `sdm_ferg`), and each differs from the campaign version of the
//! same character in the same few fields (the analyst's diff of all 20 against every campaign
//! copy, `todo/drafts/done/sdm-classes-kingman-monsters.md` section 1.3; `rule_gaps` and
//! `character_type_report` check it on the loaded data):
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
//!    _brutus, _hawk) and **a portrait** of the character's own;
//!  - **radar-range** 60 m or more, **kill reward** 0, **max-stamina** 100 and the skill block's
//!    numbers kept where the type has no skill of its own;
//!  - **speeds** Tex's 3 / 3.6 / 7.8 / 8.5 for a human on ms_militia (the Ferguson four), a
//!    species' own where it has its own clips;
//!  - **the gun** the campaign type carries, with its ammo item.
//!
//! A `Class` is that rule for one character, as a record: the campaign type it starts from
//! (`base`, as `Game::find_character_type` names it), the values that are the class's own
//! (hit points, slot, face, speeds, ammo) and where each came from (`Origin`). `Class::apply`
//! makes the type, `Game::add_classes` adds the loaded ones to `Game::character_types` under
//! the class's key with the place "demo/<library>" (`CLASS_ARCHIVE`), so the NPC and squad
//! menus, `BF_TEST_NPC`, the HUD's portraits, the damage and the dump read a class as they read
//! any type. Everything else stays the base type's: model, clips, face block, chatter, camera,
//! capsule, decals, death effect, AI block, weapons (#357's chatter rule; the camera and the AI
//! block are the open differences, below).
//!
//! The classes of #394 are the eight members of the demo's two own squads (the Kingman Posse
//! and the Mutant Horde, play_testsquad.rs); #395 adds the cut characters the data has only
//! pictures for. **Real** values are the data's (the base type's own, or the cut roster's
//! remains); **rule** values are the demo's by the rule above; **guess** values are the demo's
//! with no rule behind them. The rule and the guesses are demo-own.
//!
//! What a class does not copy from the sdm types, because the rule table doesn't list it:
//! their camera block (all 20 have a 2.9 - 4 m player camera; a class keeps its base's, the
//! campaign enemies' 1.55 m) and their shared AI block (alignment 0, vision 80, half-fov 90 ...:
//! the demo's squad AI is its own).

use super::character::Game;
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

/// The High ROF Ammo item (objecttypes `<inventory>` entry with function-type 14 and ammo-type
/// h_e5f51266 = 2, name h_02735ad0, stack-limit 600): the reserve of the Kman Auto and the
/// MK-ASLT, which the sdm Militia and Shrike Soldier carry 120 of.
pub const HIGH_ROF_AMMO: u32 = 0x0273_5AD0;
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
    /// the squad it belongs to, its type's `library` (the place reads "demo/<library>")
    pub library: &'static str,
    /// its hit points: a tier of the sdm types' band (`HITPOINT_BAND`)
    pub hitpoints: f32,
    /// its HUD slot 1 - 4, `h_072ab4af`: the place round the radar it takes (play_testsquad.rs
    /// `seat`) and its skull (`SKULLS`)
    pub slot: i64,
    /// its portrait: a face texture of common's library (32 x 32), None for the "?"
    pub face: Option<&'static str>,
    /// its speeds when they aren't the base's (crouch walk, walk, run, dash)
    pub speeds: Option<[f32; 4]>,
    /// the ammo item its gun uses and how many it carries, added to the base's items
    pub ammo: Option<(u32, i64)>,
    /// where each of those values came from (`report` prints them)
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

/// Edward Kingman, the Kingman Posse's boss: the campaign type of m03_c, Kingman's own level
/// (311 hp), with:
///  - hit points 95 (rule: the boss tier, the Shadoon's; the product owner's choice);
///  - HUD slot 1, Tex's place at the top (rule: the boss at the top, as the Shadoon and the
///    Hunter Lord; the product owner's choice);
///  - face_kingman.tga (real: one of the cut faces the data keeps in common's library);
///  - the skill CSKILL_KINGMAN 5, stamina cost 3, recovery 1.5 (real: the campaign type's; the
///    briefcase, #201, isn't built, so no skill is played);
///  - speeds 0 / 1.41 / 4.53 / 6.2 kept (guess: no sdm type is of his model; the sdm bosses
///    run 5 - 5.5 and dash 7 - 7.2, the Hunter Lord and the Shadoon, who keeps the campaign's
///    walk and trims its run and dash from 6 / 7.5);
///  - the Kman Auto (real), 5 Gas grenades (real: `h_e8c67904`), and 120 High ROF Ammo, the
///    Kman Auto's ammo-type 2 (rule: the campaign gun plus its ammo item; guess: 120, the sdm
///    Militia's and Shrike Soldier's count).
pub const KINGMAN: Class = Class {
    key: "mp_edward_kingman",
    base: "edward kingman@m03_c",
    library: "kingman_posse",
    hitpoints: 95.0,
    slot: 1,
    face: Some("face_kingman.tga"),
    speeds: None,
    ammo: Some((HIGH_ROF_AMMO, 120)),
    why: &[
        ("name", Origin::Real, "mp_edward_kingman h_1b39dea1: the cut roster's unlock flag in default-global-gameflags.xml"),
        ("hitpoints", Origin::Rule, "95: the boss tier, the sdm Shadoon's; the product owner's choice"),
        ("slot", Origin::Rule, "1: the boss at the top, Tex's place; the product owner's choice"),
        ("face", Origin::Real, "face_kingman.tga h_1cc0787c: a cut face in common's textures"),
        ("skill", Origin::Real, "CSKILL_KINGMAN 5, cost 3, recovery 1.5: the campaign type's (the briefcase, #201, isn't built)"),
        ("speeds", Origin::Guess, "the campaign type's 0 / 1.41 / 4.53 / 6.2 kept: no sdm type is his model; the sdm bosses run 5 - 5.5 and dash 7 - 7.2"),
        ("ammo", Origin::Guess, "High ROF Ammo for the Kman Auto (rule); 120 as the sdm Militia's"),
    ],
};

/// The Subversive, the Kingman Posse's senior terrorist: m03_c's campaign type (36 hp; m08's 88),
/// with hit points 70 (rule: the standard tier, Tex's), HUD slot 2, Flint's place at the right
/// (rule: the draft's seat; the product owner's choice), face_subversive.tga (real: a cut face),
/// Tex's speeds (rule: the Ferguson four's, on the same ms_militia) and the Confed LZR-23 (real;
/// the LZR line has no ammo item). Skill-set 0, cost 3, recovery 1.5 (real).
pub const SUBVERSIVE: Class = Class {
    key: "mp_subversive_marine",
    base: "subversive@m03_c",
    library: "kingman_posse",
    hitpoints: 70.0,
    slot: 2,
    face: Some("face_subversive.tga"),
    speeds: Some(MILITIA_SPEEDS),
    ammo: None,
    why: &[
        ("name", Origin::Real, "mp_subversive_marine h_1f330430: the cut roster's unlock flag"),
        ("hitpoints", Origin::Rule, "70: the standard tier, Tex's; the senior terrorist (36 hp to the Terrorist's 25)"),
        ("slot", Origin::Rule, "2: Flint's place at the right"),
        ("face", Origin::Real, "face_subversive.tga h_f7f1acff: a cut face in common's textures"),
        ("skill", Origin::Real, "none: skill-set 0, cost 3, recovery 1.5, the campaign type's"),
        ("speeds", Origin::Rule, "Tex's 3 / 3.6 / 7.8 / 8.5: the Ferguson four's, on the same ms_militia"),
    ],
};

/// The Red Hand, the Kingman Posse's heavy fighter: e05's campaign type (31 hp; e13's 128, the
/// tutorial's 23), with hit points 55 (rule: the light-armoured tier, the Shrike Heavy's), HUD
/// slot 3, Brutus's place at the bottom (rule: the heavy fighter's), Tex's speeds (rule), the
/// Confed LZR-23 (real) and **no face** (real: the data has none for the Red Hand, so its place
/// shows the "?"). The key is the demo's own (guess: the data has no cut name for it).
pub const RED_HAND: Class = Class {
    key: "mp_red_hand",
    base: "red hand@e05",
    library: "kingman_posse",
    hitpoints: 55.0,
    slot: 3,
    face: None,
    speeds: Some(MILITIA_SPEEDS),
    ammo: None,
    why: &[
        ("name", Origin::Guess, "mp_red_hand h_ebefd26c: the demo's, the data has no cut name for it"),
        ("hitpoints", Origin::Rule, "55: the light-armoured tier, the Shrike Heavy's"),
        ("slot", Origin::Rule, "3: Brutus's place at the bottom, the heavy fighter's"),
        ("face", Origin::Real, "none: the data has no face for the Red Hand; the radar shows the \"?\""),
        ("skill", Origin::Real, "none: skill-set 0, cost 3, recovery 1.5, the campaign type's"),
        ("speeds", Origin::Rule, "Tex's 3 / 3.6 / 7.8 / 8.5: the Ferguson four's, on the same ms_militia"),
    ],
};

/// The Terrorist, the Kingman Posse's grunt: m03_c's campaign type (25 hp, the lowest of the
/// three), with hit points 50 (rule: the grunt tier, Hawk's and the Shrike Soldier's), HUD slot
/// 4, Hawk's place at the left (rule: the lightest at Hawk's place, as the hounds, Hawk and
/// Ghent), face_terrorist.tga (real: a cut face), Tex's speeds (rule) and the Confed LZR-23
/// (real).
pub const TERRORIST: Class = Class {
    key: "mp_terrorist",
    base: "terrorist@m03_c",
    library: "kingman_posse",
    hitpoints: 50.0,
    slot: 4,
    face: Some("face_terrorist.tga"),
    speeds: Some(MILITIA_SPEEDS),
    ammo: None,
    why: &[
        ("name", Origin::Real, "mp_terrorist h_03589fbb: the cut roster's unlock flag"),
        ("hitpoints", Origin::Rule, "50: the grunt tier, Hawk's and the Shrike Soldier's"),
        ("slot", Origin::Rule, "4: Hawk's place at the left, the lightest member's"),
        ("face", Origin::Real, "face_terrorist.tga h_0be444b1: a cut face in common's textures"),
        ("skill", Origin::Real, "none: skill-set 0, cost 3, recovery 1.5, the campaign type's"),
        ("speeds", Origin::Rule, "Tex's 3 / 3.6 / 7.8 / 8.5: the Ferguson four's, on the same ms_militia"),
    ],
};

/// The Supermutant, the Mutant Horde's boss: e23's campaign type (669 hp), with hit points 95
/// (rule: the boss tier), HUD slot 1 (rule: the boss at the top), face_supermutant.tga (real: a
/// cut face), the skill CSKILL_BOSS_SUPERMUTANT 17, cost 1, recovery 1.5 (real: the campaign
/// type's; the ground pound, #212, isn't built), the MNR-7 Bio, 5 Energy grenades and its
/// death effect h_e19a8348 (real), 60 Bio Ammo (rule: the gun's ammo-type 8 item; guess: 60, 0.3
/// of its stack of 200, about the share the sdm types carry) and speeds 0 / 2.3 / 5 / 7 kept
/// (guess: the sdm bosses run 5 - 5.5 and dash 7 - 7.2, the Hunter Lord and the Shadoon).
pub const SUPERMUTANT: Class = Class {
    key: "mp_supermutant",
    base: "supermutant@e23",
    library: "mutant_horde",
    hitpoints: 95.0,
    slot: 1,
    face: Some("face_supermutant.tga"),
    speeds: None,
    ammo: Some((BIO_AMMO, 60)),
    why: &[
        ("name", Origin::Real, "mp_supermutant h_16122b66: the cut roster's unlock flag"),
        ("hitpoints", Origin::Rule, "95: the boss tier"),
        ("slot", Origin::Rule, "1: the boss at the top, Tex's place"),
        ("face", Origin::Real, "face_supermutant.tga h_f05a2afc: a cut face in common's textures"),
        ("skill", Origin::Real, "CSKILL_BOSS_SUPERMUTANT 17, cost 1, recovery 1.5: the campaign type's (the ground pound, #212, isn't built)"),
        ("speeds", Origin::Guess, "the campaign type's 0 / 2.3 / 5 / 7 kept: the sdm bosses run 5 - 5.5 and dash 7 - 7.2"),
        ("ammo", Origin::Guess, "Bio Ammo for the MNR-7 Bio (rule); 60, about the share of its stack the sdm types carry"),
    ],
};

/// The Molotov Mutant, the Mutant Horde's specialist: e23's campaign type (55 hp), with hit
/// points 55 (rule: the specialist tier, Flint's; a self-destruct type is spent on use), HUD slot
/// 2, Flint's place at the right (rule, and where #365 put it), the shared mutant face (guess,
/// the product owner's choice: the data has one mutant face), the skill CSKILL_MOLOTOV_MUTANT 20,
/// cost 3, recovery 1.5 (real: the campaign type's; the self-destruct, #211, isn't built), the
/// Confed LZR-10 and 2 Gas grenades (real), speeds kept (guess, as the Supermutant's). The key is
/// the demo's own (guess).
pub const MOLOTOV: Class = Class {
    key: "mp_mutant_molotov",
    base: "molotov mutant@e23",
    library: "mutant_horde",
    hitpoints: 55.0,
    slot: 2,
    face: Some("face_mutant.tga"),
    speeds: None,
    ammo: None,
    why: &[
        ("name", Origin::Guess, "mp_mutant_molotov h_02f7069a: the demo's, the cut roster names two of the three mutants at most"),
        ("hitpoints", Origin::Rule, "55: the specialist tier, Flint's"),
        ("slot", Origin::Rule, "2: Flint's place at the right, where #365 put it"),
        ("face", Origin::Guess, "face_mutant.tga h_fc5f519f, the data's one mutant face, shared; the product owner's choice"),
        ("skill", Origin::Real, "CSKILL_MOLOTOV_MUTANT 20, cost 3, recovery 1.5: the campaign type's (the self-destruct, #211, isn't built)"),
        ("speeds", Origin::Guess, "the campaign type's 0 / 2.3 / 5 / 7 kept, as the Supermutant's"),
    ],
};

/// The Ugly Mutant, the Mutant Horde's big one: e23's campaign type (55 hp, on the bigger
/// ms_mutant model), with hit points 75 (rule: Brutus's tier, the Supermutant's model), HUD slot
/// 3, Brutus's place at the bottom (rule), the shared mutant face (guess), the Confed LZR-10 and
/// 2 of the grenade item h_fd437c4d (real: an item of its own, stack-limit 999, whose pickup is the
/// Frag's h_060f95f7; the analyst's draft calls them Frag), speeds kept (guess). Skill-set 0, cost 3, recovery 1.5 (real). The key
/// is the demo's own (guess).
pub const UGLY: Class = Class {
    key: "mp_mutant_ugly",
    base: "ugly mutant@e23",
    library: "mutant_horde",
    hitpoints: 75.0,
    slot: 3,
    face: Some("face_mutant.tga"),
    speeds: None,
    ammo: None,
    why: &[
        ("name", Origin::Guess, "mp_mutant_ugly h_0ca8ee04: the demo's"),
        ("hitpoints", Origin::Rule, "75: Brutus's tier; the bigger ms_mutant model, the Supermutant's"),
        ("slot", Origin::Rule, "3: Brutus's place at the bottom"),
        ("face", Origin::Guess, "face_mutant.tga h_fc5f519f, the data's one mutant face, shared; the product owner's choice"),
        ("skill", Origin::Real, "none: skill-set 0, cost 3, recovery 1.5, the campaign type's"),
        ("speeds", Origin::Guess, "the campaign type's 0 / 2.3 / 5 / 7 kept, as the Supermutant's"),
    ],
};

/// The Filthy Mutant, the Mutant Horde's common one: e23's campaign type (55 hp; m03's copies
/// carry the corpse-eater skill, #210), with hit points 70 (rule: the standard tier, the ferals'
/// 70 - 75), HUD slot 4, Hawk's place at the left (rule: the lightest model, ms_mutant2), the
/// shared mutant face (guess), the Confed LZR-10 and 2 Gas grenades (real), speeds kept (guess).
/// Skill-set 0, cost 3, recovery 1.5 (real: e23's copy). The key is the demo's own (guess).
pub const FILTHY: Class = Class {
    key: "mp_mutant_filthy",
    base: "filthy mutant@e23",
    library: "mutant_horde",
    hitpoints: 70.0,
    slot: 4,
    face: Some("face_mutant.tga"),
    speeds: None,
    ammo: None,
    why: &[
        ("name", Origin::Guess, "mp_mutant_filthy h_fa73c507: the demo's"),
        ("hitpoints", Origin::Rule, "70: the standard tier, the ferals' 70 - 75"),
        ("slot", Origin::Rule, "4: Hawk's place at the left, the lightest model's (ms_mutant2)"),
        ("face", Origin::Guess, "face_mutant.tga h_fc5f519f, the data's one mutant face, shared; the product owner's choice"),
        ("skill", Origin::Real, "none: skill-set 0, cost 3, recovery 1.5, e23's copy (m03's carry the corpse eater, #210)"),
        ("speeds", Origin::Guess, "the campaign type's 0 / 2.3 / 5 / 7 kept, as the Supermutant's"),
    ],
};

/// Every class #394 makes (the Kingman Posse's four, then the Mutant Horde's), in the order the
/// NPC menu's data lists them. `Game::add_classes` adds those whose base type is loaded.
pub const CLASSES: [&Class; 8] = [&KINGMAN, &SUBVERSIVE, &RED_HAND, &TERRORIST, &SUPERMUTANT, &MOLOTOV, &UGLY, &FILTHY];

/// The class whose type has key `type_key` (the hash of `Class::key`), if there is one.
pub fn class_of(type_key: u32) -> Option<&'static Class> {
    CLASSES.iter().copied().find(|c| c.type_key() == type_key)
}

impl Class {
    /// Its type's key: the hash of its name, what `Game::character_types` and the hooks know it
    /// by.
    pub fn type_key(&self) -> u32 {
        h(self.key)
    }

    /// The class as a character type: `base` with the rule's columns applied (damage-multiplier
    /// 1, the player flag, no vulnerabilities or shield, kill reward 0, radar-range
    /// `RADAR_RANGE`, the stamina update on) and the class's own values (hit points, HUD slot as
    /// `footstep_type` with the slot's skull, the face, speeds, the ammo item), under its key,
    /// in the archive `CLASS_ARCHIVE` and its library. Everything else is the base's, the
    /// skill block's numbers included.
    pub fn apply(&self, base: &CharacterType) -> CharacterType {
        let mut t = base.clone();
        t.key = self.type_key();
        t.archive = CLASS_ARCHIVE.to_string();
        t.library = self.library.to_string();
        t.hitpoints = self.hitpoints;
        t.damage_multiplier = 1.0;
        t.player_type = true;
        t.vulnerabilities.clear();
        t.shield = None;
        t.award.amount = 0.0;
        t.footstep_type = self.slot;
        t.skull = usize::try_from(self.slot - 1).ok().and_then(|i| SKULLS.get(i)).copied().unwrap_or(0);
        t.portrait = self.face.map_or(0, h);
        if let Some(speeds) = self.speeds {
            t.speeds = speeds;
        }
        if let Some((item, count)) = self.ammo {
            if !t.items.iter().any(|&(i, _)| i == item) {
                t.items.push((item, count));
            }
        }
        let mut skill = t.skill.take().unwrap_or_default();
        skill.radar_range = RADAR_RANGE;
        skill.stamina_update = true;
        t.skill = Some(skill);
        t
    }

    /// The log's lines for this class as made (`t` is `apply`'s result): its values on the first,
    /// then one line for each value of its own and each column of the rule with where it came from.
    pub fn report(&self, t: &CharacterType) -> Vec<String> {
        let skill = t.skill.clone().unwrap_or_default();
        let items: Vec<String> = t.items.iter().map(|(i, n)| format!("{n} x h_{i:08x}")).collect();
        let mut out = vec![format!(
            "hp {}, slot {}, face {}, skull h_{:08x}, skill-set {} cost {} recovery {}, speeds {:?}, items [{}], damage-multiplier {}, player flag {}, vulnerabilities {}, shield {}, radar-range {}, kill reward {}",
            t.hitpoints, t.footstep_type, if t.portrait == 0 { "none".to_string() } else { format!("h_{:08x}", t.portrait) }, t.skull,
            skill.skill_set, skill.cost, skill.recovery, t.speeds, items.join(", "), t.damage_multiplier, t.player_type, t.vulnerabilities.len(),
            t.shield.as_ref().map_or("none".to_string(), |s| format!("type {}", s.kind)), skill.radar_range, t.award.amount)];
        for (name, origin, why) in self.why.iter().chain(RULE.iter()) {
            out.push(format!("    {name}: {} ({why})", origin.word()));
        }
        out
    }
}

/// The columns of the rule a type does NOT satisfy (empty: it fits), for a class and for the 20
/// sdm types alike: the player flag, damage-multiplier 1, no vulnerabilities, no shield, kill reward
/// 0, radar-range of `RADAR_RANGE` or more, a HUD slot 1 - 4 with that slot's skull, hit points in
/// `HITPOINT_BAND`, max-stamina 100. The sdm types that deviate do so with a species trait (the
/// Shrike Hound's 1.8, the Shrikes' stamina shields and factors, the hounds' factors), which
/// `character_type_report` prints beside them.
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
    gaps
}

impl Game {
    /// Adds the deathmatch classes (`CLASSES`) whose base type is loaded with its model, each
    /// as a type of its own under its key after every type of the data (`Class::apply`; a name
    /// lookup without an archive still finds the data's first). A class already added is left as
    /// it is, so a second call adds nothing. For each class its name and the key of the type,
    /// or why there is none.
    pub fn add_classes(&mut self) -> Vec<(&'static str, Result<u32, String>)> {
        self.add_classes_with(Game::has_model)
    }

    /// `add_classes` with the test of a base type's model given (`Game::has_model`: its compound
    /// archetype is loaded), so that the rest can be tried without the data's objects.
    fn add_classes_with(&mut self, has_model: impl Fn(&Game, &CharacterType) -> bool) -> Vec<(&'static str, Result<u32, String>)> {
        let mut out = vec![];
        for class in CLASSES {
            let key = class.type_key();
            if let Some(t) = self.character_types.get(&key) {
                // (added already; a type of the data with the class's name would be in its way)
                out.push((class.key, if t.is_class() { Ok(key) } else { Err(format!("a type of the data is named {}", class.key)) }));
                continue;
            }
            let made = match self.find_character_type(class.base).and_then(|k| self.character_types.get(&k)) {
                None => Err(format!("its base type {} isn't loaded", class.base)),
                Some(base) if !has_model(self, base) => Err(format!("its base type {} has no model loaded", class.base)),
                Some(base) => Ok(class.apply(base)),
            };
            out.push((class.key, made.map(|t| {
                self.add_character_type(t);
                key
            })));
        }
        out
    }

    /// The classes and the squad deathmatch types of the loaded data with the columns of the rule
    /// each doesn't satisfy (`rule_gaps`), one line each, for `character_type_report`: how the
    /// 20 shipped types fit the rule the classes are made by, and that the classes do.
    pub fn class_report(&self) -> Vec<String> {
        let mut out = vec![];
        for &k in &self.character_type_order {
            let t = &self.character_types[&k];
            let class = class_of(k).filter(|_| t.is_class());
            if class.is_none() && !(t.archive == "mp_common" && t.library.starts_with("sdm_")) {
                continue;
            }
            let gaps = rule_gaps(t);
            out.push(format!("{} h_{k:08x} [{}] \"{}\"{}: hp {}, slot {}, damage-multiplier {}, vulnerabilities {}, shield {}, radar-range {}, kill reward {} | rule gaps: {}",
                if class.is_some() { "class" } else { "sdm type" }, t.place(), self.type_label(k),
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

    /// A campaign enemy of the Kingman kind: 311 hp, vulnerabilities, a reward, the radar's 30 m,
    /// no portrait, the campaign's speeds and a gun with 5 grenades.
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
            footstep_type: 0,
            award: Award { amount: 3500.0, to_squad: true, kind: 2, ..Default::default() },
            skill: Some(SkillData { skill_set: 5, max_stamina: 100.0, cost: 3.0, recovery: 1.5, radar_range: 30.0, ..Default::default() }),
            ..Default::default()
        }
    }

    #[test]
    fn the_rule_makes_a_campaign_type_a_class() {
        let base = campaign();
        assert_eq!(rule_gaps(&base), vec!["player flag", "damage-multiplier", "vulnerabilities", "shield", "kill reward", "radar-range", "HUD slot", "hit points"]);
        let t = KINGMAN.apply(&base);
        assert!(rule_gaps(&t).is_empty(), "{:?}", rule_gaps(&t));
        // the class's own values
        assert_eq!((t.key, t.archive.as_str(), t.library.as_str(), t.place().as_str()), (h("mp_edward_kingman"), "demo", "kingman_posse", "demo/kingman_posse"));
        assert_eq!((t.hitpoints, t.footstep_type, t.skull, t.portrait), (95.0, 1, h("skull_tex.tga"), h("face_kingman.tga")));
        assert!(t.is_class());
        // the rule's columns
        assert_eq!((t.damage_multiplier, t.player_type, t.award.amount), (1.0, true, 0.0));
        assert!(t.vulnerabilities.is_empty() && t.shield.is_none());
        let skill = t.skill.as_ref().expect("a skill block");
        assert_eq!((skill.radar_range, skill.stamina_update), (60.0, true));
        // the skill's numbers are the campaign type's, and so are the gun, the model and the rest
        assert_eq!((skill.skill_set, skill.cost, skill.recovery, skill.max_stamina), (5, 3.0, 1.5, 100.0));
        assert_eq!((t.label, t.mesh, t.motion_set, t.chatter.as_str(), &t.weapons, t.speeds), (7, 11, 12, "edward_kingman_chatter", &base.weapons, base.speeds));
        // the ammo item joins the grenades
        assert_eq!(t.items, vec![(0xE8C6_7904, 5), (HIGH_ROF_AMMO, 120)]);
        // the base is untouched
        assert_eq!((base.hitpoints, base.player_type, base.is_class()), (311.0, false, false));
    }

    #[test]
    fn a_class_without_a_face_or_with_speeds_of_its_own_says_so() {
        let t = RED_HAND.apply(&campaign());
        assert_eq!((t.portrait, t.speeds, t.footstep_type, t.skull), (0, MILITIA_SPEEDS, 3, h("skull_brutus.tga")));
        // (no ammo item for the LZR line)
        assert_eq!(t.items, vec![(0xE8C6_7904, 5)]);
        assert!(rule_gaps(&t).is_empty());
        // a class doesn't add its ammo item twice
        let twice = KINGMAN.apply(&KINGMAN.apply(&campaign()));
        assert_eq!(twice.items.iter().filter(|&&(i, _)| i == HIGH_ROF_AMMO).count(), 1);
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
        // (hit points by the owner's choice of 2026-11-07, the seats by the rule: the boss at slot 1)
        let table: Vec<(&str, f32, i64, Option<&str>)> = CLASSES.iter().map(|c| (c.key, c.hitpoints, c.slot, c.face)).collect();
        assert_eq!(table, vec![
            ("mp_edward_kingman", 95.0, 1, Some("face_kingman.tga")),
            ("mp_subversive_marine", 70.0, 2, Some("face_subversive.tga")),
            ("mp_red_hand", 55.0, 3, None),
            ("mp_terrorist", 50.0, 4, Some("face_terrorist.tga")),
            ("mp_supermutant", 95.0, 1, Some("face_supermutant.tga")),
            ("mp_mutant_molotov", 55.0, 2, Some("face_mutant.tga")),
            ("mp_mutant_ugly", 75.0, 3, Some("face_mutant.tga")),
            ("mp_mutant_filthy", 70.0, 4, Some("face_mutant.tga")),
        ]);
        // each squad has the four slots once, and every class tells where its values came from
        for library in ["kingman_posse", "mutant_horde"] {
            let mut slots: Vec<i64> = CLASSES.iter().filter(|c| c.library == library).map(|c| c.slot).collect();
            slots.sort();
            assert_eq!(slots, vec![1, 2, 3, 4], "{library}");
        }
        let mut keys: Vec<u32> = CLASSES.iter().map(|c| c.type_key()).collect();
        keys.sort();
        keys.dedup();
        assert_eq!(keys.len(), CLASSES.len());
        for c in CLASSES {
            assert!(c.why.iter().any(|w| w.0 == "hitpoints") && c.why.iter().any(|w| w.0 == "slot") && c.why.iter().any(|w| w.0 == "face"), "{}", c.key);
            assert!(c.base.contains('@'), "{}", c.key);
            assert!(class_of(c.type_key()).is_some_and(|k| k.key == c.key));
        }
        assert!(class_of(h("mp_tex")).is_none());
        // the hit points are tiers of the sdm types' band
        assert!(CLASSES.iter().all(|c| (HITPOINT_BAND.0..=HITPOINT_BAND.1).contains(&c.hitpoints)));
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
            skill: Some(SkillData { skill_set: 6, max_stamina: 100.0, cost: 20.0, recovery: 1.0, radar_range: 60.0, stamina_update: true, ..Default::default() }),
            ..Default::default()
        };
        assert!(rule_gaps(&militia).is_empty());
        // a Shrike Hound's species: 1.8, factors and 180 m radar (the radar is above the rule's 60: no gap)
        let hound = CharacterType { damage_multiplier: 1.8, vulnerabilities: vec![(4, 0.1)], skill: Some(SkillData { radar_range: 180.0, ..Default::default() }), ..militia.clone() };
        assert_eq!(rule_gaps(&hound), vec!["damage-multiplier", "vulnerabilities"]);
        // a slot out of 1 - 4 and a skull that isn't the slot's
        assert!(rule_gaps(&CharacterType { footstep_type: 0, ..militia.clone() }).contains(&"HUD slot"));
        assert!(rule_gaps(&CharacterType { skull: h("skull_hawk.tga"), ..militia }).contains(&"skull"));
    }

    #[test]
    fn the_report_says_where_each_value_came_from() {
        let t = KINGMAN.apply(&campaign());
        let lines = KINGMAN.report(&t);
        assert!(lines[0].starts_with("hp 95, slot 1, face h_1cc0787c, skull h_fe6fa2af, skill-set 5 cost 3 recovery 1.5"), "{}", lines[0]);
        assert!(lines[0].contains("radar-range 60") && lines[0].contains("damage-multiplier 1") && lines[0].contains("kill reward 0"), "{}", lines[0]);
        assert!(lines.iter().any(|l| l.starts_with("    hitpoints: rule (")));
        assert!(lines.iter().any(|l| l.starts_with("    face: real (")));
        assert!(lines.iter().any(|l| l.starts_with("    speeds: guess (")));
        assert!(lines.iter().any(|l| l.starts_with("    radar-range: rule (")));
        assert_eq!(lines.len(), 1 + KINGMAN.why.len() + RULE.len());
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
        assert_eq!(made.len(), CLASSES.len());
        // only Kingman's base is loaded: the other seven say so
        assert_eq!(made.iter().filter(|r| r.1.is_ok()).map(|r| r.0).collect::<Vec<_>>(), vec!["mp_edward_kingman"]);
        assert!(made.iter().filter(|r| r.0 != "mp_edward_kingman").all(|r| r.1.as_ref().is_err_and(|e| e.contains("isn't loaded"))));
        let t = game.character_type(h("mp_edward_kingman")).expect("added");
        assert!(t.is_class() && t.player_type && t.hitpoints == 95.0 && rule_gaps(t).is_empty());
        // it comes after the data's types: a name without an archive finds the data's first, `@demo` the class
        assert_eq!(game.character_type_order, vec![0xFFDC_629E, h("mp_edward_kingman")]);
        assert_eq!(game.find_character_type("edward kingman"), Some(0xFFDC_629E));
        assert_eq!(game.find_character_type("edward kingman@m03_c"), Some(0xFFDC_629E));
        assert_eq!(game.find_character_type("edward kingman@demo"), Some(h("mp_edward_kingman")));
        assert_eq!(game.find_character_type("mp_edward_kingman"), Some(h("mp_edward_kingman")));
        assert_eq!(game.type_label(h("mp_edward_kingman")), "Edward Kingman");
        // a second call adds nothing and still reports it
        let again = game.add_classes_with(|_, _| true);
        assert_eq!(game.character_type_order.len(), 2);
        assert!(again.iter().any(|r| r.0 == "mp_edward_kingman" && r.1 == Ok(h("mp_edward_kingman"))));
        // the dump's report lists the class (and no sdm type: none is loaded)
        let lines = game.class_report();
        assert_eq!(lines.len(), 1);
        assert!(lines[0].starts_with("class h_1b39dea1 [demo/kingman_posse] \"Edward Kingman\" (from edward kingman@m03_c): hp 95, slot 1"), "{}", lines[0]);
        assert!(lines[0].ends_with("rule gaps: none"), "{}", lines[0]);
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
