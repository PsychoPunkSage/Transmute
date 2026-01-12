use egui::{Response, Sense, Ui, Vec2};

/// File drop zone widget with improved visual design
pub fn drop_zone(ui: &mut Ui, hovered: bool) -> Response {
    let desired_size = Vec2::new(ui.available_width(), 140.0);
    let (rect, response) = ui.allocate_exact_size(desired_size, Sense::click_and_drag());

    if ui.is_rect_visible(rect) {
        // Dynamic background based on interaction state
        let bg_color = if response.hovered() || hovered {
            crate::theme::Theme::PRIMARY.linear_multiply(0.15)
        } else {
            crate::theme::Theme::BG_PANEL
        };

        // Border styling with hover effect
        let stroke_color = if response.hovered() || hovered {
            crate::theme::Theme::PRIMARY
        } else {
            crate::theme::Theme::BG_HOVER
        };

        let stroke_width = if response.hovered() || hovered { 2.5 } else { 2.0 };

        // Draw background with rounded corners
        ui.painter().rect_filled(rect, 8.0, bg_color);

        // Draw dashed border for drop zone feel
        let dashes = 12;
        let dash_length = rect.width() / (dashes as f32 * 2.0);
        for i in 0..dashes {
            let start = rect.left_top() + Vec2::new(i as f32 * dash_length * 2.0, 0.0);
            let end = start + Vec2::new(dash_length, 0.0);
            ui.painter().line_segment(
                [start, end],
                egui::Stroke::new(stroke_width, stroke_color),
            );

            let start = rect.left_bottom() + Vec2::new(i as f32 * dash_length * 2.0, 0.0);
            let end = start + Vec2::new(dash_length, 0.0);
            ui.painter().line_segment(
                [start, end],
                egui::Stroke::new(stroke_width, stroke_color),
            );
        }

        let v_dashes = 8;
        let v_dash_length = rect.height() / (v_dashes as f32 * 2.0);
        for i in 0..v_dashes {
            let start = rect.left_top() + Vec2::new(0.0, i as f32 * v_dash_length * 2.0);
            let end = start + Vec2::new(0.0, v_dash_length);
            ui.painter().line_segment(
                [start, end],
                egui::Stroke::new(stroke_width, stroke_color),
            );

            let start = rect.right_top() + Vec2::new(0.0, i as f32 * v_dash_length * 2.0);
            let end = start + Vec2::new(0.0, v_dash_length);
            ui.painter().line_segment(
                [start, end],
                egui::Stroke::new(stroke_width, stroke_color),
            );
        }

        // Text content
        let text = if hovered || response.hovered() {
            "Drop files here"
        } else {
            "Drag & drop files or click to browse"
        };

        let text_color = if response.hovered() || hovered {
            crate::theme::Theme::PRIMARY
        } else {
            crate::theme::Theme::TEXT_SECONDARY
        };

        ui.painter().text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            text,
            egui::FontId::proportional(15.0),
            text_color,
        );

        // Hint text below
        if !hovered && !response.hovered() {
            let hint = "Supported: PNG, JPEG, WebP, TIFF, BMP";
            ui.painter().text(
                rect.center() + Vec2::new(0.0, 25.0),
                egui::Align2::CENTER_CENTER,
                hint,
                egui::FontId::proportional(11.0),
                crate::theme::Theme::TEXT_SECONDARY.linear_multiply(0.7),
            );
        }
    }

    response
}

/// Format selector combo box
pub fn format_selector(ui: &mut Ui, selected: &mut transmute_common::MediaFormat) -> bool {
    let formats = [
        transmute_common::MediaFormat::Png,
        transmute_common::MediaFormat::Jpeg,
        transmute_common::MediaFormat::Webp,
        transmute_common::MediaFormat::Tiff,
        transmute_common::MediaFormat::Bmp,
        transmute_common::MediaFormat::Pdf,
    ];

    let mut changed = false;

    egui::ComboBox::from_label("Format")
        .selected_text(selected.to_string())
        .show_ui(ui, |ui| {
            for format in formats {
                if ui
                    .selectable_value(selected, format, format.to_string())
                    .clicked()
                {
                    changed = true;
                }
            }
        });

    changed
}

/// Quality selector with presets and slider
pub fn quality_selector(ui: &mut Ui, quality: &mut transmute_compress::QualitySettings) -> bool {
    let mut changed = false;

    ui.label("Quality Level");
    ui.add_space(6.0);

    let mut quality_value = match quality {
        transmute_compress::QualitySettings::Custom(v) => *v,
        transmute_compress::QualitySettings::Maximum => 100,
        transmute_compress::QualitySettings::High => 95,
        transmute_compress::QualitySettings::Balanced => 85,
        transmute_compress::QualitySettings::Low => 75,
    };

    // Preset buttons with better spacing
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 6.0;

        let is_low = matches!(quality, transmute_compress::QualitySettings::Low);
        let is_balanced = matches!(quality, transmute_compress::QualitySettings::Balanced);
        let is_high = matches!(quality, transmute_compress::QualitySettings::High);
        let is_max = matches!(quality, transmute_compress::QualitySettings::Maximum);

        if ui.selectable_label(is_low, "Low (75%)").clicked() {
            *quality = transmute_compress::QualitySettings::Low;
            changed = true;
        }

        if ui.selectable_label(is_balanced, "Balanced (85%)").clicked() {
            *quality = transmute_compress::QualitySettings::Balanced;
            changed = true;
        }

        if ui.selectable_label(is_high, "High (95%)").clicked() {
            *quality = transmute_compress::QualitySettings::High;
            changed = true;
        }

        if ui.selectable_label(is_max, "Maximum (100%)").clicked() {
            *quality = transmute_compress::QualitySettings::Maximum;
            changed = true;
        }
    });

    ui.add_space(8.0);

    // Custom slider for fine-tuning
    ui.label(
        egui::RichText::new("Custom")
            .size(12.0)
            .color(crate::theme::Theme::TEXT_SECONDARY)
    );
    ui.add_space(4.0);

    if ui
        .add(
            egui::Slider::new(&mut quality_value, 1..=100)
                .suffix("%")
                .show_value(true)
        )
        .changed()
    {
        *quality = transmute_compress::QualitySettings::Custom(quality_value);
        changed = true;
    }

    changed
}

/// Progress bar with detailed status
pub fn progress_bar(ui: &mut Ui, current: usize, total: usize) {
    if total == 0 {
        return;
    }

    let progress = current as f32 / total as f32;
    let percentage = (progress * 100.0) as u32;

    // Progress label above bar
    ui.horizontal(|ui| {
        ui.label(
            egui::RichText::new(format!("Processing: {} of {}", current, total))
                .size(13.0)
                .color(crate::theme::Theme::TEXT_PRIMARY)
        );

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(
                egui::RichText::new(format!("{}%", percentage))
                    .size(13.0)
                    .color(crate::theme::Theme::PRIMARY)
            );
        });
    });

    ui.add_space(6.0);

    // Progress bar with full width
    let progress_bar = egui::ProgressBar::new(progress)
        .animate(true)
        .desired_width(ui.available_width())
        .desired_height(8.0);

    ui.add(progress_bar);
}
