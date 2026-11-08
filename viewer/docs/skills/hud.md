*Part of [skills/](README.md), the former page `skills.md`.*

## HUD

The stamina bar is the game's HUD bar widget (base FUN_000b6cc0, stamina subclass FUN_000b7bf0;
the health bar is the same base). `StaminaBar` in `play_hud.rs` keeps its state:

* **Length**: value / 150 × 249 units (FUN_000b7050), the value stamina / max-stamina × max
  health (FUN_000b7fb0), at least 1 when not 0, and the bar at least 2 units above 0: 1.66 units
  per hit point from x 73.0, as long as the health bar at full (see play.md's HUD frame).
* **Brighter while it changes**: the fill (h_e11c7d04) is mixed toward white by a brighten set to
  1.0 on a frame the value falls and 0.5 on one it rises (FUN_000b7850), fading by 0.5 a second
  (FUN_000b75a0). So it's white while a skill drains it, pale while it refills, white for a frame
  after a drop, and pale fades to teal in 1 s once full. The mix is in sRGB values, as the
  console's: the fill image is remade with it (Bevy would mix colours in linear light). Measured
  (rows y 63-67 near the start): white 248-254 against the takes' 246-255; pale (144, 187, 203)
  against take01's (140-142, 182-183, 201).
* **Trail**: when the value drops, the lost part shows in the plain teal texture from the new end
  to a shown value that follows the value down at 12 bar units a second (19.9 screen units) with
  no delay (+0x60 0 for stamina; the health bar's is 1.0, not done here). It follows a rise at 8,
  but nothing is drawn then. A steady drain shows none.
* **Low-stamina blink** (FUN_000b76c0): while the value is at most 15% of the max, a triangle
  wave between -0.5 and 0.5 at 1 a second, reset heading down whenever the value is above; the
  frame's lower half (the stamina widget's own frame, from the tutorial picture's row 30, y 57) is
  tinted while it's below 0: red for 1.0 s, normal for 1.0 s, red first. The tint is a red copy of
  those texels (their luminance in red, 0.15 of it in green, no blue, their own alpha), which
  matches the takes' red-phase pixels (~ (L, 0.15 L + 14, ~24) of the normal pixel's luminance;
  the game's own tint, bytes a0 20 20 ff at 0xb720a, combines in a way not traced). The bar
  channel's inside and the fill stay as they are.
* **The skill's name** (Berserker, Vengar, Stealth, Targeting: each skill's string id) under the
  bars in the message blue, from the switch-on frame, while on; after the off it stays 0.15 s
  (measured 0.13-0.18 s; what holds it in the code isn't traced) and fades at 3 a second (the
  widget's 3.0 at 0x39a848). The fade's alpha is raised to 2.2: the console fades in sRGB values,
  Bevy in linear light. It goes 20 units lower while the "switch to" hint shows (the code moves it
  down for each other message widget showing; which ones count, and the hint among them, is a
  guess: the takes never show the two together). A skill that never goes on holds its name itself:
  the Orbital Beam's press sets the widget's timer (`[view+0xfc]+0x74`) to 0.2 s (FUN_0013e910;
  `Skill::name_hold`), so the name shows at full from the press's frame for 0.2 s and then fades at
  3 a second (#200: take02 full 2.683-2.867 s, faint still at T0 + 0.4 s in take01 and take02).
* **Control passing** resets it to the new character's at once: no trail or brighten carried
  over, the name shown or hidden (take19 4.867 s, 14.867 s). Death: the name fades as at an off,
  the bar stops changing.
