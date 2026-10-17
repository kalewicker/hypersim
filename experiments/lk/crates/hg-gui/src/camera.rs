//! Viewport transform for the graph canvas. Maps world-space graph
//! coordinates (whatever the layout produces) to screen-space pixels.

use eframe::egui;

#[derive(Clone, Copy, Debug)]
pub struct Camera2D {
    /// World-space point currently centered in the viewport.
    pub center: egui::Vec2,
    /// Pixels per world unit. Larger = more zoomed in.
    pub zoom: f32,
}

pub const ZOOM_MIN: f32 = 0.01;
pub const ZOOM_MAX: f32 = 100.0;

impl Default for Camera2D {
    fn default() -> Self {
        // Deliberately do NOT trust this default to frame anything — the
        // first frame that has graph data must call fit_to_bounds.
        Self {
            center: egui::Vec2::ZERO,
            zoom: 1.0,
        }
    }
}

impl Camera2D {
    /// World point -> screen point, given the viewport rect.
    pub fn world_to_screen(&self, world: egui::Vec2, viewport: egui::Rect) -> egui::Pos2 {
        let centered = (world - self.center) * self.zoom;
        viewport.center() + centered
    }

    /// Screen point -> world point (needed for zoom-to-cursor).
    pub fn screen_to_world(&self, screen: egui::Pos2, viewport: egui::Rect) -> egui::Vec2 {
        (screen - viewport.center()) / self.zoom + self.center
    }

    /// Frame `bounds` (world-space AABB of all node positions) within
    /// `viewport`, with a margin so nodes aren't flush against the edge.
    pub fn fit_to_bounds(&mut self, bounds: egui::Rect, viewport: egui::Rect) {
        if bounds.width() <= f32::EPSILON && bounds.height() <= f32::EPSILON {
            // Degenerate: all nodes at one point (or one node). Center on
            // it, pick a neutral zoom rather than dividing by ~zero.
            self.center = bounds.center().to_vec2();
            self.zoom = 1.0;
            return;
        }
        self.center = bounds.center().to_vec2();
        let margin = 0.9; // use 90% of the viewport, leaving a border
        let zoom_x = viewport.width() / bounds.width().max(f32::EPSILON);
        let zoom_y = viewport.height() / bounds.height().max(f32::EPSILON);
        self.zoom = (zoom_x.min(zoom_y) * margin).clamp(ZOOM_MIN, ZOOM_MAX);
    }
}
