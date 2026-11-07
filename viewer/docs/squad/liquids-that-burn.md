*Part of [squad/](README.md), the former page `squad.md`.*

* **Liquids that burn** (#1, play_liquids.rs `burning_ahead`, `steps_into_burning`). A wingman
  doesn't walk into lava, sludge or toxic waste (a liquid whose character rate h_e0489ea0 is
  above 0, the one that hurts: levels.md, "What a liquid does to a character"): with ground in
  one ahead of it (its feet within 0.15 m of the surface or under it: where its calves' parts
  would be in, `PART_REACH`) it doesn't set off toward its place, so it waits at the shore, and
  the movement undoes any AI-steered character's step onto such ground (a wingman's, an enemy's,
  a cutscene's walk), as the ledge probe undoes a step off a drop; one already in may still step
  up and out. Water stays walkable (its rate is 0).
  * Why: the game's AI has no liquid code (no AI routine reads the liquid state or the liquid
    objects, todo/146-liquids leads §4.2); what keeps its squad out is its routes over the
    level's world points, and no harmful liquid has one over its floor on the SDM maps (sdm_e13's
    lava 0 of the 536 over it, sdm_m03's sludge 0 of 141, sdm_m07's lava 0 of 284; at most a
    couple ankle-deep at a shore), where water has many (sdm_e10's pond 87-272). Since #335 the
    squad's follow and Move To walk those routes (above, [nav.md](../nav/squad-on-routes.md)), and this is their
    safety net, which none of #335's runs needed: a data-backed rule, not a traced one. The
    enemies' goals walk routes with the same net since #336 (play_goals.rs `go`); the scripts'
    walks (#337) still lean on it.
  * Not known: whether the game's squad follows its leader into lava when he walks in: its
    follow routes to his own point (FUN_00022130), which lies on no liquid, so as read it waits at
    the nearest point (#146's shot A4, not filmed). Round a liquid between a wingman and its place
    it goes by the route (#242 take01: the squad round e13's pit by the west shore; the demo's
    the same, nav.md).
  * The level's hinters (#292, maps.md "Blockers and hinters") are walls to the AI's characters
    and not to the player's: sdm_e13's band round its outer lava shore holds the wingmen there
    as the game's AI is held, while the player walks through it into the lava. Its inner lava has
    no band; there this shore stop is what keeps them out.
  * Checked: `BF_MAP=sdm_e13 BF_CHARACTER=0 BF_START=9 BF_TEST_GOTO=-9.9,-111.3,-21.44,-105
    BF_TEST_GO_AT=4 BF_TEST_HEALTH=5000 BF_LIQUID_LOG=1` (`BF_VIEW_YAW=3.14159` to look back):
    Brutus walks into the lava, Tex and Hawk crouch at the shore and Flint stands on the rock
    behind them, 11-16 m from their places, and none of them takes a hit; `BF_LIQUID_LOG` prints
    "waits at a liquid that burns" twice a second for each. Since #335 (`BF_ROUTE_LOG=1
    BF_ORDER_LOG=2` too): their routes end at the shore's points (h_1d0e6ca6, h_e5c04065), where
    they kneel 8-9 m from Brutus, and the guard never prints.
