//! SoundCraft (Avid Pro Tools–class audio workstation) as a Craft Studio module.

#[allow(dead_code, unused_imports, unused_variables, unused_mut, unused_macros, clippy::all)]
mod upstream {
    include!(concat!(env!("OUT_DIR"), "/upstream/main.rs"));
    pub(crate) mod studio;
}

pub const MODULE: studio_core::Module = studio_core::Module {
    info: studio_core::ModuleInfo {
        id: "soundcraft",
        name: "SoundCraft",
        monogram: "So",
        summary: "Recording, editing and mixing audio",
        accent: [0xc0, 0x8a, 0x2e],
    },
    create: upstream::studio::create,
};
