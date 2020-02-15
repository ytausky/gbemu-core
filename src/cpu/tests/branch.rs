use super::*;

#[test]
fn jp_nn() {
    let mut cpu = Cpu::default();
    cpu.test_opcode(
        &[0xc3, 0x34, 0x12],
        &[
            (input!(), None),
            (input!(), None),
            (input!(), bus_read(0x1234)),
            (input!(data: 0x00), None),
        ],
    )
}

#[test]
fn jp_nz_nn_branching() {
    let mut cpu = Cpu::default();
    cpu.test_opcode(
        &[0xc2, 0x34, 0x12],
        &[
            (input!(), None),
            (input!(), None),
            (input!(), bus_read(0x1234)),
            (input!(data: 0x00), None),
        ],
    )
}

#[test]
fn jp_nz_nn_non_branching() {
    let mut cpu = Cpu::default();
    cpu.data.f.z = true;
    cpu.test_simple_instr(&[0xc2, 0x34, 0x12], &[])
}

#[test]
fn jp_z_nn_branching() {
    let mut cpu = Cpu::default();
    cpu.data.f.z = true;
    cpu.test_opcode(
        &[0xca, 0x34, 0x12],
        &[
            (input!(), None),
            (input!(), None),
            (input!(), bus_read(0x1234)),
            (input!(data: 0x00), None),
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
            (input!(), None),
            (input!(), None),
            (input!(), bus_read(0x1234)),
            (input!(data: 0x00), None),
        ],
    )
}

#[test]
fn jp_nc_nn_non_branching() {
    let mut cpu = Cpu::default();
    cpu.data.f.cy = true;
    cpu.test_simple_instr(&[0xd2, 0x34, 0x12], &[])
}

#[test]
fn jp_c_nn_branching() {
    let mut cpu = Cpu::default();
    cpu.data.f.cy = true;
    cpu.test_opcode(
        &[0xda, 0x34, 0x12],
        &[
            (input!(), None),
            (input!(), None),
            (input!(), bus_read(0x1234)),
            (input!(data: 0x00), None),
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
            (input!(), bus_read(0x0003)),
            (input!(data: RET), None),
            (input!(), bus_read(0x0000)),
            (input!(data: 0x34), None),
            (input!(), bus_read(0x0001)),
            (input!(data: 0x12), None),
            (input!(), None),
            (input!(), None),
            (input!(), bus_read(0x1234)),
            (input!(data: 0x00), None),
        ],
    )
}

#[test]
fn jr_e_min_value() {
    let mut cpu = Cpu::default();
    cpu.data.pc = 0x1000;
    cpu.test_opcode(
        &[0x18, 0x80],
        &[
            (input!(), None),
            (input!(), None),
            (input!(), bus_read(0x0f82)),
            (input!(data: 0x00), None),
        ],
    )
}

#[test]
fn jr_e_with_carry() {
    let mut cpu = Cpu::default();
    cpu.data.pc = 0x1080;
    cpu.test_opcode(
        &[0x18, 0x7e],
        &[
            (input!(), None),
            (input!(), None),
            (input!(), bus_read(0x1100)),
            (input!(data: 0x00), None),
        ],
    )
}

#[test]
fn jp_deref_hl() {
    let mut cpu = Cpu::default();
    cpu.data.h = 0x12;
    cpu.data.l = 0x34;
    cpu.test_opcode(
        &[0xe9],
        &[(input!(), bus_read(0x1234)), (input!(data: 0x00), None)],
    );
    assert_eq!(cpu.data.pc, 0x1235)
}
