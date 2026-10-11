//! DesignCraft (Adobe InDesign–class layout app) as a Craft Studio module.

#[allow(dead_code, unused_imports, unused_variables, unused_mut, unused_macros, clippy::all)]
mod upstream {
    include!(concat!(env!("OUT_DIR"), "/upstream/main.rs"));
    pub(crate) mod studio;
}

pub const MODULE: studio_core::Module = studio_core::Module {
    info: studio_core::ModuleInfo { id: "designcraft", name: "DesignCraft", monogram: "De", summary: "Page layout and publishing", accent: [0xff, 0x36, 0x8a] },
    create: upstream::studio::create,
};
