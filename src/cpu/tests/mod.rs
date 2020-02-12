use super::*;

mod alu;
mod branch;
mod ld;

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

impl Dd {
    fn encode(self) -> u8 {
        match self {
            Dd::Bc => 0b00,
            Dd::De => 0b01,
            Dd::Hl => 0b10,
            Dd::Sp => 0b11,
        }
    }
}

impl Qq {
    fn encode(self) -> u8 {
        match self {
            Qq::Bc => 0b00,
            Qq::De => 0b01,
            Qq::Hl => 0b10,
            Qq::Af => 0b11,
        }
    }
}

#[test]
fn ret() {
    let mut cpu = Cpu::default();
    cpu.regs.sp = 0x1234;
    cpu.test_opcode(
        &[RET],
        &[
            (Input::with_data(None), Some(BusOp::Read(0x1234))),
            (Input::with_data(Some(0x78)), None),
            (Input::with_data(None), Some(BusOp::Read(0x1235))),
            (Input::with_data(Some(0x56)), None),
            // M3 doesn't do any bus operation (according to LIJI32 and gekkio)
            (Input::with_data(None), None),
            (Input::with_data(None), None),
            (Input::with_data(None), Some(BusOp::Read(0x5678))),
            (Input::with_data(Some(0x00)), None),
        ],
    );
    assert_eq!(cpu.regs.sp, 0x1236)
}

#[test]
fn two_rets() {
    let mut cpu = Cpu::default();
    cpu.regs.sp = 0x1234;
    cpu.test_opcode(
        &[RET],
        &[
            (Input::with_data(None), Some(BusOp::Read(0x1234))),
            (Input::with_data(Some(0x78)), None),
            (Input::with_data(None), Some(BusOp::Read(0x1235))),
            (Input::with_data(Some(0x56)), None),
            (Input::with_data(None), None),
            (Input::with_data(None), None),
            (Input::with_data(None), Some(BusOp::Read(0x5678))),
            (Input::with_data(Some(RET)), None),
            (Input::with_data(None), Some(BusOp::Read(0x1236))),
            (Input::with_data(Some(0xbc)), None),
            (Input::with_data(None), Some(BusOp::Read(0x1237))),
            (Input::with_data(Some(0x9a)), None),
            (Input::with_data(None), None),
            (Input::with_data(None), None),
            (Input::with_data(None), Some(BusOp::Read(0x9abc))),
            (Input::with_data(Some(0x00)), None),
        ],
    );
    assert_eq!(cpu.regs.sp, 0x1238)
}

const RET: u8 = 0xc9;

#[test]
fn dispatch_interrupt_0() {
    let mut cpu = Cpu::default();
    cpu.regs.pc = 0x3000;
    cpu.regs.sp = 0x2000;
    cpu.ie = 0x01;
    cpu.test_interrupt_dispatch(0x0000)
}

#[test]
fn disabled_interrupt_0_does_not_cause_interrupt_dispatch() {
    let mut cpu = Cpu::default();
    let r#if = 0x01;
    assert_eq!(
        cpu.step(&Input { data: None, r#if }),
        Some(BusOp::Read(0x0000))
    );
    assert_eq!(
        cpu.step(&Input {
            data: Some(0x00),
            r#if
        }),
        None
    );
    assert_eq!(
        cpu.step(&Input { data: None, r#if }),
        Some(BusOp::Read(0x0001))
    );
    assert_eq!(
        cpu.step(&Input {
            data: Some(0x00),
            r#if
        }),
        None
    );
}

#[test]
fn dispatch_interrupt_1() {
    let mut cpu = Cpu::default();
    cpu.regs.pc = 0x3000;
    cpu.regs.sp = 0x2000;
    cpu.ie = 0x02;
    cpu.test_interrupt_dispatch(0x0001)
}

impl Cpu {
    fn test_simple_instr<'a, I>(&mut self, opcode: &[u8], steps: I)
    where
        I: IntoIterator<Item = &'a (Input, CpuOutput)>,
    {
        let steps: Vec<_> = steps
            .into_iter()
            .cloned()
            .chain(vec![
                (
                    Input::with_data(None),
                    Some(BusOp::Read(self.regs.pc + opcode.len() as u16)),
                ),
                (Input::with_data(Some(0x00)), None),
            ])
            .collect();
        self.test_opcode(opcode, &steps);
    }

    fn test_opcode<'a, I>(&mut self, opcode: &[u8], steps: I)
    where
        I: IntoIterator<Item = &'a (Input, CpuOutput)>,
    {
        let pc = self.regs.pc;
        for (i, byte) in opcode.iter().enumerate() {
            assert_eq!(
                self.step(&Input::with_data(None)),
                Some(BusOp::Read(pc + i as u16))
            );
            assert_eq!(self.step(&Input::with_data(Some(*byte))), None);
        }
        for (input, output) in steps {
            assert_eq!(self.step(input), *output)
        }
    }

    fn test_interrupt_dispatch(&mut self, n: u16) {
        let pc = self.regs.pc;
        let sp = self.regs.sp;
        let r#if = 0x01 << n;
        let input = Input { data: None, r#if };
        assert_eq!(self.step(&input), Some(BusOp::Read(pc)));
        assert_eq!(
            self.step(&Input {
                data: Some(0x00),
                r#if
            }),
            None
        );
        assert_eq!(self.step(&input), None);
        assert_eq!(self.step(&input), None);
        assert_eq!(self.step(&input), None);
        assert_eq!(self.step(&input), None);
        assert_eq!(self.step(&input), Some(BusOp::Write(sp - 1, high_byte(pc))));
        assert_eq!(self.step(&input), None);
        assert_eq!(self.step(&input), Some(BusOp::Write(sp - 2, low_byte(pc))));
        assert_eq!(self.step(&input), None);
        assert_eq!(
            self.step(&Input::with_data(None)),
            Some(BusOp::Read(0x0040 + 8 * n))
        );
        assert_eq!(self.step(&Input::with_data(Some(0x00))), None);
    }
}

impl Input {
    fn with_data(data: Option<u8>) -> Self {
        Self { data, r#if: 0x00 }
    }
}
