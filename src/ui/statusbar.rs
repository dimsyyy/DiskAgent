use eframe::egui::{Align, Layout, RichText, Ui};

use crate::app::DiskAgentApp;
use crate::format;
use crate::theme;

pub fn show(ui: &mut Ui, app: &mut DiskAgentApp) {
    ui.horizontal_centered(|ui| {
        ui.add_space(6.0);
        let lang = app.lang;
        let unit = app.unit;
        let decimals = app.decimals;

        if let Some(tree) = &app.scan {
            let volume = &tree.volume;
            ui.label(
                RichText::new(format!(
                    "{}: {} ({} {})",
                    app.t("Free Space"),
                    format::format_bytes(volume.free_bytes, unit, decimals, lang),
                    app.t("of"),
                    format::format_bytes(volume.total_bytes, unit, decimals, lang)
                ))
                .size(11.0)
                .color(theme::TEXT_MUTED),
            );
            ui.separator();

            if let Some(stats) = &app.stats {
                let files_resp = ui.label(
                    RichText::new(format!(
                        "{}: {}",
                        app.t("Files"),
                        format::format_count(stats.matched_files, lang)
                    ))
                    .size(11.0)
                    .color(theme::TEXT_MUTED),
                );
                files_resp.on_hover_text(format!(
                    "{}: {} {} / {} {}",
                    app.t("Scanned"),
                    format::format_count(tree.scanned_files, lang),
                    app.t("files"),
                    format::format_count(tree.scanned_dirs, lang),
                    app.t("Folders")
                ));
                ui.separator();
                ui.label(
                    RichText::new(format!(
                        "{}: {}",
                        app.t("Folders"),
                        format::format_count(stats.folders[tree.root], lang)
                    ))
                    .size(11.0)
                    .color(theme::TEXT_MUTED),
                );
                if app.filter_active() {
                    ui.separator();
                    ui.label(
                        RichText::new(app.t("Filter active"))
                            .size(11.0)
                            .color(theme::ACCENT_DARK),
                    );
                }
                if stats.excluded_files > 0 {
                    ui.separator();
                    ui.label(
                        RichText::new(format!(
                            "{} {}",
                            format::format_count(stats.excluded_files, lang),
                            app.t("Excluded")
                        ))
                        .size(11.0)
                        .color(theme::TEXT_MUTED),
                    );
                }
            }

            if let Some((message, created)) = &app.status {
                if created.elapsed().as_secs() < 10 {
                    ui.separator();
                    ui.label(
                        RichText::new(message)
                            .size(11.0)
                            .color(theme::ACCENT_DARK),
                    );
                }
            }

            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                ui.add_space(6.0);
                if !volume.fs_name.is_empty() {
                    ui.label(
                        RichText::new(format!(
                            "{} {} ({})",
                            format::format_count(volume.cluster_size, lang),
                            app.t("Bytes per Cluster"),
                            volume.fs_name
                        ))
                        .size(11.0)
                        .color(theme::TEXT_DIM),
                    );
                }
                ui.separator();
                ui.label(
                    RichText::new(format::format_duration(tree.duration.as_secs_f64()))
                        .size(11.0)
                        .color(theme::TEXT_DIM),
                );
                if let Some(id) = app.selected {
                    if let Some(stats) = &app.stats {
                        ui.separator();
                        ui.label(
                            RichText::new(format!(
                                "{}: {} ({} {})",
                                app.t("Selected"),
                                format::format_bytes(stats.size[id], unit, decimals, lang),
                                format::format_percent(app.percent_of_total(id), lang),
                                app.t("of total")
                            ))
                            .size(11.0)
                            .color(theme::TEXT_DIM),
                        );
                    }
                }
            });
        } else {
            ui.label(
                RichText::new(format!("DiskAgent {}", env!("CARGO_PKG_VERSION")))
                    .size(11.0)
                    .color(theme::TEXT_DIM),
            );
        }
    });
}
