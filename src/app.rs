use egui::CollapsingHeader;
use std::fs;
use std::path::Path;

const HOME_PATH: &str = "/home/robin/Documents";

// struct Path {
//     full_path: String,
//     file_name: String,
//     extension: String,
// }

// impl From<&fs::Path> for Path {
//     fn from(value: &fs::Path) -> Self {
//         Self {
//             full_path: value.to_string(),
//             file_name: value.file_name(),
//             extension: value.extension(),
//         }
//     }
// }

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct TemplateApp {
    selected_path: Option<String>,
}

impl Default for TemplateApp {
    fn default() -> Self {
        Self {
            selected_path: None,
        }
    }
}

impl TemplateApp {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }

        Default::default()
    }

    fn main_component(&mut self, ui: &mut egui::Ui) {
        ui.columns(2, |columns| {
            ScrollArea::vertical()
            .id_salt("explorer")
            .show(&mut columns[0], |ui| {
                CollapsingHeader::new(HOME_PATH)
                .default_open(false)
                .show(ui, |ui| {
                    self.print_document(ui, HOME_PATH);
                });
            });
            ScrollArea::vertical()
            .id_salt("media_player")
            .show(&mut columns[1], |ui| {
                // self.resolve_file(ui);
            });
        });
    }

    fn print_document(&mut self, ui: &mut egui::Ui, path: &str) {
        let home_dir = fs::read_dir(path).unwrap();
        for entry in home_dir {
            let entry = entry.unwrap();
            let path = entry.path();
            if path.is_dir() {
                CollapsingHeader::new(path.file_name().unwrap().to_str().unwrap())
                .default_open(false)
                .show(ui, |ui| {
                    self.print_document(ui, &path.display().to_string());
                });
            } else {
                let file_name = path.file_name().unwrap().to_str().unwrap();
                if ui.label(file_name).clicked() {
                    // self.selected_path = Some(path.clone());
                    println!("{file_name}");
                }
            }
        }
    }

    fn resolve_file(&mut self, ui: &mut egui::Ui) {
        // if let Some(path) = self.selected_path {
        //     // match path.extension().unwrap().to_str().unwrap() {
        //     //     _ => ui.label(path.file_name().unwrap().to_str().unwrap()),
        //     // }
        // } else {
        //     ui.label("No file selected");
        // }
    }
}

use egui::ScrollArea;
impl eframe::App for TemplateApp {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            self.main_component(ui);
        });
    }
}

fn powered_by_egui_and_eframe(ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        ui.label("Powered by ");
        ui.hyperlink_to("egui", "https://github.com/emilk/egui");
        ui.label(" and ");
        ui.hyperlink_to(
            "eframe",
            "https://github.com/emilk/egui/tree/master/crates/eframe",
        );
        ui.label(".");
    });
}
