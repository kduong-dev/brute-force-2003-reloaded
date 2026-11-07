*Part of [triggers/](README.md), the former page `triggers.md`.*

## Hints, button gates and panels

The message, input, menu and event classes (#224) are the runtime's, dispatched from
`Triggers::deliver` and `Triggers::update` like the rest; their code is in two child modules,
`play_hints.rs` (the message trigger's hint lines, the input trigger's virtual buttons, the
event-trigger's AI events) and `play_panels.rs` (the menu-trigger's panels), with their own page:
[hints.md](../hints.md). Their state lives in `Triggers::hints` (one per level). A menu-trigger
whose menu game-options doesn't define (the tutorial's trigger_menu509) opens nothing and never
sends FINISHED; a panel up stops the game's clock as the objectives menu does, so the timers
wait under it. `BF_HINT_LOG=1` logs the lines, the gates' presses, the AI events and the
panels; `BF_TEST_VB` presses a virtual button and `BF_TEST_PANEL` closes the panels.
