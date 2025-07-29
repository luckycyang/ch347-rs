use ch347_rs::ch347;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    let _p = ch347::init();
    let mut swd = ch347_rs::swd::SwdCommandSeq::new(3);
    swd.reset();
    swd.jtag_to_swd();
    swd.reset_and_idle();
    println!("id code: {:#08x}", swd.read_dp_reg(00).unwrap());

    // write abort for clear err
    swd.write_dp_reg(0, 0x01e).unwrap();

    // enable debug port
    swd.write_dp_reg(1, 0x50000000).unwrap();

    // check, value like 0xF0000000;
    println!("CTRL/STAT: {:#08x}", swd.read_dp_reg(1).unwrap());

    // write dp select to select MEM-AP and bank 0xF include IDR Reg
    swd.write_dp_reg(2, 0x0f0).unwrap();

    println!("first read IDR: {:#08x}", swd.read_ap_reg(3).unwrap());
    println!("DP RDBUUF: {:#08x}", swd.read_dp_reg(3).unwrap());

    // ready to read ap CSW(0x00)
    println!("ready to read ap CSW(0x00)");
    swd.write_dp_reg(2, 0x0).unwrap();
    println!("first raed ap CSW: {:#08x}", swd.read_ap_reg(0).unwrap());
    println!("read RDBUFF: {:#08x}", swd.read_dp_reg(3).unwrap());
    println!("second read ap CSW: {:#08x}", swd.read_ap_reg(0).unwrap());
    println!("read dp RESEND(0x8): {:#08x}", swd.read_dp_reg(2).unwrap());

    // write CSW
    println!("write CSW and read");
    swd.write_dp_reg(2, 0x0).unwrap();
    swd.write_ap_reg(0x0, 0xeb000052).unwrap();
    println!("first read CSW: {:#08x}", swd.read_ap_reg(0).unwrap());
    println!("read RDBUFF: {:#08x}", swd.read_dp_reg(3).unwrap());

    Ok(())
}
