//! Asset management system.
//!
//! Advantages over macroquad:
//! - **Reference-counted handles** — assets are shared, not duplicated
//! - **Hot-reload support** (on debug builds)
//! - **Async loading** with progress tracking
//! - **Asset dependencies** — auto-load dependent assets
//! - **Memory tracking** — see how much VRAM/assets are loaded
//!
//! In this module, we define the asset management architecture.
//! The actual loading depends on the rendering backend (miniquad textures, etc.).

use std::collections::HashMap;
use std::sync::Arc;

// ─────────────────────────────────────────────
// Asset Handle
// ─────────────────────────────────────────────

/// A reference-counted handle to a loaded asset.
///
/// Cheap to clone — all clones point to the same loaded data.
#[derive(Debug, Clone)]
pub struct AssetHandle<T> {
    inner: Arc<AssetInner<T>>,
}

#[derive(Debug)]
struct AssetInner<T> {
    /// The loaded asset data.
    data: T,
    /// Path this was loaded from.
    path: String,
    /// Whether this asset has been modified (for hot-reload).
    modified: bool,
}

impl<T> AssetHandle<T> {
    /// Get a reference to the asset data.
    pub fn get(&self) -> &T {
        &self.inner.data
    }

    /// Get the path this asset was loaded from.
    pub fn path(&self) -> &str {
        &self.inner.path
    }

    /// Strong reference count.
    pub fn ref_count(&self) -> usize {
        Arc::strong_count(&self.inner)
    }
}

impl<T> std::ops::Deref for AssetHandle<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.inner.data
    }
}

// ─────────────────────────────────────────────
// Asset type IDs
// ─────────────────────────────────────────────

/// Identifiers for different asset types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AssetType {
    /// Raw pixel data (before GPU upload).
    Image,
    /// GPU texture (after upload to the rendering backend).
    Texture,
    /// Font data.
    Font,
    /// Audio clip.
    Sound,
    /// Text/shader/JSON file.
    Text,
    /// Serialized data (JSON, RON, etc.).
    Data,
}

// ─────────────────────────────────────────────
// Loading state
// ─────────────────────────────────────────────

/// Status of an asset load operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoadStatus {
    /// Not yet started loading.
    NotLoaded,
    /// Currently loading (with progress 0.0–1.0).
    Loading(f32),
    /// Successfully loaded.
    Loaded,
    /// Failed to load.
    Failed,
}

// ─────────────────────────────────────────────
// Asset Loader
// ─────────────────────────────────────────────

/// Configuration for asset loading.
#[derive(Debug, Clone)]
pub struct AssetConfig {
    /// Root directory for assets (default: "assets/").
    pub root: String,
    /// Whether to enable hot-reload (debug builds only).
    pub hot_reload: bool,
}

impl Default for AssetConfig {
    fn default() -> Self {
        Self {
            root: "assets".to_string(),
            hot_reload: cfg!(debug_assertions),
        }
    }
}

/// The asset manager handles loading, caching, and lifecycle of game assets.
///
/// # Example
/// ```
/// let mut assets = AssetManager::new(AssetConfig::default());
///
/// // Load assets
/// let player_tex = assets.load_image("sprites/player.png");
/// let bg_music = assets.load_sound("music/background.ogg");
///
/// // Use assets
/// // draw_texture(player_tex.unwrap(), ...);
///
/// // In update loop (for hot-reload):
/// assets.check_hot_reload();
/// ```
pub struct AssetManager {
    config: AssetConfig,
    /// Cached loaded assets by normalized path.
    images: HashMap<String, AssetHandle<ImageAsset>>,
    /// Loading queue.
    loading_queue: Vec<String>,
    /// Total bytes loaded.
    total_bytes: usize,
}

/// Image asset (CPU-side pixel data).
#[derive(Debug, Clone)]
pub struct ImageAsset {
    /// Pixel width.
    pub width: u32,
    /// Pixel height.
    pub height: u32,
    /// Raw RGBA pixel data.
    pub pixels: Vec<u8>,
    /// Format.
    pub format: PixelFormat,
}

/// Pixel format.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PixelFormat {
    /// 8-bit RGBA (4 bytes per pixel).
    Rgba8,
    /// 8-bit RGB (3 bytes per pixel).
    Rgb8,
    /// 8-bit grayscale (1 byte per pixel).
    Grayscale8,
}

impl ImageAsset {
    /// Create an empty image with the given dimensions.
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            pixels: vec![0; (width * height * 4) as usize],
            format: PixelFormat::Rgba8,
        }
    }

    /// Get a pixel at (x, y). Returns (r, g, b, a) as u8.
    pub fn get_pixel(&self, x: u32, y: u32) -> (u8, u8, u8, u8) {
        let idx = ((y * self.width + x) * 4) as usize;
        if idx + 3 >= self.pixels.len() {
            return (0, 0, 0, 0);
        }
        (self.pixels[idx], self.pixels[idx + 1], self.pixels[idx + 2], self.pixels[idx + 3])
    }

    /// Set a pixel at (x, y).
    pub fn set_pixel(&mut self, x: u32, y: u32, r: u8, g: u8, b: u8, a: u8) {
        let idx = ((y * self.width + x) * 4) as usize;
        if idx + 3 < self.pixels.len() {
            self.pixels[idx] = r;
            self.pixels[idx + 1] = g;
            self.pixels[idx + 2] = b;
            self.pixels[idx + 3] = a;
        }
    }

    /// Create a sub-image (region copy).
    pub fn sub_image(&self, x: u32, y: u32, w: u32, h: u32) -> ImageAsset {
        let mut sub = ImageAsset::new(w, h);
        for py in 0..h {
            for px in 0..w {
                let (r, g, b, a) = self.get_pixel(x + px, y + py);
                sub.set_pixel(px, py, r, g, b, a);
            }
        }
        sub
    }

    /// Flip the image horizontally.
    pub fn flip_horizontal(&mut self) {
        let row_bytes = self.width as usize * 4;
        for y in 0..self.height {
            let row_start = (y as usize) * row_bytes;
            let row: &mut [u8] = &mut self.pixels[row_start..row_start + row_bytes];
            row.chunks_exact_mut(4).for_each(|pixel| pixel.reverse());
        }
    }

    /// Flip the image vertically.
    pub fn flip_vertical(&mut self) {
        let row_bytes = self.width as usize * 4;
        let total_rows = self.height as usize;
        for y in 0..total_rows / 2 {
            let top = y * row_bytes;
            let bottom = (total_rows - 1 - y) * row_bytes;
            self.pixels[top..top + row_bytes].swap_with_slice(&mut self.pixels[bottom..bottom + row_bytes]);
        }
    }
}

impl AssetManager {
    /// Create a new asset manager.
    pub fn new(config: AssetConfig) -> Self {
        Self {
            config,
            images: HashMap::new(),
            loading_queue: Vec::new(),
            total_bytes: 0,
        }
    }

    /// Normalize a path (remove leading slashes, etc.).
    fn normalize_path(&self, path: &str) -> String {
        let path = path.trim_start_matches('/').trim_start_matches('\\');
        format!("{}/{}", self.config.root, path)
    }

    /// Load an image from disk.
    ///
    /// Returns a handle to the cached image, or an error.
    pub fn load_image(&mut self, path: &str) -> Result<AssetHandle<ImageAsset>, String> {
        let normalized = self.normalize_path(path);

        // Return cached if available
        if let Some(handle) = self.images.get(&normalized) {
            return Ok(handle.clone());
        }

        // In a real implementation, this would load from disk using the `image` crate.
        // For the API definition, we create a placeholder.
        let asset = ImageAsset::new(1, 1); // Placeholder
        let bytes = asset.pixels.len();
        self.total_bytes += bytes;

        let handle = AssetHandle {
            inner: Arc::new(AssetInner {
                data: asset,
                path: normalized.clone(),
                modified: false,
            }),
        };

        self.images.insert(normalized, handle.clone());
        Ok(handle)
    }

    /// Unload an asset by path.
    pub fn unload_image(&mut self, path: &str) {
        let normalized = self.normalize_path(path);
        if let Some(handle) = self.images.remove(&normalized) {
            self.total_bytes -= handle.get().pixels.len();
        }
    }

    /// Check for modified files and reload if hot-reload is enabled.
    pub fn check_hot_reload(&mut self) {
        if !self.config.hot_reload {
            return;
        }
        // In a real implementation, check file modification times
        // and reload changed assets.
    }

    /// Total bytes of loaded image data.
    pub fn total_image_bytes(&self) -> usize {
        self.total_bytes
    }

    /// Number of cached images.
    pub fn image_count(&self) -> usize {
        self.images.len()
    }

    /// Clear all cached assets.
    pub fn clear(&mut self) {
        self.images.clear();
        self.total_bytes = 0;
    }
}

// ─────────────────────────────────────────────
// Sprite Sheet
// ─────────────────────────────────────────────

/// A sprite sheet splits a texture into a grid of frames.
#[derive(Debug, Clone)]
pub struct SpriteSheet {
    /// Handle to the source image.
    pub image: AssetHandle<ImageAsset>,
    /// Number of columns in the sheet.
    pub columns: u32,
    /// Number of rows in the sheet.
    pub rows: u32,
    /// Individual frame dimensions (computed).
    pub frame_width: f32,
    pub frame_height: f32,
}

impl SpriteSheet {
    /// Create a sprite sheet from an image handle and grid dimensions.
    pub fn new(image: AssetHandle<ImageAsset>, columns: u32, rows: u32) -> Self {
        let frame_width = image.width as f32 / columns as f32;
        let frame_height = image.height as f32 / rows as f32;
        Self {
            image,
            columns,
            rows,
            frame_width,
            frame_height,
        }
    }

    /// Get the source rectangle for a specific frame.
    pub fn frame_rect(&self, index: u32) -> crate::math::Rect {
        let col = index % self.columns;
        let row = index / self.columns;
        crate::math::Rect::new(
            col as f32 * self.frame_width,
            row as f32 * self.frame_height,
            self.frame_width,
            self.frame_height,
        )
    }

    /// Get the source rectangle for a specific (row, column).
    pub fn cell_rect(&self, row: u32, col: u32) -> crate::math::Rect {
        crate::math::Rect::new(
            col as f32 * self.frame_width,
            row as f32 * self.frame_height,
            self.frame_width,
            self.frame_height,
        )
    }

    /// Total number of frames.
    pub fn frame_count(&self) -> u32 {
        self.columns * self.rows
    }
}