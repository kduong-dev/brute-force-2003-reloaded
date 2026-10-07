//! Alchemy effect libraries (effects-<level>.ale, a Digital Anvil "UTF" file; port of ale_tool.py).
//!
//!  ALEffectLib         per effect: name, 4 floats, node refs (flag, node name hash, parent,
//!                      index) and emitter -> appearance pairs (ref indices)
//!  AlchemyNodeLibrary  per node: class hash, then parameters (u16 type, u32 name hash, value)
//!
//! Parameter meanings follow Freelancer's ALE (Brute Force hashes the names; see play_fx.rs for
//! the ones read here). Curves with several items vary with the effect's user parameter
//! ("sparam").

use std::collections::HashMap;

use super::hash::{name_hash_bytes, name_hash_exact};

#[derive(Clone, Debug)]
pub enum Value {
    Bool(bool),
    Int(i32),
    Float(f32),
    Str(String),
    U32(u32),
    Pair(u32, u32),
    /// the 9 curves of a transform (translation, rotation in degrees, scale), if present
    Transform(Vec<Curve>),
    /// float keys over a particle's life, per sparam: (sparam, easing, keys (t, v))
    Floats(Vec<(f32, u8, Vec<(f32, f32)>)>),
    Colors(Vec<(f32, u8, Vec<(f32, [f32; 3])>)>),
    Curve(Curve),
}

/// Keyframed value over an emitter's time (s), per sparam: (sparam, constant, flags, keys (t, v)).
/// Flags 0x10 / 0x20 mark keys that repeat (an icon's spin, a flickering particle life).
#[derive(Clone, Debug, Default)]
pub struct Curve(pub Vec<(f32, f32, u16, Vec<(f32, f32)>)>);

#[derive(Clone, Debug)]
pub struct Node {
    pub class: u32,
    pub name: String,
    pub params: HashMap<u32, Value>,
}

#[derive(Clone, Debug)]
pub struct Effect {
    pub name: String,
    /// (flag, node name hash, parent, index)
    pub refs: Vec<(u32, u32, u32, u32)>,
    /// (emitter ref index, appearance ref index)
    pub pairs: Vec<(u32, u32)>,
}

#[derive(Default)]
pub struct Library {
    pub effects: HashMap<u32, Effect>,
    pub nodes: HashMap<u32, Node>,
}

struct Reader<'a> {
    d: &'a [u8],
    o: usize,
}

impl Reader<'_> {
    fn bytes(&mut self, n: usize) -> Option<&[u8]> {
        let b = self.d.get(self.o..self.o + n)?;
        self.o += n;
        Some(b)
    }
    fn u8(&mut self) -> Option<u8> { self.bytes(1).map(|b| b[0]) }
    fn u16(&mut self) -> Option<u16> { self.bytes(2).map(|b| u16::from_le_bytes([b[0], b[1]])) }
    fn u32(&mut self) -> Option<u32> { self.bytes(4).map(|b| u32::from_le_bytes(b.try_into().unwrap())) }
    fn i32(&mut self) -> Option<i32> { self.u32().map(|v| v as i32) }
    fn f32(&mut self) -> Option<f32> { self.u32().map(f32::from_bits) }
    fn string(&mut self) -> Option<String> {
        let n = self.u16()? as usize;
        let b = self.bytes(n + (n & 1))?;
        let b = &b[..n];
        Some(String::from_utf8_lossy(&b[..b.iter().position(|&c| c == 0).unwrap_or(n)]).into_owned())
    }
    fn curve(&mut self) -> Option<Curve> {
        let (_easing, count) = (self.u8()?, self.u8()?);
        if count == 0 {
            return Some(Curve(vec![(0.0, self.f32()?, 0, vec![])]));
        }
        let mut items = vec![];
        for _ in 0..count {
            let (sp, v, flags, n) = (self.f32()?, self.f32()?, self.u16()?, self.u16()?);
            let mut keys = vec![];
            for _ in 0..n {
                let (t, kv) = (self.f32()?, self.f32()?);
                self.f32()?;
                self.f32()?;
                keys.push((t, kv));
            }
            items.push((sp, v, flags, keys));
        }
        Some(Curve(items))
    }
    fn value(&mut self, t: u16) -> Option<Value> {
        Some(match t & 0x7FFF {
            0x001 => Value::Bool(t & 0x8000 != 0),
            0x002 => Value::Int(self.i32()?),
            0x003 => Value::Float(self.f32()?),
            0x103 => Value::Str(self.string()?),
            0x004 => Value::U32(self.u32()?),
            0x104 => Value::Pair(self.u32()?, self.u32()?),
            0x105 => {
                let flags = self.u32()?;
                Value::Transform(if flags != 0 { (0..9).map(|_| self.curve()).collect::<Option<_>>()? } else { vec![] })
            }
            0x200 => {
                let (_e, count) = (self.u8()?, self.u8()?);
                let mut items = vec![];
                for _ in 0..count {
                    let (sp, ease, n) = (self.f32()?, self.u8()?, self.u8()?);
                    items.push((sp, ease, (0..n).map(|_| Some((self.f32()?, self.f32()?))).collect::<Option<_>>()?));
                }
                Value::Floats(items)
            }
            0x201 => {
                let (_e, count) = (self.u8()?, self.u8()?);
                let mut items = vec![];
                for _ in 0..count {
                    let (sp, ease, n) = (self.f32()?, self.u8()?, self.u8()?);
                    items.push((sp, ease, (0..n).map(|_| Some((self.f32()?, [self.f32()?, self.f32()?, self.f32()?]))).collect::<Option<_>>()?));
                }
                Value::Colors(items)
            }
            0x202 => Value::Curve(self.curve()?),
            _ => return None,
        })
    }
}

/// The leaves of a UTF file: name -> data.
fn utf_leaves(d: &[u8]) -> HashMap<String, &[u8]> {
    let mut out = HashMap::new();
    if d.get(..4) != Some(b"UTF ".as_slice()) {
        return out;
    }
    let u = |o: usize| d.get(o..o + 4).map(|b| u32::from_le_bytes(b.try_into().unwrap()) as usize).unwrap_or(0);
    let (tree, names, names_used, data) = (u(8), u(24), u(32), u(36));
    let name_at = |o: usize| {
        let s = &d[(names + o).min(d.len())..(names + names_used).min(d.len())];
        String::from_utf8_lossy(&s[..s.iter().position(|&c| c == 0).unwrap_or(s.len())]).into_owned()
    };
    let mut stack = vec![0usize];
    while let Some(mut o) = stack.pop() {
        loop {
            let e = tree + o;
            let (next, name, flags, child, size) = (u(e), u(e + 4), u(e + 8), u(e + 16), u(e + 24));
            if flags & 0x10 != 0 {
                if child != 0 {
                    stack.push(child);
                }
            } else if let Some(b) = d.get(data + child..data + child + size) {
                out.insert(name_at(name), b);
            }
            if next == 0 {
                break;
            }
            o = next;
        }
    }
    out
}

impl Library {
    pub fn parse(d: &[u8]) -> Self {
        let leaves = utf_leaves(d);
        let mut lib = Library::default();
        if let Some(b) = leaves.get("AlchemyNodeLibrary") {
            let mut r = Reader { d: b, o: 0 };
            let _ver = r.f32();
            let count = r.i32().unwrap_or(0);
            'nodes: for _ in 0..count {
                let Some(class) = r.u32() else { break };
                let mut params = HashMap::new();
                let mut name = String::new();
                loop {
                    let Some(t) = r.u16() else { break 'nodes };
                    if t == 0 {
                        break;
                    }
                    let Some(crc) = r.u32() else { break 'nodes };
                    let Some(v) = r.value(t) else { break 'nodes };
                    if let (true, Value::Str(s)) = (name.is_empty(), &v) {
                        name = s.clone();
                    }
                    params.insert(crc, v);
                }
                // effects refer to nodes by the case-kept hash of their name; keep the usual
                // (lower-case) one too
                let node = Node { class, name, params };
                lib.nodes.entry(name_hash_exact(node.name.as_bytes())).or_insert_with(|| node.clone());
                lib.nodes.entry(name_hash_bytes(node.name.as_bytes())).or_insert(node);
            }
        }
        if let Some(b) = leaves.get("ALEffectLib") {
            let mut r = Reader { d: b, o: 0 };
            let _ver = r.f32();
            let count = r.i32().unwrap_or(0);
            for _ in 0..count {
                let Some(name) = r.string() else { break };
                for _ in 0..4 {
                    r.f32();
                }
                let n = r.i32().unwrap_or(0).max(0) as usize;
                let refs = (0..n).filter_map(|_| Some((r.u32()?, r.u32()?, r.u32()?, r.u32()?))).collect();
                let m = r.i32().unwrap_or(0).max(0) as usize;
                let pairs = (0..m).filter_map(|_| Some((r.u32()?, r.u32()?))).collect();
                lib.effects.insert(name_hash_bytes(name.as_bytes()), Effect { name, refs, pairs });
            }
        }
        lib
    }
}

/// Easing as Freelancer's: 1 linear, 2 in, 3 out, 4 in-out.
pub fn ease(kind: u8, f: f32) -> f32 {
    match kind {
        2 => f * f,
        3 => 1.0 - (1.0 - f) * (1.0 - f),
        4 => f * f * (3.0 - 2.0 * f),
        _ => f,
    }
}

fn track(keys: &[(f32, f32)], kind: u8, t: f32) -> f32 {
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

/// Blend the items around `sp` (items sorted by sparam).
fn by_sparam<T>(items: &[(f32, T)], sp: f32, f: impl Fn(&T) -> f32) -> f32 {
    if items.is_empty() {
        return 0.0;
    }
    let i = items.iter().position(|(s, _)| *s >= sp).unwrap_or(items.len() - 1);
    if i == 0 || items[i].0 <= sp {
        return f(&items[i].1);
    }
    let w = (sp - items[i - 1].0) / (items[i].0 - items[i - 1].0);
    f(&items[i - 1].1) * (1.0 - w) + f(&items[i].1) * w
}

impl Curve {
    pub fn at(&self, sp: f32, t: f32) -> f32 {
        let items: Vec<(f32, (f32, u16, &Vec<(f32, f32)>))> = self.0.iter().map(|(s, v, f, k)| (*s, (*v, *f, k))).collect();
        by_sparam(&items, sp, |(v, f, k)| match k.last() {
            None => *v,
            Some(&(end, _)) if f & 0x30 != 0 && end > 1e-4 => track(k, 1, t.rem_euclid(end)),
            _ => track(k, 1, t),
        })
    }
}

impl Node {
    pub fn float(&self, p: u32) -> Option<f32> {
        match self.params.get(&p)? { Value::Float(v) => Some(*v), _ => None }
    }
    pub fn int(&self, p: u32) -> Option<i32> {
        match self.params.get(&p)? { Value::Int(v) => Some(*v), _ => None }
    }
    pub fn flag(&self, p: u32) -> bool {
        matches!(self.params.get(&p), Some(Value::Bool(true)))
    }
    pub fn string(&self, p: u32) -> Option<&str> {
        match self.params.get(&p)? { Value::Str(s) => Some(s), _ => None }
    }
    pub fn pair(&self, p: u32) -> Option<(u32, u32)> {
        match self.params.get(&p)? { Value::Pair(a, b) => Some((*a, *b)), _ => None }
    }
    pub fn curve(&self, p: u32, sp: f32, t: f32) -> Option<f32> {
        match self.params.get(&p)? { Value::Curve(c) => Some(c.at(sp, t)), _ => None }
    }
    /// A float animation over life `k` (0-1).
    pub fn floats(&self, p: u32, sp: f32, k: f32) -> Option<f32> {
        match self.params.get(&p)? {
            Value::Floats(items) => {
                let items: Vec<(f32, (u8, &Vec<(f32, f32)>))> = items.iter().map(|(s, e, k)| (*s, (*e, k))).collect();
                Some(by_sparam(&items, sp, |(e, keys)| track(keys, *e, k)))
            }
            _ => None,
        }
    }
    /// A colour animation over life `k` (0-1).
    pub fn color(&self, p: u32, sp: f32, k: f32) -> Option<[f32; 3]> {
        match self.params.get(&p)? {
            Value::Colors(items) => {
                let items: Vec<(f32, (u8, &Vec<(f32, [f32; 3])>))> = items.iter().map(|(s, e, k)| (*s, (*e, k))).collect();
                Some([0, 1, 2].map(|c| by_sparam(&items, sp, |(e, keys)| {
                    let t: Vec<(f32, f32)> = keys.iter().map(|(t, v)| (*t, v[c])).collect();
                    track(&t, *e, k)
                })))
            }
            _ => None,
        }
    }
    /// The node transform's rotation (degrees about x, y, z), at sparam 0, time 0.
    pub fn rotation(&self, p: u32) -> [f32; 3] {
        self.transform(p, 0.0)[1]
    }

    /// The node transform at time `t` (s): translation, rotation (degrees about x, y, z), scale.
    pub fn transform(&self, p: u32, t: f32) -> [[f32; 3]; 3] {
        match self.params.get(&p) {
            Some(Value::Transform(c)) if c.len() == 9 => [0, 1, 2].map(|i| [0, 1, 2].map(|j| c[i * 3 + j].at(0.0, t))),
            _ => [[0.0; 3], [0.0; 3], [1.0; 3]],
        }
    }
}
