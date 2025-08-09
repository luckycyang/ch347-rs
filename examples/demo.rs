use bitvec::field::BitField;
use ch347_rs::{ch347, jtag};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    let _p = ch347::init();

    let mut jtag = jtag::Jtager::new();
    let idcodes = jtag.init().unwrap();

    for idcode in idcodes.iter().enumerate() {
        println!("Idcode of index {}: {:#08x}", idcode.0, idcode.1);
    }

    // 如果 Tap 连接顺序是 tap0 -> tap1 -> tap2, 由于 tap2 先被推出来， 所以 0 选择的是 tap2
    jtag.select_target(0).unwrap();

    let _ = jtag.write_ir(0x0E, 4).unwrap();
    let idcode = jtag.write_dr(0xffff_ffff, 32).unwrap().load_le::<u32>();
    println!("read idcode: {idcode:#16x}");

    //  0x00 是没有东西的
    let dp00 = jtag.register_cmd(jtag::Register::Dp(0x00), None).unwrap();
    println!("DP(0x00): {dp00:#08x}");

    // 似乎没有 abort 寄存器，那我只能直接写 CTRL/STAT 使能 Ap 了
    let _ = jtag
        .register_cmd(jtag::Register::Dp(0x04), Some(0x5000_0000))
        .unwrap();

    // 期望是 0xf0000_0000
    let ctrl_stat = jtag.register_cmd(jtag::Register::Dp(0x04), None).unwrap();
    println!("DP CTRL/STAT: {ctrl_stat:#08x}");

    // 写 bank 0xF0, 因为 APIDR 在 0xFC
    let select_reg = jtag
        .register_cmd(jtag::Register::Dp(0x08), Some(0x0000_00F0))
        .unwrap();
    println!("DP SELECT: {select_reg:#08x}");

    let ap_idr = jtag.register_cmd(jtag::Register::Ap(0x0C), None).unwrap();
    println!("AP IDR: {ap_idr:#08x}");

    Ok(())
}
