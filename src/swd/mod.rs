enum Command {
    Ch347SwdInit,
    Ch347Swd,
    Ch347SwdRegW,
    Ch347SwdSeqW,
    Ch347SwdRegR,
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
