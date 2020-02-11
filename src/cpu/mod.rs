use self::regs::{Flags, Regs};
use self::{MCycle::*, Phase::*};

#[cfg(test)]
macro_rules! flags {
    ($($flag:ident),*) => {
        $crate::cpu::regs::Flags {
            $($flag: true,)*
            ..$crate::cpu::regs::Flags {
                z: false,
                n: false,
                h: false,
                cy: false,
            }
        }
    };
}

mod alu;
mod regs;

#[cfg(test)]
mod tests;

pub struct Cpu {
    pub regs: Regs,
    mode: Mode,
    m_cycle: MCycle,
    phase: Phase,
}

enum Mode {
    Instruction(InstructionExecutionState),
    Interrupt,
}

struct InstructionExecutionState {
    opcode: Opcode,
    bus_data: Option<u8>,
    fetch: bool,
    interrupt: bool,
    data: u8,
    addr: u16,
}

#[derive(Clone, Copy, PartialEq)]
enum R {
    A,
    B,
    C,
    D,
    E,
    H,
    L,
}

#[derive(Clone, Copy)]
enum Dd {
    Bc,
    De,
    Hl,
    Sp,
}

#[derive(Clone, Copy)]
enum Qq {
    Bc,
    De,
    Hl,
    Af,
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

impl From<u8> for Dd {
    fn from(encoding: u8) -> Self {
        match encoding {
            0b00 => Dd::Bc,
            0b01 => Dd::De,
            0b10 => Dd::Hl,
            0b11 => Dd::Sp,
            _ => panic!(),
        }
    }
}

impl From<u8> for Qq {
    fn from(encoding: u8) -> Self {
        match encoding {
            0b00 => Qq::Bc,
            0b01 => Qq::De,
            0b10 => Qq::Hl,
            0b11 => Qq::Af,
            _ => panic!(),
        }
    }
}

#[derive(Clone, Copy)]
enum Cc {
    Nz,
    Z,
    Nc,
    C,
}

impl From<u8> for Cc {
    fn from(encoding: u8) -> Self {
        match encoding {
            0b00 => Cc::Nz,
            0b01 => Cc::Z,
            0b10 => Cc::Nc,
            0b11 => Cc::C,
            _ => panic!(),
        }
    }
}

#[derive(Clone, Copy)]
enum AluOp {
    Add,
    Adc,
    Sub,
    Sbc,
    And,
    Xor,
    Or,
    Cp,
}

impl From<u8> for AluOp {
    fn from(encoding: u8) -> Self {
        match encoding {
            0b000 => Self::Add,
            0b001 => Self::Adc,
            0b010 => Self::Sub,
            0b011 => Self::Sbc,
            0b100 => Self::And,
            0b101 => Self::Xor,
            0b110 => Self::Or,
            0b111 => Self::Cp,
            _ => panic!(),
        }
    }
}

#[derive(Clone, Copy)]
struct Opcode(u8);

impl Opcode {
    fn split(self) -> (u8, u8, u8) {
        (self.0 >> 6, (self.0 >> 3) & 0b111, self.0 & 0b111)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum MCycle {
    M2,
    M3,
    M4,
    M5,
    M6,
    M7,
}

impl MCycle {
    fn next(self) -> Self {
        match self {
            M2 => M3,
            M3 => M4,
            M4 => M5,
            M5 => M6,
            M6 => M7,
            M7 => unreachable!(),
        }
    }
}

impl Default for Cpu {
    fn default() -> Self {
        Self {
            regs: Default::default(),
            mode: Mode::Instruction(Default::default()),
            m_cycle: M2,
            phase: Tick,
        }
    }
}

impl Default for InstructionExecutionState {
    fn default() -> Self {
        InstructionExecutionState {
            opcode: Opcode(0x00),
            bus_data: None,
            fetch: false,
            interrupt: false,
            data: 0xff,
            addr: 0xffff,
        }
    }
}

impl Cpu {
    pub fn step(&mut self, input: &Input) -> CpuOutput {
        let (mode_transition, output) = match &mut self.mode {
            Mode::Instruction(state) => RunModeCpu {
                regs: &mut self.regs,
                state,
                m_cycle: self.m_cycle,
                phase: self.phase,
                input,
            }
            .step(),
            Mode::Interrupt => InterruptModeCpu {
                regs: &mut self.regs,
                m_cycle: self.m_cycle,
                phase: self.phase,
                input,
            }
            .step(),
        };
        self.phase = match self.phase {
            Tick => Tock,
            Tock => {
                self.m_cycle = self.m_cycle.next();
                Tick
            }
        };
        if let Some(transition) = mode_transition {
            self.mode = transition.into();
            self.m_cycle = M2;
        }
        output
    }
}

#[derive(Clone, Copy)]
enum ModeTransition {
    Instruction(Opcode),
    Interrupt,
}

impl From<ModeTransition> for Mode {
    fn from(transition: ModeTransition) -> Self {
        match transition {
            ModeTransition::Instruction(opcode) => Mode::Instruction(InstructionExecutionState {
                opcode,
                bus_data: None,
                fetch: false,
                interrupt: false,
                data: 0xff,
                addr: 0xffff,
            }),
            ModeTransition::Interrupt => Mode::Interrupt,
        }
    }
}

struct RunModeCpu<'a> {
    regs: &'a mut Regs,
    state: &'a mut InstructionExecutionState,
    m_cycle: MCycle,
    phase: Phase,
    input: &'a Input,
}

impl<'a> RunModeCpu<'a> {
    fn step(&mut self) -> (Option<ModeTransition>, CpuOutput) {
        match self.phase {
            Tick => {
                let mut output = InstrExecution {
                    regs: self.regs,
                    state: self.state,
                    m_cycle: self.m_cycle,
                    sweep_m_cycle: M2,
                    output: None,
                }
                .exec_instr();
                if self.state.fetch {
                    assert_eq!(output, None);
                    if self.input.interrupt_flags != 0x00 {
                        self.state.interrupt = true;
                    } else {
                        let pc = self.regs.pc;
                        self.regs.pc += 1;
                        output = Some(BusOp::Read(pc))
                    }
                }
                (None, output)
            }
            Tock => {
                self.state.bus_data = self.input.data;
                let transition = if self.state.fetch {
                    Some(if self.state.interrupt {
                        ModeTransition::Interrupt
                    } else {
                        ModeTransition::Instruction(Opcode(self.state.bus_data.unwrap()))
                    })
                } else {
                    None
                };
                (transition, None)
            }
        }
    }
}

struct InstrExecution<'a> {
    regs: &'a mut Regs,
    state: &'a mut InstructionExecutionState,
    m_cycle: MCycle,
    sweep_m_cycle: MCycle,
    output: Option<CpuOutput>,
}

impl<'a> InstrExecution<'a> {
    fn exec_instr(mut self) -> CpuOutput {
        match self.state.opcode.split() {
            (0b00, 0b000, 0b000) => self.nop(),
            (0b00, dest, 0b001) if dest & 0b001 == 0 => self.ld_dd_nn((dest >> 1).into()),
            (0b00, 0b000, 0b010) => self.ld_deref_bc_a(),
            (0b00, 0b110, 0b100) => self.inc_deref_hl(),
            (0b00, operand, 0b100) => self.inc_r(operand.into()),
            (0b00, 0b110, 0b110) => self.ld_deref_hl_n(),
            (0b00, dest, 0b110) => self.ld_r_n(dest.into()),
            (0b00, 0b001, 0b000) => self.ld_deref_nn_sp(),
            (0b00, 0b001, 0b010) => self.ld_a_deref_bc(),
            (0b00, 0b010, 0b010) => self.ld_deref_de_a(),
            (0b00, 0b011, 0b000) => self.jr_e(),
            (0b00, 0b011, 0b010) => self.ld_a_deref_de(),
            (0b00, 0b100, 0b010) => self.ld_deref_hli_a(),
            (0b00, 0b101, 0b010) => self.ld_a_deref_hli(),
            (0b00, 0b110, 0b010) => self.ld_deref_hld_a(),
            (0b00, 0b111, 0b010) => self.ld_a_deref_hld(),
            (0b01, 0b110, 0b110) => self.halt(),
            (0b01, dest, 0b110) => self.ld_r_deref_hl(dest.into()),
            (0b01, 0b110, src) => self.ld_deref_hl_r(src.into()),
            (0b01, dest, src) => self.ld_r_r(dest.into(), src.into()),
            (0b10, op, 0b110) => self.alu_op_deref_hl(op.into()),
            (0b10, op, src) => self.alu_op_r(op.into(), src.into()),
            (0b11, dest, 0b001) if dest & 0b001 == 0 => self.pop_qq((dest >> 1).into()),
            (0b11, 0b000, 0b011) => self.jp_nn(),
            (0b11, cc, 0b010) if cc <= 0b011 => self.jp_cc_nn(cc.into()),
            (0b11, src, 0b101) if src & 0b001 == 0 => self.push_qq((src >> 1).into()),
            (0b11, op, 0b110) => self.alu_op_n(op.into()),
            (0b11, 0b001, 0b001) => self.ret(),
            (0b11, 0b100, 0b000) => self.ld_deref_n_a(),
            (0b11, 0b100, 0b010) => self.ld_deref_c_a(),
            (0b11, 0b101, 0b001) => self.jp_deref_hl(),
            (0b11, 0b101, 0b010) => self.ld_deref_nn_a(),
            (0b11, 0b110, 0b000) => self.ld_a_deref_n(),
            (0b11, 0b110, 0b010) => self.ld_a_deref_c(),
            (0b11, 0b111, 0b000) => self.ldhl_sp_e(),
            (0b11, 0b111, 0b001) => self.ld_sp_hl(),
            (0b11, 0b111, 0b010) => self.ld_a_deref_nn(),
            _ => unimplemented!(),
        };
        self.output.unwrap()
    }

    fn nop(&mut self) -> &mut Self {
        self.cycle(|cpu| cpu.fetch())
    }

    fn halt(&mut self) -> &mut Self {
        unimplemented!()
    }

    fn ld_r_r(&mut self, dest: R, src: R) -> &mut Self {
        self.cycle(|cpu| {
            cpu.regs.write(dest, cpu.regs.read(src));
            cpu.fetch()
        })
    }

    fn ld_r_n(&mut self, dest: R) -> &mut Self {
        self.cycle(|cpu| cpu.read_immediate()).cycle(|cpu| {
            cpu.regs.write(dest, cpu.state.bus_data.unwrap());
            cpu.fetch()
        })
    }

    fn ld_r_deref_hl(&mut self, dest: R) -> &mut Self {
        self.cycle(|cpu| Some(BusOp::Read(cpu.regs.hl())))
            .cycle(|cpu| {
                cpu.regs.write(dest, cpu.state.bus_data.unwrap());
                cpu.fetch()
            })
    }

    fn ld_deref_hl_r(&mut self, src: R) -> &mut Self {
        self.cycle(|cpu| Some(BusOp::Write(cpu.regs.hl(), cpu.regs.read(src))))
            .cycle(|cpu| cpu.fetch())
    }

    fn ld_deref_hl_n(&mut self) -> &mut Self {
        self.cycle(|cpu| cpu.read_immediate())
            .cycle(|cpu| Some(BusOp::Write(cpu.regs.hl(), cpu.state.bus_data.unwrap())))
            .cycle(|cpu| cpu.fetch())
    }

    fn ld_a_deref_bc(&mut self) -> &mut Self {
        self.cycle(|cpu| Some(BusOp::Read(cpu.regs.bc())))
            .cycle(|cpu| {
                cpu.regs.a = cpu.state.bus_data.unwrap();
                cpu.fetch()
            })
    }

    fn ld_a_deref_de(&mut self) -> &mut Self {
        self.cycle(|cpu| Some(BusOp::Read(cpu.regs.de())))
            .cycle(|cpu| {
                cpu.regs.a = cpu.state.bus_data.unwrap();
                cpu.fetch()
            })
    }

    fn ld_a_deref_c(&mut self) -> &mut Self {
        self.cycle(|cpu| Some(BusOp::Read(u16::from_be_bytes([0xff, cpu.regs.c]))))
            .cycle(|cpu| {
                cpu.regs.a = cpu.state.bus_data.unwrap();
                cpu.fetch()
            })
    }

    fn ld_deref_c_a(&mut self) -> &mut Self {
        self.cycle(|cpu| {
            Some(BusOp::Write(
                u16::from_be_bytes([0xff, cpu.regs.c]),
                cpu.regs.a,
            ))
        })
        .cycle(|cpu| cpu.fetch())
    }

    fn ld_a_deref_n(&mut self) -> &mut Self {
        self.cycle(|cpu| cpu.read_immediate())
            .cycle(|cpu| {
                Some(BusOp::Read(u16::from_be_bytes([
                    0xff,
                    cpu.state.bus_data.unwrap(),
                ])))
            })
            .cycle(|cpu| {
                cpu.regs.a = cpu.state.bus_data.unwrap();
                cpu.fetch()
            })
    }

    fn ld_deref_n_a(&mut self) -> &mut Self {
        self.cycle(|cpu| cpu.read_immediate())
            .cycle(|cpu| {
                Some(BusOp::Write(
                    u16::from_be_bytes([0xff, cpu.state.bus_data.unwrap()]),
                    cpu.regs.a,
                ))
            })
            .cycle(|cpu| cpu.fetch())
    }

    fn ld_a_deref_nn(&mut self) -> &mut Self {
        self.cycle(|cpu| cpu.read_immediate())
            .cycle(|cpu| {
                cpu.state.data = cpu.state.bus_data.unwrap();
                cpu.read_immediate()
            })
            .cycle(|cpu| {
                Some(BusOp::Read(u16::from_be_bytes([
                    cpu.state.bus_data.unwrap(),
                    cpu.state.data,
                ])))
            })
            .cycle(|cpu| {
                cpu.regs.a = cpu.state.bus_data.unwrap();
                cpu.fetch()
            })
    }

    fn ld_deref_nn_a(&mut self) -> &mut Self {
        self.cycle(|cpu| cpu.read_immediate())
            .cycle(|cpu| {
                cpu.state.data = cpu.state.bus_data.unwrap();
                cpu.read_immediate()
            })
            .cycle(|cpu| {
                Some(BusOp::Write(
                    u16::from_be_bytes([cpu.state.bus_data.unwrap(), cpu.state.data]),
                    cpu.regs.a,
                ))
            })
            .cycle(|cpu| cpu.fetch())
    }

    fn ld_a_deref_hli(&mut self) -> &mut Self {
        self.cycle(|cpu| {
            let hl = cpu.regs.hl();
            let incremented_hl = hl + 1;
            cpu.regs.h = high_byte(incremented_hl);
            cpu.regs.l = low_byte(incremented_hl);
            Some(BusOp::Read(hl))
        })
        .cycle(|cpu| {
            cpu.regs.a = cpu.state.bus_data.unwrap();
            cpu.fetch()
        })
    }

    fn ld_a_deref_hld(&mut self) -> &mut Self {
        self.cycle(|cpu| {
            let hl = cpu.regs.hl();
            let decremented_hl = hl - 1;
            cpu.regs.h = high_byte(decremented_hl);
            cpu.regs.l = low_byte(decremented_hl);
            Some(BusOp::Read(hl))
        })
        .cycle(|cpu| {
            cpu.regs.a = cpu.state.bus_data.unwrap();
            cpu.fetch()
        })
    }

    fn ld_deref_bc_a(&mut self) -> &mut Self {
        self.cycle(|cpu| Some(BusOp::Write(cpu.regs.bc(), cpu.regs.a)))
            .cycle(|cpu| cpu.fetch())
    }

    fn ld_deref_de_a(&mut self) -> &mut Self {
        self.cycle(|cpu| Some(BusOp::Write(cpu.regs.de(), cpu.regs.a)))
            .cycle(|cpu| cpu.fetch())
    }

    fn ld_deref_hli_a(&mut self) -> &mut Self {
        self.cycle(|cpu| {
            let hl = cpu.regs.hl();
            let incremented_hl = hl.wrapping_add(1);
            cpu.regs.h = high_byte(incremented_hl);
            cpu.regs.l = low_byte(incremented_hl);
            Some(BusOp::Write(hl, cpu.regs.a))
        })
        .cycle(|cpu| cpu.fetch())
    }

    fn ld_deref_hld_a(&mut self) -> &mut Self {
        self.cycle(|cpu| {
            let hl = cpu.regs.hl();
            let decremented_hl = hl - 1;
            cpu.regs.h = high_byte(decremented_hl);
            cpu.regs.l = low_byte(decremented_hl);
            Some(BusOp::Write(hl, cpu.regs.a))
        })
        .cycle(|cpu| cpu.fetch())
    }

    fn ld_dd_nn(&mut self, dd: Dd) -> &mut Self {
        self.cycle(|cpu| cpu.read_immediate())
            .cycle(|cpu| {
                cpu.regs.write(dd.low(), cpu.state.bus_data.unwrap());
                cpu.read_immediate()
            })
            .cycle(|cpu| {
                cpu.regs.write(dd.high(), cpu.state.bus_data.unwrap());
                cpu.fetch()
            })
    }

    fn ld_sp_hl(&mut self) -> &mut Self {
        self.cycle(|cpu| {
            cpu.regs.sp = cpu.regs.hl();
            None
        })
        .cycle(|cpu| cpu.fetch())
    }

    fn push_qq(&mut self, qq: Qq) -> &mut Self {
        self.cycle(|_| None)
            .cycle(|cpu| cpu.push_byte(cpu.regs.read(qq.high())))
            .cycle(|cpu| cpu.push_byte(cpu.regs.read(qq.low())))
            .cycle(|cpu| cpu.fetch())
    }

    fn pop_qq(&mut self, qq: Qq) -> &mut Self {
        self.cycle(|cpu| cpu.pop_byte())
            .cycle(|cpu| {
                cpu.regs.write(qq.low(), cpu.state.bus_data.unwrap());
                cpu.pop_byte()
            })
            .cycle(|cpu| {
                cpu.regs.write(qq.high(), cpu.state.bus_data.unwrap());
                cpu.fetch()
            })
    }

    fn ldhl_sp_e(&mut self) -> &mut Self {
        self.cycle(|cpu| cpu.read_immediate())
            .cycle(|cpu| {
                let e = cpu.state.bus_data.unwrap();
                let (l, flags) = alu::add(low_byte(cpu.regs.sp), e, false);
                let (h, _) = alu::add(high_byte(cpu.regs.sp), sign_extension(e), flags.cy);
                cpu.regs.h = h;
                cpu.regs.l = l;
                cpu.regs.f = flags;
                cpu.regs.f.z = false;
                None
            })
            .cycle(|cpu| cpu.fetch())
    }

    fn ld_deref_nn_sp(&mut self) -> &mut Self {
        self.cycle(|cpu| cpu.read_immediate())
            .cycle(|cpu| {
                cpu.state.addr = cpu.state.bus_data.unwrap() as u16;
                cpu.read_immediate()
            })
            .cycle(|cpu| {
                cpu.state.addr |= (cpu.state.bus_data.unwrap() as u16) << 8;
                Some(BusOp::Write(cpu.state.addr, low_byte(cpu.regs.sp)))
            })
            .cycle(|cpu| Some(BusOp::Write(cpu.state.addr + 1, high_byte(cpu.regs.sp))))
            .cycle(|cpu| cpu.fetch())
    }

    fn alu_op_r(&mut self, op: AluOp, r: R) -> &mut Self {
        self.cycle(|cpu| {
            let (result, flags) = cpu.alu_op(op, cpu.regs.a, cpu.regs.read(r));
            cpu.regs.a = result;
            cpu.regs.f = flags;
            cpu.fetch()
        })
    }

    fn alu_op_n(&mut self, op: AluOp) -> &mut Self {
        self.cycle(|cpu| cpu.read_immediate()).cycle(|cpu| {
            let (result, flags) = cpu.alu_op(op, cpu.regs.a, cpu.state.bus_data.unwrap());
            cpu.regs.a = result;
            cpu.regs.f = flags;
            cpu.fetch()
        })
    }

    fn alu_op_deref_hl(&mut self, op: AluOp) -> &mut Self {
        self.cycle(|cpu| Some(BusOp::Read(cpu.regs.hl())))
            .cycle(|cpu| {
                let (result, flags) = cpu.alu_op(op, cpu.regs.a, cpu.state.bus_data.unwrap());
                cpu.regs.a = result;
                cpu.regs.f = flags;
                cpu.fetch()
            })
    }

    fn inc_r(&mut self, r: R) -> &mut Self {
        self.cycle(|cpu| {
            let (result, flags) = alu::add(cpu.regs.read(r), 1, false);
            cpu.regs.write(r, result);
            cpu.regs.f.z = flags.z;
            cpu.regs.f.n = flags.n;
            cpu.regs.f.h = flags.h;
            cpu.fetch()
        })
    }

    fn inc_deref_hl(&mut self) -> &mut Self {
        self.cycle(|cpu| Some(BusOp::Read(cpu.regs.hl())))
            .cycle(|cpu| {
                let (result, flags) = alu::add(cpu.state.bus_data.unwrap(), 1, false);
                cpu.regs.f.z = flags.z;
                cpu.regs.f.n = flags.n;
                cpu.regs.f.h = flags.h;
                Some(BusOp::Write(cpu.regs.hl(), result))
            })
            .cycle(|cpu| cpu.fetch())
    }

    fn jp_nn(&mut self) -> &mut Self {
        self.cycle(|cpu| cpu.read_immediate())
            .cycle(|cpu| {
                cpu.state.data = cpu.state.bus_data.unwrap();
                cpu.read_immediate()
            })
            .cycle(|cpu| {
                cpu.regs.pc = u16::from_be_bytes([cpu.state.bus_data.unwrap(), cpu.state.data]);
                None
            })
            .cycle(|cpu| cpu.fetch())
    }

    fn jp_cc_nn(&mut self, cc: Cc) -> &mut Self {
        self.cycle(|cpu| cpu.read_immediate())
            .cycle(|cpu| {
                cpu.state.data = cpu.state.bus_data.unwrap();
                cpu.read_immediate()
            })
            .cycle(|cpu| {
                if cpu.evaluate_condition(cc) {
                    cpu.regs.pc = u16::from_be_bytes([cpu.state.bus_data.unwrap(), cpu.state.data]);
                    None
                } else {
                    cpu.fetch()
                }
            })
            .cycle(|cpu| cpu.fetch())
    }

    fn jr_e(&mut self) -> &mut Self {
        self.cycle(|cpu| cpu.read_immediate())
            .cycle(|cpu| {
                let e = cpu.state.bus_data.unwrap() as i8;
                cpu.regs.pc = cpu.regs.pc.wrapping_add(e as i16 as u16);
                None
            })
            .cycle(|cpu| cpu.fetch())
    }

    fn jp_deref_hl(&mut self) -> &mut Self {
        self.cycle(|cpu| {
            cpu.regs.pc = cpu.regs.hl();
            cpu.fetch()
        })
    }

    fn ret(&mut self) -> &mut Self {
        self.cycle(|cpu| cpu.pop_byte())
            .cycle(|cpu| {
                cpu.state.data = cpu.state.bus_data.unwrap();
                cpu.pop_byte()
            })
            .cycle(|cpu| {
                cpu.regs.pc = u16::from_be_bytes([cpu.state.bus_data.unwrap(), cpu.state.data]);
                None
            })
            .cycle(|cpu| cpu.fetch())
    }

    fn cycle<F>(&mut self, f: F) -> &mut Self
    where
        F: FnOnce(&mut Self) -> CpuOutput,
    {
        let output = if self.m_cycle == self.sweep_m_cycle {
            Some(f(self))
        } else {
            None
        };
        self.sweep_m_cycle = self.sweep_m_cycle.next();
        self.output = self.output.take().or(output);
        self
    }

    fn fetch(&mut self) -> CpuOutput {
        self.state.fetch = true;
        None
    }

    fn read_immediate(&mut self) -> CpuOutput {
        let pc = self.regs.pc;
        self.regs.pc += 1;
        Some(BusOp::Read(pc))
    }

    fn push_byte(&mut self, data: u8) -> CpuOutput {
        self.regs.sp -= 1;
        Some(BusOp::Write(self.regs.sp, data))
    }

    fn pop_byte(&mut self) -> CpuOutput {
        let sp = self.regs.sp;
        self.regs.sp += 1;
        Some(BusOp::Read(sp))
    }

    fn alu_op(&self, op: AluOp, lhs: u8, rhs: u8) -> (u8, Flags) {
        match op {
            AluOp::Add => alu::add(lhs, rhs, false),
            AluOp::Adc => alu::add(lhs, rhs, self.regs.f.cy),
            AluOp::Sub => alu::sub(lhs, rhs, false),
            AluOp::Sbc => alu::sub(lhs, rhs, self.regs.f.cy),
            AluOp::And => alu::and(lhs, rhs),
            AluOp::Xor => alu::xor(lhs, rhs),
            AluOp::Or => alu::or(lhs, rhs),
            AluOp::Cp => {
                let (_, flags) = alu::sub(lhs, rhs, false);
                (lhs, flags)
            }
        }
    }

    fn evaluate_condition(&self, cc: Cc) -> bool {
        match cc {
            Cc::Nz => !self.regs.f.z,
            Cc::Z => self.regs.f.z,
            Cc::Nc => !self.regs.f.cy,
            Cc::C => self.regs.f.cy,
        }
    }
}

struct InterruptModeCpu<'a> {
    regs: &'a mut Regs,
    m_cycle: MCycle,
    phase: Phase,
    input: &'a Input,
}

impl<'a> InterruptModeCpu<'a> {
    fn step(&mut self) -> (Option<ModeTransition>, CpuOutput) {
        match self.m_cycle {
            M2 => (None, None),
            M3 => (None, None),
            M4 => match self.phase {
                Tick => {
                    self.regs.sp -= 1;
                    (
                        None,
                        Some(BusOp::Write(self.regs.sp, high_byte(self.regs.pc))),
                    )
                }
                Tock => (None, None),
            },
            M5 => match self.phase {
                Tick => {
                    self.regs.sp -= 1;
                    (
                        None,
                        Some(BusOp::Write(self.regs.sp, low_byte(self.regs.pc))),
                    )
                }
                Tock => {
                    let n = self.input.interrupt_flags.trailing_zeros();
                    self.regs.pc = 0x0040 + 8 * n as u16;
                    (Some(ModeTransition::Instruction(Opcode(0x00))), None)
                }
            },
            _ => unreachable!(),
        }
    }
}

fn low_byte(addr: u16) -> u8 {
    (addr & 0x00ff) as u8
}

fn high_byte(addr: u16) -> u8 {
    (addr >> 8) as u8
}

fn sign_extension(data: u8) -> u8 {
    if data > 0x80 {
        0xff
    } else {
        0x00
    }
}

#[derive(Clone)]
struct AluOutput {
    result: u8,
    flags: Flags,
}

#[derive(Clone)]
pub struct Input {
    data: Option<u8>,
    interrupt_flags: u8,
}

#[derive(Clone, Copy, PartialEq)]
enum Phase {
    Tick,
    Tock,
}

type CpuOutput = Option<BusOp>;

#[derive(Clone, Debug, PartialEq)]
pub enum BusOp {
    Read(u16),
    Write(u16, u8),
}
