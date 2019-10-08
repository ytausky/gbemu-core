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

#[derive(Clone, Copy)]
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
        self.exec_instr(Phase::Tick, None).unwrap()
    }

    fn tock(&mut self, data: Option<u8>) {
        self.exec_instr(Phase::Tock, data);
    }

    #[inline(always)]
    fn exec_instr(&mut self, phase: Phase, data: Option<u8>) -> Option<BusOp> {
        match (self.instr, self.m_cycle, phase) {
            // NOP
            (0b00_000_000, MCycle::M1, Phase::Tick) => Some(BusOp::Read(self.pc)),
            (0b00_000_000, MCycle::M1, Phase::Tock) => {
                self.pc += 1;
                self.instr = data.unwrap();
                self.m_cycle = MCycle::M1;
                None
            }
            (0b00_000_000, _, _) => panic!(),

            // LD B,C
            (0b01_000_001, MCycle::M1, Phase::Tick) => {
                self.b = self.c;
                Some(BusOp::Read(self.pc))
            }
            (0b01_000_001, MCycle::M1, Phase::Tock) => {
                self.instr = data.unwrap();
                self.pc += 1;
                self.m_cycle = MCycle::M1;
                None
            }
            (0b01_000_001, _, _) => panic!(),

            // LD B,(HL)
            (0b01_000_110, MCycle::M1, Phase::Tick) => Some(BusOp::Read(self.hl())),
            (0b01_000_110, MCycle::M1, Phase::Tock) => {
                self.b = data.unwrap();
                self.m_cycle = MCycle::M2;
                None
            }
            (0b01_000_110, MCycle::M2, Phase::Tick) => Some(BusOp::Read(self.pc)),
            (0b01_000_110, MCycle::M2, Phase::Tock) => {
                self.instr = data.unwrap();
                self.pc += 1;
                self.m_cycle = MCycle::M1;
                None
            }
            (0b01_000_110, _, _) => panic!(),

            // LD (HL),B
            (0b01_110_000, MCycle::M1, Phase::Tick) => Some(BusOp::Write(self.hl(), self.b)),
            (0b01_110_000, MCycle::M1, Phase::Tock) => {
                self.m_cycle = MCycle::M2;
                None
            }
            (0b01_110_000, MCycle::M2, Phase::Tick) => Some(BusOp::Read(self.pc)),
            (0b01_110_000, MCycle::M2, Phase::Tock) => {
                self.instr = data.unwrap();
                self.pc += 1;
                self.m_cycle = MCycle::M1;
                None
            }
            (0b01_110_000, _, _) => panic!(),

            _ => unimplemented!(),
        }
    }
}

enum Phase {
    Tick,
    Tock,
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
        cpu.instr = 0x41;
        assert_eq!(cpu.tick(), BusOp::Read(0x0000));
        cpu.tock(Some(0x00));
        assert_eq!(cpu.b, 0x42)
    }

    #[test]
    fn ld_b_deref_hl() {
        let mut cpu = Cpu::new();
        cpu.h = 0x12;
        cpu.l = 0x34;
        cpu.instr = 0x46;
        assert_eq!(cpu.tick(), BusOp::Read(0x1234));
        cpu.tock(Some(0x42));
        assert_eq!(cpu.tick(), BusOp::Read(0x0000));
        cpu.tock(Some(0x00));
        assert_eq!(cpu.b, 0x42)
    }

    #[test]
    fn ld_deref_hl_b() {
        let mut cpu = Cpu::new();
        cpu.b = 0x42;
        cpu.h = 0x12;
        cpu.l = 0x34;
        cpu.instr = 0x70;
        assert_eq!(cpu.tick(), BusOp::Write(0x1234, 0x42));
        cpu.tock(None);
        assert_eq!(cpu.tick(), BusOp::Read(0x0000));
        cpu.tock(Some(0x00));
    }
}
