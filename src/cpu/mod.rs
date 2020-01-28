use self::{MCycle::*, Phase::*};

use std::ops::{BitAnd, BitOr, Not};

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

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Flags {
    pub z: bool,
    pub n: bool,
    pub h: bool,
    pub cy: bool,
}

impl BitAnd for Flags {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        Flags {
            z: self.z & rhs.z,
            n: self.n & rhs.n,
            h: self.h & rhs.h,
            cy: self.cy & rhs.cy,
        }
    }
}

impl BitOr for Flags {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Flags {
            z: self.z | rhs.z,
            n: self.n | rhs.n,
            h: self.h | rhs.h,
            cy: self.cy | rhs.cy,
        }
    }
}

impl Not for Flags {
    type Output = Self;

    fn not(self) -> Self::Output {
        Flags {
            z: !self.z,
            n: !self.n,
            h: !self.h,
            cy: !self.cy,
        }
    }
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

    fn read_dd(&self, dd: Dd) -> u16 {
        match dd {
            Dd::Bc => self.bc(),
            Dd::De => self.de(),
            Dd::Hl => self.hl(),
            Dd::Sp => self.sp,
        }
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

    fn read_qq_h(&self, qq: Qq) -> u8 {
        match qq {
            Qq::Bc => self.b,
            Qq::De => self.d,
            Qq::Hl => self.h,
            Qq::Af => self.a,
        }
    }

    fn read_qq_l(&self, qq: Qq) -> u8 {
        match qq {
            Qq::Bc => self.c,
            Qq::De => self.e,
            Qq::Hl => self.l,
            Qq::Af => (&self.f).into(),
        }
    }

    fn write_qq_h(&mut self, qq: Qq, data: u8) {
        match qq {
            Qq::Bc => self.b = data,
            Qq::De => self.d = data,
            Qq::Hl => self.h = data,
            Qq::Af => self.a = data,
        }
    }

    fn write_qq_l(&mut self, qq: Qq, data: u8) {
        match qq {
            Qq::Bc => self.c = data,
            Qq::De => self.e = data,
            Qq::Hl => self.l = data,
            Qq::Af => self.f = data.into(),
        }
    }
}

impl From<&Flags> for u8 {
    fn from(flags: &Flags) -> Self {
        (if flags.z { 0x80 } else { 0x00 })
            | if flags.n { 0x40 } else { 0x00 }
            | if flags.h { 0x20 } else { 0x00 }
            | if flags.cy { 0x10 } else { 0x00 }
    }
}

impl From<u8> for Flags {
    fn from(flags: u8) -> Self {
        Flags {
            z: flags & 0x80 > 0,
            n: flags & 0x40 > 0,
            h: flags & 0x20 > 0,
            cy: flags & 0x10 > 0,
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
            (0b00, operand, 0b100) => self.inc_m(operand.into()),
            (0b00, dest, 0b110) => self.ld_m_s(dest.into(), S::N),
            (0b00, 0b001, 0b000) => self.ld_deref_nn_sp(),
            (0b00, 0b001, 0b010) => self.ld_a_deref_bc(),
            (0b00, 0b010, 0b010) => self.ld_deref_de_a(),
            (0b00, 0b011, 0b010) => self.ld_a_deref_de(),
            (0b00, 0b100, 0b010) => self.ld_deref_hli_a(),
            (0b00, 0b101, 0b010) => self.ld_a_deref_hli(),
            (0b00, 0b110, 0b010) => self.ld_deref_hld_a(),
            (0b00, 0b111, 0b010) => self.ld_a_deref_hld(),
            (0b01, 0b110, 0b110) => self.halt(),
            (0b01, dest, src) => self.ld_m_s(dest.into(), S::M(src.into())),
            (0b10, op, src) => self.alu_op(op.into(), S::M(src.into())),
            (0b11, dest, 0b001) if dest & 0b001 == 0 => self.pop_qq((dest >> 1).into()),
            (0b11, src, 0b101) if src & 0b001 == 0 => self.push_qq((src >> 1).into()),
            (0b11, op, 0b110) => self.alu_op(op.into(), S::N),
            (0b11, 0b001, 0b001) => self.ret(),
            (0b11, 0b100, 0b000) => self.ld_deref_n_a(),
            (0b11, 0b100, 0b010) => self.ld_deref_c_a(),
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

    fn ld_m_s(&mut self, dest: M, src: S) -> &mut Self {
        self.read_s(src, |cpu| cpu)
            .write_m(dest)
            .cycle(|cpu| cpu.fetch())
    }

    fn ld_a_deref_bc(&mut self) -> &mut Self {
        self.cycle(|cpu| cpu.bus_read(cpu.regs.bc()))
            .cycle(|cpu| cpu.write_r(R::A, *cpu.data).fetch())
    }

    fn ld_a_deref_de(&mut self) -> &mut Self {
        self.cycle(|cpu| cpu.bus_read(cpu.regs.de()))
            .cycle(|cpu| cpu.write_r(R::A, *cpu.data).fetch())
    }

    fn ld_a_deref_c(&mut self) -> &mut Self {
        self.cycle(|cpu| cpu.bus_read(0xff00 | u16::from(cpu.regs.c)))
            .cycle(|cpu| cpu.write_r(R::A, *cpu.data).fetch())
    }

    fn ld_deref_c_a(&mut self) -> &mut Self {
        self.cycle(|cpu| cpu.bus_write(0xff00 | u16::from(cpu.regs.c), cpu.regs.a))
            .cycle(|cpu| cpu.fetch())
    }

    fn ld_a_deref_n(&mut self) -> &mut Self {
        self.cycle(|cpu| cpu.read_immediate())
            .cycle(|cpu| cpu.bus_read(0xff00 | u16::from(*cpu.data)))
            .cycle(|cpu| cpu.write_r(R::A, *cpu.data).fetch())
    }

    fn ld_deref_n_a(&mut self) -> &mut Self {
        self.cycle(|cpu| cpu.read_immediate())
            .cycle(|cpu| cpu.bus_write(0xff00 | u16::from(*cpu.data), cpu.regs.a))
            .cycle(|cpu| cpu.write_r(R::A, *cpu.data).fetch())
    }

    fn ld_a_deref_nn(&mut self) -> &mut Self {
        self.cycle(|cpu| cpu.read_immediate().write_addr_l())
            .cycle(|cpu| cpu.read_immediate().write_addr_h())
            .cycle(|cpu| cpu.bus_read(*cpu.addr))
            .cycle(|cpu| cpu.write_r(R::A, *cpu.data).fetch())
    }

    fn ld_deref_nn_a(&mut self) -> &mut Self {
        self.cycle(|cpu| cpu.read_immediate().write_addr_l())
            .cycle(|cpu| cpu.read_immediate().write_addr_h())
            .cycle(|cpu| cpu.bus_write(*cpu.addr, cpu.regs.a))
            .cycle(|cpu| cpu.fetch())
    }

    fn ld_a_deref_hli(&mut self) -> &mut Self {
        self.cycle(|cpu| cpu.bus_read(cpu.regs.hl()).increment_hl())
            .cycle(|cpu| cpu.write_r(R::A, *cpu.data).fetch())
    }

    fn ld_a_deref_hld(&mut self) -> &mut Self {
        self.cycle(|cpu| cpu.bus_read(cpu.regs.hl()).decrement_hl())
            .cycle(|cpu| cpu.write_r(R::A, *cpu.data).fetch())
    }

    fn ld_deref_bc_a(&mut self) -> &mut Self {
        self.cycle(|cpu| cpu.bus_write(cpu.regs.bc(), cpu.regs.a))
            .cycle(|cpu| cpu.fetch())
    }

    fn ld_deref_de_a(&mut self) -> &mut Self {
        self.cycle(|cpu| cpu.bus_write(cpu.regs.de(), cpu.regs.a))
            .cycle(|cpu| cpu.fetch())
    }

    fn ld_deref_hli_a(&mut self) -> &mut Self {
        self.cycle(|cpu| cpu.bus_write(cpu.regs.hl(), cpu.regs.a).increment_hl())
            .cycle(|cpu| cpu.fetch())
    }

    fn ld_deref_hld_a(&mut self) -> &mut Self {
        self.cycle(|cpu| cpu.bus_write(cpu.regs.hl(), cpu.regs.a).decrement_hl())
            .cycle(|cpu| cpu.fetch())
    }

    fn ld_dd_nn(&mut self, dd: Dd) -> &mut Self {
        self.cycle(|cpu| cpu.read_immediate().write_addr_l())
            .cycle(|cpu| cpu.read_immediate().write_addr_h())
            .cycle(|cpu| cpu.write_dd(dd, *cpu.addr).fetch())
    }

    fn ld_sp_hl(&mut self) -> &mut Self {
        self.cycle(|cpu| cpu.bus_no_op())
            .cycle(|cpu| cpu.write_dd(Dd::Sp, cpu.regs.hl()).fetch())
    }

    fn push_qq(&mut self, qq: Qq) -> &mut Self {
        self.cycle(|cpu| cpu.bus_no_op().decrement_dd(Dd::Sp))
            .cycle(|cpu| {
                cpu.bus_write(cpu.regs.sp, cpu.regs.read_qq_h(qq))
                    .decrement_dd(Dd::Sp)
            })
            .cycle(|cpu| cpu.bus_write(cpu.regs.sp, cpu.regs.read_qq_l(qq)))
            .cycle(|cpu| cpu.fetch())
    }

    fn pop_qq(&mut self, qq: Qq) -> &mut Self {
        self.cycle(|cpu| cpu.bus_read(cpu.regs.sp).write_qq_l(qq).increment_sp())
            .cycle(|cpu| cpu.bus_read(cpu.regs.sp).write_qq_h(qq).increment_sp())
            .cycle(|cpu| cpu.fetch())
    }

    fn ldhl_sp_e(&mut self) -> &mut Self {
        self.cycle(|cpu| cpu.read_immediate())
            .cycle(|cpu| {
                let cpu = cpu.alu_op(AluOp::Add, low_byte(cpu.regs.sp), *cpu.data);
                cpu.write_r(R::L, cpu.alu_result)
                    .on_tock(|cpu| cpu.alu_flags.z = false)
                    .write_f(ALL_FLAGS)
                    .bus_no_op()
            })
            .cycle(|cpu| {
                let cpu = cpu.alu_op(
                    AluOp::Adc,
                    high_byte(cpu.regs.sp),
                    sign_extension(*cpu.data),
                );
                cpu.write_r(R::H, cpu.alu_result).fetch()
            })
    }

    fn ld_deref_nn_sp(&mut self) -> &mut Self {
        self.cycle(|cpu| cpu.read_immediate().write_addr_l())
            .cycle(|cpu| cpu.read_immediate().write_addr_h())
            .cycle(|cpu| {
                cpu.bus_write(*cpu.addr, low_byte(cpu.regs.sp))
                    .increment_addr()
            })
            .cycle(|cpu| cpu.bus_write(*cpu.addr, high_byte(cpu.regs.sp)))
            .cycle(|cpu| cpu.fetch())
    }

    fn alu_op(&mut self, op: AluOp, rhs: S) -> &mut Self {
        self.read_s(rhs, |cpu| cpu)
            .micro_op(|cpu| {
                cpu.alu_op(op, cpu.regs.a, *cpu.data);
                cpu.write_r(R::A, cpu.alu_result);
                cpu.write_f(ALL_FLAGS)
            })
            .cycle(|cpu| cpu.fetch())
    }

    fn inc_m(&mut self, m: M) -> &mut Self {
        self.read_s(S::M(m), |cpu| {
            cpu.alu_op(AluOp::Add, *cpu.data, 0x01);
            cpu.on_tock(|cpu| *cpu.data = cpu.alu_result);
            cpu.write_f(flags!(z, n, h))
        })
        .write_m(m)
        .cycle(|cpu| cpu.fetch())
    }

    fn ret(&mut self) -> &mut Self {
        self.microinstruction(|cpu| cpu.pop_byte().write_addr_l())
            .microinstruction(|cpu| cpu.pop_byte().write_addr_h())
            .microinstruction(|cpu| cpu.write_pc())
            .microinstruction(|cpu| cpu.fetch())
    }

    fn read_s<F>(&mut self, s: S, f: F) -> &mut Self
    where
        F: for<'r, 's> FnOnce(&'s mut CpuProxy<'r>) -> &'s mut CpuProxy<'r>,
    {
        match s {
            S::M(M::R(r)) => self.micro_op(|cpu| f(cpu.read_r(r))),
            S::M(M::DerefHl) => self.cycle(|cpu| f(cpu.bus_read(cpu.regs.hl()))),
            S::N => self.cycle(|cpu| f(cpu.read_immediate())),
        }
    }

    fn write_m(&mut self, m: M) -> &mut Self {
        match m {
            M::R(r) => self.micro_op(|cpu| cpu.write_r(r, *cpu.data)),
            M::DerefHl => self.cycle(|cpu| cpu.bus_write(cpu.regs.hl(), *cpu.data)),
        }
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

    fn microinstruction<F>(&mut self, f: F) -> &mut Self
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

    fn execute_microinstruction(&mut self, microinstruction: &Microinstruction) -> CpuOutput {
        let addr = match microinstruction.word_select {
            WordSelect::Pc => self.regs.pc,
            WordSelect::Sp => self.regs.sp,
        };

        if *self.phase == Tock {
            if let Some(byte_writeback) = &microinstruction.byte_writeback {
                let byte = match byte_writeback.src {
                    ByteWritebackSrc::Bus => self.input.data.unwrap(),
                };
                match byte_writeback.dest {
                    ByteWritebackDest::AddrH => {
                        self.state.addr = self.state.addr & 0x00ff | u16::from(byte) << 8
                    }
                    ByteWritebackDest::AddrL => {
                        self.state.addr = self.state.addr & 0xff00 | u16::from(byte)
                    }
                }
            }
            if let Some(word_writeback) = &microinstruction.word_writeback {
                let word = match word_writeback.src {
                    WordWritebackSrc::Addr => self.state.addr,
                    WordWritebackSrc::Inc => addr + 1,
                };
                match word_writeback.dest {
                    WordWritebackDest::Pc => self.regs.pc = word,
                    WordWritebackDest::Sp => self.regs.sp = word,
                }
            }

            if microinstruction.write_opcode {
                self.mode_transition = Some(ModeTransition::Run(Opcode(self.input.data.unwrap())))
            }
        }

        if *self.phase == Tick && microinstruction.bus_read {
            Some(BusOp::Read(addr))
        } else {
            None
        }
    }

    fn cpu_proxy(&mut self) -> CpuProxy {
        CpuProxy {
            regs: self.regs,
            data: &mut self.state.data,
            addr: &mut self.state.addr,
            alu_result: 0xff,
            alu_flags: Default::default(),
            mode_transition: &mut self.mode_transition,
            phase: *self.phase,
            input: self.input,
            output: None,
        }
    }
}

struct Microinstruction {
    word_select: WordSelect,
    byte_writeback: Option<ByteWriteback>,
    word_writeback: Option<WordWriteback>,
    bus_read: bool,
    write_opcode: bool,
}

impl Default for Microinstruction {
    fn default() -> Self {
        Self {
            word_select: WordSelect::Pc,
            byte_writeback: None,
            word_writeback: None,
            bus_read: false,
            write_opcode: false,
        }
    }
}

impl Microinstruction {
    fn pop_byte(&mut self) -> &mut Self {
        self.word_select = WordSelect::Sp;
        self.word_writeback = Some(WordWriteback {
            dest: WordWritebackDest::Sp,
            src: WordWritebackSrc::Inc,
        });
        self.bus_read = true;
        self
    }

    fn write_addr_l(&mut self) -> &mut Self {
        self.byte_writeback = Some(ByteWriteback {
            dest: ByteWritebackDest::AddrL,
            src: ByteWritebackSrc::Bus,
        });
        self
    }

    fn write_addr_h(&mut self) -> &mut Self {
        self.byte_writeback = Some(ByteWriteback {
            dest: ByteWritebackDest::AddrH,
            src: ByteWritebackSrc::Bus,
        });
        self
    }

    fn write_pc(&mut self) -> &mut Self {
        self.word_writeback = Some(WordWriteback {
            dest: WordWritebackDest::Pc,
            src: WordWritebackSrc::Addr,
        });
        self
    }

    fn fetch(&mut self) -> &mut Self {
        self.word_select = WordSelect::Pc;
        self.word_writeback = Some(WordWriteback {
            dest: WordWritebackDest::Pc,
            src: WordWritebackSrc::Inc,
        });
        self.bus_read = true;
        self.write_opcode = true;
        self
    }
}

enum WordSelect {
    Pc,
    Sp,
}

struct ByteWriteback {
    dest: ByteWritebackDest,
    src: ByteWritebackSrc,
}

enum ByteWritebackDest {
    AddrH,
    AddrL,
}

enum ByteWritebackSrc {
    Bus,
}

struct WordWriteback {
    dest: WordWritebackDest,
    src: WordWritebackSrc,
}

enum WordWritebackDest {
    Pc,
    Sp,
}

enum WordWritebackSrc {
    Addr,
    Inc,
}

struct CpuProxy<'a> {
    regs: &'a mut Regs,
    data: &'a mut u8,
    addr: &'a mut u16,
    alu_result: u8,
    alu_flags: Flags,
    phase: Phase,
    mode_transition: &'a mut Option<ModeTransition>,
    input: &'a Input,
    output: Option<CpuOutput>,
}

impl<'a> CpuProxy<'a> {
    fn read_immediate(&mut self) -> &mut Self {
        self.bus_read(self.regs.pc).increment_pc()
    }

    fn increment_hl(&mut self) -> &mut Self {
        self.write_dd(Dd::Hl, self.regs.hl().wrapping_add(1))
    }

    fn decrement_hl(&mut self) -> &mut Self {
        self.write_dd(Dd::Hl, self.regs.hl() - 1)
    }

    fn decrement_dd(&mut self, dd: Dd) -> &mut Self {
        self.write_dd(dd, self.regs.read_dd(dd) - 1)
    }

    fn write_dd(&mut self, dd: Dd, addr: u16) -> &mut Self {
        let h = high_byte(addr);
        let l = low_byte(addr);
        self.on_tock(|cpu| match dd {
            Dd::Bc => {
                cpu.regs.b = h;
                cpu.regs.c = l;
            }
            Dd::De => {
                cpu.regs.d = h;
                cpu.regs.e = l;
            }
            Dd::Hl => {
                cpu.regs.h = h;
                cpu.regs.l = l;
            }
            Dd::Sp => cpu.regs.sp = addr,
        })
    }

    fn write_qq_h(&mut self, qq: Qq) -> &mut Self {
        self.on_tock(|cpu| cpu.regs.write_qq_h(qq, *cpu.data))
    }

    fn write_qq_l(&mut self, qq: Qq) -> &mut Self {
        self.on_tock(|cpu| cpu.regs.write_qq_l(qq, *cpu.data))
    }

    fn increment_pc(&mut self) -> &mut Self {
        self.on_tock(|cpu| cpu.regs.pc += 1)
    }

    fn increment_sp(&mut self) -> &mut Self {
        self.on_tock(|cpu| cpu.regs.sp += 1)
    }

    fn increment_addr(&mut self) -> &mut Self {
        self.on_tock(|cpu| *cpu.addr += 1)
    }

    fn write_addr_h(&mut self) -> &mut Self {
        self.on_tock(|cpu| *cpu.addr = *cpu.addr & 0x00ff | (u16::from(*cpu.data) << 8))
    }

    fn write_addr_l(&mut self) -> &mut Self {
        self.on_tock(|cpu| *cpu.addr = *cpu.addr & 0xff00 | u16::from(*cpu.data))
    }

    fn decode(&mut self) -> &mut Self {
        self.on_tock(|cpu| *cpu.mode_transition = Some(ModeTransition::Run(Opcode(*cpu.data))))
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
                *self.data = self.input.data.unwrap();
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
        self.on_tick(|cpu| *cpu.data = *cpu.regs.select_r(r))
    }

    fn write_r(&mut self, r: R, data: u8) -> &mut Self {
        self.on_tock(|cpu| *cpu.regs.select_r_mut(r) = data)
    }

    fn write_f(&mut self, mask: Flags) -> &mut Self {
        self.on_tock(|cpu| cpu.regs.f = cpu.regs.f & !mask | cpu.alu_flags & mask)
    }

    fn alu_op(&mut self, op: AluOp, lhs: u8, rhs: u8) -> &mut Self {
        self.on_tock(|cpu| {
            let (result, flags) = match op {
                AluOp::Add => alu::add(lhs, rhs, false),
                AluOp::Adc => alu::add(lhs, rhs, cpu.regs.f.cy),
                AluOp::Sub => alu::sub(lhs, rhs, false),
                AluOp::Sbc => alu::sub(lhs, rhs, cpu.regs.f.cy),
                AluOp::And => alu::and(lhs, rhs),
                AluOp::Xor => alu::xor(lhs, rhs),
                AluOp::Or => alu::or(lhs, rhs),
                AluOp::Cp => {
                    let (_, flags) = alu::sub(lhs, rhs, false);
                    (lhs, flags)
                }
            };
            cpu.alu_result = result;
            cpu.alu_flags = flags;
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
