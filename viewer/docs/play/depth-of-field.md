# The screen's depth of field (#38)

The blur when the player is hurt, and the blur in the scope: one pass in the original, the
camera block's depth of field (`FUN_000702a0`, 0x7173c-0x7188d) and the renderer's stage 8
(`FUN_000e7d40`), whose composite `FUN_000e5730` draws shader **0x1e** in mode 2.

## What the game does

**Scoped and alive** (cam+0x4c bit 3):

- S (+0x117) = 1, mode 2 (0x40de14), bias (+0x120) = 0;
- the focus (+0x118, a depth byte) is the aim ray's hit distance (+0x1f8, clamped to near..far,
  eased at 1000 m/s);
- the depth byte is `(1 - n/d) * f/(f - n) * 255`.

**Low health** (hit points under half the max, not scoped):

- S → 0.8 (0x40de18), mode 2;
- the focus is the character's own depth;
- bias = `hp / 0.5 - 1`.

**A cutscene or script state**: mode 3, a different soft-contrast effect (shader 0x11), not this.

**The composite** downsamples the picture to half and quarter size (2x2 box steps), then draws
one full-screen quad, SRC_ALPHA over the frame, with the depth buffer bound as a texture and the
constants c0 = S, c1 = the focus, c2 = `(bias + 1) / 2`. Shader 0x1e:

```
rgb   = 1/4 sharp + 1/2 half + 1/4 quarter
alpha = clamp(64 * (2S - 1) * |z - focus| + bias, 0, 1)
```

At S 1 the alpha reaches 1 about a centimetre past the focus, so the effect reads as **the focus
distance sharp, everything else blurred**. In the scope that focus is what the crosshair is on;
at low health it is the player, who stays sharp while the world either side of him blurs. The
slope at S 0.8 is 38.4 a metre, so the low-health side is nearly as abrupt: it is the *health
threshold* that turns it on, not a gradual ramp of blur.

## What the demo does

`play_dof.rs` puts bevy's own depth-of-field pass (`DepthOfField`, Gaussian mode) on the main
camera:

- **on** in the scope (`player.scope_level > 0`, alive), focused on `reticle::Reticle::depth` —
  the same centre ray the scope view sizes its frame by — clamped to the map's follow-camera clip
  planes and eased at 1000 m/s, taken at once on the first scoped frame;
- **on** under half health when not scoped, focused at the character's own depth;
- **off** otherwise (the component is removed).

The aperture is small so the circle of confusion saturates just past the focus, as 0x1e's slope
does; the cap (16 px) is what sets the strength, since a smaller aperture saturates the circle of
confusion at it.

**Approximation, not the formula.** Bevy's lens model is a smooth fall-off round a focal plane:
the fixed quarter/half mix, the depth byte's 8-bit quantisation and the bias's few-centimetre
shift are not reproduced. The trigger, the focus distances and the sharp-focus/blurred-rest
shape are the game's. Drawing 0x1e itself (the downsample chain, the depth read and the alpha
formula) is the remaining step.

The low-health focus is the campaign camera block's boom, 3.2 m (the classes' `camera` blocks),
a stand-in for the character's depth in the view; a run with a camera much closer or further
would put the focus a little off.

## Checking it

- `BF_DOF_LOG=1` prints each change: when the effect comes on and off, its kind, the focus and
  the health. `BF_DOF_APERTURE` and `BF_DOF_COC` override the two lens figures for tuning.
- Low health: `BF_TEST_HURT=<character>,<amount>` takes the hit points down (play_chatter.rs);
  a 60-point hit on Tex puts him at 40 of 100, under the threshold.
- The scope: the scope's own hooks (`play/scope.md`: `BF_TEST_SCOPE`, `BF_TEST_SCOPE_PRESS`,
  `BF_SCOPE_LOG`), aimed at something near against a far wall to see both sides of the focus.

**Checked in a capture** (1280 x 720, the test map): Tex hurt for 60 by `BF_TEST_HURT=tex,60,1.0`
(48 of 100 hit points), frames at 3 a capture. The effect comes on at the threshold and goes off
above it, as the log shows (`t=1.00 depth of field: low health 3.2 m, health 48%, 1 camera`, and
`t=0.00 depth of field off (health 100%)`). The same frame with and without it: the background
region's edge energy falls from about 1160 to about 70, the character in the foreground stays
sharp and the HUD with him. **Not checked**: the scope's side (its focus is the same reticle
depth the scope view sizes by, and the two share the code path), and the look against the
original's footage — the demo's strength is tuned by eye, not measured from a capture.

Still not the formula: drawing shader 0x1e itself (the half and quarter targets, the depth
read and the alpha) is the remaining step for an exact match.
