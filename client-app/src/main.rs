
mod app;
mod connection_manager;
mod settings_page;
mod home_page;
use app::ClientApp;

#[cfg(not(target_arch = "wasm32"))]
use eframe::{run_native, NativeOptions};

#[cfg(target_arch = "wasm32")]
use eframe::{WebOptions, WebRunner};

#[cfg(target_arch = "wasm32")]
mod js_functions{
    use wasm_bindgen::prelude::*;
    #[wasm_bindgen]
    extern "C" {
        pub fn hide_loading_animation();
        pub fn get_server_address() -> String;
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    let native_options = NativeOptions {
        ..Default::default()
    };
    run_native("EasyIoT-Template", native_options, Box::new(|cc| Box::new(ClientApp::new(cc, String::new())))).ok();
}


#[cfg(target_arch = "wasm32")]
fn main() {
    js_functions::hide_loading_animation();
    let ip = js_functions::get_server_address();
    let web_options = WebOptions {
        ..Default::default()
    };
    wasm_bindgen_futures::spawn_local(async {
            WebRunner::new().start(
            "the_canvas_id", // hardcode it, needs to match the id of the canvas element in index.html
            web_options,
            Box::new(|cc| Box::new(ClientApp::new(cc, ip))),
        ).await.expect("failed to start eframe");
    });
}


