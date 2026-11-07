*Part of [triggers/](README.md), the former page `triggers.md`.*

## The classes it runs

| Class (tutorial count) | What it does | Code |
|---|---|---|
| router (58) | Passes every event but FIRST_UPDATE to its targets. With h_f0d6857f it passes only those its OPERATE entries accept, as `out-signal` when that isn't -1 (e.g. `h_e9413a18`: ENTER_DETECTOR -> USER_SIGNAL1) | 0x208fc0 |
| timer (54) | OPERATE starts it, TIMER_FULL_RESET stops and reloads it, RESET_COUNTDOWN_TIME reloads it. Running and enabled it counts `current-value` down; at 0 it sends TIMER_PULSE (with the subject its OPERATE came with), then stops and reloads (h_f92b0b15), reloads (enable-looping), or stays at 0 until reset | 0x200190, 0x2002f0 |
| calculator (12) | INC / DEC / MODIFY_COUNTER (by the `<calculator value>`) / SET_COUNTER_FROM_SIG_DATA, then its test h_015914ab (0 ==, 1 <, 2 >, 3 <=, 4 >= its `target`, unsigned): CALCULATOR then TRUE, or FALSE, with the counter as data | 0x1ff5f0 |
| spawn-trigger (47) | See "Spawn-triggers" | 0x209ac0, 0x209bd0 |
| plate-detector (27), proximity-detector (14) | See "Detectors" | FUN_001f9680, FUN_001f8db0 |
| anim-trigger (7) | OPERATE plays its entry's clip (`animation-archetype`) on its first object, forward or backward (`wwit-debugging`, AM_: 3-5 backward) at its `time-scale`; with AC_PLAY or AC_RESUME it sends ANIM_START when it starts forward after playing backward (its first forward play too), ANIM_BACK_START when it starts backward after playing forward (or ANIM_START for the other modes). The demo moves doors and gates with it, their leaves on sliding, transform or hinge joints ([maps.md](../maps/doors-and-gates.md), "Doors and gates", #261); other animated objects (the tutorial's two looping props) are logged | 0x204d00, FUN_00204bb0 |
| sound (73) | PLAY_SOUND plays its sound (alike everywhere, or at its place), PLAY_DIALOG queues it on the world's dialog queue, STOP / PAUSE / RESUME; FINISHED when its length has run (#223: see "Sound- and dialog-triggers") | 0x206450, 0x206690 |
| dialog (4) | Its character says its line; FINISHED once it's said (#223) | 0x206d20, 0x206ee0 |
| message h_e9b3ae8e (37) | OPERATE: OPERATED, then the HUD's two hint lines set from its own (shown ones blink in, the others hidden) (#224: see "Hints, button gates and panels") | 0x2078e0 |
| input h_e2b9c3a2 (37) | Enabled, TRUE to its targets every frame its virtual button is held on any pad (#224) | 0x2008a0 |
| menu (10) | OPERATE opens its game-options menu over the game (a tutorial panel) and sends OPERATED; the panel's A and Y send MENU_ACCEPT / MENU_Y_BUTTON, then FINISHED once it's closed (#224) | 0x2082d0, 0x208430 |
| event (2) | An AI event about an object it listens to whose number is its `event-id`: EVENT_MATCH, then it disables itself (#224) | 0x202dd0 |
| destroyer (5) | OPERATE marks it; its update removes its objects (a listed spawn-trigger's copy; an effect object, whose particles go with it; a lamp, which goes out: #308), then disables itself. The update runs the base update first, so an OPERATE on its own FIRST_UPDATE removes them in that same update (the tutorial's four props at t 0). It removes a character the triggers spawned, never a member of the demo's squad (no level's destroyer lists one) | 0x200d70, 0x200e40 |
| waypoint (h_10ecb459, 31) | OPERATE sets its `world-point`'s valid bit (+0x28 bit 2) to its `set-valid`, -1 leaving it, for the AI's routes ([nav.md](../nav/graph.md), #334): a cover point's set-valid goes to its own +0x64 bit 5 instead, a save point's to the world (not kept). As the tutorial starts they make three door points and seven of the mine field's world points invalid | 0x202c50, FUN_00202ba0, 0x202af0 |
| property h_14cccfaa (52) | OPERATE gives its first object a reticule-action (h_e92285dc, 0 on all 52: the object is no longer a use target) and is counted (#262: see "Usable objects") | 0x20b730, FUN_0020b680 |
| evaluate (6) | Its base actions only: the evaluate-triggers' random tests pick ambient lines (a no-op here) | |
| gamevar (15) | See "Gamevar-triggers" | 0x1ffc40, FUN_001ffb80 |
| script (10) | OPERATE asks its script's player (by `script-name`) to play it from the start, pause, resume or stop, by its entry's `<script command>`; the start waits for the player's update, so two in a frame start it once (#222: [cutscenes.md](../cutscenes.md)) | 0x205000, FUN_00204ec0, FUN_00204e40 |
| letterbox (7) | OPERATE brings the letterbox in over its `show-duration`, STOP takes it out over its `hide-duration` (FUN_00073460 / FUN_000734b0); 0.5 s each without one (the constructor, 0x207599) | 0x207480, FUN_00207410 |
| warp (18) | OPERATE moves its objects (a spawn-trigger's copy) to its `world-point`: on the ground under it, facing down a direction-point's -z or as it was for a world-point; a dead character stays, and one not spawned yet has no copy to move. Its fade-in h_0ede4719 is 2 s without one (0x200734; not drawn) | 0x200650, FUN_00200480, 0x200400 |
| score (2) | OPERATE: its `amount` x (1 + 0.1 x the difficulty, with h_11819686) to the campaign squad's cash, under its key h_19ca8cf3 with h_08f457d8, else the null name (every score-trigger's is the null name: the cash's own totals). The tutorial's: score510 5000 on the primary's timer pulse, score509 1000 on the secondary's calculator: $3000 and $600 on STANDARD ([campaign.md](../campaign.md), "The cash") | 0x20ba00, FUN_0005c5e0 |
| level-over h_1f07be84 (1) | OPERATE with h_18f66101 (true by default) ends the mission (the game mode's "over" bit); without it ends the zone: a 0.5 s fade, the checkpoint save ("Saving Campaign..."), then the zone the last zone-trigger named (#226: `play_level_over.rs`, [mission.md](../mission.md)). The tutorial's fires at 8.93 s of its ending script | 0x207fe0, 0x207f70 |
| zone-trigger (0) | OPERATE names the zone (its h_14b0089f, an archive) the next zone's end goes to (#226) | 0x207de0 |
| radar (18) | OPERATE gives its objects their `radar-contact` and a blink time (#228: see "Radar-triggers") | 0x207230 |
| damage plate h_130818c0 (1) | A proximity detector that hurts what's in it while it's enabled (#228: see "The damage plate") | 0x1fa7e0, FUN_001f8510 |
| subject relay h_0cb602d0 (0; 39-77 per multiplayer map) | Binds each event's subject to its targets, which then listen to it, and passes the event on (#278: see "Subject relays") | 0x201310, FUN_001fe6c0, FUN_001fea10 |

Every other class (the AI commands, the mover, the squad, the turret, the shield ...) runs its base
actions; each of its own actions is counted as used, as its handler counts it.
