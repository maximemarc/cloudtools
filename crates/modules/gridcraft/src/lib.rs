//! GridCraft (Microsoft Excel–class spreadsheet) as a Craft Studio module.

#[allow(dead_code, unused_imports, unused_variables, unused_mut, unused_macros, clippy::all)]
mod upstream {
    include!(concat!(env!("OUT_DIR"), "/upstream/main.rs"));
    pub(crate) mod studio;
}

pub const MODULE: studio_core::Module = studio_core::Module {
    info: studio_core::ModuleInfo { id: "gridcraft", name: "GridCraft", monogram: "Gr", summary: "Spreadsheets and data", accent: [0x21, 0x8a, 0x4c] },
    create: upstream::studio::create,
};
