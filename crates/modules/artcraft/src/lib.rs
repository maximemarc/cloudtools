//! ArtCraft (ArtCraft's native generation app; needs an ArtCraft account) as a Craft Studio module.

#[allow(dead_code, unused_imports, unused_variables, unused_mut, unused_macros, clippy::all)]
mod upstream {
    include!(concat!(env!("OUT_DIR"), "/upstream/main.rs"));
    pub(crate) mod studio;
}

pub const MODULE: studio_core::Module = studio_core::Module {
    info: studio_core::ModuleInfo {
        id: "artcraft",
        name: "ArtCraft",
        monogram: "Ar",
        summary: "AI image, video and audio generation",
        accent: [0x3d, 0x6b, 0xff],
    },
    create: upstream::studio::create,
};
