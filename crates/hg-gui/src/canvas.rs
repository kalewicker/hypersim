//! Graph canvas: force-directed layout of the CompactGraphSnapshot with a
//! pan/zoom camera. Color modes: L = by label, E = energy proxy (blue ->
//! red), G = by generation.
//!
//! Canvas was showing a bare ring because no layout existed at all — the
//! old code computed every node position as `angle = i/n · τ` on a fixed
//! circle (Hypothesis C of the GUI-improvements doc, in its degenerate
//! form: positions were pure seed geometry, so a subdivided crystal drew
//! as a circle by construction). Fixed by adding `ForceLayout` (persistent
//! world-space positions keyed by real NodeId, relaxed incrementally each
//! frame) plus `Camera2D` (the old view was also unframed — camera framing
//! alone, Hypothesis A, was real but not sufficient).

use crate::camera::{Camera2D, ZOOM_MAX, ZOOM_MIN};
use crate::layout::ForceLayout;
use egui::{Color32, Pos2, Rect, Sense, Stroke, Vec2};
use hg_app_api::CompactGraphSnapshot;
use slotmap::Key;

#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub enum ColorMode {
    #[default]
    Label,
    Energy,
    Generation,
}

/// Camera + layout state for the graph canvas, owned by the app across
/// frames.
pub struct GraphView {
    pub camera: Camera2D,
    layout: ForceLayout,
    /// Keep the whole graph framed until the user pans/zooms; set on
    /// construction and on every `reset` (new run).
    auto_fit: bool,
    /// One-shot fit request (hotkey `f` / View menu).
    fit_requested: bool,
    /// View-menu toggle: thicker/brighter edges, smaller/dimmer nodes —
    /// the connectivity-comparison view (is this a tree or a mesh?).
    pub emphasize_edges: bool,
}

impl Default for GraphView {
    fn default() -> Self {
        Self {
            camera: Camera2D::default(),
            layout: ForceLayout::default(),
            auto_fit: true,
            fit_requested: false,
            emphasize_edges: false,
        }
    }
}

impl GraphView {
    /// New run: forget old positions (fresh slotmap ids may collide with
    /// stale keys) and re-enable auto-framing.
    pub fn reset(&mut self) {
        self.layout.clear();
        self.auto_fit = true;
        self.fit_requested = false;
    }

    pub fn request_fit(&mut self) {
        self.fit_requested = true;
    }

    pub fn show(&mut self, ui: &mut egui::Ui, graph: &CompactGraphSnapshot, mode: ColorMode) {
        let desired = ui.available_size();
        let (rect, response) = ui.allocate_exact_size(desired, Sense::click_and_drag());
        let painter = ui.painter_at(rect);
        painter.rect_filled(rect, 4.0, Color32::from_gray(18));

        if graph.nodes.is_empty() {
            painter.text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                "empty graph",
                egui::FontId::proportional(14.0),
                Color32::GRAY,
            );
            return;
        }

        self.layout.sync(graph);
        let moving = self.layout.step(graph);

        // Pan: drag with the primary button. Drag delta is in screen
        // pixels; divide by zoom for world units.
        if response.dragged() && response.drag_delta() != Vec2::ZERO {
            self.camera.center -= response.drag_delta() / self.camera.zoom;
            self.auto_fit = false;
        }

        // Zoom: scroll wheel, anchored at the cursor so the point under
        // the cursor stays put.
        if let Some(hover_pos) = response.hover_pos() {
            let scroll = ui.input(|i| i.smooth_scroll_delta.y);
            if scroll != 0.0 {
                let world_before = self.camera.screen_to_world(hover_pos, rect);
                let zoom_factor = (scroll * 0.001).exp(); // exponential = smooth, symmetric
                self.camera.zoom = (self.camera.zoom * zoom_factor).clamp(ZOOM_MIN, ZOOM_MAX);
                let world_after = self.camera.screen_to_world(hover_pos, rect);
                self.camera.center += world_before - world_after;
                self.auto_fit = false;
            }
        }

        if ui.input(|i| i.key_pressed(egui::Key::F)) {
            self.fit_requested = true;
        }

        if self.auto_fit || self.fit_requested {
            if let Some(bounds) = self.layout.bounds() {
                self.camera.fit_to_bounds(bounds, rect);
                self.fit_requested = false;
            }
        }

        let to_screen = |world: Vec2| -> Pos2 { self.camera.world_to_screen(world, rect) };
        let screen_pos = |id| self.layout.position(id).map(to_screen);

        // Node radius and edge width scale with zoom (world-sized, not
        // fixed screen pixels) and shrink with node count, both clamped.
        // Fixed-pixel dots made every zoomed-out graph an edge-free blob:
        // thousands of 4px dots pack solid and bury the hairline edges,
        // which is exactly the condition under which a tree and a mesh
        // render identically.
        let n = graph.nodes.len() as f32;
        let count_scale = (200.0 / n).sqrt().clamp(0.35, 1.5);
        let (node_world_r, edge_world_w, edge_color, node_alpha) = if self.emphasize_edges {
            (1.4, 0.5, Color32::from_gray(160), 140u8)
        } else {
            (2.6, 0.28, Color32::from_gray(110), 255u8)
        };
        let node_r = (node_world_r * count_scale * self.camera.zoom).clamp(0.5, 6.0);
        let edge_w = (edge_world_w * count_scale * self.camera.zoom).clamp(1.0, 3.0);

        // Edges first (nodes paint on top): polyline through member nodes
        // (hyperedges as fans). Skip segments whose bbox misses the view.
        for edge in &graph.edges {
            let pts: Vec<Pos2> = edge.nodes.iter().filter_map(|&id| screen_pos(id)).collect();
            if pts.len() >= 2 {
                let seg = |a: Pos2, b: Pos2| {
                    if Rect::from_two_pos(a, b).intersects(rect) {
                        painter.line_segment([a, b], Stroke::new(edge_w, edge_color));
                    }
                };
                for w in pts.windows(2) {
                    seg(w[0], w[1]);
                }
                if pts.len() > 2 {
                    seg(*pts.last().unwrap(), pts[0]);
                }
            }
        }

        let max_gen = graph.nodes.iter().map(|n| n.generation).max().unwrap_or(1).max(1);
        let max_energy = graph
            .nodes
            .iter()
            .map(|n| n.energy)
            .fold(0.0f32, f32::max)
            .max(1e-6);
        let cull = rect.expand(8.0);

        for node in &graph.nodes {
            let Some(p) = screen_pos(node.id) else { continue };
            if !cull.contains(p) {
                continue;
            }
            let color = match mode {
                ColorMode::Label => label_color(node.label.0),
                ColorMode::Energy => {
                    // Blue -> red by energy proxy (v4 §3 rationale).
                    let t = (node.energy / max_energy).clamp(0.0, 1.0);
                    Color32::from_rgb((t * 255.0) as u8, 40, ((1.0 - t) * 255.0) as u8)
                }
                ColorMode::Generation => {
                    let t = node.generation as f32 / max_gen as f32;
                    Color32::from_rgb(40, (80.0 + t * 175.0) as u8, 60)
                }
            };
            let color = Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), node_alpha);
            painter.circle_filled(p, node_r, color);
        }

        // Hover: show the REAL NodeId (same identifier the causal
        // inspector uses — no translation layer, v4 §2.1).
        if let Some(hover) = response.hover_pos() {
            for node in &graph.nodes {
                let Some(p) = screen_pos(node.id) else { continue };
                if p.distance(hover) < (node_r + 3.0).max(6.0) {
                    egui::show_tooltip_at_pointer(
                        ui.ctx(),
                        ui.layer_id(),
                        egui::Id::new("node_tip"),
                        |ui| {
                            ui.label(format!(
                                "node {:?}\nlabel {} gen {} energy {:.3}",
                                node.id.data(),
                                node.label.0,
                                node.generation,
                                node.energy
                            ));
                        },
                    );
                    break;
                }
            }
        }

        if moving {
            ui.ctx().request_repaint();
        }
    }
}

fn label_color(label: u32) -> Color32 {
    const PALETTE: [Color32; 8] = [
        Color32::from_rgb(102, 194, 165),
        Color32::from_rgb(252, 141, 98),
        Color32::from_rgb(141, 160, 203),
        Color32::from_rgb(231, 138, 195),
        Color32::from_rgb(166, 216, 84),
        Color32::from_rgb(255, 217, 47),
        Color32::from_rgb(229, 196, 148),
        Color32::from_rgb(179, 179, 179),
    ];
    PALETTE[(label as usize) % PALETTE.len()]
}
