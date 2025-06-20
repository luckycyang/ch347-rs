use ch347_rs::{
    ch347,
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
        .trans_bits((&[0b1111_1110, 0b01], 9))
        .enter_idle();

    jtag_state.update(&mut buf);
    let mut buf: Vec<u8> = Vec::new();

    let mut jtag_state = jtag_state
        .enter_shiftdr()
        .trans_bits((&[0xff, 0xff, 0xff, 0xff, 0xff], 33))
        .enter_idle();
    jtag_state.update(&mut buf);

    for i in 0..32 {
        idcode = idcode | (u32::from(buf[i]) << i);
    }

    println!("read idcode: 0x{:08X}", idcode);

    Ok(())
}
