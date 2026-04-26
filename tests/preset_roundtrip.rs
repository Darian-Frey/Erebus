// The Phase 7 preset roundtrip lives as a unit test inside
// `src/preset/mod.rs` since this crate is binary-only and integration
// tests can't reach internal modules without a separate library crate.
// Phase 8 polish: extract a `lib.rs` so this becomes a real integration
// test that mirrors how external consumers would use the preset API.
