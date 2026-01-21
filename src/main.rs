#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use eframe::egui;
use image::DynamicImage;

#[derive(Clone, Copy, Debug, PartialEq)]
enum ResizeHandle {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
    Top,
    Bottom,
    Left,
    Right,
    Center, // Moving
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum AspectRatioMode {
    Free,
    Original,
    Square,
    // Landscape
    R3_2,
    R4_3,
    R16_9,
    R16_10,
    // Portrait
    R2_3,
    R3_4,
    R9_16,
    R10_16,
    Custom,
}

impl Default for AspectRatioMode {
    fn default() -> Self {
        Self::Free
    }
}

impl AspectRatioMode {
    fn counterpart(&self) -> Self {
        match self {
            AspectRatioMode::R3_2 => AspectRatioMode::R2_3,
            AspectRatioMode::R4_3 => AspectRatioMode::R3_4,
            AspectRatioMode::R16_9 => AspectRatioMode::R9_16,
            AspectRatioMode::R16_10 => AspectRatioMode::R10_16,
            AspectRatioMode::R2_3 => AspectRatioMode::R3_2,
            AspectRatioMode::R3_4 => AspectRatioMode::R4_3,
            AspectRatioMode::R9_16 => AspectRatioMode::R16_9,
            AspectRatioMode::R10_16 => AspectRatioMode::R16_10,
            _ => self.clone(),
        }
    }
}

#[derive(Default)]
struct ImageCropper {
    image: Option<DynamicImage>,
    texture: Option<egui::TextureHandle>,
    crop_rect: Option<egui::Rect>, // Normalized coordinates (0.0-1.0)
    selected_handle: Option<ResizeHandle>,
    aspect_ratio_mode: AspectRatioMode,
    custom_w: u32,
    custom_h: u32,
    is_portrait: bool,
}

impl ImageCropper {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self {
            custom_w: 4,
            custom_h: 3,
            is_portrait: false,
            ..Default::default()
        }
    }

    fn load_texture(&mut self, ctx: &egui::Context) {
        if let Some(image) = &self.image {
            let size = [image.width() as _, image.height() as _];
            let image_buffer = image.to_rgba8();
            let pixels = image_buffer.as_flat_samples();
            let color_image = egui::ColorImage::from_rgba_unmultiplied(size, pixels.as_slice());
            self.texture =
                Some(ctx.load_texture("image", color_image, egui::TextureOptions::LINEAR));
            // Initialize crop rect to full image
            self.crop_rect = Some(egui::Rect::from_min_max(
                egui::Pos2::new(0.0, 0.0),
                egui::Pos2::new(1.0, 1.0),
            ));
        }
    }

    fn apply_aspect_ratio(&mut self) {
        if let (Some(image), Some(crop_rect)) = (&self.image, &mut self.crop_rect) {
            let image_size = egui::vec2(image.width() as f32, image.height() as f32);
            let target_ratio = match self.aspect_ratio_mode {
                AspectRatioMode::Free => None,
                AspectRatioMode::Original => Some(image_size.x / image_size.y),
                AspectRatioMode::Square => Some(1.0),
                AspectRatioMode::R3_2 => Some(3.0 / 2.0),
                AspectRatioMode::R4_3 => Some(4.0 / 3.0),
                AspectRatioMode::R16_9 => Some(16.0 / 9.0),
                AspectRatioMode::R16_10 => Some(16.0 / 10.0),
                AspectRatioMode::R2_3 => Some(2.0 / 3.0),
                AspectRatioMode::R3_4 => Some(3.0 / 4.0),
                AspectRatioMode::R9_16 => Some(9.0 / 16.0),
                AspectRatioMode::R10_16 => Some(10.0 / 16.0),
                AspectRatioMode::Custom => Some(self.custom_w as f32 / self.custom_h as f32),
            };

            if let Some(ratio) = target_ratio {
                // Calculate normalized target aspect ratio
                let norm_aspect = ratio * (image_size.y / image_size.x);
                let current_center = crop_rect.center();
                let current_w = crop_rect.width();
                let current_h = crop_rect.height();

                // Preserve major dimension logic
                let max_dim = current_w.max(current_h);

                let (mut new_w, mut new_h) = if norm_aspect >= 1.0 {
                    (max_dim, max_dim / norm_aspect)
                } else {
                    (max_dim * norm_aspect, max_dim)
                };

                // Fit to bounds if necessary
                if new_w > 1.0 {
                    new_w = 1.0;
                    new_h = new_w / norm_aspect;
                }
                if new_h > 1.0 {
                    new_h = 1.0;
                    new_w = new_h * norm_aspect;
                }

                *crop_rect = egui::Rect::from_center_size(current_center, egui::vec2(new_w, new_h));

                // Ensure it stays within 0.0-1.0 bounds logic
                if crop_rect.min.x < 0.0 {
                    *crop_rect = crop_rect.translate(egui::vec2(-crop_rect.min.x, 0.0));
                }
                if crop_rect.min.y < 0.0 {
                    *crop_rect = crop_rect.translate(egui::vec2(0.0, -crop_rect.min.y));
                }
                if crop_rect.max.x > 1.0 {
                    *crop_rect = crop_rect.translate(egui::vec2(1.0 - crop_rect.max.x, 0.0));
                }
                if crop_rect.max.y > 1.0 {
                    *crop_rect = crop_rect.translate(egui::vec2(0.0, 1.0 - crop_rect.max.y));
                }

                // Hard clamp if still out (e.g. too big)
                crop_rect.min = crop_rect
                    .min
                    .clamp(egui::Pos2::ZERO, egui::Pos2::new(1.0, 1.0));
                crop_rect.max = crop_rect
                    .max
                    .clamp(egui::Pos2::ZERO, egui::Pos2::new(1.0, 1.0));
            }
        }
    }

    fn hit_test(pos: egui::Pos2, rect: egui::Rect) -> Option<ResizeHandle> {
        let tolerance = 10.0;

        let min = rect.min;
        let max = rect.max;

        if pos.distance(min) < tolerance {
            return Some(ResizeHandle::TopLeft);
        }
        if pos.distance(egui::pos2(max.x, min.y)) < tolerance {
            return Some(ResizeHandle::TopRight);
        }
        if pos.distance(egui::pos2(min.x, max.y)) < tolerance {
            return Some(ResizeHandle::BottomLeft);
        }
        if pos.distance(max) < tolerance {
            return Some(ResizeHandle::BottomRight);
        }

        if (pos.x - min.x).abs() < tolerance && pos.y > min.y && pos.y < max.y {
            return Some(ResizeHandle::Left);
        }
        if (pos.x - max.x).abs() < tolerance && pos.y > min.y && pos.y < max.y {
            return Some(ResizeHandle::Right);
        }
        if (pos.y - min.y).abs() < tolerance && pos.x > min.x && pos.x < max.x {
            return Some(ResizeHandle::Top);
        }
        if (pos.y - max.y).abs() < tolerance && pos.x > min.x && pos.x < max.x {
            return Some(ResizeHandle::Bottom);
        }

        if rect.contains(pos) {
            return Some(ResizeHandle::Center);
        }

        None
    }
}

impl std::fmt::Display for AspectRatioMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            AspectRatioMode::Free => "Free",
            AspectRatioMode::Original => "Original",
            AspectRatioMode::Square => "1:1",
            AspectRatioMode::R3_2 => "3:2",
            AspectRatioMode::R4_3 => "4:3",
            AspectRatioMode::R16_9 => "16:9",
            AspectRatioMode::R16_10 => "16:10",
            AspectRatioMode::R2_3 => "2:3",
            AspectRatioMode::R3_4 => "3:4",
            AspectRatioMode::R9_16 => "9:16",
            AspectRatioMode::R10_16 => "10:16",
            AspectRatioMode::Custom => "Custom",
        };
        write!(f, "{}", s)
    }
}

impl eframe::App for ImageCropper {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Handle dropped files
        if !ctx.input(|i| i.raw.dropped_files.is_empty()) {
            let dropped_files = ctx.input(|i| i.raw.dropped_files.clone());
            if let Some(file) = dropped_files.first() {
                if let Some(path) = &file.path {
                    if let Ok(img) = image::open(path) {
                        self.image = Some(img);
                        self.load_texture(ctx);
                        self.selected_handle = None;
                    }
                }
            }
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            if ui.button("Open Image").clicked() {
                if let Some(path) = rfd::FileDialog::new()
                    .add_filter("Image", &["png", "jpg", "jpeg", "bmp"])
                    .pick_file()
                {
                    if let Ok(img) = image::open(&path) {
                        self.image = Some(img);
                        self.load_texture(ctx);
                        self.selected_handle = None;
                    }
                }
            }

            if self.texture.is_some() {
                ui.horizontal(|ui| {
                    ui.label("Aspect Ratio:");
                    let mut changed = false;
                    egui::ComboBox::from_id_salt("params_aspect_ratio")
                        .selected_text(format!("{}", self.aspect_ratio_mode))
                        .show_ui(ui, |ui| {
                            changed |= ui
                                .selectable_value(
                                    &mut self.aspect_ratio_mode,
                                    AspectRatioMode::Free,
                                    "Free",
                                )
                                .changed();
                            changed |= ui
                                .selectable_value(
                                    &mut self.aspect_ratio_mode,
                                    AspectRatioMode::Original,
                                    "Original",
                                )
                                .changed();
                            changed |= ui
                                .selectable_value(
                                    &mut self.aspect_ratio_mode,
                                    AspectRatioMode::Square,
                                    "1:1",
                                )
                                .changed();

                            ui.separator();
                            if !self.is_portrait {
                                changed |= ui
                                    .selectable_value(
                                        &mut self.aspect_ratio_mode,
                                        AspectRatioMode::R3_2,
                                        "3:2",
                                    )
                                    .changed();
                                changed |= ui
                                    .selectable_value(
                                        &mut self.aspect_ratio_mode,
                                        AspectRatioMode::R4_3,
                                        "4:3",
                                    )
                                    .changed();
                                changed |= ui
                                    .selectable_value(
                                        &mut self.aspect_ratio_mode,
                                        AspectRatioMode::R16_9,
                                        "16:9",
                                    )
                                    .changed();
                                changed |= ui
                                    .selectable_value(
                                        &mut self.aspect_ratio_mode,
                                        AspectRatioMode::R16_10,
                                        "16:10",
                                    )
                                    .changed();
                            } else {
                                changed |= ui
                                    .selectable_value(
                                        &mut self.aspect_ratio_mode,
                                        AspectRatioMode::R2_3,
                                        "2:3",
                                    )
                                    .changed();
                                changed |= ui
                                    .selectable_value(
                                        &mut self.aspect_ratio_mode,
                                        AspectRatioMode::R3_4,
                                        "3:4",
                                    )
                                    .changed();
                                changed |= ui
                                    .selectable_value(
                                        &mut self.aspect_ratio_mode,
                                        AspectRatioMode::R9_16,
                                        "9:16",
                                    )
                                    .changed();
                                changed |= ui
                                    .selectable_value(
                                        &mut self.aspect_ratio_mode,
                                        AspectRatioMode::R10_16,
                                        "10:16",
                                    )
                                    .changed();
                            }

                            ui.separator();
                            changed |= ui
                                .selectable_value(
                                    &mut self.aspect_ratio_mode,
                                    AspectRatioMode::Custom,
                                    "Custom",
                                )
                                .changed();
                        });

                    if ui.button("🔄").clicked() {
                        self.is_portrait = !self.is_portrait;
                        if self.aspect_ratio_mode == AspectRatioMode::Custom {
                            std::mem::swap(&mut self.custom_w, &mut self.custom_h);
                        } else {
                            self.aspect_ratio_mode = self.aspect_ratio_mode.counterpart();
                        }
                        changed = true;
                    }

                    if self.aspect_ratio_mode == AspectRatioMode::Custom {
                        changed |= ui
                            .add(
                                egui::DragValue::new(&mut self.custom_w)
                                    .speed(0.1)
                                    .range(1..=100),
                            )
                            .changed();
                        ui.label(":");
                        changed |= ui
                            .add(
                                egui::DragValue::new(&mut self.custom_h)
                                    .speed(0.1)
                                    .range(1..=100),
                            )
                            .changed();
                    }

                    if changed {
                        self.apply_aspect_ratio();
                    }

                    if ui.button("Save Cropped Image").clicked() {
                        if let (Some(image), Some(crop_rect)) = (&self.image, self.crop_rect) {
                            if let Some(path) = rfd::FileDialog::new()
                                .add_filter("Image", &["png", "jpg", "jpeg", "bmp"])
                                .save_file()
                            {
                                let w = image.width() as f32;
                                let h = image.height() as f32;

                                let x = (crop_rect.min.x * w).max(0.0) as u32;
                                let y = (crop_rect.min.y * h).max(0.0) as u32;
                                let width = (crop_rect.width() * w).max(1.0) as u32;
                                let height = (crop_rect.height() * h).max(1.0) as u32;

                                // Ensure bounds
                                let x = x.min(image.width() - 1);
                                let y = y.min(image.height() - 1);
                                let width = width.min(image.width() - x);
                                let height = height.min(image.height() - y);

                                let cropped = image.crop_imm(x, y, width, height);
                                if let Err(e) = cropped.save(path) {
                                    eprintln!("Failed to save image: {}", e);
                                }
                            }
                        }
                    }
                });

                ui.separator();
            }

            if let (Some(texture), Some(crop_rect)) = (&self.texture, &mut self.crop_rect) {
                const PADDING: f32 = 20.0;
                let available_size = ui.available_size();
                let max_size = available_size - egui::vec2(PADDING * 2.0, PADDING * 2.0);
                let image_size = texture.size_vec2();

                // Calculate size to fit within available space while maintaining aspect ratio
                let scale = (max_size.x / image_size.x).min(max_size.y / image_size.y);
                let display_size = image_size * scale;

                let total_display_size = display_size + egui::vec2(PADDING * 2.0, PADDING * 2.0);

                // Manual centering
                let x_offset = (available_size.x - total_display_size.x) / 2.0;
                let y_offset = (available_size.y - total_display_size.y) / 2.0;
                let start_pos = ui.cursor().min + egui::vec2(x_offset.max(0.0), y_offset.max(0.0));

                let target_rect = egui::Rect::from_min_size(start_pos, total_display_size);

                let response = ui.allocate_rect(target_rect, egui::Sense::drag());
                let painter = ui.painter_at(target_rect);

                // Center the image rect within the response rect (which includes padding)
                let image_rect = egui::Rect::from_min_size(
                    target_rect.min + egui::vec2(PADDING, PADDING),
                    display_size,
                );

                // Draw image
                painter.image(
                    texture.id(),
                    image_rect,
                    egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                    egui::Color32::WHITE,
                );

                // Convert normalized crop rect to screen coordinates
                let mut screen_crop_rect = egui::Rect::from_min_max(
                    image_rect.lerp_inside(crop_rect.min.to_vec2()),
                    image_rect.lerp_inside(crop_rect.max.to_vec2()),
                );

                // Handle Input
                if response.drag_started() {
                    if let Some(pos) = response.interact_pointer_pos() {
                        self.selected_handle = Self::hit_test(pos, screen_crop_rect);
                    }
                }

                if response.dragged() {
                    if let Some(handle) = self.selected_handle {
                        let delta = response.drag_delta();
                        let delta_norm = delta / display_size; // Normalize delta

                        // Determine target aspect ratio
                        let target_ratio = match self.aspect_ratio_mode {
                            AspectRatioMode::Free => None,
                            AspectRatioMode::Original => Some(image_size.x / image_size.y),
                            AspectRatioMode::Square => Some(1.0),
                            AspectRatioMode::R3_2 => Some(3.0 / 2.0),
                            AspectRatioMode::R4_3 => Some(4.0 / 3.0),
                            AspectRatioMode::R16_9 => Some(16.0 / 9.0),
                            AspectRatioMode::R16_10 => Some(16.0 / 10.0),
                            AspectRatioMode::R2_3 => Some(2.0 / 3.0),
                            AspectRatioMode::R3_4 => Some(3.0 / 4.0),
                            AspectRatioMode::R9_16 => Some(9.0 / 16.0),
                            AspectRatioMode::R10_16 => Some(10.0 / 16.0),
                            AspectRatioMode::Custom => {
                                Some(self.custom_w as f32 / self.custom_h as f32)
                            }
                        };

                        let norm_aspect = target_ratio.map(|r| r * (image_size.y / image_size.x));

                        if let (Some(ratio), Some(norm_aspect)) = (target_ratio, norm_aspect) {
                            // Constrained resize
                            // Helper to convert normalized width/height to screen space
                            let to_screen = |w_norm: f32, h_norm: f32| -> egui::Vec2 {
                                egui::vec2(w_norm * display_size.x, h_norm * display_size.y)
                            };
                            // Helper to convert screen dimensions back to normalized space dimensions
                            let to_norm = |w_screen: f32, h_screen: f32| -> egui::Vec2 {
                                egui::vec2(w_screen / display_size.x, h_screen / display_size.y)
                            };

                            match handle {
                                ResizeHandle::Center => {
                                    // Safe Panning: constrain delta to stay within bounds
                                    let mut final_delta = delta_norm;
                                    if crop_rect.min.x + final_delta.x < 0.0 {
                                        final_delta.x = -crop_rect.min.x;
                                    }
                                    if crop_rect.max.x + final_delta.x > 1.0 {
                                        final_delta.x = 1.0 - crop_rect.max.x;
                                    }
                                    if crop_rect.min.y + final_delta.y < 0.0 {
                                        final_delta.y = -crop_rect.min.y;
                                    }
                                    if crop_rect.max.y + final_delta.y > 1.0 {
                                        final_delta.y = 1.0 - crop_rect.max.y;
                                    }

                                    *crop_rect = crop_rect.translate(final_delta);
                                }
                                // Corner Handles: Use projection logic for smooth interactions
                                ResizeHandle::TopLeft
                                | ResizeHandle::TopRight
                                | ResizeHandle::BottomLeft
                                | ResizeHandle::BottomRight => {
                                    // 1. Identify Anchor (Fixed Point) and current Corner
                                    let (anchor, mut corner) = match handle {
                                        ResizeHandle::TopLeft => (crop_rect.max, crop_rect.min),
                                        ResizeHandle::TopRight => (
                                            egui::pos2(crop_rect.min.x, crop_rect.max.y),
                                            egui::pos2(crop_rect.max.x, crop_rect.min.y),
                                        ),
                                        ResizeHandle::BottomLeft => (
                                            egui::pos2(crop_rect.max.x, crop_rect.min.y),
                                            egui::pos2(crop_rect.min.x, crop_rect.max.y),
                                        ),
                                        ResizeHandle::BottomRight => (crop_rect.min, crop_rect.max),
                                        _ => (egui::Pos2::ZERO, egui::Pos2::ZERO), // Unreachable
                                    };

                                    // 2. Calculate suggested new dimensions in screen space
                                    // Apply delta to corner
                                    match handle {
                                        ResizeHandle::TopLeft => corner += delta_norm,
                                        ResizeHandle::TopRight => {
                                            corner.y += delta_norm.y;
                                            corner.x += delta_norm.x;
                                        }
                                        ResizeHandle::BottomLeft => {
                                            corner.x += delta_norm.x;
                                            corner.y += delta_norm.y;
                                        }
                                        ResizeHandle::BottomRight => corner += delta_norm,
                                        _ => {}
                                    }

                                    // Calculate raw new width/height (absolute)
                                    let raw_w_norm = (corner.x - anchor.x).abs();
                                    let raw_h_norm = (corner.y - anchor.y).abs();
                                    let raw_screen = to_screen(raw_w_norm, raw_h_norm);

                                    // 3. Project onto aspect ratio vector
                                    // Vector direction U = (ratio, 1.0)
                                    let u = egui::vec2(ratio, 1.0);
                                    let p = raw_screen; // Computed target vector
                                    // Projection: (P . U) / (U . U) * U
                                    let lambda = p.dot(u) / u.length_sq();
                                    let constrained_screen = u * lambda;

                                    // 4. Convert back to normalized and update rect
                                    let final_dim =
                                        to_norm(constrained_screen.x, constrained_screen.y);

                                    // Reconstruct rect from Anchor
                                    let (new_min, new_max) = match handle {
                                        ResizeHandle::TopLeft => (anchor - final_dim, anchor),
                                        ResizeHandle::TopRight => (
                                            egui::pos2(anchor.x, anchor.y - final_dim.y),
                                            egui::pos2(anchor.x + final_dim.x, anchor.y),
                                        ),
                                        ResizeHandle::BottomLeft => (
                                            egui::pos2(anchor.x - final_dim.x, anchor.y),
                                            egui::pos2(anchor.x, anchor.y + final_dim.y),
                                        ),
                                        ResizeHandle::BottomRight => (anchor, anchor + final_dim),
                                        _ => (egui::Pos2::ZERO, egui::Pos2::ZERO),
                                    };

                                    // Update crop_rect (handling potential negative flips if crossed)
                                    // But since we used .abs() and fixed anchors, we assume simple expansion/shrinkage
                                    // However, simpler to just use from_min_max and let standardization happen later
                                    // But our logic assumes anchor is fixed OPPOSITE corner.
                                    *crop_rect = egui::Rect::from_min_max(new_min, new_max);
                                }

                                // Side Handles: Drive one dimension, center the other
                                ResizeHandle::Left | ResizeHandle::Right => {
                                    // Drive Width
                                    let mut new_w = crop_rect.width();
                                    match handle {
                                        ResizeHandle::Left => {
                                            crop_rect.min.x += delta_norm.x;
                                            new_w -= delta_norm.x;
                                        }
                                        ResizeHandle::Right => {
                                            crop_rect.max.x += delta_norm.x;
                                            new_w += delta_norm.x;
                                        }
                                        _ => {}
                                    }

                                    // Constrain Height
                                    let new_h = new_w / norm_aspect;
                                    let old_center_y = crop_rect.center().y;
                                    crop_rect.min.y = old_center_y - new_h * 0.5;
                                    crop_rect.max.y = old_center_y + new_h * 0.5;
                                }
                                ResizeHandle::Top | ResizeHandle::Bottom => {
                                    // Drive Height
                                    let mut new_h = crop_rect.height();
                                    match handle {
                                        ResizeHandle::Top => {
                                            crop_rect.min.y += delta_norm.y;
                                            new_h -= delta_norm.y;
                                        }
                                        ResizeHandle::Bottom => {
                                            crop_rect.max.y += delta_norm.y;
                                            new_h += delta_norm.y;
                                        }
                                        _ => {}
                                    }

                                    // Constrain Width
                                    let new_w = new_h * norm_aspect;
                                    let old_center_x = crop_rect.center().x;
                                    crop_rect.min.x = old_center_x - new_w * 0.5;
                                    crop_rect.max.x = old_center_x + new_w * 0.5;
                                }
                            }
                        } else {
                            // Free resize
                            match handle {
                                ResizeHandle::Center => {
                                    // Safe Panning: constrain delta to stay within bounds
                                    let mut final_delta = delta_norm;
                                    if crop_rect.min.x + final_delta.x < 0.0 {
                                        final_delta.x = -crop_rect.min.x;
                                    }
                                    if crop_rect.max.x + final_delta.x > 1.0 {
                                        final_delta.x = 1.0 - crop_rect.max.x;
                                    }
                                    if crop_rect.min.y + final_delta.y < 0.0 {
                                        final_delta.y = -crop_rect.min.y;
                                    }
                                    if crop_rect.max.y + final_delta.y > 1.0 {
                                        final_delta.y = 1.0 - crop_rect.max.y;
                                    }

                                    *crop_rect = crop_rect.translate(final_delta);
                                }
                                ResizeHandle::TopLeft => {
                                    crop_rect.min += delta_norm;
                                }
                                ResizeHandle::TopRight => {
                                    crop_rect.min.y += delta_norm.y;
                                    crop_rect.max.x += delta_norm.x;
                                }
                                ResizeHandle::BottomLeft => {
                                    crop_rect.min.x += delta_norm.x;
                                    crop_rect.max.y += delta_norm.y;
                                }
                                ResizeHandle::BottomRight => {
                                    crop_rect.max += delta_norm;
                                }
                                ResizeHandle::Top => {
                                    crop_rect.min.y += delta_norm.y;
                                }
                                ResizeHandle::Bottom => {
                                    crop_rect.max.y += delta_norm.y;
                                }
                                ResizeHandle::Left => {
                                    crop_rect.min.x += delta_norm.x;
                                }
                                ResizeHandle::Right => {
                                    crop_rect.max.x += delta_norm.x;
                                }
                            }
                        }

                        // Clamp and ensure min < max
                        if crop_rect.min.x < 0.0 {
                            crop_rect.min.x = 0.0;
                        }
                        if crop_rect.min.y < 0.0 {
                            crop_rect.min.y = 0.0;
                        }
                        if crop_rect.max.x > 1.0 {
                            crop_rect.max.x = 1.0;
                        }
                        if crop_rect.max.y > 1.0 {
                            crop_rect.max.y = 1.0;
                        }
                        // TODO: Ensure min < max
                        if crop_rect.min.x > crop_rect.max.x {
                            std::mem::swap(&mut crop_rect.min.x, &mut crop_rect.max.x);
                        }
                        if crop_rect.min.y > crop_rect.max.y {
                            std::mem::swap(&mut crop_rect.min.y, &mut crop_rect.max.y);
                        }

                        // Re-calculate screen rect for display after modification
                        screen_crop_rect = egui::Rect::from_min_max(
                            image_rect.lerp_inside(crop_rect.min.to_vec2()),
                            image_rect.lerp_inside(crop_rect.max.to_vec2()),
                        );
                    }
                }

                if response.drag_stopped() {
                    self.selected_handle = None;
                }

                // Draw overlay (dimmed area outside crop)
                let overlay_color = egui::Color32::from_black_alpha(150);

                // Top
                painter.rect_filled(
                    egui::Rect::from_min_max(
                        image_rect.min,
                        egui::pos2(image_rect.max.x, screen_crop_rect.min.y),
                    ),
                    0.0,
                    overlay_color,
                );
                // Bottom
                painter.rect_filled(
                    egui::Rect::from_min_max(
                        egui::pos2(image_rect.min.x, screen_crop_rect.max.y),
                        image_rect.max,
                    ),
                    0.0,
                    overlay_color,
                );
                // Left
                painter.rect_filled(
                    egui::Rect::from_min_max(
                        egui::pos2(image_rect.min.x, screen_crop_rect.min.y),
                        egui::pos2(screen_crop_rect.min.x, screen_crop_rect.max.y),
                    ),
                    0.0,
                    overlay_color,
                );
                // Right
                painter.rect_filled(
                    egui::Rect::from_min_max(
                        egui::pos2(screen_crop_rect.max.x, screen_crop_rect.min.y),
                        egui::pos2(image_rect.max.x, screen_crop_rect.max.y),
                    ),
                    0.0,
                    overlay_color,
                );

                // Draw crop border
                painter.rect_stroke(
                    screen_crop_rect,
                    0.0,
                    egui::Stroke::new(1.0, egui::Color32::WHITE),
                );

                // Draw handles
                let handle_radius = 6.0;
                let handle_stroke = egui::Stroke::new(1.0, egui::Color32::BLACK);
                let handle_fill = egui::Color32::WHITE;

                let handles = [
                    screen_crop_rect.min,
                    screen_crop_rect.max,
                    egui::pos2(screen_crop_rect.min.x, screen_crop_rect.max.y),
                    egui::pos2(screen_crop_rect.max.x, screen_crop_rect.min.y),
                    screen_crop_rect.center_top(),
                    screen_crop_rect.center_bottom(),
                    screen_crop_rect.left_center(),
                    screen_crop_rect.right_center(),
                ];

                for pos in handles {
                    painter.circle(pos, handle_radius, handle_fill, handle_stroke);
                }
            }
        });
    }
}

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([800.0, 600.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Image Cropper",
        options,
        Box::new(|cc| Ok(Box::new(ImageCropper::new(cc)))),
    )
}
