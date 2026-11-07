*Part of [nav/](README.md), the former page `nav.md`.*

## Lava

Nothing in the route code sees lava: the corridor ray passes through liquids, and no world point
lies over a harmful one. The graph's layout keeps routes off it. The tutorial's lava yard: its 42
world points run along the walkways at y 15.8-16 (0.5 m over the floor), the chain between the
two pits h_f5015ea8 (38.1, -29.1) -> h_1862fb9d -> h_0a6c9ab1 -> h_19d4b7b6 (46.4, -20.0) with
links of 3.6-9 m. The walkway's points from h_19d4b7b6 round to h_fe0e9b66 all have h_ed6a7490
(so the walk never skips one) and, on to h_1a6a0ec5, their own 1 m radius; the walkway's edges
are lined with hinters (blockers only the AI meets), which the corridor rays meet when a
character stands off the line of the walk. So a route there goes point by point along the
walkway; the straight line between its ends crosses a pit.
