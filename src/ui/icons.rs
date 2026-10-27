use eframe::egui::{
    self, Align2, Color32, FontId, Painter, Pos2, Rect, Rounding, Shape, Stroke, Vec2,
};

use crate::theme;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[allow(dead_code)]
pub enum Icon {
    Folder,
    FolderPlus,
    Stop,
    Refresh,
    Size,
    Allocated,
    FileCount,
    Percent,
    AutoUnit,
    Expand,
    Filter,
    ClearFilter,
    HideEmpty,
    Search,
    Parent,
    Export,
    OpenLocation,
    Columns,
    Sort,
    Details,
    Treemap,
    Language,
    Shield,
    Drive,
    File,
    ChevronDown,
}

fn folder(painter: &Painter, rect: Rect) {
    let w = rect.width();
    let h = rect.height();
    let tab = Rect::from_min_max(
        Pos2::new(rect.left() + w * 0.06, rect.top() + h * 0.12),
        Pos2::new(rect.left() + w * 0.48, rect.top() + h * 0.32),
    );
    painter.rect_filled(tab, Rounding::same(1.5), theme::FOLDER_DARK);
    let body = Rect::from_min_max(
        Pos2::new(rect.left() + w * 0.04, rect.top() + h * 0.26),
        Pos2::new(rect.right() - w * 0.04, rect.bottom() - h * 0.08),
    );
    painter.rect_filled(body, Rounding::same(2.5), theme::FOLDER);
    painter.line_segment(
        [
            Pos2::new(body.left() + 1.0, body.top() + h * 0.10),
            Pos2::new(body.right() - 1.0, body.top() + h * 0.10),
        ],
        Stroke::new(1.0_f32, Color32::from_rgba_premultiplied(255, 255, 255, 60)),
    );
}

fn page(painter: &Painter, rect: Rect, color: Color32) {
    let w = rect.width();
    let h = rect.height();
    let fold = w * 0.28;
    let points = vec![
        Pos2::new(rect.left() + w * 0.22, rect.top() + h * 0.10),
        Pos2::new(rect.right() - w * 0.22 - fold, rect.top() + h * 0.10),
        Pos2::new(rect.right() - w * 0.22, rect.top() + h * 0.32),
        Pos2::new(rect.right() - w * 0.22, rect.bottom() - h * 0.10),
        Pos2::new(rect.left() + w * 0.22, rect.bottom() - h * 0.10),
    ];
    painter.add(Shape::convex_polygon(
        points,
        Color32::from_rgb(250, 251, 252),
        Stroke::new(1.2_f32, color),
    ));
    painter.line_segment(
        [
            Pos2::new(rect.right() - w * 0.22 - fold, rect.top() + h * 0.10),
            Pos2::new(rect.right() - w * 0.22 - fold, rect.top() + h * 0.32),
        ],
        Stroke::new(1.0_f32, color),
    );
    painter.line_segment(
        [
            Pos2::new(rect.right() - w * 0.22 - fold, rect.top() + h * 0.32),
            Pos2::new(rect.right() - w * 0.22, rect.top() + h * 0.32),
        ],
        Stroke::new(1.0_f32, color),
    );
}

fn arc_points(center: Pos2, radius: f32, start: f32, end: f32, steps: usize) -> Vec<Pos2> {
    (0..=steps)
        .map(|i| {
            let t = start + (end - start) * i as f32 / steps as f32;
            Pos2::new(center.x + radius * t.cos(), center.y + radius * t.sin())
        })
        .collect()
}

pub fn draw(painter: &Painter, icon: Icon, rect: Rect, color: Color32) {
    let r = rect.shrink(rect.width() * 0.06);
    let w = r.width();
    let h = r.height();
    let center = r.center();
    match icon {
        Icon::Folder => folder(painter, r),
        Icon::FolderPlus => {
            folder(painter, r);
            let c = Pos2::new(r.right() - w * 0.14, r.bottom() - h * 0.16);
            painter.line_segment(
                [Pos2::new(c.x - w * 0.10, c.y), Pos2::new(c.x + w * 0.10, c.y)],
                Stroke::new(2.0_f32, Color32::WHITE),
            );
            painter.line_segment(
                [Pos2::new(c.x, c.y - h * 0.10), Pos2::new(c.x, c.y + h * 0.10)],
                Stroke::new(2.0_f32, Color32::WHITE),
            );
        }
        Icon::Stop => {
            painter.circle_filled(center, w * 0.40, Color32::from_rgb(84, 121, 160));
            painter.rect_filled(
                Rect::from_center_size(center, Vec2::splat(w * 0.28)),
                Rounding::same(1.5),
                Color32::from_rgb(245, 247, 249),
            );
        }
        Icon::Refresh => {
            let stroke = Stroke::new(w * 0.085, color);
            let pts = arc_points(center, w * 0.32, -2.5, 1.4, 28);
            painter.add(Shape::line(pts, stroke));
            let end = Pos2::new(
                center.x + w * 0.32 * 1.4f32.cos(),
                center.y + w * 0.32 * 1.4f32.sin(),
            );
            painter.add(Shape::convex_polygon(
                vec![
                    Pos2::new(end.x - w * 0.12, end.y - w * 0.04),
                    Pos2::new(end.x + w * 0.10, end.y + w * 0.02),
                    Pos2::new(end.x - w * 0.02, end.y + w * 0.16),
                ],
                color,
                Stroke::NONE,
            ));
        }
        Icon::Size => {
            let base = r.bottom() - h * 0.08;
            for (i, frac) in [0.45f32, 0.70, 1.0].iter().enumerate() {
                let bw = w * 0.18;
                let x = r.left() + w * 0.16 + i as f32 * w * 0.25;
                let bh = h * 0.62 * frac;
                painter.rect_filled(
                    Rect::from_min_max(
                        Pos2::new(x, base - bh),
                        Pos2::new(x + bw, base),
                    ),
                    Rounding::same(1.0),
                    if i == 2 { color } else { theme::ICON_SOFT },
                );
            }
        }
        Icon::Allocated => {
            let drive = Rect::from_min_max(
                Pos2::new(r.left() + w * 0.05, r.top() + h * 0.28),
                Pos2::new(r.right() - w * 0.05, r.bottom() - h * 0.25),
            );
            painter.rect_filled(drive, Rounding::same(3.0), Color32::from_rgb(235, 240, 244));
            painter.rect_stroke(drive, Rounding::same(3.0), Stroke::new(1.4_f32, color));
            let led = Rect::from_min_size(
                Pos2::new(drive.left() + w * 0.08, drive.center().y - h * 0.03),
                Vec2::new(w * 0.20, h * 0.07),
            );
            painter.rect_filled(led, Rounding::same(1.0), Color32::from_rgb(64, 160, 96));
            painter.line_segment(
                [
                    Pos2::new(drive.left() + w * 0.46, drive.center().y),
                    Pos2::new(drive.right() - w * 0.10, drive.center().y),
                ],
                Stroke::new(1.8_f32, color),
            );
        }
        Icon::FileCount => {
            let back = Rect::from_min_max(
                Pos2::new(r.left() + w * 0.28, r.top() + h * 0.08),
                Pos2::new(r.right() - w * 0.08, r.bottom() - h * 0.26),
            );
            let front = Rect::from_min_max(
                Pos2::new(r.left() + w * 0.08, r.top() + h * 0.24),
                Pos2::new(r.right() - w * 0.28, r.bottom() - h * 0.08),
            );
            painter.rect_filled(back, Rounding::same(2.0), Color32::from_rgb(226, 236, 245));
            painter.rect_stroke(back, Rounding::same(2.0), Stroke::new(1.0_f32, theme::ICON_SOFT));
            painter.rect_filled(front, Rounding::same(2.0), Color32::from_rgb(247, 250, 252));
            painter.rect_stroke(front, Rounding::same(2.0), Stroke::new(1.4_f32, color));
            for i in 0..3 {
                let y = front.top() + h * 0.20 + i as f32 * h * 0.18;
                painter.line_segment(
                    [
                        Pos2::new(front.left() + w * 0.10, y),
                        Pos2::new(front.right() - w * 0.10, y),
                    ],
                    Stroke::new(1.2_f32, theme::ICON_SOFT),
                );
            }
        }
        Icon::Percent => {
            painter.text(
                center,
                Align2::CENTER_CENTER,
                "%",
                FontId::proportional(h * 0.92),
                color,
            );
        }
        Icon::AutoUnit => {
            let tri = vec![
                Pos2::new(r.left() + w * 0.14, r.bottom() - h * 0.18),
                Pos2::new(r.right() - w * 0.16, r.center().y),
                Pos2::new(r.left() + w * 0.14, r.top() + h * 0.18),
            ];
            painter.add(Shape::convex_polygon(
                tri,
                Color32::from_rgb(226, 236, 245),
                Stroke::new(1.6_f32, color),
            ));
            for i in 0..3 {
                let y = r.top() + h * 0.26 + i as f32 * h * 0.22;
                painter.line_segment(
                    [
                        Pos2::new(r.right() - w * 0.30, y),
                        Pos2::new(r.right() - w * 0.10, y),
                    ],
                    Stroke::new(1.3_f32, color),
                );
            }
        }
        Icon::Expand => {
            let x = r.left() + w * 0.24;
            painter.line_segment(
                [
                    Pos2::new(x, r.top() + h * 0.12),
                    Pos2::new(x, r.bottom() - h * 0.14),
                ],
                Stroke::new(1.6_f32, color),
            );
            for y in [0.26, 0.50, 0.74] {
                let yy = r.top() + h * y;
                painter.line_segment(
                    [Pos2::new(x, yy), Pos2::new(x + w * 0.22, yy)],
                    Stroke::new(1.4_f32, color),
                );
                painter.rect_filled(
                    Rect::from_center_size(
                        Pos2::new(x + w * 0.30, yy),
                        Vec2::splat(w * 0.10),
                    ),
                    Rounding::same(1.0),
                    theme::ICON_SOFT,
                );
            }
        }
        Icon::Filter => {
            let pts = vec![
                Pos2::new(r.left() + w * 0.10, r.top() + h * 0.20),
                Pos2::new(r.right() - w * 0.10, r.top() + h * 0.20),
                Pos2::new(r.center().x + w * 0.10, r.center().y + h * 0.02),
                Pos2::new(r.center().x + w * 0.10, r.bottom() - h * 0.14),
                Pos2::new(r.center().x - w * 0.02, r.bottom() - h * 0.24),
                Pos2::new(r.center().x - w * 0.02, r.center().y + h * 0.02),
            ];
            painter.add(Shape::convex_polygon(
                pts,
                Color32::from_rgb(226, 236, 245),
                Stroke::new(1.5_f32, color),
            ));
        }
        Icon::ClearFilter => {
            draw(painter, Icon::Filter, rect, color);
            let c = Pos2::new(r.right() - w * 0.18, r.bottom() - h * 0.18);
            let s = w * 0.14;
            painter.line_segment(
                [
                    Pos2::new(c.x - s, c.y - s),
                    Pos2::new(c.x + s, c.y + s),
                ],
                Stroke::new(2.2_f32, Color32::from_rgb(196, 60, 60)),
            );
            painter.line_segment(
                [
                    Pos2::new(c.x + s, c.y - s),
                    Pos2::new(c.x - s, c.y + s),
                ],
                Stroke::new(2.2_f32, Color32::from_rgb(196, 60, 60)),
            );
        }
        Icon::HideEmpty => {
            folder(painter, r);
            let c = Pos2::new(r.center().x + w * 0.10, r.center().y + h * 0.10);
            painter.circle_filled(c, w * 0.16, Color32::from_rgb(250, 251, 252));
            painter.line_segment(
                [
                    Pos2::new(c.x - w * 0.09, c.y),
                    Pos2::new(c.x + w * 0.09, c.y),
                ],
                Stroke::new(2.0_f32, Color32::from_rgb(196, 60, 60)),
            );
        }
        Icon::Search => {
            painter.circle_stroke(center, w * 0.28, Stroke::new(1.8_f32, color));
            painter.line_segment(
                [
                    Pos2::new(center.x + w * 0.20, center.y + h * 0.20),
                    Pos2::new(r.right() - w * 0.12, r.bottom() - h * 0.12),
                ],
                Stroke::new(2.2_f32, color),
            );
        }
        Icon::Parent => {
            painter.line_segment(
                [
                    Pos2::new(center.x, r.bottom() - h * 0.14),
                    Pos2::new(center.x, r.top() + h * 0.30),
                ],
                Stroke::new(1.8_f32, color),
            );
            painter.add(Shape::convex_polygon(
                vec![
                    Pos2::new(center.x, r.top() + h * 0.12),
                    Pos2::new(center.x - w * 0.18, r.top() + h * 0.34),
                    Pos2::new(center.x + w * 0.18, r.top() + h * 0.34),
                ],
                color,
                Stroke::NONE,
            ));
        }
        Icon::Export => {
            page(painter, r, color);
            painter.line_segment(
                [
                    Pos2::new(center.x + w * 0.12, r.top() + h * 0.52),
                    Pos2::new(center.x + w * 0.12, r.bottom() - h * 0.12),
                ],
                Stroke::new(2.0_f32, color),
            );
            painter.add(Shape::convex_polygon(
                vec![
                    Pos2::new(center.x + w * 0.12, r.bottom() - h * 0.06),
                    Pos2::new(center.x + w * 0.02, r.bottom() - h * 0.20),
                    Pos2::new(center.x + w * 0.22, r.bottom() - h * 0.20),
                ],
                color,
                Stroke::NONE,
            ));
        }
        Icon::OpenLocation => {
            folder(painter, r);
            let c = Pos2::new(r.center().x + w * 0.14, r.center().y + h * 0.14);
            painter.circle_filled(c, w * 0.15, Color32::from_rgb(246, 249, 252));
            painter.circle_stroke(c, w * 0.15, Stroke::new(1.3_f32, color));
            painter.line_segment(
                [
                    Pos2::new(c.x + w * 0.08, c.y + h * 0.08),
                    Pos2::new(c.x + w * 0.24, c.y + h * 0.24),
                ],
                Stroke::new(2.0_f32, color),
            );
        }
        Icon::Columns => {
            let gap = w * 0.06;
            let cw = (w * 0.80 - gap * 2.0) / 3.0;
            for i in 0..3 {
                let x = r.left() + w * 0.10 + i as f32 * (cw + gap);
                let col = Rect::from_min_max(
                    Pos2::new(x, r.top() + h * 0.16),
                    Pos2::new(x + cw, r.bottom() - h * 0.14),
                );
                painter.rect_filled(col, Rounding::same(1.5), Color32::from_rgb(234, 241, 247));
                painter.rect_stroke(col, Rounding::same(1.5), Stroke::new(1.2_f32, color));
                painter.line_segment(
                    [
                        Pos2::new(col.left() + 1.0, col.top() + h * 0.12),
                        Pos2::new(col.right() - 1.0, col.top() + h * 0.12),
                    ],
                    Stroke::new(1.4_f32, color),
                );
            }
        }
        Icon::Sort => {
            painter.text(
                Pos2::new(r.left() + w * 0.30, r.top() + h * 0.40),
                Align2::CENTER_CENTER,
                "A",
                FontId::proportional(h * 0.34),
                color,
            );
            painter.text(
                Pos2::new(r.left() + w * 0.52, r.top() + h * 0.68),
                Align2::CENTER_CENTER,
                "Z",
                FontId::proportional(h * 0.34),
                theme::ICON_SOFT,
            );
            painter.line_segment(
                [
                    Pos2::new(r.right() - w * 0.22, r.top() + h * 0.16),
                    Pos2::new(r.right() - w * 0.22, r.bottom() - h * 0.20),
                ],
                Stroke::new(1.6_f32, color),
            );
            painter.add(Shape::convex_polygon(
                vec![
                    Pos2::new(r.right() - w * 0.22, r.bottom() - h * 0.10),
                    Pos2::new(r.right() - w * 0.34, r.bottom() - h * 0.26),
                    Pos2::new(r.right() - w * 0.10, r.bottom() - h * 0.26),
                ],
                color,
                Stroke::NONE,
            ));
        }
        Icon::Details => {
            for i in 0..3 {
                let y = r.top() + h * 0.24 + i as f32 * h * 0.22;
                painter.rect_filled(
                    Rect::from_center_size(Pos2::new(r.left() + w * 0.18, y), Vec2::splat(w * 0.07)),
                    Rounding::same(1.0),
                    color,
                );
                painter.line_segment(
                    [
                        Pos2::new(r.left() + w * 0.32, y),
                        Pos2::new(r.right() - w * 0.12, y),
                    ],
                    Stroke::new(1.6_f32, color),
                );
            }
        }
        Icon::Treemap => {
            let colors = [
                Color32::from_rgb(78, 121, 167),
                Color32::from_rgb(242, 142, 43),
                Color32::from_rgb(225, 87, 89),
                Color32::from_rgb(118, 183, 178),
            ];
            let rects = [
                Rect::from_min_max(
                    Pos2::new(r.left() + w * 0.08, r.top() + h * 0.10),
                    Pos2::new(r.left() + w * 0.56, r.top() + h * 0.58),
                ),
                Rect::from_min_max(
                    Pos2::new(r.left() + w * 0.58, r.top() + h * 0.10),
                    Pos2::new(r.right() - w * 0.08, r.top() + h * 0.34),
                ),
                Rect::from_min_max(
                    Pos2::new(r.left() + w * 0.58, r.top() + h * 0.36),
                    Pos2::new(r.right() - w * 0.08, r.top() + h * 0.58),
                ),
                Rect::from_min_max(
                    Pos2::new(r.left() + w * 0.08, r.top() + h * 0.60),
                    Pos2::new(r.right() - w * 0.08, r.bottom() - h * 0.08),
                ),
            ];
            for (rect, col) in rects.iter().zip(colors) {
                painter.rect_filled(*rect, Rounding::same(1.0), col);
            }
        }
        Icon::Language => {
            painter.circle_stroke(center, w * 0.36, Stroke::new(1.5_f32, color));
            painter.line_segment(
                [
                    Pos2::new(center.x - w * 0.36, center.y),
                    Pos2::new(center.x + w * 0.36, center.y),
                ],
                Stroke::new(1.2_f32, color),
            );
            let pts = arc_points(center, w * 0.36, 0.0, std::f32::consts::TAU, 36);
            let _ = pts;
            painter.add(Shape::line(
                arc_points(center, w * 0.18, -1.57, 1.57, 18)
                    .into_iter()
                    .chain(arc_points(center, w * 0.18, 1.57, 4.71, 18))
                    .collect(),
                Stroke::new(1.1_f32, color),
            ));
        }
        Icon::Shield => {
            let pts = vec![
                Pos2::new(center.x, r.top() + h * 0.08),
                Pos2::new(r.right() - w * 0.14, r.top() + h * 0.24),
                Pos2::new(r.right() - w * 0.18, r.bottom() - h * 0.28),
                Pos2::new(center.x, r.bottom() - h * 0.06),
                Pos2::new(r.left() + w * 0.18, r.bottom() - h * 0.28),
                Pos2::new(r.left() + w * 0.14, r.top() + h * 0.24),
            ];
            painter.add(Shape::convex_polygon(
                pts,
                Color32::from_rgb(235, 242, 248),
                Stroke::new(1.4_f32, color),
            ));
            painter.text(
                center,
                Align2::CENTER_CENTER,
                "A",
                FontId::proportional(h * 0.34),
                color,
            );
        }
        Icon::Drive => {
            let drive = Rect::from_center_size(center, Vec2::new(w * 0.78, h * 0.46));
            painter.rect_filled(drive, Rounding::same(2.0), Color32::from_rgb(232, 238, 243));
            painter.rect_stroke(drive, Rounding::same(2.0), Stroke::new(1.2_f32, theme::ICON_SOFT));
            painter.rect_filled(
                Rect::from_min_size(
                    Pos2::new(drive.left() + w * 0.08, drive.center().y - h * 0.02),
                    Vec2::new(w * 0.18, h * 0.04),
                ),
                Rounding::same(1.0),
                Color32::from_rgb(80, 170, 110),
            );
        }
        Icon::File => page(painter, r, color),
        Icon::ChevronDown => {
            painter.add(Shape::convex_polygon(
                vec![
                    Pos2::new(center.x - w * 0.18, center.y - h * 0.08),
                    Pos2::new(center.x + w * 0.18, center.y - h * 0.08),
                    Pos2::new(center.x, center.y + h * 0.12),
                ],
                color,
                Stroke::NONE,
            ));
        }
    }
}

pub fn paint_expander(painter: &Painter, rect: Rect, expanded: bool, color: Color32) {
    let c = rect.center();
    let s = rect.width() * 0.24;
    let points = if expanded {
        vec![
            Pos2::new(c.x - s, c.y - s * 0.5),
            Pos2::new(c.x + s, c.y - s * 0.5),
            Pos2::new(c.x, c.y + s * 0.8),
        ]
    } else {
        vec![
            Pos2::new(c.x - s * 0.5, c.y - s),
            Pos2::new(c.x - s * 0.5, c.y + s),
            Pos2::new(c.x + s * 0.8, c.y),
        ]
    };
    painter.add(Shape::convex_polygon(points, color, Stroke::NONE));
}

pub fn paint_progress_gradient(
    painter: &Painter,
    rect: Rect,
    fraction: f32,
    strong: bool,
) {
    if fraction <= 0.0 || rect.width() <= 0.0 {
        return;
    }
    let frac = fraction.clamp(0.0, 1.0);
    let width = rect.width() * frac;
    let base_alpha = if strong { 55 } else { 30 };
    let steps = 24;
    for i in 0..steps {
        let t0 = i as f32 / steps as f32;
        let t1 = (i + 1) as f32 / steps as f32;
        let x0 = rect.left() + width * t0;
        let x1 = rect.left() + width * t1;
        let alpha = (base_alpha as f32 * (1.0 - t0 * 0.55)) as u8;
        painter.rect_filled(
            Rect::from_min_max(
                Pos2::new(x0, rect.top() + 1.0),
                Pos2::new(x1 + 0.5, rect.bottom() - 1.0),
            ),
            Rounding::ZERO,
            Color32::from_rgba_unmultiplied(120, 185, 235, alpha),
        );
    }
}

pub fn paint_percent_bar(painter: &Painter, rect: Rect, fraction: f32, selected: bool) {
    let bg = if selected {
        Color32::from_rgb(232, 242, 252)
    } else {
        Color32::from_rgb(245, 248, 251)
    };
    painter.rect_filled(rect, Rounding::same(2.0), bg);
    let frac = fraction.clamp(0.0, 1.0);
    if frac > 0.0 {
        let filled = Rect::from_min_size(rect.min, Vec2::new(rect.width() * frac, rect.height()));
        let color = if selected {
            Color32::from_rgb(120, 175, 225)
        } else {
            Color32::from_rgb(166, 202, 232)
        };
        painter.rect_filled(filled, Rounding::same(2.0), color);
    }
}

pub fn ellipsis(text: &str, max_chars: usize) -> String {
    let mut chars: Vec<char> = text.chars().collect();
    if chars.len() <= max_chars {
        return text.to_string();
    }
    if max_chars <= 1 {
        return "…".to_string();
    }
    chars.truncate(max_chars - 1);
    let mut out: String = chars.into_iter().collect();
    out.push('…');
    out
}

pub fn text_width(ui: &egui::Ui, text: &str, font: FontId) -> f32 {
    ui.fonts(|f| f.layout_no_wrap(text.to_string(), font, Color32::WHITE).size().x)
}
