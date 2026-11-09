# Special skills: stamina, the skill toggle (#120), Tex's Berserker (#121), Brutus's Spirit of Vengar (#122), Hawk's Stealth (#123) and Flint's Advanced Targeting (#124)

*This folder was the page `skills.md`, split into one file per section (#393): the sections are listed under **Sections** at the end of this page, and `../skills.md` is now a pointer to them.*

Each squad member has a special skill that stamina (the blue bar) pays for: Tex's Berserker,
Brutus's Spirit of Vengar, Hawk's Stealth, Flint's Advanced Targeting. This page is the part the
four share (`src/bin/play_skills.rs`, the HUD in `play_hud.rs`): stamina, the toggle, its sounds,
chatter and HUD, and the interface each skill plugs into; and the four skills, each its own
ticket: Tex's Berserker ([below](texs-berserker.md)), Brutus's Spirit of Vengar
([below](brutuss-spirit-of-vengar.md)), Hawk's Stealth ([below](hawks-stealth.md)) and
Flint's Advanced Targeting ([below](flints-advanced-targeting.md)). Other characters' skills
start with the effect skills' shared part and the Confed Marine's Orbital Beam (#200,
`src/bin/play_effect_skills.rs`: [below](other-characters-skills.md)), the Militia's Air Strike (#203,
`src/bin/play_air_strike.rs`: [below](air-strike.md)), the Feral Colonist's Spirit of Vengar and
the Feral Outcast's Spirit of Garner, Brutus's skill over other blocks of values
([below](ferals-spirits.md)), and the Outcast
Shaman's Wrath of Garner (#208, `src/bin/play_wrath.rs`: [below](wrath-of-garner.md)), a skill
that stays on for 3 s and returns a whole-body motion, the Supermutant's ground pound (#212,
`src/bin/play_ground_pound.rs`: [below](ground-pound.md)), the boss's 1.97 s MT 45 clip whose burst
at 0.97 s pounds the ground within 7.5 m, and Gunthar Ghent's Cloak (#205,
`src/bin/play_cloak.rs`: [below](ghents-cloak.md)), Hawk's invisibility at the press with no clip,
cost or hold fire (not filmed: the code's numbers, checked in the demo's own logs and frames), and
McTavish's Tracking (#207, [below](mctavishs-tracking.md)), Flint's Advanced Targeting over another block
of values and under another name (not filmed either), and the Molotov Mutant's Self-Destruct (#211,
`src/bin/play_molotov_mutant.rs`: [below](molotov-mutant.md)), the game's AI-only suicide attack in the product owner's
design for a player: a lit bottle held in the left hand, put away by a press or lost at 0 stamina, a lunge, 500
FLAME to himself and a cloud when a hostile comes within 5 m, the same cloud if he's killed holding it, and the
Filthy and Ugly Mutants' Corpse Eater (#210, `src/bin/play_corpse_eater.rs`: [below](corpse-eater.md)), the m03 mutants'
eating in the product owner's design for a player: V next to a body kneels him over it through the `eat_start` /
`eat_idle` / `eat_stop` chain, the gore effect at his left hand at the loop's three events, 6 HP a second (the
demo's value: the class has no heal and the data no rate), exposed until V again or he moves off the body.

Everything here comes from default.xbe (the functions are in [code-map.md](../code-map/skills-and-stamina.md), "Skills
and stamina", "HUD", "AI" and, for Brutus's view, "Effects / ALE") and the data, checked against the 32 takes in
`todo/35-special-skills/` (the spec is `spec.md` there, Parts 1 to 5).

**Sections**

* [Keys and test hooks](keys-and-test-hooks.md)
* [Stamina](stamina.md)
* [The toggle](the-toggle.md)
* [Sounds and chatter](sounds-and-chatter.md)
* [HUD](hud.md)
* [The skill interface (#121-#124)](skill-interface.md)
* [Tex's Berserker (#121)](texs-berserker.md)
* [Brutus's Spirit of Vengar (#122)](brutuss-spirit-of-vengar.md)
* [Hawk's Stealth (#123)](hawks-stealth.md)
* [Flint's Advanced Targeting (#124)](flints-advanced-targeting.md)
* [Other characters' skills: the effect skills and the Orbital Beam (#200), and the demo-own skills of the test map's classes: Kingman's Berserker, his mirrored left arm, his left-hip holster and his swap (#407, #409)](other-characters-skills.md)
* [The Air Strike: the Militia's special skill (#203)](air-strike.md)
* [The ferals' Spirit of Vengar (#202) and Spirit of Garner (#209)](ferals-spirits.md)
* [The Outcast Shaman's Wrath of Garner (#208)](wrath-of-garner.md)
* [The Supermutant's ground pound (#212)](ground-pound.md)
* [Gunthar Ghent's Cloak (#205)](ghents-cloak.md)
* [McTavish's Tracking (#207)](mctavishs-tracking.md)
* [The Molotov Mutant's Self-Destruct (#211)](molotov-mutant.md)
* [The Mutants' Corpse Eater (#210)](corpse-eater.md)
* [How it was checked (and, per skill, what each take showed)](how-it-was-checked.md)
