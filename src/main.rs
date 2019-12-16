use self::{MCycle::*, Phase::*};

fn main() {
    println!("Hello, world!");
}

#[derive(Default)]
pub struct Cpu {
    regs: Regs,
    ctrl: RunningState,
}

struct Regs {
    a: u8,
    b: u8,
    c: u8,
    d: u8,
    e: u8,
    h: u8,
    l: u8,
    pc: u16,
    sp: u16,
}

struct RunningState {
    opcode: Opcode,
    m_cycle: MCycle,
}

#[derive(Clone, Copy)]
struct Opcode(u8);

impl Opcode {
    fn split(self) -> (u8, u8, u8) {
        (self.0 >> 6, (self.0 >> 3) & 0b111, self.0 & 0b111)
    }
}

#[derive(Clone, Copy)]
enum MCycle {
    M1,
    M2,
    M3,
    M4,
}

impl MCycle {
    #[inline(always)]
    fn next(self) -> Self {
        match self {
            MCycle::M1 => MCycle::M2,
            MCycle::M2 => MCycle::M3,
            MCycle::M3 => MCycle::M4,
            MCycle::M4 => panic!(),
        }
    }
}

impl Default for Regs {
    fn default() -> Self {
        Self {
            a: 0x00,
            b: 0x00,
            c: 0x00,
            d: 0x00,
            e: 0x00,
            h: 0x00,
            l: 0x00,
            pc: 0x0000,
            sp: 0x0000,
        }
    }
}

impl Default for RunningState {
    fn default() -> Self {
        Self {
            opcode: Opcode(0x00),
            m_cycle: MCycle::M1,
        }
    }
}

impl Regs {
    fn hl(&self) -> u16 {
        (u16::from(self.h) << 8) + u16::from(self.l)
    }

    fn reg(&mut self, r: R) -> &mut u8 {
        match r {
            R::A => &mut self.a,
            R::B => &mut self.b,
            R::C => &mut self.c,
            R::D => &mut self.d,
            R::E => &mut self.e,
            R::H => &mut self.h,
            R::L => &mut self.l,
        }
    }
}

impl Cpu {
    pub fn tick(&mut self) -> Option<BusOp> {
        self.exec_instr(CpuInput {
            phase: Phase::Tick,
            data: None,
        })
    }

    pub fn tock(&mut self, data: Option<u8>) {
        assert_eq!(
            self.exec_instr(CpuInput {
                phase: Phase::Tock,
                data,
            }),
            None
        )
    }

    #[inline(always)]
    fn exec_instr(&mut self, input: CpuInput) -> Option<BusOp> {
        match self.ctrl.opcode.split() {
            (0b00, 0b000, 0b000) => self.nop(input),
            (0b01, 0b110, 0b110) => self.halt(input),
            (0b01, 0b110, src) => self.ld_deref_hl_r(src.into(), input),
            (0b01, dest, 0b110) => self.ld_r_deref_hl(dest.into(), input),
            (0b01, dest, src) => self.ld_r_r(dest.into(), src.into(), input),
            (0b11, 0b001, 0b001) => self.ret(input),
            _ => unimplemented!(),
        }
    }

    fn nop(&mut self, input: CpuInput) -> Option<BusOp> {
        self.fetch(input)
    }

    fn ld_r_r(&mut self, dest: R, src: R, input: CpuInput) -> Option<BusOp> {
        match (self.ctrl.m_cycle, input.phase) {
            (M1, Tick) => self.fetch(input),
            (M1, Tock) => {
                let value = *self.regs.reg(src);
                *self.regs.reg(dest) = value;
                self.fetch(input)
            }
            _ => unreachable!(),
        }
    }

    fn ld_r_deref_hl(&mut self, dest: R, input: CpuInput) -> Option<BusOp> {
        match (self.ctrl.m_cycle, input.phase) {
            (M1, Tick) => Some(BusOp::Read(self.regs.hl())),
            (M1, Tock) => {
                *self.regs.reg(dest) = input.data.unwrap();
                self.advance(input)
            }
            (M2, _) => self.fetch(input),
            _ => unreachable!(),
        }
    }

    fn ld_deref_hl_r(&mut self, src: R, input: CpuInput) -> Option<BusOp> {
        match (self.ctrl.m_cycle, input.phase) {
            (M1, Tick) => Some(BusOp::Write(self.regs.hl(), *self.regs.reg(src))),
            (M1, Tock) => self.advance(input),
            (M2, _) => self.fetch(input),
            _ => unreachable!(),
        }
    }

    fn halt(&mut self, _input: CpuInput) -> Option<BusOp> {
        unimplemented!()
    }

    fn ret(&mut self, input: CpuInput) -> Option<BusOp> {
        match (self.ctrl.m_cycle, input.phase) {
            (M1, Tick) => Some(BusOp::Read(self.regs.sp)),
            (M1, Tock) => {
                self.regs.pc = input.data.unwrap().into();
                self.regs.sp += 1;
                self.advance(input)
            }
            (M2, Tick) => Some(BusOp::Read(self.regs.sp)),
            (M2, Tock) => {
                self.regs.pc |= u16::from(input.data.unwrap()) << 8;
                self.regs.sp += 1;
                self.advance(input)
            }
            (M3, _) => self.advance(input),
            (M4, _) => self.fetch(input),
        }
    }

    fn advance(&mut self, input: CpuInput) -> Option<BusOp> {
        if let Tock = input.phase {
            self.ctrl.m_cycle = self.ctrl.m_cycle.next()
        }
        None
    }

    fn fetch(&mut self, input: CpuInput) -> Option<BusOp> {
        match input.phase {
            Phase::Tick => Some(BusOp::Read(self.regs.pc)),
            Phase::Tock => {
                self.ctrl.opcode = Opcode(input.data.unwrap());
                self.regs.pc += 1;
                self.ctrl.m_cycle = MCycle::M1;
                None
            }
        }
    }
}

struct CpuInput {
    phase: Phase,
    data: Option<u8>,
}

#[derive(Clone, Copy)]
enum R {
    A,
    B,
    C,
    D,
    E,
    H,
    L,
}

impl From<u8> for R {
    fn from(encoding: u8) -> Self {
        match encoding {
            0b000 => R::B,
            0b001 => R::C,
            0b010 => R::D,
            0b011 => R::E,
            0b100 => R::H,
            0b101 => R::L,
            0b111 => R::A,
            _ => panic!(),
        }
    }
}

#[derive(Clone, Copy)]
enum Phase {
    Tick,
    Tock,
}

#[derive(Debug, PartialEq)]
pub enum BusOp {
    Read(u16),
    Write(u16, u8),
}

#[cfg(test)]
mod tests {
    use super::*;

    const RS: &[R] = &[R::A, R::B, R::C, R::D, R::E, R::H, R::L];

    #[test]
    fn ld_r_r() {
        for &dest in RS {
            for &src in RS {
                test_ld_r_r(dest, src)
            }
        }
    }

    fn test_ld_r_r(dest: R, src: R) {
        let mut cpu = Cpu::default();
        let expected = 0x42;
        cpu.ctrl.opcode = encode_ld_r_r(dest, src);
        *cpu.regs.reg(src) = expected;
        assert_eq!(cpu.tick(), Some(BusOp::Read(0x0000)));
        cpu.tock(Some(0x00));
        assert_eq!(*cpu.regs.reg(dest), expected)
    }

    fn encode_ld_r_r(dest: R, src: R) -> Opcode {
        Opcode(0b01_000_000 | (dest.code() << 3) | src.code())
    }

    impl R {
        fn code(self) -> u8 {
            match self {
                R::A => 0b111,
                R::B => 0b000,
                R::C => 0b001,
                R::D => 0b010,
                R::E => 0b011,
                R::H => 0b100,
                R::L => 0b101,
            }
        }
    }

    #[test]
    fn ld_r_deref_hl() {
        for &dest in RS {
            test_ld_r_deref_hl(dest)
        }
    }

    fn test_ld_r_deref_hl(dest: R) {
        let mut cpu = Cpu::default();
        let expected = 0x42;
        cpu.regs.h = 0x12;
        cpu.regs.l = 0x34;
        cpu.ctrl.opcode = encode_ld_r_deref_hl(dest);
        assert_eq!(cpu.tick(), Some(BusOp::Read(0x1234)));
        cpu.tock(Some(expected));
        assert_eq!(cpu.tick(), Some(BusOp::Read(0x0000)));
        cpu.tock(Some(0x00));
        assert_eq!(*cpu.regs.reg(dest), expected)
    }

    fn encode_ld_r_deref_hl(dest: R) -> Opcode {
        Opcode(0b01_000_110 | (dest.code() << 3))
    }

    #[test]
    fn ld_deref_hl_r() {
        for &src in RS {
            test_ld_deref_hl_r(src)
        }
    }

    fn test_ld_deref_hl_r(src: R) {
        let mut cpu = Cpu::default();
        let expected = 0x42;
        *cpu.regs.reg(src) = expected;
        cpu.regs.h = 0x12;
        cpu.regs.l = 0x34;
        *cpu.regs.reg(src) = expected;
        cpu.ctrl.opcode = encode_ld_deref_hl_r(src);
        assert_eq!(cpu.tick(), Some(BusOp::Write(cpu.regs.hl(), expected)));
        cpu.tock(None);
        assert_eq!(cpu.tick(), Some(BusOp::Read(0x0000)));
        cpu.tock(Some(0x00));
    }

    fn encode_ld_deref_hl_r(src: R) -> Opcode {
        Opcode(0b01_110_000 | src.code())
    }

    #[test]
    fn ret() {
        let mut cpu = Cpu::default();
        cpu.regs.sp = 0x1234;
        cpu.ctrl.opcode = Opcode(0xc9);
        assert_eq!(cpu.tick(), Some(BusOp::Read(0x1234)));
        cpu.tock(Some(0x78));
        assert_eq!(cpu.tick(), Some(BusOp::Read(0x1235)));
        cpu.tock(Some(0x56));

        // M3 doesn't do any bus operation (according to LIJI32 and gekkio)
        assert_eq!(cpu.tick(), None);
        cpu.tock(None);

        assert_eq!(cpu.tick(), Some(BusOp::Read(0x5678)));
        cpu.tock(Some(0x00));
        assert_eq!(cpu.regs.sp, 0x1236)
    }
}
