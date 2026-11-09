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

`dof.wgsl` draws that shader, in `play_dof.rs`'s own render node (`DofLabel`), between the
tonemapping and the Vengar view's pass — so before the scope view's pass and the HUD, as the
game's stage 8 runs before its HUD (the scope's frame and the bars stay sharp over the blur).

- **S, the focus, the bias** are the camera block's: in the scope S 1 and bias 0, with the focus
  on `reticle::Reticle::depth` (the centre ray the scope view sizes its frame by), clamped to the
  map's follow-camera clip planes and eased at 1000 m/s, taken at once on the first scoped frame;
  under half health S 0.8, bias `hp / 0.5 - 1`, and the focus at the character's own depth.
- **The half and quarter copies** are box samples of the picture: a tap where four pixels meet is
  their 2x2 average (the half-size copy), and four such taps a second apart the 4x4 box (the
  quarter-size copy) — the same two 2x2 downsample steps without separate targets.
- **The depth** is Bevy's own depth texture, whose reverse-Z value is `near / z`
  (`perspective_infinite_reverse_rh`), so `|z - focus|` is in metres as the game's is. With MSAA
  on the depth texture is multisampled, so the pass has two pipelines and the shader reads the
  pixel's sample 0 (`#ifdef MULTISAMPLED`); the camera's depth gains `TEXTURE_BINDING`.
- **Off** is S 0: the node draws nothing, so nothing is added to or removed from the camera.
- **The picture's values**: read and written as sRGB, the console's frame-buffer values, as the
  scope view's pass does.

**Not reproduced**: the game's 8-bit depth byte (its focus is quantised, this one isn't), and the
c2 packing of the bias (the demo passes the bias directly). The character's own depth at low
health is the campaign camera block's boom, 3.2 m — a stand-in for the view's depth of him
(demo-own, 2026-11-09); a camera much closer or further would put that focus a little off.

## Checking it

- `BF_DOF_LOG=1` prints each change: when the effect comes on and off, its kind, the focus, S, the
  bias and the health.
- `BF_DOF=<from>-<to>,...` holds the effect on over those seconds whatever the health or the
  scope (its focus the character's), so the on and off play in one run.
- Low health: `BF_TEST_HURT=<character>,<amount>` takes the hit points down (play_chatter.rs); a
  60-point hit on Tex puts him at 40 of 100, under the threshold. The scope: the scope's own
  hooks (`play/scope.md`: `BF_TEST_SCOPE`, `BF_TEST_SCOPE_PRESS`, `BF_SCOPE_LOG`).

**Checked in a capture** (1280 x 720, the test map): Tex hurt for 60 by `BF_TEST_HURT=tex,60,1.0`
(48 of 100), the same frames with the effect and without it. The background region's edge energy
falls from about 1160 to about 280, the character in the foreground and the HUD stay sharp, and
the picture keeps its detail (a quarter and half box mix, not a smear). The log shows
`t=1.00 depth of field: low health 3.20 m, S 0.80, bias -0.04, health 48%`.

**Not checked**: the scope's side filmed against the original (its focus shares the scope view's
code path but has no capture yet), and a comparison of the strength with the original's footage.
