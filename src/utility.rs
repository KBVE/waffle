use egui::{self, Color32, Pos2, Rect, Response, Sense, Ui};

/// Draws a spinning loading indicator with a message.
pub fn show_loading_spinner(ui: &mut Ui, message: &str, progress: Option<f32>) -> Response {
    // Spinner parameters
    let spinner_radius = 16.0;
    let spinner_thickness = 4.0;
    let spinner_color = Color32::from_rgb(100, 200, 255);
    let spinner_size = egui::Vec2::splat(spinner_radius * 2.0 + spinner_thickness * 2.0);

    // Reserve space for spinner and message
    let (rect, response) = ui.allocate_exact_size(spinner_size, Sense::hover());
    let center = rect.center();

    // Animate spinner based on time
    let time = ui.input(|i| i.time);
    let start_angle = time as f32 * 2.0 * std::f32::consts::PI;
    let end_angle = start_angle + std::f32::consts::PI * 1.5;

    // Draw spinner arc using circle_segment for partial arc
    use egui::Stroke;
    let n_points = 64;
    let points: Vec<egui::Pos2> = (0..=n_points)
        .map(|i| {
            let t = i as f32 / n_points as f32;
            let angle = start_angle + t * (end_angle - start_angle);
            egui::pos2(
                center.x + spinner_radius * angle.cos(),
                center.y + spinner_radius * angle.sin(),
            )
        })
        .collect();
    ui.painter().add(egui::Shape::line(
        points,
        Stroke::new(spinner_thickness, spinner_color),
    ));

    // Optionally, show progress as text
    if let Some(p) = progress {
        let pct = (p * 100.0).round() as u32;
        ui.painter().text(
            center,
            egui::Align2::CENTER_CENTER,
            format!("{}%", pct),
            egui::TextStyle::Body.resolve(ui.style()),
            Color32::WHITE,
        );
    }

    // Show message below spinner
    let message_rect = Rect::from_min_max(
        Pos2::new(rect.left(), rect.bottom() + 4.0),
        Pos2::new(rect.right(), rect.bottom() + 28.0),
    );
    ui.put(message_rect, egui::widgets::Label::new(message));

    response
}

// Custom spinner with dark stone 950 background, bright cyan, and light purple
pub fn show_loading_spinner_custom(ui: &mut egui::Ui, message: &str, size: Option<f32>) -> egui::Response {
    let spinner_radius = size.unwrap_or(32.0) / 2.0;
    let spinner_thickness = 6.0;
    let spinner_color = egui::Color32::from_rgb(0, 255, 255); // bright cyan
    let accent_color = egui::Color32::from_rgb(180, 140, 255); // light purple
    let bg_color = egui::Color32::from_rgb(18, 24, 27); // stone 950
    let spinner_size = egui::Vec2::splat(spinner_radius * 2.0 + spinner_thickness * 2.0 + 16.0);

    // Reserve space for spinner and message
    let (rect, response) = ui.allocate_exact_size(spinner_size, egui::Sense::hover());
    let center = rect.center();

    // Draw border as a slightly larger rounded rectangle (simulated border)
    let border_rect = rect.expand(1.5);
    ui.painter().rect_filled(border_rect, 18.0, accent_color);
    // Draw background
    ui.painter().rect_filled(rect, 18.0, bg_color);

    // Animate spinner based on time
    let time = ui.input(|i| i.time);
    let start_angle = time as f32 * 2.0 * std::f32::consts::PI;
    let end_angle = start_angle + std::f32::consts::PI * 1.5;
    let n_points = 64;
    let points: Vec<egui::Pos2> = (0..=n_points)
        .map(|i| {
            let t = i as f32 / n_points as f32;
            let angle = start_angle + t * (end_angle - start_angle);
            egui::pos2(
                center.x + spinner_radius * angle.cos(),
                center.y + spinner_radius * angle.sin(),
            )
        })
        .collect();
    ui.painter().add(egui::Shape::line(
        points,
        egui::Stroke::new(spinner_thickness, spinner_color),
    ));
    // Accent arc
    let accent_start = start_angle + std::f32::consts::PI * 0.5;
    let accent_end = accent_start + std::f32::consts::PI * 0.5;
    let accent_points: Vec<egui::Pos2> = (0..=n_points/4)
        .map(|i| {
            let t = i as f32 / (n_points/4) as f32;
            let angle = accent_start + t * (accent_end - accent_start);
            egui::pos2(
                center.x + spinner_radius * angle.cos(),
                center.y + spinner_radius * angle.sin(),
            )
        })
        .collect();
    ui.painter().add(egui::Shape::line(
        accent_points,
        egui::Stroke::new(spinner_thickness + 1.5, accent_color),
    ));
    // Show message below spinner, bright cyan
    let message_rect = egui::Rect::from_min_max(
        egui::Pos2::new(rect.left(), rect.bottom() + 8.0),
        egui::Pos2::new(rect.right(), rect.bottom() + 36.0),
    );
    ui.put(message_rect, egui::widgets::Label::new(
        egui::RichText::new(message).color(spinner_color).strong()
    ));
    response
}
