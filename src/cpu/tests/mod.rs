use super::*;

mod alu;
mod branch;
mod interrupt;
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
    cpu.data.sp = 0x1234;
    cpu.test_opcode(
        &[RET],
        &[
            (Input::with_data(None), bus_read(0x1234)),
            (Input::with_data(Some(0x78)), None),
            (Input::with_data(None), bus_read(0x1235)),
            (Input::with_data(Some(0x56)), None),
            // M3 doesn't do any bus operation (according to LIJI32 and gekkio)
            (Input::with_data(None), None),
            (Input::with_data(None), None),
            (Input::with_data(None), bus_read(0x5678)),
            (Input::with_data(Some(0x00)), None),
        ],
    );
    assert_eq!(cpu.data.sp, 0x1236)
}

#[test]
fn two_rets() {
    let mut cpu = Cpu::default();
    cpu.data.sp = 0x1234;
    cpu.test_opcode(
        &[RET],
        &[
            (Input::with_data(None), bus_read(0x1234)),
            (Input::with_data(Some(0x78)), None),
            (Input::with_data(None), bus_read(0x1235)),
            (Input::with_data(Some(0x56)), None),
            (Input::with_data(None), None),
            (Input::with_data(None), None),
            (Input::with_data(None), bus_read(0x5678)),
            (Input::with_data(Some(RET)), None),
            (Input::with_data(None), bus_read(0x1236)),
            (Input::with_data(Some(0xbc)), None),
            (Input::with_data(None), bus_read(0x1237)),
            (Input::with_data(Some(0x9a)), None),
            (Input::with_data(None), None),
            (Input::with_data(None), None),
            (Input::with_data(None), bus_read(0x9abc)),
            (Input::with_data(Some(0x00)), None),
        ],
    );
    assert_eq!(cpu.data.sp, 0x1238)
}

const RET: u8 = 0xc9;

impl Cpu {
    fn test_simple_instr<'a, I>(&mut self, opcode: &[u8], steps: I)
    where
        I: IntoIterator<Item = &'a (Input, Output)>,
    {
        let steps: Vec<_> = steps
            .into_iter()
            .cloned()
            .chain(vec![
                (
                    Input::with_data(None),
                    bus_read(self.data.pc + opcode.len() as u16),
                ),
                (Input::with_data(Some(0x00)), None),
            ])
            .collect();
        self.test_opcode(opcode, &steps);
    }

    fn test_opcode<'a, I>(&mut self, opcode: &[u8], steps: I)
    where
        I: IntoIterator<Item = &'a (Input, Output)>,
    {
        let pc = self.data.pc;
        for (i, byte) in opcode.iter().enumerate() {
            assert_eq!(self.step(&Input::with_data(None)), bus_read(pc + i as u16));
            assert_eq!(self.step(&Input::with_data(Some(*byte))), None);
        }
        for (input, output) in steps {
            assert_eq!(self.step(input), *output)
        }
    }
}

impl Input {
    fn with_data(data: Option<u8>) -> Self {
        Self { data, r#if: 0x00 }
    }
}
