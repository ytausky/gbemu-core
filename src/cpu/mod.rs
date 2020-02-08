use self::microinstruction::*;
use self::regs::{Flags, Regs};
use self::{MCycle::*, Phase::*};

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
mod microinstruction;
mod regs;

#[cfg(test)]
mod tests;

pub struct Cpu {
    pub regs: Regs,
    mode: Mode,
    phase: Phase,
}

const ALL_FLAGS: Flags = Flags {
    z: true,
    n: true,
    h: true,
    cy: true,
};

enum Mode {
    Run(ComplexInstrExecState),
}

struct ComplexInstrExecState {
    opcode: Opcode,
    m_cycle: MCycle,
    data: u8,
    addr: u16,
}

#[derive(Clone, Copy)]
enum M {
    R(R),
    DerefHl,
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

impl From<u8> for M {
    fn from(encoding: u8) -> Self {
        match encoding {
            0b000 => M::R(R::B),
            0b001 => M::R(R::C),
            0b010 => M::R(R::D),
            0b011 => M::R(R::E),
            0b100 => M::R(R::H),
            0b101 => M::R(R::L),
            0b110 => M::DerefHl,
            0b111 => M::R(R::A),
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
    M1,
    M2,
    M3,
    M4,
    M5,
    M6,
}

impl MCycle {
    fn next(self) -> Self {
        match self {
            M1 => M2,
            M2 => M3,
            M3 => M4,
            M4 => M5,
            M5 => M6,
            M6 => panic!(),
        }
    }
}

impl Default for Cpu {
    fn default() -> Self {
        Self {
            regs: Default::default(),
            mode: Mode::Run(Default::default()),
            phase: Tick,
        }
    }
}

impl Default for ComplexInstrExecState {
    fn default() -> Self {
        Self {
            opcode: Opcode(0x00),
            m_cycle: M1,
            data: 0xff,
            addr: 0xffff,
        }
    }
}

impl Cpu {
    pub fn step(&mut self, input: &Input) -> CpuOutput {
        let (mode_transition, output) = match &mut self.mode {
            Mode::Run(stage) => RunModeCpu {
                regs: &mut self.regs,
                stage,
                phase: &self.phase,
                input,
            }
            .step(),
        };
        if let Some(transition) = mode_transition {
            self.mode = transition.into();
        }
        self.phase = match self.phase {
            Tick => Tock,
            Tock => Tick,
        };
        output
    }
}

#[derive(Clone, Copy)]
enum ModeTransition {
    Run(Opcode),
}

impl From<ModeTransition> for Mode {
    fn from(transition: ModeTransition) -> Self {
        match transition {
            ModeTransition::Run(opcode) => Mode::Run(ComplexInstrExecState {
                opcode,
                m_cycle: M1,
                data: 0xff,
                addr: 0xffff,
            }),
        }
    }
}

struct RunModeCpu<'a> {
    regs: &'a mut Regs,
    stage: &'a mut ComplexInstrExecState,
    phase: &'a Phase,
    input: &'a Input,
}

impl<'a> RunModeCpu<'a> {
    fn step(&mut self) -> (Option<ModeTransition>, CpuOutput) {
        let (transition, output) = InstrExecution {
            regs: self.regs,
            mode_transition: None,
            state: self.stage,
            phase: self.phase,
            input: self.input,
            sweep_m_cycle: M1,
            output: None,
        }
        .exec_instr();
        if transition.is_none() {
            self.stage.m_cycle = match self.phase {
                Tick => self.stage.m_cycle,
                Tock => self.stage.m_cycle.next(),
            };
        }
        (transition, output)
    }
}

struct InstrExecution<'a> {
    regs: &'a mut Regs,
    state: &'a mut ComplexInstrExecState,
    phase: &'a Phase,
    mode_transition: Option<ModeTransition>,
    input: &'a Input,
    sweep_m_cycle: MCycle,
    output: Option<CpuOutput>,
}

impl<'a> InstrExecution<'a> {
    fn exec_instr(mut self) -> (Option<ModeTransition>, CpuOutput) {
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
        (self.mode_transition, self.output.unwrap())
    }

    fn nop(&mut self) -> &mut Self {
        self.cycle(|cpu| cpu.fetch())
    }

    fn halt(&mut self) -> &mut Self {
        unimplemented!()
    }

    fn ld_r_r(&mut self, dest: R, src: R) -> &mut Self {
        self.cycle(|cpu| {
            cpu.select_data(src)
                .write_data(dest, ByteWritebackSrc::Data)
                .fetch()
        })
    }

    fn ld_r_n(&mut self, dest: R) -> &mut Self {
        self.cycle(|cpu| cpu.read_immediate().write_from_bus_to(dest))
            .cycle(|cpu| cpu.fetch())
    }

    fn ld_r_deref_hl(&mut self, dest: R) -> &mut Self {
        self.cycle(|cpu| cpu.bus_read(AddrSel::Hl).write_from_bus_to(dest))
            .cycle(|cpu| cpu.fetch())
    }

    fn ld_deref_hl_r(&mut self, src: R) -> &mut Self {
        self.cycle(|cpu| cpu.bus_write(AddrSel::Hl, src))
            .cycle(|cpu| cpu.fetch())
    }

    fn ld_deref_hl_n(&mut self) -> &mut Self {
        self.cycle(|cpu| cpu.read_immediate().write_from_bus_to(DataSel::DataBuf))
            .cycle(|cpu| cpu.bus_write(AddrSel::Hl, DataSel::DataBuf))
            .cycle(|cpu| cpu.fetch())
    }

    fn ld_a_deref_bc(&mut self) -> &mut Self {
        self.cycle(|cpu| cpu.bus_read(AddrSel::Bc).write_from_bus_to(R::A))
            .cycle(|cpu| cpu.fetch())
    }

    fn ld_a_deref_de(&mut self) -> &mut Self {
        self.cycle(|cpu| cpu.bus_read(AddrSel::De).write_from_bus_to(R::A))
            .cycle(|cpu| cpu.fetch())
    }

    fn ld_a_deref_c(&mut self) -> &mut Self {
        self.cycle(|cpu| cpu.bus_read(AddrSel::C).write_from_bus_to(R::A))
            .cycle(|cpu| cpu.fetch())
    }

    fn ld_deref_c_a(&mut self) -> &mut Self {
        self.cycle(|cpu| cpu.bus_write(AddrSel::C, R::A))
            .cycle(|cpu| cpu.fetch())
    }

    fn ld_a_deref_n(&mut self) -> &mut Self {
        self.cycle(|cpu| cpu.read_immediate().write_from_bus_to(DataSel::DataBuf))
            .cycle(|cpu| cpu.bus_read(AddrSel::DataBuf).write_from_bus_to(R::A))
            .cycle(|cpu| cpu.fetch())
    }

    fn ld_deref_n_a(&mut self) -> &mut Self {
        self.cycle(|cpu| cpu.read_immediate().write_from_bus_to(DataSel::DataBuf))
            .cycle(|cpu| cpu.bus_write(AddrSel::DataBuf, R::A))
            .cycle(|cpu| cpu.fetch())
    }

    fn ld_a_deref_nn(&mut self) -> &mut Self {
        self.cycle(|cpu| cpu.read_immediate().write_from_bus_to(DataSel::AddrL))
            .cycle(|cpu| cpu.read_immediate().write_from_bus_to(DataSel::AddrH))
            .cycle(|cpu| cpu.bus_read(AddrSel::AddrBuf).write_from_bus_to(R::A))
            .cycle(|cpu| cpu.fetch())
    }

    fn ld_deref_nn_a(&mut self) -> &mut Self {
        self.cycle(|cpu| cpu.read_immediate().write_from_bus_to(DataSel::AddrL))
            .cycle(|cpu| cpu.read_immediate().write_from_bus_to(DataSel::AddrH))
            .cycle(|cpu| cpu.bus_write(AddrSel::AddrBuf, R::A))
            .cycle(|cpu| cpu.fetch())
    }

    fn ld_a_deref_hli(&mut self) -> &mut Self {
        self.cycle(|cpu| cpu.bus_read(Hl).increment(Hl).write_from_bus_to(R::A))
            .cycle(|cpu| cpu.fetch())
    }

    fn ld_a_deref_hld(&mut self) -> &mut Self {
        self.cycle(|cpu| cpu.bus_read(Hl).decrement(Hl).write_from_bus_to(R::A))
            .cycle(|cpu| cpu.fetch())
    }

    fn ld_deref_bc_a(&mut self) -> &mut Self {
        self.cycle(|cpu| cpu.bus_write(AddrSel::Bc, R::A))
            .cycle(|cpu| cpu.fetch())
    }

    fn ld_deref_de_a(&mut self) -> &mut Self {
        self.cycle(|cpu| cpu.bus_write(AddrSel::De, R::A))
            .cycle(|cpu| cpu.fetch())
    }

    fn ld_deref_hli_a(&mut self) -> &mut Self {
        self.cycle(|cpu| cpu.bus_write(Hl, R::A).increment(Hl))
            .cycle(|cpu| cpu.fetch())
    }

    fn ld_deref_hld_a(&mut self) -> &mut Self {
        self.cycle(|cpu| cpu.bus_write(Hl, R::A).decrement(Hl))
            .cycle(|cpu| cpu.fetch())
    }

    fn ld_dd_nn(&mut self, dd: Dd) -> &mut Self {
        self.cycle(|cpu| cpu.read_immediate().write_from_bus_to(dd.low()))
            .cycle(|cpu| cpu.read_immediate().write_from_bus_to(dd.high()))
            .cycle(|cpu| cpu.fetch())
    }

    fn ld_sp_hl(&mut self) -> &mut Self {
        self.cycle(|cpu| {
            cpu.select_data(R::H)
                .write_data(DataSel::SpH, ByteWritebackSrc::Data)
        })
        .cycle(|cpu| {
            cpu.select_data(R::L)
                .write_data(DataSel::SpL, ByteWritebackSrc::Data)
                .fetch()
        })
    }

    fn push_qq(&mut self, qq: Qq) -> &mut Self {
        self.cycle(|cpu| cpu.select_addr(Sp).decrement(Sp))
            .cycle(|cpu| cpu.bus_write(Sp, qq.high()).decrement(Sp))
            .cycle(|cpu| cpu.bus_write(Sp, qq.low()))
            .cycle(|cpu| cpu.fetch())
    }

    fn pop_qq(&mut self, qq: Qq) -> &mut Self {
        self.cycle(|cpu| cpu.bus_read(Sp).write_from_bus_to(qq.low()).increment(Sp))
            .cycle(|cpu| cpu.bus_read(Sp).write_from_bus_to(qq.high()).increment(Sp))
            .cycle(|cpu| cpu.fetch())
    }

    fn ldhl_sp_e(&mut self) -> &mut Self {
        self.cycle(|cpu| cpu.read_immediate().write_from_bus_to(DataSel::DataBuf))
            .cycle(|cpu| {
                cpu.select_data(DataSel::DataBuf)
                    .alu_op(AluOp::Add, AluOperand::SpL, AluOperand::Data)
                    .write_result(R::L)
                    .reset_z()
                    .write_flags(ALL_FLAGS)
            })
            .cycle(|cpu| {
                cpu.select_data(DataSel::DataBuf)
                    .alu_op(AluOp::Adc, AluOperand::SpH, AluOperand::SignExtension)
                    .write_result(R::H)
                    .fetch()
            })
    }

    fn ld_deref_nn_sp(&mut self) -> &mut Self {
        self.cycle(|cpu| cpu.read_immediate().write_from_bus_to(DataSel::AddrL))
            .cycle(|cpu| cpu.read_immediate().write_from_bus_to(DataSel::AddrH))
            .cycle(|cpu| {
                cpu.bus_write(AddrSel::AddrBuf, DataSel::SpL)
                    .increment(WordWritebackDest::AddrBuf)
            })
            .cycle(|cpu| cpu.bus_write(AddrSel::AddrBuf, DataSel::SpH))
            .cycle(|cpu| cpu.fetch())
    }

    fn alu_op_r(&mut self, op: AluOp, r: R) -> &mut Self {
        self.cycle(|cpu| {
            cpu.select_data(r)
                .alu_op(op, AluOperand::A, AluOperand::Data)
                .write_result(R::A)
                .write_flags(ALL_FLAGS)
                .fetch()
        })
    }

    fn alu_op_n(&mut self, op: AluOp) -> &mut Self {
        self.cycle(|cpu| {
            cpu.read_immediate()
                .alu_op(op, AluOperand::A, AluOperand::Bus)
                .write_result(R::A)
                .write_flags(ALL_FLAGS)
        })
        .cycle(|cpu| cpu.fetch())
    }

    fn alu_op_deref_hl(&mut self, op: AluOp) -> &mut Self {
        self.cycle(|cpu| {
            cpu.bus_read(AddrSel::Hl)
                .alu_op(op, AluOperand::A, AluOperand::Bus)
                .write_result(R::A)
                .write_flags(ALL_FLAGS)
        })
        .cycle(|cpu| cpu.fetch())
    }

    fn inc_r(&mut self, r: R) -> &mut Self {
        self.cycle(|cpu| {
            cpu.select_data(DataSel::R(r))
                .alu_op(AluOp::Add, AluOperand::Data, AluOperand::One)
                .write_result(DataSel::R(r))
                .write_flags(flags!(z, n, h))
                .fetch()
        })
    }

    fn inc_deref_hl(&mut self) -> &mut Self {
        self.cycle(|cpu| {
            cpu.bus_read(AddrSel::Hl)
                .alu_op(AluOp::Add, AluOperand::Bus, AluOperand::One)
                .write_result(DataSel::DataBuf)
                .write_flags(flags!(z, n, h))
        })
        .cycle(|cpu| cpu.bus_write(AddrSel::Hl, DataSel::DataBuf))
        .cycle(|cpu| cpu.fetch())
    }

    fn jp_nn(&mut self) -> &mut Self {
        self.cycle(|cpu| cpu.read_immediate().write_from_bus_to(DataSel::AddrL))
            .cycle(|cpu| cpu.read_immediate().write_from_bus_to(DataSel::AddrH))
            .cycle(|cpu| cpu.write_pc())
            .cycle(|cpu| cpu.fetch())
    }

    fn jp_cc_nn(&mut self, cc: Cc) -> &mut Self {
        self.cycle(|cpu| cpu.read_immediate().write_from_bus_to(DataSel::AddrL))
            .cycle(|cpu| cpu.read_immediate().write_from_bus_to(DataSel::AddrH))
            .cycle(|cpu| cpu.fetch_if_not(cc).write_pc())
            .cycle(|cpu| cpu.fetch())
    }

    fn jr_e(&mut self) -> &mut Self {
        self.cycle(|cpu| {
            cpu.read_immediate()
                .alu_op(AluOp::Add, AluOperand::PcL, AluOperand::Bus)
                .write_data(DataSel::PcL, ByteWritebackSrc::Computation)
        })
        .cycle(|cpu| {
            cpu.select_data(DataSel::DataBuf)
                .alu_op(AluOp::Adc, AluOperand::PcH, AluOperand::SignExtension)
                .write_data(DataSel::PcH, ByteWritebackSrc::Computation)
        })
        .cycle(|cpu| cpu.fetch())
    }

    fn jp_deref_hl(&mut self) -> &mut Self {
        self.cycle(|cpu| cpu.select_addr(AddrSel::Hl).write_pc().fetch())
    }

    fn ret(&mut self) -> &mut Self {
        self.cycle(|cpu| cpu.pop_byte().write_from_bus_to(DataSel::AddrL))
            .cycle(|cpu| cpu.pop_byte().write_from_bus_to(DataSel::AddrH))
            .cycle(|cpu| cpu.write_pc())
            .cycle(|cpu| cpu.fetch())
    }

    fn cycle<F>(&mut self, f: F) -> &mut Self
    where
        F: FnOnce(&mut Microinstruction) -> &mut Microinstruction,
    {
        let output = if self.state.m_cycle == self.sweep_m_cycle {
            let mut microinstruction = Microinstruction::default();
            f(&mut microinstruction);
            Some(self.execute_microinstruction(&microinstruction))
        } else {
            None
        };
        self.sweep_m_cycle = self.sweep_m_cycle.next();
        self.output = self.output.take().or(output);
        self
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
