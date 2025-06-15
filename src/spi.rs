use core::fmt;
use std::{error::Error, thread::sleep, time::Duration};

use crate::ch347::{self};

const MAX_SPI_SPEED: u32 = 60 * 1000 * 1000;
const MIN_SPI_SPEED: u32 = MAX_SPI_SPEED >> 7;

/// SPI Command
enum SpiDeviceCommand {
    SpiSetCfg,
    SpiCsCtrl,
    SpiOutIn,
    SpiIn,
    SpiOut,
    SpiGetCfg,
}

impl From<SpiDeviceCommand> for u8 {
    fn from(value: SpiDeviceCommand) -> Self {
        match value {
            SpiDeviceCommand::SpiSetCfg => 0xC0,
            SpiDeviceCommand::SpiCsCtrl => 0xC1,
            SpiDeviceCommand::SpiOutIn => 0xC2,
            SpiDeviceCommand::SpiIn => 0xC3,
            SpiDeviceCommand::SpiOut => 0xC4,
            SpiDeviceCommand::SpiGetCfg => 0xCA,
        }
    }
}

/// Spi dir mode
/// 全双工TX/RX, 双线RX, 单线RX/TX
#[derive(Debug)]
enum Ch347SpiDir {
    SpiDir2LinesFullDuplex,
    SpiDir2LinesRx,
    SpiDir1LineRx,
    SpiDir1LineTx,
}

impl From<Ch347SpiDir> for u16 {
    fn from(value: Ch347SpiDir) -> Self {
        match value {
            Ch347SpiDir::SpiDir2LinesFullDuplex => 0x0,
            Ch347SpiDir::SpiDir2LinesRx => 0x400,
            Ch347SpiDir::SpiDir1LineRx => 0x08000,
            Ch347SpiDir::SpiDir1LineTx => 0x0C000,
        }
    }
}

impl From<u16> for Ch347SpiDir {
    fn from(value: u16) -> Self {
        match value {
            0x0 => Ch347SpiDir::SpiDir2LinesFullDuplex,
            0x400 => Ch347SpiDir::SpiDir2LinesRx,
            0x8000 => Ch347SpiDir::SpiDir1LineRx,
            0xC000 => Ch347SpiDir::SpiDir1LineTx,
            _ => panic!("Invalid u16 value for Ch347SpiDir: {}", value), // 或者返回默认值
        }
    }
}

/// Spi 工作模式， 主机/从机
#[derive(Debug)]
enum Ch347SpiMode {
    Master,
    Slave,
}

impl From<Ch347SpiMode> for u16 {
    fn from(value: Ch347SpiMode) -> Self {
        match value {
            Ch347SpiMode::Master => 0x104,
            Ch347SpiMode::Slave => 0x0,
        }
    }
}

impl From<u16> for Ch347SpiMode {
    fn from(value: u16) -> Self {
        match value {
            0x104 => Ch347SpiMode::Master,
            0x0 => Ch347SpiMode::Slave,
            _ => panic!("Invalid u16 value for Ch347SpiMode: {}", value), // 或返回默认值
        }
    }
}

/// 片选控制，Spi主机模式选择软件控制，如果工作在从机选择硬件
enum Ch347SpiNss {
    Software,
    Hardware,
}

impl From<Ch347SpiNss> for u16 {
    fn from(value: Ch347SpiNss) -> Self {
        match value {
            Ch347SpiNss::Software => 0x0200,
            Ch347SpiNss::Hardware => 0x0,
        }
    }
}

/// first_bit: MSB or LSB,
/// prescaler = x * 8. x: 0=60MHz, 1=30MHz, 2=15MHz, 3=7.5MHz, 4=3.75MHz, 5=1.875MHz, 6=937.5KHz，7=468.75KHz
/// polynomial used for the CRC calculation.
/// out_default_data: Data to output on MOSI during SPI reading
/// bpw: 每次 SPI 传输数据宽度, 默认 8
/// polarity 时钟极性 0: 空闲低电平, 1: 空闲低电平
/// phase 相位 0: 第一个时钟采样, 1: 第一个时钟采样
/// nss 片选管理 0: 硬件管理，0x200: 软件管理
/// write_read_interval 读写间隔 us
/// out_default_data 读操作 MOSI 电平, 0xFF 全 1, 0x00 全 0
/// cs_config 两个CS 极性配置, bit7: 0-CS0低有效, bit6: 0-CS1低有效
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

impl Ch347SpiConfig {
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, &'static str> {
        if bytes.len() < std::mem::size_of::<Ch347SpiConfig>() {
            return Err("Byte slice too short for Ch347SpiConfig");
        }

        let mut config = Self::default();

        unsafe {
            std::ptr::copy_nonoverlapping(
                bytes.as_ptr(),
                &mut config as *mut Ch347SpiConfig as *mut u8,
                std::mem::size_of::<Ch347SpiConfig>(),
            );
        }

        Ok(config)
    }
}

impl fmt::Display for Ch347SpiConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Ch347SpiConfig {{\n\
            \tdirection: {:?},\n\
            \tmode: {},\n\
            \tbpw: 0x{:x},\n\
            \tpolarity: 0x{:x},\n\
            \tphase: 0x{:x},\n\
            \tnss: {},\n\
            \tbuad_prescalar: 0x{:x},\n\
            \tfirst_bit: {},\n\
            \tcrc_polynomial: 0x{:x},\n\
            \twrite_read_interval: 0x{:x},\n\
            \tout_default_data: 0x{:x},\n\
            \tcs_config: 0x{:x},\n\
            \treserved: {:?}\n\
            }}",
            Ch347SpiDir::from(self.direction),
            if self.mode == 0x104 {
                "Master"
            } else {
                "Slave"
            },
            self.bpw,
            self.polarity,
            self.phase,
            if self.nss == 0x200 {
                "Software"
            } else {
                "Hardware"
            },
            self.buad_prescalar,
            if self.first_bit == 0x80 { "LSB" } else { "MSB" },
            self.crc_polynomial,
            self.write_read_interval,
            self.out_default_data,
            self.cs_config,
            self.reserved
        )
    }
}

impl Default for Ch347SpiConfig {
    fn default() -> Self {
        // defaut as master, mode 0, speed: 15Mhz, bit width 8, MSB
        Self {
            direction: Ch347SpiDir::SpiDir2LinesFullDuplex.into(),
            mode: Ch347SpiMode::Master.into(),
            bpw: 0,
            polarity: 0,
            phase: 0,
            nss: Ch347SpiNss::Software.into(),
            buad_prescalar: 2,    // 15MHz
            first_bit: 0x00,      // MSB
            crc_polynomial: 0x07, // default
            write_read_interval: 0,
            out_default_data: 0xFF,
            cs_config: 0,           // in Slave work
            reserved: [0, 0, 6, 0], // default
        }
    }
}

#[repr(C)]
pub struct Ch347SpiDevice<'a> {
    device: &'a crate::ch347::Ch347UsbDevice,
    config: Ch347SpiConfig,
    obuf: [u8; 4096],
    oindex: usize,
    ibuf: [u8; 510],
    iindex: usize,
    speed: u32,
    mode: u8,
}

#[derive(Debug)]
enum Ch347SpiError {
    BufOverflow,
    Other,
}

impl fmt::Display for Ch347SpiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Ch347SpiError::BufOverflow => write!(f, "BufOverflow"),
            Ch347SpiError::Other => write!(f, "Ch347SpiError"),
        }
    }
}

impl std::error::Error for Ch347SpiError {}

impl<'a> Ch347SpiDevice<'a> {
    pub fn new(device: &'a ch347::Ch347UsbDevice) -> Self {
        assert_eq!(std::mem::size_of::<Ch347SpiConfig>(), 26);
        let mut ibuf = [0; 64];

        // read spi config
        device.write_bulk(&[0xCA, 0x01, 0x00, 0x01]).unwrap();
        let rev = device.read_bulk(&mut ibuf).unwrap();
        log::info!("Spi config: {}", format_u8_array(&ibuf[..rev]));

        // another read, idont wat do it
        device.write_bulk(&[0xCA, 0x01, 0x00, 0x02]).unwrap();
        let rev = device.read_bulk(&mut ibuf).unwrap();
        log::info!("Oher config: {}", format_u8_array(&ibuf[..rev]));

        // set config
        // 重复设置 Spi Config 会出现成功，失败交替现象，这里来两次
        let config = Ch347SpiConfig::default();
        let mut buf: Vec<u8> = Vec::new();
        buf.push(SpiDeviceCommand::SpiSetCfg.into());
        buf.push(26);
        buf.push(0);
        buf.extend_from_slice(unsafe {
            std::slice::from_raw_parts(&config as *const Ch347SpiConfig as *const u8, 26)
        });
        for i in 0..2 {
            device.write_bulk(&buf).unwrap();
            log::info!("Sending to USB: {}", format_u8_array(&buf));
            match device.read_bulk(&mut ibuf) {
                Ok(rev) => {
                    log::info!("Rev from USB: {}", format_u8_array(&ibuf[..rev]));
                    assert_eq!(rev, 4);
                    assert!(buf[3] == 0x00 && buf[0] == 0xC0);
                    break;
                }
                Err(e) => {
                    if i == 0 {
                        println!("{}", e);
                        continue;
                    } else {
                        panic!("Init Spi config error");
                    }
                }
            }
        }

        // read spi config
        device.write_bulk(&[0xCA, 0x01, 0x00, 0x01]).unwrap();
        let rev = device.read_bulk(&mut ibuf).unwrap();
        log::info!("Spi config: {}", format_u8_array(&ibuf[..rev]));

        Self {
            device,
            config,
            obuf: [0; 4096],
            oindex: 0,
            ibuf: [0; 510],
            iindex: 0,
            speed: 0,
            mode: 0,
        }
    }

    /// Clears the output buffer and resets the output index
    fn obuf_clear(&mut self) {
        self.obuf.fill(0);
        self.oindex = 0;
    }

    /// Clears the input buffer and resets the input index
    fn ibuf_clear(&mut self) {
        self.ibuf.fill(0);
        self.iindex = 0;
    }
    fn with_byte<T: Into<u8>>(&mut self, byte: T) -> Result<(), Ch347SpiError> {
        if self.obuf.len() <= self.oindex {
            Err(Ch347SpiError::BufOverflow)
        } else {
            self.obuf[self.oindex] = byte.into();
            self.oindex += 1;
            Ok(())
        }
    }

    fn with_bytes(&mut self, bytes: &[u8]) -> Result<(), Ch347SpiError> {
        let len = bytes.len();
        let new_len = self.oindex + len;
        let i = self.oindex;
        self.obuf[i..new_len].copy_from_slice(bytes);
        self.oindex = new_len;
        Ok(())
    }

    /// Appends write command with data
    fn with_write(&mut self, buf: &[u8]) -> Result<(), Ch347SpiError> {
        Ok(())
    }

    fn with_command(&mut self, command: SpiDeviceCommand) -> Result<(), Ch347SpiError> {
        self.with_byte(command)?;
        Ok(())
    }

    fn with_length(&mut self, len: u16) -> Result<(), Ch347SpiError> {
        let low = (len & 0x00FF) as u8;
        let high = (len >> 8) as u8;
        self.with_byte(low)?;
        self.with_byte(high)?;
        Ok(())
    }

    /// Flushes the output buffer to the USB device and optionally reads response
    fn flush(&mut self, read: bool) -> Result<(), Ch347SpiError> {
        self.device
            .write_bulk(&self.obuf[..self.oindex])
            .map_err(|_| Ch347SpiError::Other)?;
        log::info!(
            "Sending to USB: {}",
            format_u8_array(&self.obuf[..self.oindex])
        );

        if read {
            self.ibuf_clear();
            let rev = self
                .device
                .read_bulk(&mut self.ibuf)
                .map_err(|_| Ch347SpiError::Other)?;
            self.iindex = rev;
        }
        self.obuf_clear();
        Ok(())
    }

    pub fn get_spi_config(&mut self) -> Result<(), Ch347SpiError> {
        self.with_command(SpiDeviceCommand::SpiGetCfg)?;
        self.with_length(1)?;
        self.with_byte(1)?;
        self.flush(true)?;
        self.config =
            Ch347SpiConfig::from_bytes(&self.ibuf[3..29]).map_err(|_| Ch347SpiError::Other)?;
        Ok(())
    }

    pub fn set_spi_config(&mut self) -> Result<(), Ch347SpiError> {
        self.with_command(SpiDeviceCommand::SpiSetCfg)?;
        self.with_length(26)?;
        let config_buf = &self.config as *const Ch347SpiConfig as *const u8;
        let buf = unsafe { std::slice::from_raw_parts(config_buf, 26) };
        self.with_bytes(buf)?;
        self.flush(true)?;
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use crate::spi::Ch347SpiConfig;

    #[test]
    fn sizeofconfig() {
        println!(
            "Size of Ch347SpiConfig: {}",
            std::mem::size_of::<Ch347SpiConfig>()
        );
        println!(
            "Alignment of Ch347SpiConfig: {}",
            std::mem::align_of::<Ch347SpiConfig>()
        );
    }
}

/// Formats a byte array as a hexadecimal string for logging
pub fn format_u8_array(arr: &[u8]) -> String {
    let formatted: Vec<String> = arr.iter().map(|&byte| format!("0x{:02x}", byte)).collect();
    format!("[{}]", formatted.join(", "))
}
