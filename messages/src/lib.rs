#![no_std]

#[derive(core::fmt::Debug)]
pub enum Message{
    On,
    Off,
}

impl Message{
    pub fn to_str(&self) -> &'static str{
        match self{
            Message::On => "On",
            Message::Off => "Off",
        }
    }
    pub fn from_str(s: &str) -> Option<Self>{
        match s{
            "On" => Some(Message::On),
            "Off" => Some(Message::Off),
            _ => None,
        }
    }
}
