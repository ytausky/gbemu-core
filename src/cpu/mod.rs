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
    m1: bool,
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
            m1: false,
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
            }
            .step(input),
            Mode::Interrupt => InterruptModeCpu {
                regs: &mut self.regs,
                m_cycle: self.m_cycle,
                phase: self.phase,
            }
            .step(input),
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
                m1: false,
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
}

impl<'a> RunModeCpu<'a> {
    fn step(&mut self, input: &Input) -> (Option<ModeTransition>, CpuOutput) {
        match self.phase {
            Tick => {
                let mut output = InstrExecution {
                    regs: self.regs,
                    state: self.state,
                    m_cycle: self.m_cycle,
                }
                .exec_instr();
                if self.state.m1 {
                    assert_eq!(output, None);
                    if input.interrupt_flags != 0x00 {
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
                self.state.bus_data = input.data;
                let transition = if self.state.m1 {
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
        }
    }

    fn nop(&mut self) -> Option<BusOp> {
        match self.m_cycle {
            M2 => self.execute_m1(),
            _ => unreachable!(),
        }
    }

    fn halt(&mut self) -> Option<BusOp> {
        unimplemented!()
    }

    fn ld_r_r(&mut self, dest: R, src: R) -> Option<BusOp> {
        match self.m_cycle {
            M2 => {
                self.regs.write(dest, self.regs.read(src));
                self.execute_m1()
            }
            _ => unreachable!(),
        }
    }

    fn ld_r_n(&mut self, dest: R) -> Option<BusOp> {
        match self.m_cycle {
            M2 => self.read_immediate(),
            M3 => {
                self.regs.write(dest, self.state.bus_data.unwrap());
                self.execute_m1()
            }
            _ => unreachable!(),
        }
    }

    fn ld_r_deref_hl(&mut self, dest: R) -> Option<BusOp> {
        match self.m_cycle {
            M2 => Some(BusOp::Read(self.regs.hl())),
            M3 => {
                self.regs.write(dest, self.state.bus_data.unwrap());
                self.execute_m1()
            }
            _ => unreachable!(),
        }
    }

    fn ld_deref_hl_r(&mut self, src: R) -> Option<BusOp> {
        match self.m_cycle {
            M2 => Some(BusOp::Write(self.regs.hl(), self.regs.read(src))),
            M3 => self.execute_m1(),
            _ => unreachable!(),
        }
    }

    fn ld_deref_hl_n(&mut self) -> Option<BusOp> {
        match self.m_cycle {
            M2 => self.read_immediate(),
            M3 => Some(BusOp::Write(self.regs.hl(), self.state.bus_data.unwrap())),
            M4 => self.execute_m1(),
            _ => unreachable!(),
        }
    }

    fn ld_a_deref_bc(&mut self) -> Option<BusOp> {
        match self.m_cycle {
            M2 => Some(BusOp::Read(self.regs.bc())),
            M3 => {
                self.regs.a = self.state.bus_data.unwrap();
                self.execute_m1()
            }
            _ => unreachable!(),
        }
    }

    fn ld_a_deref_de(&mut self) -> Option<BusOp> {
        match self.m_cycle {
            M2 => Some(BusOp::Read(self.regs.de())),
            M3 => {
                self.regs.a = self.state.bus_data.unwrap();
                self.execute_m1()
            }
            _ => unreachable!(),
        }
    }

    fn ld_a_deref_c(&mut self) -> Option<BusOp> {
        match self.m_cycle {
            M2 => Some(BusOp::Read(u16::from_be_bytes([0xff, self.regs.c]))),
            M3 => {
                self.regs.a = self.state.bus_data.unwrap();
                self.execute_m1()
            }
            _ => unreachable!(),
        }
    }

    fn ld_deref_c_a(&mut self) -> Option<BusOp> {
        match self.m_cycle {
            M2 => Some(BusOp::Write(
                u16::from_be_bytes([0xff, self.regs.c]),
                self.regs.a,
            )),
            M3 => self.execute_m1(),
            _ => unreachable!(),
        }
    }

    fn ld_a_deref_n(&mut self) -> Option<BusOp> {
        match self.m_cycle {
            M2 => self.read_immediate(),
            M3 => Some(BusOp::Read(u16::from_be_bytes([
                0xff,
                self.state.bus_data.unwrap(),
            ]))),
            M4 => {
                self.regs.a = self.state.bus_data.unwrap();
                self.execute_m1()
            }
            _ => unreachable!(),
        }
    }

    fn ld_deref_n_a(&mut self) -> Option<BusOp> {
        match self.m_cycle {
            M2 => self.read_immediate(),
            M3 => Some(BusOp::Write(
                u16::from_be_bytes([0xff, self.state.bus_data.unwrap()]),
                self.regs.a,
            )),
            M4 => self.execute_m1(),
            _ => unreachable!(),
        }
    }

    fn ld_a_deref_nn(&mut self) -> Option<BusOp> {
        match self.m_cycle {
            M2 => self.read_immediate(),
            M3 => {
                self.state.data = self.state.bus_data.unwrap();
                self.read_immediate()
            }
            M4 => Some(BusOp::Read(u16::from_be_bytes([
                self.state.bus_data.unwrap(),
                self.state.data,
            ]))),
            M5 => {
                self.regs.a = self.state.bus_data.unwrap();
                self.execute_m1()
            }
            _ => unreachable!(),
        }
    }

    fn ld_deref_nn_a(&mut self) -> Option<BusOp> {
        match self.m_cycle {
            M2 => self.read_immediate(),
            M3 => {
                self.state.data = self.state.bus_data.unwrap();
                self.read_immediate()
            }
            M4 => Some(BusOp::Write(
                u16::from_be_bytes([self.state.bus_data.unwrap(), self.state.data]),
                self.regs.a,
            )),
            M5 => self.execute_m1(),
            _ => unreachable!(),
        }
    }

    fn ld_a_deref_hli(&mut self) -> Option<BusOp> {
        match self.m_cycle {
            M2 => {
                let hl = self.regs.hl();
                let incremented_hl = hl + 1;
                self.regs.h = high_byte(incremented_hl);
                self.regs.l = low_byte(incremented_hl);
                Some(BusOp::Read(hl))
            }
            M3 => {
                self.regs.a = self.state.bus_data.unwrap();
                self.execute_m1()
            }
            _ => unreachable!(),
        }
    }

    fn ld_a_deref_hld(&mut self) -> Option<BusOp> {
        match self.m_cycle {
            M2 => {
                let hl = self.regs.hl();
                let decremented_hl = hl - 1;
                self.regs.h = high_byte(decremented_hl);
                self.regs.l = low_byte(decremented_hl);
                Some(BusOp::Read(hl))
            }
            M3 => {
                self.regs.a = self.state.bus_data.unwrap();
                self.execute_m1()
            }
            _ => unreachable!(),
        }
    }

    fn ld_deref_bc_a(&mut self) -> Option<BusOp> {
        match self.m_cycle {
            M2 => Some(BusOp::Write(self.regs.bc(), self.regs.a)),
            M3 => self.execute_m1(),
            _ => unreachable!(),
        }
    }

    fn ld_deref_de_a(&mut self) -> Option<BusOp> {
        match self.m_cycle {
            M2 => Some(BusOp::Write(self.regs.de(), self.regs.a)),
            M3 => self.execute_m1(),
            _ => unreachable!(),
        }
    }

    fn ld_deref_hli_a(&mut self) -> Option<BusOp> {
        match self.m_cycle {
            M2 => {
                let hl = self.regs.hl();
                let incremented_hl = hl.wrapping_add(1);
                self.regs.h = high_byte(incremented_hl);
                self.regs.l = low_byte(incremented_hl);
                Some(BusOp::Write(hl, self.regs.a))
            }
            M3 => self.execute_m1(),
            _ => unreachable!(),
        }
    }

    fn ld_deref_hld_a(&mut self) -> Option<BusOp> {
        match self.m_cycle {
            M2 => {
                let hl = self.regs.hl();
                let decremented_hl = hl - 1;
                self.regs.h = high_byte(decremented_hl);
                self.regs.l = low_byte(decremented_hl);
                Some(BusOp::Write(hl, self.regs.a))
            }
            M3 => self.execute_m1(),
            _ => unreachable!(),
        }
    }

    fn ld_dd_nn(&mut self, dd: Dd) -> Option<BusOp> {
        match self.m_cycle {
            M2 => self.read_immediate(),
            M3 => {
                self.regs.write(dd.low(), self.state.bus_data.unwrap());
                self.read_immediate()
            }
            M4 => {
                self.regs.write(dd.high(), self.state.bus_data.unwrap());
                self.execute_m1()
            }
            _ => unreachable!(),
        }
    }

    fn ld_sp_hl(&mut self) -> Option<BusOp> {
        match self.m_cycle {
            M2 => {
                self.regs.sp = self.regs.hl();
                None
            }
            M3 => self.execute_m1(),
            _ => unreachable!(),
        }
    }

    fn push_qq(&mut self, qq: Qq) -> Option<BusOp> {
        match self.m_cycle {
            M2 => None,
            M3 => self.push_byte(self.regs.read(qq.high())),
            M4 => self.push_byte(self.regs.read(qq.low())),
            M5 => self.execute_m1(),
            _ => unreachable!(),
        }
    }

    fn pop_qq(&mut self, qq: Qq) -> Option<BusOp> {
        match self.m_cycle {
            M2 => self.pop_byte(),
            M3 => {
                self.regs.write(qq.low(), self.state.bus_data.unwrap());
                self.pop_byte()
            }
            M4 => {
                self.regs.write(qq.high(), self.state.bus_data.unwrap());
                self.execute_m1()
            }
            _ => unreachable!(),
        }
    }

    fn ldhl_sp_e(&mut self) -> Option<BusOp> {
        match self.m_cycle {
            M2 => self.read_immediate(),
            M3 => {
                let e = self.state.bus_data.unwrap();
                let (l, flags) = alu::add(low_byte(self.regs.sp), e, false);
                let (h, _) = alu::add(high_byte(self.regs.sp), sign_extension(e), flags.cy);
                self.regs.h = h;
                self.regs.l = l;
                self.regs.f = flags;
                self.regs.f.z = false;
                None
            }
            M4 => self.execute_m1(),
            _ => unreachable!(),
        }
    }

    fn ld_deref_nn_sp(&mut self) -> Option<BusOp> {
        match self.m_cycle {
            M2 => self.read_immediate(),
            M3 => {
                self.state.addr = self.state.bus_data.unwrap() as u16;
                self.read_immediate()
            }
            M4 => {
                self.state.addr |= (self.state.bus_data.unwrap() as u16) << 8;
                Some(BusOp::Write(self.state.addr, low_byte(self.regs.sp)))
            }
            M5 => Some(BusOp::Write(self.state.addr + 1, high_byte(self.regs.sp))),
            M6 => self.execute_m1(),
            _ => unreachable!(),
        }
    }

    fn alu_op_r(&mut self, op: AluOp, r: R) -> Option<BusOp> {
        match self.m_cycle {
            M2 => {
                let (result, flags) = self.alu_op(op, self.regs.a, self.regs.read(r));
                self.regs.a = result;
                self.regs.f = flags;
                self.execute_m1()
            }
            _ => unreachable!(),
        }
    }

    fn alu_op_n(&mut self, op: AluOp) -> Option<BusOp> {
        match self.m_cycle {
            M2 => self.read_immediate(),
            M3 => {
                let (result, flags) = self.alu_op(op, self.regs.a, self.state.bus_data.unwrap());
                self.regs.a = result;
                self.regs.f = flags;
                self.execute_m1()
            }
            _ => unreachable!(),
        }
    }

    fn alu_op_deref_hl(&mut self, op: AluOp) -> Option<BusOp> {
        match self.m_cycle {
            M2 => Some(BusOp::Read(self.regs.hl())),
            M3 => {
                let (result, flags) = self.alu_op(op, self.regs.a, self.state.bus_data.unwrap());
                self.regs.a = result;
                self.regs.f = flags;
                self.execute_m1()
            }
            _ => unreachable!(),
        }
    }

    fn inc_r(&mut self, r: R) -> Option<BusOp> {
        match self.m_cycle {
            M2 => {
                let (result, flags) = alu::add(self.regs.read(r), 1, false);
                self.regs.write(r, result);
                self.regs.f.z = flags.z;
                self.regs.f.n = flags.n;
                self.regs.f.h = flags.h;
                self.execute_m1()
            }
            _ => unreachable!(),
        }
    }

    fn inc_deref_hl(&mut self) -> Option<BusOp> {
        match self.m_cycle {
            M2 => Some(BusOp::Read(self.regs.hl())),
            M3 => {
                let (result, flags) = alu::add(self.state.bus_data.unwrap(), 1, false);
                self.regs.f.z = flags.z;
                self.regs.f.n = flags.n;
                self.regs.f.h = flags.h;
                Some(BusOp::Write(self.regs.hl(), result))
            }
            M4 => self.execute_m1(),
            _ => unreachable!(),
        }
    }

    fn jp_nn(&mut self) -> Option<BusOp> {
        match self.m_cycle {
            M2 => self.read_immediate(),
            M3 => {
                self.state.data = self.state.bus_data.unwrap();
                self.read_immediate()
            }
            M4 => {
                self.regs.pc = u16::from_be_bytes([self.state.bus_data.unwrap(), self.state.data]);
                None
            }
            M5 => self.execute_m1(),
            _ => unreachable!(),
        }
    }

    fn jp_cc_nn(&mut self, cc: Cc) -> Option<BusOp> {
        match self.m_cycle {
            M2 => self.read_immediate(),
            M3 => {
                self.state.data = self.state.bus_data.unwrap();
                self.read_immediate()
            }
            M4 => {
                if self.evaluate_condition(cc) {
                    self.regs.pc =
                        u16::from_be_bytes([self.state.bus_data.unwrap(), self.state.data]);
                    None
                } else {
                    self.execute_m1()
                }
            }
            M5 => self.execute_m1(),
            _ => unreachable!(),
        }
    }

    fn jr_e(&mut self) -> Option<BusOp> {
        match self.m_cycle {
            M2 => self.read_immediate(),
            M3 => {
                let e = self.state.bus_data.unwrap() as i8;
                self.regs.pc = self.regs.pc.wrapping_add(e as i16 as u16);
                None
            }
            M4 => self.execute_m1(),
            _ => unreachable!(),
        }
    }

    fn jp_deref_hl(&mut self) -> Option<BusOp> {
        match self.m_cycle {
            M2 => {
                self.regs.pc = self.regs.hl();
                self.execute_m1()
            }
            _ => unreachable!(),
        }
    }

    fn ret(&mut self) -> Option<BusOp> {
        match self.m_cycle {
            M2 => self.pop_byte(),
            M3 => {
                self.state.data = self.state.bus_data.unwrap();
                self.pop_byte()
            }
            M4 => {
                self.regs.pc = u16::from_be_bytes([self.state.bus_data.unwrap(), self.state.data]);
                None
            }
            M5 => self.execute_m1(),
            _ => unreachable!(),
        }
    }

    fn execute_m1(&mut self) -> Option<BusOp> {
        self.state.m1 = true;
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
}

impl<'a> InterruptModeCpu<'a> {
    fn step(&mut self, input: &Input) -> (Option<ModeTransition>, CpuOutput) {
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
                    let n = input.interrupt_flags.trailing_zeros();
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
