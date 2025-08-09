use std::ptr::read;

use crate::ch347;
use bitvec::{field::BitField, vec::BitVec};

// 不带 bank, 自个处理 bank 的问题
#[derive(Debug)]
pub enum Register {
    Ap(u8),
    Dp(u8),
}

impl Register {
    fn is_ap(&self) -> bool {
        match self {
            Self::Ap(_) => true,
            _ => false,
        }
    }

    fn l2_l3(&self) -> u8 {
        match self {
            Self::Ap(addr) => *addr,
            Self::Dp(addr) => *addr,
        }
    }
}

/// Clock 为一个时钟
#[derive(Clone, Copy)]
struct Clock {
    tms: bool,
    tdi: bool,
    capture: bool,
}

impl From<Clock> for u8 {
    fn from(value: Clock) -> Self {
        let Clock { tms, tdi, .. } = value;
        (u8::from(tms) << 1) | (u8::from(tdi) << 4)
    }
}

#[derive(Default)]
struct TapInfo {
    taps: Vec<usize>, // 记录的是 Tap 的 IR 长度
    ir_pre: usize,    // 标识选择的Tap, IR命令前面需要填充的数量
    ir_pos: usize,    // 后，填充 1, 那么其他就是 Bypass
    pre: usize,
    pos: usize,
}

pub struct Jtager {
    taparam: TapInfo,
    bits: BitVec,
    clocks: Vec<Clock>,
}

impl Jtager {
    pub fn new() -> Self {
        ch347::write(&[0xD0, 0x06, 0x00, 0x00, 4, 0x00, 0x00, 0x00, 0x00]).unwrap();
        ch347::read(&mut [0; 4]).unwrap();
        Self {
            taparam: Default::default(),
            bits: BitVec::new(),
            clocks: Vec::new(),
        }
    }

    fn shift_bits(&mut self, tms: bool, tdi: bool, capture: bool) {
        if self.clocks.len() > 127 {
            self.flush();
        }

        self.clocks.push(Clock { tms, tdi, capture });
    }

    fn flush(&mut self) {
        let mut buffer = [0; 130];
        let mut obuf = vec![];
        let mut command = vec![0xD2];

        for &i in self.clocks.iter() {
            let byte = u8::from(i);
            // the byte is clock low, bit 0 = 1 that clock high
            obuf.push(byte);
            obuf.push(byte | 0x01);
        }
        command.extend_from_slice(&(obuf.len() as u16).to_le_bytes());
        command.extend_from_slice(&obuf);

        ch347::write(&command).unwrap();
        ch347::read(&mut buffer).unwrap();

        for (&c, &byte) in self.clocks.iter().zip(&buffer[3..]) {
            let Clock { capture, .. } = c;
            if capture {
                self.bits.push(byte != 0x00);
            }
        }

        self.clocks.clear();
    }

    fn read_capturd_bits(&mut self) -> Result<BitVec, String> {
        self.flush();
        Ok(std::mem::take(&mut self.bits))
    }

    // 必须要作为最开始调用的函数
    // 复位并进入Idle, 注意，我主要用于验证，这里不会记录状态机，主要保证每次Jtag操作回到Idle状态
    fn reset_idle(&mut self) {
        for i in [true, true, true, true, true, false] {
            self.shift_bits(i, true, false);
        }
    }

    // 从 Idle 进入 ShiftDR/ShiftIR
    fn enter_shift(&mut self, shiftdr: bool) {
        if !shiftdr {
            self.shift_bits(true, true, false);
        }

        for i in [true, false, false] {
            self.shift_bits(i, true, false);
        }
    }

    // 扫描 IDCODES, 复位后， 进入DR扫描IDCODE
    fn idcode_scan(&mut self) -> Vec<u32> {
        let mut idcodes = Vec::new();
        let mut end = false;

        // 进入 ShiftDR
        self.enter_shift(true);

        // 一次只拿一个 u64
        while !end {
            for _ in 0..32 {
                self.shift_bits(false, true, true);
            }

            let idcode = self.read_capturd_bits().unwrap().load_le::<u32>();
            if idcode != 0xffff_ffff {
                idcodes.push(idcode);
            } else {
                end = true;
            }
        }

        // 此时还在 ShiftDR, 直接跳回 Idle, tms = 1 1 0
        for i in [true, true, false] {
            self.shift_bits(i, true, false);
        }

        idcodes
    }

    // 扫描 taps, 注意也是复位后的操作，我测试无法调用第二次, 因为有些 IR 不会一直是复位值
    fn scan_taps(&mut self) {
        // 去 ShiftIR
        self.enter_shift(false);

        self.shift_bits(false, true, true);
        let mut pre = self.read_capturd_bits().unwrap()[0];
        let mut status = 1;

        while status > 0 {
            self.shift_bits(false, true, true);
            let pos = self.read_capturd_bits().unwrap()[0];

            if pos == false {
                status += 1;
                pre = pos;
            } else if pre != pos {
                self.taparam.taps.push(status);
                status = 1;
                pre = pos;
            } else {
                status = 0;
            }
        }

        log::info!("taps: {:?}", self.taparam.taps);

        // 回到 Idle
        // 此时还在 ShiftIR, 直接跳回 Idle, tms = 1 1 0
        for i in [true, true, false] {
            self.shift_bits(i, true, false);
        }
    }

    // 从 exit1 返回 idle
    fn exit_idle(&mut self) {
        for i in [true, false] {
            self.shift_bits(i, true, false);
        }
    }

    // 选一个 Tap 操作
    pub fn select_target(&mut self, target: usize) -> Result<(), String> {
        if target > self.taparam.taps.len() {
            return Err("索引过界，确保你选择正确的序位".into());
        }

        let mut ir_pre = 0;
        let mut ir_pos = 0;
        let mut pre = 0;
        let mut pos = 0;

        for (index, &ir_len) in self.taparam.taps.iter().enumerate() {
            if index < target {
                ir_pre += ir_len;
                pre += 1;
            } else if index == target {
                continue;
            } else {
                ir_pos += ir_len;
                pos += 1;
            }
        }

        self.taparam.ir_pre = ir_pre;
        self.taparam.ir_pos = ir_pos;
        self.taparam.pre = pre;
        self.taparam.pos = pos;

        Ok(())
    }

    // 此函数应该在 Shift 状态被调用
    // 填充位， 对于IR就是把非操作TAP写入Bypass指令
    // 对于DR就是把数据推到口子上
    fn fill_pre(&mut self, is_ir: bool) {
        let nums = if is_ir {
            self.taparam.ir_pre
        } else {
            self.taparam.pre
        };

        for _ in 0..nums {
            self.shift_bits(false, true, false);
        }
    }

    // 此函数应该在 Shift 状态被调用
    // 填充位， 对于IR就是把非操作TAP写入Bypass指令
    // 对于DR就是把数据完整推出来
    fn fill_pos(&mut self, is_ir: bool) {
        let nums = if is_ir {
            self.taparam.ir_pos
        } else {
            self.taparam.pos
        };

        // 没有就直接返回了
        if nums == 0 {
            return;
        }

        // 对于 IR 而言， 长度至少是 2, 这里至少会循环1次
        for _ in 0..nums - 1 {
            self.shift_bits(false, true, false);
        }

        // 最后一位填充位用于跳出 Shift
        if nums != 0 {
            self.shift_bits(true, true, false);
        }
    }

    // 写 IR 寄存器, 哥们问了 GPT, 最长不应该超过 32
    pub fn write_ir(&mut self, cmd: u32, ir_len: usize) -> Result<BitVec, String> {
        // 先进入 shiftir
        self.enter_shift(false);

        self.fill_pre(true);

        for i in 0..ir_len - 1 {
            self.shift_bits(false, cmd >> i & 1 == 1, true);
        }

        // 如果 pos == 0, 没有填充， 那就得跳出 Shift 状态了
        // 显然如果不再边界， 每次写IR都需要判断， 这会不会效率过低
        if self.taparam.pos != 0 {
            self.shift_bits(false, cmd >> (ir_len - 1) & 1 == 1, true);
        } else {
            self.shift_bits(true, cmd >> (ir_len - 1) & 1 == 1, true);
        }

        // 有没有 pos 都没关系
        self.fill_pos(true);

        self.exit_idle();

        self.read_capturd_bits()
    }

    // 写 DR 寄存器, 当前单个 tap 而言应该够了用 u64
    pub fn write_dr(&mut self, data: u64, dr_len: usize) -> Result<BitVec, String> {
        // 先进入 shiftir
        self.enter_shift(true);

        self.fill_pre(false);

        for i in 0..dr_len - 1 {
            self.shift_bits(false, data >> i & 1 == 1, true);
        }

        // 如果 pos == 0, 没有填充， 那就得跳出 Shift 状态了
        if self.taparam.pos != 0 {
            self.shift_bits(false, data >> (dr_len - 1) & 1 == 1, true);
        } else {
            self.shift_bits(true, data >> (dr_len - 1) & 1 == 1, true);
        }

        // 有没有 pos 都没关系
        self.fill_pos(false);

        self.exit_idle();

        self.read_capturd_bits()
    }

    // 高阶接口， 操作 Ap 和 Dp
    // 确保你已经选择正确的 tap, 这里不会帮助你选择 tap, 请注意选择正确的 Tap, 对于 stm32 不要选到
    // arm 那个 tap
    pub fn register_cmd(&mut self, address: Register, value: Option<u32>) -> Result<u32, String> {
        // 对于 swd 操作 Dp, 可以立即返回结果, ap 延时下一次返回，在jtag这里dp和ap都是延时返回

        let _ = self
            .write_ir(if address.is_ap() { 0x0B } else { 0x0A }, 4)
            .unwrap();

        if value.is_none() {
            // 没带 value 就是读数据
            let _ = self
                .write_dr(u64::from(address.l2_l3() >> 1) | 1, 35)
                .unwrap();
            // 拿到原始数据
            let raw = self
                .write_dr(u64::from(address.l2_l3() >> 1) | 1, 35)
                .unwrap();

            let data = raw.load_le::<u64>();
            let ack = data & 0b111;
            if ack != 2 {
                Err(format!(
                    "Read data from {address:?} error with ack: {ack:#03b}"
                ))
            } else {
                Ok((data >> 3) as u32)
            }
        } else {
            let data = value.unwrap();
            let _ = self
                .write_dr(
                    u64::from(data) << 3 | u64::from(address.l2_l3() >> 1) | 0,
                    35,
                )
                .unwrap();
            let raw = self
                .write_dr(
                    u64::from(data) << 3 | u64::from(address.l2_l3() >> 1) | 0,
                    35,
                )
                .unwrap();
            let data = raw.load_le::<u64>();
            let ack = data & 0b111;
            if ack != 2 {
                Err(format!(
                    "Write data: {data:#08x} to {address:?} error with ack: {ack:#03b}"
                ))
            } else {
                // 写操作会也会有值
                Ok((data >> 3) as u32)
            }
        }
    }

    // 做初始化，并扫描 IDCODE 和 Taps, 返回 idcodes, tap 信息保存在此结构体
    pub fn init(&mut self) -> Result<Vec<u32>, String> {
        self.reset_idle();
        let idcodes = self.idcode_scan();
        self.scan_taps();

        Ok(idcodes)
    }
}
