use ch347_rs::{
    ch347, format_u8_array,
    gpio::Input,
    spi::{CSPin, SpiDevice, instance::Instance},
};

pub struct SPI;
impl Instance for SPI {}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    let p = ch347::init().unwrap();
    let io1 = Input::new(p.IO1);
    // let mut spi = SpiDevice::new(p.SPI0, Default::default());
    // let mut buf = [0xAA, 0xBB, 0xCC, 0xDD];
    // embedded_hal::spi::SpiDevice::transfer_in_place(&mut spi, &mut buf).unwrap();
    //
    // println!("{}", format_u8_array(&buf));
    SPI::cs_write(CSPin::CS0, false);
    println!("{:?}", io1.read());
    SPI::cs_write(CSPin::CS0, true);
    println!("{:?}", io1.read());

    Ok(())
}
