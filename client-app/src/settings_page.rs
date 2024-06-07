
use std::sync::Arc;

use crate::app;
use eframe::egui::{self, mutex::Mutex, Color32};

pub struct SettingsPage {
    name: String,
    devices: Arc<Mutex<Vec<app::IoTDevice>>>,
    new_device_name: String,
}

impl SettingsPage {
    pub fn new(devices: Arc<Mutex<Vec<app::IoTDevice>>>) -> Self {
        Self {
            name: "🔧Settings".to_string(),
            devices,
            new_device_name: "New device".to_string(),
        }
    }
}

impl app::AppPage for SettingsPage {
    fn name(&self) -> &str {
        &self.name
    }

    fn show(&mut self, _ctx: &eframe::egui::Context) {
        egui::CentralPanel::default().show(_ctx, |ui| {
            ui.heading("Devices");
            let mut devices = self.devices.lock();
            let mut remove: isize = -1;
            for (i, device) in devices.iter_mut().enumerate() {
                let name;
                match device.ws_state{
                    app::IoTDeviceWsState::Disconnect => {name = egui::RichText::new(&device.name).color(Color32::RED)},
                    app::IoTDeviceWsState::Pending => {name = egui::RichText::new(&device.name).color(Color32::YELLOW)},
                    app::IoTDeviceWsState::Connected => {name = egui::RichText::new(&device.name).color(Color32::GREEN)},
                }
                ui.collapsing(name, |ui| {
                    ui.horizontal(|ui| {
                        ui.label("IP:");
                        ui.add_enabled_ui(device.ws_state == app::IoTDeviceWsState::Disconnect, |ui|{
                            ui.text_edit_singleline(&mut device.ip);
                        });
                    });
                    ui.horizontal(|ui| {
                        if ui.button("Remove").clicked() {
                            remove = i as isize;
                        }
                        if device.ws_state == app::IoTDeviceWsState::Disconnect{
                            if ui.button("Connect").clicked() {
                                device.connect(_ctx.clone());
                            }
                        }
                        else if device.ws_state == app::IoTDeviceWsState::Connected{
                            if ui.button("Disconnect").clicked() {
                                device.disconnect();
                            }
                        }
                    });
                    if let Some(error) = &device.error {
                        ui.label(egui::RichText::new(error).color(Color32::RED));
                    }
                });
            }
            if remove >= 0 {
                devices.remove(remove as usize);
            }
            let unique_device_name = devices.iter().all(|d| d.name != self.new_device_name);
            ui.horizontal(|ui| {
                ui.add_enabled_ui(unique_device_name, |ui|{
                    if ui.button("Add device").clicked() {
                        devices.push(app::IoTDevice::new(&self.new_device_name));
                    }
                });
                ui.text_edit_singleline(&mut self.new_device_name);
            });
            if !unique_device_name {
                ui.label(egui::RichText::new("Name already exists").color(Color32::RED));
            }
        });
    }
}
