fn main() {
    println!("Hello, world!");
}

pub struct Cpu {
    a: u8,
    b: u8,
    c: u8,
    d: u8,
    e: u8,
    h: u8,
    l: u8,
    pc: u16,
    sp: u16,
    instr: u8,
    m_cycle: MCycle,
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

impl Default for Cpu {
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
            instr: 0x00,
            m_cycle: MCycle::M1,
        }
    }
}

impl Cpu {
    fn hl(&self) -> u16 {
        (u16::from(self.h) << 8) + u16::from(self.l)
    }

    pub fn tick(&mut self) -> Option<BusOp> {
        self.exec_instr(CpuInput {
            phase: Phase::Tick,
            data: None,
        })
    }

    pub fn tock(&mut self, data: Option<u8>) {
        self.exec_instr(CpuInput {
            phase: Phase::Tock,
            data,
        });
    }

    #[inline(always)]
    fn exec_instr(&mut self, input: CpuInput) -> Option<BusOp> {
        match self.instr {
            0b00_000_000 => self.nop(input),
            0b01_000_000 => self.ld_r_r::<B, B>(input),
            0b01_000_001 => self.ld_r_r::<B, C>(input),
            0b01_000_010 => self.ld_r_r::<B, D>(input),
            0b01_000_011 => self.ld_r_r::<B, E>(input),
            0b01_000_100 => self.ld_r_r::<B, H>(input),
            0b01_000_101 => self.ld_r_r::<B, L>(input),
            0b01_000_110 => self.ld_r_deref_hl::<B>(input),
            0b01_000_111 => self.ld_r_r::<B, A>(input),
            0b01_001_000 => self.ld_r_r::<C, B>(input),
            0b01_001_001 => self.ld_r_r::<C, C>(input),
            0b01_001_010 => self.ld_r_r::<C, D>(input),
            0b01_001_011 => self.ld_r_r::<C, E>(input),
            0b01_001_100 => self.ld_r_r::<C, H>(input),
            0b01_001_101 => self.ld_r_r::<C, L>(input),
            0b01_001_110 => self.ld_r_deref_hl::<C>(input),
            0b01_001_111 => self.ld_r_r::<C, A>(input),
            0b01_010_000 => self.ld_r_r::<D, B>(input),
            0b01_010_001 => self.ld_r_r::<D, C>(input),
            0b01_010_010 => self.ld_r_r::<D, D>(input),
            0b01_010_011 => self.ld_r_r::<D, E>(input),
            0b01_010_100 => self.ld_r_r::<D, H>(input),
            0b01_010_101 => self.ld_r_r::<D, L>(input),
            0b01_010_110 => self.ld_r_deref_hl::<D>(input),
            0b01_010_111 => self.ld_r_r::<D, A>(input),
            0b01_011_000 => self.ld_r_r::<E, B>(input),
            0b01_011_001 => self.ld_r_r::<E, C>(input),
            0b01_011_010 => self.ld_r_r::<E, D>(input),
            0b01_011_011 => self.ld_r_r::<E, E>(input),
            0b01_011_100 => self.ld_r_r::<E, H>(input),
            0b01_011_101 => self.ld_r_r::<E, L>(input),
            0b01_011_110 => self.ld_r_deref_hl::<E>(input),
            0b01_011_111 => self.ld_r_r::<E, A>(input),
            0b01_100_000 => self.ld_r_r::<H, B>(input),
            0b01_100_001 => self.ld_r_r::<H, C>(input),
            0b01_100_010 => self.ld_r_r::<H, D>(input),
            0b01_100_011 => self.ld_r_r::<H, E>(input),
            0b01_100_100 => self.ld_r_r::<H, H>(input),
            0b01_100_101 => self.ld_r_r::<H, L>(input),
            0b01_100_110 => self.ld_r_deref_hl::<H>(input),
            0b01_100_111 => self.ld_r_r::<H, A>(input),
            0b01_101_000 => self.ld_r_r::<L, B>(input),
            0b01_101_001 => self.ld_r_r::<L, C>(input),
            0b01_101_010 => self.ld_r_r::<L, D>(input),
            0b01_101_011 => self.ld_r_r::<L, E>(input),
            0b01_101_100 => self.ld_r_r::<L, H>(input),
            0b01_101_101 => self.ld_r_r::<L, L>(input),
            0b01_101_110 => self.ld_r_deref_hl::<L>(input),
            0b01_101_111 => self.ld_r_r::<L, A>(input),
            0b01_110_000 => self.ld_deref_hl_r::<B>(input),
            0b01_110_001 => self.ld_deref_hl_r::<C>(input),
            0b01_110_010 => self.ld_deref_hl_r::<D>(input),
            0b01_110_011 => self.ld_deref_hl_r::<E>(input),
            0b01_110_100 => self.ld_deref_hl_r::<H>(input),
            0b01_110_101 => self.ld_deref_hl_r::<L>(input),
            0b01_110_110 => self.halt(input),
            0b01_110_111 => self.ld_deref_hl_r::<A>(input),
            0b01_111_000 => self.ld_r_r::<A, B>(input),
            0b01_111_001 => self.ld_r_r::<A, C>(input),
            0b01_111_010 => self.ld_r_r::<A, D>(input),
            0b01_111_011 => self.ld_r_r::<A, E>(input),
            0b01_111_100 => self.ld_r_r::<A, H>(input),
            0b01_111_101 => self.ld_r_r::<A, L>(input),
            0b01_111_110 => self.ld_r_deref_hl::<A>(input),
            0b01_111_111 => self.ld_r_r::<A, A>(input),
            0b11_001_001 => self.ret(input),
            _ => unimplemented!(),
        }
    }

    fn nop(&mut self, input: CpuInput) -> Option<BusOp> {
        self.fetch(input)
    }

    fn ld_r_r<D: Reg, S: Reg>(&mut self, input: CpuInput) -> Option<BusOp> {
        if let Phase::Tock = input.phase {
            let value = *self.reg(S::CODE);
            *self.reg(D::CODE) = value;
        }
        self.fetch(input)
    }

    fn ld_r_deref_hl<D: Reg>(&mut self, input: CpuInput) -> Option<BusOp> {
        match self.m_cycle {
            MCycle::M1 => match input.phase {
                Phase::Tick => Some(BusOp::Read(self.hl())),
                Phase::Tock => {
                    *self.reg(D::CODE) = input.data.unwrap();
                    self.advance()
                }
            },
            _ => self.fetch(input),
        }
    }

    fn ld_deref_hl_r<S: Reg>(&mut self, input: CpuInput) -> Option<BusOp> {
        match self.m_cycle {
            MCycle::M1 => match input.phase {
                Phase::Tick => Some(BusOp::Write(self.hl(), *self.reg(S::CODE))),
                Phase::Tock => self.advance(),
            },
            _ => self.fetch(input),
        }
    }

    fn halt(&mut self, _input: CpuInput) -> Option<BusOp> {
        unimplemented!()
    }

    fn ret(&mut self, input: CpuInput) -> Option<BusOp> {
        match self.m_cycle {
            MCycle::M1 => match input.phase {
                Phase::Tick => Some(BusOp::Read(self.sp)),
                Phase::Tock => {
                    self.pc = input.data.unwrap().into();
                    self.sp += 1;
                    self.advance()
                }
            },
            MCycle::M2 => match input.phase {
                Phase::Tick => Some(BusOp::Read(self.sp)),
                Phase::Tock => {
                    self.pc |= u16::from(input.data.unwrap()) << 8;
                    self.sp += 1;
                    self.advance()
                }
            },
            MCycle::M3 => match input.phase {
                Phase::Tick => None,
                Phase::Tock => self.advance(),
            },
            _ => self.fetch(input),
        }
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

    fn advance(&mut self) -> Option<BusOp> {
        self.m_cycle = self.m_cycle.next();
        None
    }

    fn fetch(&mut self, input: CpuInput) -> Option<BusOp> {
        match input.phase {
            Phase::Tick => Some(BusOp::Read(self.pc)),
            Phase::Tock => {
                self.instr = input.data.unwrap();
                self.pc += 1;
                self.m_cycle = MCycle::M1;
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

trait Reg {
    const CODE: R;
}

struct A;
struct B;
struct C;
struct D;
struct E;
struct H;
struct L;

impl Reg for A {
    const CODE: R = R::A;
}

impl Reg for B {
    const CODE: R = R::B;
}

impl Reg for C {
    const CODE: R = R::C;
}

impl Reg for D {
    const CODE: R = R::D;
}

impl Reg for E {
    const CODE: R = R::E;
}

impl Reg for H {
    const CODE: R = R::H;
}

impl Reg for L {
    const CODE: R = R::L;
}

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
        cpu.instr = encode_ld_r_r(dest, src);
        *cpu.reg(src) = expected;
        assert_eq!(cpu.tick(), Some(BusOp::Read(0x0000)));
        cpu.tock(Some(0x00));
        assert_eq!(*cpu.reg(dest), expected)
    }

    fn encode_ld_r_r(dest: R, src: R) -> u8 {
        0b01_000_000 | (dest.code() << 3) | src.code()
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
        cpu.h = 0x12;
        cpu.l = 0x34;
        cpu.instr = encode_ld_r_deref_hl(dest);
        assert_eq!(cpu.tick(), Some(BusOp::Read(0x1234)));
        cpu.tock(Some(expected));
        assert_eq!(cpu.tick(), Some(BusOp::Read(0x0000)));
        cpu.tock(Some(0x00));
        assert_eq!(*cpu.reg(dest), expected)
    }

    fn encode_ld_r_deref_hl(dest: R) -> u8 {
        0b01_000_110 | (dest.code() << 3)
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
        *cpu.reg(src) = expected;
        cpu.h = 0x12;
        cpu.l = 0x34;
        *cpu.reg(src) = expected;
        cpu.instr = encode_ld_deref_hl_r(src);
        assert_eq!(cpu.tick(), Some(BusOp::Write(cpu.hl(), expected)));
        cpu.tock(None);
        assert_eq!(cpu.tick(), Some(BusOp::Read(0x0000)));
        cpu.tock(Some(0x00));
    }

    fn encode_ld_deref_hl_r(src: R) -> u8 {
        0b01_110_000 | src.code()
    }

    #[test]
    fn ret() {
        let mut cpu = Cpu::default();
        cpu.sp = 0x1234;
        cpu.instr = 0xc9;
        assert_eq!(cpu.tick(), Some(BusOp::Read(0x1234)));
        cpu.tock(Some(0x78));
        assert_eq!(cpu.tick(), Some(BusOp::Read(0x1235)));
        cpu.tock(Some(0x56));

        // M3 doesn't do any bus operation (according to LIJI32 and gekkio)
        assert_eq!(cpu.tick(), None);
        cpu.tock(None);

        assert_eq!(cpu.tick(), Some(BusOp::Read(0x5678)));
        cpu.tock(Some(0x00));
        assert_eq!(cpu.sp, 0x1236)
    }
}
