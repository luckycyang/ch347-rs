use ch347_rs::{ch347, format_u8_array, jtag::builder};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    // let p = ch347::init();
    let b = builder::Builder::new()
        .reset()
        .enter_idle()
        .enter_shiftir()
        .trans_bits((&[0b1110], 4))
        .enter_idle()
        .init();

    println!("{}", format_u8_array(&b));

    Ok(())
}
