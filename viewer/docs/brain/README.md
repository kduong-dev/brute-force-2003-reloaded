# The level's characters: their brains (`src/bin/play_brain.rs`, #227)

*This folder was the page `brain.md`, split into one file per section (#393): the sections are listed under **Sections** at the end of this page, and `../brain.md` is now a pointer to them.*

What a character a level's spawn-trigger makes does on its own: the Red Hand's patrols, their
walk home, their sight and alert, their fight and aim, and the Confed marines following Tex.
It's the tutorial's step 8 (#28) and the first slices of #26 (the AI): the goals on routes and
the pursuit (#336), the cover (#384, "Cover" below), the fallback from a target too close (#383,
"The fallback" below); strafing, grenades and calling for help are #26's other parts. The squad's
own AI (`squad_ai`, [squad.md](../squad/README.md)) and the
test map's NPCs ([testtools.md](../testtools/npcs.md)) are unchanged; the hit reactions, death and blood
are [enemies.md](../enemies.md)'s.

Everything is read from default.xbe's AI: the brain at char+0x830, its goal machine
FUN_000377b0 and the goal table it calls (FUN_00012b40 fills a 0x40-byte record per goal id with
its start, end and update, each a member function of the goal holder's vtable 0x3a47b8); every
function cited is in [code-map.md](../code-map/ai.md), "AI" and "Levels and triggers". The sight rules
are #197's answer (`todo/197-ai-sight/answer.md`). The personalities are
`common/ai/personality.xmb` (`bf/personality.rs`). Since #336 the goals walk the level's
routes ("Its goals on routes" below; `src/bin/play_goals.rs`, [nav.md](../nav/enemies-on-routes.md) "The enemies on
routes").

A brain also has a command mode (brain+0x58, the PCMD_ table 0x3bd210: FREEWILL, FOLLOW, GUARD,
WINGMAN, MOVE_OUT ...), which picks the personality it runs from its type's `<ai>`; the squad's
orders set it (#318: `play_brain.rs`'s `Command`, run for the squad's AI by `play_orders.rs`,
[squad.md](../squad/command-modes.md) **Squad commands**). The level characters below keep the personality they
spawn with; the marines' mode isn't read yet.

**Sections**

* [The spawn: its home](spawn-home.md)
* [Its base goal](base-goal.md)
* [Patrol](patrol.md)
* [Sight and contacts](sight-and-contacts.md)
* [Alert and the fight](alert-and-the-fight.md)
* [Aim](aim.md)
* [The Confed marines](confed-marines.md)
* [Its goals on routes (#336)](goals-on-routes.md)
* [Cover (#384)](cover.md)
* [The fallback: a target too close (#383)](fallback.md)
* [Held by a script](held-by-a-script.md)
* [Test hooks](test-hooks.md)
* [Verified](verified.md)
* [Not done](not-done.md)
