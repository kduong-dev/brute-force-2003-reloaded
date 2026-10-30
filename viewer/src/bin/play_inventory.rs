//! The squad's inventory at a level's start (#281): the ammo, grenades, medkits and other items
//! its members' placements carry, pooled into the squad's one store. Read from default.xbe (each
//! function is cited where it's used, and has a row in viewer/docs/code-map.md) and checked
//! against xemu's reads of the store at fresh starts (todo/196-inventory-hud notes, "Starting
//! inventories"); the audit is todo/inventory-audit/answer.md.
//!
//!  - **A member's own inventory.** The character's property handler (0x112eb0, its `inventory`
//!    case) gives the placement's `<inventory>` to FUN_0014dd50: its two `<weapon>`s into the
//!    slots (play_triggers.rs `SquadSpawn::dress`), then each `<h_10dd6db2 name count>` through
//!    FUN_0014ce90, which puts an item of class-type 2 or 3 (every one the squad's placements
//!    carry) into the character's own store by FUN_0014bfe0: the stack of the type's (category,
//!    index) (its group-type and h_e5f51266) takes the type if it had none and the count, held to
//!    the type's stack-limit (type+0x180).
//!  - **The squad's store.** A member joining the squad (FUN_0005b9a0) is given the squad's store
//!    by FUN_0014cb60, its own merged into it first (FUN_0014c060: every typed stack's count added
//!    through FUN_0014bfe0, the same clamp). So there's one store for the squad: every member's
//!    inventory front points to it (xemu's reads on e34, e40 and the tutorial), and every
//!    member's item box shows the same counts. A fresh campaign's mode set-up clears the store and
//!    refills it from the save (FUN_00089ea0), which holds nothing yet; carrying the store from
//!    zone to zone is #231's.
//!  - **The demo's pieces of it.** The store is built once a level is loaded (`StartStore`, in
//!    the store's (category, index) order) and handed out: the ammo (category 1, IG_AMMO) is the
//!    reserve (play_ammo.rs, by ammo-type: the index); the grenades and explosives (2, 3) are
//!    `Player::grenades`, by the kit of the same label (play_grenade.rs's kits hold one definition
//!    per label; the placements carry the Sentry as h_e5f1f063 on m07_a, m14_a and m14_b, as
//!    h_f73de83d on m14_c and m14_d); the Medkit (5, 0: the 80-point h_192d5337, the black
//!    button's stack, 0x4a8ce4) is `Player::medkits`; the rest (the OrgSen (6, 0), the Hat (6, 3),
//!    "Alien Technology Acquired!" (8, 28) on m14_a / m14_b) are `Player::others`, listed in the
//!    item box as their type's h_1852ab6c says (the OrgSen and the Hat yes, "Alien Technology"
//!    no). Those move with control as the rest of the inventory does (play.rs `squad_control`).
//!  - **Where no placement starts the squad** (the test map's floor, the deathmatch maps) the
//!    demo keeps its own stock: the test map a full stack of every grenade type, the deathmatch
//!    maps START_GRENADES Frags (their sdm types' loadout is #144's), and the reserve full stacks
//!    of every ammo-type (play_ammo.rs).
//!  - **What the items do.** The grenades and the Medkits as before (play_grenade.rs,
//!    play_pickups.rs: the 60-point (5, 2) one too, an `Other` that heals). The OrgSen's use is
//!    play_vision.rs's (#21: put on with its clip, taken off at the next press); the Hat's is
//!    its own ticket's: G with it selected does nothing yet. A character whose skill refuses a use
//!    type (Brutus: the OrgSen's IOU_ATTACH_TO_FACE, the Hat's IOU_ATTACH_TO_EQUIP_SLOT, his
//!    skill's +0x1c 0x13c160) is told "Cannot use" (h_e784869e; #196 take26) and his box shows it
//!    red. The squad picks up more of an item by walking over it (`put_other`,
//!    play_item_pickups.rs: the OrgSen on sdm_e13).
//!  - **The first item shown** (#196): the first listable stack in the store's (category, index)
//!    order (the front's selection starts at none and FUN_0014db60 steps it to the first): e34's
//!    Sonic 4, e40's Frag 5, the tutorial's Frag 3; its NEW flag cleared, every other stack's set
//!    (play_itembox.rs `ItemBox::start`). The rest of the selection (the B button, the list, NEW,
//!    a stack running out) is play_itembox.rs's.
//!
//! Test hooks: BF_INVENTORY_LOG=1 prints the store at the start (each member's items, the pooled
//! stacks and where each goes) and each use of an item that isn't a grenade or a Medkit;
//! BF_TEST_ITEM=<label> selects the carried item of that name (a grenade type, Medkit, OrgSen...)
//! at the start; BF_TEST_INV=<name>:<n>,... sets the pool's counts at the start (a grenade type's
//! label, Medkit the (5, 0) stack, Medkit60 the (5, 2) one, another listed item's label: OrgSen,
//! Hat; 0 empties the stack; the rest keep the store's); BF_TEST_INV_NEW=<name>,... the stacks
//! still NEW at the start (play_itembox.rs).

use super::grenade::GrenadeKits;
use super::*;

/// IG_AMMO (the IG_ name table 0x3be590): the ammo's category, never listed in the item box
/// (FUN_0014d680); its stacks are the reserve (play_ammo.rs), the index the ammo-type.
pub const IG_AMMO: i64 = 1;
/// IG_GRENADE and IG_EXPLOSIVE: the thrown grenades (Frag, Gas, Sonic, Energy, Light) and the
/// placed ones (Sentry, Roller), the demo's grenade kits (play_grenade.rs).
const IG_GRENADE: i64 = 2;
const IG_EXPLOSIVE: i64 = 3;
/// IG_HEAL and the index of the Medkit (80) in it: the stack the black button uses ({5, 0} at
/// 0x4a8ce4, todo/196 leads §2.3), `Player::medkits`.
const IG_HEAL: i64 = 5;
const MEDKIT_INDEX: i64 = 0;
/// The Frag (h_060f95f7): the deathmatch maps' start stock (START_GRENADES of them).
const FRAG: u32 = 0x060F_95F7;
/// Grenades carried at the start of a map whose squad no placement starts (the deathmatch maps:
/// the sdm types' loadout carries none, #144): the demo's 3 Frags, as the tutorial's capture showed
/// before #281 (its placement's own Frag 3).
const START_GRENADES: i64 = 3;
/// "Cannot use" (h_e784869e in the string table): what Brutus's skill (+0x1c, 0x13c160) posts
/// for an item whose use type it refuses (#196 take26: Brutus with the OrgSen, LT pressed).
const S_CANNOT_USE: u32 = 0xE784_869E;
/// IFSET_GENERIC_HEALING (the IFSET_ table 0x3be3cc): an item type's function-type that heals as
/// a Medkit does (both Medkits, h_192d5337 and h_f5123ace; play_pickups.rs's MEDKIT).
const IFSET_GENERIC_HEALING: i64 = 5;
/// The 60-point Medkit's stack (h_f5123ace: group-type 5, h_e5f51266 2), BF_TEST_INV's
/// "Medkit60": the second Medkit of #196 take01's order.
const MEDKIT60: (i64, i64) = (5, 2);

/// The store built as a map starts (`start`); the use of an item that isn't a grenade or a
/// Medkit after the frame's step. (`stock` runs with the grenades' systems, play_grenade.rs, once
/// their kits are loaded.)
pub fn plugin(app: &mut App) {
    app.init_resource::<StartStore>()
        .add_systems(OnEnter(AppState::Playing), start.after(setup))
        .add_systems(Update, use_other.after(update_player).before(play_sounds).run_if(in_state(AppState::Playing)));
}

/// An item type as the store sees it, from its item data (the block at type+0x160, which a
/// stack's vtable +0x1c returns, 0x14ad70): its stack's key (category = group-type at
/// [type+0x194], index = h_e5f51266 at [type+0x198]: FUN_00149d30 reads them; the block's +0x34 /
/// +0x38, the offsets the #196 leads give), its stack-limit ([type+0x180], FUN_0014bfe0's clamp),
/// whether the item box lists it (h_1852ab6c, the block's byte +0x71, type+0x1d1: FUN_0014d680),
/// its use type (h_1ee2f4ed, the IOU_ enum, [type+0x174]) and its name (stringtable-name).
#[derive(Clone, Debug, Default)]
pub struct ItemInfo {
    pub group: i64,
    pub index: i64,
    pub limit: i64,
    pub listed: bool,
    pub use_type: i64,
    pub label: String,
}

/// Item type `kind` (FUN_00160da0 finds a type by name): an `<inventory>` entry (`Game::items`:
/// the ammo, the medkits, the OrgSen, the Hat ...) or an `<inventory-grenade>` one (read with the
/// weapons, `Game::weapons`, group-type 2 or 3: every grenade's h_1852ab6c is true in the data).
/// None for a name no loaded objecttypes defines (FUN_0014dd50 skips it).
pub fn item_info(game: &Game, kind: u32) -> Option<ItemInfo> {
    if let Some(i) = game.items.get(&kind) {
        return Some(ItemInfo { group: i.group, index: i.kind, limit: i.stack_limit, listed: i.selectable, use_type: i.use_type, label: i.label.clone() });
    }
    game.weapons.get(&kind).filter(|d| matches!(d.group_type, IG_GRENADE | IG_EXPLOSIVE)).map(|d| ItemInfo {
        group: d.group_type, index: d.item_index, limit: d.stack_limit, listed: true, use_type: d.use_type, label: d.label.clone(),
    })
}

/// One stack of a store: its key (category, index), the item type it holds (the first one added
/// to it, FUN_0014bfe0) and how many.
#[derive(Clone, Debug, PartialEq)]
pub struct Stack {
    pub group: i64,
    pub index: i64,
    pub kind: u32,
    pub count: i64,
}

/// FUN_0014bfe0: `n` of item type `kind` added to `store`: the stack of its (category, index)
/// (FUN_0014bee0 finds it; a new one here, kept in the store's order) takes the type if it had
/// none, and the count goes up by n, held to the type's stack-limit (`n = limit - count` where
/// `count + n` would pass it: so nothing at a limit of 0). Returns what was added. (FUN_00149d30
/// sends a type whose (category, index) another type holds to a stack of its own in category 0;
/// no two types a zone's squad placements carry share a key, so that isn't modelled.)
fn add(store: &mut Vec<Stack>, info: &ItemInfo, kind: u32, n: i64) -> i64 {
    let at = match store.iter().position(|s| (s.group, s.index) == (info.group, info.index)) {
        Some(at) => at,
        None => {
            let at = store.iter().position(|s| (s.group, s.index) > (info.group, info.index)).unwrap_or(store.len());
            store.insert(at, Stack { group: info.group, index: info.index, kind, count: 0 });
            at
        }
    };
    let s = &mut store[at];
    let n = n.min(info.limit - s.count).max(0);
    s.count += n;
    n
}

/// The squad's store from its members' inventories (each a placement's `<h_10dd6db2 name
/// count>` list): each member's own store first (FUN_0014dd50 -> FUN_0014ce90 -> FUN_0014bfe0, the
/// clamp per stack), merged into the squad's as it joins (FUN_0014c060: each typed stack through
/// FUN_0014bfe0 again), in the store's (category, index) order. `info` finds a type
/// (`item_info`); a name it doesn't know is skipped, and so is a count under 1 (a stack whose
/// count goes to 0 loses its type, FUN_0014aae0).
pub fn pool<'a>(members: impl IntoIterator<Item = &'a [(u32, i64)]>, info: impl Fn(u32) -> Option<ItemInfo>) -> Vec<Stack> {
    let mut squad: Vec<Stack> = vec![];
    for items in members {
        let mut own: Vec<Stack> = vec![];
        for &(kind, n) in items {
            if let (Some(i), true) = (info(kind), n > 0) {
                add(&mut own, &i, kind, n);
            }
        }
        for s in own.iter().filter(|s| s.count > 0) {
            if let Some(i) = info(s.kind) {
                add(&mut squad, &i, s.kind, s.count);
            }
        }
    }
    squad.retain(|s| s.count > 0);
    squad
}

/// The squad's store at the level's start (`pool` over the level's squad spawns, play_triggers.rs
/// `SquadSpawns`: each spawn-trigger's template, and each member placed in the world), built as the
/// map starts; `stock` and play_ammo.rs hand it out.
#[derive(Resource, Default)]
pub struct StartStore {
    /// its stacks, in the store's (category, index) order
    pub stacks: Vec<Stack>,
    /// the level starts its squad from placements, whose inventories are the start (every
    /// campaign zone); false on a map without (the test map's floor, the deathmatch maps), where
    /// the demo stocks its own
    pub placed: bool,
}

impl StartStore {
    /// The rounds of each ammo-type the squad starts with: the IG_AMMO stacks, by index.
    pub fn ammo(&self) -> impl Iterator<Item = (i64, i64)> + '_ {
        self.stacks.iter().filter(|s| s.group == IG_AMMO).map(|s| (s.index, s.count))
    }

    /// How many of grenade kit `kit` the squad starts with: its stacks' (category 2 or 3) whose
    /// type has the kit's label (one in the data per zone), held to the kit's stack-limit. (Each
    /// stack is held to its own type's already; the kit's limit only matters were two types of a
    /// label carried together, which no zone does, and a kit without one isn't held.)
    fn grenades(&self, game: &Game, kit: &WeaponDef) -> i64 {
        let n: i64 = self.stacks.iter().filter(|s| matches!(s.group, IG_GRENADE | IG_EXPLOSIVE))
            .filter(|s| item_info(game, s.kind).is_some_and(|i| i.label.eq_ignore_ascii_case(&kit.label))).map(|s| s.count).sum();
        if kit.stack_limit > 0 { n.min(kit.stack_limit) } else { n }
    }
}

/// An item of the squad's store that isn't ammo, a grenade or the Medkit (5, 0) (`Player::others`,
/// in the store's (category, index) order): the 60-point Medkit (5, 2), the OrgSen, the Hat,
/// "Alien Technology Acquired!". The OrgSen's use is play_vision.rs's (#21); the Hat's isn't in.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Carried {
    /// its item type (a `Game::items` key)
    pub kind: u32,
    pub count: i64,
    /// the item box lists it: its type's h_1852ab6c (FUN_0014d680)
    pub listed: bool,
    /// its type's use type (h_1ee2f4ed): whose skill takes it (play_skills.rs `uses_item`)
    pub use_type: i64,
    /// it heals as a Medkit does (its function-type IFSET_GENERIC_HEALING: the 60-point Medkit),
    /// used as one (play_pickups.rs `use_medkit`) and red at full health
    pub heals: bool,
}

impl Carried {
    /// The item of type `kind` (`count` of it), as the store's item data has it.
    pub fn of(game: &Game, kind: u32, count: i64) -> Option<Carried> {
        let i = item_info(game, kind)?;
        let heals = game.items.get(&kind).is_some_and(|t| t.function == IFSET_GENERIC_HEALING);
        Some(Carried { kind, count, listed: i.listed, use_type: i.use_type, heals })
    }
}

/// The map starts: the squad's store from its placements (`StartStore`), logged with
/// BF_INVENTORY_LOG.
fn start(mut commands: Commands, game: Res<GameData>, spawns: Option<Res<super::triggers::SquadSpawns>>) {
    let none = super::triggers::SquadSpawns::default();
    let spawns = spawns.as_deref().unwrap_or(&none);
    let stacks = pool(spawns.0.iter().map(|s| &s.items[..]), |k| item_info(&game.0, k));
    if std::env::var("BF_INVENTORY_LOG").is_ok() {
        let name = |k: u32| item_info(&game.0, k).map_or(format!("h_{k:08x}"), |i| i.label);
        for s in &spawns.0 {
            println!("inventory: {}'s placement carries {}", CHARACTERS[s.slot],
                     s.items.iter().map(|&(k, n)| format!("{} {n}", name(k))).collect::<Vec<_>>().join(", "));
        }
        println!("inventory: the squad's store at the start ({} stacks{}): {}", stacks.len(),
                 if spawns.0.is_empty() { "; no squad placement: the demo's own stock" } else { "" },
                 stacks.iter().map(|s| format!("({}, {}) {} {}", s.group, s.index, name(s.kind), s.count)).collect::<Vec<_>>().join(", "));
    }
    commands.insert_resource(StartStore { stacks, placed: !spawns.0.is_empty() });
}

/// The squad's items, once the controlled character is loaded: its grenades (again after a
/// respawn, play.rs `spawn_unit`, which empties them: a respawn starts with the start's), and once
/// per squad its Medkits and other items (`Player::stocked`: a respawn keeps the Medkits taken). From the store where placements start the squad (`StartStore`); on the test
/// map a full stack (stack-limit) of every grenade type; on a map without placements
/// START_GRENADES Frags (or of its first type without the Frag). The first time, the item box's
/// selection: the first listable stack in the store's order (the front's starts at none and
/// FUN_0014db60 steps it to the first: #196 AC-04), its NEW flag cleared and every other stack's
/// set (play_itembox.rs `ItemBox::start`, AC-22).
/// Test hooks: BF_TEST_GRENADE_TYPE=<label> (e.g. Sonic) selects that type, giving a stack of it if
/// none is carried; BF_TEST_ITEM=<label> selects the carried item of that name; BF_TEST_INV and
/// BF_TEST_INV_NEW set the pool and its NEW flags (the module notes).
#[allow(clippy::too_many_arguments)]
pub fn stock(mut player: ResMut<Player>, kits: Option<Res<GrenadeKits>>, test: Option<Res<super::testmap::TestMap>>, store: Res<StartStore>,
             game: Res<GameData>, mut boxed: ResMut<super::itembox::ItemBox>) {
    let Some(kits) = kits else { return };
    if player.loaded.is_none() || (player.stocked && player.grenades.len() == kits.0.len()) {
        return;
    }
    let p = &mut *player;
    let first_stock = !p.stocked;
    p.grenades = kits.0.iter().map(|k| match (test.is_some(), store.placed) {
        (true, _) => k.def.stack_limit.max(1),
        (false, true) => store.grenades(&game.0, &k.def),
        (false, false) => if k.def.name == FRAG { START_GRENADES } else { 0 },
    }).collect();
    // (a deathmatch map without the Frag: its first type)
    if test.is_none() && !store.placed && !p.grenades.iter().any(|&n| n > 0) {
        if let Some(n) = p.grenades.first_mut() {
            *n = START_GRENADES;
        }
    }
    let log = std::env::var("BF_INVENTORY_LOG").is_ok();
    if !p.stocked {
        p.stocked = true;
        p.medkits = 0;
        p.others.clear();
        for s in &store.stacks {
            let Some(i) = item_info(&game.0, s.kind) else { continue };
            match (s.group, s.index) {
                (IG_AMMO, _) => {}
                (IG_GRENADE | IG_EXPLOSIVE, _) => {
                    if log && !kits.0.iter().any(|k| k.def.label.eq_ignore_ascii_case(&i.label)) {
                        println!("inventory: {} {}: no grenade type of that name loaded: not carried", i.label, s.count);
                    }
                }
                (IG_HEAL, MEDKIT_INDEX) => {
                    p.medkits = s.count;
                    p.medkit_kind = s.kind;
                }
                _ => {
                    if let Some(c) = Carried::of(&game.0, s.kind, s.count) {
                        p.others.push(c);
                    }
                }
            }
        }
        if let Ok(v) = std::env::var("BF_TEST_INV") {
            test_inventory(p, &game.0, &kits, &v, log);
        }
    }
    // the item box's first item (a respawn keeps the selection: play_itembox.rs moves it on if it
    // ran out)
    if first_stock {
        p.item = super::itembox::first(p);
    }
    if let Some(k) = std::env::var("BF_TEST_GRENADE_TYPE").ok().and_then(|l| kits.find(&l)) {
        if p.grenades[k] <= 0 {
            p.grenades[k] = kits.0[k].def.stack_limit.max(1);
        }
        p.item = Some(Item::Grenade(k));
    }
    // BF_TEST_ITEM=<label>: the carried item of that name selected
    if let Ok(want) = std::env::var("BF_TEST_ITEM") {
        if let Some(i) = named(p, &game.0, &kits, want.trim()).filter(|&i| item_count(p, i) > 0) {
            p.item = Some(i);
        }
    }
    if first_stock {
        let all = items(p);
        let by_name = |n: &str| named(p, &game.0, &kits, n);
        boxed.start(&all, p.item, by_name);
    }
    if log {
        let names: Vec<String> = kits.0.iter().zip(&p.grenades).filter(|(_, &n)| n > 0).map(|(k, n)| format!("{} {n}", k.def.label)).collect();
        println!("inventory: {}: grenades [{}], medkits {} ({}), others [{}]; the item box shows {}", unit_name(p), names.join(", "), p.medkits,
                 item_info(&game.0, p.medkit_kind).map_or("-".to_string(), |i| i.label),
                 p.others.iter().map(|c| format!("{} {}{}", item_info(&game.0, c.kind).map_or(format!("h_{:08x}", c.kind), |i| i.label), c.count,
                                                 if c.listed { "" } else { " (not listed)" })).collect::<Vec<_>>().join(", "),
                 p.item.map_or("nothing".to_string(), |i| item_label(&game.0, &kits, p, i)));
    }
}

/// The squad's other items as a pickup fills them (the walk-over's FUN_0014ce90 -> FUN_0014bfe0,
/// play_item_pickups.rs): `n` of item type `kind` into its stack held to the type's stack-limit
/// ([type+0x180]), the stack made in the store's (category, index) order when the squad has none
/// (the OrgSen's (6, 0) after the Medkit (5, 2)); how many went in (0 for a stack at its limit, or
/// an item type the store doesn't know).
pub fn put_other(p: &mut Player, game: &Game, kind: u32, n: i64) -> i64 {
    let Some(info) = item_info(game, kind) else { return 0 };
    let key = |k: u32| item_info(game, k).map_or((i64::MAX, 0), |i| (i.group, i.index));
    match p.others.iter().position(|c| c.kind == kind) {
        Some(at) => {
            let added = n.min(info.limit - p.others[at].count).max(0);
            p.others[at].count += added;
            added
        }
        None => {
            let added = n.min(info.limit).max(0);
            if let (true, Some(c)) = (added > 0, Carried::of(game, kind, added)) {
                let at = p.others.iter().position(|o| key(o.kind) > key(kind)).unwrap_or(p.others.len());
                p.others.insert(at, c);
            }
            added
        }
    }
}

/// The squad's stack a test hook names (BF_TEST_ITEM, BF_TEST_INV_NEW): a grenade type's label,
/// Medkit (the (5, 0) stack), Medkit60 (the (5, 2) one), another listed item's label (OrgSen,
/// Hat), any case.
fn named(p: &Player, game: &Game, kits: &GrenadeKits, name: &str) -> Option<Item> {
    if let Some(k) = kits.find(name) {
        return Some(Item::Grenade(k));
    }
    if name.eq_ignore_ascii_case("medkit") {
        return Some(Item::Medkit);
    }
    let medkit60 = name.eq_ignore_ascii_case("medkit60");
    p.others.iter().map(|c| c.kind).find(|&k| item_info(game, k).is_some_and(|i| if medkit60 { (i.group, i.index) == MEDKIT60 } else { i.label.eq_ignore_ascii_case(name) }))
        .map(Item::Other)
}

/// The item type of the first (by name) listed type of `game` whose item data `want` takes.
fn type_where(game: &Game, want: impl Fn(&ItemInfo) -> bool) -> Option<u32> {
    let mut kinds: Vec<u32> = game.items.keys().copied().filter(|&k| item_info(game, k).is_some_and(|i| i.listed && want(&i))).collect();
    // (a carried type before the pickup that gives it: the OrgSen h_11433446, which the squad
    // carries, before h_05a87396, which lies in the world and names it as its pickup-archetype)
    kinds.sort_by_key(|k| (game.items.get(k).is_some_and(|i| i.gives != 0), *k));
    kinds.first().copied()
}

/// BF_TEST_INV=<name>:<n>,...: the pool's counts at the start (the module notes): a grenade type
/// by its label, Medkit the (5, 0) stack, Medkit60 the (5, 2) one, another listed item type by its
/// label; an item not carried yet is added in the store's (category, index) order. #196's
/// nine-stack snapshot: Frag:5,Gas:5,Sonic:4,Energy:5,Sentry:3,Roller:4,Medkit:2,Medkit60:2,OrgSen:1.
fn test_inventory(p: &mut Player, game: &Game, kits: &GrenadeKits, spec: &str, log: bool) {
    for entry in spec.split(',') {
        let Some((name, n)) = entry.split_once(':') else { continue };
        let (name, Ok(n)) = (name.trim(), n.trim().parse::<i64>()) else { continue };
        let n = n.max(0);
        if let Some(k) = kits.find(name) {
            p.grenades[k] = n;
        } else if name.eq_ignore_ascii_case("medkit") {
            p.medkits = n;
            if p.medkit_kind == 0 {
                p.medkit_kind = type_where(game, |i| (i.group, i.index) == (IG_HEAL, MEDKIT_INDEX)).unwrap_or(0);
            }
        } else {
            let medkit60 = name.eq_ignore_ascii_case("medkit60");
            let Some(kind) = type_where(game, |i| if medkit60 { (i.group, i.index) == MEDKIT60 } else { i.label.eq_ignore_ascii_case(name) }) else {
                println!("inventory: BF_TEST_INV: no listed item type {name}");
                continue;
            };
            match p.others.iter_mut().find(|c| c.kind == kind) {
                Some(c) => c.count = n,
                None => {
                    let Some(c) = Carried::of(game, kind, n) else { continue };
                    let key = |k: u32| item_info(game, k).map_or((i64::MAX, 0), |i| (i.group, i.index));
                    let at = p.others.iter().position(|o| key(o.kind) > key(kind)).unwrap_or(p.others.len());
                    p.others.insert(at, c);
                }
            }
        }
        if log {
            println!("inventory: BF_TEST_INV: {name} {n}");
        }
    }
}

/// An item's name in the item box (its type's stringtable-name): a grenade type's label, the
/// carried Medkit's, another item's.
pub fn item_label(game: &Game, kits: &GrenadeKits, p: &Player, item: Item) -> String {
    match item {
        Item::Grenade(k) => kits.0.get(k).map_or_else(String::new, |g| g.def.label.clone()),
        Item::Medkit => game.items.get(&p.medkit_kind).map(|t| t.label.clone()).filter(|s| !s.is_empty()).unwrap_or("Medkit".into()),
        Item::Other(kind) => game.items.get(&kind).map_or_else(|| format!("h_{kind:08x}"), |t| t.label.clone()),
    }
}

/// G (the game's LT, the use) with an item selected that isn't a grenade or a Medkit (the
/// OrgSen, the Hat; a Medkit is play_pickups.rs's). A character whose skill refuses its use type
/// (the skill's +0x1c: Brutus's 0x13c160, IOU_ATTACH_TO_FACE and IOU_ATTACH_TO_EQUIP_SLOT) is told
/// "Cannot use" (h_e784869e: his skill only writes it into the HUD entry's message, +0xe4; the
/// controller shows it at the press when "can use now" says no, FUN_000bcfe0(entry, 3.0) at
/// 0x13496a) for the item messages' 3 s from each press (play_itembox.rs MESSAGE_TIME; #196
/// take26: on through presses 3 s apart, its ink gone 2.8 s of the game's clock after the last,
/// as the demo's 2.87 s); for anyone else the use is the item's own: the OrgSen's is
/// play_vision.rs's (#21), the Hat's isn't in yet.
pub fn use_other(player: Res<Player>, mut status: ResMut<UsePanel>, game: Res<GameData>) {
    let Some(Item::Other(kind)) = player.item else { return };
    if !player.item_use || player.dead {
        return;
    }
    let Some(c) = player.others.iter().find(|c| c.kind == kind && c.count > 0 && !c.heals).copied() else { return };
    let log = std::env::var("BF_INVENTORY_LOG").is_ok();
    let label = game.0.items.get(&kind).map_or_else(|| format!("h_{kind:08x}"), |t| t.label.clone());
    if !super::skills::uses_item(&player, c.use_type) {
        let text = game.0.strings.get(&S_CANNOT_USE).cloned().unwrap_or("Cannot use".into());
        if log {
            println!("t={:.2} {}: {label} (use type {}): \"{text}\" (the skill's +0x1c)", player.sim_time, unit_name(&player), c.use_type);
        }
        status.message = Some((text, super::itembox::MESSAGE_TIME));
    } else if log {
        println!("t={:.2} {}: {label} (use type {}): the character can use it ({})", player.sim_time, unit_name(&player), c.use_type,
                 if c.use_type == super::vision::IOU_ATTACH_TO_FACE { "play_vision.rs puts it on or takes it off" } else { "its use isn't in yet: the Hat's" });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The game's records of the types used below (e34's objecttypes): (kind, group, index, limit).
    fn info(kind: u32) -> Option<ItemInfo> {
        let (group, index, limit) = match kind {
            0xEF0C_4C1E => (1, 3, 200),   // Sonic Ammo
            0x1230_2D03 => (1, 1, 50),    // 11mm Ammo
            0xEC0B_F28F => (2, 4, 10),    // Sonic
            0x060F_95F7 => (2, 0, 10),    // Frag
            0x192D_5337 => (5, 0, 25),    // Medkit
            0x1143_3446 => (6, 0, 1),     // OrgSen
            _ => return None,
        };
        Some(ItemInfo { group, index, limit, listed: true, use_type: 0, label: String::new() })
    }

    /// m09_a's Sonic Ammo: Brutus 200 + Flint 120 + Tex 160, one stack held to its 200; e34's
    /// Medkit 6 and OrgSen 1 kept; m05_a's Frags past their 10 held; the store in (category,
    /// index) order; an unknown name and a count of 0 left out.
    #[test]
    fn pools_and_clamps() {
        let brutus = [(0xEF0C_4C1E, 200), (0x060F_95F7, 8)];
        let flint = [(0xEF0C_4C1E, 120), (0x1230_2D03, 40), (0x1234_5678, 3)];
        let tex = [(0xEF0C_4C1E, 160), (0x060F_95F7, 7), (0x192D_5337, 6), (0x1143_3446, 1), (0xEC0B_F28F, 0)];
        let store = pool([&brutus[..], &flint[..], &tex[..]], info);
        let got: Vec<(i64, i64, i64)> = store.iter().map(|s| (s.group, s.index, s.count)).collect();
        assert_eq!(got, vec![(1, 1, 40), (1, 3, 200), (2, 0, 10), (5, 0, 6), (6, 0, 1)]);
    }

    /// A member's own store clamps first (FUN_0014dd50's adds): two entries of one type on one
    /// placement held to the limit before the merge.
    #[test]
    fn clamps_each_member_first() {
        let one = [(0x1143_3446, 1), (0x1143_3446, 1)];
        let two = [(0x1143_3446, 1)];
        let store = pool([&one[..], &two[..]], info);
        assert_eq!(store, vec![Stack { group: 6, index: 0, kind: 0x1143_3446, count: 1 }]);
    }
}
