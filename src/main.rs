fn main() {
    println!("Hello, world!");
}

pub struct Cpu {
    b: u8,
    c: u8,
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
            b: 0x00,
            c: 0x00,
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
            0b01_001_000 => self.ld_r_r::<C, B>(input),
            0b01_001_001 => self.ld_r_r::<C, C>(input),
            0b01_000_110 => self.ld_r_deref_hl::<B>(input),
            0b01_110_000 => self.ld_deref_hl_r::<B>(input),
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

    fn ret(&mut self, input: CpuInput) -> Option<BusOp> {
        match self.m_cycle {
            MCycle::M1 => match input.phase {
                Phase::Tick => Some(BusOp::Read(self.sp)),
                Phase::Tock => {
                    self.pc = input.data.unwrap().into();
                    self.sp += 1;
                    self.advance()
                }
            }
            MCycle::M2 => match input.phase {
                Phase::Tick => Some(BusOp::Read(self.sp)),
                Phase::Tock => {
                    self.pc |= u16::from(input.data.unwrap()) << 8;
                    self.sp += 1;
                    self.advance()
                }
            }
            MCycle::M3 => match input.phase {
                Phase::Tick => None,
                Phase::Tock => self.advance(),
            }
            _ => self.fetch(input),
        }
    }

    fn reg(&mut self, r: R) -> &mut u8 {
        match r {
            R::B => &mut self.b,
            R::C => &mut self.c,
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
    B,
    C,
}

trait Reg {
    const CODE: R;
}

struct B;
struct C;

impl Reg for B {
    const CODE: R = R::B;
}

impl Reg for C {
    const CODE: R = R::C;
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

    const RS: &[R] = &[R::B];

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
                R::B => 0b000,
                R::C => 0b001,
            }
        }
    }

    #[test]
    fn ld_b_deref_hl() {
        let mut cpu = Cpu::default();
        cpu.h = 0x12;
        cpu.l = 0x34;
        cpu.instr = 0x46;
        assert_eq!(cpu.tick(), Some(BusOp::Read(0x1234)));
        cpu.tock(Some(0x42));
        assert_eq!(cpu.tick(), Some(BusOp::Read(0x0000)));
        cpu.tock(Some(0x00));
        assert_eq!(cpu.b, 0x42)
    }

    #[test]
    fn ld_deref_hl_b() {
        let mut cpu = Cpu::default();
        cpu.b = 0x42;
        cpu.h = 0x12;
        cpu.l = 0x34;
        cpu.instr = 0x70;
        assert_eq!(cpu.tick(), Some(BusOp::Write(0x1234, 0x42)));
        cpu.tock(None);
        assert_eq!(cpu.tick(), Some(BusOp::Read(0x0000)));
        cpu.tock(Some(0x00));
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
