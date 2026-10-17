//! The level-over and zone triggers (#226, step 7 of #28; viewer/docs/mission.md): a child of
//! play_triggers.rs, as the hint and panel classes are. What they ask of the mission flow
//! (play_mission.rs) waits in `Flow` until its frame takes it.
//!
//! The level-over-trigger h_1f07be84 (vtable 0x395b20; parser FUN_00207f20, h_18f66101 at +0x40,
//! 1 by its constructor 0x2080e0; handler 0x207fe0; update 0x207f70):
//!  * disabled, its base actions (FUN_001ff0f0, FUN_001ff000);
//!  * enabled, each entry that answers: OPERATE sets +0x41 and, with h_18f66101, calls the game
//!    mode's +0x14 (world+0xc4c; the campaign mode's 0x89d20 sets its "over" bit: the mission's
//!    end); without it starts a 0.5 s fade to black (FUN_00069780, ECX 1) and calls the session's
//!    +0x1f4 (0x7ed30: the squad saved into the save-game, the campaign saved, "Saving
//!    Campaign..." on the status line for 1 s): the zone's end, a checkpoint (the campaign survey,
//!    todo/campaign-survey §6.1: the false level-overs fire with a zone-trigger); any other
//!    action, the base's; the entry counted (FUN_001feac0);
//!  * its update: its FIRST_UPDATE, and once the fade is done (FUN_00087d70) with +0x41 set and
//!    h_18f66101 clear, the session's +0x34 with the level request the zone-trigger left (the
//!    zone's post-movies, then the next zone); +0x41 cleared.
//!
//! The zone-trigger (vtable 0x395648; parser FUN_00207d80, h_14b0089f at +0x44; handler
//! 0x207de0): OPERATE hands the zone's file name to the session (its +0x60, +0x64, then
//! +0x5c(name, 0)), the zone the next level-over without h_18f66101 goes to.

use super::*;

/// What the level's level-over and zone triggers asked of the mission flow since its last frame
/// (play_mission.rs `Mission::frame` takes it).
#[derive(Default, Debug, Clone)]
pub(crate) struct Flow {
    /// a level-over with h_18f66101: the mission's end (the game mode's +0x14)
    pub(crate) over: bool,
    /// a level-over without it: the zone's end (the fade, the checkpoint save, the next zone)
    pub(crate) zone_end: bool,
    /// the zone the last zone-trigger named (the session's +0x5c), kept for the zone's end
    pub(crate) next_zone: Option<String>,
}

impl Triggers {
    /// 0x207fe0: a level-over-trigger (see the module notes).
    pub(super) fn level_over(&mut self, i: usize, ev: Ev) {
        if !self.live[i].enabled {
            return self.base_and_after(i, &ev);
        }
        let ClassState::LevelOver { ends_mission } = self.live[i].state else { return };
        let mut k = 0;
        while let Some(j) = self.next_entry(i, &ev, k) {
            k = j + 1;
            if self.live[i].t.entries[j].action != act::OPERATE {
                self.base_and_after(i, &ev);
                continue;
            }
            // (a multiplayer map has the deathmatch modes, not the campaign's: none of its levels
            // has a level-over, and the flow only runs in a mission)
            if ends_mission {
                self.flow.over = true;
            } else {
                self.flow.zone_end = true;
            }
            if self.log {
                println!("t {:.2}: {} OPERATE on {}: {}", self.clock, self.who(i), sig::name(ev.signal),
                         if ends_mission { "the mission is over (the game mode's +0x14)".to_string() } else {
                             format!("the zone is over: a 0.5 s fade, the checkpoint save, then {}",
                                     self.flow.next_zone.as_deref().unwrap_or("no zone (no zone-trigger has named one)"))
                         });
            }
            self.entry_used(i, j, &ev);
        }
    }

    /// 0x207de0: a zone-trigger: OPERATE names the zone the zone's end goes to.
    pub(super) fn zone(&mut self, i: usize, ev: Ev) {
        if !self.live[i].enabled {
            return self.base_and_after(i, &ev);
        }
        let ClassState::Zone { next } = &self.live[i].state else { return };
        let next = next.clone();
        let mut k = 0;
        while let Some(j) = self.next_entry(i, &ev, k) {
            k = j + 1;
            if self.live[i].t.entries[j].action != act::OPERATE {
                self.base_and_after(i, &ev);
                continue;
            }
            if self.log {
                println!("t {:.2}: {} OPERATE on {}: the next zone is {next}", self.clock, self.who(i), sig::name(ev.signal));
            }
            self.flow.next_zone = Some(next.clone());
            self.entry_used(i, j, &ev);
        }
    }

    /// What the level-over and zone triggers have asked since this was last taken (play_mission.rs);
    /// the next zone stays named.
    pub(crate) fn take_flow(&mut self) -> Flow {
        let next_zone = self.flow.next_zone.clone();
        std::mem::replace(&mut self.flow, Flow { next_zone, ..Flow::default() })
    }
}
