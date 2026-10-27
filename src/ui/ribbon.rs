use eframe::egui::{
    self, Align, Align2, Button, Color32, FontId, Layout, Pos2, Rect, Response, RichText, Sense,
    Stroke, Ui, Vec2,
};

use crate::app::{DiskAgentApp, SortKey, Tab};
use crate::format::Unit;
use crate::theme;
use crate::ui::icons::{self, Icon};

const BIG_W: f32 = 66.0;
const BIG_H: f32 = 60.0;

pub fn show(ui: &mut Ui, app: &mut DiskAgentApp) {
    address_bar(ui, app);
    tab_strip(ui, app);
    if !app.ribbon_collapsed {
        egui::Frame::none()
            .fill(theme::RIBBON_BODY)
            .inner_margin(egui::Margin::symmetric(6.0, 4.0))
            .show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.spacing_mut().item_spacing = Vec2::new(2.0, 0.0);
                ui.horizontal(|ui| match app.tab {
                    Tab::Home => home_tab(ui, app),
                    Tab::Scan => scan_tab(ui, app),
                    Tab::View => view_tab(ui, app),
                    Tab::Options => options_tab(ui, app),
                });
            });
    }

    // Popup for the Expand dropdown.
    if app.expand_menu_open {
        expand_menu(ui, app);
    }

    let rect = ui.max_rect();
    ui.painter().line_segment(
        [
            Pos2::new(rect.left(), rect.bottom() - 0.5),
            Pos2::new(rect.right(), rect.bottom() - 0.5),
        ],
        Stroke::new(1.0_f32, theme::BORDER),
    );
    ui.add_space(0.0);
}

fn address_bar(ui: &mut Ui, app: &mut DiskAgentApp) {
    ui.horizontal(|ui| {
        ui.add_space(4.0);
        let (logo_rect, _) = ui.allocate_exact_size(Vec2::splat(22.0), Sense::hover());
        ui.painter()
            .circle_filled(logo_rect.center(), 10.0, theme::GREEN);
        ui.painter().text(
            logo_rect.center(),
            Align2::CENTER_CENTER,
            "D",
            FontId::proportional(13.0),
            Color32::WHITE,
        );
        ui.add_space(2.0);

        let can_back = app.history_pos > 0;
        let can_fwd = app.history_pos + 1 < app.history.len();
        if nav_button(ui, false, can_back, app.t("Back")).clicked() {
            app.navigate_history(-1, ui.ctx());
        }
        if nav_button(ui, true, can_fwd, app.t("Forward")).clicked() {
            app.navigate_history(1, ui.ctx());
        }

        let right_space = if app.elevated { 30.0 } else { 58.0 };
        let path_width = (ui.available_width() - right_space).max(160.0);
        let (path_rect, path_resp) =
            ui.allocate_exact_size(Vec2::new(path_width, 23.0), Sense::click());
        ui.painter()
            .rect_filled(path_rect, egui::Rounding::same(3.0), Color32::WHITE);
        ui.painter().rect_stroke(
            path_rect,
            egui::Rounding::same(3.0),
            Stroke::new(1.0_f32, theme::BORDER),
        );
        let icon_rect = Rect::from_center_size(
            Pos2::new(path_rect.left() + 13.0, path_rect.center().y),
            Vec2::splat(15.0),
        );
        icons::draw(ui.painter(), Icon::Folder, icon_rect, theme::TEXT);
        let path_text = app
            .root_path
            .as_ref()
            .map(|p| p.to_string_lossy().into_owned())
            .or_else(|| app.settings.last_dir.clone())
            .unwrap_or_else(|| app.t("Select a folder").to_string());
        let available = (path_rect.width() - 40.0) / 6.2;
        let text = icons::ellipsis(&path_text, available.max(8.0) as usize);
        ui.painter().text(
            Pos2::new(path_rect.left() + 24.0, path_rect.center().y),
            Align2::LEFT_CENTER,
            text,
            FontId::proportional(12.0),
            theme::TEXT,
        );
        if path_resp.clicked() {
            app.select_directory(ui.ctx());
        }
        path_resp.on_hover_text(app.t("Select Directory"));

        if !app.elevated {
            let shield = small_tool_button(ui, Icon::Shield, app.t("Start as Administrator"));
            if shield.clicked() {
                app.request_admin_restart(ui.ctx());
            }
        }

        if small_tool_button(ui, Icon::ChevronDown, app.t("Ribbon")).clicked() {
            app.ribbon_collapsed = !app.ribbon_collapsed;
            app.settings_dirty = true;
        }
    });
    ui.add_space(1.0);
}

fn nav_button(ui: &mut Ui, forward: bool, enabled: bool, tip: &str) -> Response {
    let sense = if enabled { Sense::click() } else { Sense::hover() };
    let (rect, response) = ui.allocate_exact_size(Vec2::splat(22.0), sense);
    if enabled && response.hovered() {
        ui.painter()
            .rect_filled(rect.shrink(2.0), egui::Rounding::same(3.0), theme::ACCENT_SOFT);
    }
    let color = if enabled {
        theme::TEXT
    } else {
        Color32::from_rgb(195, 199, 203)
    };
    let stroke = Stroke::new(1.6_f32, color);
    let center = rect.center();
    let dir = if forward { 1.0_f32 } else { -1.0_f32 };
    let tail = Pos2::new(center.x + dir * 4.0, center.y - 4.0);
    let point = Pos2::new(center.x - dir * 3.0, center.y);
    let foot = Pos2::new(center.x + dir * 4.0, center.y + 4.0);
    ui.painter().line_segment([tail, point], stroke);
    ui.painter().line_segment([point, foot], stroke);
    response.on_hover_text(tip.to_string())
}

fn small_tool_button(ui: &mut Ui, icon: Icon, tip: &str) -> Response {
    let (rect, response) = ui.allocate_exact_size(Vec2::splat(24.0), Sense::click());
    if response.hovered() {
        ui.painter()
            .rect_filled(rect, egui::Rounding::same(3.0), theme::ACCENT_SOFT);
    }
    icons::draw(
        ui.painter(),
        icon,
        rect.shrink(3.0),
        if response.hovered() {
            theme::ACCENT
        } else {
            theme::TEXT_MUTED
        },
    );
    response.on_hover_text(tip.to_string())
}

fn tab_strip(ui: &mut Ui, app: &mut DiskAgentApp) {
    ui.horizontal(|ui| {
        ui.add_space(6.0);
        ui.spacing_mut().item_spacing.x = 1.0;
        let tabs = [
            (Tab::Home, app.t("Home")),
            (Tab::Scan, app.t("Scan")),
            (Tab::View, app.t("View")),
            (Tab::Options, app.t("Options")),
        ];
        for (tab, label) in tabs {
            let text_width = icons::text_width(ui, label, FontId::proportional(12.5));
            let width = text_width + 30.0;
            let (rect, response) =
                ui.allocate_exact_size(Vec2::new(width, 27.0), Sense::click());
            let selected = app.tab == tab;
            if selected {
                ui.painter().rect_filled(
                    Rect::from_min_max(
                        rect.min,
                        Pos2::new(rect.max.x, rect.max.y + 6.0),
                    ),
                    egui::Rounding {
                        nw: 3.0,
                        ne: 3.0,
                        sw: 0.0,
                        se: 0.0,
                    },
                    theme::TAB_ACTIVE,
                );
            } else if response.hovered() {
                ui.painter().rect_filled(
                    rect.shrink2(Vec2::new(2.0, 3.0)),
                    egui::Rounding::same(3.0),
                    Color32::from_rgb(232, 232, 232),
                );
            }
            ui.painter().text(
                rect.center(),
                Align2::CENTER_CENTER,
                label,
                FontId::proportional(12.5),
                if selected {
                    theme::ACCENT_DARK
                } else {
                    theme::TEXT
                },
            );
            if response.clicked() {
                app.tab = tab;
            }
        }

        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            ui.add_space(8.0);
            if let Some(tree) = &app.scan {
                let lang = app.lang;
                let unit = app.unit;
                let decimals = app.decimals;
                let total = tree.total_size();
                ui.label(
                    RichText::new(crate::format::format_bytes(total, unit, decimals, lang))
                        .size(11.0)
                        .color(theme::TEXT_MUTED),
                );
            }
        });
    });
}

// ----------------------------------------------------------------------
// Ribbon widgets
// ----------------------------------------------------------------------

fn group(ui: &mut Ui, caption: &str, width: f32, add: impl FnOnce(&mut Ui)) {
    ui.allocate_ui_with_layout(
        Vec2::new(width, 78.0),
        Layout::top_down(Align::Center),
        |ui| {
            ui.set_width(width);
            ui.spacing_mut().item_spacing = Vec2::new(2.0, 0.0);
            ui.allocate_ui_with_layout(
                Vec2::new(width, BIG_H),
                Layout::left_to_right(Align::Center),
                |ui| {
                    ui.spacing_mut().item_spacing = Vec2::new(1.0, 0.0);
                    add(ui);
                },
            );
            ui.add_space(2.0);
            ui.label(RichText::new(caption).size(10.0).color(theme::TEXT_DIM));
        },
    );
    separator(ui);
}

fn separator(ui: &mut Ui) {
    ui.add_space(3.0);
    let (rect, _) = ui.allocate_exact_size(Vec2::new(1.0, 62.0), Sense::hover());
    ui.painter().line_segment(
        [rect.center_top(), rect.center_bottom()],
        Stroke::new(1.0_f32, theme::BORDER),
    );
    ui.add_space(3.0);
}

fn big(
    ui: &mut Ui,
    icon: Icon,
    label: &str,
    active: bool,
    enabled: bool,
    tip: &str,
) -> Response {
    let (rect, response) = ui.allocate_exact_size(
        Vec2::new(BIG_W, BIG_H),
        if enabled { Sense::click() } else { Sense::hover() },
    );
    let hovered = response.hovered() && enabled;
    if active {
        ui.painter().rect_filled(
            rect.shrink(2.0),
            egui::Rounding::same(4.0),
            theme::ACCENT_SOFT,
        );
        ui.painter().rect_stroke(
            rect.shrink(2.0),
            egui::Rounding::same(4.0),
            Stroke::new(1.0_f32, Color32::from_rgb(160, 205, 240)),
        );
    } else if hovered {
        ui.painter().rect_filled(
            rect.shrink(2.0),
            egui::Rounding::same(4.0),
            Color32::from_rgb(232, 240, 248),
        );
    }

    let icon_color = if !enabled {
        theme::TEXT_DIM
    } else if active {
        theme::ACCENT
    } else {
        theme::TEXT
    };
    icons::draw(
        ui.painter(),
        icon,
        Rect::from_center_size(
            Pos2::new(rect.center().x, rect.top() + 20.0),
            Vec2::splat(26.0),
        ),
        icon_color,
    );

    let text_color = if !enabled {
        theme::TEXT_DIM
    } else if active {
        theme::ACCENT_DARK
    } else {
        theme::TEXT
    };
    let galley = ui.painter().layout(
        label.to_string(),
        FontId::proportional(10.8),
        text_color,
        BIG_W - 8.0,
    );
    let pos = Pos2::new(
        rect.center().x - galley.size().x * 0.5,
        rect.bottom() - galley.size().y - 2.0,
    );
    ui.painter().galley(pos, galley, text_color);
    if !tip.is_empty() {
        response.clone().on_hover_text(tip.to_string());
    }
    response
}

fn small_unit_button(ui: &mut Ui, label: &str, active: bool) -> Response {
    let (rect, response) = ui.allocate_exact_size(Vec2::new(35.0, 16.0), Sense::click());
    if active {
        ui.painter().rect_filled(
            rect,
            egui::Rounding::same(2.0),
            theme::ACCENT_SOFT,
        );
        ui.painter().rect_stroke(
            rect,
            egui::Rounding::same(2.0),
            Stroke::new(1.0_f32, theme::ACCENT),
        );
    } else if response.hovered() {
        ui.painter().rect_filled(
            rect,
            egui::Rounding::same(2.0),
            Color32::from_rgb(236, 241, 246),
        );
    }
    ui.painter().text(
        rect.center(),
        Align2::CENTER_CENTER,
        label,
        FontId::proportional(10.0),
        if active {
            theme::ACCENT_DARK
        } else {
            theme::TEXT
        },
    );
    response
}

fn unit_stack(ui: &mut Ui, app: &mut DiskAgentApp, units: &[(Unit, &str)], two_columns: bool) {
    if two_columns {
        ui.vertical(|ui| {
            ui.spacing_mut().item_spacing.y = 1.0;
            for row in units.chunks(2) {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 1.0;
                    for (unit, label) in row {
                        if small_unit_button(ui, label, app.unit == *unit).clicked() {
                            app.set_unit(*unit);
                        }
                    }
                });
            }
        });
    } else {
        ui.vertical(|ui| {
            ui.spacing_mut().item_spacing.y = 1.0;
            for (unit, label) in units {
                if small_unit_button(ui, label, app.unit == *unit).clicked() {
                    app.set_unit(*unit);
                }
            }
        });
    }
}

// ----------------------------------------------------------------------
// Tabs
// ----------------------------------------------------------------------

fn home_tab(ui: &mut Ui, app: &mut DiskAgentApp) {
    group(ui, app.t("Operations"), 208.0, |ui| {
        let has_scan = app.scan.is_some();
        let scanning = app.is_scanning();
        if big(
            ui,
            Icon::Folder,
            app.t("Select Directory"),
            false,
            true,
            app.t("Select a folder and start a scan"),
        )
        .clicked()
        {
            app.select_directory(ui.ctx());
        }
        if big(
            ui,
            Icon::Stop,
            app.t("Stop Scan"),
            false,
            scanning,
            app.t("Stop the current scan"),
        )
        .clicked()
        {
            app.stop_scan();
        }
        if big(
            ui,
            Icon::Refresh,
            app.t("Refresh"),
            false,
            has_scan && !scanning,
            app.t("Scan the current folder again"),
        )
        .clicked()
        {
            app.refresh(ui.ctx());
        }
    });

    group(ui, app.t("Mode"), 268.0, |ui| {
        column_toggle(ui, app, Icon::Size, "Size", |c| &mut c.size);
        column_toggle(ui, app, Icon::Allocated, "Allocated Space", |c| &mut c.alloc);
        column_toggle(ui, app, Icon::FileCount, "File Count", |c| &mut c.files);
        column_toggle(ui, app, Icon::Percent, "Percent", |c| &mut c.percent);
    });

    group(ui, app.t("Unit"), 118.0, |ui| {
        if big(
            ui,
            Icon::AutoUnit,
            app.t("Automatic Units"),
            app.unit == Unit::Auto,
            true,
            app.t("Automatically choose the best unit"),
        )
        .clicked()
        {
            app.set_unit(Unit::Auto);
        }
        unit_stack(
            ui,
            app,
            &[(Unit::GB, "GB"), (Unit::MB, "MB"), (Unit::KB, "KB")],
            false,
        );
    });

    group(ui, app.t("Folder Operations"), 140.0, |ui| {
        let has_scan = app.scan.is_some();
        let expand = big(
            ui,
            Icon::Expand,
            app.t("Expand"),
            false,
            has_scan,
            app.t("Expand the tree"),
        );
        if expand.clicked() {
            app.expand_menu_open = true;
            app.expand_menu_pos = expand.rect.left_bottom();
            app.expand_menu_just_opened = true;
        }
        if big(
            ui,
            Icon::FolderPlus,
            app.t("New Folder"),
            false,
            has_scan,
            app.t("Create a new folder here"),
        )
        .clicked()
        {
            app.start_new_folder();
        }
    });

    if !app.elevated {
        group(ui, app.t("Application"), 96.0, |ui| {
            if big(
                ui,
                Icon::Shield,
                app.t("Start as Admin"),
                false,
                true,
                app.t("Restart with administrator rights"),
            )
            .clicked()
            {
                app.request_admin_restart(ui.ctx());
            }
        });
    }
}

fn column_toggle(
    ui: &mut Ui,
    app: &mut DiskAgentApp,
    icon: Icon,
    label: &'static str,
    pick: impl Fn(&mut crate::app::Columns) -> &mut bool,
) {
    let active = *pick(&mut app.columns);
    if big(
        ui,
        icon,
        app.t(label),
        active,
        true,
        app.t(match label {
            "Size" => "Show the Size column",
            "Allocated Space" => "Show the Allocated Space column",
            "File Count" => "Show the File Count column",
            _ => "Show the Percent column",
        }),
    )
    .clicked()
    {
        let flag = pick(&mut app.columns);
        *flag = !*flag;
        app.rows_dirty = true;
        app.settings_dirty = true;
    }
}

fn scan_tab(ui: &mut Ui, app: &mut DiskAgentApp) {
    group(ui, app.t("Operations"), 208.0, |ui| {
        let has_scan = app.scan.is_some();
        let scanning = app.is_scanning();
        if big(ui, Icon::Folder, app.t("Select Directory"), false, true, "").clicked() {
            app.select_directory(ui.ctx());
        }
        if big(ui, Icon::Stop, app.t("Stop Scan"), false, scanning, "").clicked() {
            app.stop_scan();
        }
        if big(
            ui,
            Icon::Refresh,
            app.t("Refresh"),
            false,
            has_scan && !scanning,
            "",
        )
        .clicked()
        {
            app.refresh(ui.ctx());
        }
    });

    group(ui, app.t("Filter"), 210.0, |ui| {
        if big(
            ui,
            Icon::Filter,
            app.t("Filter"),
            app.filter.is_active(),
            app.scan.is_some(),
            app.t("Filter files and folders"),
        )
        .clicked()
        {
            app.filter_draft = app.filter.clone();
            app.filter_min_mb = app.filter.min_size as f64 / (1024.0 * 1024.0);
            app.filter_max_age = app.filter.max_age_days;
            app.filter_hide_empty = app.hide_empty;
            app.filter_open = true;
        }
        if big(
            ui,
            Icon::ClearFilter,
            app.t("Clear Filter"),
            false,
            app.filter.is_active(),
            app.t("Remove all filters"),
        )
        .clicked()
        {
            app.filter.clear();
            app.stats_dirty = true;
            app.rows_dirty = true;
            app.settings_dirty = true;
        }
        if big(
            ui,
            Icon::HideEmpty,
            app.t("Hide empty folders"),
            app.hide_empty,
            true,
            app.t("Hide folders without visible files"),
        )
        .clicked()
        {
            app.hide_empty = !app.hide_empty;
            app.stats_dirty = true;
            app.rows_dirty = true;
            app.settings_dirty = true;
        }
    });

    group(ui, app.t("Directory Tree"), 140.0, |ui| {
        let has_scan = app.scan.is_some();
        let expand = big(
            ui,
            Icon::Expand,
            app.t("Expand"),
            false,
            has_scan,
            app.t("Expand the tree"),
        );
        if expand.clicked() {
            app.expand_menu_open = true;
            app.expand_menu_pos = expand.rect.left_bottom();
            app.expand_menu_just_opened = true;
        }
        if big(
            ui,
            Icon::Search,
            app.t("Search Tree"),
            app.search_open,
            has_scan,
            app.t("Search for a folder in the tree"),
        )
        .clicked()
        {
            app.open_search();
        }
    });

    group(ui, app.t("Tools"), 208.0, |ui| {
        let has_scan = app.scan.is_some();
        if big(
            ui,
            Icon::Parent,
            app.t("Parent Folder"),
            false,
            has_scan,
            app.t("Go to parent folder"),
        )
        .clicked()
        {
            app.parent_folder();
        }
        if big(
            ui,
            Icon::Export,
            app.t("Export"),
            false,
            has_scan,
            app.t("Export visible rows as CSV"),
        )
        .clicked()
        {
            app.export_csv();
        }
        if big(
            ui,
            Icon::OpenLocation,
            app.t("Open Location"),
            false,
            has_scan,
            app.t("Open the selected item in Explorer"),
        )
        .clicked()
        {
            app.open_selected_location();
        }
    });
}

fn view_tab(ui: &mut Ui, app: &mut DiskAgentApp) {
    group(ui, app.t("Mode"), 268.0, |ui| {
        column_toggle(ui, app, Icon::Size, "Size", |c| &mut c.size);
        column_toggle(ui, app, Icon::Allocated, "Allocated Space", |c| &mut c.alloc);
        column_toggle(ui, app, Icon::FileCount, "File Count", |c| &mut c.files);
        column_toggle(ui, app, Icon::Percent, "Percent", |c| &mut c.percent);
    });

    group(ui, app.t("Unit"), 138.0, |ui| {
        if big(
            ui,
            Icon::AutoUnit,
            app.t("Automatic Units"),
            app.unit == Unit::Auto,
            true,
            app.t("Automatically choose the best unit"),
        )
        .clicked()
        {
            app.set_unit(Unit::Auto);
        }
        unit_stack(
            ui,
            app,
            &[
                (Unit::TB, "TB"),
                (Unit::GB, "GB"),
                (Unit::MB, "MB"),
                (Unit::KB, "KB"),
                (Unit::B, "B"),
            ],
            true,
        );
    });

    group(ui, app.t("Decimals"), 82.0, |ui| {
        egui::ComboBox::from_id_salt("decimals")
            .width(58.0)
            .selected_text(format!("{}", app.decimals))
            .show_ui(ui, |ui| {
                for value in 0..=3u8 {
                    if ui
                        .selectable_value(&mut app.decimals, value, format!("{value}"))
                        .clicked()
                    {
                        app.settings_dirty = true;
                    }
                }
            });
    });

    group(ui, app.t("Visible columns"), 92.0, |ui| {
        if big(
            ui,
            Icon::Columns,
            app.t("Configure columns"),
            false,
            true,
            app.t("Choose visible columns"),
        )
        .clicked()
        {
            app.columns_open = true;
        }
    });

    group(ui, app.t("Sort"), 118.0, |ui| {
        ui.vertical(|ui| {
            egui::ComboBox::from_id_salt("sort_key")
                .width(84.0)
                .selected_text(app.sort_key.label(app.lang))
                .show_ui(ui, |ui| {
                    for key in [
                        SortKey::Name,
                        SortKey::Size,
                        SortKey::Alloc,
                        SortKey::Files,
                        SortKey::Folders,
                        SortKey::Percent,
                        SortKey::Modified,
                    ] {
                        if ui
                            .selectable_label(app.sort_key == key, key.label(app.lang))
                            .clicked()
                        {
                            app.set_sort(key);
                        }
                    }
                });
            if ui
                .add(
                    Button::new(RichText::new(if app.sort_desc { "↓" } else { "↑" }).size(12.0))
                        .min_size(Vec2::new(84.0, 18.0)),
                )
                .clicked()
            {
                app.sort_desc = !app.sort_desc;
                app.rows_dirty = true;
                app.settings_dirty = true;
            }
        });
    });

    group(ui, app.t("Show or hide"), 140.0, |ui| {
        if big(
            ui,
            Icon::Details,
            app.t("Show Details"),
            app.view == crate::app::ContentView::Details,
            true,
            app.t("Show the details tree"),
        )
        .clicked()
        {
            app.view = crate::app::ContentView::Details;
            app.settings_dirty = true;
        }
        if big(
            ui,
            Icon::Treemap,
            app.t("Show Treemap Chart"),
            app.view == crate::app::ContentView::Treemap,
            true,
            app.t("Show the treemap chart"),
        )
        .clicked()
        {
            app.view = crate::app::ContentView::Treemap;
            if app.treemap_root.is_none() {
                app.treemap_root = app.scan.as_ref().map(|t| t.root);
            }
            app.settings_dirty = true;
        }
    });
}

fn options_tab(ui: &mut Ui, app: &mut DiskAgentApp) {
    group(ui, app.t("Language"), 330.0, |ui| {
        ui.vertical(|ui| {
            ui.add_space(4.0);
            egui::ComboBox::from_id_salt("language")
                .width(190.0)
                .selected_text(app.lang.label())
                .show_ui(ui, |ui| {
                    for lang in [crate::i18n::Lang::En, crate::i18n::Lang::Id] {
                        if ui
                            .selectable_label(app.lang == lang, lang.label())
                            .clicked()
                        {
                            app.lang = lang;
                            app.settings_dirty = true;
                        }
                    }
                });
            ui.add_space(4.0);
            ui.label(
                RichText::new(app.t("Language changes take effect immediately"))
                    .size(10.5)
                    .color(theme::TEXT_MUTED),
            );
        });
    });
}

fn expand_menu(ui: &mut Ui, app: &mut DiskAgentApp) {
    if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
        app.expand_menu_open = false;
        return;
    }

    let area = egui::Area::new(egui::Id::new("expand_menu"))
        .order(egui::Order::Foreground)
        .fixed_pos(app.expand_menu_pos)
        .show(ui.ctx(), |ui| {
            egui::Frame::popup(ui.style())
                .inner_margin(egui::Margin::same(4.0))
                .show(ui, |ui| {
                    ui.set_min_width(150.0);
                    for level in 1..=5u16 {
                        let label = format!("{} {level}", app.t("Level"));
                        if ui.add(Button::new(label).frame(false)).clicked() {
                            app.expand_levels(level);
                            app.expand_menu_open = false;
                        }
                    }
                    ui.separator();
                    if ui
                        .add(Button::new(app.t("Full expand")).frame(false))
                        .clicked()
                    {
                        app.expand_all();
                        app.expand_menu_open = false;
                    }
                    if ui
                        .add(Button::new(app.t("Collapse all")).frame(false))
                        .clicked()
                    {
                        app.collapse_all();
                        app.expand_menu_open = false;
                    }
                });
        });

    if !app.expand_menu_just_opened && ui.input(|i| i.pointer.any_click()) {
        if let Some(pos) = ui.input(|i| i.pointer.interact_pos()) {
            if !area.response.rect.contains(pos) {
                app.expand_menu_open = false;
            }
        }
    }
    app.expand_menu_just_opened = false;
}
