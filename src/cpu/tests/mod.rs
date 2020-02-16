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
            let mut output = Output { bus: None, ack: 0x00 };
            output_inner!(output, $($tokens)*);
            output
        }
    };
}

macro_rules! output_inner {
    ($output:ident, bus: $bus:expr) => {
        $output.bus = Some($bus);
    };
    ($output:ident, ack: $ack:expr) => {
        $output.ack = $ack;
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

const RET: u8 = 0xc9;

#[derive(Default)]
struct TestBench {
    cpu: Cpu,
    trace: CpuTrace,
    expected: CpuTrace,
}

type CpuTrace = Vec<(Input, Output)>;

impl TestBench {
    fn trace_ret(&mut self, addr: u16) {
        let sp = self.cpu.data.sp;
        self.trace_fetch(self.cpu.data.pc, &[RET]);
        self.trace_bus_read(sp, low_byte(addr));
        self.trace_bus_read(sp.wrapping_add(1), high_byte(addr));
        self.trace_bus_no_op()
    }

    fn trace_fetch(&mut self, pc: u16, encoding: &[u8]) {
        for (i, byte) in encoding.iter().enumerate() {
            self.trace_bus_read(pc.wrapping_add(i as u16), *byte)
        }
    }

    fn trace_bus_no_op(&mut self) {
        self.trace_step(input!(), output!());
        self.trace_step(input!(), output!())
    }

    fn trace_bus_read(&mut self, addr: u16, data: u8) {
        self.trace_step(input!(), output!(bus: bus_read(addr)));
        self.trace_step(input!(data: data), output!())
    }

    fn trace_step(&mut self, input: Input, output: Output) {
        self.trace.push((input.clone(), self.cpu.step(&input)));
        self.expected.push((input, output))
    }
}

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
