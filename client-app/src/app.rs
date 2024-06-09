
use eframe::egui;
use crate::{
    home_page::HomePage, 
    settings_page::SettingsPage,
    connection_manager::ConnectionManager,
};

pub struct ClientApp{
    initialized: bool,
    host_address: String,
    pages: Vec<Box<dyn AppPage>>,
    current_page: usize,
    connections: ConnectionManager,
}

impl Default for ClientApp{
    fn default() -> Self {
        Self {
            initialized: false,
            host_address: String::new(),
            pages: vec![
                Box::new(HomePage::new()),
                Box::new(SettingsPage::new()),
            ],
            current_page: 0,
            connections: ConnectionManager::default(),
        }
    }
}

impl ClientApp{
    pub fn new(_cc: &eframe::CreationContext<'_>, ip: String) -> Self {
        // Customize egui here with cc.egui_ctx.set_fonts and cc.egui_ctx.set_visuals.
        // Restore app state using cc.storage (requires the "persistence" feature).
        // Use the cc.gl (a glow::Context) to create graphics shaders and buffers that you can use
        // for e.g. egui::PaintCallback.
        Self{
            host_address: ip,
            ..Default::default()
        }
    }
    fn init(&mut self, ctx: &egui::Context){
        self.initialized = true;
        self.connections.connect_new_device("EasyIoT", &self.host_address, ctx.clone());
    }
}

impl eframe::App for ClientApp{
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        if !self.initialized{self.init(ctx);}
        self.connections.update();
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                egui::widgets::global_dark_light_mode_switch(ui);
                for (i, page) in self.pages.iter().enumerate() {
                    let name = egui::RichText::new(format!(" {} {} ", page.emote(), page.name())).heading();
                    if i == self.current_page {
                        ui.heading(name).highlight();
                    } 
                    else if ui.button(name).clicked() {
                        self.current_page = i;
                    }
                }
                if !self.host_address.is_empty(){
                    let host = egui::RichText::new(format!("Host: {}", self.host_address)).small();
                    ui.with_layout(egui::Layout::right_to_left(egui::emath::Align::Center), |ui|{
                        ui.label(host);
                        ui.separator();
                    });
                }
            });
        });
        self.pages[self.current_page].show(ctx, &mut self.connections);
    }
}


pub trait AppPage {
    fn name(& self) -> &str;
    fn emote(& self) -> &str;
    fn show(&mut self, ctx: &eframe::egui::Context, connetions: &mut ConnectionManager);
}

