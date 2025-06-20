use ch347_rs::{ch347, format_u8_array, jtag::builder};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    let mut ibuf = [0; 128];
    let p = ch347::init();
    let (s, b) = builder::Builder::new()
        .reset()
        .enter_idle()
        .enter_shiftir()
        .trans_bits((&[0b1111_1110, 0b01], 9))
        .enter_idle()
        .init();

    let mut buf = vec![0xD2, b.len() as u8, 0x00];
    buf.extend_from_slice(&b[..]);
    ch347::write(&buf).unwrap();
    ch347::read(&mut ibuf);

    let (s, b) = s
        .enter_shiftdr()
        .trans_bits((&[0xFF, 0xFF, 0xFF, 0xFF, 0xFF], 33))
        .enter_idle()
        .init();

    let mut buf = vec![0xD2, b.len() as u8, 0x00];
    buf.extend_from_slice(&b[..]);
    ch347::write(&buf).unwrap();
    ch347::read(&mut ibuf);

    Ok(())
}
