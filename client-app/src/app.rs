
use std::{sync::Arc, vec};

use eframe::egui::{self, mutex::Mutex};
use ewebsock::{WsEvent, WsReceiver, WsSender};
use crate::{home_page::HomePage, settings_page::SettingsPage};

use messages;

pub struct ClientApp{
    pages: Vec<Box<dyn AppPage>>,
    current_page: usize,
    devices: Arc<Mutex<Vec<IoTDevice>>>,
    host_ip: String,
    initialized: bool,
}

impl Default for ClientApp{
    fn default() -> Self {
        let mut s = Self {
            pages: Vec::new(),
            current_page: 0,
            devices: Arc::new(Mutex::new(vec![
                IoTDevice::new("Easy IoT Device 1"),
                IoTDevice::new("Easy IoT Device 2"),
            ])),
            host_ip: String::new(),
            initialized: false,
        };
        s.pages.push(Box::new(HomePage::new(s.devices.clone())));
        s.pages.push(Box::new(SettingsPage::new(s.devices.clone())));
        s
    }
}

impl ClientApp{
    pub fn new(_cc: &eframe::CreationContext<'_>, ip: String) -> Self {
        // Customize egui here with cc.egui_ctx.set_fonts and cc.egui_ctx.set_visuals.
        // Restore app state using cc.storage (requires the "persistence" feature).
        // Use the cc.gl (a glow::Context) to create graphics shaders and buffers that you can use
        // for e.g. egui::PaintCallback.
        let mut s = Self::default();
        s.host_ip = ip;
        s
    }

    fn init(&mut self, ctx: &egui::Context){
        self.initialized = true;
        for device in self.devices.lock().iter_mut(){
            device.ip = self.host_ip.clone()+":420";
            device.connect(ctx.clone());
            break;
        }
    }
}

impl eframe::App for ClientApp{
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        if !self.initialized {self.init(ctx);}

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                egui::widgets::global_dark_light_mode_switch(ui);
                for (i, page) in self.pages.iter().enumerate() {
                    let name = egui::RichText::new(page.name()).heading();
                    if i == self.current_page {
                        ui.heading(name).highlight();
                    } 
                    else if ui.button(name).clicked() {
                        self.current_page = i;
                    }
                }
            });
        });
        egui::CentralPanel::default().show(ctx, |_ui| {
            self.pages[self.current_page].show(ctx);
        });

        let mut devices = self.devices.lock();
        for device in devices.iter_mut(){
            device.update();
        }
        drop(devices);
    }
}


pub trait AppPage {
    fn name(& self) -> &str;
    fn show(&mut self, ctx: &eframe::egui::Context);
}

#[derive(PartialEq)]
pub enum IoTDeviceWsState {
    Disconnect,
    Pending,
    Connected,
}

pub struct IoTDevice{
    pub name: String,
    pub ip: String,
    pub ws_sender: Option<WsSender>,
    pub ws_receiver: Option<WsReceiver>,
    pub ws_state: IoTDeviceWsState,
    pub error: Option<String>,
    pub inpox: Vec<ewebsock::WsMessage>
}

impl IoTDevice{
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            ip: "".to_string(),
            ws_sender: None,
            ws_receiver: None,
            ws_state: IoTDeviceWsState::Disconnect,
            error: None,
            inpox: Vec::new(),
        }
    }

    pub fn connect(&mut self, ctx: egui::Context){
        if self.ws_state != IoTDeviceWsState::Disconnect{
            self.error = Some("Already connected".to_string());
            return;
        }
        let url = format!("ws://{}", self.ip);
        let options = ewebsock::Options::default();
        let wakeup = move || ctx.request_repaint(); // wake up UI thread on new message
        //let _res = ewebsock::connect_with_wakeup(url, options, wakeup);
        match ewebsock::connect_with_wakeup(url, options, wakeup){
            Ok((sender, resiever)) => {
                self.ws_sender = Some(sender);
                self.ws_receiver = Some(resiever);
                self.ws_state = IoTDeviceWsState::Pending;
                self.error = None;
            }
            Err(e) => {
                self.error = Some(format!("Error: {}", e));
            }
        }
    }

    pub fn disconnect(&mut self){
        if self.ws_state != IoTDeviceWsState::Connected{
            self.error = Some("Already disconnected".to_string());
            return;
        }
        self.ws_sender = None;
        self.ws_receiver = None;
        self.ws_state = IoTDeviceWsState::Disconnect;
        self.error = None;
    }

    pub fn update(&mut self){
        if let Some(receiver) = &self.ws_receiver {
            while let Some(event) = receiver.try_recv() {
                match event{
                    WsEvent::Message(msg) => {
                        self.inpox.push(msg);
                        self.error = None;
                    },
                    WsEvent::Error(e) => {
                        self.error = Some(format!("Error: {}", e));
                    },
                    WsEvent::Closed => {
                        self.ws_state = IoTDeviceWsState::Disconnect;
                        self.error = None;
                    }
                    WsEvent::Opened => {
                        self.ws_state = IoTDeviceWsState::Connected;
                        self.error = None;
                    }
                }
            }
        }
    }

    pub fn send(&mut self, msg: &messages::Message){
        if let Some(sender) = &mut self.ws_sender {
            sender.send(ewebsock::WsMessage::Text(msg.to_str().to_string()));
        }
    }
}

