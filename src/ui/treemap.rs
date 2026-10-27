use eframe::egui::{
    self, Align2, Color32, FontId, Pos2, Rect, Response, RichText, Sense, Stroke, Ui, Vec2,
};

use crate::app::DiskAgentApp;
use crate::format;
use crate::model::NodeId;
use crate::theme;
use crate::ui::icons::{self, Icon};

const PALETTE: [Color32; 8] = [
    Color32::from_rgb(91, 141, 191),
    Color32::from_rgb(230, 158, 82),
    Color32::from_rgb(116, 178, 138),
    Color32::from_rgb(199, 120, 120),
    Color32::from_rgb(142, 133, 191),
    Color32::from_rgb(102, 174, 190),
    Color32::from_rgb(208, 181, 94),
    Color32::from_rgb(150, 161, 172),
];

fn hash_name(name: &str) -> u64 {
    let mut hash: u64 = 0xcbf29ce484222325;
    for byte in name.bytes() {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

fn palette_color(name: &str, is_dir: bool) -> Color32 {
    let base = PALETTE[(hash_name(name) as usize) % PALETTE.len()];
    if is_dir {
        base
    } else {
        let gray = base.r() as u32 / 3 + base.g() as u32 / 3 + base.b() as u32 / 3;
        Color32::from_rgb(
            ((base.r() as u32 + gray) / 2) as u8,
            ((base.g() as u32 + gray) / 2) as u8,
            ((base.b() as u32 + gray) / 2) as u8,
        )
    }
}

fn text_color(bg: Color32) -> Color32 {
    let luma =
        0.299 * bg.r() as f32 + 0.587 * bg.g() as f32 + 0.114 * bg.b() as f32;
    if luma > 150.0 {
        Color32::from_rgb(30, 35, 40)
    } else {
        Color32::WHITE
    }
}

pub fn show(ui: &mut Ui, app: &mut DiskAgentApp) {
    app.ensure_rows();
    let Some(tree) = app.scan.as_ref() else { return };
    let mut root_id = app.treemap_root.unwrap_or(tree.root);
    if root_id >= tree.nodes.len() {
        root_id = tree.root;
    }
    let parent = tree.parent_id(root_id);
    let path_text = tree.nodes[root_id].path.to_string_lossy().into_owned();
    let lang = app.lang;
    let unit = app.unit;
    let decimals = app.decimals;
    let size = app
        .stats
        .as_ref()
        .map(|s| s.size[root_id])
        .unwrap_or(0);
    let root_node_id = tree.root;

    let mut new_root: Option<NodeId> = None;
    ui.horizontal(|ui| {
        ui.add_space(6.0);
        let up = tool_button(ui, Icon::Parent, parent.is_some(), app.t("Up"));
        if up.clicked() {
            if let Some(parent) = parent {
                new_root = Some(parent);
            }
        }
        if tool_button(ui, Icon::Drive, true, app.t("Root")).clicked() {
            new_root = Some(root_node_id);
        }
        ui.label(
            RichText::new(icons::ellipsis(&path_text, 110))
                .size(11.5)
                .color(theme::TEXT_MUTED),
        );
        ui.add_space(8.0);
        ui.label(
            RichText::new(format::format_bytes(size, unit, decimals, lang))
                .size(11.5)
                .color(theme::TEXT),
        );
        ui.with_layout(
            egui::Layout::right_to_left(egui::Align::Center),
            |ui| {
                ui.add_space(6.0);
                ui.label(
                    RichText::new(app.t("Double-click to zoom"))
                        .size(10.0)
                        .color(theme::TEXT_DIM),
                );
            },
        );
    });
    ui.separator();

    if let Some(id) = new_root {
        app.treemap_root = Some(id);
        root_id = id;
    }

    let Some(tree) = app.scan.as_ref() else { return };
    let Some(stats) = app.stats.as_ref() else { return };
    let selected = app.selected;

    let mut items: Vec<(NodeId, f64)> = tree.nodes[root_id]
        .children
        .iter()
        .filter(|&&id| stats.visible[id] && stats.size[id] > 0)
        .map(|&id| (id, stats.size[id] as f64))
        .collect();
    items.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    items.truncate(300);

    let rect = ui.available_rect_before_wrap();
    let (canvas, _) = ui.allocate_exact_size(rect.size(), Sense::hover());
    if items.is_empty() {
        ui.painter().text(
            canvas.center(),
            Align2::CENTER_CENTER,
            if tree.nodes[root_id].is_dir {
                app.t("No matches")
            } else {
                app.t("File")
            },
            FontId::proportional(13.0),
            theme::TEXT_MUTED,
        );
        return;
    }

    let inner = canvas.shrink(6.0);
    ui.painter().rect_filled(
        inner,
        egui::Rounding::same(3.0),
        Color32::from_rgb(250, 251, 252),
    );
    let laid_out = squarify(&items, inner);
    let mut pending_select: Option<NodeId> = None;
    let mut pending_zoom: Option<NodeId> = None;
    let mut pending_explain: Option<NodeId> = None;
    let mut pending_delete: Option<NodeId> = None;

    for (id, rect) in laid_out {
        let node = &tree.nodes[id];
        let fill = palette_color(&node.name, node.is_dir);
        let response = ui.interact(rect, ui.id().with(("treemap", id)), Sense::click());
        let hovered = response.hovered();
        let clicked = response.clicked();
        let double = response.double_clicked();
        let selected_here = selected == Some(id);

        let draw_fill = if hovered {
            lighten(fill, 0.12)
        } else {
            fill
        };
        ui.painter()
            .rect_filled(rect.shrink(0.8), egui::Rounding::same(2.0), draw_fill);
        ui.painter().rect_stroke(
            rect.shrink(0.8),
            egui::Rounding::same(2.0),
            Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(255, 255, 255, 180)),
        );
        if selected_here {
            ui.painter().rect_stroke(
                rect.shrink(1.8),
                egui::Rounding::same(2.0),
                Stroke::new(2.0_f32, theme::ACCENT),
            );
        }

        if rect.width() > 48.0 && rect.height() > 16.0 {
            let clip = ui.painter().with_clip_rect(rect.shrink(4.0));
            let color = text_color(draw_fill);
            let pct = crate::app::percent_value(tree, stats, id);
            let label = if rect.height() > 32.0 {
                format!(
                    "{}\n{}  ({})",
                    icons::ellipsis(&node.name, ((rect.width() - 8.0) / 6.0) as usize),
                    format::format_bytes(stats.size[id], unit, decimals, lang),
                    format::format_percent(pct, lang)
                )
            } else {
                icons::ellipsis(&node.name, ((rect.width() - 8.0) / 6.0) as usize)
            };
            clip.text(
                rect.left_top() + Vec2::new(4.0, 4.0),
                Align2::LEFT_TOP,
                label,
                FontId::proportional(11.0),
                color,
            );
        }

        if clicked {
            pending_select = Some(id);
        }
        if response.secondary_clicked() {
            pending_select = Some(id);
            ui.ctx().request_repaint();
        }
        if double && node.is_dir {
            pending_zoom = Some(id);
        }

        let response = response.on_hover_ui(|ui| {
            ui.label(RichText::new(&node.name).strong());
            ui.label(
                RichText::new(node.path.to_string_lossy().into_owned())
                    .size(11.0)
                    .color(theme::TEXT_MUTED),
            );
            ui.label(format!(
                "{}  ·  {} {}  ·  {} {}",
                format::format_bytes(stats.size[id], unit, decimals, lang),
                format::format_count(stats.files[id], lang),
                app.t("files"),
                format::format_count(stats.folders[id], lang),
                app.t("folders")
            ));
        });

        let mut request_explain = false;
        let mut request_delete = false;
        let is_root = id == tree.root;
        response.context_menu(|ui| {
            let explain = if node.is_dir {
                app.t("What is this folder?")
            } else {
                app.t("What is this file?")
            };
            if ui.button(explain).clicked() {
                request_explain = true;
                ui.close_menu();
            }
            ui.separator();

            let label = RichText::new(app.t("Delete permanently"));
            let label = if is_root {
                label
            } else {
                label.color(theme::DANGER)
            };
            let item = ui.add_enabled(!is_root, egui::Button::new(label));
            if item.clicked() {
                request_delete = true;
                ui.close_menu();
            }
            if is_root {
                item.on_disabled_hover_text(app.t("The scan root cannot be deleted"));
            }
        });
        if request_explain {
            pending_explain = Some(id);
            pending_select = Some(id);
        }
        if request_delete {
            pending_delete = Some(id);
            pending_select = Some(id);
        }
    }

    if let Some(id) = pending_select {
        app.selected = Some(id);
    }
    if let Some(id) = pending_zoom {
        app.treemap_root = Some(id);
        app.selected = Some(id);
    }
    if let Some(id) = pending_explain {
        app.explain_node(id);
    }
    if let Some(id) = pending_delete {
        app.request_delete(id);
    }
}

fn tool_button(ui: &mut Ui, icon: Icon, enabled: bool, tip: &str) -> Response {
    let sense = if enabled { Sense::click() } else { Sense::hover() };
    let (rect, response) = ui.allocate_exact_size(Vec2::splat(22.0), sense);
        if enabled && response.hovered() {
        ui.painter()
            .rect_filled(rect.shrink(2.0), egui::Rounding::same(3.0), theme::ACCENT_SOFT);
    }
    icons::draw(
        ui.painter(),
        icon,
        rect.shrink(3.0),
        if enabled {
            theme::ACCENT_DARK
        } else {
            Color32::from_rgb(195, 199, 203)
        },
    );
    response.on_hover_text(tip.to_string())
}

fn lighten(color: Color32, amount: f32) -> Color32 {    let blend = |c: u8| -> u8 {
        let value = c as f32 + (255.0 - c as f32) * amount;
        value.clamp(0.0, 255.0) as u8
    };
    Color32::from_rgb(blend(color.r()), blend(color.g()), blend(color.b()))
}

fn worst(row: &[f64], side: f64) -> f64 {
    let sum: f64 = row.iter().sum();
    let max = row.iter().cloned().fold(0.0, f64::max);
    let min = row.iter().cloned().fold(f64::INFINITY, f64::min);
    if sum <= 0.0 || min <= 0.0 {
        return f64::INFINITY;
    }
    ((side * side * max) / (sum * sum)).max((sum * sum) / (side * side * min))
}

fn squarify(items: &[(NodeId, f64)], rect: Rect) -> Vec<(NodeId, Rect)> {
    let total: f64 = items.iter().map(|(_, value)| *value).sum();
    if total <= 0.0 || rect.width() <= 0.0 || rect.height() <= 0.0 {
        return Vec::new();
    }
    let area = rect.area() as f64;
    let scaled: Vec<(NodeId, f64)> = items
        .iter()
        .map(|(id, value)| (*id, value / total * area))
        .collect();
    let mut out = Vec::with_capacity(scaled.len());
    squarify_inner(&scaled, rect, &mut out);
    out
}

fn squarify_inner(items: &[(NodeId, f64)], rect: Rect, out: &mut Vec<(NodeId, Rect)>) {
    if items.is_empty() || rect.width() <= 0.0 || rect.height() <= 0.0 {
        return;
    }
    if items.len() == 1 {
        out.push((items[0].0, rect));
        return;
    }

    let side = rect.width().min(rect.height()) as f64;
    let mut row: Vec<f64> = Vec::new();
    let mut index = 0;
    let mut best = f64::INFINITY;

    while index < items.len() {
        let mut trial = row.clone();
        trial.push(items[index].1);
        let ratio = worst(&trial, side);
        if row.is_empty() || ratio <= best {
            row = trial;
            best = ratio;
            index += 1;
        } else {
            break;
        }
    }

    let row_sum: f64 = row.iter().sum();
    let horizontal = rect.width() <= rect.height();
    let (strip, rest) = if horizontal {
        let height = (row_sum / rect.width() as f64) as f32;
        (
            Rect::from_min_size(rect.min, Vec2::new(rect.width(), height)),
            Rect::from_min_max(
                Pos2::new(rect.left(), rect.top() + height),
                rect.max,
            ),
        )
    } else {
        let width = (row_sum / rect.height() as f64) as f32;
        (
            Rect::from_min_size(rect.min, Vec2::new(width, rect.height())),
            Rect::from_min_max(
                Pos2::new(rect.left() + width, rect.top()),
                rect.max,
            ),
        )
    };

    let strip_length = if horizontal {
        strip.width()
    } else {
        strip.height()
    };
    let mut offset = 0.0f32;
    for (i, value) in row.iter().enumerate() {
        let fraction = (value / row_sum) as f32;
        let length = (strip_length * fraction).max(0.0);
        let cell = if horizontal {
            Rect::from_min_size(
                Pos2::new(strip.left() + offset, strip.top()),
                Vec2::new(length, strip.height()),
            )
        } else {
            Rect::from_min_size(
                Pos2::new(strip.left(), strip.top() + offset),
                Vec2::new(strip.width(), length),
            )
        };
        out.push((items[i].0, cell.shrink(0.5)));
        offset += length;
    }

    squarify_inner(&items[index..], rest, out);
}
