use ch347_rs::{ch347, jtag};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    let _p = ch347::init();

    let mut jtag = jtag::Jtager::new();
    let idcodes = jtag.init().unwrap();

    for idcode in idcodes.iter().enumerate() {
        println!("Idcode of index {}: {:#08x}", idcode.0, idcode.1);
    }

    Ok(())
}
