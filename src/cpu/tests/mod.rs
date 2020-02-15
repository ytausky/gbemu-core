use super::*;

macro_rules! input {
    ($($tokens:tt)*) => {
        {
            #[allow(unused_mut)]
            let mut input = Input { data: None, r#if: 0x00 };
            input_inner!(input, $($tokens)*);
            input
        }
    };
}

macro_rules! input_inner {
    ($input:ident, data: $data:expr, $($tokens:tt)*) => {
        input_inner!($input, data: $data);
        input_inner!($input, $($tokens)*)
    };
    ($input:ident, data: $data:expr) => {
        $input.data = Some($data)
    };
    ($input:ident, if: $if:expr, $($tokens:tt)*) => {
        input_inner!($input, if: $if);
        input_inner!($input, $($tokens)*)
    };
    ($input:ident, if: $if:expr) => {
        $input.r#if = $if
    };
    ($input:ident,) => {};
}

macro_rules! output {
    ($($tokens:tt)*) => {
        {
            #[allow(unused_mut)]
            #[allow(unused_assignments)]
            let mut output = None;
            output_inner!(output, $($tokens)*);
            output
        }
    };
}

macro_rules! output_inner {
    ($output:ident, bus: $bus:expr) => {
        $output = Some($bus);
    };
    ($output:ident,) => {};
}

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
            (input!(), output!(bus: bus_read(0x1234))),
            (input!(data: 0x78), output!()),
            (input!(), output!(bus: bus_read(0x1235))),
            (input!(data: 0x56), output!()),
            // M3 doesn't do any bus operation (according to LIJI32 and gekkio)
            (input!(), output!()),
            (input!(), output!()),
            (input!(), output!(bus: bus_read(0x5678))),
            (input!(data: 0x00), output!()),
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
            (input!(), output!(bus: bus_read(0x1234))),
            (input!(data: 0x78), output!()),
            (input!(), output!(bus: bus_read(0x1235))),
            (input!(data: 0x56), output!()),
            (input!(), output!()),
            (input!(), output!()),
            (input!(), output!(bus: bus_read(0x5678))),
            (input!(data: RET), output!()),
            (input!(), output!(bus: bus_read(0x1236))),
            (input!(data: 0xbc), output!()),
            (input!(), output!(bus: bus_read(0x1237))),
            (input!(data: 0x9a), output!()),
            (input!(), output!()),
            (input!(), output!()),
            (input!(), output!(bus: bus_read(0x9abc))),
            (input!(data: 0x00), output!()),
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
                    input!(),
                    output!(bus: bus_read(self.data.pc + opcode.len() as u16)),
                ),
                (input!(data: 0x00), output!()),
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
            assert_eq!(self.step(&input!()), output!(bus: bus_read(pc + i as u16)));
            assert_eq!(self.step(&input!(data: *byte)), output!());
        }
        for (input, output) in steps {
            assert_eq!(self.step(input), *output)
        }
    }
}
