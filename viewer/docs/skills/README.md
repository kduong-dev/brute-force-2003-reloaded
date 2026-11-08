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
`src/bin/play_air_strike.rs`: [below](air-strike.md)), and the Feral Colonist's Spirit of Vengar and
the Feral Outcast's Spirit of Garner, Brutus's skill over other blocks of values
([below](ferals-spirits.md)).

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
* [Other characters' skills: the effect skills and the Orbital Beam (#200)](other-characters-skills.md)
* [The Air Strike: the Militia's special skill (#203)](air-strike.md)
* [The ferals' Spirit of Vengar (#202) and Spirit of Garner (#209)](ferals-spirits.md)
* [How it was checked (and, per skill, what each take showed)](how-it-was-checked.md)
