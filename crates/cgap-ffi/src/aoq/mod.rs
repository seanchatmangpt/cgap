// Hand-written module root (irreducible residue): the canonical aoq-pack
// generates frame.rs / kind_enums.rs / status_maps.rs; this root re-exports
// them and carries only lint waivers. ggen sync never overwrites this file.

//! AOQ1 wire-law surface projected from the canonical marketplace aoq-pack
//! (`../ggen-marketplace/packs/aoq-pack`), flipped back from the GA-5 staged
//! fallback (`ontology-packs/aoq`, retired by lane GA-5b, 2026-11-04).

// Variant names ROTATED_1/ROTATED_2 are graph-carried (aoq:rustName); the
// generated files are never hand-edited, so the lint is waived at the root.
#[allow(non_camel_case_types)]
pub mod frame;
#[allow(non_camel_case_types)]
pub mod kind_enums;
#[allow(non_camel_case_types)]
pub mod status_maps;
