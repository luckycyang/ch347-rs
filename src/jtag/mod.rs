/// Clock 为一个时钟, Idle 是半个时钟并维持tms信号
pub enum Command {
    Idle { tms: bool },
    Clock { tms: bool, tdi: bool, capture: bool },
}

pub mod builder {
    use std::marker::PhantomData;

    use crate::jtag::builder;

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
        pub buf: Vec<u8>,
        _phantom: std::marker::PhantomData<S>,
    }

    // 通用方法实现
    impl<S: JtagState> JtagCtrlBuilder<S> {
        // 去除 buf， 并返回当前状态, 主要用于继续调试
        pub fn init(self) -> (Self, Vec<u8>) {
            let (p, data) = (self._phantom, self.buf);
            (
                Self {
                    buf: Vec::new(),
                    _phantom: p,
                },
                data,
            )
        }

        // 重置到Reset状态
        // 给5个时钟tms=1
        pub fn reset(self) -> JtagCtrlBuilder<Reset> {
            let mut buf = self.buf;
            for _ in 0..5 {
                (&mut buf).extend_from_slice(&[0x02, 0x03]);
            }
            JtagCtrlBuilder {
                buf: buf,
                _phantom: std::marker::PhantomData,
            }
        }

        // 写入单个bit
        pub fn shift_bit(mut self, tms: bool, tdi: bool) -> Self {
            let left = (u8::from(tms) << 1) | (u8::from(tdi) << 4);
            let right = left | 0x01;
            let _ = &mut self.buf.extend_from_slice(&[left, right]);
            self
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
            builder.buf.push(0x00);
            builder.buf.push(0x01);
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
            builder
                .buf
                .extend_from_slice(&[0x02, 0x03, 0x02, 0x03, 0x00, 0x01, 0x00, 0x01]);
            builder
        }

        pub fn enter_shiftdr(self) -> JtagCtrlBuilder<Shift> {
            let mut builder = JtagCtrlBuilder {
                buf: self.buf,
                _phantom: std::marker::PhantomData,
            };
            // 从Idle到ShiftDR需要TMS=1,0,0
            builder
                .buf
                .extend_from_slice(&[0x02, 0x03, 0x00, 0x01, 0x00, 0x01]);
            builder
        }
    }

    // Shift状态的特定实现
    impl JtagCtrlBuilder<Shift> {
        pub fn trans_bits(self, data: (&[u8], u8)) -> JtagCtrlBuilder<Exit> {
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

                    let byte = if (((i >> j) & 0x01) == 0x01) {
                        0x10
                    } else {
                        0x00
                    };
                    buf.push(byte | 0x00);
                    buf.push(byte | 0x01);

                    left -= 1;
                }
            }

            // last bit and enter exit tap
            let byte = if last == 0x01 { 0x10 } else { 0x00 };
            buf.push(byte | 0x02);
            buf.push(byte | 0x03);

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
            builder.buf.extend_from_slice(&[0x02, 0x03, 0x00, 0x01]);
            builder
        }
    }
}
