use super::*;

#[test]
fn jp_nn() {
    let mut cpu = Cpu::default();
    cpu.test_opcode(
        &[0xc3, 0x34, 0x12],
        &[
            (Input::with_data(None), None),
            (Input::with_data(None), None),
            (Input::with_data(None), Some(BusOp::Read(0x1234))),
            (Input::with_data(Some(0x00)), None),
        ],
    )
}

#[ignore]
#[test]
fn jp_nz_nn_branching() {
    let mut cpu = Cpu::default();
    cpu.test_opcode(
        &[0xc2, 0x34, 0x12],
        &[
            (Input::with_data(None), None),
            (Input::with_data(None), None),
            (Input::with_data(None), Some(BusOp::Read(0x1234))),
            (Input::with_data(Some(0x00)), None),
        ],
    )
}

#[ignore]
#[test]
fn jp_nz_nn_non_branching() {
    let mut cpu = Cpu::default();
    cpu.regs.f.z = true;
    cpu.test_simple_instr(&[0xc2, 0x34, 0x12], &[])
}
