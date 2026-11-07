//! The test tools' list of the ALE effects nothing in the game starts (#397, the ticket's optional
//! part): 139 of the 603 effects of common's effects-common.ale that the cut-content catalogue
//! (todo/cut-content/catalogue.md 3.2, `effects-unreferenced.txt`) found **unreferenced**: no
//! object type's alchemy entries or light effect, no trigger, world material, script or liquid
//! names them, and neither do the strings and constants of default.xbe (the sweep hashes every
//! name and 32-bit value of the data and the XBE). By the names they are cut weapons and ammo (the
//! MIRV and Thermalite explosions, the paralyser, the stun grenade), vehicle wrecks and
//! breakups, power-ups, Seer and Shrike effects, the weather and environment, a fourth blood set:
//! they may be parts of other effects (a `-master` that spawns its `-add`) or left over.
//!
//! The object menu lists them in a group of their own (play_testtools.rs `catalogue`,
//! `Kind::Ale`), and an entry plays the effect where the crosshair meets the world: an
//! `EffectSpawn` for an effect that isn't an effect type, which `start_effects` (play_effects.rs)
//! looks up as an ALE effect and plays once, with the rules the effects triggers make are played
//! with (`AleAssets::load_recorded_facing`). `BF_TEST_EFFECT=<s>,<name>,<x>,<z>` does the same
//! without the menu. Not checked against the game: nothing in it plays them, so how they were to
//! look is the demo's ALE runtime's reading, and some are fragments.

/// An unused ALE effect: its name in the effect library and the hash of it (`h(name)`, the key the
/// library and an effect type's alchemy entry use).
pub(super) type UnusedEffect = (&'static str, u32);

/// The 139 unreferenced ALE effects, by name (any case) as `effects-unreferenced.txt` lists them,
/// each with the hash it gives; `the_list_is_the_catalogues` checks each pair, the count and the
/// order.
pub(super) const UNUSED_ALE: [UnusedEffect; 139] = [
    ("bio_bullet", 0xE6848E9F),
    ("bio_bullet_shrap", 0xEAD51C9A),
    ("bio_bullet_spine", 0xE162214F),
    ("bio_bullet_trail", 0xFBC91093),
    ("blood_puff_f", 0x1B7B0ED2),
    ("blood_ragdoll", 0x037D6981),
    ("bloodsplat_f", 0xE58B17B2),
    ("boss_cyclone_start_child", 0x19DAC2FB),
    ("breakup-aagun-add", 0xE8438714),
    ("breakup-aagun-master", 0xED07F306),
    ("breakup-aagun-trail", 0x0B065016),
    ("breakup_aagun_end", 0xF3EC06C5),
    ("breakup_autocan", 0x012981A8),
    ("breakup_battery", 0xFDE2558F),
    ("breakup_ceilingdust", 0x1E0B7F9B),
    ("breakup_chamber_master", 0xF90A298C),
    ("breakup_comstat", 0xF9AE0850),
    ("breakup_console_master", 0x014C5E00),
    ("breakup_dwg_pylon", 0xEF86740E),
    ("breakup_pylon-trail", 0xEACE1C45),
    ("breakup_thatch", 0x0BF6785A),
    ("campfire_hit_sparks", 0xF2C6B7FF),
    ("duststorm", 0x16180487),
    ("duststorm_night", 0xF4357006),
    ("dwg_static", 0xF675BD33),
    ("dwg_static_master", 0x16B55490),
    ("energyball", 0x111F51E2),
    ("energyball-master", 0xF48E9B66),
    ("env_el_mdsplash", 0x1632E211),
    ("env_el_projectile", 0xEB783B75),
    ("env_el_smsplash", 0xFF72C5B4),
    ("env_el_wadering", 0xFD6A55B0),
    ("exp-air-shocksphere", 0x0C3FBF54),
    ("exp-air_b-trail", 0xE6ECF4E6),
    ("exp-chopper-trail", 0xF5E504F7),
    ("exp-forman-trail", 0xF816CD5C),
    ("exp-gas-shock", 0xE62B08A9),
    ("exp-gas-sml-shock", 0x0633C20F),
    ("exp-hummer-add", 0xEC174367),
    ("exp-hummer-master", 0x085C80AC),
    ("exp-hummer-trail", 0xEF3F78DE),
    ("exp-lrg-shocksphere", 0x1B92A423),
    ("exp-mirv-add", 0x06EDAA50),
    ("exp-mirv-dirt", 0x052DCCC6),
    ("exp-mirv-fire", 0xEF68EC43),
    ("exp-mirv-flash", 0xE301CE6B),
    ("exp-misrack-trail", 0x0BEC5B82),
    ("exp-tank-trail", 0xFE216B37),
    ("exp-thermolite-add", 0xFBB708A5),
    ("exp-thermolite-master", 0x07368BB3),
    ("exp-thermolite-trail", 0xED8142AA),
    ("exp-water-dirt", 0xFFEDAAA4),
    ("exp-water-fire#1", 0xE98A0FD1),
    ("exp-water-fire-add", 0xF6485A2F),
    ("exp-water-shrap", 0xECEEA4D7),
    ("exp_orbitalbeam_pre", 0xF67188F4),
    ("exp_shrike_tunnel_gasmain", 0x041A8058),
    ("ferg_duststorm_med_s", 0x18CC2D42),
    ("ferix_fire", 0xE20C9045),
    ("fire_short", 0x00DBBB7B),
    ("firehound_atk", 0xE7D6B1D7),
    ("firehound_atk_smoke", 0xECC61F1F),
    ("giblet_f", 0x11C6C58F),
    ("godray40_bounds", 0x1B221B7A),
    ("grenade_launch_trail", 0xFE2D8D11),
    ("grenade_launcher_smoke#1", 0xFAEEF7AA),
    ("hlord_lightning", 0xE8034BAC),
    ("hlord_sidebeam", 0xFBC3F5E7),
    ("hound_p", 0xEA11CCE0),
    ("lava-erupt#1", 0xF1A330A1),
    ("lava-fuzz", 0xF2D8F6C9),
    ("light_hit_laser_blue", 0x1B77564B),
    ("light_hound_gas", 0xFDCD00A3),
    ("light_powerup", 0x11A81304),
    ("light_powerup_garro", 0xF43451CA),
    ("light_powerup_med", 0x15D29E6D),
    ("light_shrike_strobe", 0xFEB6D155),
    ("light_teleport_activate", 0xEBAC6734),
    ("merc-bubbles", 0xEB33BC32),
    ("minigun_muzzle", 0xF1BCFB65),
    ("mudsplash", 0xEA6CDFF0),
    ("multi_health_squad_icon", 0x027A9B4B),
    ("multi_power_icon", 0x1CC3B6C6),
    ("multi_stamina_squad_icon", 0xE089DC52),
    ("paralyzer_bul", 0x006BAF8E),
    ("paralyzer_hit", 0xEE1BE353),
    ("powerup", 0xE0B90D93),
    ("powerup_med", 0xF4D4060A),
    ("rainsplash", 0xEC346535),
    ("rainsplash_master", 0xE3152A66),
    ("seer_alert", 0x1CF24F81),
    ("shadoon_warp_activate", 0x0F4B4EBC),
    ("shadoon_warp_purp", 0x0BE12757),
    ("shadoon_warp_sml", 0xEFDC23B8),
    ("shieldgen_15m", 0xE89BA11B),
    ("shrike_merc_vent", 0x0A7876ED),
    ("shrike_scan", 0x10A5490F),
    ("shrike_tunnelpipe_steam", 0xF7FD73DA),
    ("singeflame", 0xFA65DEE6),
    ("slava-erupt#2", 0x16E352D5),
    ("slava-erupt#4", 0xFB80F7E0),
    ("smeteor_beam", 0x19783C53),
    ("smeteor_rain", 0x19385842),
    ("smoke_light", 0x1833EC64),
    ("sonic_shotgun_rifle", 0xFA0BEA41),
    ("sonicbalistic", 0xFFE39265),
    ("sonicbalistic-master", 0x1CB67847),
    ("splash_acid_bullet", 0x0120405C),
    ("splash_acid_foot", 0xF0E8BFA4),
    ("splash_acid_med", 0x180D27DF),
    ("splash_acid_wade", 0xFB7FF3F3),
    ("splash_bullet", 0x15B59ED8),
    ("splash_fire_bullet", 0x19E2102F),
    ("splash_fire_foot", 0x082484FB),
    ("splash_fire_med", 0x0DA8F361),
    ("splash_fire_wade", 0x03B3C8AC),
    ("splash_foot", 0x17403076),
    ("splash_med", 0x0E3A1F85),
    ("splash_wade", 0x1CD77C21),
    ("stun_beam_s", 0x0E677FE3),
    ("stun_grenade", 0x00C8B901),
    ("stunned_fx", 0x0EB1DECA),
    ("surprise_lava", 0x16649B05),
    ("teleport_elect", 0xF6B4406F),
    ("tornado", 0x0878F4D7),
    ("tornado_master", 0xF7A4F80F),
    ("tornado_master_40", 0x115C724A),
    ("toxic-bubbles", 0xF15CF769),
    ("toxic_waste_fog_5", 0xE7347CAE),
    ("weapon_fade", 0x10E174A7),
    ("wmat_firepuff", 0xEBA51EA0),
    ("wmat_hitdust", 0x1D2FF36D),
    ("wmat_pipegas", 0x1C8B8AED),
    ("wmat_sandbag", 0xE0D234FC),
    ("wmat_waterpuff", 0xE39A9CDB),
    ("x_candle", 0xE13CA79A),
    ("x_sconce_flame", 0x04E161C6),
    ("x_sconce_flame_smoke", 0x1F701CA8),
    ("x_spore", 0x1F44CE10),
];

#[cfg(test)]
mod tests {
    use super::*;
    use bf_viewer::bf::hash::h;

    /// The list is the catalogue's: 139 effects, each name hashing to its hash, none twice, in order
    /// by name (any case).
    #[test]
    fn the_list_is_the_catalogues() {
        assert_eq!(UNUSED_ALE.len(), 139);
        for (name, hash) in UNUSED_ALE {
            assert_eq!(h(name), hash, "{name} isn't h_{hash:08x}");
        }
        let mut hashes: Vec<u32> = UNUSED_ALE.iter().map(|e| e.1).collect();
        hashes.sort_unstable();
        hashes.dedup();
        assert_eq!(hashes.len(), 139, "a hash is listed twice");
        assert!(UNUSED_ALE.windows(2).all(|w| w[0].0.to_ascii_lowercase() < w[1].0.to_ascii_lowercase()), "the list isn't in order by name");
    }

    /// The ones the catalogue's text names are there: the MIRV and Thermalite explosions, the
    /// paralyser and stun effects, the weather, the splash family (12) and the vehicle wrecks.
    #[test]
    fn the_catalogues_named_effects_are_in_it() {
        let has = |n: &str| UNUSED_ALE.iter().any(|e| e.0 == n);
        for n in ["exp-mirv-add", "exp-thermolite-master", "paralyzer_hit", "stun_grenade", "stunned_fx", "tornado", "duststorm", "exp-hummer-master", "powerup", "blood_ragdoll"] {
            assert!(has(n), "{n}");
        }
        assert_eq!(UNUSED_ALE.iter().filter(|e| e.0.starts_with("splash_")).count(), 12);
    }
}
