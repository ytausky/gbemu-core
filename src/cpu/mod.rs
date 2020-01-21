use self::{MCycle::*, Phase::*};

#[cfg(test)]
macro_rules! flags {
    ($($flag:ident),*) => {
        Flags {
            $($flag: true,)*
            ..Flags {
                z: false,
                n: false,
                h: false,
                cy: false,
            }
        }
    };
}

mod alu;

#[cfg(test)]
mod tests;

pub struct Cpu {
    pub regs: Regs,
    mode: Mode,
    phase: Phase,
}

#[derive(Default)]
pub struct Regs {
    pub a: u8,
    pub f: Flags,
    pub b: u8,
    pub c: u8,
    pub d: u8,
    pub e: u8,
    pub h: u8,
    pub l: u8,
    pub pc: u16,
    pub sp: u16,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Flags {
    pub z: bool,
    pub n: bool,
    pub h: bool,
    pub cy: bool,
}

enum Mode {
    Run(ComplexInstrExecState),
}

struct ComplexInstrExecState {
    opcode: Opcode,
    m_cycle: MCycle,
    data_buffer: u8,
}

#[derive(Clone)]
enum Src {
    Common(CommonOperand),
    Immediate,
}

#[derive(Clone)]
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
enum CommonOperand {
    Reg(R),
    DerefHl,
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
}

impl MCycle {
    fn next(self) -> Self {
        match self {
            M1 => M2,
            M2 => M3,
            M3 => M4,
            M4 => M5,
            M5 => panic!(),
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
            data_buffer: 0xff,
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

enum ModeTransition {
    Run(Opcode),
}

impl From<ModeTransition> for Mode {
    fn from(transition: ModeTransition) -> Self {
        match transition {
            ModeTransition::Run(opcode) => Mode::Run(ComplexInstrExecState {
                opcode,
                m_cycle: M1,
                data_buffer: 0xff,
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
            (0b01, 0b110, 0b110) => self.halt(),
            (0b01, dest, src) => self.ld(dest.into(), Src::Common(src.into())),
            (0b10, op, src) => self.alu_op(op.into(), Src::Common(src.into())),
            (0b11, op, 0b110) => self.alu_op(op.into(), Src::Immediate),
            (0b11, 0b001, 0b001) => self.ret(),
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

    fn ld(&mut self, dest: CommonOperand, src: Src) -> &mut Self {
        self.read_src(src).write_dest(dest).cycle(|cpu| cpu.fetch())
    }

    fn read_src(&mut self, src: Src) -> &mut Self {
        match src {
            Src::Common(CommonOperand::Reg(r)) => self.micro_op(|cpu| cpu.read_reg(r)),
            Src::Common(CommonOperand::DerefHl) => self.cycle(|cpu| cpu.read(cpu.regs.hl())),
            Src::Immediate => self.cycle(|cpu| cpu.read(cpu.regs.pc).increment_pc()),
        }
    }

    fn write_dest(&mut self, dest: CommonOperand) -> &mut Self {
        match dest {
            CommonOperand::Reg(r) => self.micro_op(|cpu| cpu.write_reg(r)),
            CommonOperand::DerefHl => self.cycle(|cpu| cpu.write(cpu.regs.hl(), *cpu.data_buffer)),
        }
    }

    fn alu_op(&mut self, op: AluOp, rhs: Src) -> &mut Self {
        self.read_src(rhs)
            .micro_op(|cpu| cpu.alu_op(op))
            .cycle(|cpu| cpu.fetch())
    }

    fn ret(&mut self) -> &mut Self {
        self.cycle(|cpu| cpu.read(cpu.regs.sp).increment_sp().set_pc_low_from_data())
            .cycle(|cpu| cpu.read(cpu.regs.sp).increment_sp().set_pc_high_from_data())
            .cycle(|cpu| cpu.no_operation())
            .cycle(|cpu| cpu.fetch())
    }

    fn cycle<F>(&mut self, f: F) -> &mut Self
    where
        F: for<'r, 's> FnOnce(&'s mut CpuProxy<'r>) -> &'s mut CpuProxy<'r>,
    {
        let output = if self.state.m_cycle == self.sweep_m_cycle {
            let mut cpu_proxy = self.cpu_proxy();
            f(&mut cpu_proxy);
            Some(cpu_proxy.output.unwrap())
        } else {
            None
        };
        self.sweep_m_cycle = self.sweep_m_cycle.next();
        self.output = self.output.take().or(output);
        self
    }

    fn micro_op<F>(&mut self, f: F) -> &mut Self
    where
        F: for<'r, 's> FnOnce(&'s mut CpuProxy<'r>) -> &'s mut CpuProxy<'r>,
    {
        if self.state.m_cycle == self.sweep_m_cycle {
            let mut cpu_proxy = self.cpu_proxy();
            f(&mut cpu_proxy);
            assert_eq!(cpu_proxy.output, None)
        }
        self
    }

    fn cpu_proxy(&mut self) -> CpuProxy {
        CpuProxy {
            regs: self.regs,
            data_buffer: &mut self.state.data_buffer,
            mode_transition: &mut self.mode_transition,
            phase: *self.phase,
            input: self.input,
            output: None,
        }
    }
}

struct CpuProxy<'a> {
    regs: &'a mut Regs,
    data_buffer: &'a mut u8,
    phase: Phase,
    mode_transition: &'a mut Option<ModeTransition>,
    input: &'a Input,
    output: Option<CpuOutput>,
}

impl<'a> CpuProxy<'a> {
    fn increment_pc(&mut self) -> &mut Self {
        self.on_tock(|cpu| cpu.regs.pc += 1)
    }

    fn increment_sp(&mut self) -> &mut Self {
        self.on_tock(|cpu| cpu.regs.sp += 1)
    }

    fn set_pc_low_from_data(&mut self) -> &mut Self {
        self.on_tock(|cpu| cpu.regs.pc = cpu.regs.pc & 0xff00 | u16::from(cpu.input.data.unwrap()))
    }

    fn set_pc_high_from_data(&mut self) -> &mut Self {
        self.on_tock(|cpu| {
            cpu.regs.pc = cpu.regs.pc & 0x00ff | (u16::from(cpu.input.data.unwrap()) << 8)
        })
    }

    fn decode(&mut self) -> &mut Self {
        self.on_tock(|cpu| {
            *cpu.mode_transition = Some(ModeTransition::Run(Opcode(cpu.input.data.unwrap())))
        })
    }

    fn no_operation(&mut self) -> &mut Self {
        self.output = Some(None);
        self
    }

    fn fetch(&mut self) -> &mut Self {
        self.read(self.regs.pc).increment_pc().decode()
    }

    fn read(&mut self, addr: u16) -> &mut Self {
        self.output = Some(match self.phase {
            Tick => Some(BusOp::Read(addr)),
            Tock => {
                *self.data_buffer = self.input.data.unwrap();
                None
            }
        });
        self
    }

    fn write(&mut self, addr: u16, data: u8) -> &mut Self {
        self.output = Some(match self.phase {
            Tick => Some(BusOp::Write(addr, data)),
            Tock => None,
        });
        self
    }

    fn read_reg(&mut self, r: R) -> &mut Self {
        self.on_tick(|cpu| *cpu.data_buffer = *cpu.regs.reg(r))
    }

    fn write_reg(&mut self, r: R) -> &mut Self {
        self.on_tock(|cpu| *cpu.regs.reg(r) = *cpu.data_buffer)
    }

    fn alu_op(&mut self, op: AluOp) -> &mut Self {
        self.on_tock(|cpu| {
            let (result, flags) = match op {
                AluOp::Add => alu::add(cpu.regs.a, *cpu.data_buffer, false),
                AluOp::Adc => alu::add(cpu.regs.a, *cpu.data_buffer, cpu.regs.f.cy),
                AluOp::Sub => alu::sub(cpu.regs.a, *cpu.data_buffer, false),
                AluOp::Sbc => alu::sub(cpu.regs.a, *cpu.data_buffer, cpu.regs.f.cy),
                AluOp::And => alu::and(cpu.regs.a, *cpu.data_buffer),
                AluOp::Xor => alu::xor(cpu.regs.a, *cpu.data_buffer),
                AluOp::Or => alu::or(cpu.regs.a, *cpu.data_buffer),
                AluOp::Cp => {
                    let (_, flags) = alu::sub(cpu.regs.a, *cpu.data_buffer, false);
                    (cpu.regs.a, flags)
                }
            };
            cpu.regs.a = result;
            cpu.regs.f = flags;
        })
    }

    fn on_tick(&mut self, f: impl FnOnce(&mut Self)) -> &mut Self {
        if self.phase == Tick {
            f(self)
        }
        self
    }

    fn on_tock(&mut self, f: impl FnOnce(&mut Self)) -> &mut Self {
        if self.phase == Tock {
            f(self)
        }
        self
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
enum R {
    A,
    B,
    C,
    D,
    E,
    H,
    L,
}

impl From<u8> for CommonOperand {
    fn from(encoding: u8) -> Self {
        match encoding {
            0b000 => CommonOperand::Reg(R::B),
            0b001 => CommonOperand::Reg(R::C),
            0b010 => CommonOperand::Reg(R::D),
            0b011 => CommonOperand::Reg(R::E),
            0b100 => CommonOperand::Reg(R::H),
            0b101 => CommonOperand::Reg(R::L),
            0b110 => CommonOperand::DerefHl,
            0b111 => CommonOperand::Reg(R::A),
            _ => panic!(),
        }
    }
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
