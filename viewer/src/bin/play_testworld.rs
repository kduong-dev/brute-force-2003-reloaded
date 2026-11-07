//! The test tools' world menus (#116), beside play_testtools.rs's (#111): a sky picker, a music
//! picker and a level switch. Demo tooling, not the game: the keys and the layout are the demo's
//! choices, none of them used by the game's controls. In a test session (`--test`) on any map.
//!
//!  - Y: the sky menu. "none", the map's own, or any level's sky: every level whose level file
//!    names a sky mesh (`Level::sky_mesh`, read from the archives when the session starts). The
//!    chosen level's sky (its layers, as `level_scene::spawn_level` draws them) replaces the
//!    map's, and its background colour the clear colour; on the flat test floor the camera then
//!    sees as far as a level's (SKY_FAR) to reach it.
//!  - U: the music menu. "off", the map's own, or any level's music bank (data/sounds/<level>.xwb
//!    with tracks), mixed as the game mixes a level's (play.rs `music_mix`: the music, its
//!    ambience bed under it, by the Types of the sound entries that play them). Left / Right: the
//!    previous / next bank at once. The test map's own is m04_a's (TEST_MUSIC, #302), playing from
//!    its start. **Shift+Enter (or Shift+click) opens a bank's tracks** (#398): every wave of the
//!    bank with the Type of the sound entry that plays it (found by the wave's place in the bank
//!    file, through the sound library's file records, as the game finds a sound's wave) and its
//!    length; Up / Down pick, Enter plays one alone, looping, at its share of the music volume;
//!    Esc or Backspace goes back to the banks. After the banks comes a group of five "bank tracks",
//!    waves whose sound entries are named otherwise than the wave (`BANK_TRACKS`, #396): each
//!    plays alone from its first bank, looping, the map's own music stopped first.
//!  - L: the level menu: the test map, then the game's level list (common campaign-bf.xmb: the
//!    squad deathmatch maps, the deathmatch arenas, the missions' zones). Enter loads the chosen
//!    one without restarting: AppState::Switching tears the map down (`end_play`, as Backspace
//!    does), loads the next on a thread as the front end does (with the test map's weapon data,
//!    so the object tool has its whole list) and plays it. The tools, instant kill and the help
//!    panel stay. The deathmatch arenas are played alone, as the front end plays them; the rest
//!    with the squad. A map that won't load (no level file) puts the test map up instead, saying
//!    so.
//!
//! Up / Down or the wheel picks in each menu, a click or Enter takes the pick; the key again or
//! Esc closes it.

use std::sync::{mpsc::Receiver, Mutex};

use super::*;
use super::testtools::{Hook, Panel, Script, Session, Tools};
use bf_viewer::bf::audio::{BankTrack, MixTrack, XWB_HEAD};
use bf_viewer::bf::bxml::SchemaSet;
use bf_viewer::bf::character::{Geoset, LEVEL_DEATHMATCH, LEVEL_SQUAD_DEATHMATCH};
use bf_viewer::bf::level::SkySpin;
use bf_viewer::level_scene::{SkyLayer, SkyLayers};
use super::testmap::TestMap;

/// How many lines of a list show at once.
const LIST_LINES: usize = 10;
/// How long a message shows (s): the test map's.
const MESSAGE_TIME: f32 = 2.0;
/// When a map's start is logged (s of the controlled character's clock, BF_TOOLS_LOG; the
/// demo's): once its scenery and pickups are listed.
const AUDIT_AT: f32 = 0.5;
/// How far the camera sees (m) under another level's sky: as far as on a level (setup's
/// 5000). The flat floor's camera has Bevy's 1000, short of sdm_e34's sky (3.2 km across).
const SKY_FAR: f32 = 5000.0;
/// The test map's own music (#302): m04_a's bank, mixed as the music menu mixes any level's
/// (play.rs `music_mix`: its music with its ambience bed under it), playing from the flat floor's
/// start, and the music menu's "this map's own" there (the product owner's choice: the flat
/// floor has none of its own).
pub(super) const TEST_MUSIC: &str = "m04_a";

/// A music or ambience track of the game's streaming wave banks (data/sounds/<level>.xwb) whose
/// sound entry is named otherwise than its wave: the entry's id is the hash of another name, so a
/// lookup by the wave's name finds no sound (todo/cut-content/catalogue.md, section 3.3, took
/// that for "no level plays it", and #398 found the levels play all five, by the sound's file
/// record). The music menu offers them in a group of their own, the "bank tracks" (#396; the
/// demo's tool, not the game's behaviour).
pub(super) struct GroupTrack {
    /// the wave's name in its banks
    name: &'static str,
    /// the banks that hold it (data/sounds/<bank>.xwb), the first read; the same bytes in each
    banks: &'static [&'static str],
    /// the sound id the banks' sound entries give it (the levels' sound-triggers play that)
    sound: u32,
    /// how long it plays once decoded (s), as the demo's decoder gives it (`load_group_track`
    /// logs the same figure): the catalogue's 104, 45, 22, 165 and 157 rounded
    secs: f32,
}

/// The five bank tracks, in the catalogue's order (catalogue.md 3.3). Each is 22050 Hz Xbox ADPCM
/// stereo like every track of the music banks; the lengths are the decoder's (`audio::adpcm_decode`
/// makes 65 samples of each channel from a 36-byte block, 1.6% more than the 64 the game's
/// bytes-a-second rate, `audio::bytes_per_second`, counts: the wave's bytes 2541744, 1093392,
/// 530496, 4018608 and 3839328 give 104.1, 44.8, 21.7, 164.5 and 157.2 s here, 102.6, 44.1, 21.4,
/// 162.2 and 155.0 by that rate). The sound ids are the banks' (todo/398-level-music/answers.md
/// section 1): music, except amb_singe_02, a bed; the sound-triggers of the banks' levels play
/// them (e05 and sdm_e40, e13 and sdm_e13, m02_a and m04_a, m08_a, sdm_m07).
pub(super) const BANK_TRACKS: [GroupTrack; 5] = [
    GroupTrack { name: "creepy_estuary_mutant_01", banks: &["e05", "sdm_e40"], sound: 0x10FB_2011, secs: 104.06 },
    GroupTrack { name: "amb_singe_02", banks: &["e13", "sdm_e13"], sound: 0x1494_837E, secs: 44.77 },
    GroupTrack { name: "ambience19-2ch", banks: &["m02_a", "m04_a"], sound: 0x0ACA_9F39, secs: 21.72 },
    GroupTrack { name: "caspian_amb_mux_temp", banks: &["m08_a"], sound: 0xE80A_7EE6, secs: 164.53 },
    GroupTrack { name: "ambiencehellish-2ch", banks: &["sdm_m07"], sound: 0xE70F_B918, secs: 157.19 },
];

/// What `BF_TEST_MUSIC` puts before a bank track's name (`bank:<name>`, #396).
const BANK_PREFIX: &str = "bank:";
/// The same hook's first name, from when the group was headed "unused" (#396): still taken.
const UNUSED_PREFIX: &str = "unused:";
/// What `BF_TEST_MUSIC` puts before a level and a track to play that one alone
/// (`track:<level>:<name or place>`, #398).
const TRACK_PREFIX: &str = "track:";
/// What `BF_TEST_MUSIC` puts before a level to open the music menu on that bank's tracks
/// (`tracks:<level>`, #398).
const TRACKS_PREFIX: &str = "tracks:";

/// The bank track a hook names (the part after `bank:`): its name in any case, else the first
/// whose name contains it (`bank:creepy`), else its place in `BANK_TRACKS` from 0.
fn find_group_track(name: &str) -> Option<usize> {
    let n = name.trim().to_ascii_lowercase();
    BANK_TRACKS.iter().position(|t| t.name.to_ascii_lowercase() == n)
        .or_else(|| (!n.is_empty()).then(|| BANK_TRACKS.iter().position(|t| t.name.to_ascii_lowercase().contains(&n))).flatten())
        .or_else(|| n.parse::<usize>().ok().filter(|&i| i < BANK_TRACKS.len()))
}

/// The track of a bank's list a hook names (the part after the level in `track:<level>:`): its
/// place in the list from 0 when it is a number, else its name in any case, else the first whose
/// name contains it (a number first, since the names have digits in them: `1` is the second).
fn find_bank_track(tracks: &[BankTrack], which: &str) -> Option<usize> {
    let w = which.trim().to_ascii_lowercase();
    if let Ok(i) = w.parse::<usize>() {
        return Some(i).filter(|&i| i < tracks.len());
    }
    tracks.iter().position(|t| t.name.to_ascii_lowercase() == w)
        .or_else(|| (!w.is_empty()).then(|| tracks.iter().position(|t| t.name.to_ascii_lowercase().contains(&w))).flatten())
}

impl GroupTrack {
    /// How the menu, the messages and the logs name it: "bank track creepy_estuary_mutant_01
    /// (e05.xwb)".
    fn label(&self) -> String {
        format!("bank track {} ({}.xwb)", self.name, self.banks[0])
    }
}

/// How the menu, the messages and the logs name a track picked from a bank's list: "track
/// caspian_action7 (m03_b.xwb)".
fn track_label(t: &BankTrack) -> String {
    format!("track {} ({}.xwb)", t.name, t.file.library)
}

/// The word for a track's role in the lists: the Type of the sound entry that plays it
/// (`audio::MUSIC_TYPE` the level's music, any other, "ambient", a bed), or that nothing does.
fn kind_word(kind: Option<u32>) -> &'static str {
    match kind {
        Some(bf_viewer::bf::audio::MUSIC_TYPE) => "music",
        Some(_) => "ambience",
        None => "no entry",
    }
}

/// What a track picked alone plays as and at what share of the music volume (`music_mix`'s):
/// "music x1.00", or "ambience x0.80" for a bed (`is_bed`).
fn share_word(t: &BankTrack) -> String {
    if super::is_bed(t.kind, &t.name) { format!("ambience x{:.2}", super::AMBIENCE_VOLUME) } else { "music x1.00".to_string() }
}

/// The bank the music menu's "this map's own" plays on `map`: its own, the test map's TEST_MUSIC.
fn own_bank(map: &str) -> String {
    if map == "flat" { TEST_MUSIC.to_string() } else { map.to_string() }
}

/// Loads level `zone`'s music bank on a thread and mixes it as `setup` mixes a level's (play.rs
/// `music_mix`, by the Types of the sound entries that play the tracks, found through that level's
/// sound bank by each wave's place in the bank file; the name only for a wave no entry plays).
fn music_thread(zone: &str, schemas: SchemaSet) -> Mutex<Receiver<MusicLoad>> {
    let (tx, rx) = std::sync::mpsc::channel();
    let name = zone.to_string();
    std::thread::spawn(move || {
        let dir = data_dir();
        let load: MusicLoad = (|| {
            let bank = Game::sound_bank_of(&schemas, &dir, &name).unwrap_or_default();
            Ok(super::music_mix(Game::load_music_mix(&dir, &name, &bank)?))
        })();
        let _ = tx.send(load);
    });
    Mutex::new(rx)
}

/// Reads a track out of its bank file (the file record's offset and size, `MusicFile::read`) and
/// mixes it as `music_mix` mixes a track given alone: the Type of the sound entry that plays it
/// says whether it's music (at the music's share) or a bed (alone, at the bed's share); only a
/// track no entry plays goes by its name. BF_TOOLS_LOG or BF_SOUND_LOG print what was read.
fn load_track(dir: &std::path::Path, track: &BankTrack, label: &str, also: &[String]) -> MusicLoad {
    let bytes = track.file.read(dir)?;
    let wav = track.file.wav(&bytes).ok_or_else(|| format!("{}: its format isn't one the decoder reads", track.name))?;
    if std::env::var("BF_TOOLS_LOG").is_ok() || std::env::var("BF_SOUND_LOG").is_ok() {
        println!("test tools: {label} read: {} bytes at {} of {}.xwb, {} Hz {}-channel audio, {:.1} s once decoded{}", track.file.size, track.file.offset,
                 track.file.library, (track.file.format >> 5) & 0x3ff_ffff, (track.file.format >> 2) & 7, bf_viewer::bf::audio::wav_secs(&wav),
                 if also.is_empty() { String::new() } else { format!("; the same wave is in {}", also.join(", ")) });
    }
    Ok(super::music_mix(vec![MixTrack { name: track.name.clone(), wav, kind: track.kind }]))
}

/// The tracks of bank `zone` (`SoundBank::bank_tracks`): the start of data/sounds/<zone>.xwb
/// (its entry table) with the level's sound bank (the file records and the entries that play the
/// waves).
fn list_tracks(dir: &std::path::Path, schemas: &SchemaSet, zone: &str) -> Result<Vec<BankTrack>, String> {
    use std::io::Read;
    let path = dir.join("sounds").join(format!("{zone}.xwb"));
    let mut head = vec![];
    std::fs::File::open(&path).and_then(|f| f.take(XWB_HEAD).read_to_end(&mut head)).map_err(|e| format!("{}: {e}", path.display()))?;
    let sounds = Game::sound_bank_of(schemas, dir, zone)?;
    let tracks = sounds.bank_tracks(zone, &head);
    if tracks.is_empty() { Err(format!("{} holds no tracks", path.display())) } else { Ok(tracks) }
}

/// Reads a bank track of the group out of its first bank on a thread (`load_track` over the track
/// as the bank lists it), as `music_thread` loads a level's bank.
fn group_thread(track: &'static GroupTrack, schemas: SchemaSet) -> Mutex<Receiver<MusicLoad>> {
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let dir = data_dir();
        let load = list_tracks(&dir, &schemas, track.banks[0]).and_then(|list| {
            let t = list.iter().find(|t| t.name == track.name).ok_or_else(|| format!("{} isn't in {}.xwb", track.name, track.banks[0]))?;
            let also: Vec<String> = track.banks[1..].iter().map(|b| format!("{b}.xwb")).collect();
            load_track(&dir, t, &track.label(), &also)
        });
        let _ = tx.send(load);
    });
    Mutex::new(rx)
}

/// Reads one track picked from a bank's list on a thread (`load_track`).
fn track_thread(track: BankTrack) -> Mutex<Receiver<MusicLoad>> {
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let _ = tx.send(load_track(&data_dir(), &track, &track_label(&track), &[]));
    });
    Mutex::new(rx)
}

/// A bank's list of tracks read on a thread (`list_tracks`).
fn list_thread(zone: &str, schemas: SchemaSet) -> Mutex<Receiver<Result<Vec<BankTrack>, String>>> {
    let (tx, rx) = std::sync::mpsc::channel();
    let zone = zone.to_string();
    std::thread::spawn(move || {
        let _ = tx.send(list_tracks(&data_dir(), &schemas, &zone));
    });
    Mutex::new(rx)
}

/// A level the menus list: its archive's name ("flat": the test map), the name the game gives
/// it (`CampaignLevel::name`; a mission's zones by their archive too), its group, and whether
/// the front end plays it alone (the deathmatch arenas).
#[derive(Clone, Debug)]
pub(super) struct Entry {
    pub(super) zone: String,
    pub(super) title: String,
    group: &'static str,
    deathmatch: bool,
}

/// What the scan of the levels found, per level (an index into `Lists::levels`): whether its
/// level file names a sky mesh, and its music bank's tracks; and how long it took (s).
struct Scan(Vec<(usize, bool, Vec<String>)>, f32);

/// The session's lists, kept across level switches: the levels, those with a sky and those with a
/// music bank (once the scan is in), and the skies made ready so far (by level: their layers and
/// background colour).
#[derive(Resource, Default)]
struct Lists {
    levels: Vec<Entry>,
    skies: Vec<usize>,
    banks: Vec<(usize, Vec<String>)>,
    scanned: bool,
    scan: Option<Mutex<Receiver<Scan>>>,
    ready: HashMap<String, (SkyLayers, [f32; 3])>,
}

/// A level's sky loaded on a thread: the game data holding its meshes and textures (dropped
/// once they're made ready), its layers, where they sit, its background colour and how it turns.
type SkyLoad = Result<(Game, Vec<Geoset>, Vec3, [f32; 3], SkySpin), String>;
/// A level's music bank decoded and mixed on a thread (`music_mix`).
type MusicLoad = Result<Vec<(String, Vec<u8>, &'static str, f32)>, String>;
/// A level loaded on a thread for the switch, and a note to show (when it fell back).
type LevelLoad = Result<(LoadedMap, Option<String>), String>;

/// The map's state of the menus (`start_map` as each map starts): the picks, what's shown and
/// playing, the map's own clear colour (before a sky changed it), what's loading.
#[derive(Resource, Default)]
struct World {
    sky_pick: usize,
    music_pick: usize,
    level_pick: usize,
    sky: String,
    music: String,
    own_clear: Option<Color>,
    sky_loading: Option<(String, Mutex<Receiver<SkyLoad>>)>,
    music_loading: Option<(String, Mutex<Receiver<MusicLoad>>)>,
    /// the level whose bank is playing (picked, or the map's own picked again; none: off or the map's
    /// own from the start), for next / previous: its place in `Lists::banks` is looked up then, so a
    /// pick made before the list was in counts too
    bank: Option<String>,
    /// whether the map's start has been logged (BF_TOOLS_LOG)
    audited: bool,
    /// the music loading is the test map's own at its start (TEST_MUSIC): it comes in without a
    /// message (nobody picked it)
    music_quiet: bool,
    /// the music menu's list of one bank's tracks, open on this bank with this pick (Shift+Enter
    /// on a bank, `BF_TEST_MUSIC=<s>,tracks:<level>`, #398); None while the list of banks shows
    tracks: Option<(String, usize)>,
    /// the track lists read so far, by bank (each is read once, on a thread)
    track_lists: HashMap<String, Vec<BankTrack>>,
    /// a track list being read
    track_loading: Option<TrackLoading>,
    /// Backspace was pressed while the track list was open: the list's back, taken off the demo's
    /// own use of the key (leaving for the front end, `hold_back`)
    back: bool,
}

/// A bank's track list being read on a thread (`list_thread`): the bank, the track to play when
/// it's in (a hook's `track:<level>:<name or place>`), and the thread's result.
struct TrackLoading {
    zone: String,
    then: Option<String>,
    rx: Mutex<Receiver<Result<Vec<BankTrack>, String>>>,
}

/// The level switch asked for, and (once it's loading) the thread's result; where from; and the
/// levels that failed to load on the way (the fall back: the map it came from, then the test
/// map), with why.
#[derive(Resource)]
struct Switch {
    to: Entry,
    from: String,
    rx: Option<Mutex<Receiver<LevelLoad>>>,
    failed: Vec<String>,
}

/// A note for the next map's first frame (a level that didn't load).
#[derive(Resource, Default)]
struct Notice(Option<String>);

/// The open world menu's text, shown by play_testtools.rs's panel.
#[derive(Resource, Default)]
pub(super) struct WorldText(pub(super) String);

/// The level switch's loading screen.
#[derive(Component)]
struct SwitchPart;

/// How many assets of each kind there are, for BF_TOOLS_LOG's account of a switch (what a map
/// loaded should be gone once it's left, but for the skies kept on purpose).
#[derive(bevy::ecs::system::SystemParam)]
struct AssetCounts<'w> {
    meshes: Res<'w, Assets<Mesh>>,
    images: Res<'w, Assets<Image>>,
    materials: Res<'w, Assets<StandardMaterial>>,
    level_materials: Res<'w, Assets<bf_viewer::level_scene::LevelMaterial>>,
    sounds: Res<'w, Assets<AudioSource>>,
    clips: Res<'w, Assets<AnimationClip>>,
    buffers: Res<'w, Assets<bevy::render::storage::ShaderStorageBuffer>>,
    bindposes: Res<'w, Assets<bevy::render::mesh::skinning::SkinnedMeshInverseBindposes>>,
    liquids: Res<'w, Assets<bf_viewer::level_scene::LiquidMaterial>>,
    graphs: Res<'w, Assets<AnimationGraph>>,
}

/// The process's working set and private bytes (MB), for BF_TOOLS_LOG's account of a switch
/// (kernel32's K32GetProcessMemoryInfo, PROCESS_MEMORY_COUNTERS_EX); none off Windows.
#[cfg(windows)]
pub(super) fn process_mb() -> Option<(f32, f32)> {
    #[repr(C)]
    #[derive(Default)]
    struct Counters {
        cb: u32,
        page_faults: u32,
        peak_working_set: usize,
        working_set: usize,
        quota_peak_paged: usize,
        quota_paged: usize,
        quota_peak_non_paged: usize,
        quota_non_paged: usize,
        pagefile: usize,
        peak_pagefile: usize,
        private: usize,
    }
    unsafe extern "system" {
        fn GetCurrentProcess() -> isize;
        fn K32GetProcessMemoryInfo(process: isize, counters: *mut Counters, cb: u32) -> i32;
    }
    let mut c = Counters { cb: std::mem::size_of::<Counters>() as u32, ..Default::default() };
    // SAFETY: a pseudo handle for this process, and a counters struct of the size given
    let ok = unsafe { K32GetProcessMemoryInfo(GetCurrentProcess(), &mut c, c.cb) } != 0;
    ok.then(|| (c.working_set as f32 / 1048576.0, c.private as f32 / 1048576.0))
}

/// See the Windows one: nothing to say elsewhere.
#[cfg(not(windows))]
pub(super) fn process_mb() -> Option<(f32, f32)> {
    None
}

impl AssetCounts<'_> {
    /// "meshes 123, images 45 (67 MB), ..., process 900 MB (private 1100 MB)"
    fn line(&self) -> String {
        let process = process_mb().map_or(String::new(), |(ws, private)| format!("; process {ws:.0} MB (private {private:.0} MB)"));
        self.counts() + &process
    }

    /// The counts alone.
    fn counts(&self) -> String {
        let image_mb = self.images.iter().map(|(_, i)| i.data.as_ref().map_or(0, |d| d.len())).sum::<usize>() as f32 / 1048576.0;
        let mesh_mb = self.meshes.iter().map(|(_, m)| m.count_vertices() * 48).sum::<usize>() as f32 / 1048576.0;
        let sound_mb = self.sounds.iter().map(|(_, s)| s.bytes.len()).sum::<usize>() as f32 / 1048576.0;
        format!("meshes {} (~{mesh_mb:.0} MB), images {} ({image_mb:.0} MB), materials {}, level materials {}, sounds {} ({sound_mb:.0} MB), clips {}, buffers {}, bindposes {}, liquids {}, graphs {}",
                self.meshes.len(), self.images.len(), self.materials.len(), self.level_materials.len(), self.sounds.len(), self.clips.len(),
                self.buffers.len(), self.bindposes.len(), self.liquids.len(), self.graphs.len())
    }
}

pub fn plugin(app: &mut App) {
    app.init_resource::<Lists>().init_resource::<World>().init_resource::<WorldText>().init_resource::<Notice>()
        .add_systems(OnEnter(AppState::Playing), start_map.after(setup).run_if(resource_exists::<TestMap>))
        .add_systems(Update, (world_menus, finish_loads, audit, world_text).chain().after(update_player).before(super::testtools::show_panel)
            .run_if(super::testtools::active))
        .add_systems(Update, hold_back.after(read_input).before(world_menus).before(super::back_to_menu).run_if(super::testtools::active))
        .add_systems(OnEnter(AppState::Switching), start_switch)
        .add_systems(Update, finish_switch.run_if(in_state(AppState::Switching)))
        .add_systems(OnExit(AppState::Switching), end_switch);
}

/// Backspace with the music menu's track list open goes back to the banks (#398), and is not the
/// demo's own use of the key, leaving the session for the front end (play.rs `back_to_menu`, which
/// reads it later in the frame): the press is taken off the input, as the end menu takes its
/// presses (play_endmenu.rs `clear_presses`), and `world_menus` is told. It runs after
/// `read_input` and before `world_menus` and `back_to_menu`, so the press is the list's back in the
/// frame it is made.
fn hold_back(mut keys: ResMut<ButtonInput<KeyCode>>, tools: Res<Tools>, mut world: ResMut<World>) {
    if tools.submenu && keys.clear_just_pressed(KeyCode::Backspace) {
        world.back = true;
    }
}

/// Whether `zone` is a deathmatch arena by the game's level list (campaign-bf.xmb Type
/// LEVEL_DEATHMATCH): the front end plays those alone, and so does a test session, whether it
/// starts on one (`--test` with BF_MAP, play.rs `main`) or switches to one.
pub fn arena(game: &Game, zone: &str) -> bool {
    game.campaign.iter().any(|l| l.kind == LEVEL_DEATHMATCH && (l.file == zone || l.zones.iter().any(|z| z == zone)))
}

/// The game's level list as the level menu shows it: the test map, then the squad deathmatch
/// maps, the deathmatch arenas and the missions (each zone), each group in the game's order.
pub(super) fn level_list(game: &Game) -> Vec<Entry> {
    let mut out = vec![Entry { zone: "flat".into(), title: "Test map".into(), group: "test", deathmatch: false }];
    let s = |id: u32| game.strings.get(&id).cloned().filter(|t| !t.trim().is_empty());
    for (kind, group) in [(LEVEL_SQUAD_DEATHMATCH, "squad deathmatch"), (LEVEL_DEATHMATCH, "deathmatch"), (0, "missions")] {
        for l in game.campaign.iter().filter(|l| if kind == 0 { l.kind != LEVEL_DEATHMATCH && l.kind != LEVEL_SQUAD_DEATHMATCH } else { l.kind == kind }) {
            let zones = if l.zones.is_empty() { vec![l.file.clone()] } else { l.zones.clone() };
            for z in &zones {
                if out.iter().any(|e| &e.zone == z) {
                    continue;
                }
                let title = s(l.name).map(|t| t.trim().to_string()).unwrap_or_else(|| z.clone());
                out.push(Entry { zone: z.clone(), title, group, deathmatch: arena(game, z) });
            }
        }
    }
    out
}

/// Which levels have a sky and a music bank (see `Scan`): each one's level file alone (its
/// archive read up to it) and the start of its music bank.
fn scan(schemas: SchemaSet, levels: Vec<String>) -> Scan {
    let t = std::time::Instant::now();
    let dir = data_dir();
    let out = levels.iter().enumerate().filter(|(_, z)| *z != "flat").map(|(i, z)| {
        let sky = Game::level_file_of(&schemas, &dir, z).ok().flatten().is_some_and(|r| bf_viewer::bf::level::Level::sky_mesh(&r) != 0);
        let mut head = vec![];
        if let Ok(f) = std::fs::File::open(dir.join("sounds").join(format!("{z}.xwb"))) {
            use std::io::Read;
            let _ = f.take(XWB_HEAD).read_to_end(&mut head);
        }
        (i, sky, bf_viewer::bf::audio::xwb_names(&head))
    }).collect();
    Scan(out, t.elapsed().as_secs_f32())
}

/// A map starts: the menus' state anew (nothing loading carries over), the level list (once a
/// session) and the scan started; the level menu on this map. The test map's music (TEST_MUSIC)
/// starts loading as the music menu loads a bank, and plays once it's in (`finish_loads`; not
/// under BF_NO_MUSIC, and BF_MUTE loads and logs it but plays nothing, as with the menu).
fn start_map(mut world: ResMut<World>, mut lists: ResMut<Lists>, game: Res<GameData>, current: Res<CurrentMap>, mut text: ResMut<WorldText>) {
    *world = World::default();
    text.0.clear();
    if lists.levels.is_empty() {
        lists.levels = level_list(&game.0);
    }
    if !lists.scanned && lists.scan.is_none() {
        let (tx, rx) = std::sync::mpsc::channel();
        let (schemas, levels) = (game.0.schemas.clone(), lists.levels.iter().map(|e| e.zone.clone()).collect());
        std::thread::spawn(move || {
            let _ = tx.send(scan(schemas, levels));
        });
        lists.scan = Some(Mutex::new(rx));
    }
    world.level_pick = lists.levels.iter().position(|e| e.zone == current.0).unwrap_or(0);
    world.sky = if current.0 == "flat" { "none".into() } else { format!("{}'s own", current.0) };
    world.music = world.sky.clone();
    if current.0 == "flat" && std::env::var("BF_NO_MUSIC").is_err() {
        world.music_loading = Some((TEST_MUSIC.into(), music_thread(TEST_MUSIC, game.0.schemas.clone())));
        world.bank = Some(TEST_MUSIC.into());
        world.music_quiet = true;
        if std::env::var("BF_TOOLS_LOG").is_ok() || std::env::var("BF_SOUND_LOG").is_ok() {
            println!("test map: its music, {TEST_MUSIC}'s bank, loading");
        }
    }
}

/// A level by name: its archive's name, else the first whose title contains it (any case).
fn find_level(lists: &Lists, name: &str) -> Option<usize> {
    let n = name.trim().to_ascii_lowercase();
    let n = if n == "test" { "flat".to_string() } else { n };
    lists.levels.iter().position(|e| e.zone == n).or_else(|| lists.levels.iter().position(|e| e.title.to_ascii_lowercase().contains(&n)))
}

/// What a menu's pick means: entry 0 "none" / "off", 1 the map's own, then the list's levels; and
/// in the music menu, after the banks, a bank track (an index into `BANK_TRACKS`, #396), or a
/// track picked from a bank's own list (#398).
enum Pick {
    None,
    Own,
    Level(usize),
    Group(usize),
    Track(Box<BankTrack>),
}

/// The menus (Y, U, L: their keys are the tool registry's, `editor::open_tools`) and their
/// hooks: picking, and asking for a sky, a bank or a switch.
#[allow(clippy::too_many_arguments)]
fn world_menus(mut commands: Commands, keys: Res<ButtonInput<KeyCode>>,
               scroll: Res<AccumulatedMouseScroll>, mut tools: ResMut<Tools>, mut world: ResMut<World>, mut lists: ResMut<Lists>,
               script: Res<Script>, current: Res<CurrentMap>, game: Res<GameData>, mut status: ResMut<UsePanel>,
               mut next: ResMut<NextState<AppState>>, mut clear: ResMut<ClearColor>, skies: Query<Entity, With<SkyLayer>>,
               music: Query<Entity, With<super::LevelMusic>>) {
    // (a plain reference, so the track lists and the picks can be borrowed side by side)
    let world = &mut *world;
    // the scan's lists, once in
    let got = lists.scan.as_ref().and_then(|r| r.lock().ok().and_then(|r| r.try_recv().ok()));
    if let Some(Scan(found, secs)) = got {
        lists.skies = found.iter().filter(|f| f.1).map(|f| f.0).collect();
        lists.banks = found.into_iter().filter(|f| !f.2.is_empty()).map(|f| (f.0, f.2)).collect();
        // (before the list was in, the music menu's picks past "this map's own" were the unused
        // tracks': the banks come in before them, so a pick on one of those moves down with it)
        if world.music_pick >= 2 {
            world.music_pick += lists.banks.len();
        }
        lists.scanned = true;
        lists.scan = None;
        println!("test tools: {} levels, {} with a sky, {} with a music bank (listed in {secs:.1} s)", lists.levels.len() - 1, lists.skies.len(), lists.banks.len());
    }
    let log = std::env::var("BF_TOOLS_LOG").is_ok();
    let mut sky: Option<Pick> = None;
    let mut bank: Option<Pick> = None;
    let mut step_bank = 0;
    let mut switch: Option<usize> = None;
    // a bank's track list asked for: the bank, and the track to play when it's in
    let mut tracks_of: Option<(String, Option<String>)> = None;
    for hook in &script.now {
        match hook {
            Hook::SkyMenu => tools.panel = Panel::Sky,
            Hook::MusicMenu => tools.panel = Panel::Music,
            Hook::LevelMenu => tools.panel = Panel::Levels,
            Hook::Sky(n) => match n.to_ascii_lowercase().as_str() {
                "none" => sky = Some(Pick::None),
                "map" => sky = Some(Pick::Own),
                _ => match find_level(&lists, n) {
                    Some(i) => sky = Some(Pick::Level(i)),
                    None => println!("test tools: no level {n}"),
                },
            },
            Hook::Music(n) => match n.to_ascii_lowercase().as_str() {
                "off" => bank = Some(Pick::None),
                "map" => bank = Some(Pick::Own),
                "next" => step_bank = 1,
                "prev" => step_bank = -1,
                // (`bank:<name>`, or `unused:<name>` as it was called: one of the five bank tracks, #396)
                u if u.starts_with(BANK_PREFIX) || u.starts_with(UNUSED_PREFIX) => {
                    let part = u.split_once(':').map_or("", |p| p.1);
                    match find_group_track(part) {
                        Some(i) => bank = Some(Pick::Group(i)),
                        None => println!("test tools: no bank track {part}"),
                    }
                }
                // (`track:<level>:<name or place>`: that bank's track alone; `tracks:<level>`: the
                // menu opens on that bank's tracks, #398)
                u if u.starts_with(TRACK_PREFIX) => match u[TRACK_PREFIX.len()..].split_once(':').map(|(z, w)| (find_level(&lists, z), w)) {
                    Some((Some(i), which)) => tracks_of = Some((lists.levels[i].zone.clone(), Some(which.to_string()))),
                    _ => println!("test tools: {n} isn't track:<level>:<name or place> of a level"),
                },
                u if u.starts_with(TRACKS_PREFIX) => match find_level(&lists, &u[TRACKS_PREFIX.len()..]) {
                    Some(i) => {
                        tools.panel = Panel::Music;
                        world.tracks = Some((lists.levels[i].zone.clone(), 0));
                        tracks_of = Some((lists.levels[i].zone.clone(), None));
                    }
                    None => println!("test tools: no level {}", &n[TRACKS_PREFIX.len()..]),
                },
                _ => match find_level(&lists, n) {
                    Some(i) => bank = Some(Pick::Level(i)),
                    None => println!("test tools: no level {n}"),
                },
            },
            Hook::Level(n) => match find_level(&lists, n) {
                Some(i) => switch = Some(i),
                None => println!("test tools: no level {n}"),
            },
            _ => {}
        }
    }
    // a track list that was being read has come in: kept, and its track played if a hook asked
    let arrived = world.track_loading.as_ref().and_then(|l| poll(&l.rx).map(|r| (l.zone.clone(), l.then.clone(), r)));
    if let Some((zone, then, result)) = arrived {
        world.track_loading = None;
        match result {
            Ok(rows) => {
                if log {
                    let list: Vec<String> = rows.iter().map(|t| format!("{} ({}, {:.1} s)", t.name, kind_word(t.kind), t.file.secs())).collect();
                    println!("test tools: tracks of {zone}: {}: {}", rows.len(), list.join(", "));
                }
                world.track_lists.insert(zone.clone(), rows);
                if then.is_some() {
                    tracks_of = Some((zone, then));
                }
            }
            Err(e) => {
                eprintln!("test tools: no tracks from {zone}: {e}");
                status.message = Some((format!("No tracks from {zone}"), MESSAGE_TIME));
                if world.tracks.as_ref().is_some_and(|t| t.0 == zone) {
                    world.tracks = None;
                }
            }
        }
    }
    // (the track list is the music menu's: it goes with it)
    if tools.panel != Panel::Music {
        world.tracks = None;
    }
    // the open menu's keys
    let step = |k: KeyCode| keys.just_pressed(k) as i32;
    let wheel = if scroll.delta.y < 0.0 { 1 } else if scroll.delta.y > 0.0 { -1 } else { 0 };
    let d = step(KeyCode::ArrowDown) - step(KeyCode::ArrowUp) + wheel;
    let take = keys.just_pressed(KeyCode::Enter) || tools.click;
    // (Shift with the pick opens a bank's tracks instead of playing its mix)
    let shift = keys.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]);
    let back = keys.just_pressed(KeyCode::Escape) || std::mem::take(&mut world.back);
    let pick_of = |k: usize, list: &[usize]| match k {
        0 => Pick::None,
        1 => Pick::Own,
        k => Pick::Level(list[k - 2]),
    };
    match tools.panel {
        Panel::Sky => {
            let n = lists.skies.len() + 2;
            world.sky_pick = (world.sky_pick as i32 + d).rem_euclid(n as i32) as usize;
            if take {
                sky = Some(pick_of(world.sky_pick, &lists.skies));
            }
        }
        // a bank's track list: Up / Down pick, Enter or a click plays the track alone, Esc or Backspace
        // goes back to the banks (Left / Right and the banks' picks wait)
        Panel::Music if world.tracks.is_some() => {
            let (zone, pick) = world.tracks.clone().unwrap_or_default();
            let rows = world.track_lists.get(&zone);
            let n = rows.map_or(0, |r| r.len());
            let pick = if n == 0 { 0 } else { (pick as i32 + d).rem_euclid(n as i32) as usize };
            world.tracks = Some((zone, pick));
            if back {
                world.tracks = None;
            } else if take {
                bank = rows.and_then(|r| r.get(pick)).map(|t| Pick::Track(Box::new(t.clone())));
            }
        }
        Panel::Music => {
            let banks: Vec<usize> = lists.banks.iter().map(|b| b.0).collect();
            // (off, the map's own, the banks, then the bank tracks)
            let n = banks.len() + 2 + BANK_TRACKS.len();
            world.music_pick = (world.music_pick as i32 + d).rem_euclid(n as i32) as usize;
            if take && shift {
                // (a bank's tracks: the picked bank's, else the map's own; not "off" or a bank track)
                let zone = match world.music_pick {
                    0 => None,
                    1 => Some(own_bank(&current.0)),
                    k => banks.get(k - 2).map(|&b| lists.levels[b].zone.clone()),
                };
                match zone {
                    Some(zone) => {
                        world.tracks = Some((zone.clone(), 0));
                        tracks_of = Some((zone, None));
                    }
                    None => status.message = Some(("Shift+Enter needs a bank (or this map's own) picked".into(), MESSAGE_TIME)),
                }
            } else if take {
                bank = Some(match world.music_pick.checked_sub(banks.len() + 2) {
                    Some(i) => Pick::Group(i),
                    None => pick_of(world.music_pick, &banks),
                });
            }
            step_bank += step(KeyCode::ArrowRight) - step(KeyCode::ArrowLeft);
        }
        Panel::Levels => {
            let n = lists.levels.len().max(1);
            world.level_pick = (world.level_pick as i32 + d).rem_euclid(n as i32) as usize;
            if take {
                switch = Some(world.level_pick);
            }
        }
        _ => {}
    }
    // a bank's track list asked for (opened, or a track of it to play): shown from what's read
    // already, else read on a thread; a track named is played once the list is there
    if let Some((zone, then)) = tracks_of {
        match world.track_lists.get(&zone) {
            None => {
                let same = world.track_loading.as_ref().is_some_and(|l| l.zone == zone);
                if same {
                    // (already being read: the track to play is added to the request)
                    if let Some(l) = world.track_loading.as_mut() {
                        l.then = then.or(l.then.take());
                    }
                } else {
                    if log {
                        println!("test tools: reading the tracks of {zone}");
                    }
                    world.track_loading = Some(TrackLoading { rx: list_thread(&zone, game.0.schemas.clone()), zone, then });
                }
            }
            Some(rows) => {
                if let Some(which) = then {
                    match find_bank_track(rows, &which) {
                        Some(i) => {
                            if let Some(t) = world.tracks.as_mut().filter(|t| t.0 == zone) {
                                t.1 = i;
                            }
                            bank = Some(Pick::Track(Box::new(rows[i].clone())));
                        }
                        None => {
                            println!("test tools: no track {which} in {zone} ({} tracks)", rows.len());
                            status.message = Some((format!("No track {which} in {zone}"), MESSAGE_TIME));
                        }
                    }
                }
            }
        }
    }
    if step_bank != 0 {
        if lists.banks.is_empty() {
            status.message = Some(("Still listing the music banks".into(), MESSAGE_TIME));
        } else {
            // (from the bank playing: one picked, else the map's own, else the list's ends)
            let n = lists.banks.len() as i32;
            let own = lists.banks.iter().position(|b| lists.levels[b.0].zone == current.0);
            let picked = world.bank.as_deref().and_then(|z| lists.banks.iter().position(|b| lists.levels[b.0].zone == z));
            let k = match picked.or(own.filter(|_| world.music != "off")) {
                Some(k) => (k as i32 + step_bank).rem_euclid(n),
                None if step_bank > 0 => 0,
                None => n - 1,
            } as usize;
            world.music_pick = k + 2;
            bank = Some(Pick::Level(lists.banks[k].0));
        }
    }
    // a sky: none, or a level's (made ready already, or loaded on a thread). While one loads,
    // other picks are ignored (each load reads the game data anew: they'd pile up)
    if let (Some(_), Some((z, _))) = (&sky, &world.sky_loading) {
        status.message = Some((format!("Still loading the sky of {z}"), MESSAGE_TIME));
        if log {
            println!("test tools: sky pick ignored: still loading the sky of {z}");
        }
        sky = None;
    }
    if let Some(p) = sky {
        let zone = match p {
            Pick::None => None,
            Pick::Own => Some(current.0.clone()).filter(|m| m != "flat"),
            Pick::Level(i) => Some(lists.levels[i].zone.clone()),
            // (the music menu's: no sky picks one)
            Pick::Group(_) | Pick::Track(_) => None,
        };
        world.sky_loading = None;
        match zone {
            None => {
                let own = *world.own_clear.get_or_insert(clear.0);
                for e in &skies {
                    commands.entity(e).despawn();
                }
                clear.0 = own;
                world.sky = "none".into();
                if log {
                    println!("test tools: sky: none");
                }
                status.message = Some(("Sky: none".into(), MESSAGE_TIME));
            }
            Some(z) if lists.ready.contains_key(&z) => {
                world.own_clear.get_or_insert(clear.0);
                world.sky = z.clone();
                let (layers, bg) = &lists.ready[&z];
                show_sky(&mut commands, layers, *bg, &skies, &mut clear);
                if log {
                    println!("test tools: sky: {z} ({} layers, ready)", layers.layers.len());
                }
                status.message = Some((format!("Sky: {z}"), MESSAGE_TIME));
            }
            Some(z) => {
                let (tx, rx) = std::sync::mpsc::channel();
                let name = z.clone();
                std::thread::spawn(move || {
                    let dir = data_dir();
                    let load = (|| {
                        let mut game = Game::load(&dir)?;
                        game.load_level(&dir, &name)?;
                        let (sky, at, bg, spin) = bf_viewer::bf::level::Level::load_sky(&game)?;
                        Ok((game, sky, at, bg, spin))
                    })();
                    let _ = tx.send(load);
                });
                world.sky_loading = Some((z.clone(), Mutex::new(rx)));
                status.message = Some((format!("Loading the sky of {z}"), MESSAGE_TIME));
            }
        }
    }
    // music: off, or a level's bank (decoded and mixed on a thread). (A pick while the test map's
    // own loads at its start takes its place: that one is dropped)
    if let (Some(_), Some((z, _)), false) = (&bank, &world.music_loading, world.music_quiet) {
        status.message = Some((format!("Still loading the music of {z}"), MESSAGE_TIME));
        if log {
            println!("test tools: music pick ignored: still loading the music of {z}");
        }
        bank = None;
    }
    if let Some(p) = bank {
        // what the pick plays: nothing, a bank's mix, or one track alone: a bank track of the
        // group, or one picked from a bank's list
        enum What {
            Off,
            Mix(String),
            Group(&'static GroupTrack),
            Track(BankTrack),
        }
        let what = match p {
            Pick::None => What::Off,
            Pick::Own => What::Mix(own_bank(&current.0)),
            Pick::Level(i) => What::Mix(lists.levels[i].zone.clone()),
            Pick::Group(i) => What::Group(&BANK_TRACKS[i]),
            Pick::Track(t) => What::Track(*t),
        };
        world.music_loading = None;
        world.music_quiet = false;
        // (a track counts as its bank for Left / Right: the next bank after it)
        world.bank = match &what {
            What::Off => None,
            What::Mix(z) => Some(z.clone()),
            What::Group(t) => Some(t.banks[0].to_string()),
            What::Track(t) => Some(t.file.library.to_string()),
        };
        match what {
            What::Off => {
                for e in &music {
                    commands.entity(e).despawn();
                }
                world.music = "off".into();
                if log {
                    println!("test tools: music off");
                }
                status.message = Some(("Music off".into(), MESSAGE_TIME));
            }
            What::Group(track) => {
                // (read from its first bank on a thread; the map's music stops once it's in, as a
                // level's bank's does)
                let label = track.label();
                world.music_loading = Some((label.clone(), group_thread(track, game.0.schemas.clone())));
                status.message = Some((format!("Loading the music of {label}"), MESSAGE_TIME));
            }
            What::Track(track) => {
                let label = track_label(&track);
                world.music_loading = Some((label.clone(), track_thread(track)));
                status.message = Some((format!("Loading the music of {label}"), MESSAGE_TIME));
            }
            What::Mix(z) => {
                world.music_loading = Some((z.clone(), music_thread(&z, game.0.schemas.clone())));
                status.message = Some((format!("Loading the music of {z}"), MESSAGE_TIME));
            }
        }
    }
    // (Esc closes the menu unless a track list is open: it only goes back, `camera_tool`)
    tools.submenu = tools.panel == Panel::Music && world.tracks.is_some();
    // a switch: the map goes (end_play), the next loads (start_switch)
    if let Some(i) = switch {
        let to = lists.levels[i].clone();
        if log {
            println!("test tools: switching from {} to {} ({})", current.0, to.zone, to.title);
        }
        tools.panel = Panel::None;
        commands.insert_resource(Switch { to, from: current.0.clone(), rx: None, failed: vec![] });
        next.set(AppState::Switching);
    }
}

/// Puts a sky up in place of the one there (`layers`, the clear colour its `background`).
fn show_sky(commands: &mut Commands, layers: &SkyLayers, background: [f32; 3], skies: &Query<Entity, With<SkyLayer>>, clear: &mut ClearColor) {
    for e in skies {
        commands.entity(e).despawn();
    }
    bf_viewer::level_scene::spawn_sky(commands, layers);
    clear.0 = Color::srgb(background[0], background[1], background[2]);
}

/// A thread's result, if it's in: its own, or an error if the thread ended without one (it
/// panicked), so nothing waits on it for ever.
fn poll<T>(rx: &Mutex<Receiver<Result<T, String>>>) -> Option<Result<T, String>> {
    match rx.lock().ok()?.try_recv() {
        Ok(x) => Some(x),
        Err(std::sync::mpsc::TryRecvError::Empty) => None,
        Err(std::sync::mpsc::TryRecvError::Disconnected) => Some(Err("the loading thread stopped".into())),
    }
}

/// A sky or a music bank loaded: the sky made ready (kept for the session) and put up, the

/// camera seeing as far as SKY_FAR; the music in place of what played.
#[allow(clippy::too_many_arguments)]
fn finish_loads(mut commands: Commands, mut world: ResMut<World>, mut lists: ResMut<Lists>, mut status: ResMut<UsePanel>,
                mut meshes: ResMut<Assets<Mesh>>, mut materials: ResMut<Assets<bf_viewer::level_scene::LevelMaterial>>,
                mut buffers: ResMut<Assets<bevy::render::storage::ShaderStorageBuffer>>, mut images: ResMut<Assets<Image>>,
                mut sources: ResMut<Assets<AudioSource>>, mut clear: ResMut<ClearColor>, skies: Query<Entity, With<SkyLayer>>,
                music: Query<Entity, With<super::LevelMusic>>, mut cam: Query<&mut Projection, With<MainCamera>>) {
    let log = std::env::var("BF_TOOLS_LOG").is_ok();
    let got = world.sky_loading.as_ref().and_then(|(z, r)| poll(r).map(|x| (z.clone(), x)));
    if let Some((z, load)) = got {
        world.sky_loading = None;
        match load {
            Ok((mut g, sky, at, bg, spin)) if !sky.is_empty() => {
                let layers = bf_viewer::level_scene::sky_layers(&mut g, &sky, at, spin, &mut meshes, &mut materials, &mut buffers, &mut images);
                world.own_clear.get_or_insert(clear.0);
                show_sky(&mut commands, &layers, bg, &skies, &mut clear);
                if log {
                    println!("test tools: sky: {z} ({} layers at {at:.0}, background {bg:.2?}, turning {:.4?} rad/s from {:.2?})", layers.layers.len(), spin.speed, spin.start);
                }
                lists.ready.insert(z.clone(), (layers, bg));
                world.sky = z.clone();
                status.message = Some((format!("Sky: {z}"), MESSAGE_TIME));
            }
            Ok(_) => status.message = Some((format!("{z} has no sky"), MESSAGE_TIME)),
            Err(e) => {
                eprintln!("test tools: no sky from {z}: {e}");
                status.message = Some((format!("No sky from {z}"), MESSAGE_TIME));
            }
        }
    }
    // (far enough for a sky's layers wherever one is up)
    if !skies.is_empty() {
        for mut p in &mut cam {
            if let Projection::Perspective(pp) = p.as_mut() {
                if pp.far < SKY_FAR {
                    pp.far = SKY_FAR;
                }
            }
        }
    }
    let got = world.music_loading.as_ref().and_then(|(z, r)| poll(r).map(|x| (z.clone(), x)));
    if let Some((z, load)) = got {
        world.music_loading = None;
        // (the test map's own at its start says nothing on screen: nobody picked it)
        let quiet = std::mem::take(&mut world.music_quiet);
        match load {
            Ok(tracks) => {
                for e in &music {
                    commands.entity(e).despawn();
                }
                let names: Vec<String> = tracks.iter().map(|t| format!("{} {}", t.2, t.0)).collect();
                // (BF_SOUND_LOG too: what plays, each track's share of the music volume, and
                // whether BF_MUTE keeps it silent)
                if log || std::env::var("BF_SOUND_LOG").is_ok() {
                    let mix: Vec<String> = tracks.iter().map(|t| format!("{} {} x{:.2}", t.2, t.0, t.3)).collect();
                    println!("test tools: music of {z}: {}{}", mix.join(", "),
                             if std::env::var("BF_MUTE").is_ok() { " (BF_MUTE: not played)" } else { ", looping" });
                }
                if std::env::var("BF_MUTE").is_err() {
                    for (_, wav, label, k) in tracks {
                        super::spawn_music(&mut commands, &mut sources, wav, label, k);
                    }
                }
                world.music = format!("{z}: {}", names.join(", "));
                if !quiet {
                    status.message = Some((format!("Music: {z}"), MESSAGE_TIME));
                }
            }
            Err(e) => {
                eprintln!("test tools: no music from {z}: {e}");
                if !quiet {
                    status.message = Some((format!("No music from {z}"), MESSAGE_TIME));
                }
            }
        }
    }
}

/// BF_TOOLS_LOG: what each map starts with (AUDIT_AT in), to show nothing carried over
/// from the map before: the entities kept from before it and its own, the sounds playing, the
/// sky's layers, the squad and NPCs, the placed breakables' boxes and the map's breakables. And
/// a note from the switch (a level that didn't load) on the status line.
#[allow(clippy::too_many_arguments)]
fn audit(mut world: ResMut<World>, mut notice: ResMut<Notice>, mut status: ResMut<UsePanel>, session: Res<Session>, current: Res<CurrentMap>,
         before: Option<Res<Before>>, all: Query<Entity>, players: Query<(), With<AudioPlayer>>, music: Query<(), With<super::LevelMusic>>,
         skies: Query<(), With<SkyLayer>>, squad: Res<Squad>, scenery: Res<super::scenery::Scenery>, player: Res<Player>,
         assets: AssetCounts) {
    // (half a second in: the map's breakables and pickups are listed by then; in the map editor's
    // edit mode, play_editor.rs, there's no body to wait for)
    if world.audited || (player.loaded.is_none() && !player.stand_in) || player.sim_time < AUDIT_AT {
        return;
    }
    world.audited = true;
    if let Some(n) = notice.0.take() {
        status.message = Some((n, MESSAGE_TIME * 2.0));
    }
    if std::env::var("BF_TOOLS_LOG").is_err() {
        return;
    }
    let kept = before.map_or(0, |b| b.0.len());
    let (boxes, breakables) = super::scenery::counts(&scenery);
    println!("test tools: map {} ({}): {} entities ({kept} from before it), {} sounds ({} music), {} sky layers, squad {} ({} NPCs), {boxes} placed boxes, {breakables} breakables",
             session.maps, current.0, all.iter().count(), players.iter().count(), music.iter().count(), skies.iter().count(),
             squad.0.len(), squad.0.iter().filter(|m| m.npc.is_some()).count());
    println!("test tools: map {} assets: {}", session.maps, assets.line());

}

/// The open world menu's text (see `WorldText`).
fn world_text(tools: Res<Tools>, world: Res<World>, lists: Res<Lists>, current: Res<CurrentMap>, mut text: ResMut<WorldText>) {
    let mut s = String::new();
    let title = |i: usize| -> String {
        let e = &lists.levels[i];
        if e.zone == "flat" { e.title.clone() } else { format!("{}  {}", e.zone, e.title) }
    };
    // a window of LIST_LINES round the pick
    let window = |pick: usize, n: usize| {
        let first = pick.saturating_sub(LIST_LINES / 2).min(n.saturating_sub(LIST_LINES));
        first..(first + LIST_LINES).min(n)
    };
    let mark = |on: bool| if on { ">" } else { " " };
    match tools.panel {
        Panel::Sky => {
            s += "SKY   (Y / Esc: close)\n";
            s += &format!("now: {}{}\n", world.sky, world.sky_loading.as_ref().map_or(String::new(), |l| format!("   (loading {})", l.0)));
            let n = lists.skies.len() + 2;
            for k in window(world.sky_pick, n) {
                let label = match k {
                    0 => "none".to_string(),
                    1 => format!("this map's own ({})", current.0),
                    k => title(lists.skies[k - 2]),
                };
                s += &format!("{} {label}\n", mark(k == world.sky_pick));
            }
            if !lists.scanned {
                s += "  (listing the levels' skies...)\n";
            }
            s += "Up / Down pick   click / Enter: show it";
        }
        // a bank's tracks (Shift+Enter on a bank, #398): every wave of its file, the Type of the
        // sound entry that plays it and its length
        Panel::Music if world.tracks.is_some() => {
            let (zone, pick) = world.tracks.clone().unwrap_or_default();
            s += &format!("MUSIC TRACKS of {zone}   (Esc: back)\n");
            s += &format!("now: {}{}\n", world.music, world.music_loading.as_ref().map_or(String::new(), |l| format!("   (loading {})", l.0)));
            match world.track_lists.get(&zone) {
                None => s += "  (listing the tracks...)\n",
                Some(rows) => {
                    for k in window(pick, rows.len()) {
                        let t = &rows[k];
                        s += &format!("{} {k:>2}  {:<26} {:<9} {:>5.1} s\n", mark(k == pick), t.name, kind_word(t.kind), t.file.secs());
                    }
                    // (the picked track's file record, the sounds that play it and its share, on lines of their own)
                    if let Some(t) = rows.get(pick) {
                        let ids: Vec<String> = t.sounds.iter().map(|id| format!("h_{id:08x}")).collect();
                        s += &format!("  {}.xwb: {} bytes at {}\n", t.file.library, t.file.size, t.file.offset);
                        s += &format!("  {}; alone: {}\n", if ids.is_empty() { "no sound entry plays it".to_string() } else { format!("sound {}", ids.join(", ")) }, share_word(t));
                    }
                }
            }
            s += "Up / Down pick   click / Enter: play it alone, looping\n";
            s += "Esc / Backspace: back to the banks";
        }
        Panel::Music => {
            s += "MUSIC   (U / Esc: close)\n";
            s += &format!("now: {}{}\n", world.music, world.music_loading.as_ref().map_or(String::new(), |l| format!("   (loading {})", l.0)));
            // (off, the map's own, the banks, then the bank tracks)
            let first_group = lists.banks.len() + 2;
            let n = first_group + BANK_TRACKS.len();
            for k in window(world.music_pick, n) {
                let label = match k {
                    0 => "off".to_string(),
                    1 if current.0 == "flat" => format!("this map's own (the test map's: {TEST_MUSIC})"),
                    1 => format!("this map's own ({})", current.0),
                    k if k < first_group => title(lists.banks[k - 2].0),
                    k => {
                        let t = &BANK_TRACKS[k - first_group];
                        format!("{}  ({}.xwb, {:.0} s)", t.name, t.banks[0], t.secs)
                    }
                };
                // (the group's heading, where the window reaches its first track)
                if k == first_group {
                    s += "  [bank tracks: sound entry named otherwise]\n";
                }
                s += &format!("{} {label}\n", mark(k == world.music_pick));
            }
            // (the picked bank's tracks on a line of their own: in the list they'd wrap)
            match world.music_pick.checked_sub(2) {
                Some(k) if k < lists.banks.len() => s += &format!("  tracks: {}\n", lists.banks[k].1.join(", ")),
                Some(k) if k >= lists.banks.len() => {
                    // (the bank track's other banks, and what the levels call it)
                    let t = &BANK_TRACKS[k - lists.banks.len()];
                    let banks: Vec<String> = t.banks.iter().map(|b| format!("{b}.xwb")).collect();
                    s += &format!("  in the wave banks {}; its sound is h_{:08x}, not the hash of its name\n", banks.join(", "), t.sound);
                }
                _ => {}
            }
            if !lists.scanned {
                s += "  (listing the music banks...)\n";
            }
            s += "Up / Down pick   click / Enter: play the mix\n";
            s += "Left / Right: previous / next bank\n";
            s += "Shift+Enter / Shift+click: the bank's tracks";
        }
        Panel::Levels => {
            s += "SWITCH LEVEL   (L / Esc: close)\n";
            s += &format!("now: {}\n", current.0);
            let mut group = "";
            for i in window(world.level_pick, lists.levels.len()) {
                let e = &lists.levels[i];
                if e.group != group {
                    group = e.group;
                    s += &format!("  [{group}]\n");
                }
                s += &format!("{} {}\n", mark(i == world.level_pick), title(i));
            }
            s += "Up / Down pick   click / Enter: load it (the tools stay)";
        }
        _ => {}
    }
    if text.0 != s {
        text.0 = s;
    }
}

/// Loads a level for the switch (on a thread), as the front end does, with the test map's
/// weapon data on top (testmap::load_weapon_data). One that doesn't load (no level file): the
/// test map instead, and a note saying so.
///
/// Test hook: BF_TEST_LEVEL_FAIL=<level> makes loading that level panic on its thread (for the
/// fall back in `finish_switch`).
fn load_level(to: &str, deathmatch: bool) -> LevelLoad {
    if std::env::var("BF_TEST_LEVEL_FAIL").is_ok_and(|z| z == to) {
        panic!("BF_TEST_LEVEL_FAIL: {to}");
    }
    let mut game = super::load_game()?;
    let level = super::load_map(&mut game, to);
    if to != "flat" && level.is_none() {
        let mut game = super::load_game()?;
        let level = super::load_map(&mut game, "flat");
        super::testmap::load_weapon_data(&mut game);
        return Ok((LoadedMap { game, level, deathmatch: false, map: "flat".into(), test: true }, Some(format!("{to} didn't load: the test map instead"))));
    }
    super::testmap::load_weapon_data(&mut game);
    Ok((LoadedMap { game, level, deathmatch, map: to.into(), test: true }, None))
}

/// The switch begins (the map is gone: `end_play`): what's left is logged (BF_TOOLS_LOG: only
/// what was there before the map, nothing of its own; screenshots on their way and their
/// observers aside, which go once saved), a loading screen goes up and the next level loads on
/// a thread.
#[allow(clippy::type_complexity)]
fn start_switch(mut commands: Commands, mut switch: ResMut<Switch>, before: Option<Res<Before>>,
                all: Query<(Entity, Has<bevy::render::view::screenshot::Screenshot>, Has<bevy::ecs::observer::Observer>)>,
                players: Query<(), With<AudioPlayer>>) {
    if std::env::var("BF_TOOLS_LOG").is_ok() {
        let kept = before.as_ref().map_or(0, |b| b.0.len());
        let new = |e: &Entity| !before.as_ref().is_some_and(|b| b.0.contains(e));
        let left = all.iter().filter(|(e, shot, obs)| new(e) && !shot && !obs).count();
        let shots = all.iter().filter(|(e, shot, obs)| new(e) && (*shot || *obs)).count();
        println!("test tools: left {}: {} entities remain ({kept} from before it, {left} of its own, {shots} screenshots on their way), {} sounds",
                 switch.from, all.iter().count(), players.iter().count());
    }

    commands.spawn((Camera2d, SwitchPart));
    commands.spawn((
        SwitchPart,
        Text::new(format!("Loading {} ({})...", switch.to.title, switch.to.zone)),
        TextFont { font_size: 22.0, ..default() },
        TextColor(Color::srgba(1.0, 1.0, 1.0, 0.9)),
        Node { position_type: PositionType::Absolute, left: Val::Percent(40.0), top: Val::Percent(48.0), ..default() },
    ));
    switch.rx = Some(load_thread(&switch.to));
}

/// Loads `to` on a thread (`load_level`).
fn load_thread(to: &Entry) -> Mutex<Receiver<LevelLoad>> {
    let (tx, rx) = std::sync::mpsc::channel();
    let (zone, deathmatch) = (to.zone.clone(), to.deathmatch);
    std::thread::spawn(move || {
        let _ = tx.send(load_level(&zone, deathmatch));
    });
    Mutex::new(rx)
}

/// The next level has loaded: play it (`begin_play`, as the front end does). One that failed (an
/// error, or its thread stopped without a result: a panic) is noted and the map it came from is
/// loaded instead, else the test map; only if that fails too does the program stop.
fn finish_switch(mut commands: Commands, mut switch: ResMut<Switch>, lists: Res<Lists>, mut next: ResMut<NextState<AppState>>,
                 mut notice: ResMut<Notice>, mut exit: EventWriter<AppExit>, mut text: Query<&mut Text, With<SwitchPart>>, assets: AssetCounts) {
    let Some(got) = switch.rx.as_ref().and_then(poll) else { return };
    match got {
        Ok((map, note)) => {
            if std::env::var("BF_TOOLS_LOG").is_ok() {
                println!("test tools: {} loaded{}", map.map, note.as_ref().map_or(String::new(), |n| format!(" ({n})")));
                println!("test tools: between the maps, assets: {}", assets.line());
            }
            let failed = (!switch.failed.is_empty()).then(|| format!("{}: {} instead", switch.failed.join("; "), map.map));
            notice.0 = failed.or(note);
            commands.remove_resource::<Switch>();
            begin_play(&mut commands, map);
            next.set(AppState::Playing);
        }
        Err(e) => {
            eprintln!("test tools: couldn't load {}: {e}", switch.to.zone);
            let failed = switch.to.zone.clone();
            // (the reason is in the log above: on screen the note stays short, clear of the radar)
            switch.failed.push(format!("{failed} didn't load"));
            // the map it came from, then the test map, each tried once
            let tried = |z: &str| switch.failed.iter().any(|f| f.starts_with(&format!("{z} ")));
            let back = [switch.from.clone(), "flat".to_string()].into_iter().find(|z| !tried(z));
            let Some(entry) = back.and_then(|z| lists.levels.iter().find(|e| e.zone == z).cloned()) else {
                exit.write(AppExit::error());
                return;
            };
            if std::env::var("BF_TOOLS_LOG").is_ok() {
                println!("test tools: {failed} didn't load: back to {}", entry.zone);
            }
            for mut t in &mut text {
                t.0 = format!("{failed} didn't load. Loading {} ({})...", entry.title, entry.zone);
            }
            switch.rx = Some(load_thread(&entry));
            switch.to = entry;
        }
    }
}


/// The loading screen goes.
fn end_switch(mut commands: Commands, parts: Query<Entity, With<SwitchPart>>) {
    for e in &parts {
        commands.entity(e).despawn();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use bf_viewer::bf::audio::MUSIC_TYPE;
    use bf_viewer::bf::hash::h;

    /// The five bank tracks are the catalogue's (todo/cut-content/catalogue.md 3.3), each with its
    /// banks, first the one it is read from; no name repeats, each plays for a plausible time, and
    /// their sound ids are not the hash of their names (that is what groups them: the sound entries
    /// are named otherwise than the waves).
    #[test]
    fn the_five_bank_tracks_are_the_catalogues() {
        let names: Vec<&str> = BANK_TRACKS.iter().map(|t| t.name).collect();
        assert_eq!(names, ["creepy_estuary_mutant_01", "amb_singe_02", "ambience19-2ch", "caspian_amb_mux_temp", "ambiencehellish-2ch"]);
        assert_eq!(BANK_TRACKS[0].banks, ["e05", "sdm_e40"]);
        assert_eq!(BANK_TRACKS[1].banks, ["e13", "sdm_e13"]);
        assert_eq!(BANK_TRACKS[2].banks, ["m02_a", "m04_a"]);
        assert_eq!(BANK_TRACKS[3].banks, ["m08_a"]);
        assert_eq!(BANK_TRACKS[4].banks, ["sdm_m07"]);
        assert!(BANK_TRACKS.iter().all(|t| (20.0..170.0).contains(&t.secs)));
        // (the 8 bank entries the catalogue counts)
        assert_eq!(BANK_TRACKS.iter().map(|t| t.banks.len()).sum::<usize>(), 8);
        assert!(BANK_TRACKS.iter().all(|t| t.sound != h(t.name)));
        // (three of the entries' names recovered: the sound ids are the hashes of these)
        assert_eq!(BANK_TRACKS[2].sound, h("ambience19_2"));
        assert_eq!(BANK_TRACKS[3].sound, h("caspian_amb_mux"));
        assert_eq!(BANK_TRACKS[4].sound, h("ambiencehellish_2"));
    }

    /// `BF_TEST_MUSIC=<s>,bank:<name>` names a track by its name in any case, a part of it, or its
    /// place in the group; anything else names none. (`unused:` is the same hook's first name.)
    #[test]
    fn a_hook_names_a_bank_track() {
        assert_eq!(find_group_track("creepy_estuary_mutant_01"), Some(0));
        assert_eq!(find_group_track("AMB_SINGE_02"), Some(1));
        assert_eq!(find_group_track(" ambience19-2ch "), Some(2));
        assert_eq!(find_group_track("caspian"), Some(3));
        assert_eq!(find_group_track("hellish"), Some(4));
        // (a part that fits two names: the first; "amb" is in four of them)
        assert_eq!(find_group_track("amb"), Some(1));
        assert_eq!(find_group_track("3"), Some(3));
        assert_eq!(find_group_track("5"), None);
        assert_eq!(find_group_track("nothing"), None);
        assert_eq!(find_group_track(""), None);
        assert_eq!((BANK_PREFIX, UNUSED_PREFIX, TRACK_PREFIX, TRACKS_PREFIX), ("bank:", "unused:", "track:", "tracks:"));
    }

    /// A track of a bank's list for the tests: 22050 Hz stereo Xbox ADPCM, 679536 bytes.
    fn bank_track(library: &str, name: &str, offset: usize, kind: Option<u32>) -> BankTrack {
        BankTrack { name: name.into(), file: bf_viewer::bf::audio::MusicFile { library: library.into(), offset, size: 679536, format: 705609 }, kind, sounds: vec![] }
    }

    /// The label the menu, the messages and the logs use says it's a bank track and where it's read
    /// from; a track of a bank's list is labelled by its own bank.
    #[test]
    fn a_track_is_labelled_with_its_bank() {
        assert_eq!(BANK_TRACKS[0].label(), "bank track creepy_estuary_mutant_01 (e05.xwb)");
        assert_eq!(BANK_TRACKS[3].label(), "bank track caspian_amb_mux_temp (m08_a.xwb)");
        assert_eq!(track_label(&bank_track("m03_b", "caspian_action7", 915456, Some(MUSIC_TYPE))), "track caspian_action7 (m03_b.xwb)");
    }

    /// `BF_TEST_MUSIC=<s>,track:<level>:<which>` names a track of the bank's list by its place from
    /// 0 (a number first: the names have digits in them), its name in any case, or a part of it.
    #[test]
    fn a_hook_names_a_track_of_a_list() {
        let rows: Vec<BankTrack> = ["caspian_action18", "caspian_action7", "caspian_action10", "amb_casp_01", "caspian_action5"].iter().enumerate()
            .map(|(i, n)| bank_track("m03_b", n, 2048 + i * 1000, None)).collect();
        assert_eq!(find_bank_track(&rows, "1"), Some(1));
        assert_eq!(find_bank_track(&rows, "4"), Some(4));
        assert_eq!(find_bank_track(&rows, "5"), None);
        assert_eq!(find_bank_track(&rows, "CASPIAN_ACTION7"), Some(1));
        assert_eq!(find_bank_track(&rows, " amb_casp_01 "), Some(3));
        // (a part that fits several names: the first)
        assert_eq!(find_bank_track(&rows, "action1"), Some(0));
        assert_eq!(find_bank_track(&rows, "casp_01"), Some(3));
        assert_eq!(find_bank_track(&rows, "nothing"), None);
        assert_eq!(find_bank_track(&rows, ""), None);
    }

    /// The lists name a track's role by the Type of the sound entry that plays it, and say what it
    /// plays as picked alone; the name decides only for a track no entry plays.
    #[test]
    fn a_tracks_role_is_its_sound_entrys_type() {
        assert_eq!(kind_word(Some(MUSIC_TYPE)), "music");
        assert_eq!(kind_word(Some(h("ambient"))), "ambience");
        assert_eq!(kind_word(None), "no entry");
        let share = |name: &str, kind: Option<u32>| share_word(&bank_track("x", name, 2048, kind));
        assert_eq!(share("caspian_action7", Some(MUSIC_TYPE)), "music x1.00");
        assert_eq!(share("amb_casp_01", Some(h("ambient"))), "ambience x0.80");
        assert_eq!(share("amb_unknown", None), "ambience x0.80");
        assert_eq!(share("unknown", None), "music x1.00");
        // (the names mislead: ambiencehellish-2ch is a music track, its entry says so)
        assert_eq!(share("ambiencehellish-2ch", Some(MUSIC_TYPE)), "music x1.00");
    }

    /// A track picked alone is mixed as `music_mix` mixes a bank's: by the Type of the sound entry
    /// that plays it (music at x1.00, a bed alone at the bed's share); the name decides only for a
    /// track no entry plays. The five bank tracks, by the Types their banks' entries have: all music
    /// but amb_singe_02, the bed.
    #[test]
    fn a_track_alone_is_music_or_ambience_by_its_sound_entry() {
        let alone = |name: &str, kind: Option<u32>| super::super::music_mix(vec![MixTrack { name: name.to_string(), wav: vec![0u8; 4], kind }]);
        let kinds = [Some(MUSIC_TYPE), Some(h("ambient")), Some(MUSIC_TYPE), Some(MUSIC_TYPE), Some(MUSIC_TYPE)];
        let mixes: Vec<&str> = BANK_TRACKS.iter().zip(kinds).map(|(t, k)| alone(t.name, k)[0].2).collect();
        assert_eq!(mixes, ["music", "ambience", "music", "music", "music"]);
        assert_eq!(alone("creepy_estuary_mutant_01", Some(MUSIC_TYPE))[0].3, 1.0);
        assert_eq!(alone("amb_singe_02", Some(h("ambient")))[0].3, 0.8);
        // (what the name rule gave them before: two of the five were told wrong)
        assert_eq!(alone("ambience19-2ch", None)[0].2, "ambience");
        assert_eq!(alone("ambiencehellish-2ch", None)[0].2, "ambience");
    }
}
