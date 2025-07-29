use ch347_rs::{self, ch347};

fn main() {
    let p = ch347::init().unwrap();
    ch347::write(&[0xAA, 0x74, 0x82, 0xD0, 0x75, 0x75, 0x00]).unwrap();
    let mut buf = [0; 4];
    ch347::read(&mut buf).unwrap();
    println!("{:?}", &buf);
    ch347::write(&[0xAA, 0x74, 0x81, 0xD1, 0xC1, 0x75, 0x00]).unwrap();
    ch347::read(&mut buf).unwrap();
    println!("{:?}", &buf);
}
