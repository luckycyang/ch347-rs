use ch347_rs::{ch347::Ch347UsbDevice, iic::IicDevice};

fn set_column(iic: &mut IicDevice, column: u8) {
    assert!(column < 128);
    iic.write_with_address(&[0x00, 0x00 | (column & 0x0F)], 0x78)
        .unwrap();
    iic.write_with_address(&[0x00, 0x10 | ((column & 0xF0) >> 4)], 0x78)
        .unwrap();
}

fn set_line(iic: &mut IicDevice, line: u8) {
    assert!(line < 8);
    iic.write_with_address(&[0x00, 0xB0 | line], 0x78).unwrap();
}

fn fill(iic: &mut IicDevice, data: u8) {
    for i in 0..8 {
        let mut buf = Vec::new();
        buf.push(0x40);
        buf.extend_from_slice(&[data; 128]);
        set_line(iic, i);
        set_column(iic, 0);
        iic.write_with_address(&buf, 0x78).unwrap();
    }
}

fn main() {
    env_logger::init();
    let ch347 = Ch347UsbDevice::new().unwrap();
    let mut iic = IicDevice::new(&ch347).unwrap();
    iic.set_speed(ch347_rs::iic::Ch347IicSpeed::Khz200).unwrap();
    let commands = [
        0xAE, // display off
        0x00, //  column low
        0x10, //  column high
        0x40, // start line
        0x81, 0xCF, 0xA1, 0xC8, 0xA6, 0xA8, 0x3F, 0xD3, 0x00, 0xD5, 0x80, 0xD9, 0xF1, 0xDA, 0x12,
        0xD8, 0x30, 0x20, 0x02, 0x8D, 0x14, 0xAF,
    ];

    for i in commands {
        iic.write_with_address(&[0x00, i], 0x78).unwrap();
    }
    let mut state: u8 = 0x18;
    loop {
        // state = state.wrapping_add(1);
        fill(&mut iic, state);
    }
}
