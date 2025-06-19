use std::thread::sleep;
use std::time::Duration;

use ch347_rs::ch347;
use ch347_rs::gpio::Output;
use ch347_rs::spi::{Config, SpiDevice};
use embedded_graphics::{pixelcolor::Rgb565, prelude::*};
use mipidsi::interface::SpiInterface;
use mipidsi::{Builder, models::ST7789};

fn main() {
    env_logger::init();
    let p = ch347::init().unwrap();
    let mut delay = ch347_rs::Delay::new();
    let spi = SpiDevice::new(
        p.SPI0,
        Config {
            speed: 0,
            mode: ch347_rs::spi::Mode::Mode0,
            bit_order: ch347_rs::spi::BitOrder::MSB,
        },
    );
    let dc = Output::new(p.IO1);
    let rst = Output::new(p.IO2);
    let mut buffer = [0; 512];
    let di = SpiInterface::new(spi, dc, &mut buffer);

    let mut display = Builder::new(ST7789, di)
        .reset_pin(rst)
        .init(&mut delay)
        .unwrap();

    let colors = [
        Rgb565::BLACK,
        Rgb565::BLUE,
        Rgb565::GREEN,
        Rgb565::GREEN,
        Rgb565::RED,
    ];
    loop {
        for color in colors.iter() {
            display.clear(*color).unwrap();
            sleep(Duration::from_millis(500));
        }
    }
}
