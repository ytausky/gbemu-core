use super::*;

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
    cpu.test_simple_instr(
        opcode,
        &[
            (Input::with_data(None), Some(BusOp::Read(cpu.regs.hl()))),
            (Input::with_data(Some(test_case.input.y)), None),
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
    cpu.test_simple_instr(&encoder(r), &[]);
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

const ADDITION_TEST_CASES: &[AluTestCase] = &[
    AluTestCase {
        input: AluInput {
            x: 0x08,
            y: 0x08,
            carry_in: false,
        },
        expected: AluOutput {
            result: 0x10,
            flags: flags!(h),
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
            flags: flags!(z, cy),
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
            flags: flags!(),
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
            flags: flags!(h),
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
            flags: flags!(cy),
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
            flags: flags!(z, cy),
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
            flags: flags!(z, h, cy),
        },
    },
];
