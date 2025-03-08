use std::fs;

use egui::CollapsingHeader;

use crate::audio::Audio;
use crate::text::Text;
use crate::video::Video;

const HOME_PATH: &str = "/home/robin/";

#[derive(serde::Deserialize, serde::Serialize)]
struct Path {
    full_path: String,
    file_name: String,
    extension: String,
}

impl Path {
    fn new(full_path: String, file_name: String, extension: String) -> Self {
        Self {
            full_path,
            file_name,
            extension,
        }
    }
}

enum ActiveUi {
    Audio,
    Text,
    Video,
}

pub struct MediaBrowser {
    selected_path: Option<Path>,
    active_ui: ActiveUi,
    audio_player: Audio,
    text: Text,
    video: Video,
}

impl Default for MediaBrowser {
    fn default() -> Self {
        Self {
            selected_path: None,
            active_ui: ActiveUi::Text,
            audio_player: Audio::default(),
            text: Text::new("No file selected".to_string()),
            video: Video::default(),
        }
    }
}

impl MediaBrowser {
    /// Called once before the first frame.
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        // if let Some(storage) = cc.storage {
        //     return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        // }

        Default::default()
    }

    fn main_component(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
        ui.columns(2, |columns| {
            ScrollArea::vertical()
                .id_salt("explorer")
                .show(&mut columns[0], |ui| {
                    CollapsingHeader::new(HOME_PATH)
                        .default_open(false)
                        .show(ui, |ui| {
                            self.print_document(ctx, ui, HOME_PATH);
                        });
                });
            ScrollArea::vertical()
                .id_salt("media_player")
                .show(&mut columns[1], |ui| match self.active_ui {
                    ActiveUi::Audio => self.audio_player.ui(ui),
                    ActiveUi::Text => self.text.ui(ui),
                    ActiveUi::Video => self.video.ui(ctx, ui),
                });
        });
    }

    fn print_document(&mut self, ctx: &egui::Context, ui: &mut egui::Ui, path: &str) {
        let home_dir = fs::read_dir(path).unwrap();
        for entry in home_dir {
            let entry = entry.unwrap();
            let path = entry.path();
            if path.is_dir() {
                CollapsingHeader::new(path.file_name().unwrap().to_str().unwrap())
                    .default_open(false)
                    .show(ui, |ui| {
                        self.print_document(ctx, ui, &path.display().to_string());
                    });
            } else {
                let file_name = path.file_name().unwrap().to_str().unwrap();
                if ui.label(file_name).clicked() {
                    self.selected_path = Some(Path::new(
                        path.display().to_string(),
                        file_name.to_string(),
                        path.extension().unwrap().to_str().unwrap().to_string(),
                    ));

                    self.resolve_file(ctx, ui);
                }
            }
        }
    }

    fn resolve_file(&mut self, ctx: &egui::Context, _ui: &mut egui::Ui) {
        if let Some(path) = &self.selected_path {
            match path.extension.as_str() {
                "png" | "jpg" => {
                    self.active_ui = ActiveUi::Text;
                    self.text.print("Un mage d'Apple.".to_string());
                }
                "txt" => {
                    self.active_ui = ActiveUi::Text;
                    self.text.print("This is a text file, printing content to be done...".to_string());
                }
                "pdf" => {
                    self.active_ui = ActiveUi::Text;
                    self.text.print("Portable Document Format.".to_string());
                }
                "mp3" => {
                    self.active_ui = ActiveUi::Audio;
                    self.audio_player
                        .play(path.full_path.clone(), path.file_name.clone());
                }
                "webm" => {
                    self.active_ui = ActiveUi::Video;
                    self.video.set_media_path(ctx, path.full_path.clone());
                }
                _ => {
                    self.active_ui = ActiveUi::Text;
                    self.text.print("Format non supporté!".to_string());
                }
            }
        }
    }
}

use egui::ScrollArea;
impl eframe::App for MediaBrowser {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, _storage: &mut dyn eframe::Storage) {
        // eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                egui::widgets::global_theme_preference_buttons(ui);
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            self.main_component(ctx, ui);
        });
    }
}
