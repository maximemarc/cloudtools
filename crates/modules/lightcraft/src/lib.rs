//! LightCraft (Adobe Lightroom–class photo library and raw developer) as a Craft Studio module.

#[allow(dead_code, unused_imports, unused_variables, unused_mut, unused_macros, clippy::all)]
mod upstream {
    include!(concat!(env!("OUT_DIR"), "/upstream/main.rs"));
    pub(crate) mod studio;
}

pub const MODULE: studio_core::Module = studio_core::Module {
    info: studio_core::ModuleInfo { id: "lightcraft", name: "LightCraft", monogram: "Li", summary: "Photo library and raw development", accent: [0x2e, 0xc4, 0xb6] },
    create: upstream::studio::create,
};
