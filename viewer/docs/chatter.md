# Chatter: the voice lines and their one gate (#155)

Every voice line a character says (a hurt grunt, a death cry, a kill line, the lines of a
hand-over, the Squad Command menu's order line, the line on reaching a Move To point, the
medkit's "healed" line, the refused gun's grumble, the skill lines) goes through
one gate and one queue for the whole world, as default.xbe's do: its 50 line call sites all call
FUN_00140a60 with the chatter manager the world owns (world+0xbe8, FUN_00140870). The values
come from each line's block in the speaker's chatter file. `src/bin/play_chatter.rs`; the
research is the analyst's `todo/36-squad-voice/leads.md` §1 and
`todo/35-special-skills/answer-120-chatter.md` §1.3-1.5; the functions are in
[code-map.md](code-map.md), "Sound".

## A line's block

`common/characters/<name>_chatter.xml` (a level's own for its enemy types: `shrike_light`...):
one `<chatter_line>` block per tag (`line_tag`), read by `bf/character.rs` (`ChatterBlock`)
with the defaults the game's parser sets (FUN_0019cd70):

| Attribute | What the gate does with it | Default |
|---|---|---|
| `probability` (h_ead503b2) | the roll, % (FUN_001374c0: 100 or more always passes) | 100 |
| `replay-delay` (h_ea21ae4b) | seconds the speaker can't say the tag again once said (char+0x7f4) | 0 |
| `priority` (h_e146dc86) | its place in the queue, higher wins | 5 |
| h_ff361b85 | the condition kind its variants' ranges test (below) | none |
| h_eecb5362 | as it starts it cuts off the playing lines of its priority or lower | true |
| h_183b88ed | the longest it waits behind the playing lines (s) | 2.0 |
| h_1800c848 | playing, it makes no new line wait | false |
| h_f4744ab4 | it can't be cut off | false |
| h_f1cb4e63 | a step of priority less beyond 15 m of the player | true |
| h_19be8951 | not said beyond 45 m of the player | true |
| `flags` h_eb640008 | said though the speaker is dead (the death cries) | - |
| `sound_id`s | the variants, each with its range of the condition value (0..100) | - |
| `<response>`s | who answers once its sound ends (below) | - |

## The gate (FUN_00140a60)

In the code's order; the first that fails refuses the line, and nothing is remembered:

1. a timed mute running (below);
2. the world's cutscene bit (+0xc50 bit 4: a script's 122 sets it, play_script.rs);
3. the speaker not in the world (a squad member its spawn-trigger hasn't brought, the editor's
   stand-in);
4. its replay delay for the tag still running (per speaker and tag; started when a line plays,
   FUN_00140cd0, counted down by the character update, FUN_0011c100);
5. no block for the tag in its chatter set;
6. beyond 45 m of the player's character, with h_19be8951 (0x8d660; the player's own lines are
   at 0 m);
7. the condition: the value of the block's kind must lie in one of its variants' ranges
   (FUN_001372c0, FUN_00137360):
   * none: 0;
   * "fighting" (h_12b74a88): the enemies the speaker's brain fights (FUN_0001b460: its
     contacts of kind 1 whose entry has bit 2, set once the contact comes within 15 m or passes
     a second test). The controlled character's brain keeps contacts too: the world's brain
     loop (FUN_000d9770) runs a brain that's off through its vtable +8 (0x1a220), which for a
     squad type a human player controls runs the same sense step as an AI's (FUN_00044480). A
     kill's line is asked in the death frame (FUN_00125f40), before the killer's next scan drops
     the dead contact, so the victim still counts: the player's kill of the last enemy can say
     02da3fee, whose variants all need 1 to 100. **A guess**: the more of the hostiles its sensor
     knows (an AI's contacts) and those it sees within its gun's reach, plus the victim for a
     kill line (counted whatever the distance: the entry's bit 2 isn't modelled);
   * char+0x5a8 (h_e7fad8d2): always 0 in the game;
   * health % (h_e63846de);
   * the alert value (h_e8f64b20): 10 for everyone (a brain's +0x770 is 0 until GOAL_PATROL's
     spotting, which the demo's brains don't run, and every character has a brain: char+0x830
     is set only by the brain's constructor and cleared only by the character's destructors);
   * the ammo ratio (h_19690a85, FUN_001265c0): the gun in hand's reserve over its clip size
     (a recharging gun: its clip x 5 over its size), x 0.2, held to 0..1, x 100;
   * the brain's mode (h_e3bdaee0): a squad member's by its command mode (squad.md, "Squad
     commands"): Cover Me 30, Stand Ground 20, Fire At Will 40, Move To 10, the others 0; 0 for
     the rest (no squad block uses it);
   * any other kind: the morale mix (FUN_001264a0): 0.4 health + 0.2 stamina + 0.1 the clip
     (1 full, 0 not, 0.75 without a gun) + 0.1 the skill (1 on, 0.75 off), x 100;
8. the roll: (the LCG mod 100) + 1 over `probability` fails (no draw at 100 or more; the
   draws below);
9. the priority: a step less beyond 15 m with h_f1cb4e63; 0 refused (a squadmate over 15 m
   away never says a priority-1 line: the skill lines, the medkit's);
10. the wait (FUN_00141020): B, the longest time left (+ 0.2 s) of the playing lines of its
    priority or lower that make others wait, and D, how much longer the lines of any priority
    play; it waits B + 0.3 s when that's within its h_183b88ed, else B, else D + 0.3 s or D,
    else it's refused. So with nothing of its priority or lower playing it goes now; behind a
    line it waits for its end (+ 0.5 s) when that's soon enough; behind one too long it goes now
    and cuts it off, unless a longer line of higher priority plays;
11. one waiting line for the whole world: a waiting line of its priority or higher refuses it
    (FUN_00140a40).

Passed, the waiting line is dropped (FUN_00141000: a new line of higher priority replaces it,
even one that plays now), and the line is built (FUN_001400d0): a variant drawn (the LCG mod
their count, FUN_00136fc0) among those whose range holds the value and that its chatter set
hasn't said yet (the manager's history, per set and tag: all are said before one repeats,
FUN_001371f0), lasting its sound's length + 0.2 s. It waits in the slot, or plays now (FUN_00140cd0): its speaker's replay delay
starts, and with h_eecb5362 the playing lines of its priority or lower that can be cut off
are stopped (FUN_00140f80).

## The queue (the manager's update, FUN_00140d70)

Each frame: the timed mutes run down; each playing line's events count down their start, then
their duration (FUN_0013f960); a line whose events have all ended is dropped; the waiting line's
wait runs down and it plays once below 0. With the cutscene bit set every line stops and the
waiting one is dropped.

A sound starting (FUN_0013fab0) plays on its speaker's voice, the face controller's (one per
character: a new sound stops the one it was saying, FUN_001435d0), unless the speaker is dead
and its block hasn't the `flags` h_eb640008, or the speaker's voice is saying a line of higher
priority (char+0x81c): then it's skipped. Its variant is marked said as it starts. A line cut
off stops its speaker's voice (0x13fb90). The HUD's speech icon by a portrait shows while that
character's voice speaks (`Player::speaking`).

**The mouth** (#325): the sound starting on a voice is said by the face controller too
(FUN_001435d0 at 0x13fb31: `voice_start` calls play_faces.rs `speak`), which plays the sound's
viseme tracks from the sound library on the face for the sound's length, and a cut-off stops it
(0x13fbde: `voice_stop` calls `stop`). The game's manager update runs before the characters' ticks,
the demo's after their step, so the line is put a frame back on the face's clock: from its second
frame its mouth reads the curve where the game's does ([faces.md](faces.md), "Lip-sync"). The face
keeps its own time left; `Player::speaking` is the chatter's.

The voice is heard in 2D at full for the controlled character (FUN_001435d0), and as the demo
hears a squad member's sounds for the others (quieter with the distance: the demo's mixing; the
game's is 3D at the character).

## The lines the demo says

| Line | Where the game says it | The demo |
|---|---|---|
| hurt lines (FUN_00125ad0's table) | every hit that reaches the hit points (CBT_DAMAGED), by the record: no attacker character and +0x5c bit 2 (falls, liquids) ef32191d; the victim's own (its blast, its gas) ff44e83f; a friend's e2d9fb13; else DTYPE_GAS 0feee721; else crossing a quarter of the max health downward 01e1dcd0, a half fab0be98, a hit of 15% of the max or less e856009f, a bigger one 0ae8104e | play.rs `hurt`, `burn` (each liquid frame's share too), the gas's and damage areas' and plates' per-frame hits; the attacker from the hit's shooter; a friend is one of the victim's team, a team not hostile to itself (the attacker's +0x1b4 vtable +0xc4, 0x3d2c0, as FUN_0004b960) |
| death cry | FUN_00125f40 in the death frame: e1213623, or ed35ffae when its movement state is 4 (a landing: inferred) | `Cause::fall` for a fall's death; Flint has no ed35ffae block, so a fall kills her silently. An enemy's from its type's chatter library |
| kill line | FUN_00125f40: a killer hostile to the victim, when their player slots differ (the victim's against the hit record's +0x1c, the shooter's: so one of the two is a player's, and an AI's kill of an AI draws none), says 13601f7b when its skill's vtable +8 says so (Tex's berserk, Flint's targeting on), else 02da3fee | the killer the hit names (`damage::Hit::by`); the controlled character has the one slot (as it is at the kill: a stand-in for the shooter's at the shot). Tex's, Brutus's and Flint's 02da3fee variants all need 1..100 enemies fought, which the victim makes in the death frame; Hawk has 0..0 ones too |
| hand-over | FUN_0005cc20: the one handed over from, by its brain mode: Cover Me (1) f415ab02, Stand Ground (3) 0e69bfe5 (e635ecc6 when the one taken is over 50 m away), Fire At Will (4) 15dc4e2d; the members its far case re-ordered answer (below) | play.rs `squad_control`, `Ask::HandOver` ([below](#lines-of-several-parts-the-order-line-and-the-hand-over-320)) |
| order line | FUN_000c1630, from the Squad Command menu's order dispatch FUN_000c1280: the giver calls the members ordered (or "everyone"), says the order and each answers | play_squad_menu.rs `give`, `Ask::Order` (below) |
| follow's walk | FUN_00021b70: the follow's walk to its place, once per order (1ecaeb98, 75%, replay 20 s, priority 2) | play.rs `squad_ai`: the formation's set-off after a Fire At Will, a stand-in for the game's follow states (#331) |
| Move To's arrival | FUN_00045fa0: a member on Move To reaching its point, or its walk failing (0x46256): e635ecc6, then GUARD | play_orders.rs `arrive` ([squad.md](squad.md), "Squad commands") |
| healed | 0x146f40, only when the medkit adds health: 12c35d69 (f832ad27 for a healer whose squad has no player) | play_pickups.rs: 10%, a 30 s replay delay, priority 1 |
| can't take that gun | FUN_000a9910 | play_weapon_pickups.rs: 100%, a 10 s replay delay |
| skill on / off / off at 0 | FUN_0010e470 / FUN_0010e3e0 | play_skills.rs ([skills.md](skills.md)) |
| charge hit | FUN_0013c1c0 | play_vengar.rs |

A type of its own (an enemy) says only its death cry here: the #129 takes' Shrikes said nothing
on their hits, though their files have hurt lines (the enemies' lines are #110's and #128's).
The #36 takes 07-10 show otherwise for a spawned Shrike shot by the squad: its hurt lines
(e856009f, 0ae8104e, fab0be98, 01e1dcd0) are gate calls, made by the same table; opening them is
`hurt_line`'s own-type test (#328).

## Responses

A block's `<response>`s whose range holds the speaker's value make a response event on the line
(FUN_001406f0), which fires as the speaker's sound ends (0x13fe50 -> FUN_0013fbf0): its responder
(for the death cries' `valid_targets` h_07b939a0: the speaker's nearest teammate, the dead
included) answers with one of its `response_id`s, appended to the same line after a roll on its
block's `probability` ((the LCG mod 100) <= it), when no waiting line has that priority or more;
no replay delay or distance rule of its own. The line takes the responder's priority, and the
waiting line is dropped.

The death cries' replies ("Tex is down", "Hawk is down": 0cceddf8, efc7d9fe, 0ace9296,
ea52e942) have only 0..0 variants of the alert value, which is 10 for every character with a
brain: they're appended with no sound, so **no squad member says them** (the code's reading,
leads §2.4; the xemu take A3 settles it). The demo said one half the time before #155.

Built for the death cries' responders only; the other `valid_targets` and `response_flags` are
the replies' ticket (#322). The order line and the hand-over (#320, below) join a second
speaker's block onto a line the same way (`Line::add_part`, FUN_00140060: no roll, replay delay
or distance check).

## Lines of several parts: the order line and the hand-over (#320)

FUN_00140060 joins one more speaker's block to a line the gate has just made: the tag is looked
up in *that speaker's* chatter set (type+0x1fc), and if it has one its sound is appended after
the parts already on the line, starting where the line's running total (line+0x28) stands. So
the parts play one after another, each starting its length + 0.2 s after the last one started,
whoever says it; there's no roll, replay delay, distance rule or condition test for a joined
part, and the line keeps its first part's priority and replay delay. `Chatter::join` is it. The
game's gate queues the line and returns it, and the caller joins at once; nothing runs between,
so `gate_made` hands the line out before `queue` puts it in the queue, and the callers join
in between. A line whose first gate call is refused isn't made, and nothing joins it.

**The order line** (FUN_000c1630, `Chatter::order_line`; asked by the menu's `give` for Stand
Ground, Fire At Will, Cover Me and Move To, the cursor's accept too, whenever any member took
the order, whether or not its mode changed: FUN_000c1280's call tests a flag set for every
member handled). The members ordered are in the squad list's slot order, Tex, Flint, Brutus,
Hawk, whatever order they were selected in (FUN_000c8700; #33 take03), and the giver, the
controlled character, is passed over if he was selected. With N of them:

* N of two or more and as many as the others of his squad (all three of Tex's three, both of
  two: 0xc1691): "everyone" (12df4fe6), the gate's call;
* else each one's **name tag**, the first the gate's call and the others joined, all said by the
  giver: the tag is the member's type's h_0d3bdcf5 (`CharacterType::name_tag`; Tex fb962749,
  Flint 116501dc, Brutus f0c93785, Hawk f5359692, the marines 181b6433), and the giver's own set
  has a block for each of the others' (a type without a tag is passed over, and a refused gate
  call tries the next member's name);
* then the order's **phrase** by the menu entry, said by the giver: 0 Stand Ground e104fe49,
  1 Fire At Will f5041bd5, 2 Cover Me 1cdff35c, 3 Move To f8046684 (the jump table at 0xc17ac;
  #33's leads had the first three the other way round: the takes' variants settle it, see the
  constants' doc);
* then each member ordered answers eaf3546f, said by that member.

All of them are 100%, no replay delay, priority 6, a 3 s wait, heard at any distance, and cut
off the lines of priority 6 or less playing (h_eecb5362). A new order while a line plays waits
behind it by the gate's arithmetic (within 3 s), cuts it off if it would wait longer, or is
refused when a line of its priority waits already.

The menu holds the game still while it's open and through its 0.3 s fade-out, and the chatter
manager is the world's: it isn't updated while the game's clock is held (`update`), so the
line is made when the order is given and its first sound starts in the first frame the game
runs again. The takes' clock is flat from the order to the fade's end and the first sound starts
in the frame after (#36 take12, #33 take03).

**The hand-over** (FUN_0005cc20, `Chatter::hand_over_line`; play.rs `squad_control` asks it
for the character left, with what `play_orders.rs hand_over` re-ordered). The gate's distances
are taken with the character left still the controlled one (the game asks before it passes
control: FUN_000c90c0 runs FUN_000c8f70, then FUN_000c8e80). The character left, alive, says by its
command mode: **Cover Me** (1) f415ab02, **Stand Ground** (3) 0e69bfe5, **Fire At Will** (4)
15dc4e2d, and nothing in the other modes; more than 50 m from the one taken (the far case:
play_orders.rs, squad.md) it has been set to Stand Ground and says **e635ecc6** instead. The
blocks have a 30 s replay delay (e635ecc6: none), priority 5 (e635ecc6: 6), a 3 s wait, no
distance cut. Then the members the far case re-ordered answer, joined to the line: each of those
set to Stand Ground with e97ce7fd (when no line was made, the character left says e635ecc6
himself first, and that member isn't joined), and of those sent moving out the first with
ed05c0d7 and the rest e97ce7fd (when no line was made, the first of them makes it itself).
The near case has no answers. Their order in the demo is its squad's, not the game's entries'
(not read). **The far case and the answers aren't filmed** (no take has a hand-over over 50 m:
#36 take14's was 40.9 m); they're the code's.

**The follow's walk** (FUN_00021b70, state 9 of the wingman follow). A member starting its walk
to its place says 1ecaeb98 through the gate (75%, a 20 s replay delay, priority 2), once per
order: brain+0x3c4 bit 0 is set as it's said and cleared by Fire At Will (FUN_00018000 for modes 0
and 4: the order, and a member left in Fire At Will by a hand-over), not by Stand Ground or Cover
Me. #36 take12 (Brutus) and #33 take05 and take06 (Flint) show it: the gate call comes in the
first frames the game runs after the order's (the clock held by the menu until then); in take05
Brutus asked it 11 s after the order, once Tex had walked off. Hawk, who followed Tex's 40 m
walk in take05, never asked it, so the game's follow starts its walks in other states too. The
demo's is a stand-in for the follow's state machine (#331): the first time since a Fire At Will
that a member's formation walk sets off (play.rs `squad_ai`, `Command::start_walk`), which is
usually when the leader next moves off. The bit starts set in the demo: the game's is clear
from the brain's constructor, so a fresh level's first follow walk would say it too, but every
take started from a snapshot mid-play and none said it before an order (#33 take04: a 40 m
walk, no line), so that case isn't built.

## The draws

The roll, the variant, the response's entry and its tag are each a draw of the game's LCG (the
state at 0x46cfd8, x 0x19660d + 0x3c6ef35f), the new state mod the count (FUN_001374c0,
FUN_00136fc0, FUN_0013fde0, FUN_001367e0, FUN_0013fbf0). In the game that state is the one
every random choice shares. The demo's chatter keeps a copy of its own (`Lcg`, from 0 at each
map's start), apart from each character's random state that drives the squad AI
(`Player::random`) and the shots' (`GameRandom`): a line's roll moves nothing else, so two runs
of the same test hooks match and a change to the chatter doesn't change a fight. (Before, they
drew from the speaker's own random state, the one its AI draws from.)

## Timed mutes

A line that hushes the chatter (h_e725a1a5) adds a mute for its length (FUN_00141100, keyed by
its trigger): every line is refused while one runs, and the first one stops the lines playing
(priority 10 or under). play_sound_triggers.rs adds them for a sound-trigger's sound (its stop
takes it off, FUN_00141180), a line of the world's dialog queue (Bratton's radio lines) and a
dialog-trigger's line. On e34 Bratton's lines at the start keep the squad quiet for most of
its first 20 s.

## Test hooks

* `BF_CHATTER_LOG=1`: each line asked (the time, who, the tag and what it is, the hit or death
  that asked it) with the check that refused it, or the line made, its variant and its wait
  (`now` or `waits r s`), its chance, priority, replay delay, condition value and distance; what
  the manager does (a sound started or skipped and why, a line ending or cut off and by what, a
  waiting line playing or dropped, a response, a timed mute); and the queue after each (the
  lines playing with their priority and time left, the waiting one, the mutes). A line of
  several parts (an order line, a hand-over with answers) is listed under its "said" row, a row
  a part: the speaker, the tag and its name, the variant, the part's start in the line and its
  length (`line #1: flint eaf3546f (order acknowledged) variant 19133e8a, from 1.81 s, 0.66 s`),
  then each sound's start as its turn comes. The same check
  refusing a character's tag again within 0.5 s of the last (a gas cloud's, a liquid's or a
  damage plate's hit asks its hurt line every frame; a burst's pellets) isn't logged again.
* `BF_CHATTER_CHANCE=<percent>`: every line's chance (100: each passes the roll).
* `BF_TEST_HURT=<character>,<amount>[:fall],<s>[,<s>...][;...]`: hits that squad member (name
  or 0-3, the controlled one too) for `amount` at those times, a bare hit with no attacker as the
  trainer's `hit` (`:fall`: a landing's record, ef32191d and the death cry ed35ffae).
* `BF_TEST_SPEAK=<s>,<who>,<sound>[;...]`: at `s` seconds (the controlled character's clock) `who`
  (`me`, a squad name or 0-3, `all`) says the sound (its hex id, `h_` or not) on its voice at priority
  10, as a line's sound starts (`voice_start`: the voice, the speech icon, the mouth) but with no gate
  or line; a character not in the world is left out. The lip-sync's test ([faces.md](faces.md)). The
  sound has to be in the map's loaded banks, its library entry (the tracks) and its file (the
  length): a line of a level's own bank says for 0 s elsewhere and moves nothing (the log shows the
  0 s). On the test map use a squad line of the common bank (Tex's skill-on line `05410be5`); take07's
  dialog line `ec50fb88` is the tutorial's (`BF_MAP=tutorial`).
* `BF_SKILL_CHATTER=<percent>` the skill lines' chance ([skills.md](skills.md)),
  `BF_TEST_HEALED=1` the medkit line's ([pickups.md](pickups.md)).
* `BF_SOUND_LOG=1` prints each voice started ("sound <id> vol <v> (the voice of <name>:
  chatter)").

## Verified

* The test map, `BF_TEST_FULL_SQUAD=1 BF_TEST_HURT="hawk,8,2,3,4;hawk,20,6;hawk,10,9;hawk,25,12;hawk,8,12.3;hawk,30,12.5" BF_CHATTER_CHANCE=100 BF_CHATTER_LOG=1`
  (Hawk 65 HP): the grunt said at 2.0 s and refused at 3.07 and 4.07 s by its replay delay
  (8.9, 7.9 s left); 20 at 6.07 s (41 -> 21, under 32.5) the half-health line, 10 at 9.07 s
  (21 -> 11, under 16.25) the low-health line; at the lethal hit its hurt line is made and
  skipped as it starts (dead, no flag) while the death cry plays through its flag; at the cry's
  end its response picks Brutus, appended with no sound (alert 10). The speech icon beside her
  portrait shows for each sound's length (2.0-2.4 s, 6.07-8.0 s, 9.07-10.7 s, and by her skull
  12.07-13.2 s).
* A fight on the test map (`BF_TEST_FULL_SQUAD=1 BF_TEST_NPC="1,brutus,enemy,fight,4.5,20"`,
  natural chances): Tex's grunts at 25% with their 10 s replay delays, the half-health line not
  rolled (75%), the low-health line (100%); his death cry and the enemy Brutus's kill line
  (an AI killing the player) rolled at 10%; the reply rolled for his nearest teammate; the
  enemy's hurt lines a step less past 15 m (21 m: priority 2 and 7); Flint's kill of the enemy
  (an AI's of an AI) asking no kill line; the enemy's death cry's response finding no teammate.
* The player killing an enemy dummy, the last enemy (`BF_TEST_NPC="1,brutus,enemy,dummy,2,-12"
  BF_TEST_AIM_HOSTILE=1.5 BF_TEST_TRIGGER=2-6`): Tex's kill line asked at the kill (1.73 s) with
  the condition value 1, the victim; at its 10% refused by the roll, with
  `BF_CHATTER_CHANCE=100` said (variant f94d25a1, priority 2) beside the victim's death cry,
  his speech icon showing 2.0-2.67 s. (Before, the value was 0 and no variant fitted.)
* The draws apart from the AI's: a test-map fight with the squad's AI shooting an enemy and
  trainer hits on Hawk and Flint every second (`BF_TEST_FULL_SQUAD=1
  BF_TEST_NPC="1,brutus,enemy,fight,1,-26" BF_TEST_HURT="hawk,3,2,3,...;flint,3,2.5,..."`), run
  at the natural chances (12 lines said, 4 not rolled) and with `BF_CHATTER_CHANCE=100` (13
  said, none refused by the roll): the shots, their rolls, the hits and the deaths match line
  for line (68 lines). e34's opening fight (the missile regression's hooks) matches over its
  first 8 s the branch's run before this change: its lines are all refused by Bratton's timed
  mutes or the distance, before any roll. Against the merge base (7f6b34e) the same fight runs
  the same until Hawk's first hurt at 5.40 s, where the old code's hurt line drew its variant
  from Hawk's own random state (her grunt 02c3b656 at 5.47 s in its sound log) and so moved her
  AI: from 6.2 s the two fights part. An A/B capture against a build from before #155 parts at
  the first squad line it says; between builds after it, the chatter moves nothing.
* The player shooting Hawk (`BF_TEST_KILL=2`, the test map's instant kill): her friendly-fire
  line rolled at 10%, her death cry, no kill line (a friend's kill).
* The skill lines (`BF_TEST_SKILL` every 14 s, 26 ons): Tex's on line said 14 times in 26 (the
  #35 takes: 13 of 26), his off line with stamina left never (takes: 0 of 10); its four
  variants all said before one repeats.
* A cut-off (`BF_TEST_SQUAD_SKILL="hawk,8" BF_TEST_HURT="hawk,8:fall,9.3"`): Hawk's skill line
  (priority 1) cut off by her fall cry (priority 4, no wait, cuts off), her speech icon going
  with it; Tex's fall death says ed35ffae; Flint's is silent (no block).
* The hand-over (`BF_TEST_SELECT=1,5.5`): Tex's follow line through the gate at the cut (9 m from
  Flint, 100%).
* The order lines (#320; `BF_TEST_FULL_SQUAD=1 BF_CHATTER_LOG=1 BF_SOUND_LOG=1
  BF_SQUAD_MENU_LOG=1 "BF_TEST_COMMAND=..."`, the test map, `BF_MUTE=1 BF_BACKGROUND=1`, a capture
  at 15 fps for its clock): take12's four orders (`2:flint:0.4;3:stand;10:brutus:0.4;11:fire;
  18:hawk:0.4;19:stand;26:hawk:0.4;27:cover`) and again on e34 after its radio mutes (`BF_MAP=e34
  BF_TEST_NO_HOSTILES=1 BF_TEST_GOTO=-83.39,-82.79,-83.39,-82.79,27 BF_TEST_GO_AT=999`, the same
  times plus 28 s): each one line of three parts, Tex's name call (Flint 116501dc, Brutus
  f0c93785, Hawk f5359692), his phrase (e104fe49, f5041bd5, e104fe49, 1cdff35c) and the member's
  eaf3546f, the first sound in the first frame after the menu's fade-out (0.07 s after the order
  on the chatter's clock, which the pause holds), the others at the sums of the sounds' lengths
  + 0.2 s (2.27, 2.73 and 4.07 s for a 0.51 s name and a 1.31 s phrase: the manager's first step
  counts against the later parts' starts). take03's (`2:hawk:0.4;2.9:brutus;3.5:flint;4.1:brutus;5:stand;
  11:hawk:0.4;11.9:brutus;12.5:flint;13.5:cover`): Flint and Hawk named in turn ("Flint!",
  "Hawk!"), Stand Ground, their two answers; then, three selected, "everyone" (12df4fe6), Cover
  Me and the answers of Flint, Brutus and Hawk in that order. The cursor's Move To to two
  members (`2:hawk:0.4;2.5:flint:0.4;3.3:move;5:accept`, `BF_TEST_CURSOR_AT=10,45`): both names,
  f8046684, both answers; their arrival lines (e635ecc6) then wait behind it (priority 6, one
  waits, the other refused for the waiting one). The speech icon by Tex's portrait for his three
  sounds, then by each member's.
* The hand-over's lines (take13's `2:hawk:0.1;6:tex:0.1;10:brutus:0.4;11:stand;17:brutus:0.1;
  21:tex:0.1`): Tex's f415ab02 at the first cut, Hawk's at the second, Tex's at the third refused
  by its replay delay (16.1 s left), Brutus on Stand Ground says 0e69bfe5 (variant 119385c9, as
  take13's). The far case (`BF_MAP=e34 BF_SCRIPT=off BF_TEST_NO_HOSTILES=1
  BF_TEST_GOTO=-83.39,-82.79,-25,-78,27 BF_TEST_SPRINT=4 BF_TEST_ORDER=hawk,guard@2
  BF_TEST_SELECT=2,26`, 54.6 m): Tex, set to Stand Ground, says e635ecc6, then Brutus (moving out
  to him) joins with ed05c0d7 and Flint with e97ce7fd; Brutus's arrival line (e635ecc6) waits
  behind it and plays after, Flint's is refused for the waiting one. At 14.5 s the same
  hand-over is refused by e34's radio mute, as every line.
* A heavy hit during an order line (take02's: `2:hawk:0.4;3:stand`, `BF_TEST_HURT=hawk,60,2.72`):
  Hawk's low-health line (priority 8) waits 2.70 s (the line's 2.20 s left + 0.2 + 0.3) and
  plays after the order's last part, not cutting it off.
* Fire At Will, then Tex walks (`2:brutus:0.4;3:fire;4:flint:0.4;5:hawk:0.4;6:fire`,
  `BF_TEST_GOTO=3.5,40,3.5,5 BF_TEST_GO_AT=9 BF_CHATTER_CHANCE=100`): as the three set off at
  9 s Hawk says 1ecaeb98 (priority 2), Brutus's waits behind it and Flint's is refused for the
  waiting line; nothing before the walk, and no such line after a Cover Me or Stand Ground.
* Other chatter unchanged, main (`8c384af`) against this branch, the same hooks at 15 fps: the
  hurt and death recipe of the first bullet above (74 log rows), a fight with hits on Hawk (71
  rows), e34's first 40 s with its fight, mutes and a Move To's arrival (374 rows) match row for
  row; the follow-mode hand-over (`BF_TEST_SELECT=1,5.5`) differs only by its distance (0 m: the
  gate measures it from the character left, still the controlled one, not 8 m from the one taken).
* e34 with the scripts off (`BF_SCRIPT=off BF_TEST_HEALTH=50 BF_TEST_ITEM=Medkit
  BF_TEST_MEDKIT=1.5 BF_TEST_HEALED=1`): Bratton's first radio line at 3.0 s mutes the chatter
  for 5.67 s, cutting off the three lines playing (Tex's friendly-fire line, two Shrike Hounds'
  death cries); the follow line at 4.6 s and the hits during it refused; the medkit line made but
  skipped as Tex's voice was on his priority-4 friendly-fire line (Flint's shot, 1.67 s);
  Flint's friendly-fire line (Brutus's shot) waiting 0.76 s behind Tex's, then dropped when a
  Hound's death cry passed the gate. With the scripts on, e34's opening cutscene refuses the
  medkit line (the cutscene bit). One of e34's Shrike Hounds is placed NEUTRAL (team 2), and
  its bites are a foe's (FUN_00125ad0's friend test, `attacker`), not friendly fire: theirs are DTYPE_GAS (4),
  the gas cough (the missile regression's hooks: Hawk's at 5.40 s).
* The tutorial (`BF_TEST_PANEL="a,1"` and the start area's walk, hits on Tex): the grunt's 25%
  and replay delays, the half-health line at 61 -> 55 of 115.

## Against the #36 takes

`todo/36-squad-voice/` (xemu, e34 with every enemy dead, gdb breakpoints on the gate and the
sound event's start; notes.md). Times are on the game clock: each take's watch log gives the
game time `gt` beside the video's, which also counts the VM's pauses at the breakpoints. So the
video's 0.3-0.5 s between a hit or a gate call and its sound (a death cry, a hurt line, the
follow line) is under 0.04 s of game time, the frame the manager starts the sound in:

* A3 (take03-05: Hawk, Brutus, Flint killed by one trainer hit): each one's death cry plays
  with the dead-speaker flag set (priority 9), and **no teammate reply follows in the rest of
  the take** (9.4-9.6 s of game time; 10.2 s of video): as the demo now does (the reply is
  appended with no sound, its variants needing an alert value of 0).
* A1 (take01: Hawk hit by the trainer): the hit that takes her under half her health says
  fab0be98 at priority 8, the same rule as the demo's (its variant f9f8e847, as the demo's
  first pick in the run above).
* A2 (take02): a priority-8 hurt line asked during a priority-6 order line waits and starts
  2.97 s later on the game clock (the video's 3.9 s), after the order's last part, instead of
  cutting it off: the gate's wait (behind a line of its priority or lower, within its 5 s). The
  demo's order line (#320) does the same: the hit 0.45 s into it (`BF_TEST_HURT=hawk,60,2.72`)
  makes the hurt line wait the line's time left + 0.5 s, 2.70 s, and play after its last part.
* D1 (take12: Stand Ground to Flint, Fire At Will to Brutus, Stand Ground then Cover Me to
  Hawk, each a tap-select, an order button): one gate call for each order, Tex's, with the
  member's name tag (116501dc, f0c93785, f5359692), then Tex's two sounds and the member's
  answer (eaf3546f), the phrase variants in e104fe49, f5041bd5, e104fe49 and 1cdff35c's lists
  (the tags' map, from the takes' variants). Each part's start on the game clock (`gt`, flat from
  the order to the menu's fade-out's end, the first sound in the first frame after) follows the
  sum of the sounds' lengths + 0.2 s before it, to within 0.037 s: after the name 0.505 s
  predicted against 0.468 s measured, the answer at 2.083 against 2.080; 0.816 / 0.799 and
  2.371 / 2.369; 0.546 / 0.526 and 1.854 / 1.844; 0.546 / 0.523 and 1.606 / 1.593. #33's takes
  (the game clock again): take03's two orders (two named, then everyone: 0.505 / 0.504, 1.051 /
  1.024, 1.865 / 1.839, 2.527 / 2.497; 0.726 / 0.720, 1.502 / 1.495, 2.333 / 2.321, 3.256 /
  3.238), take05's Fire At Will to all (0.726 / 0.705, 1.790 / 1.753, 2.620 / 2.594, 3.544 /
  3.528), take06's two (0.726 / 0.701, 1.790 / 1.767, 2.620 / 2.574, 3.544 / 3.518; the Cover
  Me to all, given while the first still had its last two answers to say with the menu holding
  the clock, waited behind them and began 0.56 s after the last ended, the gate's wait of the
  line's time left + 0.2 + 0.3 s: 0.691 / 0.667, 1.467 / 1.454, 2.129 / 2.103, 3.329 / 3.316),
  take08's Move To (0.546 / 0.528, 1.322 / 1.289). The demo's tags, speakers and order
  of parts are the takes' (the test map's runs above), and its variants are drawn, so the words
  differ from a take's by the draw. take12's Brutus also asks 1ecaeb98 in the first frames after
  his Fire At Will (gate call, priority 2): the demo's stand-in asks it at his next walk to his
  place.
* D3 (take13, take14): the follow line asked at the hand-over's cut (Tex, variant f71090f4,
  priority 5, its sound 0.04 s later on the game clock: 0.3 s of video), and refused at the next
  hand-over 11.8 s later (15 s of video) by its 30 s replay delay; Brutus on Stand Ground says
  0e69bfe5 when left (variant 119385c9); take14's Hawk, on Stand Ground 40.9 m from Tex, says
  0e69bfe5 too, not e635ecc6 (not over 50 m). The demo's lines are the same, by the mode of the
  one left (`hand_over_tag`).
* B1 (takes 07-10): the squad's AI kills of the Shrike ask no kill line (the slot rule), and the
  enemy's death cry is a gate call made with the speaker already dead.

## Not done

* The AI's combat lines and the replies other than the death cries' (#321, #322 from #36's
  plan), the lines' facial expressions (#326: the `<expression>` events; the lip-sync is done,
  #325, above).
* The order line's and the hand-over's parts have no expression or overlay events of their own
  (FUN_001400d0's FUN_001403c0 and FUN_001401e0: #326). The orders given by the level's scripts
  or the d-pad's tutorial hand-over to the marines (#265) don't exist here, so the marines' name
  tag (181b6433) is read but never said. The far hand-over's answers (e97ce7fd, ed05c0d7) and
  `e635ecc6` for it are the code's alone; their order among the members is the demo's squad
  order, not the game's entries'. The squad's size for "everyone" counts the members in the
  world, the dead among them (the game's squad entries; unread whether a death removes one).
  1ecaeb98 on a fresh level's first follow walk (the game's bit is clear from the start) and on
  EVT_LEADER_KILLED isn't built (above).
* A dialog-trigger's line isn't said on its character's voice (it plays as the triggers' own
  voice, play_sound_triggers.rs): in the game it takes the voice at priority 10 (FUN_001267b0),
  stopping its chatter; its mute still refuses every line while it runs. Its speaker's face says it
  (#325, [faces.md](faces.md)), but the chatter doesn't see it: a chatter sound started on that
  character meanwhile (a line without the hush flag) isn't skipped, and takes the face.
* The 4th argument of the gate (the line's other party) is read only by a response kind no
  squad block uses: left out.
