*Part of [triggers/](README.md), the former page `triggers.md`.*

## Not done here, or not known

* The classes left to later steps (the object properties: above).
* Radar-triggers (#228): the radar sound of a set with h_12b33046 (none of the tutorial's has
  it) isn't heard in any take; contacts 1, 4, 5 and 6 are kept on their objects but the radar
  draws only 2, 3 and 7 (the alignment and boss contacts are #160's enemies on the radar).
* The damage plate (#228): the HUD's damage direction indicator (take15 shows it on all four
  sides while the plate hurts Tex: FUN_000b3010 / FUN_000b2ef0) and the health bar's flash aren't
  drawn. The 0.2 s an occupant stays in its list after leaving is the detector's (not checked
  for the plate on its own).
* The lava bursts (#228) are a brighter yellow than take08's orange column, which is also
  wider near the camera, and take08's orange embers scattered round them are missing. Their
  effect type h_e8918da4 lists one ALE effect, h_011bcd96 (slava-erupt-jet: its cube emitter, its
  one appearance, and a gravity and a turbulence field, slava-eruptgravity-jet and slava-turb-jet);
  the demo's ALE doesn't run gravity and turbulence fields (ale_fx.rs, AIR_FIELDS), which would
  scatter the particles and bring them down. The take's view from Tex's place (the valley and
  the sky beyond the pit) isn't the demo's (rock walls): the level's, not the bursts'.
* The character's spawn fade-in isn't drawn.
* The member check's roster test (0x8d430) is assumed to pass every squad type.
* Placed characters add no collision to the arena (#230, arena.rs): a character's body is its
  own physics object, not part of the static collision. Before, those no spawn-trigger names
  added their bodies' at their placements, and m09_a's squad started on them and stepped off.
* A character's body in a detector is the demo's cylinder, not its Ipion body against the
  phantom; a dead character leaves a detector 0.2 s after it dies (the game's ragdoll may stay
  in the phantom: not known).
* A death's DESTROYED comes from the object's +0x144(1) (a game object's FUN_00157260, which a
  character's FUN_00112830 runs first): id 0 and data -1, as the demo sends it, but for a
  character its subject is a character from its vtable +0xf4 object's +0x45c (perhaps its last
  attacker: not traced), not the dead object as the demo has it (#221's reading; nothing in the
  tutorial reads it).
* Usable objects (#262): the AI doesn't use a console or a panel (the game's door-point branch
  that would use a world button is dead: code-map.md, FUN_00026820); a console that survives its
  use (m14's, m09's, three of e16's: no property trigger on its use) is used again by the next
  press and sends USE_OBJECT again; whether the game does (nothing in FUN_00158140 stops it) isn't seen:
  m14's consoles weren't got in the game (todo/262-consoles notes: m14_a's was on a damaging floor,
  no prompt showed in several aimed views, and a 1.5 s hold did nothing: it may not be live until
  the chapter's script state). A console's display name "Shrike console" (m09, m14) is the data's.
  The panels' own one use and their 0.5 s hold are the demo's (maps.md).
* Animated objects only looped (the tutorial's looping props h_e5b1f2c8, h_0cbffd71, the fans
  and other props of e01, e16, e23 and m03_a, m08_b's dish) aren't moved: an object on transform
  joints or hinges is moved, as a door, only when an anim-trigger plays it once (#261,
  [maps.md](../maps/doors-and-gates.md), "Doors and gates"). An anim-trigger's AC_PAUSE / STOP / RESET_TIME and the
  looping and oscillating modes are read as forward or backward; a door's clips each run on a
  clock of its own since #262 (m01_a's gate plays its two leaves at x5.5 and x1).
* The start points' facing (+z, arena.rs) hasn't been checked against footage; spawn-triggers
  face -z (above).
* Sound-triggers (#223):
  * The opening script's USER2s to router443 (6.575 s) and router441 (11.025 s) silence the lava
    pits after its lava shot (#222). Runs that don't play it (`BF_TEST_GOTO`, `BF_SCRIPT`) have
    the west pit's bursts (every 4 / 6 s) and the east pit's loop play on from the start (heard
    only within their 40 m: the rolloff curve);
    `BF_TRIGGER_FIRE="trigger_router443,USER2,6.575;trigger_router441,USER2,11.025"` sends them.
  * A dialog-trigger's line is a plain sound, not the character's voice (its priority 10 over
    chatter, its mouth); Bratton's lines have no radio filter. (Its hush flag mutes the chatter,
    #155: [chatter.md](../chatter.md).)
  * 3D voices are a volume only (the demo's sounds have no left and right) and have no reverb:
    the level's I3DL2 environment (the world's +0xb54, the sound system's +0x78) and each 3D
    voice's send to it, the world's indoor or outdoor amount as its listener stands (0x17f390),
    aren't modelled. The listener and h_15bc1769 measure from the player character's feet (which
    point of the character FUN_00178c10 and 0x8d660 take isn't traced). The demo has the one
    listener; the game counts a share of up to four (split screen) for Types 2-5.
  * The fade-out's dB-linear shape and its 60 dB are measured (above), not read from the code
    (the envelope is the sound chip's). That a stop's flag 0 (FUN_002063c0) is the fade-out is
    read from the code since #254 (a stream's StopEx 0x182f30: FlushEx with ENVELOPE).
  * After a music replace the old voice's handle stays in the sound system's map (0x17eff0
    clears only its +0x1c bit 0), so its trigger's later stop reaches that stream, whatever it
    plays by then if the pool (8 streams) has handed it out again: tutorial_01's timer525 at
    168 s could fade out another stream. Not modelled: the demo's old voice is gone.
  * Types 2-5's beds are checked only in the demo's logs and frames: no take covers a level
    with them (the tutorial has none).
  * The takes' timers run ~3-4% fast against their watch logs' app clock: timer525's 168 s stop
    of the start music fades from clk ~163.2 s in take04, timer462's 5 s after the refresher
    opens BASIC CONTROLS at clk 31.08-31.10 s (take03 / take04) where the code says 31.30, while
    the script's and the sounds' times match the clock (Tex's spawn at 17.09, the co-op hint at
    FINISHED + 0.1 s). The tester reads it as the app clock running slow (0.94-0.96) through the
    watcher's pauses: the code's 31.30 stands; compare durations with that in mind.
