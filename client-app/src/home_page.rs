use crate::{
    app,
    connection_manager::ConnectionManager,
};
use messages::Message;
use eframe::egui;



pub struct HomePage {
    on: bool,
}

impl HomePage {
    pub fn new() -> Self {
        Self {
            on: false,
        }
    }
}

impl app::AppPage for HomePage {
    fn name(&self) -> &str {"Home"}

    fn emote(&self) -> &str {"🏠"}

    fn show(&mut self, _ctx: &eframe::egui::Context, connetions: &mut ConnectionManager) {
        egui::CentralPanel::default().show(_ctx, |ui| {
            ui.vertical_centered(|ui|{
                ui.add_space(ui.available_height()*0.42);
                if connetions.connection_count() == 0{
                    ui.label("No devices connected").highlight();
                }
                else{
                    if self.on{
                        if ui.button(egui::RichText::new("OFF").size(42.).color(egui::Color32::RED)).clicked(){
                            self.on = false;
                            connetions.send_to_all(&Message::Off);
                        }
                    }
                    else{
                        if ui.button(egui::RichText::new("ON").size(42.).color(egui::Color32::GREEN)).clicked(){
                            self.on = true;
                            connetions.send_to_all(&Message::On);
                        }
                    }
                }
            });
        });
    }
}
