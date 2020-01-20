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

struct AluInput {
    x: u8,
    y: u8,
    carry_in: bool,
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

#[test]
fn sub_a() {
    let mut cpu = Cpu::default();
    cpu.regs.a = 0x07;
    cpu.test_simple_instr(&encode_sub_r(R::A), &[]);
    assert_eq!(cpu.regs.a, 0);
    assert_eq!(cpu.regs.f, flags!(z, n))
}

#[test]
fn sub_b() {
    let mut cpu = Cpu::default();
    cpu.regs.b = 0x01;
    cpu.test_simple_instr(&encode_sub_r(R::B), &[]);
    assert_eq!(cpu.regs.a, 0xff);
    assert_eq!(cpu.regs.f, flags!(n, h, cy))
}

#[test]
fn sub_c() {
    let mut cpu = Cpu::default();
    cpu.regs.c = 0x10;
    cpu.test_simple_instr(&encode_sub_r(R::C), &[]);
    assert_eq!(cpu.regs.a, 0xf0);
    assert_eq!(cpu.regs.f, flags!(n, cy))
}

#[test]
fn sub_d() {
    let mut cpu = Cpu::default();
    cpu.regs.a = 0x10;
    cpu.regs.d = 0x01;
    cpu.test_simple_instr(&encode_sub_r(R::D), &[]);
    assert_eq!(cpu.regs.a, 0x0f);
    assert_eq!(cpu.regs.f, flags!(n, h))
}

fn encode_sub_r(r: R) -> Vec<u8> {
    vec![0b10_010_000 | r.code()]
}

#[test]
fn sub_n() {
    let mut cpu = Cpu::default();
    cpu.regs.a = 0x07;
    cpu.test_simple_instr(&[0b11_010_110, 0x05], &[]);
    assert_eq!(cpu.regs.a, 0x02);
    assert_eq!(cpu.regs.f, flags!(n))
}

#[test]
fn sbc_a() {
    let mut cpu = Cpu::default();
    cpu.regs.a = 0x07;
    cpu.test_simple_instr(&encode_sbc_r(R::A), &[]);
    assert_eq!(cpu.regs.a, 0);
    assert_eq!(cpu.regs.f, flags!(z, n))
}

#[test]
fn sbc_b() {
    let mut cpu = Cpu::default();
    cpu.regs.a = 0x07;
    cpu.regs.b = 0x07;
    cpu.regs.f.cy = true;
    cpu.test_simple_instr(&encode_sbc_r(R::B), &[]);
    assert_eq!(cpu.regs.a, 0xff);
    assert_eq!(cpu.regs.f, flags!(n, h, cy))
}

fn encode_sbc_r(r: R) -> Vec<u8> {
    vec![0b10_011_000 | r.code()]
}

#[test]
fn and_a() {
    let mut cpu = Cpu::default();
    cpu.regs.a = 0x42;
    cpu.test_simple_instr(&encode_and_r(R::A), &[]);
    assert_eq!(cpu.regs.a, 0x42);
    assert_eq!(cpu.regs.f, flags!(h))
}

#[test]
fn and_b() {
    let mut cpu = Cpu::default();
    cpu.regs.a = 0x0f;
    cpu.regs.b = 0x55;
    cpu.test_simple_instr(&encode_and_r(R::B), &[]);
    assert_eq!(cpu.regs.a, 0x05);
    assert_eq!(cpu.regs.f, flags!(h))
}

#[test]
fn and_c() {
    let mut cpu = Cpu::default();
    cpu.regs.a = 0x0f;
    cpu.regs.b = 0xf0;
    cpu.test_simple_instr(&encode_and_r(R::C), &[]);
    assert_eq!(cpu.regs.a, 0x00);
    assert_eq!(cpu.regs.f, flags!(z, h))
}

fn encode_and_r(r: R) -> Vec<u8> {
    vec![0b10_100_000 | r.code()]
}

#[test]
fn xor_a() {
    let mut cpu = Cpu::default();
    cpu.regs.a = 0x42;
    cpu.test_simple_instr(&encode_xor_r(R::A), &[]);
    assert_eq!(cpu.regs.a, 0x00);
    assert_eq!(cpu.regs.f, flags!(z))
}

#[test]
fn xor_b() {
    let mut cpu = Cpu::default();
    cpu.regs.a = 0x55;
    cpu.regs.b = 0xaa;
    cpu.test_simple_instr(&encode_xor_r(R::B), &[]);
    assert_eq!(cpu.regs.a, 0xff);
    assert_eq!(cpu.regs.f, flags!())
}

fn encode_xor_r(r: R) -> Vec<u8> {
    vec![0b10_101_000 | r.code()]
}

#[test]
fn or_a() {
    let mut cpu = Cpu::default();
    cpu.regs.a = 0x55;
    cpu.test_simple_instr(&encode_or_r(R::A), &[]);
    assert_eq!(cpu.regs.a, 0x55);
    assert_eq!(cpu.regs.f, flags!())
}

#[test]
fn or_b() {
    let mut cpu = Cpu::default();
    cpu.regs.a = 0x05;
    cpu.regs.b = 0x55;
    cpu.test_simple_instr(&encode_or_r(R::B), &[]);
    assert_eq!(cpu.regs.a, 0x55);
    assert_eq!(cpu.regs.f, flags!())
}

#[test]
fn or_c() {
    let mut cpu = Cpu::default();
    cpu.regs.a = 0x05;
    cpu.regs.c = 0x54;
    cpu.test_simple_instr(&encode_or_r(R::C), &[]);
    assert_eq!(cpu.regs.a, 0x55);
    assert_eq!(cpu.regs.f, flags!())
}

#[test]
fn or_d() {
    let mut cpu = Cpu::default();
    cpu.test_simple_instr(&encode_or_r(R::D), &[]);
    assert_eq!(cpu.regs.a, 0x00);
    assert_eq!(cpu.regs.f, flags!(z))
}

fn encode_or_r(r: R) -> Vec<u8> {
    vec![0b10_110_000 | r.code()]
}

#[test]
fn cp_a() {
    let mut cpu = Cpu::default();
    cpu.regs.a = 0x07;
    cpu.test_simple_instr(&encode_cp_r(R::A), &[]);
    assert_eq!(cpu.regs.a, 0x07);
    assert_eq!(cpu.regs.f, flags!(z, n))
}

#[test]
fn cp_b() {
    let mut cpu = Cpu::default();
    cpu.regs.b = 0x01;
    cpu.test_simple_instr(&encode_cp_r(R::B), &[]);
    assert_eq!(cpu.regs.a, 0x00);
    assert_eq!(cpu.regs.f, flags!(n, h, cy))
}

#[test]
fn cp_c() {
    let mut cpu = Cpu::default();
    cpu.regs.c = 0x10;
    cpu.test_simple_instr(&encode_cp_r(R::C), &[]);
    assert_eq!(cpu.regs.a, 0x00);
    assert_eq!(cpu.regs.f, flags!(n, cy))
}

#[test]
fn cp_d() {
    let mut cpu = Cpu::default();
    cpu.regs.a = 0x10;
    cpu.regs.d = 0x01;
    cpu.test_simple_instr(&encode_cp_r(R::D), &[]);
    assert_eq!(cpu.regs.a, 0x10);
    assert_eq!(cpu.regs.f, flags!(n, h))
}

fn encode_cp_r(r: R) -> Vec<u8> {
    vec![0b10_111_000 | r.code()]
}
