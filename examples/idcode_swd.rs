use ch347_rs::ch347;

fn main() {
    env_logger::init();
    let p = ch347::init().unwrap();

    let mut buf = [0; 128];

    // init swd
    ch347::write(&[0xE5, 8, 0, 0x40, 0x42, 0x0f, 0x00, 7, 0x00, 0x00, 0x00]).unwrap();
    ch347::read(&mut buf).unwrap();

    // reset and enter idle
    ch347::write(&[
        0xE8, 10, 0x00, 0xA1, 56, 0x00, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x0f,
    ])
    .unwrap();
    ch347::read(&mut buf).unwrap();

    // read idcode
    // 校验位为 Apndp RnW A[3:2] 的偶校验，也可以说是对 [4:0] 的奇校验
    let obuf = [0xE8, 0x04, 0x00, 0xA2, 0x22, 0x00, 0b10100101];
    ch347::write(&obuf).unwrap();
    ch347::read(&mut buf).unwrap();

    ch347::write(&obuf).unwrap();
    ch347::read(&mut buf).unwrap();
}
