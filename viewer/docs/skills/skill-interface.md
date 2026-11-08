*Part of [skills/](README.md), the former page `skills.md`.*

## The skill interface (#121-#124)

`SkillKind` is a skill's own part, as the game's skill classes override their base class's
vtable; the four are `Berserker`, `Vengar`, `Stealth` and `Targeting` in `play_skills.rs`, made by
skill-set (CSKILL_ 3, 1, 4, 2) when the character spawns, and the effect skills' are
`play_effect_skills.rs`'s (`kind`: the Orbital Beam, 9, #200; [Other characters' skills](other-characters-skills.md)),
with the Air Strike's in `play_air_strike.rs` (6, #203; [The Air Strike](air-strike.md): a toggle that
calls, pays and answers Now, an update that counts its timer and makes the next call or ends it, a
drain of 0) and the Wrath's in `play_wrath.rs` (13, #208; [The Wrath of Garner](wrath-of-garner.md): a toggle
that fires, switches itself on and returns 2, an update that ends it when its timer is out), and Gunthar
Ghent's Cloak in `play_cloak.rs` (8, #205; [Gunthar Ghent's Cloak](ghents-cloak.md): Hawk's controller,
`on`, `off`, `visibility`, `noise` and a 1x `fired` behind a toggle that switches at the press, no `used_item`
and no `holds_fire`, with `cloaks` true for the look) and McTavish's Tracking is Flint's `Targeting` with his
block and name (`tracking_kind`, 12, #207; [McTavish's Tracking](mctavishs-tracking.md)). Each method's
default is the base
class's; the skill is taken out of its character while they run, so they get the character
(`u.skill.on`, `u.stamina`, its weapons...):

| Method | vtable | Default | |
|---|---|---|---|
| `name` | +0x4c | | the string id the HUD shows |
| `toggle(u, out)` | +0x04 | | `Refused`, `Now { overlay }`, `AtEvent(event)`, `Stays` (#200: the Orbital Beam's) or `Motion` (#208: returns 2, the whole-body MT_TOGGLE_SPECIAL motion: the Wrath's) |
| `drain(u, cost, dt)` | +0xbc | `cost` × dt | Tex's ×0.5 with the gun in hand empty, Brutus's charge lump, Hawk's blade ×5 |
| `on(u, out)` / `off(u)` | +0x120 / +0x124 | nothing | the skill's effects start and end |
| `update(u, dt)` | +0xc0 | nothing | every frame, on or off, alive; true sends the toggle command: the skill's own ends |
| `ended_by_actions` | +0x7c | false (0x20e200) | true (0x213690, Tex's and Flint's): while on, the switch key, the reload key and the black button end the skill instead (`skill_input`) |
| `lets_overlays_play` | +0xa4 | true (0x262a80) | false (Tex's 0x13d850): while on, no other overlay starts (`overlays_blocked`: the reload, the switch, the throw, item use) |
| `lets_act(u)` | +0x14 | true (0x213690) | the last word of FUN_00117210, whether the character is free to act (`lets_act`, `free_to_act`): Flint's (0x13b980) and the Wrath's (the same method) say no while on, so no reload, weapon switch, grenade or medkit starts and no crouch command is sent (the key is gated, 0x134df6: play.rs `read_input`, the crouch hooks); no X hold arms in the demo (its simplification: the game defers or cancels the completed use, 0x1207b9; #208: play_usables.rs, play_weapon_pickups.rs) |
| `roll_factor(u)` | +0xec | 1.0 (0x10e3d0) | the shooter's factor on its damage rolls (FUN_0022c940; `skills::roll_factor`, play_damage.rs `Hit::skill`): Flint's doubled and scoped rolls |
| `damage_factor(u)` | +0xd8 | 1.0 (0x10e3d0) | the factor on the damage the character takes (0x125600 at 0x125747, after the difficulty step; `skills::damage_factor`, play_damage.rs `Hit::victim_skill`): Brutus's 0.85 while on (0x13c140) |
| `range(u, def, range)` | +0x54 | the range (0x10e3a0) | a gun's range in the character's hands (FUN_002324c0; `weapon_range`): Flint's rifles x1.25 |
| `move_cap(u, m)` | +0x38 | the stick (0x10e8f0) | the most of the move stick the character goes by (FUN_001195c0; `move_capped`): Flint's 0.55, a walk |
| `lock_box(u, scale)` | +0x100 | the scale (0x10e8f0) | the crosshair's lock box scale (FUN_000c5d20; `lock_box`): Flint's 1.0 |
| `lock_shares(u)` | +0xc4 | none (0x2894f0) | the lock box's shares of the screen the skill puts in the target selection (`lock_shares`): Flint's 0.4 / 0.4 |
| `crosshair_alpha(u, alpha, fill)` | +0xfc | the alpha (0x10e8f0) | the crosshair's alpha (FUN_000c6280; `crosshair_alpha`): Flint's fades it as her marker fills |
| `has_marker` | (+0xc4) | false | the skill shows the targeting marker for the controlled character (play_targeting.rs): Flint's |
| `keeps_gun_up(u)` | (FUN_00123ea0) | false | the skill keeps the gun raised, holding its ready timer (`Skill::raised`: the ready pose, `gun_raised`) and with it the follow camera's aim shift (play_camera.rs: the camera in): Flint's while on |
| `bars_scope(u)` | | false | the demo's choice: no scope in Tex's berserk motions |
| `fired(u, def)` | +0xb0 | nothing (0x2c60c0) | each round the character's gun fires (FUN_0022f640 from the fire loop; `skills::fired`, play.rs `step_player`): Hawk's flash and cost while stealthed |
| `used_item(u, use_type)` | +0x104 | nothing (0x2c60c0) | an item used, by its IOU_ use type (FUN_00121170; `skills::used_item`: play.rs's throw and set-down, play_pickups.rs's medkit): Hawk's cost while stealthed |
| `uses_item(use_type)` | +0x1c | true (0x262a80) | whether the character can use an item of that IOU_ use type (the items' "can use" 0x146c70 asks it; `skills::uses_item`, play.rs `item_usable`, play_inventory.rs `use_other`): Brutus's (FUN_0013c160) refuses the OrgSen's IOU_ATTACH_TO_FACE and the Hat's IOU_ATTACH_TO_EQUIP_SLOT, red in his box and "Cannot use" at G (#281, #196 take26) |
| `visibility(u)` | +0xe4 | 1.0 (0x10e3d0) | how visible the character is (`skills::visibility`): the AI's sight (play_brain.rs `Sensor`) and the crosshair's selection (play_reticle.rs) read it. Hawk's falls as she fades |
| `noise(u)` | +0xe8 | 1.0 (0x10e3d0) | the factor on its footsteps (`skills::noise`, play_liquids.rs `step`, which `step_player` asks at each foot event): Hawk's a quarter stealthed |
| `holds_fire(u)` | +0x108 | false (0x20e200) | an AI character holds fire (`skills::holds_fire`, play.rs `update_player`: no target for the squad AI or a test map NPC): Hawk while stealthed |
| `effect_data` | (FUN_0013de50's copy) | none | an effect skill's `<effect-data>` (`bf/character.rs` `EffectData`): what its spawner starts when it sets `Skill::fire` (play_effect_skills.rs `fire_effects`, #200): the Orbital Beam's |
| `cloaks` | (the demo's) | false | the skill fades the body through the invisibility controller its `on` / `off` switch (`skills::cloaks`, play_stealth.rs: the body's copies, the refraction): Hawk's Stealth and Gunthar Ghent's Cloak. Not a method of the game's: it asks the skill what its name was asked before, `NAME_STEALTH` |
| `attack_skill` | +0x10c | false (0x20e200) | an attack for the AI (0x213690: the effect skills): GOAL_TOGGLE_COMBAT_SKILL's start would write its target's position into the skill's point; the demo's goal refuses such a skill (`skills::attack_skill`, play_orders.rs `toggle_ability`) |

`toggle` and `on` get `Outside`: what a skill reaches beyond its character, the squad's ammo
`Reserve` (Tex's top-ups) and the log switch. A toggle that does its own work and switches
nothing answers `Toggle::Stays` (the Orbital Beam's FUN_0013e910, which returns 0 and never
activates); one that switches itself on and returns 2 answers `Toggle::Motion` (the Wrath's
FUN_0013f240): `command` carries out its activate and starts the type's MT 45 motion
(`start_pose`: `Toggling::pose`, held still by `holds_still`). A skill that ends itself (the Wrath's
timer) can't switch off from its `update`: the update asks for the toggle command and its toggle
answers `Now` (the game's update calls FUN_0010e3e0 itself). An effect skill's spawner, its target point and its name's hold are kept on `Skill`
(`fire`, `point`, `name_hold`), which play_effect_skills.rs and the HUD read. The free functions of the same names in
`play_skills.rs` ask the controlled or any character's skill for the other systems (play.rs,
play_reticle.rs, play_damage.rs's callers, play_targeting.rs).

Shared, not per skill (FUN_0010e470 / FUN_0010e3e0): the 10% check (`can_activate`), the on
flag, the start / end sound for the controlled character, the chatter line. `free_to_act`
(FUN_00117210) is what Tex's and Hawk's toggles ask; it ends with the skill's `lets_act`, which
the reload (play_ammo.rs), the weapon switch and the grenade (play.rs) and the medkit
(play_pickups.rs) ask on their own. A skill's effects outside the character (a
material, the post-process, the HUD marker, the AI's view of Hawk) read `Player::skill`
(`on`, `toggling`, `special_motions`, `double_hit`, `cloak`), the hooks above and the character
(Tex's `Player::second`) from their own systems: Brutus's charge and Vengar view are
play_vengar.rs's systems, which ask `skills::vengar_on`; Hawk's body (and Gunthar Ghent's, #205) is
play_stealth.rs's, which asks `skills::cloaks` (`SkillKind::cloaks`) and `skills::opacity`.

A system outside the character can't reach a concrete skill's own state through `Box<dyn
SkillKind>`: what others need is a hook (above) or kept on `Skill` / `Player` (Tex's second gun,
`Player::second`; Flint's doubled-roll flag, `Skill::double_hit`; the stamina Brutus's charge
hit takes, `Skill::pending`, which play_vengar.rs sets and his drain takes; Hawk's invisibility
controller, `Skill::cloak`, the character's in the game too, char+0x54c, which her on and off
switch and play_stealth.rs draws by). Flint's marker is
the HUD's own state (`targeting::Marker`), as the game's widget is.
