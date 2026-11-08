*Part of [skills/](README.md), the former page `skills.md`.*

## Keys and test hooks

| Key | |
|---|---|
| V | the controlled character's skill on / off: the game's White button (VB_SPECIAL_ABILITY, `game-options-en.xml` line 151, which sends CT_TOGGLE_SPECIAL at 0x134cff). Not while dead, nor while the controls don't reach the character: a cutscene, a script's hold, out of the world (play_script.rs `blocks_input`, #256) |
| Q, R | in Tex's berserk and Flint's targeting: end it, instead of switching weapons (Q, the game's Y) or reloading (R, the game's X; only when a reload could start, which then follows the skill's end; on R's press, where the game's X ends it when let go). See [Tex's Berserker](texs-berserker.md) |

* `BF_TEST_SKILL=<s>[,<s>...]`: presses V at those times (once each); a press while the controls
  are off (a cutscene, #256) is lost, as the key's is.
* `BF_TEST_STAMINA=<n>[@<s>][,...]`: sets the controlled character's stamina to n at those times
  (at the start without one), as the takes' pokes: `30@1.9` with `BF_TEST_SKILL=2.0` is take02.
* `BF_TEST_GIVE_SKILL=<skill-set>[,<cost>[,<recovery>]]` (#200): the character controlled at a
  map's start takes that skill class (9, the Orbital Beam; the dm Confed Marine's cost 55 and
  recovery 2.3 unless given; 6, the Air Strike, #203: the Militia's 20 and 1; 14 and 15, the Feral
  Colonist's Spirit of Vengar and the Feral Outcast's Spirit of Garner, #202 and #209: their types'
  2.9 / 1.05 and 3.3 / 1, [the ferals' skills](ferals-spirits.md)). A type of its own with the
  Orbital Beam (the Confed Marine: `BF_TEST_SQUAD="3:confed marine@sdm_ferg"`) or the Air Strike
  (the Militia: `BF_TEST_SQUAD="3:militia@sdm_ferg"`) has it from its type ([Other characters'
  skills](other-characters-skills.md), [The Air Strike](air-strike.md)), and so does a Feral
  Colonist or Outcast (`BF_TEST_SQUAD="3:feral colonist@sdm_feral"`).
* `BF_SKILL_CHATTER=<percent>`: the skill lines' `probability` (100: every line the rest of the
  chatter gate lets through), to hear them.
* `BF_SKILL_LOG=1`: each toggle command (the key's, at 0, a skill's own) and its answer, each
  overlay and when it switches, each switch with its sound and line, and every squad member's
  stamina each half second with its state (`on`, or `off x` its refill factor). For Tex, Hawk and
  Flint also each refusal and why (not those of the command at 0, which comes every frame), and
  the switch and reload keys ending their skill; for Tex the top-up at the press and the second
  gun drawn; for Flint whether the rifle was marked (her sniper motions) and each doubled roll;
  for Hawk each shot's and item's cost while stealthed, her invisibility controller's state,
  level, her visibility, her skin's alpha and the refraction's alpha and offset in the
  half-second line while it isn't off, her model's copies and refraction twins made
  (play_stealth.rs) and each time her body goes see-through or whole again. For the Orbital Beam
  (#200) each press paid or refused under its cost, and the target point with how it was found
  (the lock, the aim ray's terrain or face, the second ray) and the effects started; for the Air
  Strike (#203) each call (the press's or the timer's), its stamina, and the switch off by the
  press, by the update's own end or at 0.
* `BF_TEST_STEALTH_FLASH=<s>[,<s>...]`: gives every stealthed Hawk a round's flash at those times
  (her factor to 0, as a shot, through her skill's +0xb0 with a gun of no damage: nothing fired,
  nothing paid). An AI Hawk holds fire while stealthed, so this is how the flash is seen from a
  squadmate's view (#198's take07; the shot list's H5).
* `BF_TEST_SQUAD_SKILL=<character>,<s>[,<s>...]`: gives a squadmate the Squad Command menu's
  Toggle Ability at those times (by name or 0-3) without the menu: its goal, which sends the
  toggle until the skill has switched ([The toggle](the-toggle.md), #333), also for a member a test
  hook holds in place. With `BF_TEST_HOSTILE=<character>` (grenades.md) it puts a stealthed Hawk
  on the other side, for the crosshair's rule. The menu itself:
  `BF_TEST_COMMAND="2:brutus:0.4;3:toggle"` (squad.md).
* `BF_SIGHT_LOG=1` (play.rs): each AI character finding or losing its target, with the distance,
  the target's visibility and the reach (play_brain.rs `Sensor::reach`), and for the squad and
  the test map's NPCs each contact found and forgotten: when an enemy loses a fading Hawk.
* `BF_TEST_DOUBLE=<percent>`: the chance of Flint's doubled rolls while her targeting is on (100:
  every one), to see the x2 and its flash.
* `BF_TARGETING_LOG=1`: Flint's targeting marker when it changes (on, the lock, the fill at 0 or
  1, where its corners head, the target's size, the glyphs), `=2` every frame; the first lock on
  a character type prints its bounding sphere.
* `BF_TEST_SPRINT=<s>` (play.md): sprints toward the `BF_TEST_GOTO` goal from that time, for the
  dash's refill. With `BF_TEST_CROUCH`, `BF_TEST_SELECT`, `BF_TEST_DIE`, `BF_CAPTURE`
  (`BF_CAPTURE_FPS=60` for the blink and the fades).
* For Tex's berserk (play.md's ammo and shot hooks): `BF_TEST_WEAPON="Confed LZR-50;Jax-iC"` (a
  list fills the weapon slots in order: the takes' loadout), `BF_START_WEAPON=1` (the Jax-iC in
  hand, as takes 05 and 26), `BF_TEST_CLIP=<n>[,<n>...]` (a clip per slot: `99,4` is take05's
  poke), `BF_TEST_TRIGGER`, `BF_TEST_SWITCH`, `BF_TEST_RELOAD=<s>[,...]` (presses R),
  `BF_TEST_THROW`, `BF_TEST_CROUCH_KEY=<s>[,...]` (presses Z: a crouch press, not
  `BF_TEST_CROUCH`'s hold), `BF_TEST_SCOPE=<step>[,<from s>]` (the scope asked for from that time,
  without `BF_TEST_GOTO`, whose autopilot doesn't read it); on the test map
  `BF_TEST_NPC_HP=<s>,<n|aim|all>,<hp>` (testtools.md) gives an NPC take04's 999 HP, and
  `BF_RETICLE_LOG=1` (play.md) shows the lock's reach.
* For Flint's targeting, the same, and `BF_TEST_SWEEP=<deg>|<from>,<to>,<deg/s>` (play.md) to
  turn the crosshair off and back onto an enemy, `BF_TEST_INSTANT_KILL=0` so a test session's
  NPC outlives the first hit, `BF_TEST_AIM=1` for the scope (`BF_TEST_SCOPE` isn't read under
  `BF_TEST_GOTO`'s autopilot).
* For Hawk's stealth (`BF_CHARACTER=2` is Hawk): the takes' loadout `BF_TEST_WEAPON="Saryl-23;Rail
  RVR"` (take14's `Jax-iR`), `BF_TEST_GRENADE_TYPE=Sonic`, `BF_TEST_THROW=<s>,<hold>`,
  `BF_TEST_TRIGGER`, `BF_TEST_MEDKIT`; a live enemy (take14's e40 Shrike, its `vision-range` 75)
  on the test map: `-- --test BF_NO_SQUAD=1 BF_TEST_GOTO=4.5,20,4.5,20
  BF_TEST_NPC="0.5,shrike soldier@e40,enemy,fight,4.5,4.3"` (15.7 m off, take14's distance), with
  `BF_SIGHT_LOG` and `BF_COMBAT_LOG`.
* For Brutus's charge (`BF_CHARACTER=0` is Brutus): on the test map (`-- --test`) a frozen enemy
  ahead, `BF_TEST_NPC=0.5,tex,enemy,frozen,4.5,14` with `BF_TEST_NPC_HP=1.0,1,300` (take08's
  300 HP; 125 for take22's kill) and `BF_TEST_INSTANT_KILL=0`, then the dash at it:
  `BF_TEST_GOTO=4.5,20,4.5,10 BF_TEST_GO_AT=4.0 BF_TEST_SPRINT=4.0` (`BF_TEST_GO_AT=<s>`,
  maps.md: when the goto sets off, here after standing 4 s as take08). With `BF_SKILL_LOG=1` the log has
  each charge hit (whom, the distance, the angle off his facing, the damage, a kill), his line,
  the stamina taken, each switch of the Vengar view, and the hit points of every character with
  Vengar on each half second; `BF_COMBAT_LOG=1` shows the hit's factors, and on a hit Brutus takes
  with Vengar on " x the victim's skill 0.85".
