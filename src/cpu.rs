use self::{MCycle::*, Phase::*};

pub struct Cpu {
    regs: Regs,
    mode: Mode,
    phase: Phase,
}

#[derive(Default)]
struct Regs {
    a: u8,
    f: CpuFlags,
    b: u8,
    c: u8,
    d: u8,
    e: u8,
    h: u8,
    l: u8,
    pc: u16,
    sp: u16,
}

#[derive(Clone, Debug, Default, PartialEq)]
struct CpuFlags {
    z: bool,
    n: bool,
    h: bool,
    cy: bool,
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
    pub fn step(&mut self, input: &CpuInput) -> CpuOutput {
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
    input: &'a CpuInput,
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
    input: &'a CpuInput,
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
    input: &'a CpuInput,
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
        flags: CpuFlags {
            z: sum == 0,
            n: false,
            h: (x & 0x0f) + (y & 0x0f) + u8::from(*carry_in) > 0x0f,
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
    flags: CpuFlags,
}

pub struct CpuInput {
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

#[derive(Debug, PartialEq)]
pub enum BusOp {
    Read(u16),
    Write(u16, u8),
}

#[cfg(test)]
mod tests {
    use super::*;

    const RS: &[R] = &[R::A, R::B, R::C, R::D, R::E, R::H, R::L];

    #[test]
    fn ld_r_r() {
        for &dest in RS {
            for &src in RS {
                test_ld_r_r(dest, src)
            }
        }
    }

    fn test_ld_r_r(dest: R, src: R) {
        let mut cpu = Cpu::default();
        let data = 0x42;
        *cpu.regs.reg(src) = data;
        cpu.test_opcode(
            &encode_ld_r_r(dest, src),
            &[
                (CpuInput::with_data(None), Some(BusOp::Read(0x0001))),
                (CpuInput::with_data(Some(0x00)), None),
            ],
        );
        assert_eq!(*cpu.regs.reg(dest), data)
    }

    fn encode_ld_r_r(dest: R, src: R) -> Vec<u8> {
        vec![0b01_000_000 | (dest.code() << 3) | src.code()]
    }

    impl R {
        fn code(self) -> u8 {
            match self {
                R::A => 0b111,
                R::B => 0b000,
                R::C => 0b001,
                R::D => 0b010,
                R::E => 0b011,
                R::H => 0b100,
                R::L => 0b101,
            }
        }
    }

    #[test]
    fn ld_r_deref_hl() {
        for &dest in RS {
            test_ld_r_deref_hl(dest)
        }
    }

    fn test_ld_r_deref_hl(dest: R) {
        let mut cpu = Cpu::default();
        let data = 0x42;
        cpu.regs.h = 0x12;
        cpu.regs.l = 0x34;
        cpu.test_opcode(
            &encode_ld_r_deref_hl(dest),
            &[
                (CpuInput::with_data(None), Some(BusOp::Read(0x1234))),
                (CpuInput::with_data(Some(data)), None),
                (CpuInput::with_data(None), Some(BusOp::Read(0x0001))),
                (CpuInput::with_data(Some(0x00)), None),
            ],
        );
        assert_eq!(*cpu.regs.reg(dest), data)
    }

    fn encode_ld_r_deref_hl(dest: R) -> Vec<u8> {
        vec![0b01_000_110 | (dest.code() << 3)]
    }

    #[test]
    fn ld_deref_hl_r() {
        for &src in RS {
            test_ld_deref_hl_r(src)
        }
    }

    fn test_ld_deref_hl_r(src: R) {
        let mut cpu = Cpu::default();
        let data = 0x42;
        cpu.regs.h = 0x12;
        cpu.regs.l = 0x34;
        *cpu.regs.reg(src) = data;
        cpu.test_opcode(
            &encode_ld_deref_hl_r(src),
            &[
                (
                    CpuInput::with_data(None),
                    Some(BusOp::Write(cpu.regs.hl(), data)),
                ),
                (CpuInput::with_data(None), None),
                (CpuInput::with_data(None), Some(BusOp::Read(0x0001))),
                (CpuInput::with_data(Some(0x00)), None),
            ],
        );
    }

    fn encode_ld_deref_hl_r(src: R) -> Vec<u8> {
        vec![0b01_110_000 | src.code()]
    }

    #[test]
    fn add() {
        for test_case in ADDITION_TEST_CASES {
            if !test_case.input.carry_in {
                test_adder_for_all_r(&encode_add_a_r, test_case);
                test_add_deref_hl(test_case)
            }
        }
    }

    fn encode_add_a_r(r: R) -> Vec<u8> {
        vec![0b10_000_000 | r.code()]
    }

    fn test_add_deref_hl(test_case: &AluTestCase) {
        const ADD_DEREF_HL: &[u8] = &[0x86];
        test_addition_deref_hl(ADD_DEREF_HL, test_case)
    }

    fn test_addition_deref_hl(opcode: &[u8], test_case: &AluTestCase) {
        let mut cpu = Cpu::default();
        cpu.regs.a = test_case.input.x;
        cpu.regs.f.cy = test_case.input.carry_in;
        cpu.regs.h = 0x12;
        cpu.regs.l = 0x34;
        cpu.test_opcode(
            opcode,
            &[
                (CpuInput::with_data(None), Some(BusOp::Read(cpu.regs.hl()))),
                (CpuInput::with_data(Some(test_case.input.y)), None),
                (CpuInput::with_data(None), Some(BusOp::Read(0x0001))),
                (CpuInput::with_data(Some(0x00)), None),
            ],
        );
        assert_eq!(cpu.regs.a, test_case.expected.result);
        assert_eq!(cpu.regs.f, test_case.expected.flags)
    }

    #[test]
    fn adc_a_r() {
        for test_case in ADDITION_TEST_CASES {
            test_adder_for_all_r(&encode_adc_a_r, test_case);
            test_adc_deref_hl(test_case)
        }
    }

    fn encode_adc_a_r(r: R) -> Vec<u8> {
        vec![0b10_001_000 | r.code()]
    }

    fn test_adc_deref_hl(test_case: &AluTestCase) {
        const ADC_A_DEREF_HL: &[u8] = &[0x8e];
        test_addition_deref_hl(ADC_A_DEREF_HL, test_case)
    }

    fn test_adder_for_all_r<F: Fn(R) -> Vec<u8>>(encoder: &F, test_case: &AluTestCase) {
        if test_case.is_applicable_for_a() {
            test_adder(R::A, encoder, test_case)
        }
        for &r in &[R::B, R::C, R::D, R::E, R::H, R::L] {
            test_adder(r, encoder, test_case)
        }
    }

    fn test_adder<F: Fn(R) -> Vec<u8>>(r: R, encoder: &F, test_case: &AluTestCase) {
        let mut cpu = Cpu::default();
        cpu.regs.a = test_case.input.x;
        *cpu.regs.reg(r) = test_case.input.y;
        cpu.regs.f.cy = test_case.input.carry_in;
        cpu.test_opcode(
            &encoder(r),
            &[
                (CpuInput::with_data(None), Some(BusOp::Read(0x0001))),
                (CpuInput::with_data(Some(0x00)), None),
            ],
        );
        assert_eq!(cpu.regs.a, test_case.expected.result);
        assert_eq!(cpu.regs.f, test_case.expected.flags)
    }

    struct AluTestCase {
        input: AluInput,
        expected: AluOutput,
    }

    impl AluTestCase {
        fn is_applicable_for_a(&self) -> bool {
            self.input.x == self.input.y
        }
    }

    macro_rules! cpu_flags {
        ($($flag:ident),*) => {
            CpuFlags {
                $($flag: true,)*
                ..CpuFlags {
                    z: false,
                    n: false,
                    h: false,
                    cy: false,
                }
            }
        };
    }

    const ADDITION_TEST_CASES: &[AluTestCase] = &[
        AluTestCase {
            input: AluInput {
                x: 0x08,
                y: 0x08,
                carry_in: false,
            },
            expected: AluOutput {
                result: 0x10,
                flags: cpu_flags!(h),
            },
        },
        AluTestCase {
            input: AluInput {
                x: 0x80,
                y: 0x80,
                carry_in: false,
            },
            expected: AluOutput {
                result: 0x00,
                flags: cpu_flags!(z, cy),
            },
        },
        AluTestCase {
            input: AluInput {
                x: 0x12,
                y: 0x34,
                carry_in: false,
            },
            expected: AluOutput {
                result: 0x46,
                flags: cpu_flags!(),
            },
        },
        AluTestCase {
            input: AluInput {
                x: 0x0f,
                y: 0x01,
                carry_in: false,
            },
            expected: AluOutput {
                result: 0x10,
                flags: cpu_flags!(h),
            },
        },
        AluTestCase {
            input: AluInput {
                x: 0xf0,
                y: 0xf0,
                carry_in: false,
            },
            expected: AluOutput {
                result: 0xe0,
                flags: cpu_flags!(cy),
            },
        },
        AluTestCase {
            input: AluInput {
                x: 0xf0,
                y: 0x10,
                carry_in: false,
            },
            expected: AluOutput {
                result: 0x00,
                flags: cpu_flags!(z, cy),
            },
        },
        AluTestCase {
            input: AluInput {
                x: 0xff,
                y: 0x00,
                carry_in: true,
            },
            expected: AluOutput {
                result: 0x00,
                flags: cpu_flags!(z, h, cy),
            },
        },
    ];

    #[test]
    fn ret() {
        let mut cpu = Cpu::default();
        cpu.regs.sp = 0x1234;
        cpu.test_opcode(
            &[0xc9],
            &[
                (CpuInput::with_data(None), Some(BusOp::Read(0x1234))),
                (CpuInput::with_data(Some(0x78)), None),
                (CpuInput::with_data(None), Some(BusOp::Read(0x1235))),
                (CpuInput::with_data(Some(0x56)), None),
                // M3 doesn't do any bus operation (according to LIJI32 and gekkio)
                (CpuInput::with_data(None), None),
                (CpuInput::with_data(None), None),
                (CpuInput::with_data(None), Some(BusOp::Read(0x5678))),
                (CpuInput::with_data(Some(0x00)), None),
            ],
        );
        assert_eq!(cpu.regs.sp, 0x1236)
    }

    impl Cpu {
        fn test_opcode<'a>(
            &mut self,
            opcode: &[u8],
            steps: impl IntoIterator<Item = &'a (CpuInput, CpuOutput)>,
        ) {
            let pc = self.regs.pc;
            for (i, byte) in opcode.iter().enumerate() {
                assert_eq!(
                    self.step(&CpuInput::with_data(None)),
                    Some(BusOp::Read(pc + i as u16))
                );
                assert_eq!(self.step(&CpuInput::with_data(Some(*byte))), None);
            }
            for (input, output) in steps {
                assert_eq!(self.step(input), *output)
            }
        }
    }

    impl CpuInput {
        fn with_data(data: Option<u8>) -> Self {
            Self { data }
        }
    }
}
