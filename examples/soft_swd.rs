use std::{process::id, str::RSplitN, thread::sleep, time::Duration};

use ch347_rs::{
    ch347, command,
    gpio::{Flex, Input, Output, types::PinState},
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
    }

    pub fn idle(&mut self) {
        self.trans_output();
        self.swdio.set_low().unwrap();
        for _ in 0..8 {
            self.clk.set_low().unwrap();
            delay(1);
            self.clk.set_high().unwrap();
            delay(1);
        }
    }

    pub fn shift_bit(&mut self, bit: bool) {
        if bit {
            self.swdio.set_high().unwrap();
        } else {
            self.swdio.set_low().unwrap();
        };
        self.clk.set_low().unwrap();
        delay(1);
        self.clk.set_high().unwrap();
        delay(1);
    }

    pub fn read_idcode(&mut self) {
        self.trans_output();

        // jtag 2 swd
        self.reset();
        let bits = 0xE79Eu16;
        for i in 0..16 {
            self.shift_bit(bits >> i & 0x01 == 0x01);
        }
        self.reset();

        // 50 以上的高电平会进入复位
        self.reset();
        // 通常来说 8  个idle, 确保下个 start 能正常开始
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
            "ack: {:#03b}, idcode: {:#08x}, parity: {}",
            ack, idcode, parity
        );
        // 通常来说 8  个idle, 确保下个 start 能正常开始
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
            "ack: {:#03b}, idcode: {:#08x}, parity: {}",
            ack, idcode, parity
        );
    }
}

fn main() {
    env_logger::init();
    let p = ch347::init().unwrap();
    let mut buf = [0; 128];
    ch347::write(&[
        0xE5, 0x08, 0x00, 0x40, 0x42, 0x0f, 0x00, 0, 0x00, 0x00, 0x00,
    ])
    .unwrap();
    ch347::read(&mut buf).unwrap();

    ch347::write(&[
        0xE8, 10, 0x00, 0xA1, 56, 0x00, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x0f,
    ])
    .unwrap();
    ch347::read(&mut buf).unwrap();

    ch347::write(&[0xE8, 4, 0x00, 0xA2, 0x22, 0x00, 0xA5]).unwrap();
    ch347::read(&mut buf).unwrap();
}
