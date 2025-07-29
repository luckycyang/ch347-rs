use ch347_rs::{self, ch347, i2c::I2cbus};
use core::fmt::Write;
use ssd1306::{I2CDisplayInterface, Ssd1306, mode::DisplayConfig, size::DisplaySize128x64};

fn main() {
    env_logger::init();
    let p = ch347::init().unwrap();
    let i2c = I2cbus::new(p.I2C, Default::default());

    let interface = I2CDisplayInterface::new(i2c);
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
