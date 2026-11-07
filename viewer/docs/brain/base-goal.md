*Part of [brain/](README.md), the former page `brain.md`.*

## Its base goal

When nothing else is on (no contact, not going home), the brain does what its personality's
EVT_BASE (3) entry draws (FUN_00054c40, by weight):

| Personality | EVT_BASE | Who |
|---|---|---|
| h_e176c6ce | GOAL_PATROL 1, GOAL_IDLE_VARY 0 | the first patrol's TUTORIAL_terrorist_2 and _18 (template h_f2b759fb), the base's _6 (h_ffd92afd) |
| h_f0330ad7, h_eed3a76a, h_0bb586cf | GOAL_IDLE_VARY | the other Red Hand: they stand where they are (its pauses, cautious looks, crouches and walks home and about are #26's) |
| the placement's wingman one, h_ebe6044c (-> h_ea20a17d) | GOAL_WINGMAN_FOLLOW 1, GOAL_IDLE 0 | a squad-type character on the player's side: the V-squad's two Confed marines |

A goal whose start refuses (a patrol with no route) is struck out and the draw made again
(FUN_00054bc0); with every weight 0 the first goal is taken.
