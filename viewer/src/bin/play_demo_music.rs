//! The demo's own music choices for two deathmatch maps (#403): `creepy_estuary_mutant_01` on
//! Estuary Showdown (sdm_e40) and `ambiencehellish-2ch` on Cavern of Fire (sdm_m07). **These are the
//! product owner's choices, not the game's**: the shipped game plays neither track on any level (the
//! cut-content catalogue, section 3.3; #396; #398). Both sit in their map's streaming wave bank
//! (data/sounds/<map>.xwb) with no sound entry whose id is the hash of their name: the maps' music
//! sound-triggers (sdm_e40's and sdm_m07's h_1844a2b7, Type 0, FIRST_UPDATE, looping, a 1 s
//! fade-in, vol 0.47) ask for sounds (h_10fb2011, h_e70fb918) whose music-bank files are as long as
//! these two waves (102.62 s and 155.01 s by the byte rate, `SoundBank::length`; creepy is 2541744
//! bytes, hellish 3839328) but whose ids match no wave of the bank, so the demo finds no sample
//! data for them and they stay silent (BF_SOUND_LOG prints them). The music menu of the test map
//! plays the five tracks alone (play_testworld.rs `UNUSED_TRACKS`); this is the only place a
//! level plays one.
//!
//! **The table** (`CHOICES`) is the one named place the choices live: map, track, whether it is
//! music or an ambience bed, and how it plays. Each row is commented as the product owner's
//! choice. It is the explicit exception to any rule that drops the waves of a level's bank that no
//! sound entry names (#398): these tracks have no sound entry and play anyway, by name.
//!  * A row played by the demo (`Play::Demo`) is a plain loop started as the level starts, at the
//!    music volume (`music_volume`) times its share, the shares `music_mix` gives a level's bank
//!    (play.rs: music 1.0, an ambience bed `AMBIENCE_VOLUME`). It is marked `LevelMusic`, so the test
//!    tools' music picker and level switch stop it as they stop a level's music.
//!  * A row played by the map's own sound-trigger (`Play::Trigger`) is a track the game does play
//!    (sdm_e40's light_bird_bed_1, h_fa8c83e7, a Type 2 outdoor bed that follows whether the
//!    player is outdoors or in; sdm_m07's amb_shad_int2, h_fa8c83e7 too, Type 0 at vol 0.25:
//!    play_sound_triggers.rs). Deathmatch silences every music-bank
//!    track of the sound-triggers (play.rs `setup`: "deathmatch plays without music"; the demo's own
//!    choice, from the first version), so on these two maps the table lets these rows through
//!    (`trigger_plays`) and the trigger plays them as it always does, at its own volume and level.
//!
//! **One music voice** (#254, the game's 0x17eff0, play_sound_triggers.rs `MusicSlot`): the table
//! has at most one music row per map, and the demo's loop is not a sound-trigger's voice, so the
//! slot is not involved. The two maps' own music triggers have no data to play, so nothing is
//! replaced; no deathmatch map has a match-end music (the demo has none, and the triggers' cues,
//! like the tutorial's, are campaign levels').
//!
//! Test hooks: `BF_NO_MUSIC=1` turns all of it off, `BF_MUTE=1` runs and logs without playing,
//! `BF_MUSIC_VOLUME` sets the level. The log (always, on these maps) says which row plays how: "demo
//! music (#403, the product owner's choice): ...".

use bf_viewer::bf::character::Game;
use bf_viewer::bf::hash::h;
use bf_viewer::bf::triggers::Class;

use super::*;

/// What a row is: the music, or an ambience bed under it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Role {
    Music,
    Bed,
}

impl Role {
    /// The word the logs and `spawn_music`'s label use ("music" or "ambience", play.rs `music_mix`).
    fn label(self) -> &'static str {
        match self {
            Role::Music => "music",
            Role::Bed => "ambience",
        }
    }
}

/// How a row plays (see the module notes).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Play {
    /// the demo starts it: a loop at this share of the music volume
    Demo { share: f32 },
    /// the map's own sound-trigger plays it (it asks for the track by name), at its own volume
    Trigger,
}

/// One of the product owner's choices.
#[derive(Clone, Copy, Debug)]
pub struct Choice {
    /// the map (its archive name, `CurrentMap`)
    pub map: &'static str,
    /// the track: its name in the map's wave bank (data/sounds/<map>.xwb); a sound-trigger's
    /// sound id is its hash (`h`)
    pub track: &'static str,
    /// music or ambience bed
    pub role: Role,
    /// how it plays
    pub play: Play,
}

/// The table: the demo's music choices for two deathmatch maps. Each row is the product owner's
/// choice (asked for 2026-11-07, #403), not the game's.
pub const CHOICES: [Choice; 4] = [
    // Estuary Showdown: the product owner's choice of music, looping. The track is the bank's first
    // entry (104.1 s), the one the game never plays: the map's music trigger asks for h_10fb2011.
    // Share 1.0, the music's share in play.rs `music_mix`.
    Choice { map: "sdm_e40", track: "creepy_estuary_mutant_01", role: Role::Music, play: Play::Demo { share: 1.0 } },
    // Estuary Showdown's outdoor bed, kept under the music at the trigger's own level (Type 2,
    // h_fa8c83e7: full outdoors, down indoors). The product owner asked to keep it; it is the game's
    // own bed, which deathmatch's silence on music-bank tracks would otherwise hide.
    Choice { map: "sdm_e40", track: "light_bird_bed_1", role: Role::Bed, play: Play::Trigger },
    // Cavern of Fire: the product owner's choice of an ambience bed, alongside amb_shad_int2.
    // The bank's second entry (157.2 s), also never played by the game. Share 0.8, the bed share of
    // play.rs `music_mix` (`AMBIENCE_VOLUME`), which is also what the music menu gives it alone.
    Choice { map: "sdm_m07", track: "ambiencehellish-2ch", role: Role::Bed, play: Play::Demo { share: super::AMBIENCE_VOLUME } },
    // Cavern of Fire's own bed (Type 0 at the trigger's vol 0.25, h_fa8c83e7), kept playing for the
    // same reason as the bird bed: the product owner's choice is to hear the new bed beside it.
    Choice { map: "sdm_m07", track: "amb_shad_int2", role: Role::Bed, play: Play::Trigger },
];

/// The choices for `map`.
pub fn for_map(map: &str) -> impl Iterator<Item = &'static Choice> + '_ {
    CHOICES.iter().filter(move |c| c.map == map)
}

/// Whether the table lets sound `sound` (a sound-trigger's, a hash of a track's name) play on
/// `map` in deathmatch, where the sound-triggers' music-bank tracks are otherwise silenced
/// (play_sound_triggers.rs `voices`).
pub fn trigger_plays(map: &str, sound: u32) -> bool {
    for_map(map).any(|c| c.play == Play::Trigger && h(c.track) == sound)
}

/// The module's system: the demo's rows start as the level does, after `setup`.
pub fn plugin(app: &mut App) {
    app.add_systems(OnEnter(AppState::Playing), start.after(super::setup));
}

/// Starts the demo's rows for the map (`Play::Demo`) and logs every row of the map, one line each.
/// BF_NO_MUSIC turns it all off; BF_MUTE logs without playing.
fn start(mut commands: Commands, mut sources: ResMut<Assets<AudioSource>>, current: Res<CurrentMap>, map: Res<MapLevel>) {
    let rows: Vec<&Choice> = for_map(&current.0).collect();
    if rows.is_empty() {
        return;
    }
    let (mute, off) = (std::env::var("BF_MUTE").is_ok(), std::env::var("BF_NO_MUSIC").is_ok());
    let tag = format!("demo music (#403, the product owner's choice) on {}", current.0);
    if off {
        println!("{tag}: BF_NO_MUSIC, none");
        return;
    }
    let tracks = match Game::load_music(&data_dir(), &current.0) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("{tag}: no music bank: {e}");
            return;
        }
    };
    for c in rows {
        match c.play {
            Play::Trigger => {
                let by = map.0.as_ref().and_then(|l| l.triggers.iter().find(|t| matches!(&t.class, Class::Sound(s) if s.sound == h(c.track))));
                println!("{tag}: {} {} {}", c.role.label(), c.track, match by {
                    Some(_) => "is played by the map's sound-trigger (kept in deathmatch)",
                    None => "has no sound-trigger on this map: nothing plays it",
                });
            }
            Play::Demo { share } => {
                let Some((_, wav)) = tracks.iter().find(|(n, _)| n == c.track) else {
                    eprintln!("{tag}: {} isn't in the map's wave bank", c.track);
                    continue;
                };
                println!("{tag}: {} {} x{share:.2}, looping{}", c.role.label(), c.track, if mute { " (BF_MUTE: not played)" } else { "" });
                if !mute {
                    spawn_music(&mut commands, &mut sources, wav.clone(), c.role.label(), share);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The table is the two maps' four rows, each map with at most one music row (#254's one music
    /// voice), no row twice, and the shares of the demo's loops are the ones `music_mix` gives.
    #[test]
    fn the_table_is_the_two_maps_choices() {
        assert_eq!(for_map("sdm_e40").count(), 2);
        assert_eq!(for_map("sdm_m07").count(), 2);
        assert_eq!(for_map("sdm_e34").count(), 0);
        for map in ["sdm_e40", "sdm_m07"] {
            assert!(for_map(map).filter(|c| c.role == Role::Music).count() <= 1);
        }
        let e40 = for_map("sdm_e40").find(|c| c.track == "creepy_estuary_mutant_01").unwrap();
        assert_eq!((e40.role, e40.play), (Role::Music, Play::Demo { share: 1.0 }));
        let m07 = for_map("sdm_m07").find(|c| c.track == "ambiencehellish-2ch").unwrap();
        assert_eq!((m07.role, m07.play), (Role::Bed, Play::Demo { share: super::super::AMBIENCE_VOLUME }));
        for (i, a) in CHOICES.iter().enumerate() {
            assert!(CHOICES[i + 1..].iter().all(|b| (a.map, a.track) != (b.map, b.track)));
        }
    }

    /// Only the two maps' kept beds are let through in deathmatch, by the hash of their names; the
    /// demo's own rows and any other map's tracks are not (their triggers stay silent).
    #[test]
    fn only_the_kept_beds_pass_the_deathmatch_silence() {
        assert!(trigger_plays("sdm_e40", h("light_bird_bed_1")));
        assert!(trigger_plays("sdm_m07", h("amb_shad_int2")));
        assert!(!trigger_plays("sdm_e40", h("creepy_estuary_mutant_01")));
        assert!(!trigger_plays("sdm_m07", h("ambiencehellish-2ch")));
        assert!(!trigger_plays("sdm_e40", h("amb_shad_int2")));
        assert!(!trigger_plays("sdm_e34", h("caspian_action20")));
    }
}
