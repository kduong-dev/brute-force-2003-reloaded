*Part of [play/](README.md), the former page `play.md`.*

How it works (`src/bf/locomotion.rs`): animation names are hashes, so clips are chosen from
their data: root-channel velocity (forward is -Z; sprints are fastest), clean looping, head
height (rejects crouched / kneeling variants) and arm span (rejects arms-out reference poses).
The squad's only sideways clips are dodge rolls, so aimed movement is legs-run plus an
upper-body twist spread over the spine and head (`aim_chain` / `twist`). Movement is root
motion: each frame the character moves by exactly what the playing clips' root channels move,
crossfaded over 0.2 s, so speeds and footfalls match the original animations.
Standing with the stick at rest, the controlled character keeps its heading (#294): only the
stick's move and turn request and the aim turn it (FUN_001195c0 makes no request from a stick in
its dead zone). The AI's characters settle to a heading of their own (`face_yaw`: a squadmate
the leader's way, an enemy where its motion left it); a script's walk, a warp and a hit
reaction's end leave one on the controlled character too, and it used to turn back to it at
3.15 rad/s whenever it stood still (e34's opening walk; the tutorial's warps of Tex,
trigger_warp_tex_patrol and trigger_warp_tex_to_vsquad). `BF_HEADING_LOG=1` prints the
controlled character's heading every frame, what the step turned it, the aim's yaw, its place
and what may turn it standing (the stick, the aim, the gun raised, a slide, a throw), with a
`face_yaw` left on it. Standing still on slide-flagged ground built a slide up too, which
turned the character downhill (maps.md, "Sliding": `stands_still`). Checked (#294), each of
the four standing with no input, its heading unchanged to the fifth decimal:

* at sdm_e34's start and on its 36° face at (23, 10) (`BF_TEST_GOTO=23,10,23,10`), 95 s each
  in real time; before, Flint slid off the face at 41.6 s and Tex (another run) at 82.9 s,
  each turning about 180°;
* on that face at the capture's 15 fps (`BF_CAPTURE_FRAMES=900 BF_CAPTURE_EVERY=150`), 61.5 s:
  before, Flint slid at 1.3 s; the place moves 0.7-2.3 cm (the idle's root sway);
* after the tutorial's trigger_warp_tex_patrol (`BF_MAP=tutorial BF_TEST_GOTO=31,92,36,96
  BF_TEST_GO_AT=4 "BF_TRIGGER_FIRE=h_095ae7a3,11,2"`): Tex runs to the goal and keeps the
  heading he stops with (-1.99 rad, 5 s on), where before he turned back 140° to the warp's
  (+1.55) over 0.85 s;
* a run onto that face (#285's test, `BF_TEST_GOTO=30,10,-30,10`) still slides at 2.47 s.

Jumping uses the game's named clips (`Sc_w1_jump_crouch/launch/fall/land`, `Sc_4leg_w1_jump_*`;
names recovered from the motion scripts) with simple gravity; a jump pressed during a dodge is
buffered for 0.4 s.
