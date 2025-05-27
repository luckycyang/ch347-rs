use std::error::Error;
use std::fmt;
enum Ch347IicCommand {
    Ch347CmdI2cStream, // 命令流起始标志
    Ch347CmdI2cStmEnd, // 命令流结束标志
    Ch347CmdI2cStmSta, // 生成 I2C 起始条件（START）
    Ch347CmdI2cStmSto, // 生成 I2C 停止条件（STOP）
    Ch347CmdI2cStmOut, // 写操作，后接数据长度和数据
    Ch347CmdI2cStmIn,  // 读操作，后接读取字节数
    Ch347CmdI2cStmSet, // 设置 I2C 总线速度
}

impl From<Ch347IicCommand> for u8 {
    fn from(value: Ch347IicCommand) -> Self {
        match value {
            Ch347IicCommand::Ch347CmdI2cStream => 0xAA,
            Ch347IicCommand::Ch347CmdI2cStmEnd => 0x00,
            Ch347IicCommand::Ch347CmdI2cStmSta => 0x74,
            Ch347IicCommand::Ch347CmdI2cStmSto => 0x75,
            Ch347IicCommand::Ch347CmdI2cStmOut => 0x80,
            Ch347IicCommand::Ch347CmdI2cStmIn => 0xC0,
            Ch347IicCommand::Ch347CmdI2cStmSet => 0x60,
        }
    }
}

pub enum Ch347IicSpeed {
    Khz20,
    Khz50,
    Khz100,
    Khz200,
    Khz400,
    Khz750,
    Mhz1,
}

impl From<Ch347IicSpeed> for u8 {
    fn from(speed: Ch347IicSpeed) -> u8 {
        match speed {
            Ch347IicSpeed::Khz20 => 0,
            Ch347IicSpeed::Khz50 => 1,
            Ch347IicSpeed::Khz100 => 2,
            Ch347IicSpeed::Khz200 => 3,
            Ch347IicSpeed::Khz400 => 4,
            Ch347IicSpeed::Khz750 => 5,
            Ch347IicSpeed::Mhz1 => 6,
        }
    }
}

pub struct IicDevice<'a> {
    device: &'a crate::ch347::Ch347UsbDevice,
    speed: Ch347IicSpeed,
    obuf: [u8; 80],
    oindex: usize,
    ibuf: [u8; 80],
    iindex: usize,
}

#[derive(Debug)]
pub enum I2CError {
    HardwareError(i32), // 硬件错误
    Timeout,            // 传输超时
    InvalidResponse,    // 无效的硬件响应
}

impl fmt::Display for I2CError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            I2CError::HardwareError(code) => write!(f, "I2C hardware error: {}", code),
            I2CError::Timeout => write!(f, "I2C transmission timeout"),
            I2CError::InvalidResponse => write!(f, "Invalid I2C response from device"),
        }
    }
}

impl Error for I2CError {}

fn format_u8_array(arr: &[u8]) -> String {
    let formatted: Vec<String> = arr.iter().map(|&byte| format!("0x{:02x}", byte)).collect();
    format!("[{}]", formatted.join(", "))
}

impl<'a> IicDevice<'a> {
    pub fn new(device: &'a crate::ch347::Ch347UsbDevice) -> IicDevice<'a> {
        // in ch347t, gpio3/scl
        // device.write_bulk(&[0xCC, 0x08, 0x00, 0x00, 0x00, 0x00, 0xF8, 0x00, 0x00, 0x00, 0x00])
        // in ch347 when set speed to send E2 comamnd
        let mut buf = [0; 4];
        device
            .write_bulk(&[
                0xE2, 0x08, 0x00, 0x00, 0x00, 0x81, 0x81, 0x00, 0x00, 0x00, 0x00,
            ])
            .expect("Can't init iic device");
        device.read_bulk(&mut buf).expect("Can't init iic device");
        log::info!("e2 rev: {}", format_u8_array(&buf));
        Self {
            device,
            speed: Ch347IicSpeed::Khz100,
            obuf: [0; 80],
            oindex: 0,
            ibuf: [0; 80],
            iindex: 0,
        }
    }

    fn obuf_clear(&mut self) {
        self.obuf.copy_from_slice(&[0; 80]);
        self.oindex = 0;
    }

    fn ibuf_clear(&mut self) {
        self.ibuf.copy_from_slice(&[0; 80]);
        self.oindex += 1;
    }

    pub fn write_with_data(&mut self, data: &[u8]) {
        let mut left = data.len();
        let mut ptr = 0;
        let mut is_first = true;

        while left > 0 {
            let wlen = if left > 63 { 63 } else { left };
            let chunk = &data[ptr..(ptr + wlen)];

            self.with_stream_start();

            if is_first {
                self.with_start();
                is_first = false;
            }

            self.with_write(chunk);

            if left == wlen {
                self.with_stop();
            }

            self.with_stream_end();

            self.flush(true);

            ptr += wlen;
            left -= wlen;
        }
    }

    pub fn write_with_address(&mut self, buf: &[u8], address: u8) {
        let mut data = vec![address];
        data.extend_from_slice(buf);
        self.write_with_data(&data);
    }
    pub fn read_with_address(
        &mut self,
        buf: &mut [u8],
        address: u8,
        len: usize,
    ) -> Result<(), I2CError> {
        const MAX_I2C_XFER: usize = 63; // 最大单次读取字节数
        let mut byteoffset = 0; // 已读取的字节偏移
        let mut bytes_to_read = len.min(buf.len()); // 确保不超过缓冲区长度

        while bytes_to_read > 0 {
            // 计算本次读取的字节数
            let read_len = if bytes_to_read > MAX_I2C_XFER {
                MAX_I2C_XFER
            } else {
                bytes_to_read
            };

            // 清空输出缓冲区
            self.obuf_clear();

            // 构造 I2C 命令
            self.with_stream_start();
            self.with_start();
            // 写入设备地址（读模式：地址左移 1 位并置最低位为 1）
            self.with_write(&[(address << 1) | 1]);

            // 设置读取字节数
            if read_len > 1 {
                self.with_command(
                    u8::from(Ch347IicCommand::Ch347CmdI2cStmIn) | (read_len - 1) as u8,
                );
            }
            self.with_command(Ch347IicCommand::Ch347CmdI2cStmIn);
            self.with_stop();
            self.with_stream_end();

            // 执行传输
            self.flush(true);

            // 检查接收到的数据
            if self.iindex < read_len + 1 {
                return Err(I2CError::InvalidResponse);
            }
            if self.ibuf[0] != 1 {
                return Err(I2CError::Timeout);
            }

            // 复制数据到输出缓冲区
            let dest_slice = &mut buf[byteoffset..byteoffset + read_len];
            dest_slice.copy_from_slice(&self.ibuf[1..1 + read_len]);

            // 更新偏移和剩余字节
            byteoffset += read_len;
            bytes_to_read -= read_len;
        }

        Ok(())
    }

    fn flush(&mut self, read: bool) {
        log::info!(
            "send data to usb: {}",
            format_u8_array(&self.obuf[..self.oindex])
        );
        self.device
            .write_bulk(&self.obuf[0..self.oindex])
            .expect("send data to usb device error");
        if read {
            self.ibuf_clear();
            let rev = self
                .device
                .read_bulk(&mut self.ibuf)
                .expect("read data from usb device error");
            self.iindex = rev;
            log::info!(
                "rev data from usb: {}",
                format_u8_array(&self.ibuf[..self.iindex])
            );
            self.iindex = rev;
        }
        self.obuf_clear();
    }

    fn with_command<T>(&mut self, command: T)
    where
        T: Into<u8>,
    {
        self.obuf[self.oindex] = command.into();
        self.oindex += 1;
    }

    fn with_stream_start(&mut self) {
        self.with_command(Ch347IicCommand::Ch347CmdI2cStream);
    }

    fn with_stream_end(&mut self) {
        self.with_command(Ch347IicCommand::Ch347CmdI2cStmEnd);
    }

    fn with_start(&mut self) {
        self.with_command(Ch347IicCommand::Ch347CmdI2cStmSta);
    }

    fn with_stop(&mut self) {
        self.with_command(Ch347IicCommand::Ch347CmdI2cStmSto);
    }

    fn with_write(&mut self, buf: &[u8]) {
        let len = buf.len();
        self.with_command(u8::from(Ch347IicCommand::Ch347CmdI2cStmOut) | len as u8);
        let new_index = self.oindex + len;
        self.obuf[self.oindex..new_index].copy_from_slice(buf);
        self.oindex = new_index;
    }

    pub fn set_speed(&mut self, speed: Ch347IicSpeed) {
        self.with_stream_start();
        self.with_command(u8::from(speed) | u8::from(Ch347IicCommand::Ch347CmdI2cStmSet));
        self.with_stream_end();
        self.flush(false);
    }
}
