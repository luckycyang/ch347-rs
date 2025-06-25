use ch347_rs::{
    ch347, format_u8_array,
    jtag::{
        Command,
        builder::{Builder, JtagState},
    },
};

fn main() {
    env_logger::init();
    let _p = ch347::init().unwrap();

    let mut buf = [0; 128];

    // init swd
    ch347::write(&[0xE5, 8, 0, 0x40, 0x42, 0x0f, 0x00, 0x00, 0x00, 0x00, 0x00]).unwrap();
    ch347::read(&mut buf).unwrap();

    let mut obuf = vec![0xE8, 11, 0, 0xA1, 64, 0];
    for _ in 0..7 {
        obuf.push(0xff);
    }
    obuf.push(0);
    ch347::write(&obuf).unwrap();
    ch347::read(&mut buf).unwrap();

    // read idcode
    let obuf = [0xE8, 0x04, 0x00, 0xA2, 0x22, 0x00, 0x81];
    ch347::write(&obuf).unwrap();
    ch347::read(&mut buf).unwrap();

    ch347::write(&obuf).unwrap();
    ch347::read(&mut buf).unwrap();
}
