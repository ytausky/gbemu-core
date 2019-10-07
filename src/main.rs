fn main() {
    println!("Hello, world!");
}

struct Cpu {
    b: u8,
    c: u8,
    h: u8,
    l: u8,
    pc: u16,
    instr: u8,
    m_cycle: MCycle,
}

enum MCycle {
    M1,
    M2,
}

impl Cpu {
    fn new() -> Self {
        Self {
            b: 0x00,
            c: 0x00,
            h: 0x00,
            l: 0x00,
            pc: 0x0000,
            instr: 0x00,
            m_cycle: MCycle::M1,
        }
    }

    fn hl(&self) -> u16 {
        (u16::from(self.h) << 8) + u16::from(self.l)
    }

    fn tick(&mut self) -> BusOp {
        if let MCycle::M1 = self.m_cycle {
            BusOp::Read(self.pc)
        } else {
            match self.instr {
                0b01_000_110 => match self.m_cycle {
                    MCycle::M1 => panic!(),
                    MCycle::M2 => BusOp::Read(self.hl()),
                },
                0b01_110_000 => match self.m_cycle {
                    MCycle::M1 => panic!(),
                    MCycle::M2 => BusOp::Write(self.hl(), self.b),
                },
                _ => unimplemented!(),
            }
        }
    }

    fn tock(&mut self, data: Option<u8>) {
        if let MCycle::M1 = self.m_cycle {
            self.pc += 1;
            self.instr = data.unwrap();
        }
        match self.instr {
            0b00_000_000 => (),
            0b01_000_001 => match self.m_cycle {
                MCycle::M1 => {
                    self.b = self.c;
                    self.m_cycle = MCycle::M1
                }
                _ => panic!(),
            },
            0b01_000_110 => match self.m_cycle {
                MCycle::M1 => self.m_cycle = MCycle::M2,
                MCycle::M2 => {
                    self.b = data.unwrap();
                    self.m_cycle = MCycle::M1
                }
            },
            0b01_110_000 => match self.m_cycle {
                MCycle::M1 => self.m_cycle = MCycle::M2,
                MCycle::M2 => self.m_cycle = MCycle::M1,
            },
            _ => unimplemented!(),
        }
    }
}

#[derive(Debug, PartialEq)]
enum BusOp {
    Read(u16),
    Write(u16, u8),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ld_b_c() {
        let mut cpu = Cpu::new();
        cpu.c = 0x42;
        assert_eq!(cpu.tick(), BusOp::Read(0x0000));
        cpu.tock(Some(0x41));
        assert_eq!(cpu.b, 0x42)
    }

    #[test]
    fn ld_b_deref_hl() {
        let mut cpu = Cpu::new();
        cpu.h = 0x12;
        cpu.l = 0x34;
        assert_eq!(cpu.tick(), BusOp::Read(0x0000));
        cpu.tock(Some(0x46));
        assert_eq!(cpu.tick(), BusOp::Read(0x1234));
        cpu.tock(Some(0x42));
        assert_eq!(cpu.b, 0x42)
    }

    #[test]
    fn ld_deref_hl_b() {
        let mut cpu = Cpu::new();
        cpu.b = 0x42;
        cpu.h = 0x12;
        cpu.l = 0x34;
        assert_eq!(cpu.tick(), BusOp::Read(0x0000));
        cpu.tock(Some(0x70));
        assert_eq!(cpu.tick(), BusOp::Write(0x1234, 0x42));
    }
}
