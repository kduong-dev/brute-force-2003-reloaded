*Part of [skills/](README.md), the former page `skills.md`.*

## Tex's Berserker (#121)

"Uses two weapons at once." (h_0bd0cbed). Tex draws the gun of his other weapon slot into his
left hand and fires both. `Berserker` in `play_skills.rs` (skill vtable 0x39d058, constructor
FUN_0013d440), the second gun's fire in `play_shots.rs` / `play.rs` (`Player::second`,
`SecondGun`), its motions in `play.rs` (`SpecialClips`); measured against takes 01-05, 21, 25
and 26 (spec Part 2).

* **The press** (his toggle, 0x13d490), either way, needs a berserk transition from the motion
  he's in (the current MOTION's transition for motion type 45, char+0x1fc's +0x534): in
  `ms_tex.xml` the stand, walk, run, side and back steps and the dash have one, so not crouched
  or getting up or down, in a jump, a dodge, a dive or a slide, nor knocked down. Switching on is
  also refused with a gun already in the second hand, scoped (char+0x7a8), when he isn't free
  (FUN_00117210), under 10%, within his 1.0 s timer, with no gun in the other slot, or when that
  gun's clip is still 0 after its top-up (take25 10.0 s: both guns and the reserve emptied,
  nothing happens). Switching off needs the second gun out and the timer run out. A refused press
  shows and plays nothing.
* **The top-up**: an accepted press switching on tops the other slot's gun up from the squad's
  reserve, min(clip size − clip, reserve) (0x13d643-0x13d692). take25: the Jax-iC 2 → 36 and the
  reserve 200 → 166 at the press, before the switch; the demo the same. A recharging gun (the
  LZRs) has no reserve and isn't topped up. "Out of Ammo" (h_ff471882, 0x13d6e2) can't show:
  the code refuses without a second gun before it gets there.
* **The clips are the type's, not the squad name's.** The press's `Sc_w1_bz_start` and the off's
  `Sc_w1_bz_stop` are `ms_tex`'s `<overlay motion-type=44 / 43>`, which the game's command handler
  reads from the character's *type* (the type's overlay list, type+0x2cc, 0x122500-0x122587), so
  every type with `ms_tex` plays them: the squad's Tex h_fc310d72, the deathmatch library's Tex
  h_e00f4e1d and sdm_bf's h_fa6ca831 (the "Brute Force" squad of the test map's squad menu,
  `BF_TEST_SQUAD_SLOT="0.3,me,tex@sdm_bf"`), the last two types of their own in the demo
  (`Player::own_type`: `data_name` ""). #407's `own_type_skill` gave a type of its own the Berserker, and
  `spawn_unit` looked the clips up by the squad name (`Game::skill_overlays` keyed by "tex"), which a
  type of its own hasn't got: the two deathmatch Texes switched it on and off in the press's own frame,
  the second gun appearing in his left hand and vanishing from it with no reach to the rack and no
  put-back. They look them up by the type's motion set now (`Game::type_skill_overlays`, keyed like
  `motion_sets`); Kingman's `ms_kingman` has no overlay of type 44 or 43, so his Berserker stays
  clipless as the demo has it ([his page](other-characters-skills.md)). Before #407 those two Texes had no
  Berserker at all (a type of its own had no skill but the effect skills': at ce07a6a, the commit before
  it, V on the sdm_bf Tex logged nothing and did nothing), so this wasn't a clip lost from a working
  animation: the campaign Tex's draw and stow are pixel for pixel the same at ce07a6a, at 5482136 and
  after the change (the Tex region of 150 frames of `BF_MAP=sdm_e34`, on and off).
* **The motions** change at the press (the toggle asks for motion type 45, FUN_0012eab0, at
  0x13d6ca): idle, walk, run, the side and back steps, the jump and the slide become
  `Sc_w1_bz_*` (`SpecialClips`; prefix-type 0, the same with either gun in hand), back at the
  press switching off. The berserk set has no transition to a crouch, a dash, a dodge or a dive,
  so in berserk Z doesn't crouch, Shift runs (`bz_run`) and C doesn't roll. A crouch pressed in
  it is dropped, not kept for after the off (kept, he would drop into a crouch at the off): the
  demo's choice until shot T3 (crouch in berserk) shows the game's. `Sc_w1_bz_start` /
  `bz_stop` (upper body, 1.0 s) play over them (#120's toggle).
* **The switch** (0.50 s into `bz_start`, the event eaba3d48 at 0x118ae7, after the activate):
  the other slot's gun goes into his left hand (FUN_0014d090), held by its trigger grip at his
  `SECOND_HAND` hardpoint h_15ab6d29 (the inventory's second-hand holder names it at 0x14c7b8),
  and loaded from the reserve at once if it isn't full (FUN_0022daf0 with 0). The weapon box
  lists both guns with their names for `HUD_LIST_TIME` (take01: 2.750-5.417 s), then shows two
  rows, both in the held guns' orange: the gun in hand on top, the second gun below (take04 8.5
  s). At the off event (14ee2201, 0x1180bb) the gun goes back to its slot (FUN_0014d1a0) and the
  box lists the weapons again, the stowed one in blue (take01 18.767 s).
* **Fire**: the fire command sets both guns' fire flags (0x122621-0x122655), so the trigger fires
  both from the same frame, each at its own rate from its own clip, with its own cooldown, burst
  counter, accuracy and muzzle effect (`play_shots.rs`: `FireState::run` steps every gun in the
  same game frames). Each gun's shots fly from its own muzzle to the one crosshair's point and
  hit with its own damage (each `Shot` carries its gun's Damage, damage-type and ammo-type). No
  damage factor: berserk's firepower is the second gun. With the gun in hand empty the second
  fires on alone (take05), and a recharging second gun keeps recharging. A tap fires both: the
  press's request sets both guns' fire flags each frame until an animation event clears it
  (0x0f8408d5 at 0x117b12, in a clip not placed), and the demo keeps it for each gun until that
  gun's first shot (`FireState::pull`), so one still cooling down fires when it's ready. A
  minigun's barrels spin in either hand.
* **The crosshair's lock** reaches as far as the shorter-reaching gun, measured from its muzzle
  (FUN_000bf7a0 at 0xbf82a-0xbf8aa keeps the smaller of the two guns' FUN_00232520 x 0.7, and
  that gun for FUN_000bff40 at 0xbfb14): with the LZR-50 in hand (63 m) and the Jax-iC second
  (42 m), the Jax-iC's, from his left hand. The lock box and the centre ray stay the gun in
  hand's (play.md § The crosshair's colours).
* **The drain** (0x13d720): `stamina-cost` × dt, × 0.5 (game-options' `<combat-skills>` block
  h_04aa86aa, h_1469a0df, `CombatSkills::berserk_empty_drain`) while the gun in hand's clip is 0.
* **What ends it**, each through the toggle, so `bz_stop` plays and it goes off 0.5 s later: V;
  stamina at 0; the second gun's clip at 0 (his update FUN_0013d780, once the timer has run out);
  Q, the switch key (the game's Y: the controller sends the toggle instead of the switch,
  0x134db4-0x134dde); R, the reload key (the game's X: 0x134c50, when a reload could start,
  FUN_00120c50; the reload asked for stays, +0xa1 bit 2, and follows berserk's end); death. The
  game's X ends it when let go (0x134c2b-0x134c4a: held, but less than its hold time; take26's
  off came about 0.6 s after X came up: whether to the log or the picture the notes don't say,
  so it holds 0.02-0.06 s or 0.14-0.21 s of input delay and lag, #140), the demo's R on the press: the demo's choice (its R
  reloads on the press too). The black button (the medkit shortcut, 0x134367) ends it the same
  way in the game; the demo has no key for it. G (the game's LT) doesn't: it throws or uses
  nothing (take26 20.5 s).
* **While it's on** no other overlay starts (his overlay filter 0x13d850, asked for each queued
  overlay at 0x11a507): no weapon switch, grenade throw (the count stays) or medkit. And no
  reload: the character's reload (FUN_00120d40) returns at once while a gun is in the second
  hand, so an empty gun in hand clicks dry under the trigger (each gun's empty cycle clicks with
  its own empty sound) and reloads once berserk is over (take05: the reserve taken at 15.19 s).
* **No scope** from the accepted press to the second gun's stowing (his toggle only refuses a
  press made scoped, char+0x7a8): the demo's choice until shot T2 (the scope in berserk) shows
  whether the game lets him. Scoped, the second gun's muzzle effect washed the view purple and a
  shot in the start clip left from behind his shoulder.
* Handing control over, as for every skill: it stays on, the AI fires both guns, and the stamina
  drains (take19's rule).
