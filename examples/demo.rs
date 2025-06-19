use ch347_rs::{
    ch347,
    i2c::{Config, I2cbus},
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let p = ch347::init().unwrap();
    let mut buf = [0; 16];
    // ch347::write(&[0xAA, 0x74, 0x81, 0x3C << 1, 0x75, 0x00]).unwrap();
    ch347::write(&[
        0xE2, 0x08, 0x00, 0x00, 0x00, 0x81, 0x81, 0x00, 0x00, 0x00, 0x00,
    ])
    .unwrap();
    ch347::read(&mut buf).unwrap();
    println!("{:?}", &buf);

    ch347::write(&[
        0xaa, 0x74, 0x89, 0x78, 0x40, 0xff, 0x00, 0xff, 0x00, 0xff, 0x00, 0xff, 0x75, 0x00,
    ])
    .unwrap();
    ch347::read(&mut buf).unwrap();
    println!("{:?}", &buf);

    Ok(())
}
