//! PhotoCraft (Adobe Photoshop–class image editor) as a Craft Studio module.

#[allow(dead_code, unused_imports, unused_variables, unused_mut, unused_macros, clippy::all)]
mod upstream {
    include!(concat!(env!("OUT_DIR"), "/upstream/main.rs"));
    pub(crate) mod studio;
}

pub const MODULE: studio_core::Module = studio_core::Module {
    info: studio_core::ModuleInfo {
        id: "photocraft",
        name: "PhotoCraft",
        monogram: "Ph",
        summary: "Image editing, layers and PSD",
        accent: [0x31, 0xa8, 0xff],
    },
    create: upstream::studio::create,
};
