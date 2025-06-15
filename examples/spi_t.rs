use ch347_rs::{ch347, spi::Ch347SpiDevice};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    let device = ch347::Ch347UsbDevice::new()?;
    let spi = Ch347SpiDevice::new(&device);

    Ok(())
}
