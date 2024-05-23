# EasyIoT-Template
This project aims to be the fastest start into your next embedded Rust project. At first for the Espressif ESP32 only, but designed to be easily adjusted for use with other [embassy](https://embassy.dev/)-supported boards (other esp-boards, STM32-family, nRF52, nRF53, nRF91 and RP2040). 

## The Vision
is to deliver a code base which enables your microcontroller project to be controlled and monitored with any browser from the start on. This will be possible with just a wifi enabled board and Rust, no need to buy and setup a server with domain to act as a messenger between your IoT device and client app. 

## The basic concept
The client-app is a Rust web-app that uses the [eframe](https://github.com/emilk/eframe_template) gui framework together with the [egui](https://github.com/emilk/egui/) library for the user interface and the [ewebsock](https://github.com/rerun-io/ewebsock)et library for fast bi-directional communication with the IoT-device. This enables your project to be easily testet localy and later deployed to any device with a browser.

The microcontroller code is based on the [embassy](https://embassy.dev/) crates with a `#![no_std]`, no alloc environment and a custom websocket-server implementation on top of [embassy-net](https://docs.embassy.dev/embassy-net/git/default/index.html) tcp-sockets. 

## Project status
This is part of my bachelor's thesis in technical information technology at the [OTH-Regensburg](https://www.oth-regensburg.de/) university and therefore currently in a very early development phase. The first release can be expected comming october (2024). After that development will be continued depending on how my free time allows it and contributions will be welcomend. 

## Planned features 

| Description | Status |
|-------------|--------|
|OTA-Updates  | planned oct. 2024|
|dynamic wifi-connections|planned oct. 2024|
|cargo-generate support|planned oct. 2024|
|minimal websocket support|planned oct. 2024|
|http support|maybe someday|
|full websocket support|maybe someday|
|ssl support|maybe someday|
|multi client setup|maybe someday|
|multi device setup|maybe someday|

