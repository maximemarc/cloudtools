//! The GPU device. Every app renders on the one device the window opens, so it is requested with
//! what the most demanding standalone app asks for, not egui's default (WebGPU's baseline limits
//! and 8192 px textures):
//!
//! - EffectCraft: every limit the adapter offers, 2D textures capped at 16384 px like its headless
//!   renderer, so its compositor sizes work the same in the window and in exports;
//! - PhotoCraft: the adapter's texture and buffer sizes (large canvases and layers);
//! - PdfCraft: the adapter's texture size (surfaces wider than 8192 px across several displays).
//!
//! PhotoCraft and PdfCraft take the adapter's texture size uncapped when they run alone; here they
//! get at most 16384 px, which only matters on adapters that offer more (they tile below it).

/// The largest 2D texture side requested (EffectCraft's cap).
const MAX_TEXTURE_SIDE: u32 = 16384;

pub fn wgpu_options() -> eframe::WgpuConfiguration {
    let mut config = eframe::WgpuConfiguration::default();
    if let eframe::egui_wgpu::WgpuSetup::CreateNew(setup) = &mut config.wgpu_setup {
        setup.device_descriptor = std::sync::Arc::new(|adapter| {
            let limits = device_limits(adapter.limits());
            let info = adapter.get_info();
            log::info!("gpu: {} ({:?}), textures up to {} px", info.name, info.backend, limits.max_texture_dimension_2d);
            eframe::wgpu::DeviceDescriptor { label: Some("Craft Studio device"), required_limits: limits, ..Default::default() }
        });
    }
    config
}

/// The limits to request from an adapter that offers `adapter`.
fn device_limits(adapter: eframe::wgpu::Limits) -> eframe::wgpu::Limits {
    eframe::wgpu::Limits { max_texture_dimension_2d: adapter.max_texture_dimension_2d.min(MAX_TEXTURE_SIDE), ..adapter }
}

#[cfg(test)]
mod tests {
    use super::*;
    use eframe::wgpu::Limits;

    #[test]
    fn caps_textures_and_keeps_the_adapter_limits() {
        let big = Limits { max_texture_dimension_2d: 32768, max_buffer_size: 1 << 34, ..Limits::default() };
        let limits = device_limits(big.clone());
        assert_eq!(limits.max_texture_dimension_2d, MAX_TEXTURE_SIDE);
        assert_eq!(limits.max_buffer_size, big.max_buffer_size);
        assert_eq!(limits.max_storage_buffer_binding_size, big.max_storage_buffer_binding_size);
    }

    #[test]
    fn never_asks_for_more_than_the_adapter_offers() {
        let small = Limits::downlevel_webgl2_defaults();
        let limits = device_limits(small.clone());
        assert_eq!(limits.max_texture_dimension_2d, small.max_texture_dimension_2d);
        assert!(limits.check_limits(&small));
    }
}
