use crate::ch347;
#[allow(dead_code)]
enum Command {
    Ch347SwdInit,
    Ch347Swd,
    Ch347SwdRegW,
    Ch347SwdSeqW,
    Ch347SwdRegR,
}

pub struct SwdCommandSeq {
    subcommand: Vec<SubCommand>,
    rlen: u16,
}

impl SwdCommandSeq {
    pub fn new(speed: u8) -> Self {
        let mut ibuf = [0; 4];
        ch347::write(&[
            0xE5, 0x08, 0x00, 0x40, 0x42, 0x0f, 0x00, speed, 0x00, 0x00, 0x00,
        ])
        .unwrap();
        ch347::read(&mut ibuf).unwrap();

        Self {
            subcommand: Vec::new(),
            rlen: 0,
        }
    }

    pub fn push(&mut self, c: SubCommand) {
        let len = match c {
            // 0xA2 + ACK + DATA + TURN
            SubCommand::RegR { .. } => 1 + 1 + 4 + 1,
            // 0xA0 + ACK
            SubCommand::RegW { .. } => 1 + 1,
        };
        self.rlen += len;
        self.subcommand.push(c);
    }

    pub fn take(&mut self) -> Vec<u8> {
        let mut buf = Vec::new();
        for &c in self.subcommand.iter() {
            if c.is_read() {
                buf.extend_from_slice(&[0xA2, 0x22, 0x00]);
                buf.push(u8::from(c));
            } else {
                if let SubCommand::RegW { data, .. } = c {
                    // 对 data 进行校验
                    let mut count = 0u8;
                    for i in 0..32 {
                        if data >> i & 0x01 == 0x01 {
                            count += 1;
                        }
                    }
                    buf.extend_from_slice(&[0xA0, 0x29, 0x00]);
                    buf.push(u8::from(c));
                    buf.extend_from_slice(&(data.to_le_bytes()));
                    log::info!("data: {:#08x}, with parity: {}", data, count % 2);
                    buf.push(count % 2);
                }
            }
        }
        buf
    }

    pub fn flush(&mut self) {
        let subcommand = self.take();
        let mut obuf = Vec::new();
        // 0xE8 low high + subcommand
        log::info!("flush command ready to read {} bytes", 3 + self.rlen);
        let mut ibuf = [0; 128];
        obuf.push(0xE8);
        obuf.extend_from_slice(&(subcommand.len() as u16).to_le_bytes());
        obuf.extend_from_slice(&subcommand);

        ch347::write(&obuf).unwrap();
        ch347::read(&mut ibuf).unwrap();

        // update read buffer
        // TODO
        self.subcommand.clear();
        self.rlen = 0;
    }

    pub fn seq(&self, data: &[u8]) {
        let mut subcommand = vec![0xA1];
        subcommand.extend_from_slice(&((data.len() * 8) as u16).to_le_bytes());
        subcommand.extend_from_slice(data);

        let mut obuf = vec![0xE8];
        let mut ibuf = [0; 4];

        obuf.extend_from_slice(&(subcommand.len() as u16).to_le_bytes());
        obuf.extend_from_slice(&subcommand);

        ch347::write(&obuf).unwrap();
        ch347::read(&mut ibuf).unwrap();
    }

    pub fn reset(&self) {
        self.seq(&[0xff; 7]);
    }

    pub fn idle(&self) {
        self.seq(&[0; 1]);
    }

    pub fn reset_and_idle(&self) {
        self.reset();
        self.idle();
    }

    pub fn jtag_to_swd(&self) {
        self.reset();
        self.seq(&(0xE79Eu16).to_le_bytes());
        self.reset();
    }

    pub fn read_ap_reg(&self, address: u8) -> Result<u32, Box<dyn std::error::Error>> {
        let commmand = SubCommand::RegR {
            address,
            is_dp: false,
        };
        let obuf = [0xE8, 4, 0, 0xA2, 0x22, 0, u8::from(commmand)];
        let mut ibuf = [0; 10];
        ch347::write(&obuf).unwrap();
        ch347::read(&mut ibuf).unwrap();
        let rev = u32::from(ibuf[5])
            | u32::from(ibuf[6]) << 8
            | u32::from(ibuf[7]) << 16
            | u32::from(ibuf[8]) << 24;
        Ok(rev)
    }
    pub fn read_dp_reg(&self, address: u8) -> Result<u32, Box<dyn std::error::Error>> {
        let commmand = SubCommand::RegR {
            address,
            is_dp: true,
        };
        let obuf = [0xE8, 4, 0, 0xA2, 0x22, 0, u8::from(commmand)];
        let mut ibuf = [0; 10];
        ch347::write(&obuf).unwrap();
        ch347::read(&mut ibuf).unwrap();
        let rev = u32::from(ibuf[5])
            | u32::from(ibuf[6]) << 8
            | u32::from(ibuf[7]) << 16
            | u32::from(ibuf[8]) << 24;
        Ok(rev)
    }

    fn write_reg(&self, address: u8, is_dp: bool, data: u32) -> Result<(), ()> {
        let command = SubCommand::RegW {
            address,
            is_dp,
            data,
        };
        let mut obuf = vec![0xE8, 9, 0x00];
        let mut ibuf = [0; 5];
        // 对 data 进行校验
        let mut count = 0u8;
        for i in 0..32 {
            if data >> i & 0x01 == 0x01 {
                count += 1;
            }
        }
        obuf.extend_from_slice(&[0xA0, 0x29, 0x00]);
        obuf.push(u8::from(command));
        obuf.extend_from_slice(&(data.to_le_bytes()));
        log::info!("data: {:#08x}, with parity: {}", data, count % 2);
        obuf.push(count % 2);
        ch347::write(&obuf).unwrap();
        ch347::read(&mut ibuf).unwrap();

        // check ack
        // TODO
        Ok(())
    }

    pub fn write_ap_reg(&self, address: u8, data: u32) -> Result<(), ()> {
        self.write_reg(address, false, data)?;
        Ok(())
    }
    pub fn write_dp_reg(&self, address: u8, data: u32) -> Result<(), ()> {
        self.write_reg(address, true, data)?;
        Ok(())
    }
}

#[derive(Debug, Clone, Copy)]
pub enum SubCommand {
    RegW { address: u8, is_dp: bool, data: u32 },
    RegR { address: u8, is_dp: bool },
}

impl Default for SubCommand {
    // default read idcode
    fn default() -> Self {
        Self::RegR {
            address: 0,
            is_dp: true,
        }
    }
}

impl SubCommand {
    pub fn is_read(&self) -> bool {
        match *self {
            SubCommand::RegR { .. } => true,
            SubCommand::RegW { .. } => false,
        }
    }
}

impl From<Command> for u8 {
    fn from(value: Command) -> Self {
        match value {
            Command::Ch347SwdInit => 0xE5,
            Command::Ch347Swd => 0xE8,
            Command::Ch347SwdRegW => 0xA0,
            Command::Ch347SwdSeqW => 0xA1,
            Command::Ch347SwdRegR => 0xA2,
        }
    }
}

impl From<SubCommand> for u8 {
    fn from(value: SubCommand) -> Self {
        let c = match value {
            SubCommand::RegR { address, is_dp } => {
                0b10000001 | (address << 3) | 0x04 | if is_dp { 0x00 } else { 0x02 }
            }
            SubCommand::RegW { address, is_dp, .. } => {
                0b10000001 | (address << 3) | 0x00 | if is_dp { 0x00 } else { 0x02 }
            }
        };
        let mut count = 0;
        for i in 1..=4 {
            if c >> i & 0x01 == 0x01 {
                count += 1;
            }
        }
        let c = if count % 2 != 0 { c | 0x20 } else { c };
        c
    }
}

// Init
// 0xE5
// 0x08
// 0x00
// 0x40
// 0x42
// 0x0f
// 0x00
// clockrate, same as jtag speed clockrate
// 0x00
// 0x00
// 0x00
