use std::thread::spawn;

use ch347_rs::{Delay, ch347, gpio::Output};
use embedded_hal::{delay::DelayNs, digital::OutputPin};

fn main() {
    env_logger::init();
    let p = ch347::init().unwrap();

    let mut buf = [0; 128];

    // init swd
    ch347::write(&[0xE5, 8, 0, 0x40, 0x42, 0x0f, 0x00, 0, 0x00, 0x00, 0x00]).unwrap();
    ch347::read(&mut buf).unwrap();

    // read idcode
    let obuf = [0xE8, 0x04, 0x00, 0xA2, 0x22, 0x00, 0x81];
    ch347::write(&obuf).unwrap();
    ch347::read(&mut buf).unwrap();

    ch347::write(&obuf).unwrap();
    ch347::read(&mut buf).unwrap();
}
