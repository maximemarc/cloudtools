//! WordCraft (Microsoft Word–class word processor) as a Craft Studio module.

#[allow(dead_code, unused_imports, unused_variables, unused_mut, unused_macros, clippy::all)]
mod upstream {
    include!(concat!(env!("OUT_DIR"), "/upstream/main.rs"));
    pub(crate) mod studio;
}

pub const MODULE: studio_core::Module = studio_core::Module {
    info: studio_core::ModuleInfo { id: "wordcraft", name: "WordCraft", monogram: "Wo", summary: "Writing and document design", accent: [0x2b, 0x5c, 0xd6] },
    create: upstream::studio::create,
};
