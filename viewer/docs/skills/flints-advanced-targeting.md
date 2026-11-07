*Part of [skills/](README.md), the former page `skills.md`.*

## Flint's Advanced Targeting (#124)

"Automatically targets enemies." (h_0f1ee6cd). While it's on, the crosshair's lock box grows to
most of the screen, so an enemy near the crosshair is locked and the guns, which aim at the
lock, find it; a HUD marker frames the box or the locked enemy. `Targeting` in
`play_skills.rs` (skill vtable 0x39d2c8, constructor FUN_0013c9e0), the marker in
`play_targeting.rs`, the lock box in `play_reticle.rs`, the aim and the sniper motions in
`play.rs`; measured against takes 15-17, 28, 29 and 31 (spec Part 5). The tuning is
game-options' `<combat-skills>` block h_0725cdb1 (`CombatSkills::tracking`).

* **The press** (her toggle, FUN_0013cad0), either way, needs a motion type 45 transition from
  her motion (the motion record's +0x534): in `ms_flint.xml` her idles, walk, run, side and back
  steps, dash, crouch idle and crouch walks have one, so not getting down or up, in a jump, a
  dodge, a dive or a slide, knocked down, nor with a grenade in the hand (the throw,
  `throw_grenade`, MT 42, has none; the demo counts its charge and a pending release too, as its
  charge runs in her own motion) (`sniper_transition`). Switching on is also refused
  with a gun in the second hand, no gun in hand, or its clip at 0; then the 10% check. It
  switches at the press (takes 15-17, 29, 31), with no overlay; her start / end sounds and lines
  as #120's.
* **The sniper motions**: switched on with a gun of the tracking-weapon-type (h_108b56f9:
  WTYPE_RIFLE, the L-Shot-75; her data's class is `WeaponDef::weapon_type`) the toggle marks the
  skill (+0x48) and asks for motion type 45, which takes her into `Sc_w1_sniper_idle` and the
  `Sc_w1_sn_*` walks (play.rs `SpecialClips`): the rifle raised to the shoulder (take15 2.5 s);
  the off asks again, back to her own motions. Their motion state is an idle or a walk, which is
  why the takes' logs never showed 45. They lead to no run, crouch, dash, dodge, dive, throw or
  jump: those are dropped while she's in them. Switched on kneeling, the demo has her get up
  through her crouch2stand into the sniper idle, its choice: the crouch idle's type 45
  transition goes straight to `Sc_w1_sniper_idle` (with the motion blend; not filmed). With the
  pistol there's no mark: the toggle raises the gun (FUN_00123ea0) and she keeps her motions.
* **While on** (her update, 0x13cbf0): the gun is kept raised (FUN_00123ea0 every frame), and
  the gun in hand's clip at 0 sends the toggle, in the shot's frame, before the reload (take29).
  She walks: her +0x38 caps the move stick at 0.55 (0x13ceb0), as the scope does
  (FUN_001195c0), under the run's 0.65 (0x3a53f0, 0x11973a): no run, no sprint (`move_capped`;
  this frame's gait only, the inputs as read). Two of the demo's choices there: the AI's Flint
  walks too (the game's cap is on the stick's way), and the sprint (the game's dash) is barred
  with the pistol as well, where the game's dash command comes from the stick before the cap
  (0x134f52) and her own motions, which the pistol keeps, have a dash (not filmed).
* **The pose and the camera** (`Skill::raised`): raising the gun every frame holds its ready
  timer (char+0x7dc) at 4 s (FUN_00123f60), and the follow camera's aim shift (char+0x430,
  0x128280, play_camera.rs `step_shift`) is held at 3 s while that runs, for the controlled
  character: the camera in, 1.75 m along the crosshair's ray, as after a shot
  ([camera.md](../camera.md)). After the off the ready pose stays 4 s and the camera 3 s more,
  coming out over its last 0.67 s (s = clamp(1.5 x the timer, 0, 1)), 3 x as fast dashing. take17 (no enemy, no shot): in by
  2.27 s (on at 2.09), the rifle down at 15.5-15.8 s and the camera out over 18.13-18.6 s (off
  at 11.55). A reload after the off ends the hold at once: a fit to take29, where the reload
  right after the off left the rifle lowered once done and the camera out at 7.6-8.0 s, 2.3-3 s
  after it began; what clears the game's timer there isn't found (the reload keeps a raised gun
  raised, 0x120e63-0x120e6e; #126's answers Q11 read the reload's overlay as one that can't aim,
  which zeroes the timer: play.rs `can_aim`). Since #126 a shot's camera follows the same rule (4 s
  of ready timer, 2.33 + 0.67 s of shift); the demo's own post-shot pose (0.7 s,
  `Player::aim_hold`) is left as it was.
* **The lock box** (her +0xc4 and +0x100): each frame the target selection runs the controlled
  character's skill's +0xc4 (0xbd0c6); hers (0x13cd30) puts her box shares in the selection
  while on, h_108f0925 0.4 across and h_081520f0 0.4 down (the reticle's own 0.14 x 0.75 and
  0.14 back while off), and her +0x100 gives the box scale 1.0 in place of the gun's
  h_e1e41c2d (the L-Shot's 0.1, the Jax-iP's 0.6). So the box is 128 x 96 units each way round
  the crosshair, and an enemy 12 degrees off it is locked at the switch (take15). The crosshair
  turns red as usual, but fades out (below).
* **The guns aim at the lock** (every character's, not Flint's only; play.md § The crosshair's
  colours, step 4): with a lock the reticle gives the controlled character its point (0xc5b90 ->
  FUN_0011e9d0, char+0x804), the character update aims its guns at it (0x11aa20 -> 0x22dc60),
  and each muzzle's shot goes there (FUN_0022e5b0). That's the "aim assist" take28 measured, the
  reticle on the frozen Shrike's body: 1 hit in 8 off, 7 in 8 on. The demo's check differs in its
  setup: its bodies are solid cylinders the centre ray hits, so its shots on the body hit with
  the skill off too; it aims 7 degrees off the body instead, inside the box (below). Every gun is
  steered to the point: the demo's choice, as FUN_0022e5b0 leaves a weapon whose data +0x170 is
  set alone (which weapons have it isn't traced).
* **Damage** (her +0xec, 0x13cc70, in the roll FUN_0022c940, so every roll she makes: a shot's,
  a missile's and its blast's, a breakable object's): x1.5 (h_191dafb6) while scoped, on or off;
  while on, a roll of the game's random numbers at most h_18bf34d4 0.15 doubles it
  (h_fe103e1b 2). The two stack (x3). take28: one hit of seven doubled (80.6 against the others'
  42.7), the scoped ones x1.36-1.59. On top come the hit part's factor and the difficulty step
  (play.md § Damage).
* **Range** (her +0x54, 0x13ce80): her rifles (h_ec0b2194, WTYPE_RIFLE) reach x1.25
  (h_f82a974c), on or off: the shot's ray and its falloff, the crosshair's centre ray and the
  lock's reach (`weapon_range`). The L-Shot-75: 225 m in place of 180.
* **The crosshair** (her +0xfc, 0x13cce0): while on, its alpha is 0.9 x (1 - the marker's fill),
  so it fades out over the marker's 0.625 s (take15: red and dimming 2.4-2.8 s, gone after) and is
  back at the off in one frame (`Reticle::alpha`).
* **The marker** (the HUD widget h_121cf7bb her +0xc4 makes; `play_targeting.rs`):
  * Two bracket corners, the 64 x 64 texture h_e1b45a4c and the same turned half a turn, drawn
    1:1 in HUD units. Without a lock they frame the wide box, (192, 96) to (448, 288) round the
    crosshair (take17: the bracket lines at x 204 / 438, y 100 / 279). With a lock they frame
    the locked character: its bounding sphere (`Character::bounds`, the union of its skin mesh's
    geoset spheres: the Shrike Heavy's 1.24 m) projected, its size 640 x the radius / the depth in
    the game's 70-degree view, the box half that size across and three quarters of it down, at
    least 32 units, kept inside the HUD's safe area (game-options' 48 x 36 margins, 3 units in:
    in the scope the corners sit at the screen's corners, take15 14.6-18.5 s).
  * Switched off they head for the screen's sides, (-256, -96) and (384, 96) from the crosshair,
    so switched on they slide in from there. They move at 450 units a second inward and 900
    outward across, 400 up and down (take15: wide to tight 0.25-0.3 s, tight to wide 0.1-0.15 s).
  * With a lock, the glyph strip h_0f00136e over the top left corner shows the target's health,
    a glyph for each 15 hit points rounded up, up to 8 (all eight on the takes' 999-hp Shrike),
    and a third of the wave h_0e3d0ad2 scrolls under the bottom right one.
  * Its fill rises at 1.6 a second while on and falls at 2 while off (the takes' log: 0.56-0.61
    s, 0.46-0.57 s); its alpha is the fill, less up to 32% in a 4 s pulse (the takes' flag bit
    toggling every 2.06 s).
  * A lock while off (the crosshair's own, the pistol's on the enemy at take16's off) gives the
    corners a 64 x 64 box at the crosshair as the marker fades: take16 9.17-9.55 s shows them
    closing in, the strips still there.
  * A doubled roll draws the corners in red for a frame (the cyan texture x red: nearly black).
  * The game draws it additively; the demo blends it over with the alpha's sRGB look: the same
    over the takes' dark ground.
* **What ends it**: V; stamina at 0 (in that frame, take17); the gun in hand's clip at 0 (take29);
  Q, the switch key (the game's Y, take31 18.0 s: no switch); R, the reload key, when a reload
  could start (the game's X), the reload following; death. The black button ends it in the game
  (take31 10.0 s); the demo has no key for it. B, the item box, doesn't (take31 14.0 s). Each
  goes through the toggle, so in a jump (no type 45 transition) even stamina 0 waits for the
  landing.
* **Not free to act** (her +0x14, 0x13b980, "the skill is off", where the other three's say yes,
  0x213690): FUN_00117210, whether a character is free to act, ends by asking it, so while she
  tracks no reload starts (FUN_00120d40, 0x120e16), no weapon switch (FUN_0011e130, 0x11e135),
  no item use but an IOU_JUST_USE one (FUN_001210c0, 0x121139: a grenade thrown or placed, the
  medkit) and no melee (CT_ATTACK, 0x1226d2; the demo has none). The keys that end it come first,
  so the reload R asks for and the empty clip's start in the frame after the off. G (the game's
  LT) throws nothing and the medkit isn't used while she tracks: from the code, not filmed.
* **Not done**: her +0xf8, the tracking-reticule-speed 2 (FUN_000c5e20: how fast the crosshair
  moves onto a lock; hidden while she tracks); her +0x58, 0.8 for the scoped lock box in place of
  the base's 0.2 (0x11cfb5: the demo's scope keeps the gun's box for everyone: a #141 follow-up);
  the camera lock the tutorial describes (CT_TARGET_LOCK_TOGGLE 18 has no sender in retail, and
  take31 shows none).
