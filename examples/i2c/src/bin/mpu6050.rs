use std::{thread::sleep, time::Duration};

use ch347_rs::{ch347, i2c::I2cbus};
use mpu6050::*;

#[derive(Debug)]
struct Delay;

impl embedded_hal::delay::DelayNs for Delay {
    fn delay_ns(&mut self, ns: u32) {
        sleep(Duration::from_nanos(u64::from(ns)));
    }
}

impl<UXX: Into<u64>> embedded_hal_027::blocking::delay::DelayMs<UXX> for Delay
where
    u64: From<UXX>,
{
    fn delay_ms(&mut self, ms: UXX) {
        sleep(Duration::from_millis(u64::from(ms)));
    }
}

fn main() {
    env_logger::init();
    let p = ch347::init().unwrap();
    let i2c = I2cbus::new(p.I2C, Default::default());
    let mut delay = Delay;

    let mut mpu = Mpu6050::new(i2c);
    mpu.init(&mut delay).unwrap();

    loop {
        // get roll and pitch estimate
        let acc = mpu.get_acc_angles().unwrap();
        println!("r/p: {:?}", acc);
    }
}
