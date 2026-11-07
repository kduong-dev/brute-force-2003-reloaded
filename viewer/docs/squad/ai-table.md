*Part of [squad/](README.md), the former page `squad.md`.*

* **The squad's AI table.** A squadmate's personality (`common/ai/personality.xmb`) is its
  command mode's (#318, **Squad commands** below): FOLLOW's `h_169b2725` by default, WINGMAN's
  `h_ea20a17d` (each character's own `personality-wingman` is empty and inherits it) after Fire
  At Will, GUARD's `h_155e52d6` after Stand Ground, MOVE_OUT's `h_0f69b8b2` after Move To. It
  maps events to goals, with priorities and weights. Its events and goals are numbers. The
  names come from default.xbe's name tables: events at 0x3bdc14 (`EVT_DEAD` = 10, … 96), goals
  at 0x3bce78 (`GOAL_DODGE` = 59, … 134). Without enemies, these are the wingman personality's
  entries, which the demo's dodges use in every mode (FOLLOW's own have `GOAL_DODGE` 1 :
  `GOAL_NOOP` 3 for EVT_SHOT_AT and always `GOAL_DODGE` for EVT_DAMAGED_BY_PC; GUARD's take no
  dodge: not done):

  | Event | Goals (weight) | Here |
  |---|---|---|
  | `EVT_GRENADE_NEAR` (pri 28) | `GOAL_DIVE` | dive away from a grenade that lands near |
  | `EVT_SHOT_AT` (20.15) | `GOAL_DODGE` 1, `GOAL_NOOP` 7 | a shot of yours passing within 1.5 m (the distance is a guess): sidestep one time in eight |
  | `EVT_DAMAGED_BY_PC` (15) | `GOAL_DODGE` 1, `GOAL_NOOP` 2 | shot by you: sidestep one time in three |
  | `EVT_BASE` (1) | `GOAL_WINGMAN_FOLLOW` | the follow (#331) |
  | `EVT_IDLE_VARY_DATA` | `IDLE_VARY_DATA_PAUSE_CROUCH` 15, `…_PAUSE_CAUTIOUS` 12 (equal weights) | holding position, the wingman's (Fire At Will) draw: the first pause kneels (as in the captures), then kneeling and cautious pauses (standing, weapon ready, looking left and right) are drawn, lasting 15 and 12 s |

  The idle draw is the mode's personality's EVT_IDLE_VARY_DATA entry (#318): the table's row is
  the wingman's, not Stand Ground's. FOLLOW's (the default) adds `IDLE_VARY_DATA_PAUSE` (97, a
  plain standing pause) for 6 s, the three equally likely after the first crouch; GUARD's
  (Stand Ground) has `PAUSE_CROUCH` alone: it kneels and stays down. (Before #318 the squad drew
  the wingman's table in every case.) `BF_AI_LOG=1` prints their idle choices ("crouch",
  "cautious", "pause") and dodges.
