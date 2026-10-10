# Test map developer tools (`src/bin/play_testtools.rs`, #111)

Demo tooling for trying features out on the test map ([play.md](play.md), "Test map"),
not the game: the keys, speeds and layout are the demo's choices, none of them used by the
game's controls. A panel on the right lists the open tool's keys; the controls panel (H, top
left) lists them all.

| Key | Tool |
|---|---|
| F | **free camera** on / off. Mouse: look (once the mouse is captured); WASD: fly along the view; Space / Ctrl: up / down; Shift: 4x faster (6 m/s, 24 m/s). The character stands still meanwhile (its inputs are dropped). **Left click**: teleport the controlled character to the ground under the crosshair, **right click**: to the ground under the camera, both facing the camera's way, the follow camera behind them. F alone goes back without moving them |
| N | **NPC menu**: Up / Down or the wheel picks Brutus, Flint, Hawk or Tex; J sets friend / ENEMY, B dummy / fight (for the next one; enemy dummy at first); click or Enter spawns one on the ground under the crosshair, facing the controlled character. N or Esc closes it (Esc keeps the mouse captured) |
| J / B / Delete | outside the menu, on the NPC under the crosshair (one the ray passes through, else the nearest within 1.2 m of where it meets the ground, the dead too): switch its team / switch dummy and fight / remove it. Shift+Delete removes every NPC |
| O | **object menu**: every pickup type, every hand weapon (the rack's list), each grenade type and the breakable scenery, in groups (81 entries). Up / Down picks, Page Up / Down jumps a group; a see-through copy (alpha 0.6, a faint blue glow) stands on the ground under the crosshair; the wheel turns it 15 degrees a notch; click or Enter puts one there, and the menu stays open for more. O or Esc closes it |

* **NPCs** are `Player`s added at the end of the squad with `npc` set, so everything that acts
  on the squad acts on them: the player's shots (with instant kill), blasts, Sentries,
  ragdolls, blood. They aren't in the formation, can't be taken control of (1-4, the hand-over
  after a death) and don't show on the portraits, health bars or speech icons. Friends are on
  the squad's team (0), enemies on `ENEMY_TEAM` (1).
  * An enemy sets off the squad's Sentries, and the crosshair isn't green on it.
  * A **dummy** stands where it was put, facing the way it was put, a target for the player:
    the AI never fires at it (as BF_TEST_HOSTILE's held squadmate; the product owner's
    choice: the squad shoots only enemies set to fight). It doesn't dive from grenades.
  * One set to **fight** stays put (but dives from a grenade landing near, as the squad
    does) and aims and fires at the nearest hostile in sight, and the squad AI fires at it. This is the squad AI's fire, which until now never had a target
    (`play.rs`: `hostile_in_sight`, `ai_fire`): an AI character sees anyone on another team
    within its weapon's range (at least 20 m) with nothing of the world or the pillars between
    its eyes (1.6 m up) and their chest (1.1 m up), all round; it faces them, turns the gun onto
    them and fires in its bursts (the existing random delay, bursts and pauses); the aim is
    exact. Its shots stop at the first body on their line but its own, a teammate in the way
    included, and hurt by the held weapon's damage when they get there (`AiHits`). In the main
    game everyone is on team 0, so nobody has a target and nothing changes. The target
    (`Player::ai_target`) is the AI's only: it's dropped when control passes to a character
    (the number keys, the hand-over after a death) and never kept by the controlled one, whose
    gun and body follow the crosshair from the first frame.
  * NPCs take no pickups (medkits and fruit are the squad's).
  * Removing one takes its body; the gun it dropped, its blood and its DNA stay.
  * The menu lists the data's other character types (militia, mutant, feral_colonist,
    shadoon, the hounds ...) as "not yet, #110": `Player` needs a squad character's model and
    clips. #110 adds them to `npc_types`.
  * Several characters can share a squad character (an NPC Tex beside the squad's Tex), so
    what reaches a body later tells them apart by `Player::who` (a squad member's character, an
    NPC's number): Energy bolts and Sonic rings land on the body they were drawn to, and the
    thrower's own share of a Frag, Gas, Energy or Sonic blast is the squad thrower's only, not an
    NPC's of the same character.
* **Objects**, put on the ground at the crosshair turned the way the copy shows:
  * a pickup is a level's inventory-object as the grid's (`play_pickups.rs`: `LatePickup`
    seats it on its lowest point, upright as modelled): medkits and fruit can be taken,
    everything is loose. Two entries of one name (the placed and the carried Medkit, two Bio
    Ammo) carry their hash;
  * a weapon lies on its side; walking into it takes it as from the rack, and it's gone;
  * a grenade lies as modelled; walking over it (within 1 m) adds one of its type up to its
    stack-limit ("Took Sentry"), else "<type>: full" (the test map starts with full stacks);
  * **breakable scenery**: the radiation barrel (h_e04e5a0e), the missile rack (h_fbdcd828)
    and the supply crate (h_09a6856d), sdm_e34's kinds (see "Interactive scenery"), standing
    as modelled. One put down joins `play_scenery.rs`'s list as a map's does (its
    `LateBreakable`: hitpoints, centre, debris made ready) and breaks, explodes and chains as
    there: shots, grenade blasts and other objects' damage areas, by the type's factors (the
    rack's Type 10 blast sets off barrels and crates within its 8 m; a barrel's Type 3 cloud
    sets off nothing). The flat floor has no collision, so its model's box stands in for it:
    `ray_hit` stops at the box (shots end on it and set it off, the crosshair meets it);
    characters walk through it, and through what it leaves behind (the rack's stand).
* Not done: moving NPCs, removing placed objects, saving a layout, enemy species (#110).

## Test hooks

Times in s of the controlled character's clock; several entries of one hook separated by `;`;
yaw and pitch in degrees. `BF_TOOLS_LOG=1` prints each step as it's due and what it did. The
help panel: `BF_TEST_HELP=<s>[;<s>...]` presses H.

| Hook | Does |
|---|---|
| `BF_TEST_FREECAM=<s>[,<x>,<y>,<z>,<yaw>,<pitch>]` | F: the free camera from where the camera is, or put there. `BF_TEST_FREECAM_OFF=<s>`: F again |
| `BF_TEST_FLY=<from>,<to>,<right>,<up>,<forward>[,fast]` | fly that way (each -1..1) |
| `BF_TEST_TELEPORT=<s>[,camera]` | the free camera's left click (teleport to the crosshair's ground); `camera`: the right click |
| `BF_TEST_NPC_MENU=<s>` / `BF_TEST_TOOL_CLOSE=<s>` | N / Esc |
| `BF_TEST_NPC=<s>,<character>,<friend\|enemy>,<dummy\|fight>[,<x>,<z>]` | spawn one at x, z or under the crosshair, facing the controlled character |
| `BF_TEST_NPC_TEAM`, `BF_TEST_NPC_FIGHT`, `BF_TEST_NPC_REMOVE` `=<s>,<n\|aim\|all>` | switch the team / the behaviour of, or remove, the n-th NPC spawned, the one under the crosshair, or all |
| `BF_TEST_PLACER=<s>,<object>[,<yaw>]` | O with that object picked (its label in any case, else the first containing it), turned that way |
| `BF_TEST_PLACE=<s>[,<object>,<x>,<z>[,<yaw>]]` | the object menu's click (at the crosshair); or that object put at x, z |

`BF_COMBAT_LOG=1` also names NPCs ("flint (NPC 1)") and prints each AI shot that meets a body
("brutus (NPC 3) shoots hawk (1.2 m)").

## Verified

Test map, captures at 15 fps, frames looked at:
* Help: hidden at the start with "H: help"; `BF_TEST_HELP=2` shows the panel with the tools'
  line.
* Free camera and teleport (`BF_TEST_GOTO=0,10,0,10 BF_TEST_FREECAM=1.5,6,5,16,20,-30
  BF_TEST_FLY=2,3,0,0,1 BF_TEST_TELEPORT=3.5`): the view cuts to the pose and flies 6 m along
  the view while Tex stands still; the teleport puts him at 1.96, 4.91, where the crosshair ray
  from the flown-to pose meets the floor, with the follow camera behind him. `camera` put him
  under the camera (30, 40). (With a `BF_TEST_GOTO` goal off the start the autopilot walks him
  back: use one equal to the start.)
* NPCs (`BF_TEST_GOTO=4.5,24,4.5,23.5 BF_TEST_FREECAM=0.5,17,2.5,14,90,-10
  BF_TEST_NPC="1,flint,enemy,dummy,1.5,15;1,hawk,friend,dummy,7.5,16;3,brutus,enemy,fight,4.5,9"
  BF_TEST_NPC_FIGHT=8,1 BF_TEST_NPC_TEAM=11,1 BF_TEST_NPC_REMOVE="13,2;14.5,all"
  BF_COMBAT_LOG=1`): the two dummies stand facing Tex and nobody fires at them; enemy Brutus
  (fight) opens up on the squad's Hawk and they fire back (Hawk, Flint, Brutus), the enemy
  Flint dummy in the line taking some of the squad's Brutus's shots; Brutus is knocked down
  and killed. Flint switched to fight is fired on by the squad and fires back; switched to
  friend, the firing stops. The removals take Hawk, then the rest.
* Spawning at the crosshair (`BF_TEST_NPC=2,hawk,enemy,dummy` with the free camera and the
  menu open): Hawk stands where the crosshair meets the floor; `aim` switched his team and
  removed him.
* Sentry (`BF_TEST_GRENADE_TYPE=Sentry BF_TEST_GOTO=4.5,24,4.5,24 BF_TEST_THROW=1,0.1
  BF_TEST_FREECAM="2.5,30,3,40,30,-25;8,9.5,2.5,29,30,-22" BF_TEST_TELEPORT=3,camera
  BF_TEST_NPC=9,flint,enemy,dummy,4.5,22 BF_SENTRY_LOG=1`): Tex sets one down, is teleported
  away with the squad following, an enemy Flint dummy is spawned 1.6 m from it and it goes off
  at the first check, killing him.
* Objects (`BF_TEST_GOTO=4.5,30,4.5,30 BF_TEST_FREECAM=1,4.5,2,25,0,-45
  BF_TEST_PLACER="1.2,medkit,0;3,Confed LZR-50,45;4.5,frag,0;6.3,Confed LZR-50,135"
  BF_TEST_PLACE="2;3.8;5.3;7" BF_TEST_FLY=...`): the copy stands at the crosshair, turned
  45 and 135 degrees as asked, and each click leaves the real one there. Walking over them
  (`BF_TEST_GOTO=4.5,30,4.5,12 BF_TEST_HEALTH=50 BF_TEST_GRENADE_TYPE=Sentry BF_TEST_THROW=0.3,0.1
  BF_TEST_PLACE="0.2,medkit h_f5123ace,4.5,26,0;0.2,sentry,4.5,22,0;0.2,Saryl-45,4.5,18,30"`):
  the Medkit goes into the inventory, "Took Sentry" (9 to 10), "Took Saryl-45" (the character
  respawned with it, 30 / 300).
* Breakable scenery: `BF_TEST_GOTO=6,30,6,29.5 BF_TEST_FIRE=1 BF_CAMERA_PITCH=-0.12
  BF_TEST_PLACE="0.2,missile rack,6,18,0;0.2,radiation barrel,3.5,18,0;0.2,radiation
  barrel,9.5,17,0;0.2,supply crate,1.5,15,0" BF_SCENERY_LOG=1`: Tex's first shot stops on the
  rack (1 hp) and breaks it; its blast (Type 10, x10) breaks both barrels and the crate the
  next frame; the fireballs, the pieces flying, the rack's stand left behind; and the shot no
  longer flies on through it into Flint 18 m behind (it did before `ray_hit` met the boxes).
  A Frag thrown at a rack (`BF_TEST_THROW=1,0.6 BF_CAMERA_PITCH=-0.15`, rack at 6,16, barrel at
  8,11): the blast breaks the rack 4.5 m away, and the rack's area the barrel 9.6 m from the
  blast, outside the Frag's 8 m. A barrel hit next to a rack (`BF_TEST_HIT=1.5
  BF_TEST_HIT_NEAR=3.5,18`): the barrel goes, the rack stays (the data's Type 3 x0). The
  rack's see-through copy shows turned 30 degrees.
* The hand-over (`--test BF_TEST_INSTANT_KILL=0 BF_TEST_GOTO=4.5,24,4.5,24 BF_TEST_FIRE=1
  BF_TEST_NPC="1,brutus,enemy,fight,12.5,14" BF_TEST_SELECT=0,2.2 BF_TEST_NPC_FIGHT=4,1
  BF_SHOT_LOG=1`): the squad's Brutus, firing at the enemy Brutus, is taken control of at
  2.8 s; his body turns from 116 degrees off onto the crosshair within 0.4 s (`BF_SHOT_LOG`:
  residual 106 at 2.93 s, 10 from 3.33 s on, the standing aim's own) and his shots follow the
  crosshair (dir along the ray), where before the fix they stayed on the old target for 6 s and
  more.
* Energy and Sonic (`BF_TEST_GRENADE_TYPE=Energy` / `Sonic BF_TEST_GOTO=4.5,24,4.5,24
  BF_CAMERA_PITCH=-0.4 BF_TEST_THROW=1.5,0.3
  BF_TEST_NPC="0.5,flint,enemy,dummy,4.5,18;0.5,tex,enemy,dummy,6,16;0.5,hawk,friend,dummy,3,15"
  BF_TEST_INSTANT_KILL=0 BF_COMBAT_LOG=1`): each bolt and ring lands on its own body, the NPCs
  at their places ("flint (NPC 1)" 1.4 m out, the squad's Flint 2.6 m out, each its own
  damage); Tex, the thrower, takes his own share (33.2 at 7.4 m), the NPC Tex the blast's
  (77.4 at 1.9 m).
* A Frag beside a dummy (`BF_TEST_THROW=1.5,0.3`, enemy Flint dummy at 4.5,19, friend Hawk
  set to fight at 7,19): Flint stands where he was put, hurt (2.6 m); Hawk dives off to 10.8,
  18.0.
* The main game is unchanged (after #85 merged): `BF_MAP=sdm_e34 BF_TEST_GOTO=0,0,0,-20
  BF_TEST_FIRE=1 BF_TEST_HIT=2`, 75 frames, the same as main's build but for frames that also
  differ between two runs of main's own build (frames 2-3 and a few 1-level pixels: run to run
  noise); on the test map the Sentry's
  `BF_TEST_HOSTILE=flint,4,0,0.5` run logs the same checks and blast (a few pixels of smoke
  differ: effect seeds come from entity numbers, and the tools add an entity).
* Not tested in a capture: the keys themselves (the hooks drive the same code, but the key
  and mouse handling is only read, not run), PageUp / PageDown, Shift+Delete.
