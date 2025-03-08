use egui::{Window, Grid, DragValue, Slider, };
use egui_video::{Player};

pub struct Video {
    // audio_device: AudioDevice,
    player: Option<Player>,
    media_path: String,
    stream_size_scale: f32,
    seek_frac: f32,
}

impl Default for Video {
    fn default() -> Self {
        Self {
            // audio_device: AudioDevice::new().unwrap(),
            player: None,
            media_path: String::new(),
            stream_size_scale: 1.0,
            seek_frac: 0.0,
        }
    }
}

impl Video {
    pub fn ui(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
        if let Some(player) = self.player.as_mut() {
            Window::new("info").show(ctx, |ui| {
                Grid::new("info_grid").show(ui, |ui| {
                    ui.label("frame rate");
                    ui.label(player.framerate.to_string());
                    ui.end_row();

                    ui.label("size");
                    ui.label(format!("{}x{}", player.size.x, player.size.y));
                    ui.end_row();

                    ui.label("elapsed / duration");
                    ui.label(player.duration_text());
                    ui.end_row();

                    ui.label("state");
                    ui.label(format!("{:?}", player.player_state.get()));
                    ui.end_row();

                    ui.label("has audio?");
                    ui.label(player.audio_streamer.is_some().to_string());
                    ui.end_row();

                    ui.label("has subtitles?");
                    ui.label(player.subtitle_streamer.is_some().to_string());
                    ui.end_row();
                });
            });
            Window::new("controls").show(ctx, |ui| {
                ui.horizontal(|ui| {
                    if ui.button("seek to:").clicked() {
                        player.seek(self.seek_frac);
                    }
                    ui.add(
                        DragValue::new(&mut self.seek_frac)
                        .speed(0.05)
                        .range(0.0..=1.0),
                    );
                    ui.checkbox(&mut player.options.looping, "loop");
                });
                ui.horizontal(|ui| {
                    ui.label("size scale");
                    ui.add(Slider::new(&mut self.stream_size_scale, 0.0..=2.));
                });
                ui.separator();
                ui.horizontal(|ui| {
                    if ui.button("play").clicked() {
                        player.start()
                    }
                    if ui.button("unpause").clicked() {
                        player.resume();
                    }
                    if ui.button("pause").clicked() {
                        player.pause();
                    }
                    if ui.button("stop").clicked() {
                        player.stop();
                    }
                });
                ui.horizontal(|ui| {
                    ui.label("volume");
                    let mut volume = player.options.audio_volume.get();
                    if ui
                        .add(Slider::new(
                            &mut volume,
                            0.0..=player.options.max_audio_volume,
                        ))
                        .changed()
                        {
                            player.options.audio_volume.set(volume);
                        };
                });
            });

            player.ui(ui, player.size * self.stream_size_scale);
        }
    }

    pub fn set_media_path(&mut self, ctx: &egui::Context, media_path: String) {
        self.media_path = media_path;
        let player = egui_video::Player::new(ctx, &self.media_path).expect("can't create video player.");
        self.player = Some(player);
    }
}
