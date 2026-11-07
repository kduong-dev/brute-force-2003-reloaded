# Level triggers (`src/bin/play_triggers.rs`, `src/bf/triggers.rs`, #220)

*This folder was the page `triggers.md`, split into one file per section (#393): the sections are listed under **Sections** at the end of this page, and `../triggers.md` is now a pointer to them.*

A level's logic is its triggers: routers, timers, calculators, spawn-triggers, detectors,
anim-triggers and some twenty more classes, wired to each other in the level file. The tutorial
has 500 (58 routers, 54 timers, 47 spawn-triggers, 27 plates, 14 proximity detectors ...). This
is the runtime that runs them, the first step of the tutorial (#28, plan step 1); the later
steps plug their classes into it (gamevar and score since step 2, #221, with the game flags and
the cash they work on: [campaign.md](../campaign.md); script, letterbox and warp since step 3,
#222, with the cutscenes they drive: [cutscenes.md](../cutscenes.md); sound and dialog since step
4, #223, with the voices and the level's music they play; message, input, menu and event since
step 5, #224, with the hint lines, button gates and panels they show: [hints.md](../hints.md);
level-over and zone since step 7, #226, with the mission's end: [mission.md](../mission.md);
radar-triggers and the damage plate since step 9, #228, with the radar's objective dots they
set: [play.md](../play/radar.md), "Radar").
Everything here is read from default.xbe's trigger code (the addresses are in the code's
comments and in [code-map.md](../code-map/levels-and-triggers.md), "Levels and triggers"); nothing is fitted.

`bf/triggers.rs` reads them: every element of the level file that carries the trigger base's
`h_137926ff` (FUN_001fedb0's attributes), with its entries, targets (`h_e91cf6a8`), objects
(`h_0b6b92ba`), transform and its class's own attributes (`Trigger`, `Class`); the element is
kept whole for the classes later steps run. `Level::triggers` holds them in the file's order.

**Sections**

* [How the game runs them](how-the-game-runs-them.md)
* [The classes it runs](the-classes-it-runs.md)
* [Gamevar-triggers](gamevar-triggers.md)
* [Detectors](detectors.md)
* [Spawn-triggers](spawn-triggers.md)
* [Subject relays](subject-relays.md)
* [Sound- and dialog-triggers](sound-and-dialog-triggers.md)
* [Object events](object-events.md)
* [Usable objects](usable-objects.md)
* [Hints, button gates and panels](hints-button-gates-and-panels.md)
* [Radar-triggers](radar-triggers.md)
* [The damage plate](the-damage-plate.md)
* [Edit mode](edit-mode.md)
* [Test hooks](test-hooks.md)
* [Verified](verified.md)
* [Not done here, or not known](not-done-here.md)
