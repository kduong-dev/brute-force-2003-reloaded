//! The HUD fading out with the letterbox's bars (#380; found in #222 and #282). Written from the
//! code (the addresses are in viewer/docs/code-map/cutscenes.md, "Cutscenes"): the HUD's update
//! FUN_000b8e30 sets the HUD's alpha, every frame, to clamp(1 - 2 x the bars' amount, 0, 1) (the
//! view's +0x78, `script::Cutscenes::bars`), and every top-level widget's draw takes it
//! (FUN_000bb170), which hands it to the widget's own draws as a multiplier of their alpha. So the
//! HUD is whole with the bars out, fades as they come in, is gone by half bars and comes back as
//! they go. The takes agree (viewer/docs/cutscenes.md, "The letterbox"): take03 has no HUD at 26.3
//! s with the bars at 0.37 of full, a faint one at 26.4 s (0.17) and all of it at 26.6 s; take09 at
//! 39.95 s has it half there at 0.14. Before #380 the demo cut it at 0.25 of full, a fit.
//!
//! The mission title card (play_title.rs) is a widget FUN_000bb170 doesn't give the alpha
//! ([hud+0x135c], the credits' parent), so it isn't touched: it is not a `HudRoot`.
//!
//! **How it is drawn.** Bevy's UI has no group opacity, and the HUD's parts set their own colours
//! and alphas each frame (the pings' fades, the portraits' flashes, the messages), so the alpha is
//! not written into them. It is applied to what the UI hands the renderer: after Bevy's extract
//! systems have made each node's quad or glyph run (`ExtractedUiNodes`), the ones that belong to a
//! `HudRoot` (the HUD, the item box, the Squad Command menu and the hint lines: the four roots the
//! cutscene's bars used to hide) have their alpha multiplied by the share the HUD's alpha comes to
//! on the demo's blend (`HudFade::drawn`). That is the game's own multiply too: each of a widget's
//! draws takes the alpha, layer by layer, not the widget as a group.
//!
//! **The blend.** The console blends the stored sRGB values and Bevy blends in linear light, so
//! the same alpha doesn't give the same picture, and a fade is drawn with the alpha raised to a
//! power (`BLEND_POWER`, 1.4). It is not the 2.2 of play_hud.rs's and play_title.rs's fades:
//! 2.2 is exact for a layer over black (those fades are over the title's black bars or a dark
//! ground), but over a lit picture the power that gives the console's result is lower and
//! depends on the backdrop. A worked example: a layer of stored value s 0.9 over a picture of
//! d 0.45 at the game's alpha 0.33 comes out at an effective stored alpha of 0.13 with 2.2,
//! 0.29 with 1.4 and 0.43 with 1.0, where the console's is 0.33. So no single power is right for
//! every part (the radar's dark glass, the ammo panel's bright digits), and the takes' per-frame
//! strengths settled 1.4 as the best compromise (cutscenes.md, "The letterbox"). The real fix,
//! blending the UI in the console's stored values, is a follow-up (the same issue as the lamps'
//! glow, #379).
//!
//! The scope's frame and Flint's overlay (play_scope_view.rs) are drawn in their own pass in the
//! console's blend, so that pass takes the HUD's alpha itself (`HudFade::alpha`), not `drawn`.
//!
//! Test hook: BF_HUD_FADE_LOG=1 prints the HUD's alpha at each frame it changes (with the bars'
//! amount), to compare with the takes' brightness.

use std::collections::HashSet;

use bevy::render::{Extract, ExtractSchedule, RenderApp};
use bevy::ui::{ExtractedUiNodes, RenderUiSystem};

use super::*;

/// The alpha the bars leave the HUD, clamp(1 - `FADE_PER_BAR` x the bars' amount, 0, 1): the HUD
/// update's FUN_000b8e30 doubles the amount (so the HUD is gone at half bars).
const FADE_PER_BAR: f32 = 2.0;
/// The power the alpha is raised to for the demo's blend, which is linear light where the console
/// blends the stored sRGB values: a FIT, 1.4, not the 2.2 of play_hud.rs's fades and play_title.rs
/// `SRGB_POWER`. The takes' HUD is the game's alpha itself in stored values (take03's bars going
/// out at video 28.7-29.3 s and take09's coming in at 39.7-40.3 s: the health bar's strength is
/// 0.87 / 0.75 / 0.60 / 0.45 / 0.34 / 0.21 at bars 0.05 / 0.13 / 0.19 / 0.25 / 0.33 / 0.39, the
/// game's 0.90 / 0.75 / 0.61 / 0.50 / 0.34 / 0.22). The demo's frames through the same
/// transition (the stored-value alpha per region, cutscenes.md) read 0.14-0.22 at the game's 0.33
/// with 2.2, 0.37-0.44 with 1.0, and 0.32 (health bar) to 0.42 (radar, item box) with 1.4: the
/// best single power over the HUD's parts. It is below 2.2 because 2.2 is exact only for a layer
/// over black; over a lit picture the power the console's stored-value blend needs is lower and
/// depends on the backdrop (a stored 0.9 over 0.45 at the game's alpha 0.33: effective 0.13 with
/// 2.2, 0.29 with 1.4, 0.43 with 1.0), so the parts differ (the module notes).
const BLEND_POWER: f32 = 1.4;

/// The HUD's alpha under the letterbox, and the nodes it applies to.
#[derive(Resource)]
pub(crate) struct HudFade {
    /// the game's alpha: clamp(1 - 2 x the bars' amount, 0, 1) (FUN_000b8e30); 1 in the map
    /// editor's edit mode, where the bars aren't drawn and the HUD is the editor's
    pub alpha: f32,
    /// every node under a `HudRoot`, collected only while the alpha is under 1 (empty otherwise)
    nodes: HashSet<Entity>,
    /// the alpha last logged (BF_HUD_FADE_LOG)
    logged: f32,
}

impl Default for HudFade {
    fn default() -> Self {
        Self { alpha: 1.0, nodes: HashSet::new(), logged: 1.0 }
    }
}

impl HudFade {
    /// The multiplier the UI's nodes take on the demo's blend: the alpha to `BLEND_POWER` (1.4, a
    /// fit, not 2.2: see the module notes).
    pub fn drawn(&self) -> f32 {
        self.alpha.powf(BLEND_POWER)
    }
}

/// The alpha the HUD's update gives for bars `bars` of full (FUN_000b8e30): clamp(1 - 2 bars, 0, 1).
pub(crate) fn alpha_for(bars: f32) -> f32 {
    (1.0 - FADE_PER_BAR * bars).clamp(0.0, 1.0)
}

pub fn plugin(app: &mut App) {
    app.init_resource::<HudFade>()
        // (after the UI's layout: the HUD's record-text glyphs (play_fonts.rs `draw`, before
        // UiSystem::Prepare) and game texts (play_text.rs `draw`, before UiSystem::Layout) are spawned
        // by PostUpdate systems that run before it, and a glyph node made this frame and not yet in
        // the set would draw at full alpha for a frame)
        .add_systems(PostUpdate, collect.after(bevy::ui::UiSystem::Layout).run_if(in_state(AppState::Playing)))
        .add_systems(OnExit(AppState::Playing), |mut fade: ResMut<HudFade>| *fade = HudFade::default());
    if let Some(render) = app.get_sub_app_mut(RenderApp) {
        // (after every extract that makes a node of the UI: the quads, images, borders, text)
        render.add_systems(
            ExtractSchedule,
            apply.after(RenderUiSystem::ExtractBackgrounds)
                .after(RenderUiSystem::ExtractImages)
                .after(RenderUiSystem::ExtractBorders)
                .after(RenderUiSystem::ExtractTextShadows)
                .after(RenderUiSystem::ExtractText),
        );
    }
}

/// The HUD's alpha for this frame (`view` in play_script.rs has stepped the bars), and while it is
/// under 1 the nodes under the HUD's roots that it applies to.
fn collect(cutscenes: Res<script::Cutscenes>, mode: Option<Res<State<super::editor::EditorMode>>>, mut fade: ResMut<HudFade>,
           roots: Query<Entity, With<hud::HudRoot>>, children: Query<&Children>) {
    // (in the map editor's edit mode the bars aren't drawn and the HUD is the editor's)
    let editing = mode.is_some_and(|m| *m.get() == super::editor::EditorMode::Edit);
    let alpha = if editing { 1.0 } else { alpha_for(cutscenes.bars()) };
    if fade.alpha != alpha {
        fade.alpha = alpha;
    }
    if alpha >= 1.0 {
        if !fade.nodes.is_empty() {
            fade.nodes.clear();
        }
    } else {
        let mut nodes = HashSet::new();
        for root in &roots {
            nodes.insert(root);
            nodes.extend(children.iter_descendants(root));
        }
        fade.nodes = nodes;
    }
    if std::env::var("BF_HUD_FADE_LOG").is_ok() && fade.logged != alpha {
        println!("hud fade: bars {:.3}, alpha {alpha:.3} (drawn {:.3}), {} nodes", cutscenes.bars(), fade.drawn(), fade.nodes.len());
        fade.logged = alpha;
    }
}

/// The render world's half (ExtractSchedule): the extracted quads and glyph runs of the HUD's
/// nodes take the alpha, as FUN_000bb170's multiply gives each draw of a widget.
fn apply(fade: Extract<Res<HudFade>>, mut extracted: ResMut<ExtractedUiNodes>) {
    if fade.nodes.is_empty() {
        return;
    }
    let k = fade.drawn();
    for node in &mut extracted.uinodes {
        if fade.nodes.contains(&node.main_entity.id()) {
            node.color.alpha *= k;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The alpha is 1 with the bars out, falls twice as fast as they come in, and is 0 from half.
    #[test]
    fn alpha_is_one_minus_twice_the_bars() {
        assert_eq!(alpha_for(0.0), 1.0);
        assert!((alpha_for(0.17) - 0.66).abs() < 1e-5);
        assert!((alpha_for(0.25) - 0.5).abs() < 1e-5);
        assert_eq!(alpha_for(0.5), 0.0);
        assert!((alpha_for(0.37) - 0.26).abs() < 1e-5);
        assert_eq!(alpha_for(1.0), 0.0);
    }

    /// The blend's power leaves a whole HUD whole and a gone one gone.
    #[test]
    fn drawn_share() {
        let mut f = HudFade::default();
        assert_eq!(f.drawn(), 1.0);
        f.alpha = 0.0;
        assert_eq!(f.drawn(), 0.0);
        f.alpha = 0.5;
        assert!((f.drawn() - 0.5_f32.powf(BLEND_POWER)).abs() < 1e-6);
    }
}
