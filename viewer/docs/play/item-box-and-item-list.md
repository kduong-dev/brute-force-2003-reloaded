*Part of [play/](README.md), the former page `play.md`.*

### The item box and the item list (#196)

`src/bin/play_itembox.rs`, from the player controller's item section (0x1332a0), the HUD item
widget (FUN_000ab8d0, its update 0xabc60 / FUN_000ac220) and the inventory front's selection
(FUN_0014d960, FUN_0014db60); each function is cited in the code and has a row in
[code-map.md](../code-map/README.md). Measured against the takes in `todo/196-inventory-hud/` (its
`spec.md`, AC-01 to AC-35).

* **The selection** is the inventory front's wanted pair (`Player::item`; none is the game's
  (10, -1)), one stack of the squad's store at a time. The order is the store's, category then
  index, round: Frag, Gas, Sonic, Energy, Light, Sentry, Roller, Medkit (80), Medkit (60),
  OrgSen, Hat. A stack is listable while it holds any (one that reaches 0 loses its type) and its
  type's h_1852ab6c lists it. It goes with control: a squad switch or the death camera's hand-over
  copies it to the new member (FUN_000c8fa0; take17, take29: every member's box reads the item).
* **Each frame of the world** (FUN_0014db60): a none selection becomes the first listable stack
  (a level's start: e34's Sonic 4, e40's Frag 5, the tutorial's Frag 3); the shown pair, which
  "can use now" checks, follows the wanted one unless a throw or a Medkit's use is under way; a
  shown stack that ran out sets both to none, so the box is blank for that frame and shows the
  first stack the next (take09: "Sonic", one blank frame, "Frag 5"). A pickup never moves the
  selection (take34: the box stays on the Sonic while the Medkit stack fills).
* **Tab, the B button** (0x1340e1): held, its timer counts and a grenade's charge stays 0; let go
  under 0.25 s (0x3a5028) it steps to the next item, with the step's sound when it changed
  (FUN_000abfc0); at 0.25 s the item list opens (FUN_000ac080: its sound, the game paused,
  `PAUSE_ITEMS`) and stays until Tab is let go (FUN_000ac160: its sound, unpaused). Nothing is
  picked at the release: the item is what the list was scrolled to. The pause stops the world as
  the menus do (`frame_dt` 0: the characters, grenades in the air, effects, timers); the item
  widget runs on the HUD's own clock meanwhile (`hud_dt`: take02 slides it while the app clock is
  still). Movement, look, fire, G, B, the skill key and the 1-4 keys reach nothing while it's
  open.
* **The scroll** (FUN_000ac370), while the list is out past 0.75 and Tab is held: the stick past
  0.5 on its larger axis, right or up the next item, left or down the previous; a step at once,
  then one each time its ramp passes 1 at 4 a second (0.25 s and a frame: 8 frames at 30 a
  second, take08). The keyboard's stick: W or D the next, S or A the previous (the product
  owner's call D1); the wheel a step a notch, up the next (D2): the demo's, as the game has no
  wheel, so its notches are queued while the list is open and stepped one a frame once the
  scroll runs (none lost to the slide's first 0.375 s or to two in a frame; a fine wheel's
  fractions of a line and a touchpad's pixels, 100 a notch, add up to whole ones). The pad's
  left stick and d-pad are #154's (`ItemInput`, which the keyboard fills now).
* **G, the game's LT**: nothing selected, "Inventory is empty" (shown again each frame it's held,
  0x1349c3, so 3 s from the release); the shown pair not the wanted one (a Tab tap during a
  throw), "Not ready" (code only); else a grenade charges and is thrown on the release, the rest
  are used on the press. A Tab or B press while G is held leaves it dead until it's let go
  (ctrl+0x68: take19, take12), and so does a use (ctrl+0xa0 bit 7); once G is up the lock is
  free again (0x1346a7), so a use never leaves G or B dead.
* **B, the Black button** (0x134316): a charge dropped; Tex's or Flint's skill on: switched off
  instead (#35); no Medkit (5, 0): "No Medkits" (the 60-point one doesn't count, take11); else the
  Medkit selected (the item before it saved) and used the next frame as if G were pressed
  ("No need to heal" at full health), and on the release the saved item back (take10: the box
  Medkit, the Sonic back as Black is let go). Held on, another Medkit each time the last one's
  use is over and the character is still hurt (FUN_001210c0 with the Medkit's own "can use",
  0x147070; code only). The controller is the player's, not the character's: a hand-over during
  the hold gives the saved item back to the new member at the release (FUN_000c8fa0 leaves it).
* **The box** (FUN_000ab460, FUN_000ab580, FUN_000ab740): at (528, 375), 64 × 64 units, hidden
  while nothing's selected (take21). Its panel h_e16927fb (64 × 64, white); the type's icon
  h_e5ec3f1f 64 × 64 at its corner, grey while the character can't use it now; the name (the
  type's stringtable-name) centred, its top at 45; the count right-aligned 4 in from the edge,
  its top at 31, shown from 2 up; the body font at 12 (the 12-point record, play_fonts.rs) with a
  shadow; (192, 215, 255), or (150, 70, 70) while it can't be used now: a Medkit at full health,
  the OrgSen or the Hat for Brutus (take01, take10, take26). A dead member's box isn't red
  (take18). Its character dead, the box fades out from the death's frame (0xabc60 sets its fade
  rate, +0x11, to 127 a second; FUN_000ba730 takes it off the fade level, +0x12, in whole
  levels: 5 a frame at the game's 30, so out in 1.7 s) and stays out through the death camera;
  the next member's box is at full at once (take18: the icon gone 1.66-1.73 s of the game's
  clock after the death, the picture frozen twice just after it as the game hitched; Flint's
  box at full at 6.9 s).
* **The list** (FUN_000ac500, FUN_000ac5e0, FUN_000ac7f0): its open level rises 2 a second while
  it's open and falls 3 after, on the HUD's clock (out in 0.5 s, gone 0.33 s after the release);
  up the next item and the one after, left the previous and the one before, at int(open × 70)
  and twice that from the box, faded in as open³; their panels (30, 30, 30) at 153. A slot showing
  an earlier one's stack is hidden in the order box, left 1, up 1, left 2, up 2: one stack shows
  no slot, two left 1 only, three left 1 and up 1, four all but up 2 (take25's logs).
* **NEW** (0xad0c0): every stack has the flag from its making (FUN_0014ac90); the first slot to
  show it (the box, or a list slot as the list opens) clears it and runs its label, 2 s on the
  HUD's clock: "NEW" (h_0ba41874) in (255, 160, 106) at (10, 2), on while fract(2 × time left)
  ≥ 0.3 (0.35 s on, 0.15 s off), its alpha 255 × min(1, 2 × time left). A slot's label stops when
  its stack changes. At a level's start the stack shown first is clear and every other set
  (take27: the first box has no label; take28 / take32: the first taps show NEW). A hidden list
  slot counts too, in the widget's child order box, up 1, up 2, left 1, left 2 (FUN_000b21f0 and
  0xad0c0 don't test the slot's hidden flag): with Sonic selected and Energy and Roller NEW, the
  hidden up 2's Roller takes Roller's NEW and the left 1 that shows Roller has no label (code
  only: no take has a short list with NEW stacks).
  * **A pickup and NEW** (#19, #21): the flag is set only by the stack's constructors (FUN_0014ac90
    and its twin 0x14b540; nothing sets it when a stack is typed or emptied again), so a pickup
    needs no NEW of its own. A stack nobody has shown yet, a Gas stack of 0 that a pack fills or
    the OrgSen's, still has it: the first frame the box shows it "NEW" blinks beside its count
    (take04: "NEW Gas 3", then 6 and "NEW 9" as the packs come in a half second apart, the
    label's 2 s on from the first; take09: "NEW OrgSen"). A stack that ran out and is filled again
    shows none. A pickup doesn't move the selection but into an empty box: with nothing selected
    the box takes the first stack, the pack's (take04: "Gas" selected by the first pack).
* **The messages**: the item ones show for 3 s (FUN_000bcfe0(entry, 3.0)), restarted by each
  press, on the message line (above): "Inventory is empty", "No Medkits", "No need to heal",
  "Cannot use", "Not ready". take26's "Cannot use" stays on through presses 3 s apart (on 6.2
  to 15.15 s of video) and is gone 2.8 s of the game's clock after the last (the watch log's
  clk: 682.48 s at the press, 685.27 as the line's ink goes), where the demo's takes 2.87 s to
  the same threshold; the take's notes' "~6.5 s each" is that run of presses.
* **The sounds** (FUN_000bb220, 2D at their library volume, h_eff0deab 0.7): the list's open
  0x18fc2db8 and close 0xef4bd74d, the next item 0xf8f93c5e and the previous 0x1ab2fc23 (only when
  the item changed). Logged with `BF_SOUND_LOG`; not heard in the takes (their game sound was at
  the noise floor), so their loudness isn't checked.
* **Not done**: the pad (#154); the weapon panel and the skill's name showing while the list is
  open (take02, the product owner's call D8: a follow-up); split screen (64 × 32, no icon); the
  OrgSen's and the Hat's uses (#21); "Can't use this item." (FUN_001301e0's use types per
  character, not traced: only Brutus's refusals are in, his skill's "Cannot use"); "Not ready"
  for a character that isn't free (FUN_000abbb0 asks FUN_00117210 too: G pressed during a throw's
  motion, a Medkit's use, a reload or Flint's targeting; only the pair's part is in); the rest of
  the HUD's death fade (the health bars, the weapon panel); whether the HUD's other timers (the
  message line, the throw meter) run under the list's pause (they stop in the demo); the sound
  groups paused under it (FUN_0007a360); Hawk's special pair (10, -2); the two silent LT presses
  of take13 / take15 (the code's alternating ctrl+0x68 explains them, code-map's 0x1332a0 row;
  not copied, as the spec asks).

Test hooks (the hooks' clock is the sim time plus the time the list's pause held it, so a hold's
end comes while the world is stopped):
* `BF_TEST_ITEM_INPUT="<s>:<input>:<hold s>;..."` holds an input from that time: `tab`, `lt` (G),
  `black` (B), `next`, `prev` (the stick); a hold shorter than a frame still shows for one.
  `<s>:wheel:<notches>` turns the wheel that many notches in one frame (up +, down -).
* `BF_TEST_ITEM_LIST=1` holds Tab from the start (the list opens and the game pauses for good).
* `BF_TEST_NEXT_GRENADE=<s>[,<s>...]` taps Tab (0.1 s) at each time (the T key it pressed is
  gone: the product owner's call D3). `BF_TEST_THROW` (with `BF_TEST_GOTO`) holds G.
* `BF_TEST_INV=<name>:<n>,...` sets the pool at the start: a grenade type's label, `Medkit` (the
  (5, 0) stack), `Medkit60` (the (5, 2) one), another listed item's label (`OrgSen`, `Hat`); 0
  empties a stack, the rest keep the store's; a grenade type the level defines but neither the
  squad nor SQUAD_GRENADES has is loaded for it (e34's Sentry). `BF_TEST_INV_NEW=<name>,...`: the
  stacks still NEW at the start, every other clear. #196's nine-stack pool:
  `BF_TEST_INV=Frag:5,Gas:5,Sonic:4,Energy:5,Sentry:3,Roller:4,Medkit:2,Medkit60:2,OrgSen:1
  BF_TEST_INV_NEW=Gas,Energy,Sentry,Medkit60 BF_TEST_ITEM=Sonic`.
* `BF_ITEM_LOG=1` prints the controls' events (taps, the list, Black, uses, refusals, a stack
  running out, NEW) and the state on every change and every 0.1 s while the list is out: the hooks'
  time, the sim time and the HUD's, the open level, the timers, the selection and the shown pair
  as (category, index), the five slots (box, up 1, up 2, left 1, left 2; `h` hidden), the lock,
  Black's saved item, the counts with their NEW flags, the labels' times.

Verified (e34's range spot `BF_MAP=e34 BF_TEST_GOTO=-53.56,30.81,-53.56,30.81 BF_TEST_SWEEP=167.8
BF_CAMERA_PITCH=-0.0093 BF_TRIGGER_FIRE=trigger_letterbox204,12,0.05`, the nine-stack pool where
it says, `BF_ITEM_LOG=1`; frames tiled beside the takes'):
* The box: "Sonic 4" beside take25 at 1.5 s: the icon's orange at the same box-relative x
  17.3-45.3, y 9.0; the name's and the count's ink within a unit of the take's. Ten taps 1.5 s
  apart (take01): Energy NEW, Sentry NEW, Roller, Medkit, Medkit (60) NEW, OrgSen, Frag, Gas NEW,
  Sonic, Energy, each with the next sound; the Medkits grey and red at full health, the OrgSen
  teal, as take01's frames.
* The list, Tab held 2.0-5.0 at 30 fps: open 0.267 s after the press (8 frames), the sim time
  still from then to the release; the slots 2,4 2,5 3,3 2,1 2,0 as take25; NEW on Gas, Energy and
  Sentry as it opens (take24), their labels blinking and gone 2 s after the open, frame for frame
  as take02 at 10 fps; out at 0.533 s, back in 0.33 s. Two, three and four stacks: take25's slot
  logs. A Frag thrown before the open hangs in the air through it (`BF_GRENADE_LOG`: down 0.33 s
  of sim time after it left the hand, which spanned the 2.8 s pause).
* The scroll (take05 / take08): steps 0.267 s apart while held, two for a 0.4 s push, none in
  the slide's first 0.375 s, the item kept at the release.
* Taps of 0.10, 0.18, 0.22 s step, 0.30 and 0.40 s open the list (take04; at 30 fps a 0.24 s hold
  is 8 frames, 0.267 s, and opens it). A tap while a Frag charges: the charge dropped, the Gas
  selected, nothing thrown though G stays down (take19), a later press throws the Gas. A tap in
  the throw's motion: the box shows the Gas at once, the shown pair follows as the grenade leaves
  the hand, G meanwhile "Not ready".
* Black at 60 hp: Medkit selected and used (6 to 5, 115 hp), the Sonic back at the release; at
  full health the grey and red Medkit and "No need to heal" (take10, its message's ink at the
  same x 269.3-370.0); the (5, 0) stack empty: "No Medkits", the box unchanged (take11); during a
  charge with G held to 4.0: the charge dropped, the Medkit used, the Frag back, no throw (take12);
  Tex's berserk on: the skill switched off.
* The last Sonic thrown: "Sonic", one blank frame, "Frag 5" (take09); a third throw takes a Frag.
  Nothing carried: no box, "Inventory is empty" at y 292.0-306.7 as take21; Tab held opens the
  list with no slot. The tutorial's medkits taken: the box stays "Frag 3", taps reach Gas and the
  Medkit, each NEW (take34). Brutus with the OrgSen: grey, red, "Cannot use" (take26). Tex taps
  to Energy, control to Hawk: her box "Energy 5" (take17). NEW's blink at 60 fps: 21 frames on,
  9 off, three times, then the fade (take23: 22 / 11 / 24 / 8 / 24 / 9 video frames at the take's
  0.9 speed). e40's first item Frag 5, the tutorial's Frag 3.
* Regressions: `BF_TEST_MEDKIT`, the 60-point Medkit healing 60, the test map's kits and
  `BF_TEST_NEXT_GRENADE`, `BF_AUTOPILOT`'s throw at 29 s, `cargo test --bin bf_play`.
* The use lock (`BF_TEST_HEALTH=10`): the Medkit selected, G heals (90), Black after it heals
  (115), Black again says "No need to heal" (the log: the lock on the Medkit at the press,
  "another" while G stays down, free at its release); a one-frame G (`BF_TEST_ITEM_USE`) then
  Black; two Black presses with the Medkit current; Tab three times to the Medkit, G, then Black.
  Black held 3 s: a Medkit (90), another once the first use was over (115), the Sonic back at the
  release. A switch to Hawk asked during a Black hold (`BF_TEST_SELECT=2,1.2`): her box on the
  Medkit until Black is let go, then "Sonic 4".
* NEW in a hidden slot: Sonic, Energy and Roller (`BF_TEST_INV=Medkit:0,OrgSen:0`), Tab held: up
  1's Energy shows NEW, left 1's Roller none (the log: NEW on the hidden up 2).
* Brutus's "Cannot use" with G presses 3 s apart: on 3 s from each press, its ink gone 2.87 s
  after the last (take26: 2.8 s of the game's clock).
* The death box (`BF_TEST_DIE=2.0`, the Energy selected, 30 fps): the icon's and the name's
  brightness fall in a line, out 1.7 s after the death (take18: 1.66-1.73 s of the game's clock),
  out through the death camera, the next member's box at full at the hand-over.
* The wheel (`<s>:wheel:2` 0.1 s after the list opened): both notches stepped once the scroll ran,
  a frame apart; one back later.
