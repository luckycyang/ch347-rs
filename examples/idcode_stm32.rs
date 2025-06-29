use ch347_rs::ch347;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    let _p = ch347::init();
    let mut swd = ch347_rs::swd::SwdCommandSeq::new(3);
    swd.seq(&[0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff]);
    swd.seq(&(0xE79Eu16).to_le_bytes());
    swd.seq(&[0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff]);
    swd.seq(&[0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x00]);
    swd.push(Default::default());
    swd.flush();
    swd.push(Default::default());
    swd.flush();

    Ok(())
}
