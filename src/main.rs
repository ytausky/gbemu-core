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

impl MCycle {
    #[inline(always)]
    fn next(self) -> Self {
        match self {
            MCycle::M1 => MCycle::M2,
            MCycle::M2 => panic!(),
        }
    }
}

macro_rules! instrs {
    (
        cpu = $cpu:expr, phase = $phase:expr, data = $data:expr;
        $(
            $opcode:pat => { $($details:tt)+ }
        )*
    ) => {
        match $cpu.instr {
            $(
                $opcode => instrs! {
                    @opcode
                    $cpu, $phase, $data;
                    ;
                    $($details)*
                },
            )*
            _ => unimplemented!(),
        }
    };
    (
        @opcode
        $cpu:expr, $phase:expr, $data:expr;
        $(
            $prefix_m_cycle:ident { $($prefix_details:tt)* }
        )*
        ;
        $current_m_cycle:ident { $($current_details:tt)* }
        $(
            $tail_m_cycle:ident { $($tail_details:tt)* }
        )+
    ) => {
        instrs! {
            @opcode
            $cpu, $phase, $data;
            $(
                $prefix_m_cycle { $($prefix_details)* }
            )*
            $current_m_cycle { $($current_details)* }
            ;
            $(
                $tail_m_cycle { $($tail_details)* }
            )*
        }
    };
    (
        @opcode
        $cpu:expr, $phase:expr, $data:expr;
        $(
            $prefix_m_cycle:ident { $($prefix_details:tt)* }
        )*
        ;
        $last_m_cycle:ident { $($last_details:tt)* }
    ) => {
        match $cpu.m_cycle {
            $(
                MCycle::$prefix_m_cycle => {
                    instrs!(
                        @m_cycle
                        $cpu, $phase, $data;
                        $($prefix_details)*;
                        {};
                        { $cpu.m_cycle = MCycle::$prefix_m_cycle.next(); None }
                    )
                }
            )*
            MCycle::$last_m_cycle => {
                instrs!(
                    @m_cycle
                    $cpu, $phase, $data;
                    $($last_details)*;
                    { ; Some(BusOp::Read($cpu.pc)) };
                    {
                        $cpu.instr = $data.unwrap();
                        $cpu.pc += 1;
                        $cpu.m_cycle = MCycle::M1;
                        None
                    }
                )
            }
            #[allow(unreachable_patterns)]
            _ => panic!(),
        }
    };
    (
        @m_cycle
        $cpu:expr, $phase:expr, $data:expr;
        Tick $tick:block Tock $tock:block;
        { $($tick_epilogue:tt)* };
        $tock_epilogue:block
    ) => {
        match $phase {
            Phase::Tick => { { Some($tick) } $($tick_epilogue)* }
            Phase::Tock => { $tock $tock_epilogue }
        }
    };
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
        instrs!(
            cpu = self, phase = phase, data = data;

            // NOP
            0b00_000_000 => {
                M1 { Tick {} Tock {} }
            }

            // LD B,C
            0b01_000_001 => {
                M1 { Tick {} Tock { self.b = self.c } }
            }

            // LD B,(HL)
            0b01_000_110 => {
                M1 { Tick { BusOp::Read(self.hl()) } Tock { self.b = data.unwrap() }}
                M2 { Tick {} Tock {} }
            }

            // LD (HL),B
            0b01_110_000 => {
                M1 { Tick { BusOp::Write(self.hl(), self.b) } Tock {} }
                M2 { Tick {} Tock {} }
            }
        )
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
