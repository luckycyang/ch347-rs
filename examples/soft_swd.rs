use std::{process::id, str::RSplitN, thread::sleep, time::Duration};

use ch347_rs::{
    ch347,
    gpio::{Flex, Output, types::PinState},
};
use embedded_hal::digital::{InputPin, OutputPin};

fn delay(ms: u64) {
    sleep(Duration::from_millis(ms));
}

struct Swd<'a, U> {
    swdio: Flex<'a>,
    clk: U,
}

impl<'a, U: OutputPin> Swd<'a, U> {
    pub fn new(swdio: Flex<'a>, clk: U) -> Self {
        Self { swdio, clk }
    }

    pub fn trans_output(&mut self) {
        self.swdio.set_output(PinState::Low);
    }

    pub fn trans_input(&mut self) {
        self.swdio.set_input();
    }

    pub fn reset(&mut self) {
        self.trans_output();
        self.swdio.set_high().unwrap();
        for _ in 0..50 {
            self.clk.set_low().unwrap();
            delay(1);
            self.clk.set_high().unwrap();
            delay(1);
        }
        delay(1);
        self.clk.set_low().unwrap();
    }

    pub fn idle(&mut self) {
        self.trans_output();
        self.swdio.set_low().unwrap();
        for _ in 0..8 {
            self.clk.set_low().unwrap();
            delay(1);
            self.clk.set_high().unwrap();
        }
        self.clk.set_low().unwrap();
    }

    pub fn shift_bit(&mut self, bit: bool) {
        self.clk.set_low().unwrap();
        if bit {
            self.swdio.set_high().unwrap();
        } else {
            self.swdio.set_low().unwrap();
        };
        delay(1);
        self.clk.set_high().unwrap();
        delay(1);
        self.clk.set_low().unwrap();
    }

    pub fn read_idcode(&mut self) {
        self.trans_output();
        self.reset();
        self.idle();

        let command = 0b10100101;

        for i in 0..8 {
            self.shift_bit(command >> i & 0x01 == 0x01);
        }

        // tn
        self.trans_input();
        self.clk.set_low().unwrap();
        delay(1);
        self.clk.set_high().unwrap();
        delay(1);
        self.clk.set_low().unwrap();

        let mut ack = 0u8;
        let mut idcode = 0u32;
        let mut parity = false;

        for i in 0..3 {
            self.clk.set_low().unwrap();
            delay(1);
            self.clk.set_high().unwrap();
            delay(1);
            let rev = self.swdio.is_high().unwrap();
            if rev {
                ack = ack | (0x01 << i);
            }
        }
        for i in 0..32 {
            self.clk.set_low().unwrap();
            delay(1);
            self.clk.set_high().unwrap();
            delay(1);
            let rev = self.swdio.is_high().unwrap();
            if rev {
                idcode = idcode | (0x01 << i);
            }
        }
        self.clk.set_low().unwrap();
        delay(1);
        self.clk.set_high().unwrap();
        delay(1);
        parity = self.swdio.is_high().unwrap();
        self.clk.set_low().unwrap();

        println!(
            "ack: {:#03x}, idcode: {:#08x}, parity: {}",
            ack, idcode, parity
        );
    }
}

fn main() {
    env_logger::init();
    let p = ch347::init().unwrap();
    let swdio = Flex::new(p.IO1);
    let clk = Output::new(p.IO2);

    let mut swd = Swd::new(swdio, clk);
    swd.reset();
    swd.read_idcode();
}
