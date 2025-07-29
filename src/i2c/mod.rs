use std::marker::PhantomData;

use embassy_hal_internal::Peripheral;

use crate::ch347;

pub mod instance {
    use crate::ch347;
    pub trait Instance {
        fn write_with_address(address: u8, buf: &[u8]) {
            let mut obuf = vec![address << 1];
            obuf.extend_from_slice(buf);
            let mut left = obuf.len();
            let mut ptr = 0;
            let mut is_first = true;

            // 剩余发送数据
            while left > 0 {
                // 发多少数据就多少个ACK, 返回数据没有命令头
                let mut ibuf = [0; 63];
                // 一次只能发送 63 byte
                let wlen = left.min(63);

                let chunk = &obuf[ptr..wlen];

                // Stream start
                let mut command = vec![0xAA];

                // 如果不是开始则不用Sta信号
                if is_first {
                    command.push(0x74);
                    is_first = false;
                }

                command.push(0x80 | chunk.len() as u8);
                command.extend_from_slice(chunk);

                // 如果是最后的数据包则发送 Stop 信号
                if left == wlen {
                    command.push(0x75);
                }
                command.push(0x00);

                ch347::write(&command).unwrap();

                let _rev = ch347::read(&mut ibuf).unwrap();

                ptr += wlen;
                left -= wlen;
            }
        }

        fn read_with_address(address: u8, buf: &mut [u8]) {
            // 读取时序是发送读i2c从机地址和寄存器地址，然后接受
            // 反正一次最多接收63字节
            let mut ibuf = [0; 64];

            let command = vec![
                0xAA,
                0x74,
                0x81,
                (address << 1) | 1,
                0xC0 | buf.len() as u8,
                0x75,
                0x00,
            ];
            ch347::write(&command).unwrap();
            let rev = ch347::read(&mut ibuf).unwrap();
            // assert_eq!(rev, ibuf.len()); // 1 个 ACK + 数据接收
            buf.copy_from_slice(&ibuf[1..rev]);
        }
    }
}

pub trait Instance: Peripheral<P = Self> + instance::Instance + 'static + Send {}
impl instance::Instance for crate::hal::peripherals::I2C {}
impl Instance for crate::hal::peripherals::I2C {}

pub struct I2cbus<'d, T: Instance> {
    _i2c: PhantomData<&'d T>,
}

impl<'d, T: Instance> I2cbus<'d, T> {
    pub fn new(_i2c: impl Peripheral<P = T>, config: Config) -> Self {
        // 我也不知道具体是什么，可能是设置引脚复用
        ch347::write(&[
            0xE2, 0x08, 0x00, 0x00, 0x00, 0x81, 0x81, 0x00, 0x00, 0x00, 0x00,
        ])
        .unwrap();
        let mut _ibuf = [0; 4];
        ch347::read(&mut _ibuf).unwrap();

        // set speed
        let buf = [0xAA, 0x60 | config.speed, 0x00];
        ch347::write(&buf).unwrap();
        Self { _i2c: PhantomData }
    }

    pub fn write_with_address(&self, address: u8, buf: &[u8]) {
        T::write_with_address(address, buf);
    }

    pub fn read_with_address(&self, address: u8, buf: &mut [u8]) {
        T::read_with_address(address, buf);
    }
}

/// 这是从 ch347demo
/// 0: 20KHz
/// 1: 50KHz
/// 2: 100KHz
/// 3: 200KHz
/// 4: 400KHz
/// 5: 750KHz
/// 6: 1MHz
pub struct Config {
    pub speed: u8,
}

impl Default for Config {
    fn default() -> Self {
        Self { speed: 2 }
    }
}

mod embedded_hal_v100_impl {
    use crate::i2c::Instance;
    use embedded_hal::i2c::*;

    use super::I2cbus;

    impl<'d, T: Instance> ErrorType for I2cbus<'d, T> {
        type Error = core::convert::Infallible;
    }

    impl<'d, T: Instance> I2c for I2cbus<'d, T> {
        fn transaction(
            &mut self,
            address: u8,
            operations: &mut [Operation<'_>],
        ) -> Result<(), Self::Error> {
            for op in operations.iter_mut() {
                match op {
                    Operation::Read(buf) => {
                        self.read_with_address(address, buf);
                    }
                    Operation::Write(buf) => {
                        self.write_with_address(address, buf);
                    }
                }
            }
            Ok(())
        }
    }
}

mod embedded_hal_v027_impl {
    use embedded_hal_027::blocking::i2c::*;

    use crate::i2c::{I2cbus, Instance};

    impl<'d, T: Instance> WriteRead for I2cbus<'d, T> {
        type Error = core::convert::Infallible;
        fn write_read(
            &mut self,
            address: u8,
            bytes: &[u8],
            buffer: &mut [u8],
        ) -> Result<(), Self::Error> {
            <Self as embedded_hal::i2c::I2c>::write_read(self, address, bytes, buffer)?;
            Ok(())
        }
    }

    impl<'d, T: Instance> Write for I2cbus<'d, T> {
        type Error = core::convert::Infallible;
        fn write(&mut self, address: u8, bytes: &[u8]) -> Result<(), Self::Error> {
            <Self as embedded_hal::i2c::I2c>::write(self, address, bytes).unwrap();
            Ok(())
        }
    }
}
