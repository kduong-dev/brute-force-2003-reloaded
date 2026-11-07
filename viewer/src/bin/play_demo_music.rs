//! Deathmatch's music silence, and the one place the demo lets two maps' own music through it
//! (#403, corrected by #398). Deathmatch plays without music: every music-bank track the
//! sound-triggers ask for is silenced there (play.rs `setup`, play_sound_triggers.rs `voices`),
//! the ambience beds among them, a demo choice from the first version. The product owner asked
//! for Estuary Showdown (sdm_e40) and Cavern of Fire (sdm_m07) to keep their music, and this table
//! (`CHOICES`) is what is let through, by sound id.
//!
//! **These tracks are the game's own.** #403 first took `creepy_estuary_mutant_01` and
//! `ambiencehellish-2ch` for tracks the shipped game never plays, because the demo found no sample
//! data for the maps' music sound-triggers (sdm_e40's and sdm_m07's h_1844a2b7, Type 0,
//! FIRST_UPDATE, looping, a 1 s fade-in): the sounds they ask for, h_10fb2011 and h_e70fb918, are
//! the hash of the sound entries' names, not of the waves' (h_e70fb918 is `ambiencehellish_2`;
//! h_10fb2011's name isn't recovered), and the demo looked a wave up by the hash of its name. The
//! game finds a wave by its sound's file record, the offset and size in the level's bank file
//! (FUN_0017bf40, FUN_00183710; `SoundBank::music_file`, play_sound_triggers.rs `MusicBank`), and
//! by that the maps' own triggers play the two tracks: sdm_e40's music is
//! `creepy_estuary_mutant_01` (Type music, 102.6 s, under the bird bed) and sdm_m07's is
//! `ambiencehellish-2ch` (Type music, 155.0 s, not a bed; `amb_shad_int2` is its bed). e05, the
//! campaign level, opens on the same creepy track. So the table adds nothing to what plays: it
//! only exempts the four sounds from deathmatch's silence, and what remains the product owner's
//! choice is that exemption, not the tracks.
//!
//! **One music voice** (#254, the game's 0x17eff0, play_sound_triggers.rs `MusicSlot`): each of the
//! two maps has one music sound-trigger, so nothing is replaced; no deathmatch map has a match-end
//! music (the demo has none, and the triggers' cues, like the tutorial's, are campaign levels').
//!
//! Test hooks: `BF_NO_MUSIC=1` turns all music off, the table's included, `BF_MUTE=1` runs and logs
//! without playing, `BF_MUSIC_VOLUME` sets the level, `BF_DEATHMATCH=1` plays a map as a
//! deathmatch session. The log (a deathmatch session on these maps, always) says which rows the
//! map's sound-triggers play: "deathmatch music (#403, the product owner's choice) on sdm_e40: ...".

use bf_viewer::bf::triggers::Class;

use super::*;

/// What a row is: the map's music, or an ambience bed under it. The sound's Type in the map's
/// sound bank says which (`SoundBank::sound_type`): `MUSIC_TYPE` the music, "ambient" a bed.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Role {
    Music,
    Bed,
}

impl Role {
    /// The word the log uses ("music" or "ambience", play.rs `music_mix`).
    fn label(self) -> &'static str {
        match self {
            Role::Music => "music",
            Role::Bed => "ambience",
        }
    }
}

/// One sound deathmatch keeps on a map: the product owner's choice.
#[derive(Clone, Copy, Debug)]
pub struct Choice {
    /// the map (its archive name, `CurrentMap`)
    pub map: &'static str,
    /// the sound id a sound-trigger of the map plays (`SoundTrigger::sound`): the hash of the sound
    /// entry's name in the map's sound bank, which is not always the wave's (see the module notes)
    pub sound: u32,
    /// the wave it plays: its name in the map's bank (data/sounds/<map>.xwb), for the log
    pub track: &'static str,
    /// music or ambience bed
    pub role: Role,
}

/// The table: the four sounds deathmatch keeps, two on each of two maps. Each row is the product
/// owner's choice (asked for 2026-11-07, #403) that the map keeps its music in deathmatch; the
/// tracks and their Types are the game's (sound ids, entry Types and waves by the sweep of the 38
/// level banks in todo/398-level-music/answers.md).
pub const CHOICES: [Choice; 4] = [
    // Estuary Showdown's music, the bank's first wave (102.6 s; also e05's): the music trigger
    // h_1844a2b7's sound, an entry whose name isn't recovered.
    Choice { map: "sdm_e40", sound: 0x10FB_2011, track: "creepy_estuary_mutant_01", role: Role::Music },
    // Estuary Showdown's outdoor bed (Type 2 trigger h_fa8c83e7: full outdoors, 15-35% indoors),
    // under the music; the entry is named as its wave (`h("light_bird_bed_1")`).
    Choice { map: "sdm_e40", sound: 0xE59D_5087, track: "light_bird_bed_1", role: Role::Bed },
    // Cavern of Fire's music, the bank's second wave (155.0 s): Type music, not a bed; the entry is
    // named `ambiencehellish_2` (`h("ambiencehellish_2")`), the wave `ambiencehellish-2ch`.
    Choice { map: "sdm_m07", sound: 0xE70F_B918, track: "ambiencehellish-2ch", role: Role::Music },
    // Cavern of Fire's bed (Type 0 trigger h_fa8c83e7 at vol 0.25), beside the music; the entry is
    // named as its wave (`h("amb_shad_int2")`).
    Choice { map: "sdm_m07", sound: 0x0544_355E, track: "amb_shad_int2", role: Role::Bed },
];

/// The choices for `map`.
pub fn for_map(map: &str) -> impl Iterator<Item = &'static Choice> + '_ {
    CHOICES.iter().filter(move |c| c.map == map)
}

/// Whether the table lets sound `sound` (a sound-trigger's, a sound id) play on `map` in
/// deathmatch, where the sound-triggers' music-bank tracks are otherwise silenced
/// (play_sound_triggers.rs `voices`).
pub fn trigger_plays(map: &str, sound: u32) -> bool {
    for_map(map).any(|c| c.sound == sound)
}

/// The module's system: the log, once the level has started, after `setup`.
pub fn plugin(app: &mut App) {
    app.add_systems(OnEnter(AppState::Playing), start.after(super::setup));
}

/// In a deathmatch session on one of the table's maps, one log line a row: whether the map has a
/// sound-trigger that plays it. BF_NO_MUSIC says it plays nothing.
fn start(current: Res<CurrentMap>, map: Res<MapLevel>, mode: Res<Mode>) {
    let rows: Vec<&Choice> = for_map(&current.0).collect();
    if rows.is_empty() || !mode.deathmatch {
        return;
    }
    let tag = format!("deathmatch music (#403, the product owner's choice) on {}", current.0);
    if std::env::var("BF_NO_MUSIC").is_ok() {
        println!("{tag}: BF_NO_MUSIC, none");
        return;
    }
    for c in rows {
        let by = map.0.as_ref().and_then(|l| l.triggers.iter().find(|t| matches!(&t.class, Class::Sound(s) if s.sound == c.sound)));
        println!("{tag}: {} {} (h_{:08x}) {}", c.role.label(), c.track, c.sound, match by {
            Some(_) => "is played by the map's sound-trigger (kept in deathmatch)",
            None => "has no sound-trigger on this map: nothing plays it",
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bf_viewer::bf::hash::h;

    /// The table is the two maps' four rows, each map with one music row (#254's one music voice)
    /// and one bed, no sound twice; the sound ids of the beds and of Cavern of Fire's music are the
    /// hashes of their sound entries' names (the music's is `ambiencehellish_2`, the wave being
    /// `ambiencehellish-2ch`), Estuary Showdown's music id is one whose name isn't recovered.
    #[test]
    fn the_table_is_the_two_maps_kept_sounds() {
        assert_eq!(for_map("sdm_e40").count(), 2);
        assert_eq!(for_map("sdm_m07").count(), 2);
        assert_eq!(for_map("sdm_e34").count(), 0);
        for map in ["sdm_e40", "sdm_m07"] {
            assert_eq!(for_map(map).filter(|c| c.role == Role::Music).count(), 1);
            assert_eq!(for_map(map).filter(|c| c.role == Role::Bed).count(), 1);
        }
        let sound = |map: &str, track: &str| for_map(map).find(|c| c.track == track).map(|c| c.sound);
        assert_eq!(sound("sdm_e40", "light_bird_bed_1"), Some(h("light_bird_bed_1")));
        assert_eq!(sound("sdm_m07", "amb_shad_int2"), Some(h("amb_shad_int2")));
        assert_eq!(sound("sdm_m07", "ambiencehellish-2ch"), Some(h("ambiencehellish_2")));
        assert_ne!(sound("sdm_m07", "ambiencehellish-2ch"), Some(h("ambiencehellish-2ch")));
        assert_eq!(sound("sdm_e40", "creepy_estuary_mutant_01"), Some(0x10FB_2011));
        assert_ne!(sound("sdm_e40", "creepy_estuary_mutant_01"), Some(h("creepy_estuary_mutant_01")));
        for (i, a) in CHOICES.iter().enumerate() {
            assert!(CHOICES[i + 1..].iter().all(|b| (a.map, a.sound) != (b.map, b.sound)));
        }
    }

    /// The table lets through each map's four sounds, by sound id, on its own map only; the other
    /// deathmatch maps' tracks (and a wave's name that isn't a sound id) stay silent.
    #[test]
    fn only_the_two_maps_sounds_pass_the_deathmatch_silence() {
        assert!(trigger_plays("sdm_e40", h("light_bird_bed_1")));
        assert!(trigger_plays("sdm_e40", 0x10FB_2011));
        assert!(trigger_plays("sdm_m07", h("amb_shad_int2")));
        assert!(trigger_plays("sdm_m07", 0xE70F_B918));
        assert!(!trigger_plays("sdm_e40", h("creepy_estuary_mutant_01")));
        assert!(!trigger_plays("sdm_m07", h("ambiencehellish-2ch")));
        assert!(!trigger_plays("sdm_e40", h("amb_shad_int2")));
        assert!(!trigger_plays("sdm_e40", 0xE70F_B918));
        assert!(!trigger_plays("sdm_e34", h("caspian_action20")));
    }
}
