//! CADCraft (Autodesk AutoCAD–class drafting app) as a Craft Studio module.

#[allow(dead_code, unused_imports, unused_variables, unused_mut, unused_macros, clippy::all)]
mod upstream {
    include!(concat!(env!("OUT_DIR"), "/upstream/main.rs"));
    pub(crate) mod studio;
}

pub const MODULE: studio_core::Module = studio_core::Module {
    info: studio_core::ModuleInfo {
        id: "cadcraft",
        name: "CADCraft",
        monogram: "Ca",
        summary: "Computer-aided design and drafting",
        accent: [0xc8, 0x3a, 0x3a],
    },
    create: upstream::studio::create,
};
