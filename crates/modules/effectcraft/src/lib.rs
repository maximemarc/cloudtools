//! EffectCraft (Adobe After Effects–class compositor) as a Craft Studio module.

#[allow(dead_code, unused_imports, unused_variables, unused_mut, unused_macros, clippy::all)]
mod upstream {
    include!(concat!(env!("OUT_DIR"), "/upstream/main.rs"));
    pub(crate) mod studio;
}

pub const MODULE: studio_core::Module = studio_core::Module {
    info: studio_core::ModuleInfo { id: "effectcraft", name: "EffectCraft", monogram: "Ef", summary: "Motion graphics and visual effects", accent: [0x7c, 0x6c, 0xff] },
    create: upstream::studio::create,
};
