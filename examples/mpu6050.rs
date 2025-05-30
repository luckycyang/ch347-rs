use std::{thread::sleep, time::Duration};

use ch347_rs::{ch347::Ch347UsbDevice, iic::IicDevice};
use mpu6050::*;

#[derive(Debug)]
struct Delay;

impl embedded_hal::delay::DelayNs for Delay {
    fn delay_ns(&mut self, ns: u32) {
        sleep(Duration::from_nanos(u64::from(ns)));
    }
}

impl<UXX: Into<u64>> embedded_hal_old::blocking::delay::DelayMs<UXX> for Delay
where
    u64: From<UXX>,
{
    fn delay_ms(&mut self, ms: UXX) {
        sleep(Duration::from_millis(u64::from(ms)));
    }
}

fn main() {
    env_logger::init();
    let ch347 = Ch347UsbDevice::new().unwrap();
    let mut iic = IicDevice::new(&ch347).unwrap();
    iic.set_speed(ch347_rs::iic::Ch347IicSpeed::Khz100).unwrap();
    let mut delay = Delay;

    let mut mpu = Mpu6050::new(iic);
    mpu.init(&mut delay).unwrap();

    loop {
        // get roll and pitch estimate
        let acc = mpu.get_acc_angles().unwrap();
        println!("r/p: {:?}", acc);
    }
}
