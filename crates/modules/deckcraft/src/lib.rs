//! DeckCraft (Microsoft PowerPoint–class presentation app) as a Craft Studio module.

#[allow(dead_code, unused_imports, unused_variables, unused_mut, unused_macros, clippy::all)]
mod upstream {
    include!(concat!(env!("OUT_DIR"), "/upstream/main.rs"));
    pub(crate) mod studio;
}

pub const MODULE: studio_core::Module = studio_core::Module {
    info: studio_core::ModuleInfo { id: "deckcraft", name: "DeckCraft", monogram: "Dk", summary: "Presentations and slide shows", accent: [0xd2, 0x4b, 0x26] },
    create: upstream::studio::create,
};
