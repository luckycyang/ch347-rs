use ch347_rs::{ch347::Ch347UsbDevice, iic::IicDevice};
use core::fmt::Write;
use ssd1306::{I2CDisplayInterface, Ssd1306, mode::DisplayConfig, size::DisplaySize128x64};
fn main() {
    env_logger::init();
    let ch347 = Ch347UsbDevice::new().unwrap();
    let mut iic = IicDevice::new(&ch347).unwrap();
    iic.set_speed(ch347_rs::iic::Ch347IicSpeed::Mhz1).unwrap();

    let interface = I2CDisplayInterface::new(iic);
    let mut display = Ssd1306::new(
        interface,
        DisplaySize128x64,
        ssd1306::prelude::DisplayRotation::Rotate0,
    )
    .into_terminal_mode();
    display.init().unwrap();
    display.clear().unwrap();
    unsafe {
        for i in 97..123 {
            display
                .write_str(core::str::from_utf8_unchecked(&[i]))
                .unwrap();
        }

        for c in 65..91 {
            let _ = display.write_str(core::str::from_utf8_unchecked(&[c]));
        }
    }

    // there is bug that just output the last ascii as `!`, also want out `foo` that is just `o`
    write!(display, "{}", unsafe {
        core::str::from_utf8_unchecked(b"hello, world!")
    })
    .unwrap();
}
