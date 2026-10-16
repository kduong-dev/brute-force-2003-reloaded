//! The game flags (#221): the campaign's state as a tree of typed records, read and written by
//! path. The session keeps one tree per save slot (its vtable +0x210, 0x78d60); a new campaign's
//! comes from `common/default-gameflags.xmb` (campaign-bf.xmb's root names the file,
//! h_1a468238 "default-gameflags", with its schema `gameflags`). bf_play's use of it is
//! src/bin/play_flags.rs and viewer/docs/campaign.md.
//!
//!   <gameflags-file>
//!     <campaign> <shadoon-look default="-4" value="-4"/> <save-game> <character-1> ...
//!     <tex> <h_03761692 default="false" value="false"/> </tex>   (and hawk, brutus, flint, the
//!                                                                 three Confed marine types)
//!     <h_0630ae70> <h_e26b9b3e> <h_eaf792f4> <enabled default="true" value="true"/>
//!                                            <state default="0" value="0"/>   (missions/m00/..)
//!
//! A folder is an element with children; a record is one with `value` and `default`, its type
//! the schema's (float-gameflag, int-gameflag, boolean-gameflag: the BXML values come typed). The
//! game's records are 16 bytes, {name hash, value +4, default +8, type +0xc: 1 float, 2 int,
//! 3 bool}, in a vector per folder (+0x14..+0x18), found name by name (FUN_000782c0,
//! FUN_00077da0); read as a float by FUN_00077e80, as an int by FUN_00077e00, as a bool by
//! FUN_00077e40, and written by FUN_00077eb0.

use super::bxml::{Element, Value};
use super::hash::h;

/// A record's value (or default) as its type stores it (the record's +0xc: 1 float, 2 int,
/// 3 bool).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum FlagValue {
    Float(f32),
    Int(i32),
    Bool(bool),
}

/// FUN_00077eb0's test for a bool: a float under this (absolute) writes false.
const BOOL_EPSILON: f32 = 1e-6;

impl FlagValue {
    /// As a float (FUN_00077e80): an int as it is, a bool 1 or 0.
    pub fn as_f32(self) -> f32 {
        match self {
            FlagValue::Float(v) => v,
            FlagValue::Int(v) => v as f32,
            FlagValue::Bool(b) => if b { 1.0 } else { 0.0 },
        }
    }

    /// As an int (FUN_00077e00): a float truncated, a bool 1 or 0.
    pub fn as_int(self) -> i32 {
        match self {
            FlagValue::Float(v) => v as i32,
            FlagValue::Int(v) => v,
            FlagValue::Bool(b) => b as i32,
        }
    }

    /// As a bool (FUN_00077e40): a float truncated to an int, then not 0; an int not 0.
    pub fn as_bool(self) -> bool {
        match self {
            FlagValue::Float(v) => v as i32 != 0,
            FlagValue::Int(v) => v != 0,
            FlagValue::Bool(b) => b,
        }
    }

    /// `v` as this record's type stores it (FUN_00077eb0): a float as it is, an int truncated,
    /// a bool true unless |v| is under 1e-6.
    pub fn with(self, v: f32) -> FlagValue {
        match self {
            FlagValue::Float(_) => FlagValue::Float(v),
            FlagValue::Int(_) => FlagValue::Int(v as i32),
            FlagValue::Bool(_) => FlagValue::Bool(v.abs() >= BOOL_EPSILON),
        }
    }

    /// The data's value as a record: a bool, an int or a float as the schema typed it (a string,
    /// as the decoder leaves an untyped one, read as a number or true / false).
    fn from_value(v: &Value) -> Option<FlagValue> {
        match v {
            Value::Bool(b) => Some(FlagValue::Bool(*b)),
            Value::Int(i) => Some(FlagValue::Int(*i as i32)),
            Value::Float(f) => Some(FlagValue::Float(*f as f32)),
            Value::Str(s) => {
                let s = s.trim();
                match s {
                    "true" => Some(FlagValue::Bool(true)),
                    "false" => Some(FlagValue::Bool(false)),
                    _ => s.parse::<i32>().map(FlagValue::Int).ok().or_else(|| s.parse::<f32>().ok().map(FlagValue::Float)),
                }
            }
            _ => None,
        }
    }
}

impl std::fmt::Display for FlagValue {
    /// For logs: a bool as true / false, an int as it is, a float with what it needs.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FlagValue::Float(v) => write!(f, "{v}"),
            FlagValue::Int(v) => write!(f, "{v}"),
            FlagValue::Bool(b) => write!(f, "{b}"),
        }
    }
}

/// A node of the tree: a folder of named nodes, in the file's order, or a record.
#[derive(Clone, Debug)]
pub enum FlagNode {
    Folder(Vec<(u32, FlagNode)>),
    Record { value: FlagValue, default: FlagValue },
}

/// A flag tree (see the module notes).
#[derive(Clone, Debug, Default)]
pub struct GameFlags {
    root: Vec<(u32, FlagNode)>,
}

/// A path's node among `nodes` (FUN_000782c0 / FUN_00077da0: name by name from the top).
fn find<'a>(nodes: &'a [(u32, FlagNode)], path: &[u32]) -> Option<&'a FlagNode> {
    let (first, rest) = path.split_first()?;
    let node = &nodes.iter().find(|(n, _)| n == first)?.1;
    match (rest.is_empty(), node) {
        (true, n) => Some(n),
        (false, FlagNode::Folder(c)) => find(c, rest),
        (false, FlagNode::Record { .. }) => None,
    }
}

/// `find`, for writing.
fn find_mut<'a>(nodes: &'a mut [(u32, FlagNode)], path: &[u32]) -> Option<&'a mut FlagNode> {
    let (first, rest) = path.split_first()?;
    let node = &mut nodes.iter_mut().find(|(n, _)| n == first)?.1;
    match (rest.is_empty(), node) {
        (true, n) => Some(n),
        (false, FlagNode::Folder(c)) => find_mut(c, rest),
        (false, FlagNode::Record { .. }) => None,
    }
}

/// An element as a node: a record when it has a `value` or a `default`, else a folder of its
/// children.
fn node(e: &Element) -> FlagNode {
    let value = e.attr(h("value")).and_then(FlagValue::from_value);
    let default = e.attr(h("default")).and_then(FlagValue::from_value);
    match (value, default) {
        (Some(v), Some(d)) => FlagNode::Record { value: v, default: d },
        (Some(v), None) | (None, Some(v)) => FlagNode::Record { value: v, default: v },
        (None, None) => FlagNode::Folder(e.children.iter().map(|c| (c.name, node(c))).collect()),
    }
}

impl GameFlags {
    /// The tree of a flags file's root (`<gameflags-file>`).
    pub fn from_root(root: &Element) -> GameFlags {
        GameFlags { root: root.children.iter().map(|c| (c.name, node(c))).collect() }
    }

    /// Whether the tree has nothing (no flags file was read).
    pub fn is_empty(&self) -> bool {
        self.root.is_empty()
    }

    /// The record at `path`: its value and default; None when the path isn't a record.
    pub fn get(&self, path: &[u32]) -> Option<(FlagValue, FlagValue)> {
        match find(&self.root, path)? {
            FlagNode::Record { value, default } => Some((*value, *default)),
            FlagNode::Folder(_) => None,
        }
    }

    /// The value at `path`, None without such a record.
    pub fn value(&self, path: &[u32]) -> Option<FlagValue> {
        self.get(path).map(|(v, _)| v)
    }

    /// Writes `v` to the record at `path` as its type stores it (FUN_00077eb0): its value before
    /// and after, None without such a record (the game writes nothing then).
    pub fn write(&mut self, path: &[u32], v: f32) -> Option<(FlagValue, FlagValue)> {
        match find_mut(&mut self.root, path)? {
            FlagNode::Record { value, .. } => {
                let old = *value;
                *value = old.with(v);
                Some((old, *value))
            }
            FlagNode::Folder(_) => None,
        }
    }

    /// Writes the whole number `v` to the record at `path` as FUN_00081550 and FUN_00084230 store
    /// one (a name hash in an int record, the save-game slots' archetype): an int as it is, a
    /// float as the unsigned number, a bool `v` not 0. Its value before and after, None without
    /// such a record.
    pub fn write_int(&mut self, path: &[u32], v: i32) -> Option<(FlagValue, FlagValue)> {
        match find_mut(&mut self.root, path)? {
            FlagNode::Record { value, .. } => {
                let old = *value;
                *value = match old {
                    FlagValue::Int(_) => FlagValue::Int(v),
                    FlagValue::Float(_) => FlagValue::Float(v as u32 as f32),
                    FlagValue::Bool(_) => FlagValue::Bool(v != 0),
                };
                Some((old, *value))
            }
            FlagNode::Folder(_) => None,
        }
    }

    /// Sets the record at `path` back to its default (the record's +4 = its +8, as FUN_00082040
    /// and FUN_00083fb0 do): its value before and after.
    pub fn reset(&mut self, path: &[u32]) -> Option<(FlagValue, FlagValue)> {
        match find_mut(&mut self.root, path)? {
            FlagNode::Record { value, default } => {
                let old = *value;
                *value = *default;
                Some((old, *value))
            }
            FlagNode::Folder(_) => None,
        }
    }

    /// Every record under `path` (the whole tree for an empty one) with its full path, in the
    /// file's order.
    pub fn records(&self, path: &[u32]) -> Vec<(Vec<u32>, FlagValue, FlagValue)> {
        fn walk(nodes: &[(u32, FlagNode)], at: &mut Vec<u32>, out: &mut Vec<(Vec<u32>, FlagValue, FlagValue)>) {
            for (n, node) in nodes {
                at.push(*n);
                match node {
                    FlagNode::Record { value, default } => out.push((at.clone(), *value, *default)),
                    FlagNode::Folder(c) => walk(c, at, out),
                }
                at.pop();
            }
        }
        let mut out = vec![];
        let mut at = path.to_vec();
        match path.is_empty() {
            true => walk(&self.root, &mut at, &mut out),
            false => match find(&self.root, path) {
                Some(FlagNode::Folder(c)) => walk(c, &mut at, &mut out),
                Some(FlagNode::Record { value, default }) => out.push((at, *value, *default)),
                None => {}
            },
        }
        out
    }
}
