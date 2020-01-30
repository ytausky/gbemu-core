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

#[test]
fn jp_nz_nn_non_branching() {
    let mut cpu = Cpu::default();
    cpu.regs.f.z = true;
    cpu.test_simple_instr(&[0xc2, 0x34, 0x12], &[])
}

#[test]
fn jp_z_nn_branching() {
    let mut cpu = Cpu::default();
    cpu.regs.f.z = true;
    cpu.test_opcode(
        &[0xca, 0x34, 0x12],
        &[
            (Input::with_data(None), None),
            (Input::with_data(None), None),
            (Input::with_data(None), Some(BusOp::Read(0x1234))),
            (Input::with_data(Some(0x00)), None),
        ],
    )
}

#[test]
fn jp_z_nn_non_branching() {
    let mut cpu = Cpu::default();
    cpu.test_simple_instr(&[0xca, 0x34, 0x12], &[])
}

#[test]
fn jp_nc_nn_branching() {
    let mut cpu = Cpu::default();
    cpu.test_opcode(
        &[0xd2, 0x34, 0x12],
        &[
            (Input::with_data(None), None),
            (Input::with_data(None), None),
            (Input::with_data(None), Some(BusOp::Read(0x1234))),
            (Input::with_data(Some(0x00)), None),
        ],
    )
}

#[test]
fn jp_nc_nn_non_branching() {
    let mut cpu = Cpu::default();
    cpu.regs.f.cy = true;
    cpu.test_simple_instr(&[0xd2, 0x34, 0x12], &[])
}

#[test]
fn jp_c_nn_branching() {
    let mut cpu = Cpu::default();
    cpu.regs.f.cy = true;
    cpu.test_opcode(
        &[0xda, 0x34, 0x12],
        &[
            (Input::with_data(None), None),
            (Input::with_data(None), None),
            (Input::with_data(None), Some(BusOp::Read(0x1234))),
            (Input::with_data(Some(0x00)), None),
        ],
    )
}

#[test]
fn jp_c_nn_non_branching() {
    let mut cpu = Cpu::default();
    cpu.test_simple_instr(&[0xda, 0x34, 0x12], &[])
}

#[test]
fn jp_c_nn_non_branching_then_ret() {
    let mut cpu = Cpu::default();
    cpu.test_opcode(
        &[0xda, 0x34, 0x12],
        &[
            (Input::with_data(None), Some(BusOp::Read(0x0003))),
            (Input::with_data(Some(RET)), None),
            (Input::with_data(None), Some(BusOp::Read(0x0000))),
            (Input::with_data(Some(0x34)), None),
            (Input::with_data(None), Some(BusOp::Read(0x0001))),
            (Input::with_data(Some(0x12)), None),
            (Input::with_data(None), None),
            (Input::with_data(None), None),
            (Input::with_data(None), Some(BusOp::Read(0x1234))),
            (Input::with_data(Some(0x00)), None),
        ],
    )
}

#[test]
fn jr_deref_hl() {
    let mut cpu = Cpu::default();
    cpu.regs.h = 0x12;
    cpu.regs.l = 0x34;
    cpu.test_opcode(
        &[0xe9],
        &[
            (Input::with_data(None), Some(BusOp::Read(0x1234))),
            (Input::with_data(Some(0x00)), None),
        ],
    );
    assert_eq!(cpu.regs.pc, 0x1235)
}
