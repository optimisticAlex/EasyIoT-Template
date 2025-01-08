
# EasyIoT-Template
This is a research project that came to life as part of my [bachelor-thesis](https://github.com/optimisticAlex/EasyIoT-Template/releases/tag/prototype0.1.0) (written in german) at [OTH-Regensburg](https://www.oth-regensburg.de/) University. 

The thesis explains basic concepts of the internet of things, explores with the code in this repository the Rust way of developing such a system and compares it with C/C++ and Arduino. It comes to the conclusion that Rust is very promising for embedded / IoT development but the ecosystem is evolving to quickly to make a lasting statement.  

> 📌 **Note**: Because of the still evolving embedded Rust ecosystem and limited resources, development is currently paused. (Last time succesfully compiled: nov. 2024)

<!-- 
Initially, it's designed for the Espressif ESP32, but it can be adjusted for use with other [embassy](https://embassy.dev/)-supported boards (such as other ESP boards, STM32-family, nRF52, nRF53, nRF91, and RP2040).

## The VisionThe following TeckStack can be used .
The goal is to deliver a codebase that enables your microcontroller project to be controlled and monitored from any browser right from the start. This will be possible with just a Wi-Fi-enabled board and Rust, eliminating the need to purchase and set up a server with a domain to act as a messenger between your IoT device and client app.
-->

## The Basic Concept
This repository wants to be be a jumpstart template for developing IoT systems in pure Rust, supporting many scenarios and embedded targets with focus on the developer and enduser experience. Due to missing resources (aka time and money) currently *'esp32 with browser-app in home network'* is the only supported scenario. (Though the TeckStack has proven to be fun to work with and versatile enough to support probably any scenario):

The client app is a Rust web app that uses the [eframe](https://github.com/emilk/eframe_template) GUI framework along with the [egui](https://github.com/emilk/egui/) library for the user interface and the [ewebsock](https://github.com/rerun-io/ewebsock) library for fast bi-directional communication with the IoT device. This setup allows your project to be easily tested locally and later deployed to any device with a browser.

The microcontroller code is based on the [embassy](https://embassy.dev/) crates with a `#![no_std]` environment, and a custom WebSocket server implementation on top of [embassy-net](https://docs.embassy.dev/embassy-net/git/default/index.html) TCP sockets.

## How to Use
- You need [rust](https://www.rust-lang.org/tools/install), [trunk](https://trunkrs.dev/) and the [esp-toolchain](https://docs.esp-rs.org/book/installation/index.html) to be installed.
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
- Type the IP (without the '/24' suffix) into the address bar of any browser on the same network (loading time for the wasm code is around 5 seconds).
- If everything worked, the ESP should be connected, and when you click the button, you should see the corresponding message in the ESP output.
- You can connect more ESPs in the settings page of the client app.

Details on how to expand the code to fit your needs will be included in the first release. This is just an early demo!

## Future Features
| Description               | Status         |
|---------------------------|----------------|
| OTA Updates               | Maybe Someday  |
| Dynamic Wi-Fi Connections | Maybe Someday  |
| Cargo-Generate Support    | Maybe Someday  |
| Full WebSocket Support    | Maybe Someday  |
| SSL/TLS Support           | Maybe Someday  |
| Multi-Client Setup        | Maybe Someday  |

