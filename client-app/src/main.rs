
mod app;
use app::ClientApp;

#[cfg(not(target_arch = "wasm32"))]
use eframe::{run_native, NativeOptions};

#[cfg(target_arch = "wasm32")]
use eframe::{WebOptions, WebRunner};


#[cfg(not(target_arch = "wasm32"))]
fn main() {
    let native_options = NativeOptions {
        ..Default::default()
    };
    run_native("EasyIoT-Template", native_options, Box::new(|cc| Box::new(ClientApp::new(cc)))).ok();
}


#[cfg(target_arch = "wasm32")]
fn main() {
    let web_options = WebOptions {
        ..Default::default()
    };
    wasm_bindgen_futures::spawn_local(async {
            WebRunner::new().start(
            "the_canvas_id", // hardcode it, needs to match the id of the canvas element in index.html
            web_options,
            Box::new(|cc| Box::new(ClientApp::new(cc))),
        ).await.expect("failed to start eframe");
    });
}


