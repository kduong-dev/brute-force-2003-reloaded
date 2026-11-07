*Part of [grenades/](README.md), the former page `grenades.md`.*

What each type does so far, and doesn't:

| Type | Does | Not yet |
|---|---|---|
| Frag (#75) | everything above | the "Tech Upgrade Frag Grenade!" upgrade |
| Energy (#74) | timer 1.75, stun_grenade_master + stun_hit_s, f875b6c6, decal h_f4d65518; its bolts crawl out and carry the damage (39-97.5, 9 m) to every body in reach, throwing down the living and throwing corpses (see **Energy grenade**) | the stunned_fx 30 s arc; the per-liquid effects h_0cbcb8f2; the "Tech Upgrade" |
| Gas (#76) | timer 1, gas-grenade's olive cloud (with its air fields), 1b35643e, the poison: 17.7 HP/s within 3 m for 5.5 s, Flint x0.05 | the "Tech Upgrade" (the ticket's); the cloud's fade (lingers ~1-2 s, see above); the flash's ~1 s flicker (smoke-grenade-flsh's keyed emit count: one burst here); the shared HUD's damage feedback (white bar with a dark-red trailing segment, the direction chevron) |
| Light (#77) | timer 1.5; phosphor_grenade (a tall thin blue beam of upright streaks, the glow and sparks at its base) + light_phosphor (a blue-white point light, reach 18.8-24.3 m) burning 30 s as the data has them (the spike's rate drops to 0 at 29.45 s); ignition h_1080aaf1 and f5260e35 after 0.2 s; no damage, no decal, no tint, pickups left alone; the canister stays where it lies, its trail stopped, until the effects have run (31.1 s) | the end of the burn isn't recorded (the recording shows >= 15.1 s); the beam is paler and less solid than recorded and the base's white core smaller (see **Light grenade**); the sparks fly too far (no gravity field, #101); the light's strength not measured against the recording |
| Sonic (#81) | goes off on first contact (timer 0), grenade_sonic (dome, ring, godrays) + light_sonic_grenade, f36fb063, decal h_fb24bcb7; the damage comes with the ring, the thrower's falls off by 5 m, nobody is knocked down (see **Sonic grenade**) | the "Tech Upgrade"; the godrays (broad columns, not two narrow shafts), the ring (a wall, not a flat band), the decal's look |
| Roller (#79) | set down (place_hi); seeks the nearest hostile in sight within 50 m every 0.5 s (from 1 s), hops after its point (a fit to the takes: 3.1, 5.6, 7.2, 8.5 m/s at its first hops, a ceiling of 8.5), stops dead at walls, goes off within 2.5 m of its point or at its 25 s fuse; h_013b3ca8 (the Frag's six effects + light_explosion) + h_065168c9; its rolling sound (see **Roller**) | the per-material ground puff h_0cbcb8f2 (no grenade has one yet); a shot or a blast taking its 1 hp; the set-down hand (above) |
| Thermo (#82) | set down (place_hi), 15 s fuse from the landing; m09_b's own charge set down armed at the load, its fuse held by the startup cinematic (24 s after the level's start); the Roller's effect set at 3 m + the impact sound h_16611581 (see **Thermo**) | whether a shot sets it off; enemy damage by distance; its look and sounds (not filmed) |
| Sentry (#80) | set down (place_hi), lies there with its red tips lit (#300); goes off for a hostile within 3 m with no friend within 3 m, or when shot (see **Sentry**), or by a blast that takes its 1 hp (see **Interactive scenery**); exp-mine + light_explosion, h_145f09e5 | disarming an enemy's mine; the AI keeping clear of it; the tips' glare; the thrower's damage beyond the radius (below) |

Not done for any: ALE fields other than the Gas cloud's and the trail's rise (exp-lrg-air,
the gravity and turbulence fields: #101); decals don't
follow uneven ground (a plane along the slope under the middle: a big scorch on a bumpy
hillside is partly buried); the sounds' play-length; distance falloff for blast sounds beyond
a volume.
