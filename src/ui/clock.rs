use std::cell::Cell;
use std::rc::Rc;

use gtk4::cairo;
use gtk4::prelude::*;

pub struct CircularClock {
    drawing_area: gtk4::DrawingArea,
    progress: Rc<Cell<f64>>,
}

impl CircularClock {
    pub fn new() -> Self {
        let progress: Rc<Cell<f64>> = Rc::new(Cell::new(0.0));
        let progress_clone = progress.clone();

        let drawing_area = gtk4::DrawingArea::new();
        drawing_area.add_css_class("circular-clock");
        drawing_area.set_size_request(200, 200);

        drawing_area.set_draw_func(move |_widget, cr: &cairo::Context, width, height| {
            let progress = progress_clone.get();
            let center_x = width as f64 / 2.0;
            let center_y = height as f64 / 2.0;
            let radius = (width as f64 / 2.0).min(height as f64 / 2.0) - 12.0;

            // Background track
            cr.set_source_rgb(0.15, 0.15, 0.22);
            cr.set_line_width(8.0);
            cr.set_line_cap(cairo::LineCap::Round);
            cr.move_to(center_x, center_y - radius);
            cr.arc(center_x, center_y, radius, 0.0, 2.0 * std::f64::consts::PI);
            cr.stroke().ok();

            // Progress arc
            if progress > 0.0 {
                cr.set_source_rgb(0.65, 0.89, 0.63);
                cr.set_line_width(8.0);
                cr.set_line_cap(cairo::LineCap::Round);
                let end_angle = progress * 2.0 * std::f64::consts::PI;
                cr.move_to(center_x, center_y - radius);
                cr.arc(center_x, center_y, radius, 0.0, end_angle);
                cr.stroke().ok();
            }

            // Center text
            let text = format!("{:.0}%", progress * 100.0);
            cr.set_source_rgb(0.8, 0.83, 0.95);
            cr.set_font_size(28.0);
            if let Ok(extents) = cr.text_extents(&text) {
                cr.move_to(
                    center_x - extents.width() / 2.0,
                    center_y + extents.height() / 2.0,
                );
            }
            cr.show_text(&text).ok();
        });

        CircularClock {
            drawing_area,
            progress,
        }
    }

    pub fn set_progress(&self, value: f64) {
        self.progress.set(value.min(1.0).max(0.0));
        self.drawing_area.queue_draw();
    }

    pub fn widget(&self) -> &gtk4::DrawingArea {
        &self.drawing_area
    }
}
