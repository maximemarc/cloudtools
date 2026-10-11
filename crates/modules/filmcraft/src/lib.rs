//! FilmCraft (Adobe Premiere Pro–class video editor) as a Craft Studio module.

#[allow(dead_code, unused_imports, unused_variables, unused_mut, unused_macros, clippy::all)]
mod upstream {
    include!(concat!(env!("OUT_DIR"), "/upstream/main.rs"));
    pub(crate) mod studio;
}

pub const MODULE: studio_core::Module = studio_core::Module {
    info: studio_core::ModuleInfo {
        id: "filmcraft",
        name: "FilmCraft",
        monogram: "Fi",
        summary: "Video editing, colour and sound",
        accent: [0x99, 0x5b, 0xff],
    },
    create: upstream::studio::create,
};
