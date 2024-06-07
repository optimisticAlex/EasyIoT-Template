# EasyIoT-Template
This project aims to provide a quick start for your next embedded Rust project. Initially, it's designed for the Espressif ESP32, but it can be easily adjusted for use with other [embassy](https://embassy.dev/)-supported boards (other ESP boards, STM32-family, nRF52, nRF53, nRF91, and RP2040).

## The Vision
The goal is to deliver a codebase that enables your microcontroller project to be controlled and monitored from any browser right from the start. This will be possible with just a Wi-Fi-enabled board and Rust, eliminating the need to purchase and set up a server with a domain to act as a messenger between your IoT device and client app.

## The Basic Concept
The client app is a Rust web app that uses the [eframe](https://github.com/emilk/eframe_template) GUI framework along with the [egui](https://github.com/emilk/egui/) library for the user interface, and the [ewebsock](https://github.com/rerun-io/ewebsock)et library for fast bi-directional communication with the IoT device. This setup allows your project to be easily tested locally and later deployed to any device with a browser.

The microcontroller code is based on the [embassy](https://embassy.dev/) crates with a `#![no_std]`, no alloc environment, and a custom WebSocket server implementation on top of [embassy-net](https://docs.embassy.dev/embassy-net/git/default/index.html) TCP sockets.

## Project Status
This project is part of my bachelor's thesis in Technical Information Technology at [OTH-Regensburg](https://www.oth-regensburg.de/) University and is currently in a very early development phase. The first release is expected in October 2024. Development will continue as my free time allows, and contributions are welcome after the first release.

## Planned Features

| Description | Status |
|-------------|--------|
|OTA Updates  | Planned Oct. 2024|
|Dynamic Wi-Fi Connections| Planned Oct. 2024|
|Cargo-Generate Support| Planned Oct. 2024|
|Minimal WebSocket Support| Planned Oct. 2024|
|HTTP Support| Maybe Someday|
|Full WebSocket Support| Maybe Someday|
|SSL Support| Maybe Someday|
|Multi-Client Setup| Maybe Someday|
|Multi-Device Setup| Maybe Someday|