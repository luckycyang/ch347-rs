use ch347_rs::{
    ch347, format_u8_array,
    jtag::builder::{self, Builder},
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    let mut ibuf = [0; 128];
    let _p = ch347::init();
    let mut buf: Vec<u8> = Vec::new();

    let mut idcode = 0;

    let mut jtag_state = Builder::new()
        .reset()
        .enter_idle()
        .enter_shiftir()
        .trans_bits((&[0xff, 0xff, 0x7f, 0xff], 32))
        .enter_idle();

    jtag_state.update(&mut buf);
    println!("sshift ir rev: {}", format_u8_array(&buf));
    buf.clear();

    let mut jtag_state = jtag_state
        .enter_shiftdr()
        .trans_bits((&[0xff, 0xff, 0xff, 0xff, 0xff], 33))
        .enter_idle();
    jtag_state.update(&mut buf);

    for i in 0..32 {
        idcode = idcode | (u32::from(buf[i]) << i);
    }
    println!("read idcode: 0x{:08X}", idcode);
    buf.clear();

    let mut jtag_state = Builder::new()
        .reset()
        .enter_idle()
        .enter_shiftir()
        .trans_bits((&[0xff, 0xff, 0xff, 0xff], 32))
        .enter_idle();

    jtag_state.update(&mut buf);
    println!("sshift ir rev: {}", format_u8_array(&buf));
    buf.clear();

    Ok(())
}
