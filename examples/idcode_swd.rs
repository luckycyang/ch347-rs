use std::thread::spawn;

use ch347_rs::{Delay, ch347, gpio::Output};
use embedded_hal::{delay::DelayNs, digital::OutputPin};

fn main() {
    env_logger::init();
    let p = ch347::init().unwrap();

    let mut io1 = Output::new(p.IO1);
    spawn(move || {
        loop {
            io1.set_low().unwrap();
            io1.set_high().unwrap();
        }
    });
    let mut delay = Delay::new();
    delay.delay_ms(500);

    let mut buf = [0; 128];

    // init swd
    ch347::write(&[0xE5, 8, 0, 0x40, 0x42, 0x0f, 0x00, 3, 0x00, 0x00, 0x00]).unwrap();
    ch347::read(&mut buf).unwrap();

    // read idcode
    let obuf = [0xE8, 0x04, 0x00, 0xA2, 0x22, 0x00, 0x81];
    ch347::write(&obuf).unwrap();
    ch347::read(&mut buf).unwrap();

    ch347::write(&obuf).unwrap();
    ch347::read(&mut buf).unwrap();
}
