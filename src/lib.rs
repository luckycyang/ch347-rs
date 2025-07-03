use std::{thread::sleep, time::Duration};

pub mod ch347;
pub mod command;
pub mod gpio;
pub mod hal;
pub mod i2c;
pub mod jtag;
pub mod spi;
pub mod swd;

pub fn format_u8_array(arr: &[u8]) -> String {
    let formatted: Vec<String> = arr.iter().map(|&byte| format!("0x{:02x}", byte)).collect();
    format!("[{}]", formatted.join(", "))
}

pub struct Delay;

impl Delay {
    pub fn new() -> Self {
        Self
    }
}

impl embedded_hal::delay::DelayNs for Delay {
    fn delay_ns(&mut self, ns: u32) {
        sleep(Duration::from_nanos(ns as u64));
    }
}
