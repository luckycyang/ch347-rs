use ch347_rs::ch347;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    let _p = ch347::init();
    let swd = ch347_rs::swd::SwdCommandSeq::new(3);
    swd.reset();
    swd.jtag_to_swd();
    swd.reset_and_idle();
    println!("id code: {:#08x}", swd.read_dp_reg(00).unwrap());

    // write abort for clear err
    println!("write dp 0");
    swd.write_dp_reg(0, 0x01e).unwrap();

    // enable debug port
    println!("write dp 1");
    swd.write_dp_reg(1, 0x50000000).unwrap();

    // check, value like 0xF0000000;
    println!("read dp 1");
    println!("CTRL/STAT: {:#08x}", swd.read_dp_reg(1).unwrap());

    // write dp select to select MEM-AP and bank 0xF include IDR Reg
    println!("write dp 2");
    swd.write_dp_reg(2, 0x0f0).unwrap();

    println!("first read IDR: {:#08x}", swd.read_ap_reg(3).unwrap());
    // 对于 AP 而言， 读的结果在下条指令返回
    println!("second read IDR: {:#08x}", swd.read_ap_reg(3).unwrap());

    // 或者读 RDBUFF 0x0C
    // swd.write_dp_reg(2, 0x00).unwrap();
    // println!("second read IDR: {:#08x}", swd.read_ap_reg(3).unwrap());

    Ok(())
}
