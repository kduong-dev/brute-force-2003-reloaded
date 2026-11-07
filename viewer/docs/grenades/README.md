# Grenades (`src/bin/play_grenade.rs`)

*This folder was the page `grenades.md`, split into one file per section (#393): the sections are listed under **Sections** at the end of this page, and `../grenades.md` is now a pointer to them.*

Every grenade type comes from the loaded levels' `inventory-grenade` definitions (read into
`WeaponDef`, `src/bf/weapon.rs`); nothing per type is hard-coded except which definitions the
squad carries when several share a label (the Sentry has three, the Roller two: the ones the
recordings show, `SQUAD_GRENADES`; a level that defines only another one its squad's placements
carry, or that it puts down itself, takes that one: m14_c's and m14_d's Sentry h_f73de83d, and the
same Sentry that e16, e34 and e40 lay as live mines, see **Thermo**) and their order, which Tab
steps through and the item box shows first: the game's store order, (category, index) = the
group-type and h_e5f51266, Frag, Gas, Sonic, Energy, Light, Sentry, Roller (#281,
[play.md](../play/items.md) "Items: the squad's starting inventory"; BF_TEST_INV's grenade types the level
defines but the squad doesn't carry are loaded too, #196). The squad carries only those seven. The
Molotov (e01's h_f42e0faa, a labelled grenade with an icon) is the mutants' weapon (#78), not the
squad's: its definition is read like the others' for an AI thrower to use (`BF_TEST_INV=Molotov:5
BF_TEST_GRENADE_TYPE=Molotov BF_TEST_THROW=2.0,0.5` throws one for a look, #373: its explosion's
effect type h_1a256542 carries a burning cloud of its own, 78 FLAME a second for 1 s within 7 m,
which burns beside the bottle's one-off Damage, [effects.md](../effects.md)). Per type:

| Field | Where | Frag |
|---|---|---|
| fuse | `timer`, counted from the first ground contact; 0 goes off on the first contact, no bounce | 1.5 s |
| explosion | h_053c429f -> a weapon: `<Damage max min damage-type radius>` | h_15889e57: 58.5-65, type 10, 8 m |
| blast effect | explosion bullet h_ec23d593 -> effect type: ALE effects, light effect, sounds (`EffectType`, `src/bf/character.rs`) | h_1fe9ae17: exp-lrg-dirt, -fire#1, -fire, -flash, -shrap, exp-fire-add; light_explosion; 145f09e5 |
| the effect type's own areas | the blast effect type's `<Damage>` areas and `<shake>`s (#373): started with the blast, over their duration (`fly_grenades` asks `play_effects.rs`, [effects.md](../effects.md)) | none (the Molotov's h_1a256542 has a Damage area) |
| impact sound | bullet h_e5618348 | fb4b3604 |
| decal | bullet h_06a27365 -> `<decal>` | h_ff1b711e: ff534b01, RGBA 10 10 10 200, 3 x 3 |
| trail | h_18b6ab72's first slot -> effect type | h_10a5508f: grenade_trail + 10318b29 (the hiss) |
| HUD icon, label | h_e5ec3f1f, stringtable-name | fe20b919, "Frag" |
| use | h_1ee2f4ed: 3 IOU_THROW_TO_USE, 2 IOU_PLACE_ON_GROUND; function-type 8 / 13 (Sentry / Roller) | thrown |
| event sound | `<event state=7 h_fa04e025>` | e43166d1 (the meter appears) |

`BF_GRENADE_LOG=1` prints each type's fields at load (`grenade <label> ...`), each throw's launch
("Frag leaves the hand 2.13 m over the floor at the thrower's feet at 25.6 m/s, 6.0 degrees up
(mass 0.9)", #161), each grenade's first contact (how long after leaving the hand, how far from
the thrower), each blast and decal, and a throw called off.

**Sections**

* [Throwing, placing, the fuse, Tab](throwing-and-placing.md)
* [The blast](the-blast.md)
* [Damage, damage types, the red tint, the HUD](damage-and-hud.md)
* [The Gas's cloud (#76)](gas.md)
* [What each type does so far, and doesn't](what-each-type-does.md)
* [Sentry](sentry.md)
* [Roller](roller.md)
* [Light grenade](light-grenade.md)
* [Sonic grenade](sonic-grenade.md)
* [Energy grenade](energy-grenade.md)
* [Thermo (#82)](thermo.md)
* [Test hooks and what was verified (Frag, Gas, hand-over, crosshair, the meter)](test-hooks-and-verified.md)
