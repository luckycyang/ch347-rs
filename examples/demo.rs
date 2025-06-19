use ch347_rs::{ch347, format_u8_array, spi::instance::Instance};

pub struct SPI;
impl Instance for SPI {}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    let _p = ch347::init().unwrap();
    SPI::set_config(Default::default());

    let mut ibuf = [0; 4];
    let obuf = [0xAA, 0xBB, 0xCC, 0xDD];
    SPI::write_and_read(&mut ibuf, &obuf);
    println!("rev: {}", format_u8_array(&ibuf));

    Ok(())
}
