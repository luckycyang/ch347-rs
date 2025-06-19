use std::marker::PhantomData;

use embassy_hal_internal::Peripheral;

use crate::hal::{self};

pub mod instance {
    use crate::{
        ch347, format_u8_array,
        spi::{CSPin, Ch347SpiConfig, Config},
    };

    pub trait Instance {
        fn set_config(config: Config) {
            let cfg = Ch347SpiConfig::from(config);
            let mut ibuf = [0; 64];
            let mut buf: Vec<u8> = Vec::new();
            buf.push(0xC0);
            buf.push(26);
            buf.push(0);
            buf.extend_from_slice(unsafe {
                std::slice::from_raw_parts(&cfg as *const Ch347SpiConfig as *const u8, 26)
            });

            for i in 0..2 {
                ch347::write(&buf).unwrap();
                match ch347::read(&mut ibuf) {
                    Ok(rev) => {
                        assert_eq!(rev, 4);
                        assert!(buf[3] == 0x00 && buf[0] == 0xC0);
                        break;
                    }
                    Err(e) => {
                        if i == 0 {
                            println!("{:?}", e);
                            continue;
                        } else {
                            panic!("Init Spi config error");
                        }
                    }
                }
            }

            // is that cfg same of obuf
            ch347::write(&[0xCA, 0x01, 0x00, 0x01]).unwrap();
            ch347::read(&mut ibuf).unwrap();
        }

        fn cs_write(pin: CSPin, level: bool) {
            let state = if level { 0x80 | 0x40 } else { 0x80 };
            let index = if pin == CSPin::CS0 { 3 } else { 8 };
            // obuf + 3 是 CS0, obuf + 8 是 CS1
            let mut obuf = [
                0xC1, 0x0A, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            ];
            obuf[index] = state;
            ch347::write(&obuf).unwrap();
        }

        /// 一次最多发 4093 个byte
        fn write(buf: &[u8]) {
            let mut left = buf.len();
            let mut ptr = 0;
            let mut obuf = [0; 510];
            let mut ibuf = [0; 4];
            obuf[0] = 0xC4;
            while left > 0 {
                // 事实证明，发送也不能超过 507, 否则直接暴毙
                let wlen = left.min(507);
                obuf[1] = ((wlen as u16) & 0x00FF) as u8;
                obuf[2] = ((wlen as u16) >> 8) as u8;
                log::info!("write: {} bytes, low: {}, high: {}", wlen, obuf[1], obuf[2]);

                let chunk = &buf[ptr..ptr + wlen];
                (&mut obuf[3..3 + wlen]).copy_from_slice(chunk);

                ch347::write(&obuf[..3 + wlen]).unwrap();

                // consume rev data, as sussese, ibuf[3] == 0x00
                ch347::read(&mut ibuf).unwrap();

                left -= wlen;
                ptr += wlen;
            }
        }

        // 每次做多读 507 字节, 共 2^32
        fn read(buf: &mut [u8]) {
            let mut left = buf.len();
            let mut ptr = 0;
            let mut obuf = [0xC3, 0x04, 0x00, 0x00, 0x00, 0x00, 0x00];
            obuf[3] = left as u8;
            obuf[4] = ((left as u16) >> 8) as u8;
            obuf[5] = ((left as u32) >> 16) as u8;
            obuf[6] = ((left as u32) >> 24) as u8;
            let mut ibuf = [0; 510];
            ch347::write(&obuf).unwrap();

            while left > 0 {
                let wlen = left.min(507);

                ch347::read(&mut ibuf).unwrap();

                buf[ptr..ptr + wlen].copy_from_slice(&ibuf[3..3 + wlen]);

                ptr += wlen;
                left -= wlen;
            }
        }

        fn write_and_read(ibuf: &mut [u8], obuf: &[u8]) {
            assert_eq!(ibuf.len(), obuf.len());
            let mut left = ibuf.len();
            let mut ptr = 0;
            let mut command = [0; 510];
            let mut buffer = [0; 510];
            command[0] = 0xC2;

            while left > 0 {
                let wlen = left.min(507);

                command[1] = wlen as u8;
                command[2] = ((wlen as u16) >> 8) as u8;

                (&mut command[3..3 + wlen]).copy_from_slice(&obuf[ptr..ptr + wlen]);
                ch347::write(&command[..3 + wlen]).unwrap();

                ch347::read(&mut buffer).unwrap();
                ibuf[ptr..ptr + wlen].copy_from_slice(&buffer[3..3 + wlen]);

                ptr += wlen;
                left -= wlen;
            }
        }
    }
}

impl instance::Instance for hal::peripherals::SPI0 {}
pub trait Instance: Peripheral<P = Self> + instance::Instance + 'static + Send {}
impl Instance for hal::peripherals::SPI0 {}

#[derive(Debug, PartialEq, PartialOrd)]
pub enum CSPin {
    CS0,
    CS1,
}

pub enum Mode {
    Mode0,
    Mode1,
    Mode2,
    Mode3,
}

#[derive(Debug, PartialEq, Eq)]
pub enum BitOrder {
    MSB,
    LSB,
}

/// speed is (60 * 1000 * 1000) >> speed, as 0: 60M, 1: 30M
pub struct Config {
    pub speed: u16,
    pub mode: Mode,
    pub bit_order: BitOrder,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            speed: 2,
            mode: Mode::Mode0,
            bit_order: BitOrder::MSB,
        }
    }
}

#[repr(C)]
pub struct Ch347SpiConfig {
    direction: u16,
    mode: u16,
    bpw: u16,
    polarity: u16, // CPOL
    phase: u16,    // CPHA
    nss: u16,
    buad_prescalar: u16,
    first_bit: u16,
    crc_polynomial: u16,
    write_read_interval: u16,
    out_default_data: u8,
    cs_config: u8,
    reserved: [u8; 4],
}

impl Default for Ch347SpiConfig {
    fn default() -> Self {
        Self {
            direction: 0x00,
            mode: 0x104,
            bpw: 0x0000,
            polarity: 0x00,
            phase: 0x00,
            nss: 0x0200,
            buad_prescalar: 2,
            first_bit: 0x00,
            crc_polynomial: 0x07,
            write_read_interval: 0x00,
            out_default_data: 0xFF,
            cs_config: 0x00,
            reserved: [0, 0, 6, 0],
        }
    }
}

impl From<Config> for Ch347SpiConfig {
    fn from(value: Config) -> Self {
        let mut cfg: Ch347SpiConfig = Default::default();
        cfg.buad_prescalar = value.speed;
        match value.mode {
            Mode::Mode0 => {
                cfg.polarity = 0;
                cfg.phase = 0;
            }
            Mode::Mode1 => {
                cfg.polarity = 0;
                cfg.phase = 1;
            }
            Mode::Mode2 => {
                cfg.polarity = 2;
                cfg.phase = 0;
            }
            Mode::Mode3 => {
                cfg.polarity = 2;
                cfg.phase = 1;
            }
        }
        cfg.first_bit = if value.bit_order == BitOrder::MSB {
            0x0000
        } else {
            0x0080
        };
        cfg
    }
}

/// SPI 与部分 GPIO 复用, 后续再说
/// SpiDevice 额外具有 CS
pub struct SpiDevice<'d, T: Instance> {
    _spi: PhantomData<&'d T>,
}

impl<'d, T: Instance> SpiDevice<'d, T> {
    pub fn new(_spi: impl Peripheral<P = T>, config: Config) -> Self {
        T::set_config(config);
        Self { _spi: PhantomData }
    }

    pub fn write_data(&self, buf: &[u8]) {
        T::cs_write(CSPin::CS0, false);
        T::write(buf);
        T::cs_write(CSPin::CS0, true);
    }

    pub fn read_data(&self, buf: &mut [u8]) {
        T::cs_write(CSPin::CS0, false);
        T::read(buf);
        T::cs_write(CSPin::CS0, true);
    }

    pub fn write_and_read(&self, ibuf: &mut [u8], obuf: &[u8]) {
        T::cs_write(CSPin::CS0, false);
        T::write_and_read(ibuf, obuf);
        T::cs_write(CSPin::CS0, true);
    }

    pub fn write_and_read_in_place(&self, buf: &mut [u8]) {
        // 做不到单片机那种细致的传输
        let mut obuf = Vec::new();
        obuf.extend_from_slice(&buf);
        T::cs_write(CSPin::CS0, false);
        self.write_and_read(buf, &obuf);
        T::cs_write(CSPin::CS0, true);
    }
}

// SpiBus 是 SCK, MISO, MOSI

mod embedded_hal_v100_impl {
    use std::{thread::sleep, time::Duration};

    use embedded_hal::spi::*;

    use crate::spi::Instance;

    impl<'d, T: Instance> ErrorType for super::SpiDevice<'d, T> {
        type Error = core::convert::Infallible;
    }

    impl<'d, T: Instance> SpiDevice for super::SpiDevice<'d, T> {
        fn transaction(&mut self, operations: &mut [Operation<'_, u8>]) -> Result<(), Self::Error> {
            // 通常来说 operations 只有一个
            for op in operations.iter_mut() {
                match op {
                    Operation::Read(buf) => self.read_data(buf),
                    Operation::Write(buf) => self.write_data(buf),
                    Operation::Transfer(ibuf, obuf) => self.write_and_read(ibuf, obuf),
                    Operation::TransferInPlace(buf) => self.write_and_read_in_place(buf),
                    Operation::DelayNs(ns) => {
                        sleep(Duration::from_nanos(*ns as u64));
                    }
                }
            }
            Ok(())
        }
    }
}

// SpiBus + GPIO, waiting, 看那个spi模块的crate用SpiBus, 先测 mipidsi crate
