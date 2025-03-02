use soloud::*;
use std::thread::{self, JoinHandle};
use std::sync::{Arc, Mutex};
use std::sync::mpsc::{self, Sender, Receiver, TryRecvError};

#[derive(Debug)]
pub enum AudioControl {
    Play,
    Pause,
    Stop,
    Seek(usize),
    InfoTED,
    InfoSID,
}

pub struct AudioPlayer {
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
    pub fn play(&mut self, full_path: String, audio_file_name: String) {
        let (sender, receiver) = mpsc::channel::<AudioControl>();
        self.sender = Some(sender);

        let thread_sl = Arc::clone(&self.sl);
        self.audio_thread = Some(
            thread::spawn(|| music_handler_thread_runner(full_path, thread_sl, receiver))
        );
    }

    pub fn ui(&mut self, ui: &mut egui::Ui) {
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

    pub fn pause_play(&mut self, action: AudioControl) {
        if let Some(tx) = self.sender.as_mut() {
            let _ = tx.send(action);
            self.is_paused = !self.is_paused;
        };
    }

    pub fn stop(&mut self) {
        if let Some(tx) = self.sender.as_mut() {
            let _ = tx.send(AudioControl::Stop);
            self.sender = None;
            self.audio_file_name = "No audio playing".to_string();
        };
    }

    pub fn info_ted(&mut self) {
        if let Some(tx) = self.sender.as_mut() {
            let _ = tx.send(AudioControl::InfoTED);
        }
    }

    pub fn info_sid(&mut self) {
        if let Some(tx) = self.sender.as_mut() {
            let _ = tx.send(AudioControl::InfoSID);
        }
    }

    pub fn get_pause_play_button_label(&self) -> &'static str {
        if self.is_paused {
            "Play"
        } else {
            "Pause"
        }
    }

    pub fn get_pause_play_action(&self) -> AudioControl {
        if self.is_paused {
            AudioControl::Play
        } else {
            AudioControl::Pause
        }
    }
}

pub fn music_handler_thread_runner(full_path: String, thread_sl: Arc<Mutex<Soloud>>, receiver: Receiver<AudioControl>) {
    let mut wav = Wav::default();
    println!("{full_path}");
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
