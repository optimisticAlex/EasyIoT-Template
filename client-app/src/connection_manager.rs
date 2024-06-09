use eframe::egui;
use messages::Message;
use ewebsock::{WsSender, WsReceiver, WsEvent};

pub struct ConnectionManager{
    devices: Vec<IoTDevice>,
    new_device_name: String,
}

impl Default for ConnectionManager{
    fn default() -> Self {
        Self{
            devices: Vec::new(),
            new_device_name: String::new(),
        }
    }
}

impl ConnectionManager{
    pub fn show(&mut self, ui: &mut egui::Ui){
        ui.label("Devices").highlight();
        let mut to_remove: isize = -1;
        for (i, device) in self.devices.iter_mut().enumerate(){
            let name;
            use egui::Color32;
            match device.ws_state{
                IoTDeviceWsState::Disconnect => {name = egui::RichText::new(&device.name).color(Color32::RED)},
                IoTDeviceWsState::Pending => {name = egui::RichText::new(&device.name).color(Color32::YELLOW)},
                IoTDeviceWsState::Connected => {name = egui::RichText::new(&device.name).color(Color32::GREEN)},
            }
            ui.collapsing(name, |ui| {
                ui.horizontal(|ui| {
                    ui.label("IP:");
                    ui.add_enabled_ui(device.ws_state == IoTDeviceWsState::Disconnect, |ui|{
                        ui.text_edit_singleline(&mut device.ip);
                    });
                });
                ui.horizontal(|ui| {
                    if ui.button("remove").clicked(){
                        to_remove = i as isize;
                    }
                    if device.ws_state == IoTDeviceWsState::Disconnect{
                        if ui.button("Connect").clicked() {
                            device.connect(ui.ctx().clone());
                        }
                    }
                    else if device.ws_state == IoTDeviceWsState::Connected{
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
        if to_remove >= 0{
            self.devices.remove(to_remove as usize);
        }
        let unique_name = self.devices.iter().all(|d| d.name != self.new_device_name) && !self.new_device_name.is_empty();
        ui.horizontal(|ui|{
            ui.add_enabled_ui(unique_name, |ui|{
                if ui.button("Add device").clicked(){
                    self.devices.push(IoTDevice::new(&self.new_device_name));
                    self.new_device_name.clear();
                }
            });
            ui.text_edit_singleline(&mut self.new_device_name);
        });
    }
    pub fn connection_count(&self) -> usize{
        let mut count = 0;
        for device in &self.devices{
            if device.ws_state == IoTDeviceWsState::Connected{
                count += 1;
            }
        }
        count
    }
    pub fn send_to_all(&mut self, msg: &Message){
        for device in &mut self.devices{
            if device.ws_state == IoTDeviceWsState::Connected{
                device.send(msg);
            }
        }
    }
    pub fn update(&mut self){
        for device in &mut self.devices{
            device.poll();
        }
    }
    pub fn connect_new_device(&mut self, name: &str, ip: &str, ctx: egui::Context){
        let mut d = IoTDevice::new(name);
        d.ip = ip.to_string();
        d.connect(ctx);
        self.devices.push(d);
    }
}

#[derive(PartialEq)]
enum IoTDeviceWsState {
    Disconnect,
    Pending,
    Connected,
}

struct IoTDevice{
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
        let url = format!("ws://{}:{}", self.ip, messages::MESSAGE_PORT);
        let options = ewebsock::Options::default();
        let wakeup = move || ctx.request_repaint(); // wake up UI thread on new message
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

    pub fn poll(&mut self){
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