use crate::state::{AppState, FileStatus, Operation, ProcessingState};
use crate::theme::Theme;
use crate::widgets;
use egui::{CentralPanel, ScrollArea, SidePanel, TopBottomPanel};
use std::path::PathBuf;
use std::sync::Arc;
use transmute_core::Converter;

pub struct TransmuteApp {
    state: AppState,
    converter: Arc<Converter>,
    show_settings: bool,
    processing_handle: Option<std::thread::JoinHandle<()>>,
}

impl TransmuteApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Configure theme
        Theme::configure(&cc.egui_ctx);

        // Initialize converter
        let converter = Converter::new().expect("Failed to initialize converter");

        Self {
            state: AppState::new(),
            converter: Arc::new(converter),
            show_settings: false,
            processing_handle: None,
        }
    }

    fn render_top_bar(&mut self, _ctx: &egui::Context, ui: &mut egui::Ui) {
        // Add consistent padding to top bar
        ui.add_space(4.0);

        ui.horizontal(|ui| {
            ui.add_space(8.0);

            // App title with better spacing
            ui.heading(egui::RichText::new("Transmute").size(20.0));

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.add_space(8.0);

                // Settings button with consistent styling
                if ui
                    .button(egui::RichText::new("Settings").size(14.0))
                    .clicked()
                {
                    self.show_settings = !self.show_settings;
                }

                ui.add_space(12.0);

                // GPU status indicator with clear visual separation
                let settings = self.state.settings();
                let (gpu_text, gpu_color) = if settings.use_gpu {
                    ("GPU Enabled", Theme::SUCCESS)
                } else {
                    ("CPU Mode", Theme::TEXT_SECONDARY)
                };

                ui.label(
                    egui::RichText::new(gpu_text)
                        .size(13.0)
                        .color(gpu_color)
                );
            });
        });

        ui.add_space(4.0);
    }

    fn render_operation_selector(&mut self, ui: &mut egui::Ui) {
        // Section label with consistent styling
        ui.label(
            egui::RichText::new("Operation")
                .size(15.0)
                .color(Theme::TEXT_PRIMARY)
        );
        ui.add_space(8.0);

        let current_op = self.state.operation();

        // Vertical layout for better readability
        ui.vertical(|ui| {
            if ui
                .selectable_label(current_op == Operation::Convert, "Convert Format")
                .clicked()
            {
                self.state.set_operation(Operation::Convert);
            }

            if ui
                .selectable_label(current_op == Operation::Compress, "Compress Image")
                .clicked()
            {
                self.state.set_operation(Operation::Compress);
            }

            if ui
                .selectable_label(current_op == Operation::Enhance, "Enhance Quality")
                .clicked()
            {
                self.state.set_operation(Operation::Enhance);
            }

            if ui
                .selectable_label(current_op == Operation::Merge, "Merge to PDF")
                .clicked()
            {
                self.state.set_operation(Operation::Merge);
            }
        });
    }

    fn render_operation_settings(&mut self, ui: &mut egui::Ui) {
        // Settings panel with proper frame and padding
        egui::Frame::group(ui.style())
            .inner_margin(egui::Margin::same(12.0))
            .show(ui, |ui| {
                match self.state.operation() {
                    Operation::Convert => {
                        ui.label(
                            egui::RichText::new("Format Settings")
                                .size(14.0)
                                .color(Theme::TEXT_PRIMARY)
                        );
                        ui.add_space(8.0);

                        let mut format = self.state.target_format();
                        if widgets::format_selector(ui, &mut format) {
                            self.state.set_target_format(format);
                        }
                    }

                    Operation::Compress => {
                        ui.label(
                            egui::RichText::new("Compression Settings")
                                .size(14.0)
                                .color(Theme::TEXT_PRIMARY)
                        );
                        ui.add_space(8.0);

                        let mut quality = self.state.quality();
                        if widgets::quality_selector(ui, &mut quality) {
                            self.state.set_quality(quality);
                        }
                    }

                    Operation::Enhance => {
                        ui.label(
                            egui::RichText::new("Enhancement Settings")
                                .size(14.0)
                                .color(Theme::TEXT_PRIMARY)
                        );
                        ui.add_space(8.0);

                        ui.label("Scale Factor");
                        ui.add_space(4.0);

                        ui.horizontal(|ui| {
                            let scale = self.state.scale_factor();
                            if ui.selectable_label(scale == 2, "2x Upscale").clicked() {
                                self.state.set_scale_factor(2);
                            }
                            if ui.selectable_label(scale == 4, "4x Upscale").clicked() {
                                self.state.set_scale_factor(4);
                            }
                        });

                        ui.add_space(8.0);
                        ui.colored_label(
                            Theme::WARNING,
                            egui::RichText::new("Requires Phase 4 models")
                                .size(12.0)
                        );
                    }

                    Operation::Merge => {
                        ui.label(
                            egui::RichText::new("PDF Settings")
                                .size(14.0)
                                .color(Theme::TEXT_PRIMARY)
                        );
                        ui.add_space(8.0);

                        ui.label("Merge all images into a single PDF document");
                        ui.add_space(4.0);

                        ui.label(
                            egui::RichText::new("Output DPI: 300")
                                .size(12.0)
                                .color(Theme::TEXT_SECONDARY)
                        );
                    }
                }
            });
    }

    fn render_drop_zone(&mut self, ui: &mut egui::Ui) {
        // File drop zone
        let response = widgets::drop_zone(ui, false);

        if response.clicked() {
            // Open file picker
            if let Some(paths) = rfd::FileDialog::new()
                .add_filter("Images", &["png", "jpg", "jpeg", "webp", "tiff", "bmp"])
                .pick_files()
            {
                self.state.add_files(paths);
            }
        }

        // Handle drag-drop
        ui.ctx().input(|i| {
            if !i.raw.dropped_files.is_empty() {
                let paths: Vec<PathBuf> = i
                    .raw
                    .dropped_files
                    .iter()
                    .filter_map(|f| f.path.clone())
                    .collect();
                self.state.add_files(paths);
            }
        });
    }

    fn render_file_list(&mut self, ui: &mut egui::Ui) {
        let files = self.state.input_files();

        if files.is_empty() {
            // Empty state with better vertical centering
            ui.add_space(40.0);
            ui.vertical_centered(|ui| {
                ui.label(
                    egui::RichText::new("No files selected")
                        .size(14.0)
                        .color(Theme::TEXT_SECONDARY)
                );
            });
            return;
        }

        // File list with proper spacing (ScrollArea handled by parent)
        for (idx, file) in files.iter().enumerate() {
            // Each file entry in a frame with consistent padding
            egui::Frame::none()
                .inner_margin(egui::Margin::symmetric(8.0, 6.0))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        // Status icon with fixed width for alignment
                        let icon = Theme::status_icon(&file.status);
                        let color = Theme::status_color(&file.status);
                        ui.colored_label(color, egui::RichText::new(icon).size(16.0));

                        ui.add_space(8.0);

                        // Filename with consistent styling
                        ui.label(
                            egui::RichText::new(
                                file.path
                                    .file_name()
                                    .and_then(|n| n.to_str())
                                    .unwrap_or("Unknown")
                            )
                            .size(13.0)
                        );

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            // Remove button for pending files
                            if matches!(file.status, FileStatus::Pending) {
                                if ui.small_button("Remove").clicked() {
                                    self.state.remove_file(idx);
                                }
                            }

                            // Error message display
                            if let Some(error) = &file.error_message {
                                ui.add_space(8.0);
                                ui.colored_label(
                                    Theme::ERROR,
                                    egui::RichText::new(error).size(12.0)
                                );
                            }
                        });
                    });
                });

            // Separator between items (except for last item)
            if idx < files.len() - 1 {
                ui.add_space(2.0);
                ui.separator();
                ui.add_space(2.0);
            }
        }
    }

    fn render_output_settings(&mut self, ui: &mut egui::Ui) {
        // Output directory section with clear labeling
        ui.label(
            egui::RichText::new("Output Directory")
                .size(14.0)
                .color(Theme::TEXT_PRIMARY)
        );
        ui.add_space(6.0);

        // Output path display with frame
        egui::Frame::none()
            .inner_margin(egui::Margin::symmetric(8.0, 6.0))
            .stroke(egui::Stroke::new(1.0, Theme::BG_HOVER))
            .rounding(egui::Rounding::same(4.0))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    // Clone to avoid borrowing issues
                    let output_path_owned = self.state.output_dir().clone();
                    let output_text = output_path_owned
                        .as_ref()
                        .and_then(|p| p.to_str())
                        .unwrap_or("Default: ~/Downloads/transmute");

                    ui.label(
                        egui::RichText::new(output_text)
                            .size(12.0)
                            .color(Theme::TEXT_SECONDARY)
                    );
                });
            });

        ui.add_space(6.0);

        // Action buttons with consistent spacing
        ui.horizontal(|ui| {
            if ui.button("Browse").clicked() {
                if let Some(dir) = rfd::FileDialog::new().pick_folder() {
                    self.state.set_output_dir(Some(dir));
                }
            }

            if self.state.output_dir().is_some() {
                if ui.button("Reset to Default").clicked() {
                    self.state.set_output_dir(None);
                }
            }
        });
    }

    fn render_action_buttons(&mut self, ui: &mut egui::Ui) {
        let files = self.state.input_files();
        let can_process =
            !files.is_empty() && matches!(self.state.processing_state(), ProcessingState::Idle);

        ui.horizontal(|ui| {
            // Primary action button with prominent styling
            let process_text = match self.state.operation() {
                Operation::Convert => "Convert Files",
                Operation::Compress => "Compress Files",
                Operation::Enhance => "Enhance Files",
                Operation::Merge => "Merge to PDF",
            };

            // Primary button with consistent sizing
            let button = egui::Button::new(
                egui::RichText::new(process_text)
                    .size(14.0)
            )
            .fill(if can_process { Theme::PRIMARY } else { Theme::BG_HOVER })
            .min_size(egui::Vec2::new(140.0, 32.0));

            if ui.add_enabled(can_process, button).clicked() {
                self.start_processing();
            }

            ui.add_space(8.0);

            // Secondary clear button
            let clear_button = egui::Button::new(
                egui::RichText::new("Clear All")
                    .size(14.0)
            )
            .min_size(egui::Vec2::new(100.0, 32.0));

            if ui.add_enabled(!files.is_empty(), clear_button).clicked() {
                self.state.clear_files();
            }
        });
    }

    fn render_progress(&mut self, ui: &mut egui::Ui) {
        match self.state.processing_state() {
            ProcessingState::Idle => {}

            ProcessingState::Running { current, total } => {
                // Progress bar in a dedicated frame
                egui::Frame::none()
                    .inner_margin(egui::Margin::symmetric(0.0, 8.0))
                    .show(ui, |ui| {
                        widgets::progress_bar(ui, current, total);
                    });
            }

            ProcessingState::Complete { success, failed } => {
                // Completion status with clear visual feedback
                egui::Frame::none()
                    .inner_margin(egui::Margin::symmetric(8.0, 8.0))
                    .fill(Theme::BG_PANEL)
                    .rounding(egui::Rounding::same(4.0))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            if success > 0 {
                                ui.colored_label(
                                    Theme::SUCCESS,
                                    egui::RichText::new(format!("{} succeeded", success))
                                        .size(13.0)
                                );
                            }

                            if failed > 0 {
                                ui.add_space(12.0);
                                ui.colored_label(
                                    Theme::ERROR,
                                    egui::RichText::new(format!("{} failed", failed))
                                        .size(13.0)
                                );
                            }
                        });
                    });
            }
        }
    }

    fn render_natural_language(&mut self, ui: &mut egui::Ui) {
        // Natural language section with proper frame
        egui::Frame::group(ui.style())
            .inner_margin(egui::Margin::same(12.0))
            .show(ui, |ui| {
                ui.label(
                    egui::RichText::new("Natural Language")
                        .size(14.0)
                        .color(Theme::TEXT_PRIMARY)
                );
                ui.add_space(6.0);

                // Text input with full width
                let mut command = self.state.nl_command();
                let text_edit = egui::TextEdit::singleline(&mut command)
                    .hint_text("e.g., 'convert image.jpg to png'")
                    .desired_width(f32::INFINITY);

                if ui.add(text_edit).changed() {
                    self.state.set_nl_command(command);
                }

                ui.add_space(8.0);

                // Execute button with clear action
                ui.horizontal(|ui| {
                    if ui
                        .button(
                            egui::RichText::new("Execute Command")
                                .size(13.0)
                        )
                        .clicked()
                    {
                        self.execute_nl_command();
                    }

                    ui.add_space(8.0);

                    // Help text
                    ui.label(
                        egui::RichText::new("Experimental feature")
                            .size(11.0)
                            .color(Theme::TEXT_SECONDARY)
                    );
                });
            });
    }

    fn render_settings_panel(&mut self, ctx: &egui::Context) {
        let mut close_window = false;
        let mut save_settings = false;
        let mut settings = self.state.settings();

        egui::Window::new("Settings")
            .open(&mut self.show_settings)
            .default_width(400.0)
            .resizable(false)
            .show(ctx, |ui| {
                // Settings content with proper spacing
                ui.add_space(8.0);

                ui.label(
                    egui::RichText::new("Application Settings")
                        .size(16.0)
                        .color(Theme::TEXT_PRIMARY)
                );

                ui.add_space(12.0);
                ui.separator();
                ui.add_space(12.0);

                // GPU acceleration setting
                ui.horizontal(|ui| {
                    ui.checkbox(&mut settings.use_gpu, "");
                    ui.label(
                        egui::RichText::new("Enable GPU Acceleration")
                            .size(14.0)
                    );
                });

                ui.add_space(4.0);
                ui.label(
                    egui::RichText::new("Use GPU for faster image processing")
                        .size(11.0)
                        .color(Theme::TEXT_SECONDARY)
                );

                ui.add_space(16.0);

                // Auto-open output setting
                ui.horizontal(|ui| {
                    ui.checkbox(&mut settings.auto_open_output, "");
                    ui.label(
                        egui::RichText::new("Auto-open Output Folder")
                            .size(14.0)
                    );
                });

                ui.add_space(4.0);
                ui.label(
                    egui::RichText::new("Open output folder after processing completes")
                        .size(11.0)
                        .color(Theme::TEXT_SECONDARY)
                );

                ui.add_space(20.0);
                ui.separator();
                ui.add_space(12.0);

                // Action buttons
                ui.horizontal(|ui| {
                    let save_button = egui::Button::new(
                        egui::RichText::new("Save Settings")
                            .size(14.0)
                    )
                    .fill(Theme::PRIMARY)
                    .min_size(egui::Vec2::new(120.0, 32.0));

                    if ui.add(save_button).clicked() {
                        save_settings = true;
                        close_window = true;
                    }

                    ui.add_space(8.0);

                    let cancel_button = egui::Button::new(
                        egui::RichText::new("Cancel")
                            .size(14.0)
                    )
                    .min_size(egui::Vec2::new(100.0, 32.0));

                    if ui.add(cancel_button).clicked() {
                        close_window = true;
                    }
                });
            });

        if save_settings {
            self.state.update_settings(|s| *s = settings);
        }
        if close_window {
            self.show_settings = false;
        }
    }

    fn start_processing(&mut self) {
        let state = self.state.clone();
        let converter = Arc::clone(&self.converter);
        let operation = state.operation();

        // Spawn background thread for processing (egui runs without tokio runtime)
        let handle = std::thread::spawn(move || {
            let files = state.input_files();
            let total = files.len();

            state.set_processing_state(ProcessingState::Running { current: 0, total });

            let mut success = 0;
            let mut failed = 0;

            // Special handling for merge operation - combine all images into single PDF
            if operation == Operation::Merge {
                // Mark all files as processing
                for idx in 0..files.len() {
                    state.update_file_status(idx, FileStatus::Processing, None, None);
                }

                // Collect all input paths
                let input_paths: Vec<PathBuf> = files.iter().map(|f| f.path.clone()).collect();

                // Generate output path
                let output_path = if let Some(dir) = state.output_dir() {
                    dir.join("merged.pdf")
                } else {
                    // Use default output directory from path manager
                    let path_manager = transmute_common::PathManager::new().unwrap();
                    let default_dir = path_manager.default_output_dir();
                    std::fs::create_dir_all(default_dir).ok();
                    default_dir.join("merged.pdf")
                };

                // Perform merge
                let result = converter.images_to_pdf(input_paths, output_path.clone(), None);

                match result {
                    Ok(pdf_path) => {
                        // Mark all files as complete with the same output path
                        for idx in 0..files.len() {
                            state.update_file_status(
                                idx,
                                FileStatus::Complete,
                                Some(pdf_path.clone()),
                                None,
                            );
                        }
                        success = files.len();
                    }
                    Err(e) => {
                        // Mark all files as failed with the same error
                        for idx in 0..files.len() {
                            state.update_file_status(
                                idx,
                                FileStatus::Failed,
                                None,
                                Some(e.to_string()),
                            );
                        }
                        failed = files.len();
                    }
                }

                state.set_processing_state(ProcessingState::Running {
                    current: total,
                    total,
                });
                state.set_processing_state(ProcessingState::Complete { success, failed });
                return;
            }

            // Individual file processing for other operations
            for (idx, file) in files.iter().enumerate() {
                state.update_file_status(idx, FileStatus::Processing, None, None);

                let result = match operation {
                    Operation::Convert => converter.convert_image(
                        &file.path,
                        state.target_format(),
                        state.output_dir(),
                    ),

                    Operation::Compress => converter
                        .compress_image(
                            &file.path,
                            state.target_format(),
                            state.quality(),
                            state.output_dir(),
                        )
                        .map(|(path, _)| path),

                    Operation::Enhance => Err(transmute_common::Error::ConversionError(
                        "Enhancement not implemented".into(),
                    )),

                    Operation::Merge => unreachable!("Merge handled above"),
                };

                match result {
                    Ok(output_path) => {
                        state.update_file_status(
                            idx,
                            FileStatus::Complete,
                            Some(output_path),
                            None,
                        );
                        success += 1;
                    }
                    Err(e) => {
                        state.update_file_status(
                            idx,
                            FileStatus::Failed,
                            None,
                            Some(e.to_string()),
                        );
                        failed += 1;
                    }
                }

                state.set_processing_state(ProcessingState::Running {
                    current: idx + 1,
                    total,
                });
            }

            state.set_processing_state(ProcessingState::Complete { success, failed });
        });

        self.processing_handle = Some(handle);
    }

    fn execute_nl_command(&mut self) {
        let command = self.state.nl_command();
        if command.is_empty() {
            return;
        }

        let converter = Arc::clone(&self.converter);
        let _state = self.state.clone();

        // Spawn background thread for NL command execution (egui runs without tokio runtime)
        std::thread::spawn(move || {
            match converter.execute_command(&command) {
                Ok(outputs) => {
                    tracing::info!("NL command succeeded: {} outputs", outputs.len());
                }
                Err(e) => {
                    tracing::error!("NL command failed: {}", e);
                }
            }
        });
    }
}

impl eframe::App for TransmuteApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Top bar with consistent styling
        TopBottomPanel::top("top_panel")
            .frame(
                egui::Frame::none()
                    .fill(Theme::BG_PANEL)
                    .inner_margin(egui::Margin::symmetric(12.0, 8.0))
            )
            .show(ctx, |ui| {
                self.render_top_bar(ctx, ui);
            });

        // Left side panel for settings with proper spacing
        SidePanel::left("settings_panel")
            .default_width(300.0)
            .resizable(false)
            .frame(
                egui::Frame::none()
                    .fill(Theme::BG_PANEL)
                    .inner_margin(egui::Margin::same(16.0))
            )
            .show(ctx, |ui| {
                // Settings section header
                ui.label(
                    egui::RichText::new("Configuration")
                        .size(16.0)
                        .strong()
                );
                ui.add_space(12.0);
                ui.separator();
                ui.add_space(16.0);

                // Operation selector
                self.render_operation_selector(ui);
                ui.add_space(16.0);

                // Operation-specific settings
                self.render_operation_settings(ui);
                ui.add_space(16.0);

                // Output directory settings
                self.render_output_settings(ui);
                ui.add_space(20.0);

                ui.separator();
                ui.add_space(16.0);

                // Natural language interface
                self.render_natural_language(ui);
            });

        // Bottom panel for fixed action buttons and progress
        TopBottomPanel::bottom("bottom_panel")
            .frame(
                egui::Frame::none()
                    .fill(Theme::BG_DARK)
                    .inner_margin(egui::Margin::same(20.0))
            )
            .show(ctx, |ui| {
                // Progress indicator in fixed-height area
                ui.allocate_ui_with_layout(
                    egui::Vec2::new(ui.available_width(), 50.0),
                    egui::Layout::top_down(egui::Align::LEFT),
                    |ui| {
                        self.render_progress(ui);
                    }
                );

                ui.add_space(12.0);

                // Action buttons fixed at bottom
                self.render_action_buttons(ui);
            });

        // Main content area with scrollable file list
        CentralPanel::default()
            .frame(
                egui::Frame::none()
                    .fill(Theme::BG_DARK)
                    .inner_margin(egui::Margin::same(20.0))
            )
            .show(ctx, |ui| {
                // Files section header
                ui.label(
                    egui::RichText::new("Files")
                        .size(18.0)
                        .strong()
                );
                ui.add_space(12.0);

                // Drop zone for file selection
                self.render_drop_zone(ui);
                ui.add_space(16.0);

                // File list with scrollable area
                egui::Frame::none()
                    .fill(Theme::BG_PANEL)
                    .rounding(egui::Rounding::same(6.0))
                    .inner_margin(egui::Margin::same(12.0))
                    .show(ui, |ui| {
                        // ScrollArea takes all remaining vertical space
                        ScrollArea::vertical()
                            .auto_shrink([false; 2])
                            .show(ui, |ui| {
                                self.render_file_list(ui);
                            });
                    });
            });

        // Settings modal overlay
        if self.show_settings {
            self.render_settings_panel(ctx);
        }

        // Request repaint during processing for smooth progress updates
        if matches!(
            self.state.processing_state(),
            ProcessingState::Running { .. }
        ) {
            ctx.request_repaint();
        }
    }
}
