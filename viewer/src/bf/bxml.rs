//! BXML (.xmb) documents and BXSD (.xsb) schemas. Port of xmb_tool.py.
//!
//! A document is a header, namespace table and name-hash table, followed by LZ-compressed
//! 16-bit tokens (0x0nnn child, 0xAnnn attribute, 0xBnnn empty child, 0xC000 end, 0xDnnn text,
//! 0x9nnn source line) interleaved with raw value bytes. How many bytes each value takes comes
//! from the schema type of the attribute / element, so documents are decoded against schemas.

use std::collections::{HashMap, HashSet, VecDeque};

use super::hash::h;

pub type R<T> = Result<T, String>;

// ---- reader ---------------------------------------------------------------------------------

pub struct Reader<'a> {
    d: &'a [u8],
    pub pos: usize,
}

impl<'a> Reader<'a> {
    pub fn new(d: &'a [u8], pos: usize) -> Self {
        Self { d, pos }
    }
    pub fn bytes(&mut self, n: usize) -> R<&'a [u8]> {
        if self.pos + n > self.d.len() {
            return Err(format!("read of {n} bytes at {:#x} past end ({:#x})", self.pos, self.d.len()));
        }
        let s = &self.d[self.pos..self.pos + n];
        self.pos += n;
        Ok(s)
    }
    pub fn u8(&mut self) -> R<u8> {
        Ok(self.bytes(1)?[0])
    }
    pub fn u16(&mut self) -> R<u16> {
        let b = self.bytes(2)?;
        Ok(u16::from_le_bytes([b[0], b[1]]))
    }
    pub fn u32(&mut self) -> R<u32> {
        let b = self.bytes(4)?;
        Ok(u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
    }
    pub fn varint(&mut self) -> R<u64> {
        let (mut v, mut s) = (0u64, 0);
        loop {
            let b = self.u8()?;
            v |= ((b & 0x7F) as u64) << s;
            s += 7;
            if b & 0x80 == 0 {
                return Ok(v);
            }
        }
    }
    /// Tagged hash list: 0x00 = none, 0x2N = N hashes (0x21 local name, 0x22 ns-uri + name).
    pub fn qname(&mut self) -> R<Option<Vec<u32>>> {
        let t = self.u8()?;
        if t == 0 {
            return Ok(None);
        }
        (0..(t & 0x0F)).map(|_| self.u32()).collect::<R<Vec<_>>>().map(Some)
    }
}

// ---- token stream (XBE 0x281bf0 / 0x281d30) -----------------------------------------------------

/// Block-structured LZ over u16 words sharing the file cursor with the value bytes. A block is
/// loaded only when a raw word is needed. Flag 0: literal; flag 1: copy (w>>9)+2 words from the
/// 512-word ring at absolute position w & 0x1FF.
struct TokenStream {
    wide: bool,
    words: Vec<u16>,
    idx: usize,
    ring: [u16; 512],
    wp: usize,
    copy: usize,
    rp: usize,
    flags: u16,
    bit: u32,
}

impl TokenStream {
    fn new(r: &mut Reader, wide: bool) -> R<Self> {
        let mut s = Self { wide, words: vec![], idx: 0, ring: [0; 512], wp: 1, copy: 0, rp: 0, flags: 0, bit: 0 };
        s.flags = s.raw(r)?.0;
        s.bit = 0;
        Ok(s)
    }

    fn raw(&mut self, r: &mut Reader) -> R<(u16, bool)> {
        let mut new = false;
        if self.idx >= self.words.len() {
            let n = r.u16()? as usize;
            if n < 3 {
                return Err(format!("bad token block header {n} at {:#x}", r.pos - 2));
            }
            let mut w = Vec::with_capacity(n);
            w.push(n as u16);
            for _ in 1..n {
                w.push(r.u16()?);
            }
            self.words = w;
            self.idx = if self.wide { 3 } else { 2 };
            self.wp = 1;
            new = true;
        }
        let w = *self.words.get(self.idx).ok_or("token block underrun")?;
        self.idx += 1;
        Ok((w, new))
    }

    fn next(&mut self, r: &mut Reader) -> R<u16> {
        loop {
            if self.copy > 0 {
                let v = self.ring[self.rp & 0x1FF];
                self.rp += 1;
                self.ring[self.wp] = v;
                self.wp = (self.wp + 1) & 0x1FF;
                self.copy -= 1;
                return Ok(v);
            }
            if self.bit == 16 {
                self.flags = self.raw(r)?.0;
                self.bit = 0;
            }
            let mut f = (self.flags >> self.bit) & 1;
            self.bit += 1;
            let (mut w, new) = self.raw(r)?;
            if new {
                self.flags = w;
                f = w & 1;
                self.bit = 1;
                w = self.raw(r)?.0;
            }
            if f == 0 {
                self.ring[self.wp] = w;
                self.wp = (self.wp + 1) & 0x1FF;
                return Ok(w);
            }
            self.rp = (w & 0x1FF) as usize;
            self.copy = ((w >> 9) + 2) as usize;
        }
    }
}

// ---- schemas --------------------------------------------------------------------------------------

pub const K_COMPLEX: u16 = 0x01;
pub const K_SIMPLE: u16 = 0x02;
pub const K_LIST: u16 = 0x06;
pub const K_ELEM: u16 = 0x09;
pub const K_ATTR: u16 = 0x0A;
pub const K_ELEMREF: u16 = 0x18;
pub const K_GELEM: u16 = 0x19;
pub const K_PROPS: u16 = 0x80;
pub const K_EXT: u16 = 0x101;

const XS_URI: u32 = h("http://www.w3.org/2001/XMLSchema");
const DAXNS: u32 = h("daxns");
const H_STRINGID: u32 = h("stringid");
const H_WSTRING: u32 = h("wstring");
const H_INDEXSTR: u32 = 0xF548_750A;

#[derive(Clone)]
pub struct SNode {
    pub kind: u16,
    pub name: Option<Vec<u32>>,
    pub ty: Option<Vec<u32>>,
    pub children: Vec<usize>,
    pub ns: u32,
}

impl SNode {
    fn local(&self) -> Option<u32> {
        self.name.as_ref().and_then(|v| v.last().copied())
    }
}

enum Res {
    Node(usize),
    Builtin(u32, u32),
}

#[derive(Clone, Debug)]
pub enum Codec {
    Fixed(u8),
    Hash,
    Index,
    Str,
    WStr,
    List(Box<Codec>),
}

/// (Clone: a thread that reads level archives on its own takes a copy, play_testworld.rs)
#[derive(Default, Clone)]
pub struct SchemaSet {
    nodes: Vec<SNode>,
    globals: HashMap<(u32, u32), usize>,
    targets: Vec<u32>,
}

impl SchemaSet {
    pub fn add(&mut self, data: &[u8]) -> R<()> {
        if !data.starts_with(b"BXSD") {
            return Err("not BXSD".into());
        }
        let mut r = Reader::new(data, 0x10);
        r.varint()?;
        let target = *r.qname()?.ok_or("schema without target namespace")?.last().unwrap();
        for _ in 0..r.varint()? {
            r.u32()?;
            r.u32()?;
        }
        r.pos = u32::from_le_bytes(data[8..12].try_into().unwrap()) as usize;
        let mut tops = vec![];
        for _ in 0..r.varint()? {
            tops.push(self.node(&mut r, target)?);
        }
        if r.pos != data.len() {
            return Err(format!("schema parse ended at {:#x} of {:#x}", r.pos, data.len()));
        }
        if !self.targets.contains(&target) {
            self.targets.push(target);
        }
        for t in tops {
            if let Some(l) = self.nodes[t].local() {
                self.globals.entry((target, l)).or_insert(t);
            }
        }
        Ok(())
    }

    fn node(&mut self, r: &mut Reader, ns: u32) -> R<usize> {
        let kind = r.u16()?;
        let name = r.qname()?;
        let ty = r.qname()?;
        let idx = self.nodes.len();
        self.nodes.push(SNode { kind, name, ty, children: vec![], ns });
        for _ in 0..r.varint()? {
            let c = self.node(r, ns)?;
            self.nodes[idx].children.push(c);
        }
        Ok(idx)
    }

    fn resolve(&self, qn: &Option<Vec<u32>>, ns: u32) -> Option<Res> {
        let q = qn.as_ref()?;
        let key = if q.len() >= 2 { (q[0], q[1]) } else { (ns, q[0]) };
        Some(match self.globals.get(&key) {
            Some(&i) => Res::Node(i),
            None => Res::Builtin(key.0, key.1),
        })
    }

    /// Children of a complex type, following K_EXT inheritance to the base.
    fn members(&self, mut node: usize) -> Vec<usize> {
        let mut out = vec![];
        let mut seen = HashSet::new();
        while seen.insert(node) {
            let n = &self.nodes[node];
            out.extend(&n.children);
            match (n.kind == K_EXT).then(|| self.resolve(&n.ty, n.ns)).flatten() {
                Some(Res::Node(b)) => node = b,
                _ => break,
            }
        }
        out
    }

    pub fn global_elem(&self, name: u32, nss: &[u32]) -> Option<usize> {
        nss.iter().find_map(|ns| {
            let g = *self.globals.get(&(*ns, name))?;
            matches!(self.nodes[g].kind, K_GELEM | K_ELEM | K_ELEMREF | K_SIMPLE).then_some(g)
        })
    }

    fn find_member(&self, ctx: Option<usize>, name: u32, attr: bool) -> Option<usize> {
        let ctx = ctx?;
        let mut queue: VecDeque<usize> = self.members(ctx).into();
        while let Some(c) = queue.pop_front() {
            let n = &self.nodes[c];
            if n.local() == Some(name) {
                if attr && matches!(n.kind, K_ATTR | K_SIMPLE | K_GELEM) {
                    return Some(c);
                }
                if !attr && matches!(n.kind, K_ELEM | K_GELEM | K_PROPS | K_ELEMREF | K_SIMPLE) {
                    return Some(c);
                }
            }
            if n.kind == K_PROPS && self.nodes[ctx].kind == K_PROPS {
                queue.extend(&n.children);
            }
        }
        if !attr {
            // list-typed attributes (vectors) are encoded as a child element + text token
            if let Some(c) = self.members(ctx).into_iter()
                .find(|&c| self.nodes[c].local() == Some(name) && self.nodes[c].kind == K_ATTR)
            {
                return Some(c);
            }
            let own = self.nodes[ctx].ns;
            let mut nss = vec![own];
            nss.extend(self.targets.iter().copied().filter(|&t| t != own));
            return self.global_elem(name, &nss);
        }
        None
    }

    fn content(&self, decl: Option<usize>) -> Option<usize> {
        let d = decl?;
        let n = &self.nodes[d];
        if matches!(n.kind, K_ELEM | K_PROPS) {
            return Some(d);
        }
        match self.resolve(&n.ty, n.ns) {
            Some(Res::Node(t)) if matches!(self.nodes[t].kind, K_COMPLEX | K_EXT | K_ELEM) => Some(t),
            _ => None,
        }
    }

    fn builtin(ns: u32, name: u32) -> Codec {
        if ns == XS_URI {
            for (n, f) in [("boolean", b'?'), ("byte", b'b'), ("unsignedByte", b'B'), ("short", b'h'),
                           ("unsignedShort", b'H'), ("int", b'i'), ("unsignedInt", b'I'), ("float", b'f'),
                           ("double", b'd'), ("long", b'q'), ("unsignedLong", b'Q')] {
                if h(n) == name {
                    return Codec::Fixed(f);
                }
            }
        }
        Codec::Str
    }

    fn codec(&self, decl: usize) -> Option<Codec> {
        let n = &self.nodes[decl];
        self.codec_walk(n.kind, n.local(), n.ty.clone(), n.ns, true)
    }

    fn codec_walk(&self, mut kind: u16, mut local: Option<u32>, mut ty: Option<Vec<u32>>, mut ns: u32,
                  mut first: bool) -> Option<Codec> {
        for _ in 0..64 {
            if !first {
                if local == Some(H_STRINGID) && ns == DAXNS {
                    return Some(Codec::Hash);
                }
                if local == Some(H_INDEXSTR) {
                    return Some(Codec::Index);
                }
                if local == Some(H_WSTRING) && ns == DAXNS {
                    return Some(Codec::WStr);
                }
                if kind == K_LIST {
                    return match self.resolve(&ty, ns) {
                        Some(Res::Node(_)) => Some(Codec::List(Box::new(self.codec_walk(K_SIMPLE, None, ty, ns, true)?))),
                        Some(Res::Builtin(a, b)) => Some(Codec::List(Box::new(Self::builtin(a, b)))),
                        None => Some(Codec::Str),
                    };
                }
            }
            first = false;
            if ty.is_none() {
                return matches!(kind, K_SIMPLE | K_ATTR).then_some(Codec::Str);
            }
            match self.resolve(&ty, ns)? {
                Res::Builtin(a, b) => return Some(Self::builtin(a, b)),
                Res::Node(t) => {
                    let tn = &self.nodes[t];
                    if matches!(tn.kind, K_COMPLEX | K_EXT | K_ELEM) {
                        return None;
                    }
                    kind = tn.kind;
                    local = tn.local();
                    ty = tn.ty.clone();
                    ns = tn.ns;
                }
            }
        }
        Some(Codec::Str)
    }
}

// ---- document model -------------------------------------------------------------------------------

#[derive(Clone, Debug)]
pub enum Value {
    Bool(bool),
    Int(i64),
    Float(f64),
    Hash(u32),
    Str(String),
    List(Vec<Value>),
}

impl Value {
    pub fn as_i64(&self) -> Option<i64> {
        match self {
            Value::Int(v) => Some(*v),
            Value::Bool(b) => Some(*b as i64),
            Value::Float(f) => Some(*f as i64),
            Value::Hash(h) => Some(*h as i64),
            _ => None,
        }
    }
    pub fn as_f32(&self) -> Option<f32> {
        match self {
            Value::Float(f) => Some(*f as f32),
            Value::Int(v) => Some(*v as f32),
            _ => None,
        }
    }
    /// stringid values are hashes; plain string values are hashed so both compare alike
    pub fn as_hash(&self) -> Option<u32> {
        match self {
            Value::Hash(h) => Some(*h),
            Value::Str(s) => Some(super::hash::name_hash_bytes(s.as_bytes())),
            _ => None,
        }
    }
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Value::Str(s) => Some(s),
            _ => None,
        }
    }
    pub fn floats(&self) -> Vec<f32> {
        match self {
            Value::List(v) => v.iter().filter_map(|x| x.as_f32()).collect(),
            other => other.as_f32().into_iter().collect(),
        }
    }
    pub fn ints(&self) -> Vec<i64> {
        match self {
            Value::List(v) => v.iter().filter_map(|x| x.as_i64()).collect(),
            other => other.as_i64().into_iter().collect(),
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct Element {
    pub name: u32,
    pub attrs: Vec<(u32, Value)>,
    pub children: Vec<Element>,
    pub text: Option<Value>,
}

impl Element {
    pub fn attr(&self, name: u32) -> Option<&Value> {
        self.attrs.iter().find(|(k, _)| *k == name).map(|(_, v)| v)
    }
    pub fn children_named(&self, name: u32) -> impl Iterator<Item = &Element> {
        self.children.iter().filter(move |c| c.name == name)
    }
    pub fn child(&self, name: u32) -> Option<&Element> {
        self.children.iter().find(|c| c.name == name)
    }
    /// Depth-first walk over this element and all descendants.
    pub fn walk(&self) -> Vec<&Element> {
        let mut out = vec![];
        let mut stack = vec![self];
        while let Some(e) = stack.pop() {
            out.push(e);
            for c in e.children.iter().rev() {
                stack.push(c);
            }
        }
        out
    }
}

struct Parser<'a, 'b> {
    r: Reader<'a>,
    ts: TokenStream,
    names: Vec<u32>,
    s: &'b SchemaSet,
}

impl Parser<'_, '_> {
    fn nm(&self, idx: u16) -> u32 {
        *self.names.get(idx as usize).unwrap_or(&0xFFFF_FFFF)
    }

    fn count(&mut self, t: u16) -> R<usize> {
        if t & 0x800 == 0 {
            return Ok((t & 0xFFF) as usize);
        }
        let (mut n, mut shift) = ((t & 0x7FF) as usize, 11);
        loop {
            let w = self.ts.next(&mut self.r)?;
            n |= ((w & 0x7FFF) as usize) << shift;
            shift += 15;
            if w & 0x8000 == 0 {
                return Ok(n);
            }
        }
    }

    fn fixed(&mut self, f: u8) -> R<Value> {
        let r = &mut self.r;
        Ok(match f {
            b'?' => Value::Bool(r.u8()? != 0),
            b'b' => Value::Int(r.u8()? as i8 as i64),
            b'B' => Value::Int(r.u8()? as i64),
            b'h' => Value::Int(r.u16()? as i16 as i64),
            b'H' => Value::Int(r.u16()? as i64),
            b'i' => Value::Int(r.u32()? as i32 as i64),
            b'I' => Value::Int(r.u32()? as i64),
            b'f' => Value::Float(f32::from_bits(r.u32()?) as f64),
            b'd' => Value::Float(f64::from_le_bytes(r.bytes(8)?.try_into().unwrap())),
            b'q' => Value::Int(i64::from_le_bytes(r.bytes(8)?.try_into().unwrap())),
            _ => Value::Int(u64::from_le_bytes(r.bytes(8)?.try_into().unwrap()) as i64),
        })
    }

    fn value(&mut self, codec: Option<&Codec>, count: usize, what: &str) -> R<Value> {
        let codec = codec.ok_or_else(|| format!("{what}: type unknown at {:#x}", self.r.pos))?;
        Ok(match codec {
            Codec::Fixed(f) => {
                if count == 1 {
                    self.fixed(*f)?
                } else {
                    Value::List((0..count).map(|_| self.fixed(*f)).collect::<R<_>>()?)
                }
            }
            Codec::Hash => Value::Hash(self.r.u32()?),
            Codec::Index => Value::Int(self.r.varint()? as i64 - 1),
            Codec::Str => {
                let n = self.r.varint()? as usize;
                Value::Str(self.r.bytes(n)?.iter().map(|&b| b as char).collect())
            }
            Codec::WStr => {
                let n = self.r.varint()? as usize;
                let b = self.r.bytes(n)?;
                let u: Vec<u16> = b.chunks_exact(2).map(|c| u16::from_le_bytes([c[0], c[1]])).collect();
                Value::Str(String::from_utf16_lossy(&u))
            }
            Codec::List(inner) => match **inner {
                Codec::Fixed(_) => match self.value(Some(inner), count, what)? {
                    Value::List(v) => Value::List(v),
                    one => Value::List(vec![one]),
                },
                _ => Value::List((0..count).map(|_| self.value(Some(inner), 1, what)).collect::<R<_>>()?),
            },
        })
    }

    fn element(&mut self, el: &mut Element, decl: Option<usize>) -> R<()> {
        let ctx = self.s.content(decl);
        loop {
            let t = self.ts.next(&mut self.r)?;
            match t & 0xF000 {
                0x9000 => {
                    self.count(t)?;
                }
                0x0000 | 0xB000 => {
                    let mut child = Element { name: self.nm(t & 0xFFF), ..Default::default() };
                    if t & 0xF000 == 0 {
                        let cdecl = self.s.find_member(ctx, child.name, false);
                        self.element(&mut child, cdecl)?;
                    }
                    el.children.push(child);
                }
                0xA000 => {
                    let name = self.nm(t & 0xFFF);
                    let codec = self.s.find_member(ctx, name, true).and_then(|d| self.s.codec(d));
                    let v = self.value(codec.as_ref(), 1, "attribute")?;
                    el.attrs.push((name, v));
                }
                0xC000 => return Ok(()),
                0xD000 => {
                    let n = self.count(t)?;
                    let codec = decl.and_then(|d| self.s.codec(d));
                    el.text = Some(self.value(codec.as_ref(), n, "text")?);
                }
                _ => return Err(format!("unknown token {t:04x} at {:#x}", self.r.pos)),
            }
        }
    }
}

/// Decode a BXML document against the schemas.
pub fn parse(data: &[u8], schemas: &SchemaSet) -> R<Element> {
    if !data.starts_with(b"BXML") {
        return Err("not BXML".into());
    }
    let names_off = u32::from_le_bytes(data[8..12].try_into().unwrap()) as usize;
    let mut r = Reader::new(data, 0x10);
    let nns = r.varint()?;
    let mut namespaces = vec![];
    for _ in 0..nns {
        namespaces.push((r.u32()?, r.u32()?));
    }
    r.pos = names_off;
    let names = (0..r.varint()?).map(|_| r.u32()).collect::<R<Vec<_>>>()?;
    let wide = (data[4], data[5]) >= (1, 1);
    let ts = TokenStream::new(&mut r, wide)?;
    let mut p = Parser { r, ts, names, s: schemas };
    let t = p.ts.next(&mut p.r)?;
    let mut root = Element { name: p.nm(t & 0xFFF), ..Default::default() };
    let mut nss: Vec<u32> = namespaces.iter().filter(|(pfx, _)| *pfx == 0).map(|(_, u)| *u).collect();
    nss.extend(namespaces.iter().filter(|(pfx, _)| *pfx != 0).map(|(_, u)| *u));
    let decl = schemas.global_elem(root.name, &nss);
    p.element(&mut root, decl)?;
    if p.r.pos != data.len() {
        return Err(format!("decoding stopped at {:#x} of {:#x}", p.r.pos, data.len()));
    }
    Ok(root)
}
