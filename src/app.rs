use egui::CollapsingHeader;
use std::fs;

const HOME_PATH: &str = "/home/robin/Music";

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

pub struct TemplateApp {
    selected_path: Option<Path>,
    audio_player: AudioPlayer,
}

impl Default for TemplateApp {
    fn default() -> Self {
        Self {
            selected_path: None,
            audio_player: AudioPlayer::default(),
        }
    }
}

impl TemplateApp {
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
                    self.audio_player.ui(ui);
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
                    self.selected_path = Some(Path::new(
                        path.display().to_string(),
                        file_name.to_string(),
                        path.extension().unwrap().to_str().unwrap().to_string(),
                    ));

                    self.resolve_file(ui);
                }
            }
        }
    }

    fn resolve_file(&mut self, ui: &mut egui::Ui) {
        if let Some(path) = &self.selected_path {
            match path.extension.as_str() {
                "png" | "jpg" => {ui.label("C'est une image");},
                "txt" => {ui.label("Basic text file");},
                "pdf" => {ui.label("Portable Document Format");},
                "mp3" => {self.audio_player.play(path)},
                _ => {println!("{}", path.file_name.as_str());},
            }
        } else {
            ui.label("No file selected");
        }
    }
}

use soloud::*;
use std::thread::{self, JoinHandle};
use std::sync::{Arc, Mutex};
use std::sync::mpsc::{self, Sender, Receiver, TryRecvError};

#[derive(Debug)]
enum AudioControl {
    Play,
    Pause,
    Stop,
    Seek(usize),
    InfoTED,
    InfoSID,
}

struct AudioPlayer {
    sl: Arc<Mutex<Soloud>>,
    wav: Wav,
    audio_file_name: String,
    is_paused: bool,
    sender: Option<Sender<AudioControl>>,
    audio_thread: Option<JoinHandle<()>>,
}

impl Default for AudioPlayer {
    fn default() -> Self {
        Self {
            sl: Arc::new(Mutex::new(Soloud::default().unwrap())),
            wav: Wav::default(),
            audio_file_name: "No audio playing".to_string(),
            is_paused: false,
            sender: None,
            audio_thread: None,
        }
    }
}

impl AudioPlayer {
    fn play(&mut self, audio_file_path: &Path) {
        self.audio_file_name = audio_file_path.file_name.clone();

        let full_path = audio_file_path.full_path.clone();
        println!("playing {full_path}");

        let (sender, receiver) = mpsc::channel::<AudioControl>();
        self.sender = Some(sender);

        let thread_sl = Arc::clone(&self.sl);
        self.audio_thread = Some(
            thread::spawn(|| music_handler_thread_runner(full_path, thread_sl, receiver))
        );
    }

    fn ui(&mut self, ui: &mut egui::Ui) {
        ui.label(&self.audio_file_name);
        // Pause/play.
        if ui.add(egui::Button::new(
            self.get_pause_play_button_label())
        ).clicked() {
            let action = self.get_pause_play_action();
            self.pause_play(action);
        }
        // Stop
        if ui.add(egui::Button::new("Stop")).clicked() {
            self.stop();
        }
        // Info.
        if ui.add(egui::Button::new("Infos TED")).clicked() {
            self.info_ted();
        }
        if ui.add(egui::Button::new("Infos SID")).clicked() {
            self.info_sid();
        }
    }

    fn pause_play(&mut self, action: AudioControl) {
        if let Some(tx) = self.sender.as_mut() {
            let _ = tx.send(action);
            self.is_paused = !self.is_paused;
        };
    }

    fn stop(&mut self) {
        if let Some(tx) = self.sender.as_mut() {
            let _ = tx.send(AudioControl::Stop);
            self.sender = None;
            self.audio_file_name = "No audio playing".to_string();
        };
    }

    fn info_ted(&mut self) {
        if let Some(tx) = self.sender.as_mut() {
            let _ = tx.send(AudioControl::InfoTED);
        }
    }

    fn info_sid(&mut self) {
        if let Some(tx) = self.sender.as_mut() {
            let _ = tx.send(AudioControl::InfoSID);
        }
    }

    fn get_pause_play_button_label(&self) -> &'static str {
        if self.is_paused {
            "Play"
        } else {
            "Pause"
        }
    }

    fn get_pause_play_action(&self) -> AudioControl {
        if self.is_paused {
            AudioControl::Play
        } else {
            AudioControl::Pause
        }
    }
}

fn music_handler_thread_runner(full_path: String, thread_sl: Arc<Mutex<Soloud>>, receiver: Receiver<AudioControl>) {
    let mut wav = Wav::default();
    wav.load(&std::path::Path::new(&full_path)).unwrap();
    let handler = thread_sl.lock().unwrap().play(&wav);
    thread_sl.lock().unwrap().voice_count();
    while thread_sl.lock().unwrap().voice_count() > 0 {
        std::thread::sleep(std::time::Duration::from_millis(100));
        match receiver.try_recv() {
            Ok(control) => {
                let mut sl = thread_sl.lock().unwrap();
                match control {
                    AudioControl::Play => sl.set_pause(handler, false),
                    AudioControl::Pause => sl.set_pause(handler, true),
                    AudioControl::Stop => sl.stop(handler),
                    AudioControl::Seek(_) => todo!(),
                    AudioControl::InfoTED => {
                        println!("----- TED");
                        for i in 0..=31 {
                            println!("{}", sl.info(handler, i));
                        }
                        println!("----- FIN");
                    },
                    AudioControl::InfoSID => {
                        println!("----- SID");
                        for i in 64..=69 {
                            println!("{}", sl.info(handler, i));
                        }
                        println!("----- FIN");
                    },
                }
            },
            Err(TryRecvError::Disconnected) => break,
            Err(TryRecvError::Empty) => {},
        }
    }
}

use egui::ScrollArea;
impl eframe::App for TemplateApp {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        // eframe::set_value(storage, eframe::APP_KEY, self);
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
