//! The campaign's missions (common/campaign-bf.xmb, #221): its squad, the extra marines that
//! stand in for missing members, and each mission's zones and objectives. bf_play's use of it is
//! src/bin/play_flags.rs and viewer/docs/campaign.md. (The same file's level list for the front
//! end is `character::CampaignLevel`.)
//!
//!   <campaign gameflags-schema="gameflags" h_1a468238="default-gameflags" h_e9d108ea="Brute Force">
//!     <squad-char char-id="tex" h_1b561f3f=..>               x4 (FUN_001a4640)
//!     <h_1f45a087 archetype-id="h_ef8787e4" h_1f07654b=.. h_1b561f3f=..>   x3, the marine types
//!     <level h_046391fd="h_e26b9b3e">                        a mission, by its gameflag id (m00)
//!       <h_1b8535f0 ..>                                      its front-end entry
//!       <zone file-name="tutorial" h_ed54d255="brutus hawk flint">  (FUN_001a3d40) <pre-movies>
//!       <primary-objectives PATH="h_0630ae70 h_e26b9b3e h_eaf792f4" display-id=.. h_f71c3e99="5000"/>
//!       <secondary-objectives ..>                            (FUN_0018d400)
//!
//! The root's parser is FUN_001a4560, a level's FUN_001a4000.

use super::bxml::{Element, Value};
use super::hash::h;

/// The root's h_1a468238: the default game flags' file, without its .xmb (FUN_001a4560).
const H_FLAGS_FILE: u32 = 0x1A46_8238;
/// The extra squad members' element, h_1f45a087 (FUN_001a4640: archetype-id, h_1f07654b their
/// display name, h_1b561f3f), the campaign's +0x80..+0x84 list of 12-byte records.
const H_EXTRA: u32 = 0x1F45_A087;
/// A level's mission id, h_046391fd (h_e26b9b3e "m00": the missions' gameflag folder name).
const H_MISSION_ID: u32 = 0x0463_91FD;
/// A zone's h_ed54d255: the squad members it doesn't have (the zone's +0x40..+0x44 list, read by
/// the session's +0xec and +0xf0).
const H_UNAVAILABLE: u32 = 0xED54_D255;
/// An objective's cash, h_f71c3e99 (FUN_0018d400): the end screen's per-objective award.
const H_CASH: u32 = 0xF71C_3E99;

/// One of a mission's zones (a level archive).
#[derive(Clone, Debug, Default)]
pub struct Zone {
    /// file-name, lower case: the level archive (data/<file>.tgz)
    pub file: String,
    /// h_ed54d255: the squad members (squad-char ids) this zone doesn't have
    pub unavailable: Vec<u32>,
}

/// A mission objective (`<primary-objectives>` / `<secondary-objectives>`, FUN_0018d400).
#[derive(Clone, Debug, Default)]
pub struct Objective {
    /// primary (else secondary)
    pub primary: bool,
    /// PATH: its folder in the game flags (missions/<mission>/<objective>), holding `enabled`
    /// (bool) and `state` (int: 0 open, 1 completed, -1 failed)
    pub path: Vec<u32>,
    /// display-id: its line's string id
    pub text: u32,
    /// h_f71c3e99: its cash on the end screen (5000 and 1000 in the tutorial)
    pub cash: i64,
}

/// A mission (`<level>`).
#[derive(Clone, Debug, Default)]
pub struct Mission {
    /// h_046391fd: its id, the name of its gameflag folder (h_e26b9b3e "m00")
    pub id: u32,
    pub zones: Vec<Zone>,
    /// its objectives, primary and secondary, in the file's order
    pub objectives: Vec<Objective>,
}

/// The campaign file (see the module notes).
#[derive(Clone, Debug, Default)]
pub struct Campaign {
    /// `<squad-char char-id>`: the squad's four types, in the file's order (tex, brutus, flint,
    /// hawk); the campaign's +0x5c list of 8-byte records
    pub squad: Vec<u32>,
    /// `<h_1f45a087 archetype-id>`: the types that fill a zone's missing members' places (the
    /// three Confed marine types), in order
    pub extras: Vec<u32>,
    pub missions: Vec<Mission>,
    /// h_1a468238: the default game flags' file ("default-gameflags")
    pub flags_file: String,
}

/// A list attribute's names: the attribute repeated, one value per entry (as the BXML stores a
/// list: "brutus hawk flint" is three), or a List value; strings hashed as names.
fn names(e: &Element, k: u32) -> Vec<u32> {
    e.attrs.iter().filter(|(n, _)| *n == k).flat_map(|(_, v)| match v {
        Value::List(l) => l.iter().filter_map(|x| x.as_hash()).collect::<Vec<_>>(),
        v => v.as_hash().into_iter().collect(),
    }).filter(|&x| x != 0).collect()
}

impl Campaign {
    /// The campaign of campaign-bf.xmb's root.
    pub fn from_root(root: &Element) -> Campaign {
        let id = |e: &Element, k: u32| e.attr(k).and_then(|v| v.as_hash()).unwrap_or(0);
        Campaign {
            squad: root.children_named(h("squad-char")).map(|c| id(c, h("char-id"))).filter(|&x| x != 0).collect(),
            extras: root.children_named(H_EXTRA).map(|c| id(c, h("archetype-id"))).filter(|&x| x != 0).collect(),
            missions: root.children_named(h("level")).map(|l| Mission {
                id: id(l, H_MISSION_ID),
                zones: l.children_named(h("zone")).map(|z| Zone {
                    file: z.attr(h("file-name")).and_then(|v| v.as_str()).unwrap_or("").to_lowercase(),
                    unavailable: names(z, H_UNAVAILABLE),
                }).collect(),
                objectives: l.children.iter().filter(|c| c.name == h("primary-objectives") || c.name == h("secondary-objectives"))
                    .map(|o| Objective {
                        primary: o.name == h("primary-objectives"),
                        path: names(o, h("PATH")),
                        text: id(o, h("display-id")),
                        cash: o.attr(H_CASH).and_then(|v| v.as_i64()).unwrap_or(0),
                    }).collect(),
            }).collect(),
            flags_file: root.attr(H_FLAGS_FILE).and_then(|v| v.as_str()).map(|s| s.trim().to_string()).filter(|s| !s.is_empty())
                .unwrap_or_else(|| "default-gameflags".into()),
        }
    }

    /// The mission one of whose zones is the level archive `map` (any case), and that zone.
    pub fn mission_of(&self, map: &str) -> Option<(&Mission, &Zone)> {
        let map = map.to_lowercase();
        self.missions.iter().find_map(|m| m.zones.iter().find(|z| z.file == map).map(|z| (m, z)))
    }
}
