use super::*;

macro_rules! input {
    () => {
        Input { data: None, r#if: 0x00 }
    };
    ($field:ident $($tokens:tt)*) => {
        {
            let mut input = input!();
            input!(@ input, $field $($tokens)*);
            input
        }
    };
    (@ $input:ident, $field:ident: $value:expr, $($tokens:tt)*) => {
        input!(@ $input, $field: $value);
        input!(@ $input, $($tokens)*)
    };
    (@ $input:ident, data: $data:expr) => {
        $input.data = Some($data)
    };
    (@ $input:ident, if: $if:expr) => {
        $input.r#if = $if
    };
}

macro_rules! output {
    () => {
        Output { bus: None, ack: 0x00 }
    };
    ($field:ident $($tokens:tt)*) => {
        {
            let mut output = output!();
            output!(@ output, $field $($tokens)*);
            output
        }
    };
    (@ $output:ident, bus: $bus:expr) => {
        $output.bus = Some($bus);
    };
    (@ $output:ident, ack: $ack:expr) => {
        $output.ack = $ack;
    };
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

struct TestBench {
    cpu: Cpu,
    r#if: u8,
    trace: CpuTrace,
    expected: CpuTrace,
}

type CpuTrace = Vec<(Input, Output)>;

impl Default for TestBench {
    fn default() -> Self {
        let mut cpu = Cpu::default();
        cpu.data.sp = 0xd000;
        cpu.data.ie = 0x1f;
        cpu.data.ime = true;
        Self {
            cpu,
            r#if: 0x00,
            trace: Default::default(),
            expected: Default::default(),
        }
    }
}

impl TestBench {
    fn trace_nop(&mut self) {
        self.trace_fetch(self.cpu.data.pc, &[NOP])
    }

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
        self.trace_step(None, output!());
        self.trace_step(None, output!())
    }

    fn trace_bus_read(&mut self, addr: u16, data: u8) {
        self.trace_step(None, output!(bus: bus_read(addr)));
        self.trace_step(Some(data), output!())
    }

    fn trace_bus_write(&mut self, addr: u16, data: u8) {
        self.trace_step(None, output!(bus: bus_write(addr, data)));
        self.trace_step(None, output!())
    }

    fn trace_step(&mut self, data: Option<u8>, output: Output) {
        let input = Input {
            data,
            r#if: self.r#if,
        };
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
