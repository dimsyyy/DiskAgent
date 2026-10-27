use eframe::egui::{
    self, Align2, Button, RichText, ScrollArea, Sense, TextEdit, Vec2,
};

use crate::app::DiskAgentApp;
use crate::filter::Filter;
use crate::format;
use crate::theme;

pub fn show(ctx: &egui::Context, app: &mut DiskAgentApp) {
    if app.filter_open {
        filter_window(ctx, app);
    }
    if app.columns_open {
        columns_window(ctx, app);
    }
    if app.new_folder_open {
        new_folder_window(ctx, app);
    }
    if app.delete_open {
        delete_window(ctx, app);
    }
    if app.search_open {
        search_window(ctx, app);
    }
}

fn filter_window(ctx: &egui::Context, app: &mut DiskAgentApp) {
    let mut open = true;
    egui::Window::new(app.t("Filter files"))
        .open(&mut open)
        .collapsible(false)
        .resizable(false)
        .anchor(Align2::CENTER_CENTER, Vec2::ZERO)
        .show(ctx, |ui| {
            ui.set_min_width(420.0);
            ui.add_space(4.0);
            ui.label(RichText::new(app.t("Include (masks, separated by ;)")).size(11.5));
            ui.add(
                TextEdit::singleline(&mut app.filter_draft.include)
                    .hint_text("*.iso; *.zip")
                    .desired_width(f32::INFINITY),
            );
            ui.add_space(6.0);
            ui.label(RichText::new(app.t("Exclude (masks, separated by ;)")).size(11.5));
            ui.add(
                TextEdit::singleline(&mut app.filter_draft.exclude)
                    .hint_text("*.tmp; Thumbs.db")
                    .desired_width(f32::INFINITY),
            );
            ui.add_space(4.0);
            ui.label(
                RichText::new(app.t("The mask field uses * and ? wildcards"))
                    .size(10.0)
                    .color(theme::TEXT_DIM),
            );
            ui.add_space(8.0);
            egui::Grid::new("filter-grid")
                .num_columns(2)
                .spacing(Vec2::new(10.0, 6.0))
                .show(ui, |ui| {
                    ui.label(app.t("Minimum size (MB)"));
                    ui.add(
                        egui::DragValue::new(&mut app.filter_min_mb)
                            .speed(0.5)
                            .range(0.0..=1_000_000.0),
                    );
                    ui.end_row();
                    ui.label(app.t("Maximum age (days, 0 = off)"));
                    ui.add(
                        egui::DragValue::new(&mut app.filter_max_age)
                            .speed(1.0)
                            .range(0..=36_500),
                    );
                    ui.end_row();
                });
            ui.add_space(4.0);
            let hide_empty_label = app.t("Hide empty folders");
            ui.checkbox(&mut app.filter_hide_empty, hide_empty_label);
            ui.add_space(10.0);
            ui.horizontal(|ui| {
                if ui
                    .add(Button::new(app.t("Apply")).min_size(Vec2::new(80.0, 26.0)))
                    .clicked()
                {
                    apply_filter(app);
                }
                if ui.add(Button::new(app.t("Reset"))).clicked() {
                    app.filter_draft = Filter::default();
                    app.filter_min_mb = 0.0;
                    app.filter_max_age = 0;
                    app.filter_hide_empty = false;
                }
                if ui.add(Button::new(app.t("Cancel"))).clicked() {
                    app.filter_open = false;
                }
            });
        });
    if !open {
        app.filter_open = false;
    }
}

fn apply_filter(app: &mut DiskAgentApp) {
    app.filter = app.filter_draft.clone();
    app.filter.min_size = (app.filter_min_mb.max(0.0) * 1024.0 * 1024.0) as u64;
    app.filter.max_age_days = app.filter_max_age;
    app.hide_empty = app.filter_hide_empty;
    app.stats_dirty = true;
    app.rows_dirty = true;
    app.filter_open = false;
    app.settings_dirty = true;
}

fn columns_window(ctx: &egui::Context, app: &mut DiskAgentApp) {
    let mut open = true;
    egui::Window::new(app.t("Configure columns"))
        .open(&mut open)
        .collapsible(false)
        .resizable(false)
        .anchor(Align2::CENTER_CENTER, Vec2::ZERO)
        .show(ctx, |ui| {
            ui.set_min_width(250.0);
            ui.add_space(2.0);
            ui.label(
                RichText::new(format!("{}: {}", app.t("Name"), app.t("All")))
                    .size(10.5)
                    .color(theme::TEXT_DIM),
            );
            ui.add_space(4.0);
            let l_size = app.t("Size");
            let l_alloc = app.t("Allocated Space");
            let l_files = app.t("File Count");
            let l_folders = app.t("Folders");
            let l_percent = app.t("Percent");
            let l_modified = app.t("Last Modified");
            ui.checkbox(&mut app.columns.size, l_size);
            ui.checkbox(&mut app.columns.alloc, l_alloc);
            ui.checkbox(&mut app.columns.files, l_files);
            ui.checkbox(&mut app.columns.folders, l_folders);
            ui.checkbox(&mut app.columns.percent, l_percent);
            ui.checkbox(&mut app.columns.modified, l_modified);
            ui.add_space(10.0);
            ui.horizontal(|ui| {
                if ui.add(Button::new(app.t("Reset to defaults"))).clicked() {
                    app.columns = crate::app::Columns::default();
                }
                if ui.add(Button::new(app.t("Close"))).clicked() {
                    app.columns_open = false;
                }
            });
            app.rows_dirty = true;
            app.settings_dirty = true;
        });
    if !open {
        app.columns_open = false;
    }
}

fn new_folder_window(ctx: &egui::Context, app: &mut DiskAgentApp) {
    let mut open = true;
    let parent = app
        .selected
        .and_then(|id| app.scan.as_ref().map(|tree| (id, &tree.nodes[id])))
        .map(|(_, node)| {
            if node.is_dir {
                node.path.clone()
            } else {
                node.parent
                    .and_then(|p| app.scan.as_ref().map(|t| t.nodes[p].path.clone()))
                    .unwrap_or_else(|| node.path.clone())
            }
        })
        .or_else(|| app.root_path.clone());

    egui::Window::new(app.t("New folder"))
        .open(&mut open)
        .collapsible(false)
        .resizable(false)
        .anchor(Align2::CENTER_CENTER, Vec2::ZERO)
        .show(ctx, |ui| {
            ui.set_min_width(360.0);
            if let Some(parent) = &parent {
                ui.label(
                    RichText::new(format!("{} {}", app.t("Parent:"), parent.display()))
                        .size(10.5)
                        .color(theme::TEXT_MUTED),
                );
            }
            ui.add_space(6.0);
            ui.label(app.t("Folder name"));
            let response = ui.add(
                TextEdit::singleline(&mut app.new_folder_name).desired_width(f32::INFINITY),
            );
            if app.new_folder_open && !response.has_focus() {
                response.request_focus();
            }
            let submitted =
                response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                if ui.button(app.t("Create")).clicked() || submitted {
                    app.create_folder();
                }
                if ui.button(app.t("Cancel")).clicked() {
                    app.new_folder_open = false;
                }
            });
        });
    if !open {
        app.new_folder_open = false;
    }
}

fn delete_window(ctx: &egui::Context, app: &mut DiskAgentApp) {
    let Some(id) = app.delete_target else {
        app.delete_open = false;
        return;
    };
    let info = app
        .scan
        .as_ref()
        .and_then(|tree| tree.nodes.get(id))
        .map(|node| (node.path.to_string_lossy().into_owned(), node.is_dir));
    let Some((path, is_dir)) = info else {
        app.cancel_delete();
        return;
    };

    let mut open = true;
    let mut confirmed = false;
    let mut cancelled = false;

    egui::Window::new(app.t("Delete permanently?"))
        .open(&mut open)
        .collapsible(false)
        .resizable(false)
        .anchor(Align2::CENTER_CENTER, Vec2::ZERO)
        .show(ctx, |ui| {
            ui.set_min_width(470.0);
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new(if is_dir {
                        app.t("Folder")
                    } else {
                        app.t("File")
                    })
                    .strong()
                    .color(theme::DANGER),
                );
                ui.label(
                    RichText::new(&path)
                        .size(11.0)
                        .color(theme::TEXT_MUTED),
                );
            });
            ui.add_space(6.0);
            ui.label(
                RichText::new(app.t(
                    "This item will be deleted permanently. This action cannot be undone.",
                ))
                .size(11.0)
                .color(theme::TEXT),
            );
            if is_dir {
                ui.add_space(2.0);
                ui.label(
                    RichText::new(app.t("Deleting a folder also deletes all of its contents."))
                        .size(11.0)
                        .color(theme::DANGER),
                );
            }
            ui.add_space(12.0);
            ui.horizontal(|ui| {
                let explain = if is_dir {
                    app.t("What is this folder?")
                } else {
                    app.t("What is this file?")
                };
                if ui.button(explain).clicked() {
                    app.explain_node(id);
                }
                let delete = Button::new(
                    RichText::new(app.t("Delete")).color(egui::Color32::WHITE),
                )
                .fill(theme::DANGER);
                if ui.add(delete).clicked() {
                    confirmed = true;
                }
                if ui.button(app.t("Cancel")).clicked() {
                    cancelled = true;
                }
            });
        });

    if confirmed {
        app.confirm_delete();
    } else if cancelled || !open || ctx.input(|input| input.key_pressed(egui::Key::Escape)) {
        app.cancel_delete();
    }
}

fn search_window(ctx: &egui::Context, app: &mut DiskAgentApp) {
    let mut open = true;
    egui::Window::new(app.t("Search in tree"))
        .open(&mut open)
        .collapsible(false)
        .resizable(true)
        .default_width(420.0)
        .anchor(Align2::CENTER_CENTER, Vec2::ZERO)
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                let hint = app.t("Type a name to search");
                let response = ui.add(
                    TextEdit::singleline(&mut app.search_text)
                        .hint_text(hint)
                        .desired_width(220.0),
                );
                if app.search_focus {
                    response.request_focus();
                    app.search_focus = false;
                }
                let submitted =
                    response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
                if ui.button(app.t("Search")).clicked() || submitted {
                    app.run_search();
                }
                if ui.button(app.t("Next result")).clicked() {
                    app.next_search_result();
                }
            });
            ui.add_space(6.0);
            let lang = app.lang;
            if app.search_hits.is_empty() {
                ui.label(
                    RichText::new(app.t("No matches"))
                        .size(11.0)
                        .color(theme::TEXT_DIM),
                );
            } else {
                ui.label(
                    RichText::new(format!(
                        "{} {}",
                        format::format_count(app.search_hits.len() as u64, lang),
                        app.t("Matches")
                    ))
                    .size(11.0)
                    .color(theme::TEXT_MUTED),
                );
                ui.separator();
                let hits = app.search_hits.clone();
                ScrollArea::vertical()
                    .max_height(220.0)
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        for id in hits.into_iter().take(100) {
                            let info = app.scan.as_ref().map(|tree| {
                                (
                                    tree.nodes[id].name.clone(),
                                    tree.nodes[id].path.to_string_lossy().into_owned(),
                                )
                            });
                            let Some((name, path)) = info else { continue };
                            let label = format!("{name}   —   {path}");
                            if ui
                                .selectable_label(
                                    app.selected == Some(id),
                                    RichText::new(label).size(11.0),
                                )
                                .clicked()
                            {
                                app.select_node(id, true);
                            }
                        }
                    });
            }
        });
    if !open {
        app.search_open = false;
    }
    let _ = Sense::click();
}
