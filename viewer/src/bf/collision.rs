//! Collision surfaces: objects-<level>.ipn, Ipion "compact surfaces" (the game's physics
//! engine, later Havok), listed by name in the objects file:
//!
//!   <h_104db1cd data-file="objects-<lvl>.ipn"><h_f3f067ad name=N h_e9e44859=OFFSET/>...
//!
//! Archetypes name theirs with `<h_f3f067ad h_051ac63e=N>`; the level's terrain blocks and
//! blockers name theirs the same way (h_051ac63e). Layout (little endian):
//!
//!   surface  0x00 mass centre, rotation inertia (6 f32), radius (f32),
//!            0x1c u32 (byte size << 8 | deviation), 0x20 u32 ledge-tree root offset, 3 u32
//!   node     28 bytes: i32 right child (0 = leaf), i32 ledge offset (from the node), centre
//!            (3 f32), radius, 3 box bytes + 1 (the left child follows the node)
//!   ledge    i32 point offset, i32 client data, u32 flags (size / 16 << 8), i16 triangles,
//!            i16; then the triangles
//!   triangle u32 index:12 | pierce:12 | material:7 | virtual:1, then 3 edges: u32 start point
//!            index:16 | opposite:15 | virtual:1 (the edges' start points are the corners)
//!   point    x, y, z, f32 (16 bytes, in the owner's frame: world for terrain blocks)
//!
//! The material's low 5 bits are the world-material id (footstep / hit sounds); bit 5 marks a
//! surface characters slide down (the game reads it as triangle header bit 29, see
//! decompiled/xbe/ghidra/README.md "Player movement").

use bevy::math::Vec3;

/// A triangle's material: world-material id and the slide flag.
pub const MATERIAL_ID: u8 = 0x1F;
pub const MATERIAL_SLIDE: u8 = 0x20;

fn i32_at(d: &[u8], o: usize) -> Option<i32> {
    d.get(o..o + 4).map(|b| i32::from_le_bytes(b.try_into().unwrap()))
}

fn f32_at(d: &[u8], o: usize) -> Option<f32> {
    d.get(o..o + 4).map(|b| f32::from_le_bytes(b.try_into().unwrap()))
}

/// The triangles (corners, material) of the surface at `offset` in an .ipn blob.
pub fn surface(d: &[u8], offset: usize) -> Vec<([Vec3; 3], u8)> {
    let mut out = vec![];
    let Some(root) = i32_at(d, offset + 0x20) else { return out };
    let mut stack = vec![offset + root as usize];
    let mut guard = 0;
    while let Some(node) = stack.pop() {
        guard += 1;
        if guard > 1_000_000 {
            break;
        }
        let (Some(right), Some(ledge)) = (i32_at(d, node), i32_at(d, node + 4)) else { continue };
        if right != 0 {
            // inner node: left child right after this one, right child at the offset
            stack.push(node + 28);
            stack.push((node as i64 + right as i64) as usize);
            continue;
        }
        let l = (node as i64 + ledge as i64) as usize;
        let (Some(points), Some(count)) = (i32_at(d, l), d.get(l + 12..l + 14).map(|b| i16::from_le_bytes([b[0], b[1]]))) else { continue };
        let base = (l as i64 + points as i64) as usize;
        for t in 0..count.max(0) as usize {
            let a = l + 16 + t * 16;
            let Some(head) = i32_at(d, a) else { break };
            let material = ((head as u32 >> 24) & 0x7F) as u8;
            let corner = |k: usize| -> Option<Vec3> {
                let p = base + (i32_at(d, a + 4 + k * 4)? as u32 & 0xFFFF) as usize * 16;
                Some(Vec3::new(f32_at(d, p)?, f32_at(d, p + 4)?, f32_at(d, p + 8)?))
            };
            if let (Some(p0), Some(p1), Some(p2)) = (corner(0), corner(1), corner(2)) {
                out.push(([p0, p1, p2], material));
            }
        }
    }
    out
}
