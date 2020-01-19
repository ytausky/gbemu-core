use self::{MCycle::*, Phase::*};

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
    Run(Stage),
}

enum Stage {
    Execute(ExecuteState),
}

enum ExecuteState {
    Simple(SimpleInstrExecState),
    Complex(ComplexInstrExecState),
}

enum DecodedOpcode {
    Simple(SimpleInstr),
    Complex(Opcode),
}

impl ExecuteState {
    fn new(decoded_opcode: DecodedOpcode) -> Self {
        match decoded_opcode {
            DecodedOpcode::Simple(instr) => ExecuteState::Simple(SimpleInstrExecState {
                instr,
                step: MicroStep::Read,
            }),
            DecodedOpcode::Complex(opcode) => ExecuteState::Complex(ComplexInstrExecState {
                opcode,
                m_cycle: M1,
            }),
        }
    }
}

struct ComplexInstrExecState {
    opcode: Opcode,
    m_cycle: MCycle,
}

#[derive(Clone)]
struct SimpleInstrExecState {
    instr: SimpleInstr,
    step: MicroStep,
}

#[derive(Clone)]
struct SimpleInstr {
    src: Src,
    op: Op,
    dest: Option<CommonOperand>,
}

#[derive(Clone)]
enum Src {
    Common(CommonOperand),
}

#[derive(Clone)]
enum Op {
    Ld,
    Add,
    Adc,
    Sub,
    Sbc,
}

#[derive(Clone, Copy)]
enum CommonOperand {
    Reg(R),
    DerefHl,
}

#[derive(Clone)]
enum MicroStep {
    Read,
    Action(u8),
    Write(AluOutput),
    Fetch,
}

#[derive(Clone, Copy)]
struct Opcode(u8);

impl Opcode {
    fn decode(self) -> Option<DecodedOpcode> {
        match self.split() {
            (0b01, dest, src) => Some(DecodedOpcode::Simple(SimpleInstr {
                src: Src::Common(src.into()),
                op: Op::Ld,
                dest: Some(dest.into()),
            })),
            (0b10, 0b000, src) => Some(DecodedOpcode::Simple(SimpleInstr {
                src: Src::Common(src.into()),
                op: Op::Add,
                dest: Some(CommonOperand::Reg(R::A)),
            })),
            (0b10, 0b001, src) => Some(DecodedOpcode::Simple(SimpleInstr {
                src: Src::Common(src.into()),
                op: Op::Adc,
                dest: Some(CommonOperand::Reg(R::A)),
            })),
            (0b10, 0b010, src) => Some(DecodedOpcode::Simple(SimpleInstr {
                src: Src::Common(src.into()),
                op: Op::Sub,
                dest: Some(CommonOperand::Reg(R::A)),
            })),
            (0b10, 0b011, src) => Some(DecodedOpcode::Simple(SimpleInstr {
                src: Src::Common(src.into()),
                op: Op::Sbc,
                dest: Some(CommonOperand::Reg(R::A)),
            })),
            _ => Some(DecodedOpcode::Complex(self)),
        }
    }

    fn split(self) -> (u8, u8, u8) {
        (self.0 >> 6, (self.0 >> 3) & 0b111, self.0 & 0b111)
    }
}

#[derive(Clone, Copy)]
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
            mode: Mode::Run(Stage::Execute(ExecuteState::Complex(Default::default()))),
            phase: Tick,
        }
    }
}

impl Default for ComplexInstrExecState {
    fn default() -> Self {
        Self {
            opcode: Opcode(0x00),
            m_cycle: M1,
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
        let (new_mode, output) = match &mut self.mode {
            Mode::Run(stage) => RunModeCpu {
                regs: &mut self.regs,
                stage,
                phase: &self.phase,
                input,
            }
            .step(),
        };
        self.mode = new_mode;
        self.phase = match self.phase {
            Tick => Tock,
            Tock => Tick,
        };
        output
    }
}

struct RunModeCpu<'a> {
    regs: &'a mut Regs,
    stage: &'a mut Stage,
    phase: &'a Phase,
    input: &'a Input,
}

impl<'a> RunModeCpu<'a> {
    fn step(&mut self) -> (Mode, CpuOutput) {
        match self.stage {
            Stage::Execute(ExecuteState::Simple(state)) => SimpleInstrExecution {
                regs: self.regs,
                state,
                phase: self.phase,
                input: self.input,
            }
            .step(),
            Stage::Execute(ExecuteState::Complex(state)) => ComplexInstrExecution {
                regs: self.regs,
                new_mode: {
                    let m_cycle = match self.phase {
                        Tick => state.m_cycle,
                        Tock => state.m_cycle.next(),
                    };
                    Mode::Run(Stage::Execute(ExecuteState::Complex(
                        ComplexInstrExecState {
                            opcode: state.opcode,
                            m_cycle,
                        },
                    )))
                },
                state,
                phase: self.phase,
                input: self.input,
            }
            .exec_instr(),
        }
    }
}

struct SimpleInstrExecution<'a> {
    regs: &'a mut Regs,
    state: &'a mut SimpleInstrExecState,
    phase: &'a Phase,
    input: &'a Input,
}

impl<'a> SimpleInstrExecution<'a> {
    fn step(&mut self) -> (Mode, CpuOutput) {
        loop {
            let output = match &self.state.step {
                MicroStep::Read => self.read(),
                MicroStep::Action(operand) => {
                    let operand = *operand;
                    self.act(operand)
                }
                MicroStep::Write(result) => {
                    let result = result.clone();
                    self.write(result)
                }
                MicroStep::Fetch => match self.phase {
                    Tick => Some(Some(BusOp::Read(self.regs.pc))),
                    Tock => {
                        self.regs.pc += 1;
                        return (
                            Mode::Run(Stage::Execute(ExecuteState::new(
                                Opcode(self.input.data.unwrap()).decode().unwrap(),
                            ))),
                            None,
                        );
                    }
                },
            };
            if let Some(output) = output {
                return (
                    Mode::Run(Stage::Execute(ExecuteState::Simple(self.state.clone()))),
                    output,
                );
            }
        }
    }

    fn read(&mut self) -> Option<CpuOutput> {
        match self.state.instr.src {
            Src::Common(CommonOperand::Reg(r)) => {
                self.state.step = MicroStep::Action(*self.regs.reg(r));
                None
            }
            Src::Common(CommonOperand::DerefHl) => match self.phase {
                Tick => Some(Some(BusOp::Read(self.regs.hl()))),
                Tock => {
                    self.state.step = MicroStep::Action(self.input.data.unwrap());
                    Some(None)
                }
            },
        }
    }

    fn act(&mut self, operand: u8) -> Option<CpuOutput> {
        let output = match self.state.instr.op {
            Op::Ld => AluOutput {
                result: operand,
                flags: self.regs.f.clone(),
            },
            Op::Add => alu_addition(&AluInput {
                x: self.regs.a,
                y: operand,
                carry_in: false,
            }),
            Op::Adc => alu_addition(&AluInput {
                x: self.regs.a,
                y: operand,
                carry_in: self.regs.f.cy,
            }),
            Op::Sub => alu_subtraction(&AluInput {
                x: self.regs.a,
                y: operand,
                carry_in: false,
            }),
            Op::Sbc => alu_subtraction(&AluInput {
                x: self.regs.a,
                y: operand,
                carry_in: self.regs.f.cy,
            }),
        };
        self.state.step = MicroStep::Write(output);
        None
    }

    fn write(&mut self, output: AluOutput) -> Option<CpuOutput> {
        self.regs.f = output.flags;
        if let Some(dest) = self.state.instr.dest {
            match dest {
                CommonOperand::Reg(r) => {
                    *self.regs.reg(r) = output.result;
                    self.state.step = MicroStep::Fetch;
                    None
                }
                CommonOperand::DerefHl => match self.phase {
                    Tick => Some(Some(BusOp::Write(self.regs.hl(), output.result))),
                    Tock => {
                        self.state.step = MicroStep::Fetch;
                        Some(None)
                    }
                },
            }
        } else {
            None
        }
    }
}

struct ComplexInstrExecution<'a> {
    regs: &'a mut Regs,
    state: &'a ComplexInstrExecState,
    phase: &'a Phase,
    new_mode: Mode,
    input: &'a Input,
}

impl<'a> ComplexInstrExecution<'a> {
    fn exec_instr(mut self) -> (Mode, CpuOutput) {
        let output = match self.state.opcode.split() {
            (0b00, 0b000, 0b000) => self.nop(),
            (0b01, 0b110, 0b110) => self.halt(),
            (0b11, 0b001, 0b001) => self.ret(),
            _ => unimplemented!(),
        };
        (self.new_mode, output)
    }

    fn nop(&mut self) -> CpuOutput {
        self.fetch()
    }

    fn halt(&mut self) -> CpuOutput {
        unimplemented!()
    }

    fn ret(&mut self) -> CpuOutput {
        match (self.state.m_cycle, *self.phase) {
            (M1, Tick) => Some(BusOp::Read(self.regs.sp)),
            (M1, Tock) => {
                self.regs.pc = self.input.data.unwrap().into();
                self.regs.sp += 1;
                None
            }
            (M2, Tick) => Some(BusOp::Read(self.regs.sp)),
            (M2, Tock) => {
                self.regs.pc |= u16::from(self.input.data.unwrap()) << 8;
                self.regs.sp += 1;
                None
            }
            (M3, _) => None,
            (M4, _) => self.fetch(),
            _ => unreachable!(),
        }
    }

    fn fetch(&mut self) -> CpuOutput {
        match self.phase {
            Phase::Tick => Some(BusOp::Read(self.regs.pc)),
            Phase::Tock => {
                self.regs.pc += 1;
                self.new_mode = Mode::Run(Stage::Execute(ExecuteState::new(
                    Opcode(self.input.data.unwrap()).decode().unwrap(),
                )));
                None
            }
        }
    }
}

fn alu_addition(AluInput { x, y, carry_in }: &AluInput) -> AluOutput {
    let (partial_sum, overflow1) = x.overflowing_add(*y);
    let (sum, overflow2) = partial_sum.overflowing_add((*carry_in).into());
    AluOutput {
        result: sum,
        flags: Flags {
            z: sum == 0,
            n: false,
            h: (x & 0x0f) + (y & 0x0f) + u8::from(*carry_in) > 0x0f,
            cy: overflow1 | overflow2,
        },
    }
}

fn alu_subtraction(AluInput { x, y, carry_in }: &AluInput) -> AluOutput {
    let (partial_diff, overflow1) = x.overflowing_sub(*y);
    let (diff, overflow2) = partial_diff.overflowing_sub((*carry_in).into());
    AluOutput {
        result: diff,
        flags: Flags {
            z: diff == 0,
            n: true,
            h: (x & 0x0f)
                .wrapping_sub(y & 0x0f)
                .wrapping_sub((*carry_in).into())
                > 0x0f,
            cy: overflow1 | overflow2,
        },
    }
}

struct AluInput {
    x: u8,
    y: u8,
    carry_in: bool,
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

#[derive(Clone, Copy)]
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
