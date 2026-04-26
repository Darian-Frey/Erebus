// Offline export: renders the current scene at a chosen resolution into a
// dedicated set of HDR/bloom/output targets, reads back the tonemapped 8-bit
// pixels, and writes them to disk via `image` (PNG) or `exr` (EXR; later).
//
// Phase 6 minimum: equirectangular PNG up to 8K width. Tiling for 16K+,
// EXR linear HDR, and 6-face cubemap export are queued for Phase 6.5/7.

pub mod png;
pub mod tiling;
pub mod exr;
#[cfg(not(target_arch = "wasm32"))]
pub mod cubemap;
pub mod equirect;
#[cfg(target_arch = "wasm32")]
pub mod web;

use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportFormat {
    Png,
    Exr,
}

impl ExportFormat {
    pub fn extension(&self) -> &'static str {
        match self {
            ExportFormat::Png => "png",
            ExportFormat::Exr => "exr",
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            ExportFormat::Png => "PNG (sRGB tonemapped)",
            ExportFormat::Exr => "EXR (linear HDR)",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportKind {
    Equirect,
    Cubemap,
}

impl ExportKind {
    pub fn label(&self) -> &'static str {
        match self {
            ExportKind::Equirect => "Equirect (2:1)",
            ExportKind::Cubemap => "Cubemap (6 faces)",
        }
    }
}

#[derive(Debug, Clone)]
pub struct ExportRequest {
    pub format: ExportFormat,
    pub kind: ExportKind,
    /// For equirect: width of the output image (height = width / 2).
    /// For cubemap: per-face square dimension.
    pub width: u32,
    /// Optional pre-resolved destination path. If `None`, the update loop
    /// opens a file dialog before running the export.
    pub path: Option<PathBuf>,
}
