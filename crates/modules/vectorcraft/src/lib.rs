//! VectorCraft (Adobe Illustrator–class vector editor) as a Craft Studio module.

#[allow(dead_code, unused_imports, unused_variables, unused_mut, unused_macros, clippy::all)]
mod upstream {
    include!(concat!(env!("OUT_DIR"), "/upstream/main.rs"));
    pub(crate) mod studio;
}

pub const MODULE: studio_core::Module = studio_core::Module {
    info: studio_core::ModuleInfo { id: "vectorcraft", name: "VectorCraft", monogram: "Ve", summary: "Vector illustration", accent: [0xff, 0x9a, 0x00] },
    create: upstream::studio::create,
};
