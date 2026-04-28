use std::cell::Cell;
use std::rc::Rc;

use gtk4::cairo;
use gtk4::prelude::*;

use crate::db::HabitMode;

pub struct CircularClock {
    drawing_area: gtk4::DrawingArea,
    progress: Rc<Cell<f64>>,
    elapsed_secs: Rc<Cell<u32>>,
    mode: Rc<Cell<HabitMode>>
}

impl CircularClock {
    pub fn new() -> Self {
        let progress: Rc<Cell<f64>> = Rc::new(Cell::new(0.0));
        let elapsed_secs: Rc<Cell<u32>> = Rc::new(Cell::new(0));
        let mode: Rc<Cell<HabitMode>> = Rc::new(Cell::new(HabitMode::Timed));
        let progress_clone = progress.clone();
        let elapsed_clone = elapsed_secs.clone();
        let mode_clone = mode.clone();

        let drawing_area = gtk4::DrawingArea::new();
        drawing_area.add_css_class("circular-clock");
        drawing_area.set_size_request(100, 100);

        drawing_area.set_draw_func(move |_widget, cr: &cairo::Context, width, height| {
            let progress = progress_clone.get();
            let elapsed = elapsed_clone.get();
            let is_stopwatch = matches!(mode_clone.get(), HabitMode::Stopwatch);
            let center_x = width as f64 / 2.0;
            let center_y = height as f64 / 2.0;
            let radius = (width as f64 / 2.0).min(height as f64 / 2.0) - 12.0;

            // Background track
            cr.set_source_rgb(0.212, 0.227, 0.310);
            cr.set_line_width(6.0);
            cr.set_line_cap(cairo::LineCap::Round);
            cr.move_to(center_x, center_y - radius);
            cr.arc(
                center_x,
                center_y,
                radius,
                -std::f64::consts::PI / 2.0,
                3.0 * std::f64::consts::PI / 2.0,
            );
            cr.stroke().ok();

            // Progress arc
            if !is_stopwatch && progress > 0.0 {
                cr.set_source_rgb(0.65, 0.89, 0.63);
                cr.set_line_width(6.0);
                cr.set_line_cap(cairo::LineCap::Round);
                let end_angle = -std::f64::consts::PI / 2.0 + progress * 2.0 * std::f64::consts::PI;
                cr.move_to(center_x, center_y - radius);
                cr.arc(
                    center_x,
                    center_y,
                    radius,
                    -std::f64::consts::PI / 2.0,
                    end_angle,
                );
                cr.stroke().ok();
            }

            // Center text
            let text = if is_stopwatch {
                let hrs = elapsed / 3600;
                let mins = (elapsed % 3600) / 60;
                let secs = elapsed % 60;
                if hrs > 0 {
                    format!("{:02}:{:02}:{:02}", hrs, mins, secs)
                } else {
                    format!("{:02}:{:02}", mins, secs)
                }
            } else {
                format!("{:.0}%", progress * 100.0)
            };
            cr.set_source_rgb(0.8, 0.83, 0.95);
            cr.set_font_size(18.0);
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
            elapsed_secs,
            mode,
        }
    }

    pub fn set_progress(&self, value: f64) {
        self.progress.set(value.min(1.0).max(0.0));
        self.elapsed_secs.set(0);
        self.drawing_area.queue_draw();
    }

    pub fn set_stopwatch_elapsed(&self, secs: u32) {
        self.elapsed_secs.set(secs);
        self.progress.set(0.0);
        self.drawing_area.queue_draw();
    }

    pub fn set_mode(&self, mode: HabitMode) {
        self.mode.set(mode);
        self.drawing_area.queue_draw();
    }

    pub fn get_mode(&self) -> HabitMode {
        self.mode.get()
    }

    pub fn widget(&self) -> &gtk4::DrawingArea {
        &self.drawing_area
    }
}
