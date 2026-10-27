use eframe::egui::{
    self, Align2, Color32, FontId, Pos2, Rect, RichText, Sense, Stroke, Ui, Vec2,
};

use crate::app::{DiskAgentApp, SortKey};
use crate::format;
use crate::model::NodeId;
use crate::platform;
use crate::theme;
use crate::ui::icons::{self, Icon};

const HEADER_H: f32 = 26.0;
const ROW_H: f32 = 22.0;

#[derive(Copy, Clone)]
struct Col {
    key: SortKey,
    label: &'static str,
    width: f32,
    right: bool,
}

fn build_layout(app: &DiskAgentApp, available: f32) -> Vec<Col> {
    let lang = app.lang;
    let mut cols = vec![Col {
        key: SortKey::Name,
        label: crate::i18n::t(lang, "Name"),
        width: 0.0,
        right: false,
    }];
    let mut fixed = 0.0f32;
    let mut push = |key: SortKey, label: &'static str, width: f32| {
        cols.push(Col {
            key,
            label,
            width,
            right: true,
        });
        fixed += width;
    };
    if app.columns.size {
        push(SortKey::Size, crate::i18n::t(lang, "Size"), 88.0);
    }
    if app.columns.alloc {
        push(SortKey::Alloc, crate::i18n::t(lang, "Allocated"), 88.0);
    }
    if app.columns.files {
        push(SortKey::Files, crate::i18n::t(lang, "Files"), 56.0);
    }
    if app.columns.folders {
        push(SortKey::Folders, crate::i18n::t(lang, "Folders"), 56.0);
    }
    if app.columns.percent {
        push(SortKey::Percent, crate::i18n::t(lang, "% of Parent"), 88.0);
    }
    if app.columns.modified {
        push(SortKey::Modified, crate::i18n::t(lang, "Last Modified"), 92.0);
    }
    cols[0].width = (available - fixed).max(180.0);
    cols
}

pub fn show(ui: &mut Ui, app: &mut DiskAgentApp) {
    app.ensure_rows();
    let available = ui.available_width().max(320.0);
    let cols = build_layout(app, available);

    // Header ---------------------------------------------------------
    let (header_rect, _) =
        ui.allocate_exact_size(Vec2::new(available, HEADER_H), Sense::hover());
    ui.painter()
        .rect_filled(header_rect, egui::Rounding::ZERO, Color32::from_rgb(247, 248, 249));
    ui.painter().line_segment(
        [
            Pos2::new(header_rect.left(), header_rect.bottom() - 0.5),
            Pos2::new(header_rect.right(), header_rect.bottom() - 0.5),
        ],
        Stroke::new(1.0_f32, theme::BORDER),
    );

    let mut x = header_rect.left();
    for (i, col) in cols.iter().enumerate() {
        let cell = Rect::from_min_size(
            Pos2::new(x, header_rect.top()),
            Vec2::new(col.width, header_rect.height()),
        );
        let response = ui.interact(
            cell,
            ui.id().with(("details-header", i)),
            Sense::click(),
        );
        if response.clicked() {
            app.set_sort(col.key);
        }
        let color = if app.sort_key == col.key {
            theme::ACCENT_DARK
        } else {
            theme::TEXT_MUTED
        };
        if col.right {
            ui.painter().text(
                Pos2::new(cell.right() - 8.0, cell.center().y),
                Align2::RIGHT_CENTER,
                col.label,
                FontId::proportional(11.0),
                color,
            );
            if app.sort_key == col.key {
                ui.painter().text(
                    Pos2::new(cell.left() + 6.0, cell.center().y),
                    Align2::LEFT_CENTER,
                    if app.sort_desc { "▾" } else { "▴" },
                    FontId::proportional(10.0),
                    theme::ACCENT,
                );
            }
        } else {
            ui.painter().text(
                Pos2::new(cell.left() + 8.0, cell.center().y),
                Align2::LEFT_CENTER,
                col.label,
                FontId::proportional(11.0),
                color,
            );
            if app.sort_key == col.key {
                ui.painter().text(
                    Pos2::new(cell.right() - 12.0, cell.center().y),
                    Align2::CENTER_CENTER,
                    if app.sort_desc { "▾" } else { "▴" },
                    FontId::proportional(10.0),
                    theme::ACCENT,
                );
            }
        }
        if i + 1 < cols.len() {
            ui.painter().line_segment(
                [
                    Pos2::new(cell.right() - 0.5, cell.top() + 5.0),
                    Pos2::new(cell.right() - 0.5, cell.bottom() - 5.0),
                ],
                Stroke::new(1.0_f32, Color32::from_rgb(232, 234, 236)),
            );
        }
        x += col.width;
    }

    if app.rows.is_empty() {
        ui.add_space(28.0);
        ui.vertical_centered(|ui| {
            ui.label(
                egui::RichText::new(app.t("No matches"))
                    .size(12.0)
                    .color(theme::TEXT_MUTED),
            );
        });
        return;
    }

    // Rows -----------------------------------------------------------
    let lang = app.lang;
    let unit = app.unit;
    let decimals = app.decimals;
    let selected = app.selected;
    let row_scroll = app.row_scroll.take();

    let mut pending_select: Option<NodeId> = None;
    let mut pending_toggle: Option<NodeId> = None;
    let mut pending_open: Option<NodeId> = None;
    let mut pending_explain: Option<NodeId> = None;
    let mut pending_delete: Option<NodeId> = None;

    {
        let tree = app.scan.as_ref().expect("scan checked by caller");
        let stats = match app.stats.as_ref() {
            Some(s) => s,
            None => return,
        };
        let rows = &app.rows;
        let total_size = stats.size[tree.root].max(1);

        let mut scroll = egui::ScrollArea::vertical().auto_shrink([false, false]);
        if let Some(index) = row_scroll {
            scroll = scroll.vertical_scroll_offset(index as f32 * ROW_H);
        }

        scroll.show_rows(ui, ROW_H, rows.len(), |ui, range| {
            ui.spacing_mut().item_spacing.y = 0.0;
            let width = ui.available_width();
            for index in range {
                let id = rows[index];
                let node = tree.node(id);
                let (rect, response) = ui.allocate_exact_size(
                    Vec2::new(width, ROW_H),
                    Sense::click(),
                );
                let hovered = response.hovered();
                let is_selected = selected == Some(id);
                let bg = if is_selected {
                    theme::SELECT
                } else if hovered {
                    theme::ROW_HOVER
                } else if index % 2 == 1 {
                    theme::ROW_ALT
                } else {
                    Color32::WHITE
                };
                ui.painter().rect_filled(rect, egui::Rounding::ZERO, bg);
                if is_selected {
                    ui.painter().rect_stroke(
                        rect,
                        egui::Rounding::ZERO,
                        Stroke::new(1.0_f32, theme::SELECT_BORDER),
                    );
                }

                if response.clicked() {
                    pending_select = Some(id);
                }
                if response.secondary_clicked() {
                    pending_select = Some(id);
                    ui.ctx().request_repaint();
                }
                if response.double_clicked() {
                    if node.is_dir {
                        pending_toggle = Some(id);
                    } else {
                        pending_open = Some(id);
                    }
                }
                let path_tip = tree.path_of(id).display().to_string();
                let response = response.on_hover_text(path_tip);

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

                let name_rect = Rect::from_min_size(
                    rect.min,
                    Vec2::new(cols[0].width, ROW_H),
                );
                let pct_parent = app.percent_of(id);
                let pct_total = stats.size[id] as f64 / total_size as f64 * 100.0;
                icons::paint_progress_gradient(
                    ui.painter(),
                    name_rect,
                    (pct_parent / 100.0) as f32,
                    pct_total >= 15.0,
                );

                let base_x = name_rect.left() + 4.0 + node.depth as f32 * 13.0;
                if node.is_dir && !node.children.is_empty() {
                    let exp_rect = Rect::from_center_size(
                        Pos2::new(base_x + 7.0, name_rect.center().y),
                        Vec2::splat(13.0),
                    );
                    let exp_response = ui.interact(
                        exp_rect,
                        ui.id().with(("expander", id)),
                        Sense::click(),
                    );
                    icons::paint_expander(
                        ui.painter(),
                        exp_rect,
                        app.expanded.contains(&id),
                        if exp_response.hovered() {
                            theme::ACCENT_DARK
                        } else {
                            theme::TEXT_MUTED
                        },
                    );
                    if exp_response.clicked() {
                        pending_toggle = Some(id);
                    }
                }

                let icon_rect = Rect::from_center_size(
                    Pos2::new(base_x + 25.0, name_rect.center().y),
                    Vec2::splat(16.0),
                );
                icons::draw(
                    ui.painter(),
                    if node.is_dir { Icon::Folder } else { Icon::File },
                    icon_rect,
                    theme::FILE,
                );

                let size_prefix = format::format_bytes(
                    stats.size[id],
                    unit,
                    decimals,
                    lang,
                );
                let font = FontId::proportional(11.8);
                let small_font = FontId::proportional(11.0);
                let mut name_x = base_x + 36.0;
                if name_rect.width() > 280.0 {
                    let size_width = ui.fonts(|f| {
                        f.layout_no_wrap(size_prefix.clone(), small_font.clone(), theme::TEXT_MUTED)
                            .size()
                            .x
                    });
                    ui.painter().text(
                        Pos2::new(name_x, name_rect.center().y),
                        Align2::LEFT_CENTER,
                        &size_prefix,
                        small_font.clone(),
                        theme::TEXT_MUTED,
                    );
                    name_x += size_width + 7.0;
                }

                let display_name = if id == tree.root {
                    node.path.to_string_lossy().into_owned()
                } else {
                    node.name.clone()
                };
                let max_chars = ((name_rect.right() - name_x - 4.0) / 6.5).max(3.0) as usize;
                let shown = icons::ellipsis(&display_name, max_chars);
                let color = if node.compressed {
                    Color32::from_rgb(28, 74, 168)
                } else if node.hidden {
                    theme::TEXT_MUTED
                } else {
                    theme::TEXT
                };
                let clip = ui.painter().with_clip_rect(
                    Rect::from_min_max(
                        name_rect.min,
                        Pos2::new(name_rect.right() - 2.0, name_rect.bottom()),
                    )
                    .intersect(ui.clip_rect()),
                );
                let position = Pos2::new(name_x, name_rect.center().y);
                clip.text(
                    position,
                    Align2::LEFT_CENTER,
                    shown,
                    font.clone(),
                    color,
                );
                if pct_total >= 15.0 {
                    clip.text(
                        Pos2::new(name_x + 0.5, name_rect.center().y),
                        Align2::LEFT_CENTER,
                        icons::ellipsis(&display_name, max_chars),
                        font.clone(),
                        color,
                    );
                }

                let mut cx = rect.left() + cols[0].width;
                for col in cols.iter().skip(1) {
                    let cell = Rect::from_min_size(
                        Pos2::new(cx, rect.top()),
                        Vec2::new(col.width, ROW_H),
                    );
                    match col.key {
                        SortKey::Size => {
                            ui.painter().text(
                                Pos2::new(cell.right() - 8.0, cell.center().y),
                                Align2::RIGHT_CENTER,
                                format::format_bytes(stats.size[id], unit, decimals, lang),
                                small_font.clone(),
                                theme::TEXT,
                            );
                        }
                        SortKey::Alloc => {
                            ui.painter().text(
                                Pos2::new(cell.right() - 8.0, cell.center().y),
                                Align2::RIGHT_CENTER,
                                format::format_bytes(stats.alloc[id], unit, decimals, lang),
                                small_font.clone(),
                                theme::TEXT,
                            );
                        }
                        SortKey::Files => {
                            ui.painter().text(
                                Pos2::new(cell.right() - 8.0, cell.center().y),
                                Align2::RIGHT_CENTER,
                                format::format_count(stats.files[id], lang),
                                small_font.clone(),
                                theme::TEXT,
                            );
                        }
                        SortKey::Folders => {
                            ui.painter().text(
                                Pos2::new(cell.right() - 8.0, cell.center().y),
                                Align2::RIGHT_CENTER,
                                format::format_count(stats.folders[id], lang),
                                small_font.clone(),
                                theme::TEXT,
                            );
                        }
                        SortKey::Percent => {
                            let bar = cell.shrink2(Vec2::new(5.0, 4.0));
                            icons::paint_percent_bar(
                                ui.painter(),
                                bar,
                                (pct_parent / 100.0) as f32,
                                is_selected,
                            );
                            ui.painter().text(
                                cell.center(),
                                Align2::CENTER_CENTER,
                                format::format_percent(pct_parent, lang),
                                small_font.clone(),
                                theme::TEXT,
                            );
                        }
                        SortKey::Modified => {
                            let text = node
                                .modified
                                .map(platform::format_time)
                                .unwrap_or_else(|| "—".to_string());
                            ui.painter().text(
                                Pos2::new(cell.right() - 8.0, cell.center().y),
                                Align2::RIGHT_CENTER,
                                text,
                                small_font.clone(),
                                theme::TEXT_MUTED,
                            );
                        }
                        SortKey::Name => {}
                    }
                    cx += col.width;
                }
            }
        });
    }

    if let Some(id) = pending_toggle {
        app.toggle_expanded(id);
    }
    if let Some(id) = pending_select {
        app.selected = Some(id);
    }
    if let Some(id) = pending_open {
        app.selected = Some(id);
        app.open_selected_location();
    }
    if let Some(id) = pending_explain {
        app.explain_node(id);
    }
    if let Some(id) = pending_delete {
        app.request_delete(id);
    }
}
