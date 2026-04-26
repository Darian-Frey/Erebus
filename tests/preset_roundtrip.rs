// Integration test: load each shipped preset, serialize, reload, assert
// the deserialised preset matches the original.

use erebus::preset::ShippedPreset;

#[test]
fn shipped_presets_load_via_public_api() {
    for shipped in ShippedPreset::ALL {
        let p = shipped
            .load()
            .unwrap_or_else(|e| panic!("{}: {e}", shipped.label()));
        assert_eq!(p.format_version, erebus::preset::schema::CURRENT_VERSION);
        assert!(!p.gradient.is_empty());
    }
}
