use std::cell::RefCell;

use ch347_rs::gpio::{GpioConfig, Input, Output};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let ch347 = ch347_rs::ch347::Ch347UsbDevice::new()?;
    let gpio_config = RefCell::new(GpioConfig::from_device(&ch347));
    let mut io1 = Output::new(&gpio_config, 1);
    let io2 = Input::new(&gpio_config, 2);
    let mut state;

    for _ in 0..10 {
        io1.toggle();
        state = if io2.is_high() { 1 } else { 0 };
        println!("io2 state: {}", state);
    }

    Ok(())
}
