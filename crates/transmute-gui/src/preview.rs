use egui::{ColorImage, TextureHandle, TextureOptions};
use image::GenericImageView;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread::{self, JoinHandle};

/// Loading state for images
#[derive(Debug, Clone, PartialEq)]
pub enum LoadingState {
    Pending,
    Loading,
    Loaded,
    Failed(String),
}

/// Image metadata extracted during loading
#[derive(Debug, Clone)]
pub struct ImageMetadata {
    pub width: u32,
    pub height: u32,
    pub format: String,
    pub color_type: String,
    pub has_alpha: bool,
    pub file_size: u64,
}

/// Cached image with texture and metadata
pub struct CachedImage {
    pub texture: TextureHandle,
    pub metadata: ImageMetadata,
    pub state: LoadingState,
    pub last_access: std::time::Instant,
}

/// Request to load an image
#[derive(Debug)]
pub struct LoadRequest {
    pub path: PathBuf,
    pub thumbnail_size: Option<u32>,
}

/// Response from the loader thread
pub struct LoadResponse {
    pub path: PathBuf,
    pub result: Result<(ColorImage, ImageMetadata), String>,
    pub is_thumbnail: bool,
}

/// LRU texture cache with configurable max entries
pub struct TextureCache {
    cache: HashMap<PathBuf, CachedImage>,
    thumbnail_cache: HashMap<PathBuf, CachedImage>,
    max_entries: usize,
    max_thumbnail_entries: usize,
}

impl TextureCache {
    pub fn new(max_entries: usize, max_thumbnail_entries: usize) -> Self {
        Self {
            cache: HashMap::new(),
            thumbnail_cache: HashMap::new(),
            max_entries,
            max_thumbnail_entries,
        }
    }

    /// Get a full-size cached image
    pub fn get(&mut self, path: &Path) -> Option<&CachedImage> {
        if let Some(entry) = self.cache.get_mut(path) {
            entry.last_access = std::time::Instant::now();
            Some(entry)
        } else {
            None
        }
    }

    /// Get a thumbnail cached image
    pub fn get_thumbnail(&mut self, path: &Path) -> Option<&CachedImage> {
        if let Some(entry) = self.thumbnail_cache.get_mut(path) {
            entry.last_access = std::time::Instant::now();
            Some(entry)
        } else {
            None
        }
    }

    /// Insert a full-size image into cache
    pub fn insert(&mut self, path: PathBuf, image: CachedImage) {
        self.evict_full_if_needed();
        self.cache.insert(path, image);
    }

    /// Insert a thumbnail into cache
    pub fn insert_thumbnail(&mut self, path: PathBuf, image: CachedImage) {
        self.evict_thumbnails_if_needed();
        self.thumbnail_cache.insert(path, image);
    }

    /// Check if we have a thumbnail for this path
    pub fn has_thumbnail(&self, path: &Path) -> bool {
        self.thumbnail_cache.contains_key(path)
    }

    /// Check if we have a full-size image for this path
    pub fn has_full(&self, path: &Path) -> bool {
        self.cache.contains_key(path)
    }

    /// Evict oldest entries if full-size cache is full
    fn evict_full_if_needed(&mut self) {
        if self.cache.len() >= self.max_entries {
            // Find oldest entry
            if let Some((oldest_path, _)) = self
                .cache
                .iter()
                .min_by_key(|(_, v)| v.last_access)
                .map(|(k, v)| (k.clone(), v.last_access))
            {
                self.cache.remove(&oldest_path);
            }
        }
    }

    fn evict_thumbnails_if_needed(&mut self) {
        if self.thumbnail_cache.len() >= self.max_thumbnail_entries {
            if let Some((oldest_path, _)) = self
                .thumbnail_cache
                .iter()
                .min_by_key(|(_, v)| v.last_access)
                .map(|(k, v)| (k.clone(), v.last_access))
            {
                self.thumbnail_cache.remove(&oldest_path);
            }
        }
    }

    /// Clear all cached images
    pub fn clear(&mut self) {
        self.cache.clear();
        self.thumbnail_cache.clear();
    }
}

/// Background image loader with channels
pub struct ImageLoader {
    request_tx: Sender<LoadRequest>,
    response_rx: Receiver<LoadResponse>,
    _worker: JoinHandle<()>,
    pending_requests: std::collections::HashSet<PathBuf>,
}

impl ImageLoader {
    pub fn new() -> Self {
        let (request_tx, request_rx) = channel::<LoadRequest>();
        let (response_tx, response_rx) = channel::<LoadResponse>();

        let worker = thread::spawn(move || {
            Self::worker_loop(request_rx, response_tx);
        });

        Self {
            request_tx,
            response_rx,
            _worker: worker,
            pending_requests: std::collections::HashSet::new(),
        }
    }

    fn worker_loop(request_rx: Receiver<LoadRequest>, response_tx: Sender<LoadResponse>) {
        while let Ok(request) = request_rx.recv() {
            let is_thumbnail = request.thumbnail_size.is_some();
            let result = Self::load_image(&request.path, request.thumbnail_size);

            let response = LoadResponse {
                path: request.path,
                result,
                is_thumbnail,
            };

            if response_tx.send(response).is_err() {
                break;
            }
        }
    }

    fn load_image(
        path: &Path,
        thumbnail_size: Option<u32>,
    ) -> Result<(ColorImage, ImageMetadata), String> {
        // Get file size
        let file_size = std::fs::metadata(path)
            .map(|m| m.len())
            .unwrap_or(0);

        // Load image using image crate
        let img = image::open(path).map_err(|e| format!("Failed to open image: {}", e))?;

        let (width, height) = img.dimensions();

        // Extract format from extension
        let format = path
            .extension()
            .and_then(|e| e.to_str())
            .map(|s| s.to_uppercase())
            .unwrap_or_else(|| "Unknown".to_string());

        // Determine color type
        let color_type = match img.color() {
            image::ColorType::L8 => "Grayscale",
            image::ColorType::La8 => "Grayscale + Alpha",
            image::ColorType::Rgb8 => "RGB",
            image::ColorType::Rgba8 => "RGBA",
            image::ColorType::L16 => "Grayscale 16-bit",
            image::ColorType::La16 => "Grayscale + Alpha 16-bit",
            image::ColorType::Rgb16 => "RGB 16-bit",
            image::ColorType::Rgba16 => "RGBA 16-bit",
            image::ColorType::Rgb32F => "RGB 32-bit float",
            image::ColorType::Rgba32F => "RGBA 32-bit float",
            _ => "Unknown",
        }
        .to_string();

        let has_alpha = img.color().has_alpha();

        let metadata = ImageMetadata {
            width,
            height,
            format,
            color_type,
            has_alpha,
            file_size,
        };

        // Resize if thumbnail requested
        let final_img = if let Some(size) = thumbnail_size {
            img.thumbnail(size, size)
        } else {
            // Limit max preview size to 800px
            if width > 800 || height > 800 {
                img.thumbnail(800, 800)
            } else {
                img
            }
        };

        // Convert to RGBA for egui
        let rgba = final_img.to_rgba8();
        let (w, h) = rgba.dimensions();
        let pixels = rgba.into_raw();

        let color_image = ColorImage::from_rgba_unmultiplied([w as usize, h as usize], &pixels);

        Ok((color_image, metadata))
    }

    /// Request loading a thumbnail
    pub fn request_thumbnail(&mut self, path: PathBuf) {
        if self.pending_requests.contains(&path) {
            return;
        }

        self.pending_requests.insert(path.clone());
        let _ = self.request_tx.send(LoadRequest {
            path,
            thumbnail_size: Some(48),
        });
    }

    /// Request loading a full-size preview image
    pub fn request_full(&mut self, path: PathBuf) {
        if self.pending_requests.contains(&path) {
            return;
        }

        self.pending_requests.insert(path.clone());
        let _ = self.request_tx.send(LoadRequest {
            path,
            thumbnail_size: None,
        });
    }

    /// Poll for completed loads and return them
    pub fn poll_responses(&mut self) -> Vec<LoadResponse> {
        let mut responses = Vec::new();
        while let Ok(response) = self.response_rx.try_recv() {
            self.pending_requests.remove(&response.path);
            responses.push(response);
        }
        responses
    }

    /// Check if a path has a pending request
    pub fn is_pending(&self, path: &Path) -> bool {
        self.pending_requests.contains(path)
    }
}

impl Default for ImageLoader {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a texture from a ColorImage
pub fn create_texture(
    ctx: &egui::Context,
    name: impl Into<String>,
    image: ColorImage,
) -> TextureHandle {
    ctx.load_texture(name, image, TextureOptions::LINEAR)
}

/// Format file size for display
pub fn format_file_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = 1024 * KB;
    const GB: u64 = 1024 * MB;

    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}
