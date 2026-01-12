use egui::{Color32, Rounding, Style, Vec2};

/// Application theme colors
pub struct Theme;

impl Theme {
    // Primary colors
    pub const PRIMARY: Color32 = Color32::from_rgb(99, 102, 241); // Indigo
    pub const PRIMARY_HOVER: Color32 = Color32::from_rgb(129, 140, 248);
    pub const SUCCESS: Color32 = Color32::from_rgb(34, 197, 94); // Green
    pub const ERROR: Color32 = Color32::from_rgb(239, 68, 68); // Red
    pub const WARNING: Color32 = Color32::from_rgb(251, 191, 36); // Yellow
    pub const INFO: Color32 = Color32::from_rgb(59, 130, 246); // Blue

    // Backgrounds
    pub const BG_DARK: Color32 = Color32::from_rgb(17, 24, 39); // Gray-900
    pub const BG_PANEL: Color32 = Color32::from_rgb(31, 41, 55); // Gray-800
    pub const BG_HOVER: Color32 = Color32::from_rgb(55, 65, 81); // Gray-700

    // Text
    pub const TEXT_PRIMARY: Color32 = Color32::from_rgb(243, 244, 246); // Gray-100
    pub const TEXT_SECONDARY: Color32 = Color32::from_rgb(156, 163, 175); // Gray-400

    /// Configure egui style with clean, modern appearance
    pub fn configure(ctx: &egui::Context) {
        let mut style = Style::default();

        // Consistent spacing throughout the application
        style.spacing.item_spacing = Vec2::new(8.0, 8.0);
        style.spacing.button_padding = Vec2::new(16.0, 8.0);
        style.spacing.window_margin = egui::Margin::same(16.0);
        style.spacing.indent = 16.0;
        style.spacing.slider_width = 180.0;
        style.spacing.combo_width = 120.0;

        // Smooth, modern rounding
        let rounding = Rounding::same(6.0);
        style.visuals.widgets.noninteractive.rounding = rounding;
        style.visuals.widgets.inactive.rounding = rounding;
        style.visuals.widgets.hovered.rounding = rounding;
        style.visuals.widgets.active.rounding = rounding;
        style.visuals.window_rounding = Rounding::same(8.0);
        style.visuals.menu_rounding = Rounding::same(6.0);

        // Dark mode color scheme
        style.visuals.dark_mode = true;
        style.visuals.override_text_color = Some(Self::TEXT_PRIMARY);

        // Panel backgrounds
        style.visuals.panel_fill = Self::BG_PANEL;
        style.visuals.extreme_bg_color = Self::BG_DARK;

        // Widget backgrounds
        style.visuals.widgets.noninteractive.bg_fill = Self::BG_PANEL;
        style.visuals.widgets.noninteractive.weak_bg_fill = Self::BG_PANEL;

        style.visuals.widgets.inactive.bg_fill = Self::BG_PANEL;
        style.visuals.widgets.inactive.weak_bg_fill = Self::BG_PANEL;

        style.visuals.widgets.hovered.bg_fill = Self::BG_HOVER;
        style.visuals.widgets.hovered.weak_bg_fill = Self::BG_HOVER;

        style.visuals.widgets.active.bg_fill = Self::PRIMARY;
        style.visuals.widgets.active.weak_bg_fill = Self::PRIMARY;

        // Stroke/border colors for clean separation
        style.visuals.widgets.noninteractive.bg_stroke =
            egui::Stroke::new(1.0, Self::BG_HOVER);
        style.visuals.widgets.inactive.bg_stroke =
            egui::Stroke::new(1.0, Self::BG_HOVER);
        style.visuals.widgets.hovered.bg_stroke =
            egui::Stroke::new(1.5, Self::PRIMARY);
        style.visuals.widgets.active.bg_stroke =
            egui::Stroke::new(2.0, Self::PRIMARY);

        // Selection color
        style.visuals.selection.bg_fill = Self::PRIMARY.linear_multiply(0.3);
        style.visuals.selection.stroke = egui::Stroke::new(1.0, Self::PRIMARY);

        // Window styling
        style.visuals.window_fill = Self::BG_PANEL;
        style.visuals.window_stroke = egui::Stroke::new(1.0, Self::BG_HOVER);
        style.visuals.window_shadow = egui::epaint::Shadow {
            offset: Vec2::new(0.0, 4.0),
            blur: 16.0,
            spread: 0.0,
            color: Color32::from_black_alpha(100),
        };

        ctx.set_style(style);
    }

    /// Status color for file processing
    pub fn status_color(status: &crate::state::FileStatus) -> Color32 {
        match status {
            crate::state::FileStatus::Pending => Self::TEXT_SECONDARY,
            crate::state::FileStatus::Processing => Self::INFO,
            crate::state::FileStatus::Complete => Self::SUCCESS,
            crate::state::FileStatus::Failed => Self::ERROR,
        }
    }

    /// Status icon for file processing
    pub fn status_icon(status: &crate::state::FileStatus) -> &'static str {
        match status {
            crate::state::FileStatus::Pending => "⏸",
            crate::state::FileStatus::Processing => "⏳",
            crate::state::FileStatus::Complete => "✓",
            crate::state::FileStatus::Failed => "✗",
        }
    }
}
