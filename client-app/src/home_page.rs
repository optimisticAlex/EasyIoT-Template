
use std::sync::Arc;

use crate::app;
use eframe::egui::{self, mutex::Mutex};

use messages;

pub struct HomePage {
    name: String,
    devices: Arc<Mutex<Vec<app::IoTDevice>>>,
}

impl HomePage {
    pub fn new(devices: Arc<Mutex<Vec<app::IoTDevice>>>) -> Self {
        Self {
            name: "🏠Home".to_string(),
            devices,
        }
    }
}

impl app::AppPage for HomePage {
    fn name(&self) -> &str {
        &self.name
    }

    fn show(&mut self, _ctx: &eframe::egui::Context) {
        egui::CentralPanel::default().show(_ctx, |ui| {
            let mut devices = self.devices.lock();
            for d in devices.iter_mut() {
                if d.ws_state == app::IoTDeviceWsState::Connected {
                    ui.horizontal(|ui|{
                        ui.label(&d.name);
                        if ui.button("On").clicked(){
                            d.send(&messages::Message::On);
                        }
                        if ui.button("Off").clicked(){
                            d.send(&messages::Message::Off);
                        }
                    });
                } else {
                    ui.label(format!("{} is not connected", d.name));
                }
            }
        });
    }
}
