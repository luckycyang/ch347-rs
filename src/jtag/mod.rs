/// Clock 为一个时钟
/// 或许我们需要半周期的 Idle
#[derive(Clone, Copy)]
pub enum Command {
    Clock { tms: bool, tdi: bool, capture: bool },
}

impl From<Command> for u8 {
    fn from(value: Command) -> Self {
        let Command::Clock { tms, tdi, capture } = value;
        (u8::from(tms) << 1) | (u8::from(tdi) << 4)
    }
}

pub mod builder {
    use super::Command;
    use std::{fmt::write, marker::PhantomData};

    use crate::{ch347, i2c::Config, jtag::builder};

    pub struct Unknown;
    pub struct Reset;
    pub struct Idle;
    pub struct Shift;
    pub struct Exit;

    // 为状态定义标记trait
    pub trait JtagState {}
    impl JtagState for Unknown {}
    impl JtagState for Reset {}
    impl JtagState for Idle {}
    impl JtagState for Shift {}
    impl JtagState for Exit {}

    pub struct Builder;
    impl Builder {
        pub fn new() -> JtagCtrlBuilder<Unknown> {
            JtagCtrlBuilder::<Unknown> {
                buf: Vec::new(),
                _phantom: PhantomData,
            }
        }
    }

    // Builder结构体，使用幽灵数据来跟踪状态
    pub struct JtagCtrlBuilder<S: JtagState> {
        pub buf: Vec<Command>,
        _phantom: std::marker::PhantomData<S>,
    }

    // 通用方法实现
    impl<S: JtagState> JtagCtrlBuilder<S> {
        pub fn add_command(&mut self, command: Command) {
            self.buf.push(command);
        }

        pub fn update(&mut self, ibuf: &mut Vec<u8>) {
            let len = self.buf.len();
            let mut buffer = [0; 4096];
            let mut obuf = vec![0xD2];
            let buf = (&self.buf)
                .iter()
                .fold(Vec::with_capacity(len * 2), |mut acc, &x| {
                    let byte = u8::from(x);
                    acc.push(byte);
                    acc.push(byte | 0x01);
                    acc
                });
            log::info!("command len: {}, obuf len: {}", len, len * 2);

            // maybe data len dont over 127
            obuf.push(buf.len() as u8);
            obuf.push(0);
            obuf.extend_from_slice(&buf);
            ch347::write(&obuf).unwrap();
            ch347::read(&mut buffer).unwrap();

            for (&c, &b) in self.buf.iter().zip(&buffer[3..]) {
                let Command::Clock { capture, .. } = c;
                if capture {
                    ibuf.push(b);
                }
            }
            // 清楚命令
            self.buf.clear();
        }

        // 重置到Reset状态
        // 给5个时钟tms=1
        pub fn reset(self) -> JtagCtrlBuilder<Reset> {
            let mut buf = self.buf;
            for _ in 0..5 {
                (&mut buf).push(Command::Clock {
                    tms: true,
                    tdi: true,
                    capture: false,
                });
            }
            JtagCtrlBuilder {
                buf: buf,
                _phantom: std::marker::PhantomData,
            }
        }
    }

    // Reset状态的特定实现
    impl JtagCtrlBuilder<Reset> {
        pub fn enter_idle(self) -> JtagCtrlBuilder<Idle> {
            let mut builder = JtagCtrlBuilder {
                buf: self.buf,
                _phantom: std::marker::PhantomData,
            };
            // 从Reset到Idle需要TMS=0
            builder.buf.push(Command::Clock {
                tms: false,
                tdi: true,
                capture: false,
            });
            builder
        }
    }

    // Idle状态的特定实现
    impl JtagCtrlBuilder<Idle> {
        pub fn enter_shiftir(self) -> JtagCtrlBuilder<Shift> {
            let mut builder = JtagCtrlBuilder {
                buf: self.buf,
                _phantom: std::marker::PhantomData,
            };
            // 从Idle到ShiftIR需要TMS=1,1,0,0
            for _ in 0..2 {
                builder.buf.push(Command::Clock {
                    tms: true,
                    tdi: true,
                    capture: false,
                });
            }
            for _ in 0..2 {
                builder.buf.push(Command::Clock {
                    tms: false,
                    tdi: true,
                    capture: false,
                });
            }
            builder
        }

        pub fn enter_shiftdr(self) -> JtagCtrlBuilder<Shift> {
            let mut builder = JtagCtrlBuilder {
                buf: self.buf,
                _phantom: std::marker::PhantomData,
            };
            // 从Idle到ShiftDR需要TMS=1,0,0
            builder.buf.push(Command::Clock {
                tms: true,
                tdi: false,
                capture: false,
            });
            for _ in 0..2 {
                builder.buf.push(Command::Clock {
                    tms: false,
                    tdi: true,
                    capture: false,
                });
            }
            builder
        }
    }

    // Shift状态的特定实现
    impl JtagCtrlBuilder<Shift> {
        pub fn trans_bits(self, data: (&[u8], u32)) -> JtagCtrlBuilder<Exit> {
            let mut buf = self.buf;
            let mut left = data.1;
            let mut last = 0x00;

            // trans N - 1 bits
            for &i in data.0.iter() {
                if left == 1 {
                    last = i & 0x1;
                    break;
                }
                for j in 0..8 {
                    if left == 1 {
                        last = i >> j & 0x01;
                        break;
                    }

                    let byte = if ((i >> j) & 0x01) == 0x01 {
                        Command::Clock {
                            tms: false,
                            tdi: true,
                            capture: true,
                        }
                    } else {
                        Command::Clock {
                            tms: false,
                            tdi: false,
                            capture: true,
                        }
                    };
                    buf.push(byte);

                    left -= 1;
                }
            }

            // last bit and enter exit tap
            let byte = if last == 0x01 {
                Command::Clock {
                    tms: true,
                    tdi: true,
                    capture: true,
                }
            } else {
                Command::Clock {
                    tms: true,
                    tdi: false,
                    capture: true,
                }
            };
            buf.push(byte);

            JtagCtrlBuilder {
                buf,
                _phantom: std::marker::PhantomData,
            }
        }
    }

    // Exit状态的特定实现
    impl JtagCtrlBuilder<Exit> {
        pub fn enter_idle(self) -> JtagCtrlBuilder<Idle> {
            let mut builder = JtagCtrlBuilder {
                buf: self.buf,
                _phantom: std::marker::PhantomData,
            };
            // 从Exit到Idle需要TMS=1,0
            builder.buf.push(Command::Clock {
                tms: true,
                tdi: true,
                capture: false,
            });
            builder.buf.push(Command::Clock {
                tms: false,
                tdi: true,
                capture: false,
            });
            builder
        }
    }
}
