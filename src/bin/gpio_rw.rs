use std::cell::RefCell;

use ch347_rs::gpio::{Flex, Gpio_Config, Output};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let ch347 = ch347_rs::ch347::Ch347UsbDevice::new()?;
    let gpio_config = RefCell::new(Gpio_Config::from_device(&ch347));
    let gpio1 = Output::new(&gpio_config, 1);
    let gpio2 = Flex::from_config(&gpio_config, 2);

    for _ in 0..10 {
        gpio1.toggle();
        println!("gpio2 input state: {}", gpio2.is_low());
    }

    Ok(())
}
