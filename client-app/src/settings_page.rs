
use crate::{
    app,
    connection_manager::ConnectionManager,
};
use eframe::egui;

pub struct SettingsPage {
}

impl SettingsPage {
    pub fn new() -> Self {
        Self {
        }
    }
}

impl app::AppPage for SettingsPage {
    fn name(&self) -> &str {"Settings"}

    fn emote(& self) -> &str {"🔧"}

    fn show(&mut self, _ctx: &eframe::egui::Context, connetions: &mut ConnectionManager) {
        egui::CentralPanel::default().show(_ctx, |ui| {
            connetions.show(ui);
        });
    }
}
