pub struct Text {
    data: String,
}

impl Text {
    pub fn new(data: String) -> Self {
        Self { data }
    }

    pub fn print(&mut self, data: String) {
        self.data = data;
    }

    pub fn ui(&mut self, ui: &mut egui::Ui) {
        ui.label(&self.data);
    }
}
