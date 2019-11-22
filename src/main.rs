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
            0b01_000_001 => self.ld_r_r::<B, C>(input),
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
            let value = *S::access(self);
            *D::access(self) = value;
        }
        self.fetch(input)
    }

    fn ld_r_deref_hl<D: Reg>(&mut self, input: CpuInput) -> Option<BusOp> {
        match self.m_cycle {
            MCycle::M1 => match input.phase {
                Phase::Tick => Some(BusOp::Read(self.hl())),
                Phase::Tock => {
                    *D::access(self) = input.data.unwrap();
                    self.advance()
                }
            },
            _ => self.fetch(input),
        }
    }

    fn ld_deref_hl_r<S: Reg>(&mut self, input: CpuInput) -> Option<BusOp> {
        match self.m_cycle {
            MCycle::M1 => match input.phase {
                Phase::Tick => Some(BusOp::Write(self.hl(), *S::access(self))),
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

trait Reg {
    fn access(cpu: &mut Cpu) -> &mut u8;
}

struct B;
struct C;

impl Reg for B {
    fn access(cpu: &mut Cpu) -> &mut u8 {
        &mut cpu.b
    }
}

impl Reg for C {
    fn access(cpu: &mut Cpu) -> &mut u8 {
        &mut cpu.c
    }
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

    #[test]
    fn ld_b_c() {
        let mut cpu = Cpu::default();
        cpu.c = 0x42;
        cpu.instr = 0x41;
        assert_eq!(cpu.tick(), Some(BusOp::Read(0x0000)));
        cpu.tock(Some(0x00));
        assert_eq!(cpu.b, 0x42)
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
