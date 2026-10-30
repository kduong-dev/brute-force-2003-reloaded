//! The lamps that go out with something (#308; viewer/docs/levels.md "Lights that go with their
//! object"): a lamp is a point light-object of the level (`LevelLamp`), and the game takes one
//! out of the world in two ways.
//!  * **With its object.** A light-object with a `<parent>` link of type 1 or 2 to a placed object
//!    goes out with it. The game's light-object ticks (vtable +0x6c, 0x14f1d0) and each tick asks
//!    its link (FUN_0015ed30) whether its object is still there; once the link's object pointer
//!    has been cleared (the object's destructor clears every weak pointer to it, FUN_001553f0 ->
//!    FUN_0008f720: the end of a break's countdown, FUN_0015ae30, or a destroyer's removal) that
//!    returns 1, the tick returns 0, and the world takes the light out (FUN_000d9840 ->
//!    FUN_000db3e0), whatever the link's type. So a torch's glow (m01_a, m01_b, m05_b, m05_y: a
//!    lamp on the holder's hardpoint, with the flame beside it, a placed effect:
//!    play_placed_fx.rs) is put out when its holder breaks, and m14_d's three lamps with the
//!    capsules they hang on. 15 lamps over 7 levels hang on a breakable object
//!    (`LevelLamp::parent`); more hang on one that only dies and stays (the 40 hp sconces), which
//!    never go.
//!  * **By a destroyer-trigger.** 1019 lamps over 25 levels are named by a destroyer (the mission
//!    levels' lamp posts and fixtures: m04_c has 140): each has a router listening to its
//!    fixture's DESTROYED and a destroyer behind it, and the destroyer takes the light out at
//!    once, FUN_00200b70 -> FUN_000db3e0 (a light-object has no debris list). play_triggers.rs
//!    runs the destroyer and hands the lamp here (`LampsOut::put_out`); before, it logged "removes
//!    the placed ..." and left the lamp lit for good.
//!
//! The level's lamps are lit by the level materials from a buffer they share (lamps.wgsl,
//! `LevelLamps`), and by the characters through `lamp_light_at`: a lamp put out has its colour
//! zeroed in the buffer and is skipped there (`LampsOut`).
//!
//! Test hook: `BF_LINK_LOG=1` logs each lamp put out (and each placed effect taken away or
//! stopped, play_placed_fx.rs). The recipes are in scenery.md ("Verified").

use std::collections::HashSet;

use bevy::render::storage::ShaderStorageBuffer;
use bf_viewer::level_scene::{lamp_off, touch_materials, LevelLamps, LevelMaterial};

use super::*;

/// The lamps put out on this map, and what's left to do about them.
#[derive(Resource, Default)]
pub struct LampsOut {
    /// the lamps put out so far: their indices in `Level::lamps`
    pub off: HashSet<usize>,
    /// the lamps that hang on a placed object that can go, with its breakable's number
    /// (`Arena::breakable`); made when the map's arena is up
    watch: Option<Vec<(usize, usize)>>,
    /// lamps a destroyer-trigger has taken out, not put out yet (`put_out`)
    pending: Vec<usize>,
    /// frames until the level materials are touched, after the buffer changed (`lamps_out`)
    touch: Option<u32>,
}

impl LampsOut {
    /// A destroyer-trigger takes lamp `lamp` (its index in `Level::lamps`) out of the world: it's
    /// put out on this frame's `lamps_out`, or the next if the triggers ran after it.
    pub fn put_out(&mut self, lamp: usize) {
        if !self.off.contains(&lamp) && !self.pending.contains(&lamp) {
            self.pending.push(lamp);
        }
    }
}

/// Frames the light buffer's change is given to reach the GPU before the materials' bind groups
/// are made again from it: a bind group made before the new buffer is prepared would still hold
/// the old one. One frame on, the lamp's light left the railing on m01_a in the frame its holder's
/// model did (the railing's mean red 79.48 -> 79.31); with two frames it took a frame more. Touched
/// in the same frame, the bind groups might be made before the buffer is prepared: not tried.
const TOUCH_AFTER: u32 = 1;

pub fn plugin(app: &mut App) {
    app.init_resource::<LampsOut>()
        .add_systems(OnEnter(AppState::Playing), (|mut commands: Commands| commands.insert_resource(LampsOut::default())).after(setup))
        .add_systems(Update, lamps_out.after(super::scenery::SceneryStep).run_if(in_state(AppState::Playing)));
}

/// Puts out each lamp whose object has gone (`Arena::is_gone`) and each a destroyer has taken
/// (`LampsOut::put_out`), on that frame: in the light buffer the level materials read (`lamp_off`;
/// their bind groups are made again a frame on, `touch_materials`) and for the characters'
/// (`LampsOut::off`, `lamp_characters`).
#[allow(clippy::too_many_arguments)]
fn lamps_out(map: Res<MapLevel>, player: Res<Player>, lamps: Option<Res<LevelLamps>>, mut out: ResMut<LampsOut>,
             mut buffers: ResMut<Assets<ShaderStorageBuffer>>, mut materials: ResMut<Assets<LevelMaterial>>) {
    let Some(level) = map.0.as_ref() else { return };
    let arena = world::arena();
    if out.watch.is_none() {
        if let Some(arena) = &arena {
            let watch = level.lamps.iter().enumerate().filter(|(_, l)| l.parent != 0).filter_map(|(i, l)| {
                let object = level.objects.iter().position(|o| o.name == l.parent)?;
                Some((i, arena.breakable(object)?))
            }).collect();
            out.watch = Some(watch);
        }
    }
    let mut gone: Vec<(usize, &str)> = vec![];
    if let Some(arena) = &arena {
        gone.extend(out.watch.iter().flatten().filter(|(i, b)| !out.off.contains(i) && arena.is_gone(*b)).map(|&(i, _)| (i, "its object has gone")));
    }
    let pending = std::mem::take(&mut out.pending);
    for i in pending {
        if !out.off.contains(&i) && !gone.iter().any(|g| g.0 == i) {
            gone.push((i, "a destroyer takes it"));
        }
    }
    if !gone.is_empty() {
        if let Some(buffer) = lamps.and_then(|l| buffers.get_mut(&l.0)) {
            for &(i, _) in &gone {
                lamp_off(buffer, i);
            }
        }
        if std::env::var("BF_LINK_LOG").is_ok() {
            for &(i, why) in &gone {
                let l = &level.lamps[i];
                println!("t {:.2}: lamp h_{:08x} at {:.2} goes out: {why}", player.sim_time, l.name, l.at);
            }
        }
        out.off.extend(gone.iter().map(|g| g.0));
        out.touch = Some(TOUCH_AFTER);
    }
    if let Some(left) = out.touch {
        if left == 0 {
            let n = touch_materials(&mut materials);
            if std::env::var("BF_LINK_LOG").is_ok() {
                println!("t {:.2}: {n} level materials bound to the light buffer again", player.sim_time);
            }
            out.touch = None;
        } else {
            out.touch = Some(left - 1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// #308: a lamp a destroyer takes is queued once, and not again once it's out.
    #[test]
    fn a_lamp_is_put_out_once() {
        let mut out = LampsOut::default();
        out.put_out(3);
        out.put_out(3);
        out.put_out(5);
        assert_eq!(out.pending, vec![3, 5]);
        out.pending.clear();
        out.off.insert(5);
        out.put_out(5);
        assert!(out.pending.is_empty());
        out.put_out(7);
        assert_eq!(out.pending, vec![7]);
    }
}
