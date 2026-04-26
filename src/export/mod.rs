// Offline export: renders the current scene at a chosen resolution into a
// dedicated set of HDR/bloom/output targets, reads back the tonemapped 8-bit
// pixels, and writes them to disk via `image` (PNG) or `exr` (EXR; later).
//
// Phase 6 minimum: equirectangular PNG up to 8K width. Tiling for 16K+,
// EXR linear HDR, and 6-face cubemap export are queued for Phase 6.5/7.

pub mod png;
pub mod tiling;
pub mod exr;
pub mod cubemap;
pub mod equirect;

use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportFormat {
    Png,
}

impl ExportFormat {
    pub fn extension(&self) -> &'static str {
        match self {
            ExportFormat::Png => "png",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportKind {
    Equirect,
}

#[derive(Debug, Clone)]
pub struct ExportRequest {
    pub format: ExportFormat,
    pub kind: ExportKind,
    /// Width of the output image in pixels. Equirect height = width / 2.
    pub width: u32,
    /// Optional pre-resolved destination path. If `None`, the update loop
    /// opens a file dialog before running the export.
    pub path: Option<PathBuf>,
}
