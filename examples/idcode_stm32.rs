use ch347_rs::ch347;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    let _p = ch347::init();
    let swd = ch347_rs::swd::SwdCommandSeq::new(3);
    swd.jtag_to_swd();
    swd.reset_and_idle();
    let rev = swd.read_dp_reg(00).unwrap();
    println!("read idcode; {:#08x}", rev);

    Ok(())
}
