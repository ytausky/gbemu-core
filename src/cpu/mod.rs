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
    addr_h: u8,
    addr_l: u8,
}

#[derive(Clone)]
enum S {
    M(M),
    N,
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
            addr_h: 0xff,
            addr_l: 0xff,
        }
    }
}

impl Regs {
    fn bc(&self) -> u16 {
        self.pair(R::B, R::C)
    }

    fn de(&self) -> u16 {
        self.pair(R::D, R::E)
    }

    fn hl(&self) -> u16 {
        self.pair(R::H, R::L)
    }

    fn pair(&self, h: R, l: R) -> u16 {
        (u16::from(*self.select_r(h)) << 8) + u16::from(*self.select_r(l))
    }

    fn select_r(&self, r: R) -> &u8 {
        match r {
            R::A => &self.a,
            R::B => &self.b,
            R::C => &self.c,
            R::D => &self.d,
            R::E => &self.e,
            R::H => &self.h,
            R::L => &self.l,
        }
    }

    fn select_r_mut(&mut self, r: R) -> &mut u8 {
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
                addr_h: 0xff,
                addr_l: 0xff,
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
            (0b00, dest, 0b110) => self.ld(dest.into(), S::N),
            (0b00, 0b001, 0b010) => self.ld_a_deref_bc(),
            (0b00, 0b011, 0b010) => self.ld_a_deref_de(),
            (0b01, 0b110, 0b110) => self.halt(),
            (0b01, dest, src) => self.ld(dest.into(), S::M(src.into())),
            (0b10, op, src) => self.alu_op(op.into(), S::M(src.into())),
            (0b11, op, 0b110) => self.alu_op(op.into(), S::N),
            (0b11, 0b001, 0b001) => self.ret(),
            (0b11, 0b100, 0b000) => self.ld_deref_n_a(),
            (0b11, 0b100, 0b010) => self.ld_deref_c_a(),
            (0b11, 0b101, 0b010) => self.ld_deref_nn_a(),
            (0b11, 0b110, 0b000) => self.ld_a_deref_n(),
            (0b11, 0b110, 0b010) => self.ld_a_deref_c(),
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

    fn ld(&mut self, dest: M, src: S) -> &mut Self {
        self.read_s(src).write_m(dest).cycle(|cpu| cpu.fetch())
    }

    fn ld_a_deref_bc(&mut self) -> &mut Self {
        self.cycle(|cpu| cpu.bus_read(cpu.regs.bc()).write_r(R::A))
            .cycle(|cpu| cpu.fetch())
    }

    fn ld_a_deref_de(&mut self) -> &mut Self {
        self.cycle(|cpu| cpu.bus_read(cpu.regs.de()).write_r(R::A))
            .cycle(|cpu| cpu.fetch())
    }

    fn ld_a_deref_c(&mut self) -> &mut Self {
        self.cycle(|cpu| cpu.bus_read(0xff00 | u16::from(cpu.regs.c)).write_r(R::A))
            .cycle(|cpu| cpu.fetch())
    }

    fn ld_deref_c_a(&mut self) -> &mut Self {
        self.cycle(|cpu| cpu.bus_write(0xff00 | u16::from(cpu.regs.c), cpu.regs.a))
            .cycle(|cpu| cpu.fetch())
    }

    fn ld_a_deref_n(&mut self) -> &mut Self {
        self.cycle(|cpu| cpu.read_immediate())
            .cycle(|cpu| cpu.bus_read(0xff00 | u16::from(*cpu.data_buffer)))
            .cycle(|cpu| cpu.write_r(R::A).fetch())
    }

    fn ld_deref_n_a(&mut self) -> &mut Self {
        self.cycle(|cpu| cpu.read_immediate())
            .cycle(|cpu| cpu.bus_write(0xff00 | u16::from(*cpu.data_buffer), cpu.regs.a))
            .cycle(|cpu| cpu.write_r(R::A).fetch())
    }

    fn ld_a_deref_nn(&mut self) -> &mut Self {
        self.cycle(|cpu| cpu.read_immediate().write_addr_l())
            .cycle(|cpu| cpu.read_immediate().write_addr_h())
            .cycle(|cpu| cpu.bus_read(cpu.addr()).write_r(R::A))
            .cycle(|cpu| cpu.fetch())
    }

    fn ld_deref_nn_a(&mut self) -> &mut Self {
        self.cycle(|cpu| cpu.read_immediate().write_addr_l())
            .cycle(|cpu| cpu.read_immediate().write_addr_h())
            .cycle(|cpu| cpu.bus_write(cpu.addr(), cpu.regs.a))
            .cycle(|cpu| cpu.fetch())
    }

    fn read_s(&mut self, s: S) -> &mut Self {
        match s {
            S::M(M::R(r)) => self.micro_op(|cpu| cpu.read_r(r)),
            S::M(M::DerefHl) => self.cycle(|cpu| cpu.bus_read(cpu.regs.hl())),
            S::N => self.cycle(|cpu| cpu.read_immediate()),
        }
    }

    fn write_m(&mut self, m: M) -> &mut Self {
        match m {
            M::R(r) => self.micro_op(|cpu| cpu.write_r(r)),
            M::DerefHl => self.cycle(|cpu| cpu.bus_write(cpu.regs.hl(), *cpu.data_buffer)),
        }
    }

    fn alu_op(&mut self, op: AluOp, rhs: S) -> &mut Self {
        self.read_s(rhs)
            .micro_op(|cpu| cpu.alu_op(op))
            .cycle(|cpu| cpu.fetch())
    }

    fn ret(&mut self) -> &mut Self {
        self.cycle(|cpu| cpu.bus_read(cpu.regs.sp).increment_sp().write_addr_l())
            .cycle(|cpu| cpu.bus_read(cpu.regs.sp).increment_sp().write_addr_h())
            .cycle(|cpu| cpu.bus_no_op().write_pc())
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
            addr_h: &mut self.state.addr_h,
            addr_l: &mut self.state.addr_l,
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
    addr_h: &'a mut u8,
    addr_l: &'a mut u8,
    phase: Phase,
    mode_transition: &'a mut Option<ModeTransition>,
    input: &'a Input,
    output: Option<CpuOutput>,
}

impl<'a> CpuProxy<'a> {
    fn read_immediate(&mut self) -> &mut Self {
        self.bus_read(self.regs.pc).increment_pc()
    }

    fn increment_pc(&mut self) -> &mut Self {
        self.on_tock(|cpu| cpu.regs.pc += 1)
    }

    fn increment_sp(&mut self) -> &mut Self {
        self.on_tock(|cpu| cpu.regs.sp += 1)
    }

    fn write_addr_h(&mut self) -> &mut Self {
        self.on_tock(|cpu| *cpu.addr_h = *cpu.data_buffer)
    }

    fn write_addr_l(&mut self) -> &mut Self {
        self.on_tock(|cpu| *cpu.addr_l = *cpu.data_buffer)
    }

    fn write_pc(&mut self) -> &mut Self {
        self.on_tock(|cpu| cpu.regs.pc = cpu.addr())
    }

    fn addr(&self) -> u16 {
        u16::from(*self.addr_h) << 8 | u16::from(*self.addr_l)
    }

    fn decode(&mut self) -> &mut Self {
        self.on_tock(|cpu| {
            *cpu.mode_transition = Some(ModeTransition::Run(Opcode(*cpu.data_buffer)))
        })
    }

    fn fetch(&mut self) -> &mut Self {
        self.bus_read(self.regs.pc).increment_pc().decode()
    }

    fn bus_no_op(&mut self) -> &mut Self {
        self.output = Some(None);
        self
    }

    fn bus_read(&mut self, addr: u16) -> &mut Self {
        self.output = Some(match self.phase {
            Tick => Some(BusOp::Read(addr)),
            Tock => {
                *self.data_buffer = self.input.data.unwrap();
                None
            }
        });
        self
    }

    fn bus_write(&mut self, addr: u16, data: u8) -> &mut Self {
        self.output = Some(match self.phase {
            Tick => Some(BusOp::Write(addr, data)),
            Tock => None,
        });
        self
    }

    fn read_r(&mut self, r: R) -> &mut Self {
        self.on_tick(|cpu| *cpu.data_buffer = *cpu.regs.select_r(r))
    }

    fn write_r(&mut self, r: R) -> &mut Self {
        self.on_tock(|cpu| *cpu.regs.select_r_mut(r) = *cpu.data_buffer)
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
