use super::*;

mod alu;
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

#[test]
fn ret() {
    let mut cpu = Cpu::default();
    cpu.regs.sp = 0x1234;
    cpu.test_opcode(
        &[0xc9],
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
}

impl Input {
    fn with_data(data: Option<u8>) -> Self {
        Self { data }
    }
}
