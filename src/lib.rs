pub mod ch347;
pub mod command;
pub mod gpio;
pub mod hal;
pub mod i2c;
pub mod spi;

pub fn format_u8_array(arr: &[u8]) -> String {
    let formatted: Vec<String> = arr.iter().map(|&byte| format!("0x{:02x}", byte)).collect();
    format!("[{}]", formatted.join(", "))
}
