use self::{MCycle::*, Phase::*};

fn main() {
    println!("Hello, world!");
}

pub struct Cpu {
    regs: Regs,
    state: CpuState,
}

#[derive(Default)]
struct Regs {
    a: u8,
    f: FlagReg,
    b: u8,
    c: u8,
    d: u8,
    e: u8,
    h: u8,
    l: u8,
    pc: u16,
    sp: u16,
}

#[derive(Debug, Default, PartialEq)]
struct FlagReg {
    z: bool,
    n: bool,
    h: bool,
    cy: bool,
}

enum CpuState {
    Running(RunningState),
}

struct RunningState {
    opcode: Opcode,
    m_cycle: MCycle,
    phase: Phase,
}

impl RunningState {
    fn next(&self) -> CpuState {
        let (m_cycle, phase) = match self.phase {
            Tick => (self.m_cycle, Tock),
            Tock => (self.m_cycle.next(), Tick),
        };
        CpuState::Running(RunningState {
            opcode: self.opcode,
            m_cycle,
            phase,
        })
    }
}

#[derive(Clone, Copy)]
struct Opcode(u8);

impl Opcode {
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
            state: CpuState::Running(Default::default()),
        }
    }
}

impl Default for RunningState {
    fn default() -> Self {
        Self {
            opcode: Opcode(0x00),
            m_cycle: MCycle::M1,
            phase: Tick,
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
    pub fn step(&mut self, input: &CpuInput) -> Option<BusOp> {
        let (next_state, output) = match &mut self.state {
            CpuState::Running(state) => RunningCpu {
                regs: &mut self.regs,
                next_state: state.next(),
                state,
                input,
            }
            .exec_instr(),
        };
        self.state = next_state;
        output
    }
}

struct RunningCpu<'a> {
    regs: &'a mut Regs,
    state: &'a mut RunningState,
    next_state: CpuState,
    input: &'a CpuInput,
}

impl<'a> RunningCpu<'a> {
    fn exec_instr(mut self) -> (CpuState, Option<BusOp>) {
        let output = match self.state.opcode.split() {
            (0b00, 0b000, 0b000) => self.nop(),
            (0b01, 0b110, 0b110) => self.halt(),
            (0b01, 0b110, src) => self.ld_deref_hl_r(src.into()),
            (0b01, dest, 0b110) => self.ld_r_deref_hl(dest.into()),
            (0b01, dest, src) => self.ld_r_r(dest.into(), src.into()),
            (0b10, 0b000, src) => self.add(src.into(), false),
            (0b10, 0b001, src) => self.add(src.into(), self.regs.f.cy),
            (0b11, 0b001, 0b001) => self.ret(),
            _ => unimplemented!(),
        };
        (self.next_state, output)
    }

    fn nop(&mut self) -> Option<BusOp> {
        self.fetch()
    }

    fn ld_r_r(&mut self, dest: R, src: R) -> Option<BusOp> {
        match (self.state.m_cycle, self.state.phase) {
            (M1, Tick) => self.fetch(),
            (M1, Tock) => {
                let value = *self.regs.reg(src);
                *self.regs.reg(dest) = value;
                self.fetch()
            }
            _ => unreachable!(),
        }
    }

    fn ld_r_deref_hl(&mut self, dest: R) -> Option<BusOp> {
        match (self.state.m_cycle, self.state.phase) {
            (M1, Tick) => Some(BusOp::Read(self.regs.hl())),
            (M1, Tock) => {
                *self.regs.reg(dest) = self.input.data.unwrap();
                None
            }
            (M2, _) => self.fetch(),
            _ => unreachable!(),
        }
    }

    fn ld_deref_hl_r(&mut self, src: R) -> Option<BusOp> {
        match (self.state.m_cycle, self.state.phase) {
            (M1, Tick) => Some(BusOp::Write(self.regs.hl(), *self.regs.reg(src))),
            (M1, Tock) => None,
            (M2, _) => self.fetch(),
            _ => unreachable!(),
        }
    }

    fn halt(&mut self) -> Option<BusOp> {
        unimplemented!()
    }

    fn add(&mut self, r: R, cy_in: bool) -> Option<BusOp> {
        match (self.state.m_cycle, self.state.phase) {
            (M1, Tick) => self.fetch(),
            (M1, Tock) => {
                let x = self.regs.a;
                let y = *self.regs.reg(r);
                let (partial_sum, overflow1) = x.overflowing_add(y);
                let (sum, overflow2) = partial_sum.overflowing_add(cy_in.into());
                self.regs.a = sum;
                self.regs.f.z = sum == 0;
                self.regs.f.h = (x & 0x0f) + (y & 0x0f) + u8::from(cy_in) > 0x0f;
                self.regs.f.cy = overflow1 | overflow2;
                self.fetch()
            }
            _ => unreachable!(),
        }
    }

    fn ret(&mut self) -> Option<BusOp> {
        match (self.state.m_cycle, self.state.phase) {
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

    fn fetch(&mut self) -> Option<BusOp> {
        match self.state.phase {
            Phase::Tick => Some(BusOp::Read(self.regs.pc)),
            Phase::Tock => {
                self.regs.pc += 1;
                self.next_state = CpuState::Running(RunningState {
                    opcode: Opcode(self.input.data.unwrap()),
                    m_cycle: M1,
                    phase: Tick,
                });
                None
            }
        }
    }
}

pub struct CpuInput {
    data: Option<u8>,
}

#[derive(Clone, Copy)]
enum R {
    A,
    B,
    C,
    D,
    E,
    H,
    L,
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

#[derive(Clone, Copy)]
enum Phase {
    Tick,
    Tock,
}

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
            encode_ld_r_r(dest, src),
            &[
                (CpuInput::with_data(None), Some(BusOp::Read(0x0001))),
                (CpuInput::with_data(Some(0x00)), None),
            ],
        );
        assert_eq!(*cpu.regs.reg(dest), data)
    }

    fn encode_ld_r_r(dest: R, src: R) -> u8 {
        0b01_000_000 | (dest.code() << 3) | src.code()
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
            encode_ld_r_deref_hl(dest),
            &[
                (CpuInput::with_data(None), Some(BusOp::Read(0x1234))),
                (CpuInput::with_data(Some(data)), None),
                (CpuInput::with_data(None), Some(BusOp::Read(0x0001))),
                (CpuInput::with_data(Some(0x00)), None),
            ],
        );
        assert_eq!(*cpu.regs.reg(dest), data)
    }

    fn encode_ld_r_deref_hl(dest: R) -> u8 {
        0b01_000_110 | (dest.code() << 3)
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
            encode_ld_deref_hl_r(src),
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

    fn encode_ld_deref_hl_r(src: R) -> u8 {
        0b01_110_000 | src.code()
    }

    #[test]
    fn add_a_a() {
        let test_cases = &[
            (
                0x08,
                FlagReg {
                    h: true,
                    ..Default::default()
                },
            ),
            (
                0x80,
                FlagReg {
                    z: true,
                    cy: true,
                    ..Default::default()
                },
            ),
        ];
        for (a, flags) in test_cases {
            test_add_a_r(R::A, *a, *a, flags)
        }
    }

    #[test]
    fn add_a_r() {
        let test_cases = &[
            (0x12, 0x34, Default::default()),
            (
                0x0f,
                0x1,
                FlagReg {
                    h: true,
                    ..Default::default()
                },
            ),
            (
                0xf0,
                0xf0,
                FlagReg {
                    cy: true,
                    ..Default::default()
                },
            ),
            (
                0xf0,
                0x10,
                FlagReg {
                    z: true,
                    cy: true,
                    ..Default::default()
                },
            ),
        ];
        for (x, y, flags) in test_cases {
            for &r in &[R::B, R::C, R::D, R::E, R::H, R::L] {
                test_add_a_r(r, *x, *y, flags)
            }
        }
    }

    fn test_add_a_r(r: R, x: u8, y: u8, flags: &FlagReg) {
        let mut cpu = Cpu::default();
        cpu.regs.a = x;
        *cpu.regs.reg(r) = y;
        cpu.test_opcode(
            encode_add_a_r(r),
            &[
                (CpuInput::with_data(None), Some(BusOp::Read(0x0001))),
                (CpuInput::with_data(Some(0x00)), None),
            ],
        );
        assert_eq!(cpu.regs.a, x.wrapping_add(y));
        assert_eq!(cpu.regs.f, *flags)
    }

    fn encode_add_a_r(r: R) -> u8 {
        0b10_000_000 | r.code()
    }

    #[test]
    fn adc_a_r() {
        let test_cases = &[(
            0xff,
            0x00,
            true,
            FlagReg {
                z: true,
                h: true,
                cy: true,
                ..Default::default()
            },
        )];
        for (x, y, cy_in, flags) in test_cases {
            for &r in &[R::B, R::C, R::D, R::E, R::H, R::L] {
                test_adc_a_r(r, *x, *y, *cy_in, flags)
            }
        }
    }

    fn test_adc_a_r(r: R, x: u8, y: u8, cy_in: bool, flags: &FlagReg) {
        let mut cpu = Cpu::default();
        cpu.regs.a = x;
        *cpu.regs.reg(r) = y;
        cpu.regs.f.cy = cy_in;
        cpu.test_opcode(
            encode_adc_a_r(r),
            &[
                (CpuInput::with_data(None), Some(BusOp::Read(0x0001))),
                (CpuInput::with_data(Some(0x00)), None),
            ],
        );
        assert_eq!(cpu.regs.a, x.wrapping_add(y).wrapping_add(cy_in.into()));
        assert_eq!(cpu.regs.f, *flags)
    }

    fn encode_adc_a_r(r: R) -> u8 {
        0b10_001_000 | r.code()
    }

    #[test]
    fn ret() {
        let mut cpu = Cpu::default();
        cpu.regs.sp = 0x1234;
        cpu.test_opcode(
            0xc9,
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
            opcode: u8,
            steps: impl IntoIterator<Item = &'a (CpuInput, Option<BusOp>)>,
        ) {
            assert_eq!(
                self.step(&CpuInput::with_data(None)),
                Some(BusOp::Read(0x0000))
            );
            assert_eq!(self.step(&CpuInput::with_data(Some(opcode))), None);
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
