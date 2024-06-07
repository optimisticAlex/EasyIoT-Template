
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, signal::Signal};

use esp_hal::rmt::{TxChannel, PulseCode};

const ONE : PulseCode = PulseCode{level1: true, level2: false, length1: 75, length2: 29};
const ZERO : PulseCode = PulseCode{level1: true, level2: false, length1: 29, length2: 75};
const END : PulseCode = PulseCode{level1: false, level2: false, length1: 0, length2: 0};



#[embassy_executor::task]
pub async fn worker_task(control: &'static Signal<CriticalSectionRawMutex, messages::Message>, mut channel: esp_hal::rmt::Channel<esp_hal::Blocking, 0>){
    loop {
        match control.wait().await {
            messages::Message::On => {
                esp_println::println!("Worker: On");
                let data = create_pulse_code(Color { r: 0, g: 0, b: 255 });
                let transaction = channel.transmit(&data);
                channel = transaction.wait().unwrap();
            }
            messages::Message::Off => {
                esp_println::println!("Worker: Off");
                let data = create_pulse_code(Color { r: 0, g: 0, b: 0 });
                let transaction = channel.transmit(&data);
                channel = transaction.wait().unwrap();
            }
        }
    }
}


#[derive(Copy, Clone)]
struct Color{
    r: u8,
    g: u8,
    b: u8
}

fn create_pulse_code(c: Color) -> [PulseCode; 25]{
    let mut data = [ZERO; 25];
    for i in 0..8{
        if (c.g >> i) & 1 == 1{
            data[7-i] = ONE;
        }
    }
    for i in 0..8{
        if (c.r >> i) & 1 == 1{
            data[15-i] = ONE;
        }
    }
    for i in 0..8{
        if (c.b >> i) & 1 == 1{
            data[23-i] = ONE;
        }
    }
    data[24] = END;
    data
}