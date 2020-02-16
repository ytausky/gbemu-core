use super::*;

#[test]
fn jp_nn_jumps_to_target() {
    let mut bench = TestBench::default();
    bench.assert_branching_jp_jumps_to_target(None)
}

#[test]
fn ret_after_jp_nn() {
    let mut bench = TestBench::default();
    bench.trace_branching_jp(None, 0x1234);
    bench.trace_ret(0x5678);
    assert_eq!(bench.trace, bench.expected)
}

#[test]
fn branching_jp_nz_nn_jumps_to_target() {
    let mut bench = TestBench::default();
    bench.assert_branching_jp_jumps_to_target(Some(Cc::Nz))
}

#[test]
fn ret_after_branching_jp_nz_nn() {
    let mut bench = TestBench::default();
    bench.trace_branching_jp(Some(Cc::Nz), 0x1234);
    bench.trace_ret(0x5678);
    assert_eq!(bench.trace, bench.expected)
}

#[test]
fn non_branching_jp_nz_nn_does_not_jump_to_target() {
    let mut bench = TestBench::default();
    bench.assert_non_branching_jp_does_not_jump_to_target(Cc::Nz)
}

#[test]
fn ret_after_non_branching_jp_nz_nn() {
    let mut bench = TestBench::default();
    bench.set_condition_flag(!Cc::Nz);
    bench.trace_fetch(&encode_jp(Some(Cc::Nz), 0x1234));
    bench.trace_ret(0x5678);
    assert_eq!(bench.trace, bench.expected)
}

#[test]
fn branching_jp_z_nn_jumps_to_target() {
    let mut bench = TestBench::default();
    bench.assert_branching_jp_jumps_to_target(Some(Cc::Z))
}

#[test]
fn non_branching_jp_z_nn_does_not_jump_to_target() {
    let mut bench = TestBench::default();
    bench.assert_non_branching_jp_does_not_jump_to_target(Cc::Z)
}

#[test]
fn branching_jp_nc_nn_jumps_to_target() {
    let mut bench = TestBench::default();
    bench.assert_branching_jp_jumps_to_target(Some(Cc::Nc))
}

#[test]
fn non_branching_jp_nc_nn_does_not_jump_to_target() {
    let mut bench = TestBench::default();
    bench.assert_non_branching_jp_does_not_jump_to_target(Cc::Nc)
}

#[test]
fn branching_jp_c_nn_jumps_to_target() {
    let mut bench = TestBench::default();
    bench.assert_branching_jp_jumps_to_target(Some(Cc::C))
}

#[test]
fn non_branching_jp_c_nn_does_not_jump_to_target() {
    let mut bench = TestBench::default();
    bench.assert_non_branching_jp_does_not_jump_to_target(Cc::C)
}

impl TestBench {
    fn assert_branching_jp_jumps_to_target(&mut self, cc: Option<Cc>) {
        let target = 0x1234;
        self.trace_branching_jp(cc, target);
        assert_eq!(self.cpu.data.pc, target)
    }

    fn trace_branching_jp(&mut self, cc: Option<Cc>, target: u16) {
        if let Some(cc) = cc {
            self.set_condition_flag(cc)
        }
        self.trace_fetch(&encode_jp(cc, target));
        self.trace_bus_no_op()
    }

    fn assert_non_branching_jp_does_not_jump_to_target(&mut self, cc: Cc) {
        let encoding = encode_jp(Some(cc), 0x1234);
        self.set_condition_flag(!cc);
        self.trace_fetch(&encoding);
        assert_eq!(self.cpu.data.pc, encoding.len() as u16)
    }
}

fn encode_jp(cc: Option<Cc>, addr: u16) -> Vec<u8> {
    vec![
        match cc {
            None => 0xc3,
            Some(Cc::Nz) => 0xc2,
            Some(Cc::Z) => 0xca,
            Some(Cc::Nc) => 0xd2,
            Some(Cc::C) => 0xda,
        },
        low_byte(addr),
        high_byte(addr),
    ]
}

#[test]
fn jr_e_min_value() {
    let mut cpu = Cpu::default();
    cpu.data.pc = 0x1000;
    cpu.test_opcode(
        &[0x18, 0x80],
        &[
            (input!(), output!()),
            (input!(), output!()),
            (input!(), output!(bus: bus_read(0x0f82))),
            (input!(data: 0x00), output!()),
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
            (input!(), output!()),
            (input!(), output!()),
            (input!(), output!(bus: bus_read(0x1100))),
            (input!(data: 0x00), output!()),
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
        &[
            (input!(), output!(bus: bus_read(0x1234))),
            (input!(data: 0x00), output!()),
        ],
    );
    assert_eq!(cpu.data.pc, 0x1235)
}

impl TestBench {
    fn set_condition_flag(&mut self, cc: Cc) {
        match cc {
            Cc::Nz => self.cpu.data.f.z = false,
            Cc::Z => self.cpu.data.f.z = true,
            Cc::Nc => self.cpu.data.f.cy = false,
            Cc::C => self.cpu.data.f.cy = true,
        }
    }
}

impl Not for Cc {
    type Output = Self;

    fn not(self) -> Self::Output {
        match self {
            Cc::Nz => Cc::Z,
            Cc::Z => Cc::Nz,
            Cc::Nc => Cc::C,
            Cc::C => Cc::Nc,
        }
    }
}
