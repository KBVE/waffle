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
