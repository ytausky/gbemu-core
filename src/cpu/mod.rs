use self::{MCycle::*, Phase::*};
use self::instruction::RunModeCpu;

use std::ops::{BitAnd, BitOr, Not};

#[cfg(test)]
macro_rules! flags {
    ($($flag:ident),*) => {
        $crate::cpu::Flags {
            $($flag: true,)*
            ..$crate::cpu::Flags {
                z: false,
                n: false,
                h: false,
                cy: false,
            }
        }
    };
}

mod alu;
mod instruction;

#[cfg(test)]
mod tests;

pub struct Cpu {
    pub regs: Regs,
    pub ie: u8,
    pub ime: bool,
    state: State,
    m_cycle: MCycle,
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

enum State {
    Instruction(InstructionExecutionState),
    Interrupt,
}

struct InstructionExecutionState {
    opcode: Opcode,
    bus_data: Option<u8>,
    m1: bool,
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
            ie: 0x00,
            ime: false,
            state: State::Instruction(Default::default()),
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
            data: 0xff,
            addr: 0xffff,
        }
    }
}

impl Cpu {
    pub fn step(&mut self, input: &Input) -> CpuOutput {
        let (mode_transition, output) = match &mut self.state {
            State::Instruction(state) => RunModeCpu {
                regs: &mut self.regs,
                ie: &mut self.ie,
                state,
                m_cycle: self.m_cycle,
                phase: self.phase,
            }
            .step(input),
            State::Interrupt => InterruptModeCpu {
                regs: &mut self.regs,
                ime: &mut self.ime,
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
            self.state = transition.into();
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

impl From<ModeTransition> for State {
    fn from(transition: ModeTransition) -> Self {
        match transition {
            ModeTransition::Instruction(opcode) => State::Instruction(InstructionExecutionState {
                opcode,
                bus_data: None,
                m1: false,
                data: 0xff,
                addr: 0xffff,
            }),
            ModeTransition::Interrupt => State::Interrupt,
        }
    }
}

struct InterruptModeCpu<'a> {
    regs: &'a mut Regs,
    ime: &'a mut bool,
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
                    *self.ime = false;
                    let n = input.r#if.trailing_zeros();
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
    r#if: u8,
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

enum RegSelect {
    R(R),
    F,
    SpH,
    SpL,
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
        u16::from_be_bytes([self.read(h), self.read(l)])
    }

    fn read(&self, reg_select: impl Into<RegSelect>) -> u8 {
        match reg_select.into() {
            RegSelect::R(R::A) => self.a,
            RegSelect::R(R::B) => self.b,
            RegSelect::R(R::C) => self.c,
            RegSelect::R(R::D) => self.d,
            RegSelect::R(R::E) => self.e,
            RegSelect::R(R::H) => self.h,
            RegSelect::R(R::L) => self.l,
            RegSelect::F => self.f.into(),
            RegSelect::SpH => (self.sp >> 8) as u8,
            RegSelect::SpL => (self.sp & 0x00ff) as u8,
        }
    }

    fn write(&mut self, reg_select: impl Into<RegSelect>, data: u8) {
        match reg_select.into() {
            RegSelect::R(R::A) => self.a = data,
            RegSelect::R(R::B) => self.b = data,
            RegSelect::R(R::C) => self.c = data,
            RegSelect::R(R::D) => self.d = data,
            RegSelect::R(R::E) => self.e = data,
            RegSelect::R(R::H) => self.h = data,
            RegSelect::R(R::L) => self.l = data,
            RegSelect::F => self.f = data.into(),
            RegSelect::SpH => self.sp = self.sp & 0x00ff | u16::from(data) << 8,
            RegSelect::SpL => self.sp = self.sp & 0xff00 | u16::from(data),
        }
    }
}

impl From<Flags> for u8 {
    fn from(flags: Flags) -> Self {
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

impl From<R> for RegSelect {
    fn from(r: R) -> Self {
        RegSelect::R(r)
    }
}

impl Dd {
    fn high(self) -> RegSelect {
        match self {
            Dd::Bc => RegSelect::R(R::B),
            Dd::De => RegSelect::R(R::D),
            Dd::Hl => RegSelect::R(R::H),
            Dd::Sp => RegSelect::SpH,
        }
    }

    fn low(self) -> RegSelect {
        match self {
            Dd::Bc => RegSelect::R(R::C),
            Dd::De => RegSelect::R(R::E),
            Dd::Hl => RegSelect::R(R::L),
            Dd::Sp => RegSelect::SpL,
        }
    }
}

impl Qq {
    fn high(self) -> RegSelect {
        match self {
            Qq::Bc => RegSelect::R(R::B),
            Qq::De => RegSelect::R(R::D),
            Qq::Hl => RegSelect::R(R::H),
            Qq::Af => RegSelect::R(R::A),
        }
    }

    fn low(self) -> RegSelect {
        match self {
            Qq::Bc => RegSelect::R(R::C),
            Qq::De => RegSelect::R(R::E),
            Qq::Hl => RegSelect::R(R::L),
            Qq::Af => RegSelect::F,
        }
    }
}
