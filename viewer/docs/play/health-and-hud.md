*Part of [play/](README.md), the former page `play.md`.*

## Health, HUD, locomotion, weapons and sounds

Squad: the four squad members are all in play, each with their own health (the characters'
combat-target hitpoints: Tex 115, Brutus 105, Flint 90, Hawk 65) shown on the radar's diagonal
edges. Each is the game's health channel: its frame outline and its fill, two 32 x 32 sprites in
the diagonal between a portrait and the next, the fill showing the health left in whole hundredths
from the portrait's end and draining toward it as in the game (**The portraits' parts**, #354);
a hit flashes the portrait and the channel red. For whoever you control, health also shows in the
top-left bar. Grenade blasts hurt everyone in range (the explosion's Damage max, falling
off to nothing at its radius; the thrower takes a flat share, see **Grenades**), and every hit asks its hurt line of the chatter gate
(a pain grunt e856009f for a small one, "I'm hit" 0ae8104e, the half- and low-health lines, the thrower's own, a friend's: [chatter.md](../chatter.md)). Friendly
fire: your shots hit squadmates in the way (the weapon's damage, x0.7 on your own side, by the hit part struck and the difficulty: see **Damage**; their line the friendly-fire one, e2d9fb13 "Careful!"
/ "Stop shooting at me!", 10% with a 30 s replay delay), and the crosshair turns green while its centre is on one (see **The crosshair's colours**). At 0 health a
character dies: death cry (chatter e1213623: the game's CBT_DEAD handler FUN_00115f90 says it through FUN_00125f40 (0x125fb4) and the chatter gate (FUN_00140a60, 0x125fc9), or ed35ffae when the dying character's movement state +0x1f0 is 4, the state a landing leaves, FUN_0012b740: a death from a fall, inferred, the demo's for a fall's death; then the killer's kill line: [chatter.md](../chatter.md)), the body goes limp as the game's ragdoll ([ragdoll.md](../ragdoll.md), #259: a rigid body for each hit part and a small ball for each of their ancestors, joined with the game's joint limits, its masses, damping and 66 Hz step, every other bone, the face's among them, carried as it was at death; the level's invisible blockers and hinters don't hold a corpse: only a character's body meets them, so a knocked-down one, which stands up where its ragdoll lies, keeps them, #298; a body that stood still with no hit to kill it (a fall, a liquid, a Gas cloud's tick, `BF_TEST_DIE`) falls from the first frame, as the game's does (#366's take04, a Gas kill: mean speed 0.34, 0.47, 0.75, 0.97 m/s a frame apart; #259's 0.2 s hold is gone), and one that moved keeps its speed; a scenery area's kill, an Energy blast's and a damage plate's are thrown by their own records (a rack's blast 12.7 m/s, a plate's straight up) and a radiation barrel's cloud vaporises the body instead, no ragdoll ([ragdoll.md](../ragdoll.md), #366); a kill by a shot or a blast throws the body along it, 5-11 m at ~16 m/s the game's way (#307: two shoves the death asks for, and an up-kick a weak hit's body takes); a shot or a blast on a body moves it (a blast's shove throws a corpse's every body), and it bleeds where it's hit; `BF_TEST_DIE=<s>` drops the player dead at that time, once; `BF_TEST_DIE=1` at 1 s). Both their guns drop, the one in hand and the one on their back (the ragdoll's start drops whatever both slots hold, FUN_0014dfd0): each falls from where it was, thrown by half of what killed them plus a 1.5 m/s hop, and tumbles as a loose object (see "Pickups are loose"), then lies as a weapon pickup with the rounds it had (pickups.md "Weapon pickups", #17). Then the portrait greys out and the blip leaves
the radar; if you die, the death camera circles your body and control passes to the next
living member after 4 s (see **Death camera** below; `BF_TEST_KILL=<character>` makes the
player shoot that member). The inventory is shared: the
grenades go with control. Every portrait's tab shows its member's order (the follow arrows
until one is given: see **Radar**, "The command tabs"); a speech icon appears inside the
radar's ring, by the portrait's inner side, while that member talks, a "!" under it while the
member's special skill is on and a target ring while it is scoped (16 x 16 units, the texture 1:1
in pale blue; the game's table `ICON_AT` in play_portrait_parts.rs, #332's measured places for the
first, **The portraits' parts**, #355). The one you control is the player; the other
three are AI that follow you (the game's follow, #331, [squad.md](../squad/the-follow.md) "The follow": they
hold while you're within 11 m, chase a route to your point when you're farther, trail a dash about
14 m back and stop 9-10 m behind you), look where you look, and
reload on their own. They never fire just because you do - only at enemies they can see (by the AI's sensor: their vision-range, x1.15 for one they know, 90 degrees either way of their facing, a clear line; [brain.md](../brain/sight-and-contacts.md) "Sight and contacts", #197), within their gun's range (the level's characters on another side, and the test map's NPCs set to fight, see [testtools.md](../testtools/npcs.md)). Collision: everyone is a 0.4 m circle, pushed out of the pillars and apart from each other. Their footsteps and shots are heard quieter with distance, and they show as
green blips on the radar (turning with the camera, at the radar's scale: see **Radar** below). `BF_NO_SQUAD=1` (or `BF_DEATHMATCH=1`) plays deathmatch: alone, without the radar.

HUD health / stamina frame: the original's own art, the tutorial's picture of it (`h_17f34cbe`,
256 × 64, only in tutorial.tgz: loaded for it directly). It's cut in pieces: the + and bolt icons
(drawn solid), the frame's left cap, a middle that stretches and the right cap (drawn as glass, 45%:
Bevy blends see-through UI in linear light, so that's what matches a capture's body colour,
22 46 86, ours 17 51 91). Its baked bars are cleared, so the bars' empty part is see-through as in
the game (#35 take10: the sky shows through the empty stamina channel; it was the picture's navy
before), and the live bars drawn over them (rows 22–27 and 35–40). The bars are the game's bar
widget's length (FUN_000b7050: value / 150 × 249 units), 1.66 units per hit point of the
character's maximum health (`combat-target hitpoints`: Tex 115, Brutus 105, Flint 90, Hawk 65;
`BF_DUMP_HITPOINTS=1` lists them), from x 73.0, at least 2 units above 0: Tex's are 190.9 long,
x 73.4–263.9 in the demo and in #35 take01 (it was 1.6 per hit point from x 74, 3.5% short). The
stamina bar's value is stamina / max-stamina × max health, so it's as long as the health bar at
full. The frame's middle stretches with them and its right cap follows their end, and a squad
switch resizes it (the picture's right cap ends 12 units past the bars, the game's frame ~9.4:
the demo's frame is ~3 units longer on the right, #35 takes 01, 06, 19). The stamina bar's
colours, trail and low-stamina blink, and the skill's name under it: [skills.md](../skills/hud.md).
Without the tutorial's picture, the frame below is used. A HEALTH or STAMINA POWER walked over
when it can't do anything flashes the bar it would fill amber, four times in 1.2 s (the bar
widget's flash, 0x146900: `BarFlash`, [pickups.md](../pickups/power-ups.md) "Power-ups", #15).

HUD textures found for the panels: the health / energy frame is built from its corner
`h_0cdf5876` (32 × 16 end cap, mirrored four ways; it holds the step from the raised ends down to
the middle, whose level its inner column carries along), drawn as glass at 55% over a faint
tint. Its two channels are open in texel rows 6–11 and (mirrored) 20–25, columns 5–174; each bar
fills its channel exactly, and its icon is level with it. The bar fills (`h_e801997b` red,
`h_e11c7d04` blue, 4 × 8) are coloured in 6 of their 8 rows, matching the channels' 6; only those
rows are drawn. (Placed by hand, the bars sat over their channels' lines and left gaps at both
ends; drawn whole, the clear rows put the colour off-centre.) That corner isn't referenced in
the data and the demo picked it by shape; the HUD code names it, for split screen: h_0cdf5876
is the health bar's split-screen frame's left piece (0xb79f0 at 0xb7a53, with h_e0b02233 and
h_06a1d26e; full screen h_e5634eaf, h_eee38815, h_17470408). The item box's panel is the game's
own, the 64 × 64 h_e16927fb (FUN_000ab580, 0xab614; h_10891a71, 64 × 32, its split-screen one,
which the demo drew 9-sliced before #196): see **The item box and the item list**. (The
tutorial's `h_17f34cbe` is only a baked picture of the whole bar.) The radar's health channels are
not made from the radar frame's dark texels any more (a fit before #354): they are the game's own
sprites, **The portraits' parts**.

HUD (`src/bin/play_hud.rs`), as in the game: health / energy top left, weapon panel top right
(the weapon's own icon from its definition at its own size, 128 x 64 or the pistols' 64 x 64
from the panel's left, #64; clip / the squad's reserve, or "clip Regen" for a
recharging gun; every weapon listed with names for 2.5 s after a switch, the held one in orange; in Tex's berserk two rows, the gun in hand over the second hand's, both orange, [skills.md](../skills/texs-berserker.md)), radar with the squad's portraits round it (Tex top, Hawk left, Flint right, Brutus bottom; only the members in the world: **Radar**, "The portraits"),
the item box bottom right (`play_itembox.rs`, **The item box and the item list**), the message line under the middle (the HUD entry's message widget: "No need to heal", "Inventory is empty"; the body font at 12 in (192, 255, 255) with a shadow, its last 0.5 s fading, FUN_000b32d0 / 0xb33b0), and the held weapon's own crosshair (its definition's `reticule-prefix` texture, e.g. f4ea7a92 for the MK-ASLT), 64 units square, above centre (y 192 of 480) - aiming and shots go
through it; red, green, yellow or blue as the game tints it (**The crosshair's colours**, below). Layout is in the game's 640 x 480 screen (measured from an xemu capture) inside a
centred 4:3 area; textures are the game's own from `common.tgz` (stored upside down, drawn
flipped). The HUD is drawn by the main camera's child, the Vengar view's heat camera
(play_vengar.rs, marked `IsDefaultUiCamera`), over the world and the heat figures, whatever the
cameras' order. `BF_NO_HUD=1` hides it. Ammo, the switch hint and the crosshair's
size and hiding: see **Ammo, reload and the crosshair** below.
