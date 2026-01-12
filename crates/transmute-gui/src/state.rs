use parking_lot::Mutex;
use std::path::PathBuf;
use std::sync::Arc;
use transmute_common::MediaFormat;
use transmute_compress::QualitySettings;

/// Application state (shared across UI and background tasks)
#[derive(Clone)]
pub struct AppState {
    inner: Arc<Mutex<AppStateInner>>,
}

struct AppStateInner {
    /// Input files queue
    pub input_files: Vec<InputFile>,

    /// Selected operation
    pub operation: Operation,

    /// Target format for conversion
    pub target_format: MediaFormat,

    /// Quality setting for compression
    pub quality: QualitySettings,

    /// Scale factor for enhancement
    pub scale_factor: u32,

    /// Output directory
    pub output_dir: Option<PathBuf>,

    /// Current processing state
    pub processing: ProcessingState,

    /// Natural language command
    pub nl_command: String,

    /// Settings
    pub settings: Settings,
}

#[derive(Debug, Clone)]
pub struct InputFile {
    pub path: PathBuf,
    pub status: FileStatus,
    pub output_path: Option<PathBuf>,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum FileStatus {
    Pending,
    Processing,
    Complete,
    Failed,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Operation {
    Convert,
    Compress,
    Enhance,
    Merge,
}

#[derive(Debug, Clone)]
pub enum ProcessingState {
    Idle,
    Running { current: usize, total: usize },
    Complete { success: usize, failed: usize },
}

#[derive(Debug, Clone)]
pub struct Settings {
    pub use_gpu: bool,
    pub auto_open_output: bool,
    pub dark_mode: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            use_gpu: true,
            auto_open_output: false,
            dark_mode: true,
        }
    }
}

impl AppState {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(AppStateInner {
                input_files: Vec::new(),
                operation: Operation::Convert,
                target_format: MediaFormat::Jpeg,
                quality: QualitySettings::High,
                scale_factor: 2,
                output_dir: None,
                processing: ProcessingState::Idle,
                nl_command: String::new(),
                settings: Settings::default(),
            })),
        }
    }

    /// Add files to input queue
    pub fn add_files(&self, paths: Vec<PathBuf>) {
        let mut inner = self.inner.lock();
        for path in paths {
            // Avoid duplicates
            if !inner.input_files.iter().any(|f| f.path == path) {
                inner.input_files.push(InputFile {
                    path,
                    status: FileStatus::Pending,
                    output_path: None,
                    error_message: None,
                });
            }
        }
    }

    /// Clear all input files
    pub fn clear_files(&self) {
        let mut inner = self.inner.lock();
        inner.input_files.clear();
        inner.processing = ProcessingState::Idle;
    }

    /// Remove file at index
    pub fn remove_file(&self, index: usize) {
        let mut inner = self.inner.lock();
        if index < inner.input_files.len() {
            inner.input_files.remove(index);
        }
    }

    /// Get current operation
    pub fn operation(&self) -> Operation {
        self.inner.lock().operation
    }

    /// Set operation
    pub fn set_operation(&self, op: Operation) {
        self.inner.lock().operation = op;
    }

    /// Get target format
    pub fn target_format(&self) -> MediaFormat {
        self.inner.lock().target_format
    }

    /// Set target format
    pub fn set_target_format(&self, format: MediaFormat) {
        self.inner.lock().target_format = format;
    }

    /// Get quality setting
    pub fn quality(&self) -> QualitySettings {
        self.inner.lock().quality
    }

    /// Set quality
    pub fn set_quality(&self, quality: QualitySettings) {
        self.inner.lock().quality = quality;
    }

    /// Get scale factor
    pub fn scale_factor(&self) -> u32 {
        self.inner.lock().scale_factor
    }

    /// Set scale factor
    pub fn set_scale_factor(&self, scale: u32) {
        self.inner.lock().scale_factor = scale;
    }

    /// Get output directory
    pub fn output_dir(&self) -> Option<PathBuf> {
        self.inner.lock().output_dir.clone()
    }

    /// Set output directory
    pub fn set_output_dir(&self, dir: Option<PathBuf>) {
        self.inner.lock().output_dir = dir;
    }

    /// Get input files (cloned)
    pub fn input_files(&self) -> Vec<InputFile> {
        self.inner.lock().input_files.clone()
    }

    /// Get processing state
    pub fn processing_state(&self) -> ProcessingState {
        self.inner.lock().processing.clone()
    }

    /// Update file status
    pub fn update_file_status(
        &self,
        index: usize,
        status: FileStatus,
        output: Option<PathBuf>,
        error: Option<String>,
    ) {
        let mut inner = self.inner.lock();
        if let Some(file) = inner.input_files.get_mut(index) {
            file.status = status;
            file.output_path = output;
            file.error_message = error;
        }
    }

    /// Set processing state
    pub fn set_processing_state(&self, state: ProcessingState) {
        self.inner.lock().processing = state;
    }

    /// Get NL command
    pub fn nl_command(&self) -> String {
        self.inner.lock().nl_command.clone()
    }

    /// Set NL command
    pub fn set_nl_command(&self, cmd: String) {
        self.inner.lock().nl_command = cmd;
    }

    /// Get settings
    pub fn settings(&self) -> Settings {
        self.inner.lock().settings.clone()
    }

    /// Update settings
    pub fn update_settings<F>(&self, f: F)
    where
        F: FnOnce(&mut Settings),
    {
        let mut inner = self.inner.lock();
        f(&mut inner.settings);
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}
