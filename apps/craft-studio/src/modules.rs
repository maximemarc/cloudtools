//! The apps compiled into this build: one Cargo feature per app, all on by default
//! (`cargo run -p craft-studio --no-default-features --features photocraft,pdfcraft`).

// One push per enabled feature, so no `vec![]`.
#[allow(clippy::vec_init_then_push)]
pub fn all() -> Vec<studio_core::Module> {
    #[allow(unused_mut)]
    let mut modules = Vec::new();
    #[cfg(feature = "artcraft")]
    modules.push(studio_artcraft::MODULE);
    #[cfg(feature = "photocraft")]
    modules.push(studio_photocraft::MODULE);
    #[cfg(feature = "lightcraft")]
    modules.push(studio_lightcraft::MODULE);
    #[cfg(feature = "filmcraft")]
    modules.push(studio_filmcraft::MODULE);
    #[cfg(feature = "effectcraft")]
    modules.push(studio_effectcraft::MODULE);
    #[cfg(feature = "vectorcraft")]
    modules.push(studio_vectorcraft::MODULE);
    #[cfg(feature = "designcraft")]
    modules.push(studio_designcraft::MODULE);
    #[cfg(feature = "pdfcraft")]
    modules.push(studio_pdfcraft::MODULE);
    #[cfg(feature = "wordcraft")]
    modules.push(studio_wordcraft::MODULE);
    #[cfg(feature = "gridcraft")]
    modules.push(studio_gridcraft::MODULE);
    #[cfg(feature = "deckcraft")]
    modules.push(studio_deckcraft::MODULE);
    #[cfg(feature = "cadcraft")]
    modules.push(studio_cadcraft::MODULE);
    #[cfg(feature = "soundcraft")]
    modules.push(studio_soundcraft::MODULE);
    modules
}
