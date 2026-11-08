*Part of [maps/](README.md), the former page `maps.md`.*

* **Gates, their wall panels and consoles.** A gate with wall panels or consoles opens from them:
  * The panels are `world-button-object`s (h_16f22d4b, type h_ee4c83c9, `reticule-action` 1); the
    consoles (#262) are plain `game-object`s with `reticule-action` 1: 37 in 14 zones (e34's two,
    e23's three, e16's eight, m08_a's two, m08_b's two, m09_b's three, m09_c's two, m09_x's three,
    m14's eight, m04_c's three, m02_b's one), of ten types with their own names (h_1dd3a49d):
    "console", "Shrike console" (m09's and m14's), "panel" (e16's four), "control pod" (the Seers'
    shield consoles of m02_b and m04_c: the shield itself is #236's) and "Meteor Control Center"
    (m14_c's and m14_d's). Used, a panel sends its signal (h_eaf8a35b: 14, USE_OBJECT) and data
    (h_f1660a0d) (0x163060), a console USE_OBJECT with data -1 (FUN_00158140, a plain game object's
    use), to the triggers listening to it (the trigger runtime, [triggers.md](../triggers/README.md)), the
    user as the subject. A game-object with another reticule-action (the thousands with 0, e34's five
    lock targets with 3, Hawk's bypass with 6: #145) is none of this: no prompt.
  * A `router-trigger` lists the panels and consoles among its objects (h_0b6b92ba) and passes
    their signal on (h_e91cf6a8) to the gate's `anim-trigger`s. Those animate the gate object
    (their own h_0b6b92ba) with one clip per leaf on signal 14, once only (h_f136d22d 1, where the
    proximity doors' signal 50 / 42 entries say 2147483647), and send on to the gate's sound
    trigger. So a used gate opens and stays open. On sdm_e34: panels h_14643319 / h_1d63038f ->
    router h_1c61561d -> anim-triggers h_f993d7ef / h_e8ad85ec -> gate h_14690535. The
    tutorial's panels go through their sound-triggers instead: the panel's terminal_03 plays
    (1.04 s), and its end (FINISHED) opens the gate's two leaves (2.0 s at time-scale 2.5) and
    plays the gate's grind (#223, take11). On the campaign's e34 (#262): consoles h_09da26aa
    (-34.5, 33.6, 28.2) and h_00dd163c (-37.0, 33.7, 60.3) -> router h_14eeda94 (h_1bbd1197 for the
    other console) -> the anim-trigger h_e5f08fe4, which plays one leaf's clip at once, the timer
    h_e4c2a242 (5 s), whose pulse has anim-trigger h_e446c3cf play the other's, the gate's sound
    metal_door_5 twice (h_f396d45f at the use, h_fa91e4c9 at the pulse), the four door points'
    waypoint triggers, and a Bratton line (sound-trigger h_f10a4b7e, dxud_bratton_us00565, 1.73 s,
    then h_ea031ac4's us00566) on the world's dialog queue: the beeps the take hears 0.4 s after
    the press. Either console opens the gate.
  * Using one (play_usables.rs, #262; todo/262-consoles takes 01, 03, 04, 05 for the consoles;
    take11 of the tutorial and todo/gate button.mp4 for the panels): the HUD's use tracker, the
    one that finds a lying gun ([pickups.md](../pickups/weapon-pickups.md), "Weapon pickups"; FUN_000bf1e0 and
    FUN_000bf2f0). An object whose reticule-action is 1 is the target when the character is on its
    front side (its model's hardpoints h_ee389721 and h_10f81d34 give it a facing, the way from the
    button to the use point; e34's console faces its model's -x, a panel +z), its use point
    (h_ee389721, 0.5 m in front of the button) is at most 1.7 m above or below the character's
    position and 2.5 m away across (3.6 m for the target it already has), the way to its button is
    within 60 degrees of the aim (the dot with the aim's heading at least 0.5), and the line from
    the use point to the character is clear (FUN_000de330, the object itself in its skip list;
    `Usable::sees`); the one most in front wins, and a lying gun more in front takes the frame.
    The use point is 1.03 m in front of e34's console h_09da26aa's origin, so the prompt shows from
    about 3 m off, as take01 has it (3.2 m from the origin at 1.70 s, none at 3.9 m). The consoles
    take X held 0.3 s (the player controller's hold, which every target in the code has: a 0.2 s
    tap does nothing, take03), once a press: the use fires at the hold's end with no animation
    (0x1581f0), and the prompt goes with it (take04: 0.43 s after the press), the chirps starting
    0.4 s after it. A panel keeps the 0.5 s it had before (a guess; take11's sounds 0.8 s after the
    press, which isn't explained: hints.md). While a skill says the character isn't free to act (the
    Wrath of Garner on, Flint's targeting: `skills::lets_act`, #208) no hold arms: the demo's
    simplification; the game defers or cancels the completed use (the use update FUN_00120290 asks
    FUN_00117210 at 0x1207b9 once the hold is done; shot S1 of #200, not filmed). E is the demo's X. While it's usable, "Hold [X] to
    activate %s." shows under the health bar: the game's use prompt, "Hold " + the X button's glyph
    + "to activate %s." with the object's display name, a glyph line drawn as the hint lines are,
    at the first one's place ([hints.md](../hints.md), "The use prompt"; take11 matches it to a unit,
    take04's console to the line's place and words). The blue target ring (the four-notch reticle
    texture h_1d2a68d0, tinted) sits on the button.
  * Used once: a console, and the tutorial's and sdm_e34's panels, stop being targets because the
    level's property trigger (h_14cccfaa, [triggers.md](../triggers/README.md)) hears their USE_OBJECT and
    sets their reticule-action to 0 (0x20b730): the prompt never comes back and a second hold does
    nothing (take05: three holds). A router that hears both of e34's consoles (h_1bbd1197), as one
    that hears both of the tutorial's panels, feeds each one's property trigger, so using either
    makes both dead (take11: the other panel offers no prompt after the first was used); e16's
    h_00dd163c and h_1bd44786 take the panels h_066a5235 and h_0f6d62a3 with them. Two start dead
    (a property trigger on FIRST_UPDATE: e16's console h_133a9b4c and m08_a's panel h_00c61205), and
    e16's panels h_14643319 and h_1d63038f and its "panel" h_f5bee5a1 go dead together when either
    panel is used (their router turns the USE_OBJECT into a TIMER_PULSE, out-signal 8, for the three
    property triggers). The consoles with no property trigger on their use stay usable, as the code
    leaves them (FUN_00158140 counts nothing): m14's seven that have none (m14_d's h_1b2a7b46 has one),
    m09's eight (theirs fire on DESTROYED), e16's "panel"s h_fcb9d537, h_e7b0848d and h_0ad321b8,
    m08_a's h_172b1fec and m08_b's h_026bb91e; and e34's campaign panels (which the demo still makes
    one use, as before: used, or the doors their signal reaches unchanged already opening, and no
    prompt; the demo's choice). A second use sends USE_OBJECT again; every entry behind it in the
    data that matters is once (the score, anim and script triggers), so m14's gate plays once
    (m14_a's prompt comes back when its gate's script, which holds the player, has ended: 7.3 s in
    the demo). m14's take wasn't obtained (todo/262-consoles notes: m14_a's console stood on a damaging
    floor, no prompt showed in several aimed views and a 1.5 s hold did nothing, so it may not be live
    until the chapter's script state): its prompt, "Hold [X] to activate Shrike console.", is the
    data's name, not seen. The runs that did use them are in [triggers.md](../triggers/verified.md), "Verified"
    (#262).
  * The button glows green: material h_f87dca06 is shader type h_1df292a1 (851 materials), a
    lit shader whose glow texture h_e01baa40 is tinted by its constant of the same name
    (0.276 0.914 0.220 here) and added as light.
  * `BF_DOOR_LOG=1` lists the doors and the panels and consoles (each one's button, use point and
    facing, and what it opens unchanged; a door on transform joints or hinges with how many leaves
    are on each), and logs each door the triggers open or close and its leaves each second: how
    far each sliding leaf has slid, or, for a door on transform joints or hinges, how far each leaf
    is from its closed place (m) and how far it has turned (degrees); and each use target, use and
    property trigger's change of an object's reticule-action. `BF_TEST_USE=<s>[,<hold>][;<s>[,<hold>]...]`
    holds use from each time for its hold in seconds, a second without one (`BF_TEST_USE="3,0.2;4.5,1.2"`:
    take03's tap and hold). `BF_TEST_DOOR_SHUT=<h_xxxxxxxx>[,...]` (#346) holds the doors of those names (as
    `BF_DOOR_LOG` prints them) shut whatever their triggers say: a door on a squad member's route
    that doesn't open, for the walk's door step ([nav.md](../nav/door-step-and-corner-hold.md), "The door step").
