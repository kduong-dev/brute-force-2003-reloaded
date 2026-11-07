*Part of [triggers/](README.md), the former page `triggers.md`.*

## Edit mode

The map editor's edit mode ([editor.md](../editor.md)) holds the triggers (#230; the product owner's
call, the game has no editor): the runtime's frame (`play_triggers.rs` `run`) and the damage
plate's hits (`play_damage_plate.rs` `deal`) don't run while `EditorMode::Edit` is up. So the
timers keep the time they had left, no detector sees anyone, no spawn-trigger makes a copy (the
lava yard's bursts stop), no script or cutscene they started goes on, and the world's object
events (deaths, pickups, panels) wait in `TriggerEvents`. Back in play they go on where they
were: a timer due 2 s after the switch to edit pulses 2 s after the switch back. Effects already
made play out. The sound-triggers' own frame (the music and the ambience beds,
play_sound_triggers.rs) isn't held.
