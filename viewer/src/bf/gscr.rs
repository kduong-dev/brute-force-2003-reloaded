//! A level's cutscene scripts, scripts/<name>.gscr in its archive (#222; viewer/docs/cutscenes.md).
//!
//! A script is Lua 4 source the game runs through its embedded Lua (FUN_0026c960: the globals
//! `duration`, `entities` and `events`, read into a record by FUN_0026db20 for an entity and
//! FUN_0026cb40 for an event): Digital Anvil's THN cutscene format, as Freelancer's. The files are
//! data, a few assignments of tables, so this reads them without a Lua VM:
//!
//!   duration = 26.254000;
//!   entities = { {entity_name = "camera_opening", type = CAMERA, template_name = "NULL",
//!                 spatialprops = {pos = {x, y, z}, orient = { {..}, {..}, {..} }},
//!                 cameraprops = {fovh = 36.87, hvaspect = 1.33, nearplane = 0.005, farplane = 500}}, ... };
//!   events = { {0.0, USER_EVENT, {"trigger_letterbox14"}, {user_event_data = "103, 1, 11"}},
//!              {0.0, START_PATH_ANIMATION, {"camera_show_wallgate_terrorist", "path8"},
//!               {duration = 6.144, start_percent = 0, stop_percent = 1, offset = {0, 0, 0},
//!                up = Y_AXIS, front = NEG_Z_AXIS, flags = POSITION}}, ... };
//!
//! The capitalised names are globals the game registers before it runs one (FUN_0026b7d0, from
//! the tables at 0x3bb1dc / 0x3bb24c / 0x3bb2c4 and its own constants): `constant` has them. All
//! 179 scripts of the game's levels read (bf_level's BF_SCRIPT_CHECK).

use bevy::math::{Mat3, Vec3};

use super::hash::h;

/// A value of a script's Lua: a number, a string, a global's name (CAMERA, Y_AXIS ...), a sum
/// of values (`POSITION + ORIENTATION`), or a table.
#[derive(Clone, Debug, PartialEq)]
pub enum Lua {
    Num(f64),
    Str(String),
    Name(String),
    Sum(Vec<Lua>),
    Table(Table),
}

/// A Lua table as the scripts write them: positional items and `key = value` fields, in order.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Table {
    pub items: Vec<Lua>,
    pub fields: Vec<(String, Lua)>,
}

impl Table {
    /// Field `key`'s value.
    pub fn get(&self, key: &str) -> Option<&Lua> {
        self.fields.iter().find(|(k, _)| k == key).map(|(_, v)| v)
    }

    /// Field `key` as a number (a global's value for a name).
    pub fn num(&self, key: &str) -> Option<f64> {
        self.get(key).and_then(Lua::num)
    }

    /// Field `key` as a string.
    pub fn str(&self, key: &str) -> Option<&str> {
        self.get(key).and_then(Lua::str)
    }

    /// Field `key` as a table.
    pub fn table(&self, key: &str) -> Option<&Table> {
        self.get(key).and_then(Lua::table)
    }
}

impl Lua {
    /// As a number: a number, a global's value (`constant`), or a sum of them (the flags'
    /// `POSITION + ORIENTATION`, which Lua adds).
    pub fn num(&self) -> Option<f64> {
        match self {
            Lua::Num(n) => Some(*n),
            Lua::Name(n) => constant(n),
            Lua::Sum(v) => v.iter().map(Lua::num).sum(),
            _ => None,
        }
    }

    /// As a string.
    pub fn str(&self) -> Option<&str> {
        match self {
            Lua::Str(s) => Some(s),
            _ => None,
        }
    }

    /// As a table.
    pub fn table(&self) -> Option<&Table> {
        match self {
            Lua::Table(t) => Some(t),
            _ => None,
        }
    }

    /// A table of three numbers as a vector (`pos`, `offset`).
    pub fn vec3(&self) -> Option<Vec3> {
        let t = self.table()?;
        let n: Vec<f32> = t.items.iter().filter_map(|v| v.num()).map(|v| v as f32).collect();
        (n.len() >= 3).then(|| Vec3::new(n[0], n[1], n[2]))
    }
}

/// The value of a global the game registers for its scripts (FUN_0026b7d0), None for one it
/// doesn't: the entity types (names 0x3bb1dc, values 0x3bb214), the event types (names 0x3bb24c,
/// values 0x3bb288), the axes (0x3bb2c4, by place) and the path and attachment flags (pushed one
/// by one: USE_SCRIPT_DURATION 1, POSITION 2, PATH_POSITION 2, ORIENTATION 4, LOOK_AT 8,
/// ENTITY_RELATIVE 32, PARENT_CHILD 64).
pub fn constant(name: &str) -> Option<f64> {
    let v = match name {
        "UNKNOWN_ENTITY" => 0, "COMPOUND" => 1, "DEFORMABLE" => 2, "CAMERA" => 3, "MONITOR" => 4, "LIGHT" => 5, "SOUND" => 6,
        "MARKER" => 7, "HARDPOINT" => 8, "SCENE" => 9, "SUB_SCENE" => 10, "MOTION_PATH" => 11, "DELETED" => 12, "PSYS" => 13,
        "UNDEFINED_EVENT" => 0, "USER_EVENT" => 1, "SET_CAMERA" => 2, "START_SOUND" => 3, "START_LIGHT_PROP_ANIM" => 4,
        "START_CAMERA_PROP_ANIM" => 5, "START_PATH_ANIMATION" => 6, "START_SPATIAL_PROP_ANIM" => 7, "ATTACH_ENTITY" => 8,
        "CONNECT_HARDPOINTS" => 9, "START_MOTION" => 10, "START_IK" => 11, "START_SUB_SCENE" => 12, "START_PSYS" => 13,
        "START_PSYS_PROP_ANIM" => 14,
        "X_AXIS" => 0, "Y_AXIS" => 1, "Z_AXIS" => 2, "NEG_X_AXIS" => 3, "NEG_Y_AXIS" => 4, "NEG_Z_AXIS" => 5,
        "USE_SCRIPT_DURATION" => 1, "POSITION" => 2, "PATH_POSITION" => 2, "ORIENTATION" => 4, "LOOK_AT" => 8,
        "ENTITY_RELATIVE" => 32, "PARENT_CHILD" => 64,
        _ => return None,
    };
    Some(v as f64)
}

/// The lexer's tokens.
#[derive(Clone, Debug, PartialEq)]
enum Tok {
    Num(f64),
    Str(String),
    Name(String),
    Sym(char),
}

/// Splits a script into tokens: numbers, strings ('..' or ".." with Lua's escapes), names, the
/// symbols { } , ; = + -; `--` comments to the line's end are skipped.
fn tokens(src: &str) -> Result<Vec<Tok>, String> {
    let b = src.as_bytes();
    let mut out = vec![];
    let mut i = 0;
    while i < b.len() {
        let c = b[i] as char;
        if c.is_whitespace() {
            i += 1;
        } else if c == '-' && b.get(i + 1) == Some(&b'-') {
            while i < b.len() && b[i] != b'\n' {
                i += 1;
            }
        } else if c == '"' || c == '\'' {
            let mut s = String::new();
            i += 1;
            while i < b.len() && b[i] as char != c {
                if b[i] == b'\\' && i + 1 < b.len() {
                    i += 1;
                    s.push(match b[i] { b'n' => '\n', b't' => '\t', x => x as char });
                } else {
                    s.push(b[i] as char);
                }
                i += 1;
            }
            if i >= b.len() {
                return Err("a string isn't closed".into());
            }
            i += 1;
            out.push(Tok::Str(s));
        } else if c.is_ascii_digit() || (c == '.' && b.get(i + 1).is_some_and(|d| d.is_ascii_digit())) {
            let start = i;
            while i < b.len() && (b[i].is_ascii_digit() || b[i] == b'.'
                || ((b[i] == b'e' || b[i] == b'E') && i > start)
                || ((b[i] == b'-' || b[i] == b'+') && (b[i - 1] == b'e' || b[i - 1] == b'E'))) {
                i += 1;
            }
            let text = &src[start..i];
            out.push(Tok::Num(text.parse().map_err(|_| format!("bad number {text}"))?));
        } else if c.is_ascii_alphabetic() || c == '_' {
            let start = i;
            while i < b.len() && (b[i].is_ascii_alphanumeric() || b[i] == b'_') {
                i += 1;
            }
            out.push(Tok::Name(src[start..i].to_string()));
        } else if "{},;=+-".contains(c) {
            out.push(Tok::Sym(c));
            i += 1;
        } else {
            return Err(format!("unexpected '{c}'"));
        }
    }
    Ok(out)
}

/// A recursive-descent reader over the tokens (see `parse_lua`).
struct Reader {
    toks: Vec<Tok>,
    at: usize,
}

impl Reader {
    fn peek(&self) -> Option<&Tok> {
        self.toks.get(self.at)
    }

    fn next(&mut self) -> Option<Tok> {
        let t = self.toks.get(self.at).cloned();
        self.at += 1;
        t
    }

    fn eat(&mut self, c: char) -> bool {
        if self.peek() == Some(&Tok::Sym(c)) {
            self.at += 1;
            true
        } else {
            false
        }
    }

    /// expr := term { '+' term }
    fn expr(&mut self) -> Result<Lua, String> {
        let first = self.term()?;
        if self.peek() != Some(&Tok::Sym('+')) {
            return Ok(first);
        }
        let mut sum = vec![first];
        while self.eat('+') {
            sum.push(self.term()?);
        }
        Ok(Lua::Sum(sum))
    }

    /// term := number | '-' number | string | name | table
    fn term(&mut self) -> Result<Lua, String> {
        match self.next() {
            Some(Tok::Num(n)) => Ok(Lua::Num(n)),
            Some(Tok::Sym('-')) => match self.next() {
                Some(Tok::Num(n)) => Ok(Lua::Num(-n)),
                t => Err(format!("a number after '-', found {t:?}")),
            },
            Some(Tok::Str(s)) => Ok(Lua::Str(s)),
            Some(Tok::Name(n)) => Ok(Lua::Name(n)),
            Some(Tok::Sym('{')) => self.table(),
            t => Err(format!("a value, found {t:?}")),
        }
    }

    /// table := '{' [ field { (',' | ';') field } [',' | ';'] ] '}', a field `name = expr` or an
    /// expr (the '{' already read).
    fn table(&mut self) -> Result<Lua, String> {
        let mut t = Table::default();
        loop {
            if self.eat('}') {
                return Ok(Lua::Table(t));
            }
            let keyed = matches!((self.toks.get(self.at), self.toks.get(self.at + 1)), (Some(Tok::Name(_)), Some(Tok::Sym('='))));
            if keyed {
                let Some(Tok::Name(k)) = self.next() else { unreachable!() };
                self.at += 1;
                let v = self.expr()?;
                t.fields.push((k, v));
            } else {
                let v = self.expr()?;
                t.items.push(v);
            }
            if !self.eat(',') && !self.eat(';') {
                return if self.eat('}') { Ok(Lua::Table(t)) } else { Err(format!("',' or '}}' in a table, found {:?}", self.peek())) };
            }
        }
    }
}

/// A script's globals as it assigns them (`name = value;`), in order. The scripts are nothing but
/// such assignments (FUN_0026c960 reads `duration`, `entities` and `events` back from Lua's
/// globals after running one).
pub fn parse_lua(src: &str) -> Result<Vec<(String, Lua)>, String> {
    let mut r = Reader { toks: tokens(src)?, at: 0 };
    let mut out = vec![];
    while r.peek().is_some() {
        if r.eat(';') {
            continue;
        }
        let name = match r.next() {
            Some(Tok::Name(n)) => n,
            t => return Err(format!("a global's name, found {t:?} (token {})", r.at)),
        };
        if !r.eat('=') {
            return Err(format!("'=' after {name}"));
        }
        out.push((name, r.expr()?));
    }
    Ok(out)
}

/// A script entity's type (the entity record's +0, FUN_0026db20; the names and values of
/// 0x3bb1dc / 0x3bb214). The tutorial's: SCENE, MARKER (a level trigger with template "trig",
/// the script itself with "script", a world point with "wpoint"), CAMERA (a level camera-object
/// of that name), COMPOUND (a character or a spawn copy of that name) and MOTION_PATH (a level
/// path-object of that name): FUN_00175090 binds them so.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EntityKind {
    Unknown,
    Compound,
    Deformable,
    Camera,
    Monitor,
    Light,
    Sound,
    Marker,
    Hardpoint,
    Scene,
    SubScene,
    MotionPath,
    Deleted,
    Psys,
}

impl EntityKind {
    /// The type of the game's value (0x3bb214's order).
    fn of(v: i64) -> EntityKind {
        use EntityKind::*;
        [Unknown, Compound, Deformable, Camera, Monitor, Light, Sound, Marker, Hardpoint, Scene, SubScene, MotionPath, Deleted, Psys]
            .get(v as usize).copied().unwrap_or(Unknown)
    }
}

/// A camera entity's `cameraprops` (FUN_0026db20). The game's cutscene cameras are the level's
/// camera-objects of the same names, whose own FOV is what shows (`level::LevelCamera`); these
/// are the script's copies (fovh 36.87 on every one of the tutorial's), a fallback.
#[derive(Clone, Copy, Debug)]
pub struct CameraProps {
    pub fovh: f32,
    pub hvaspect: f32,
    pub near: f32,
    pub far: f32,
}

/// A script entity (`entities`): its name and hash, type, template ("trig", "script", "wpoint",
/// "NULL" ...), its place (`spatialprops`: `pos`, and `orient`, whose rows are the entity's x, y
/// and z axes: the level transform's columns, as camera_show_wallgate_terrorist's shows), and
/// its camera's props.
#[derive(Clone, Debug)]
pub struct Entity {
    pub name: String,
    pub hash: u32,
    pub kind: EntityKind,
    pub template: String,
    pub pos: Vec3,
    pub orient: Mat3,
    pub camera: Option<CameraProps>,
}

/// An axis of a path animation's `up` / `front` (the axes' globals, X_AXIS 0 .. NEG_Z_AXIS 5;
/// defaults Y_AXIS up and NEG_Z_AXIS front, FUN_00275180).
pub fn axis(v: i64) -> Vec3 {
    match v {
        0 => Vec3::X,
        1 => Vec3::Y,
        2 => Vec3::Z,
        3 => Vec3::NEG_X,
        4 => Vec3::NEG_Y,
        _ => Vec3::NEG_Z,
    }
}

/// A START_PATH_ANIMATION's parameters (the event's record, FUN_0026cb40 case 6; the running
/// event's FUN_00275180): `duration` (s), `start_percent` / `stop_percent` (the path's stretch
/// it covers, 0..1), `offset` (added to the path's point in its frame), `up` / `front` (axes:
/// for LOOK_AT), `flags` (POSITION 2, ORIENTATION 4, LOOK_AT 8; USE_SCRIPT_DURATION 1: it runs
/// to the script's end) and `event_flags` (not 0: the path is followed by its keys' own times,
/// not by `start_percent`..`stop_percent`: the path controller's +0x8c, FUN_001765a0).
#[derive(Clone, Debug)]
pub struct PathAnimation {
    pub duration: f32,
    pub start: f32,
    pub stop: f32,
    pub offset: Vec3,
    pub up: i64,
    pub front: i64,
    pub flags: u32,
    pub event_flags: u32,
}

/// The path animation flags (the globals FUN_0026b7d0 registers; the running event tests its
/// +0x28 with them, 0x275290).
pub mod flags {
    /// USE_SCRIPT_DURATION: runs until the script ends (0x275250: no end time)
    pub const USE_SCRIPT_DURATION: u32 = 1;
    /// POSITION: the target goes to the path's point
    pub const POSITION: u32 = 2;
    /// ORIENTATION: and turns as the path's keys do
    pub const ORIENTATION: u32 = 4;
    /// LOOK_AT: with ORIENTATION, it looks along the path instead (FUN_0028b000 from `up` and
    /// `front`)
    pub const LOOK_AT: u32 = 8;
}

/// A USER_EVENT's `user_event_data` = "<code>, <n>, args" (FUN_001753f0: sscanf "%d, %d", then
/// the code's own fields; `n` picks how many its variant reads). The arguments are kept as the
/// strings between the commas; the code's reader is the player's (play_script.rs).
#[derive(Clone, Debug)]
pub struct UserEvent {
    pub code: i32,
    pub variant: i32,
    pub args: Vec<String>,
    pub raw: String,
}

impl UserEvent {
    /// The event's data string read (see `UserEvent`).
    pub fn parse(raw: &str) -> UserEvent {
        let mut parts = raw.split(',').map(|s| s.trim().to_string());
        let code = parts.next().and_then(|s| s.parse().ok()).unwrap_or(0);
        let variant = parts.next().and_then(|s| s.parse().ok()).unwrap_or(0);
        UserEvent { code, variant, args: parts.collect(), raw: raw.to_string() }
    }

    /// Argument `i` as a whole number (0 when it isn't one).
    pub fn int(&self, i: usize) -> i64 {
        self.args.get(i).and_then(|s| s.split(':').next()).and_then(|s| s.trim().parse::<f64>().ok()).map_or(0, |v| v as i64)
    }
}

/// What an event does (its type, the event record's +0, FUN_0026cb40).
#[derive(Clone, Debug)]
pub enum EventKind {
    /// USER_EVENT (1): the game's own codes, see `UserEvent`
    User(UserEvent),
    /// START_PATH_ANIMATION (6): its first entity follows its second, a path
    Path(PathAnimation),
    /// any other type, by its global's value (ATTACH_ENTITY 8 ...: #232's), with its table
    Other(i64, Table),
}

/// A script event (`events`): its time (s from the script's start), the entities it names, and
/// what it does.
#[derive(Clone, Debug)]
pub struct Event {
    pub time: f32,
    pub entities: Vec<String>,
    pub kind: EventKind,
}

/// A cutscene script: its name (the file's, which the script-triggers' `script-name` hashes:
/// FUN_00174180 registers each by it), `duration` (s), entities and events (in time order, the
/// file's order kept for equal times).
#[derive(Clone, Debug)]
pub struct Script {
    pub name: String,
    /// The level whose archive it came from (`Game::load_level` sets it; empty from `parse`):
    /// the demo plays only the current map's (bf_play loads e01's archive too, for its audio).
    pub level: String,
    pub duration: f32,
    pub entities: Vec<Entity>,
    pub events: Vec<Event>,
}

impl Script {
    /// Reads a script (`name` is the file's name without .gscr).
    pub fn parse(name: &str, src: &str) -> Result<Script, String> {
        let globals = parse_lua(src)?;
        let get = |k: &str| globals.iter().rev().find(|(n, _)| n == k).map(|(_, v)| v);
        let duration = get("duration").and_then(Lua::num).ok_or("no duration")? as f32;
        let entities = get("entities").and_then(Lua::table).ok_or("no entities")?.items.iter()
            .filter_map(Lua::table).map(entity).collect::<Result<Vec<_>, _>>()?;
        let mut events = get("events").and_then(Lua::table).ok_or("no events")?.items.iter()
            .filter_map(Lua::table).map(event).collect::<Result<Vec<_>, _>>()?;
        events.sort_by(|a, b| a.time.total_cmp(&b.time));
        Ok(Script { name: name.to_string(), level: String::new(), duration, entities, events })
    }

    /// The entity named `name` (as events name them; case is the file's).
    pub fn entity(&self, name: &str) -> Option<&Entity> {
        self.entities.iter().find(|e| e.name == name)
    }
}

/// One `entities` table (FUN_0026db20's fields).
fn entity(t: &Table) -> Result<Entity, String> {
    let name = t.str("entity_name").ok_or("an entity without entity_name")?.to_string();
    let spatial = t.table("spatialprops");
    let pos = spatial.and_then(|s| s.get("pos")).and_then(Lua::vec3).unwrap_or(Vec3::ZERO);
    let rows: Vec<Vec3> = spatial.and_then(|s| s.table("orient")).map(|o| o.items.iter().filter_map(Lua::vec3).collect()).unwrap_or_default();
    let orient = if rows.len() >= 3 { Mat3::from_cols(rows[0], rows[1], rows[2]) } else { Mat3::IDENTITY };
    let camera = t.table("cameraprops").map(|c| CameraProps {
        fovh: c.num("fovh").unwrap_or(36.87) as f32,
        hvaspect: c.num("hvaspect").unwrap_or(4.0 / 3.0) as f32,
        near: c.num("nearplane").unwrap_or(0.005) as f32,
        far: c.num("farplane").unwrap_or(500.0) as f32,
    });
    Ok(Entity {
        hash: h(&name),
        name,
        kind: EntityKind::of(t.num("type").unwrap_or(0.0) as i64),
        template: t.str("template_name").unwrap_or("NULL").to_string(),
        pos,
        orient,
        camera,
    })
}

/// One `events` table: {time, type, {entities}, {params}} (FUN_0026cb40).
fn event(t: &Table) -> Result<Event, String> {
    let time = t.items.first().and_then(Lua::num).ok_or("an event without a time")? as f32;
    let kind = t.items.get(1).and_then(Lua::num).ok_or("an event without a type")? as i64;
    let entities: Vec<String> = t.items.get(2).and_then(Lua::table).map(|e| e.items.iter().filter_map(|v| v.str().map(str::to_string)).collect())
        .unwrap_or_default();
    let params = t.items.get(3).and_then(Lua::table).cloned().unwrap_or_default();
    let kind = match kind {
        1 => EventKind::User(UserEvent::parse(params.str("user_event_data").unwrap_or(""))),
        6 => EventKind::Path(PathAnimation {
            duration: params.num("duration").unwrap_or(0.0) as f32,
            // (`percent1` / `percent2`: the same fields' other names, FUN_0026cb40)
            start: params.num("start_percent").or(params.num("percent1")).unwrap_or(0.0) as f32,
            stop: params.num("stop_percent").or(params.num("percent2")).unwrap_or(1.0) as f32,
            offset: params.get("offset").and_then(Lua::vec3).unwrap_or(Vec3::ZERO),
            up: params.num("up").unwrap_or(1.0) as i64,
            front: params.num("front").unwrap_or(5.0) as i64,
            // (no `duration`: USE_SCRIPT_DURATION, which a `flags` field then replaces: the
            // record's flags are set to 1 first and overwritten by the field, FUN_0026cb40)
            flags: params.num("flags").map(|f| f as u32)
                .unwrap_or(if params.get("duration").is_none() { flags::USE_SCRIPT_DURATION } else { 0 }),
            event_flags: params.num("event_flags").unwrap_or(0.0) as u32,
        }),
        k => EventKind::Other(k, params),
    };
    Ok(Event { time, entities, kind })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The opening's shape: the reader keeps the duration, the entity types and the events'
    /// codes and flags.
    #[test]
    fn reads_a_script() {
        let src = r#"duration = 26.254000;
entities =
{
	{
		entity_name = "camera_opening",
		type = CAMERA,
		template_name = "NULL",
		template_id = 0,
		spatialprops =
		{
			pos = {3.527845, 19.826359, 72.073914},
			orient = { {0.149536, -0.000003, 0.988757},{-0.000017, 1.000000, 0.000005},{-0.988757, -0.000017, 0.149536} }
		},
		cameraprops =
		{
			fovh = 36.869896,
			hvaspect = 1.333333,
			nearplane = 0.005000,
			farplane = 500.000000
		}
	},
	{
		entity_name = "path0",
		type = MOTION_PATH,
		template_name = "NULL",
		template_id = 0,
		pathprops =
		{

			path_type = "",
			path_data = ""
		}
	}
};

events =
{
	{0.000000, USER_EVENT, {"trigger_letterbox14"},  {user_event_data = "103, 1, 11"}},
	{10.679000, START_PATH_ANIMATION, {"camera_opening", "path0"}, {duration = 9.069000, start_percent = 0.000000, stop_percent = 1.000000, offset = {0.000000,0.000000,0.000000}, up = Y_AXIS, front = NEG_Z_AXIS, flags = POSITION + ORIENTATION}},
	{0.000000, USER_EVENT, {"x"},  {user_event_data = "105, 4, 0, 1, 1, 0, 1:wpoint1380 "}}
};"#;
        let s = Script::parse("tutorial_start_long", src).unwrap();
        assert_eq!(s.duration, 26.254);
        assert_eq!(s.entities.len(), 2);
        assert_eq!(s.entities[0].kind, EntityKind::Camera);
        assert_eq!(s.entities[1].kind, EntityKind::MotionPath);
        assert!((s.entities[0].camera.unwrap().fovh - 36.87).abs() < 0.01);
        assert_eq!(s.events.len(), 3);
        let EventKind::User(u) = &s.events[0].kind else { panic!() };
        assert_eq!((u.code, u.variant, u.int(0)), (103, 1, 11));
        let EventKind::User(u) = &s.events[1].kind else { panic!() };
        assert_eq!((u.code, u.int(3), u.args[4].as_str()), (105, 0, "1:wpoint1380"));
        let EventKind::Path(p) = &s.events[2].kind else { panic!() };
        assert_eq!(p.flags, flags::POSITION | flags::ORIENTATION);
        assert_eq!(s.events[2].entities, vec!["camera_opening".to_string(), "path0".to_string()]);
    }
}
