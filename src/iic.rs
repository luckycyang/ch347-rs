use std::error::Error;
use std::fmt;

/// I2C command codes for CH347 device
#[derive(Clone, Copy)]
enum Ch347IicCommand {
    Ch347CmdI2cStream, // Command stream start
    Ch347CmdI2cStmEnd, // Command stream end
    Ch347CmdI2cStmSta, // I2C START condition
    Ch347CmdI2cStmSto, // I2C STOP condition
    Ch347CmdI2cStmOut, // Write operation, followed by data length and data
    Ch347CmdI2cStmIn,  // Read operation, followed by byte count
    Ch347CmdI2cStmSet, // Set I2C bus speed
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

/// I2C bus speed settings for CH347
#[derive(Clone, Copy)]
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

/// I2C device abstraction for CH347 USB device
pub struct IicDevice<'a> {
    device: &'a crate::ch347::Ch347UsbDevice,
    speed: Ch347IicSpeed,
    obuf: [u8; 80],
    oindex: usize,
    ibuf: [u8; 80],
    iindex: usize,
}

/// I2C-specific error types
#[derive(Debug)]
pub enum I2CError {
    HardwareError(i32), // Hardware-related error with code
    Timeout,            // Transmission timeout
    InvalidResponse,    // Invalid response from device
    BufferOverflow,     // Buffer capacity exceeded
    UsbError(String),   // USB communication error
}

impl fmt::Display for I2CError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            I2CError::HardwareError(code) => write!(f, "I2C hardware error: {}", code),
            I2CError::Timeout => write!(f, "I2C transmission timeout"),
            I2CError::InvalidResponse => write!(f, "Invalid I2C response from device"),
            I2CError::BufferOverflow => write!(f, "I2C buffer overflow"),
            I2CError::UsbError(msg) => write!(f, "USB error: {}", msg),
        }
    }
}

impl Error for I2CError {}

/// Formats a byte array as a hexadecimal string for logging
fn format_u8_array(arr: &[u8]) -> String {
    let formatted: Vec<String> = arr.iter().map(|&byte| format!("0x{:02x}", byte)).collect();
    format!("[{}]", formatted.join(", "))
}

impl<'a> IicDevice<'a> {
    /// Creates a new I2C device instance with default speed (100 kHz)
    pub fn new(device: &'a crate::ch347::Ch347UsbDevice) -> Result<Self, I2CError> {
        // Initialize CH347 I2C interface
        let init_cmd = [
            0xE2, 0x08, 0x00, 0x00, 0x00, 0x81, 0x81, 0x00, 0x00, 0x00, 0x00,
        ];
        device
            .write_bulk(&init_cmd)
            .map_err(|e| I2CError::UsbError(e.to_string()))?;

        let mut buf = [0; 4];
        let read_len = device
            .read_bulk(&mut buf)
            .map_err(|e| I2CError::UsbError(e.to_string()))?;
        log::info!("I2C init response: {}", format_u8_array(&buf[..read_len]));

        Ok(Self {
            device,
            speed: Ch347IicSpeed::Khz100,
            obuf: [0; 80],
            oindex: 0,
            ibuf: [0; 80],
            iindex: 0,
        })
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

    /// Writes data to the I2C device, handling chunking for large transfers
    pub fn write_with_data(&mut self, data: &[u8]) -> Result<(), I2CError> {
        let mut left = data.len();
        let mut ptr = 0;
        let mut is_first = true;

        while left > 0 {
            let wlen = left.min(63); // Max 63 bytes per transfer
            if self.oindex + wlen + 4 > self.obuf.len() {
                return Err(I2CError::BufferOverflow);
            }

            let chunk = &data[ptr..ptr + wlen];

            self.with_stream_start().unwrap();
            if is_first {
                self.with_start().unwrap();
                is_first = false;
            }
            self.with_write(chunk).unwrap();
            if left == wlen {
                self.with_stop().unwrap();
            }
            self.with_stream_end().unwrap();

            self.flush(true)?;

            ptr += wlen;
            left -= wlen;
        }
        Ok(())
    }

    /// Writes data to the I2C device with a specified address
    pub fn write_with_address(&mut self, buf: &[u8], address: u8) -> Result<(), I2CError> {
        let mut data = vec![address];
        data.extend_from_slice(buf);
        self.write_with_data(&data)
    }

    /// Reads data from the I2C device with a specified address
    pub fn read_with_address(
        &mut self,
        buf: &mut [u8],
        address: u8,
        len: usize,
    ) -> Result<(), I2CError> {
        self.obuf_clear();
        self.with_stream_start()?;
        self.with_start()?;
        // Write device address in read mode (shift left and set read bit)
        self.with_write(&[address])?;
        self.with_command(u8::from(Ch347IicCommand::Ch347CmdI2cStmIn) | (len) as u8)?;
        self.with_stop()?;
        self.with_stream_end()?;
        self.flush(true)?;

        if self.iindex - 1 != len {
            Err(I2CError::InvalidResponse)
        } else {
            buf.copy_from_slice(&self.ibuf[1..self.iindex]);
            Ok(())
        }
    }

    /// Flushes the output buffer to the USB device and optionally reads response
    fn flush(&mut self, read: bool) -> Result<(), I2CError> {
        if self.oindex > self.obuf.len() {
            return Err(I2CError::BufferOverflow);
        }

        log::info!(
            "Sending to USB: {}",
            format_u8_array(&self.obuf[..self.oindex])
        );
        self.device
            .write_bulk(&self.obuf[..self.oindex])
            .map_err(|e| I2CError::UsbError(e.to_string()))?;

        if read {
            self.ibuf_clear();
            let rev = self
                .device
                .read_bulk(&mut self.ibuf)
                .map_err(|e| I2CError::UsbError(e.to_string()))?;
            self.iindex = rev;
            log::info!("Received from USB: {}", format_u8_array(&self.ibuf[..rev]));
        }
        self.obuf_clear();
        Ok(())
    }

    /// Appends a command to the output buffer
    fn with_command<T: Into<u8>>(&mut self, command: T) -> Result<(), I2CError> {
        if self.oindex >= self.obuf.len() {
            return Err(I2CError::BufferOverflow);
        }
        self.obuf[self.oindex] = command.into();
        self.oindex += 1;
        Ok(())
    }

    /// Appends I2C stream start command
    fn with_stream_start(&mut self) -> Result<(), I2CError> {
        self.with_command(Ch347IicCommand::Ch347CmdI2cStream)
    }

    /// Appends I2C stream end command
    fn with_stream_end(&mut self) -> Result<(), I2CError> {
        self.with_command(Ch347IicCommand::Ch347CmdI2cStmEnd)
    }

    /// Appends I2C START condition
    fn with_start(&mut self) -> Result<(), I2CError> {
        self.with_command(Ch347IicCommand::Ch347CmdI2cStmSta)
    }

    /// Appends I2C STOP condition
    fn with_stop(&mut self) -> Result<(), I2CError> {
        self.with_command(Ch347IicCommand::Ch347CmdI2cStmSto)
    }

    /// Appends write command with data
    fn with_write(&mut self, buf: &[u8]) -> Result<(), I2CError> {
        let len = buf.len();
        if len > 63 || self.oindex + len + 1 > self.obuf.len() {
            return Err(I2CError::BufferOverflow);
        }
        self.with_command(u8::from(Ch347IicCommand::Ch347CmdI2cStmOut) | len as u8)?;
        self.obuf[self.oindex..self.oindex + len].copy_from_slice(buf);
        self.oindex += len;
        Ok(())
    }

    /// Sets the I2C bus speed
    pub fn set_speed(&mut self, speed: Ch347IicSpeed) -> Result<(), I2CError> {
        self.speed = speed;
        self.obuf_clear();
        self.with_stream_start()?;
        self.with_command(u8::from(speed) | u8::from(Ch347IicCommand::Ch347CmdI2cStmSet))?;
        self.with_stream_end()?;
        self.flush(false)?;
        Ok(())
    }
}

mod embedded_hal_impls {
    use super::{I2CError, IicDevice};
    use embedded_hal::i2c::{ErrorType, I2c, Operation};

    impl ErrorType for IicDevice<'_> {
        type Error = I2CError;
    }

    impl embedded_hal::i2c::Error for I2CError {
        fn kind(&self) -> embedded_hal::i2c::ErrorKind {
            match self {
                _ => embedded_hal::i2c::ErrorKind::Other,
            }
        }
    }

    impl I2c for IicDevice<'_> {
        /// Executes a sequence of I2C operations (read/write) for a given address
        fn transaction(
            &mut self,
            address: u8,
            operations: &mut [Operation<'_>],
        ) -> Result<(), Self::Error> {
            for op in operations.iter_mut() {
                match op {
                    Operation::Read(buf) => {
                        self.read_with_address(buf, (address << 1) | 1, buf.len())?;
                    }
                    Operation::Write(buf) => {
                        // For write, include address in the data stream
                        self.write_with_address(buf, address << 1)?;
                    }
                }
            }
            Ok(())
        }
    }
}

mod embedded_hal_old {
    use embedded_hal_old as embedded_hal_0_2_7;

    use super::{I2CError, IicDevice};

    impl embedded_hal_0_2_7::blocking::i2c::WriteRead for IicDevice<'_> {
        type Error = I2CError;
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

    impl embedded_hal_0_2_7::blocking::i2c::Write for IicDevice<'_> {
        type Error = I2CError;
        fn write(&mut self, address: u8, bytes: &[u8]) -> Result<(), Self::Error> {
            <Self as embedded_hal::i2c::I2c>::write(self, address, bytes)?;
            Ok(())
        }
    }
}
