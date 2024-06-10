# EasyIoT-Template
This project aims to provide a quick start for your next embedded Rust project. Initially, it's designed for the Espressif ESP32, but it can be easily adjusted for use with other [embassy](https://embassy.dev/)-supported boards (such as other ESP boards, STM32-family, nRF52, nRF53, nRF91, and RP2040).

## The Vision
The goal is to deliver a codebase that enables your microcontroller project to be controlled and monitored from any browser right from the start. This will be possible with just a Wi-Fi-enabled board and Rust, eliminating the need to purchase and set up a server with a domain to act as a messenger between your IoT device and client app.

## The Basic Concept
The client app is a Rust web app that uses the [eframe](https://github.com/emilk/eframe_template) GUI framework along with the [egui](https://github.com/emilk/egui/) library for the user interface and the [ewebsock](https://github.com/rerun-io/ewebsock) library for fast bi-directional communication with the IoT device. This setup allows your project to be easily tested locally and later deployed to any device with a browser.

The microcontroller code is based on the [embassy](https://embassy.dev/) crates with a `#![no_std]`, no-alloc environment, and a custom WebSocket server implementation on top of [embassy-net](https://docs.embassy.dev/embassy-net/git/default/index.html) TCP sockets.

## Project Status
This project is part of my bachelor's thesis in Technical Information Technology at [OTH-Regensburg](https://www.oth-regensburg.de/) University and is currently in a very early development phase. The first release can be expected in October 2024. Development will continue as my free time allows, and contributions are welcome after the first release.

## How to Use
- You need to have [rust](https://www.rust-lang.org/tools/install) and the [esp-toolchain](https://docs.esp-rs.org/book/installation/index.html) installed.
- Create your own repo with the `Use this template` button at the top right of this page.
- Navigate to the client-app directory:
  - Run `trunk build --release` or `trunk serve --release --no-autoreload`.
  - If `--release` is not specified, the wasm binary will be much larger, and flashing to the ESP will take longer.
  - If `--no-autoreload` is not specified, Trunk will insert a WebSocket connection to itself, which is unwanted when flashed to the ESP.
- Navigate to the embedded-app directory:
  - Edit the Wi-Fi SSID and PASSWORD to fit your network (needs to be 2.4 GHz).
  - Run `cargo run`.
  - This will take a while.
  - When finished, you should see something like:
``` 
INFO - Wifi connected :)
INFO - Link is up!
INFO - Waiting to get IP address...
INFO - Got IP: 192.168.178.42/24
INFO - Spawning 2 HTTP servers with files:
index.html: 5742 Bytes
client-app-ffd9262d9fc92823.js: 61874 Bytes
client-app-ffd9262d9fc92823_bg.wasm: 2185175 Bytes
INFO - Spawning websocket and worker task

INFO - Waiting for websocket connection...
```
- Type the IP into the address bar of any browser on the same network (loading time for the wasm code is around 5 seconds).
- If everything worked, the ESP should be connected, and when you click the button, you should see the corresponding message in the ESP output.
- You can connect more ESPs in the settings page of the client app.

Details on how to expand the code to fit your needs will be included in the first release. This is just an early demo!

## Future Features
| Description               | Status         |
|---------------------------|----------------|
| OTA Updates               | Planned Oct. 2024 |
| Dynamic Wi-Fi Connections | Planned Oct. 2024 |
| Cargo-Generate Support    | Planned Oct. 2024 |
| HTTP Support              | Maybe Someday  |
| Full WebSocket Support    | Maybe Someday  |
| SSL Support               | Maybe Someday  |
| Multi-Client Setup        | Maybe Someday  |

