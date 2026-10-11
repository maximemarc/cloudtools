//! PdfCraft (Adobe Acrobat–class PDF workbench) as a Craft Studio module.

#[allow(dead_code, unused_imports, unused_variables, unused_mut, unused_macros, clippy::all)]
mod upstream {
    include!(concat!(env!("OUT_DIR"), "/upstream/main.rs"));
    pub(crate) mod studio;
}

pub const MODULE: studio_core::Module = studio_core::Module {
    info: studio_core::ModuleInfo { id: "pdfcraft", name: "PdfCraft", monogram: "Pd", summary: "Read, edit, fill and sign PDFs", accent: [0xe0, 0x3c, 0x31] },
    create: upstream::studio::create,
};
