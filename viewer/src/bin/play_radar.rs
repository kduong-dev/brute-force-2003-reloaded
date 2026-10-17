//! The radar-triggers and the radar-contacts they give the level's objects (#228, the tutorial's
//! step 9 of #28; viewer/docs/triggers.md, "Radar-triggers", and play.md, "Radar"). A child of
//! play_triggers.rs: its class is the runtime's (its handler is `Triggers`' and dispatched from
//! there), and its system hands what it set to the world and to the radar (play_hud.rs). Written
//! from default.xbe's code (the addresses are in the comments and in viewer/docs/code-map.md); the
//! tutorial's takes confirm the timing (take07: the blip's move and its blink).
//!
//! The radar-trigger (vtable 0x395168; its parser 0x2071a0, its handler 0x207230, its constructor
//! 0x2073b0): disabled, its base actions; enabled, each entry that answers with OPERATE gives each
//! of its objects (a listed spawn-trigger's copy, +0xc4, for a spawn-trigger) that's in the world
//! its `radar-contact` (+0x40) with its time h_e2145a6c (+0x44) through FUN_0015b310, and with
//! h_12b33046 (+0x48, true unless the data says not) and a contact of 2 or 3 the game options'
//! radar sound (h_f674fc4f) is queued on the world's dialog queue (FUN_00179d60, flag 0); any other
//! action is the base's. The entry is counted after (FUN_001feac0).
//!
//! FUN_0015b310(object, value, time): the object's +0x7a = value (RCONTACT_: 0 NONE, 1 ALIGNMENT,
//! 2 OBJECTIVE, 3 OBJECTIVE_NO_CURSOR, 4 OBJECTIVE_SECONDARY, 5 INVISIBLE, 6 BOSS_ALIGNMENT, 7
//! IMPORTANT_PICKUP), kept in the world's contact list while it's 1, 2, 3, 6 or 7; its +0x10c =
//! the larger of +0x10c and the time, and while that's above 0 the object is in the world's update
//! list, whose update (0x157c40) counts +0x10c down by dt to 0. The radar blinks an objective's
//! dot while +0x10c is above 0 (FUN_000aece0, play_hud.rs).
//!
//! The tutorial's 18: nine pairs along the route (leads §7.9), each a set (3, time 2, once,
//! h_12b33046 false: no sound) and a clear (0, time 0, h_12b33046 true: no sound either, the
//! contact being 0); the first set on FIRST_UPDATE, the rest from the circles and plates as the
//! squad reaches them, the last pair on the farcast-out pad. The save points' chime that comes with
//! them is a sound-trigger's (play_sound_triggers.rs).
//!
//! Test hooks: BF_TRIGGER_LOG=1 logs each OPERATE (play_triggers.rs's log); BF_RADAR_LOG=1 each
//! contact set on an object (its name, type, place, value and time) and each name set that's in
//! the world under no object.

use super::*;

/// One radar-contact a radar-trigger set (FUN_0015b310's call at 0x207321), waiting for the world
/// (`apply`): the object by name (a level object's, or a character's trigger name: a spawn
/// copy's), the value, the time, and who set it (for the log).
pub(crate) struct ContactSet {
    object: u32,
    contact: u8,
    time: f32,
    by: String,
}

/// The radar-triggers' side of the runtime (`Triggers::radar`): the contacts they set this frame,
/// and the radar sound with its length (looked up as the map starts, `new`).
#[derive(Default)]
pub(crate) struct RadarWorld {
    sets: Vec<ContactSet>,
    /// the game options' `<radar>` sound (h_ee13e943, settings +0x15c) and its length (the
    /// sound system's +0x64, as FUN_00179d60 queues it); 0 when the banks lack it
    cue: u32,
    cue_length: f32,
}

impl RadarWorld {
    /// As a map starts: the radar sound from the game options, its length from the loaded banks.
    pub(crate) fn new(game: &Game) -> RadarWorld {
        let cue = game.radar.sound;
        RadarWorld { sets: vec![], cue, cue_length: game.sounds.length(cue).unwrap_or(0.0) }
    }
}

/// An object's radar-contact time and where it is, as the radar reads it (the object's +0x10c,
/// counted down by its update, 0x157c40).
#[derive(Clone, Copy, Debug)]
pub(crate) struct Contact {
    /// who it is, for the radar's "seen" map (FUN_000ae360 keys it by the object's +4): a level
    /// object's index, or a character's trigger name with the top bit set
    pub key: u32,
    /// where it is (its feet for a character)
    pub at: Vec3,
    /// its radar-contact (the object's +0x7a)
    pub contact: u8,
    /// the blink's time left (+0x10c, s)
    pub time: f32,
}

/// The radar's objective contacts this frame (FUN_000ae580 walks the world's contact list and hands
/// those whose +0x7a is 2, 3 or 7 to FUN_000aece0): what play_hud.rs's radar draws as yellow (or
/// the memory chip's green) dots and rim markers. The squad and the other characters are drawn
/// from their own state there.
#[derive(Resource, Default)]
pub(crate) struct RadarContacts(pub Vec<Contact>);

/// The contacts FUN_000ae580 sends to FUN_000aece0: RCONTACT_OBJECTIVE, _OBJECTIVE_NO_CURSOR and
/// _IMPORTANT_PICKUP (the RCONTACT_ table 0x3beaac).
pub(crate) const OBJECTIVE_CONTACTS: [u8; 3] = [2, 3, 7];

/// The blink times of the objects whose +0x10c a radar-trigger raised: a level object's by its
/// index, a character's by its trigger name. Reset as each map starts.
#[derive(Resource, Default)]
struct BlinkTimes {
    objects: HashMap<usize, f32>,
    characters: HashMap<u32, f32>,
}

/// The radar-triggers' world side (`apply`), which the HUD's radar draws after (play_hud.rs).
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct RadarApply;

/// The radar's frame: after the triggers' (`super::run`), before the HUD draws.
pub fn plugin(app: &mut App) {
    app.init_resource::<RadarContacts>()
        .init_resource::<BlinkTimes>()
        .add_systems(OnEnter(AppState::Playing), |mut commands: Commands| {
            commands.insert_resource(RadarContacts::default());
            commands.insert_resource(BlinkTimes::default());
        })
        .add_systems(Update, apply.in_set(RadarApply).after(super::run).run_if(in_state(AppState::Playing)));
}

impl Triggers {
    /// 0x207230: a radar-trigger. Disabled, its base actions (then FUN_001ff000). Enabled, each
    /// entry that answers: OPERATE sets its objects' contact (`ContactSet`, done by `apply`) and,
    /// with h_12b33046 and a 2 or a 3, queues the radar's sound once an object took it; any other
    /// action the base's (with FUN_001ff000). Then the entry is counted (FUN_001feac0).
    pub(super) fn radar_trigger(&mut self, i: usize, ev: Ev) {
        if !self.live[i].enabled {
            return self.base_and_after(i, &ev);
        }
        let mut k = 0;
        while let Some(j) = self.next_entry(i, &ev, k) {
            k = j + 1;
            if self.live[i].t.entries[j].action != act::OPERATE {
                self.base_and_after(i, &ev);
                continue;
            }
            let ClassState::Radar { contact, time, cue } = self.live[i].state else { return };
            let by = self.label(self.live[i].t.name);
            let mut set_any = false;
            for o in self.live[i].t.objects.clone() {
                // (a listed trigger: a spawn-trigger's copy, 0x2072e5-0x207304; any other
                // trigger isn't an object and is skipped)
                let object = match self.by_name.get(&o).map(|&n| &self.live[n].state) {
                    Some(ClassState::Spawn { copy, .. }) => *copy,
                    Some(_) => 0,
                    None => o,
                };
                if object == 0 {
                    continue;
                }
                set_any = true;
                self.radar.sets.push(ContactSet { object, contact, time, by: by.clone() });
            }
            if self.log {
                println!("t {:.2}: {} OPERATE on {}: radar-contact {contact}, time {time} to {}", self.clock, self.who(i), sig::name(ev.signal),
                         self.live[i].t.objects.iter().map(|&o| self.label(o)).collect::<Vec<_>>().join(", "));
            }
            // the radar's sound on the dialog queue (0x207343-0x20736a; the flag 0: no hush)
            if set_any && cue && (contact == 2 || contact == 3) && self.radar.cue != 0 {
                let (sound, length) = (self.radar.cue, self.radar.cue_length);
                self.queue_line(sound, length, &by);
            }
            self.entry_used(i, j, &ev);
        }
    }
}

/// The world's side of the radar-triggers, each frame after the triggers' (FUN_0015b310 on each
/// object they named, then each object's update, 0x157c40): a level object takes the contact (its
/// `Placement::radar_contact`, which the crosshair reads too) and a character its own
/// (`Player::radar_contact`); the larger time is kept and counted down by dt to 0. Then the radar's
/// objective contacts (2, 3 and 7: the level's objects and the characters) are handed to the HUD
/// (`RadarContacts`).
#[allow(clippy::too_many_arguments)]
fn apply(time: Res<Time>, mut tr: ResMut<Triggers>, mut map: ResMut<MapLevel>, mut player: ResMut<Player>, mut squad: ResMut<Squad>,
         mut blinks: ResMut<BlinkTimes>, mut contacts: ResMut<RadarContacts>) {
    let dt = frame_dt(&time);
    let log = std::env::var("BF_RADAR_LOG").is_ok();
    let now = player.sim_time;
    let sets = std::mem::take(&mut tr.radar.sets);
    // the objects' updates first (this frame's dt), then the new times on top: FUN_0015b310 keeps
    // the larger, so a time set now is whole on this frame's radar
    let BlinkTimes { objects, characters } = &mut *blinks;
    for t in objects.values_mut().chain(characters.values_mut()) {
        *t = (*t - dt).max(0.0);
    }
    objects.retain(|_, t| *t > 0.0);
    characters.retain(|_, t| *t > 0.0);
    for s in sets {
        let mut found = false;
        for (k, o) in map.0.iter_mut().flat_map(|l| l.objects.iter_mut()).enumerate().filter(|(_, o)| o.name == s.object) {
            o.radar_contact = s.contact;
            found = true;
            if s.time > 0.0 {
                let t = blinks.objects.entry(k).or_insert(0.0);
                *t = t.max(s.time);
            }
            if log {
                println!("t {now:.2}: {} sets radar-contact {} (time {}) on h_{:08x} (type h_{:08x}) at {:.1}", s.by, s.contact, s.time, o.name, o.kind,
                         o.transform.w_axis.truncate());
            }
        }
        for u in std::iter::once(&mut *player).chain(squad.0.iter_mut()).filter(|u| u.trigger_name == s.object && !u.absent) {
            u.radar_contact = s.contact;
            found = true;
            if s.time > 0.0 {
                let t = blinks.characters.entry(s.object).or_insert(0.0);
                *t = t.max(s.time);
            }
            if log {
                println!("t {now:.2}: {} sets radar-contact {} (time {}) on the character {}", s.by, s.contact, s.time, unit_name(u));
            }
        }
        if !found && log {
            println!("t {now:.2}: {}: nothing called h_{:08x} in the world to take radar-contact {}", s.by, s.object, s.contact);
        }
    }
    // the radar's objective contacts (FUN_000ae580: 2, 3 and 7 to FUN_000aece0)
    contacts.0.clear();
    for (k, o) in map.0.iter().flat_map(|l| l.objects.iter()).enumerate().filter(|(_, o)| OBJECTIVE_CONTACTS.contains(&o.radar_contact)) {
        contacts.0.push(Contact { key: k as u32, at: o.transform.w_axis.truncate(), contact: o.radar_contact,
                                  time: blinks.objects.get(&k).copied().unwrap_or(0.0) });
    }
    for u in std::iter::once(&*player).chain(squad.0.iter()).filter(|u| !u.dead && !u.absent && !u.stand_in && OBJECTIVE_CONTACTS.contains(&u.radar_contact)) {
        let key = 0x8000_0000 | u.trigger_name;
        contacts.0.push(Contact { key, at: u.position + Vec3::Y * GROUND, contact: u.radar_contact,
                                  time: blinks.characters.get(&u.trigger_name).copied().unwrap_or(0.0) });
    }
}
