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
    bench.trace_fetch(bench.cpu.data.pc, &encode_jp(Some(Cc::Nz), 0x1234));
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
        self.trace_fetch(self.cpu.data.pc, &encode_jp(cc, target));
        self.trace_bus_no_op()
    }

    fn assert_non_branching_jp_does_not_jump_to_target(&mut self, cc: Cc) {
        let encoding = encode_jp(Some(cc), 0x1234);
        self.set_condition_flag(!cc);
        self.trace_fetch(self.cpu.data.pc, &encoding);
        self.trace_nop();
        assert_eq!(self.cpu.data.pc, encoding.len() as u16 + 1)
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
    let mut bench = TestBench::default();
    bench.trace_fetch(bench.cpu.data.pc, &encode_jr(None, 0x80));
    bench.trace_bus_no_op();
    assert_eq!(bench.cpu.data.pc, 0xff82)
}

#[test]
fn jr_e_with_carry() {
    let mut bench = TestBench::default();
    bench.cpu.data.pc = 0x1080;
    bench.trace_fetch(bench.cpu.data.pc, &encode_jr(None, 0x7e));
    bench.trace_bus_no_op();
    assert_eq!(bench.cpu.data.pc, 0x1100)
}

#[test]
fn branching_jr_nz_e_jumps_to_target() {
    let mut bench = TestBench::default();
    bench.assert_branching_jr_jumps_to_target(Some(Cc::Nz))
}

#[test]
fn ret_after_branching_jr_nz_e() {
    let mut bench = TestBench::default();
    bench.trace_branching_jr(Some(Cc::Nz), 0x42);
    bench.trace_ret(0x5678);
    assert_eq!(bench.trace, bench.expected)
}

#[test]
fn non_branching_jr_nz_e_does_not_jump_to_target() {
    let mut bench = TestBench::default();
    bench.assert_non_branching_jr_does_not_jump_to_target(Cc::Nz)
}

#[test]
fn ret_after_non_branching_jr_nz_e() {
    let mut bench = TestBench::default();
    bench.set_condition_flag(!Cc::Nz);
    bench.trace_fetch(bench.cpu.data.pc, &encode_jr(Some(Cc::Nz), 0x42));
    bench.trace_ret(0x5678);
    assert_eq!(bench.trace, bench.expected)
}

#[test]
fn branching_jr_z_e_jumps_to_target() {
    let mut bench = TestBench::default();
    bench.assert_branching_jr_jumps_to_target(Some(Cc::Z))
}

#[test]
fn non_branching_jr_z_e_does_not_jump_to_target() {
    let mut bench = TestBench::default();
    bench.assert_non_branching_jr_does_not_jump_to_target(Cc::Z)
}

#[test]
fn branching_jr_nc_e_jumps_to_target() {
    let mut bench = TestBench::default();
    bench.assert_branching_jr_jumps_to_target(Some(Cc::Nc))
}

#[test]
fn non_branching_jr_nc_e_does_not_jump_to_target() {
    let mut bench = TestBench::default();
    bench.assert_non_branching_jr_does_not_jump_to_target(Cc::Nc)
}

#[test]
fn branching_jr_c_e_jumps_to_target() {
    let mut bench = TestBench::default();
    bench.assert_branching_jr_jumps_to_target(Some(Cc::C))
}

#[test]
fn non_branching_jr_c_e_does_not_jump_to_target() {
    let mut bench = TestBench::default();
    bench.assert_non_branching_jr_does_not_jump_to_target(Cc::C)
}

impl TestBench {
    fn assert_branching_jr_jumps_to_target(&mut self, cc: Option<Cc>) {
        let e = 0x42;
        let offset = e as i8 as i16 as u16;
        let expected_pc = self.cpu.data.pc.wrapping_add(offset).wrapping_add(2);
        self.trace_branching_jr(cc, e);
        assert_eq!(self.cpu.data.pc, expected_pc)
    }

    fn trace_branching_jr(&mut self, cc: Option<Cc>, e: u8) {
        if let Some(cc) = cc {
            self.set_condition_flag(cc)
        }
        self.trace_fetch(self.cpu.data.pc, &encode_jr(cc, e));
        self.trace_bus_no_op()
    }

    fn assert_non_branching_jr_does_not_jump_to_target(&mut self, cc: Cc) {
        let encoding = encode_jr(Some(cc), 0x42);
        self.set_condition_flag(!cc);
        self.trace_fetch(self.cpu.data.pc, &encoding);
        self.trace_nop();
        assert_eq!(self.cpu.data.pc, encoding.len() as u16 + 1)
    }
}

fn encode_jr(cc: Option<Cc>, e: u8) -> Vec<u8> {
    vec![
        match cc {
            None => 0x18,
            Some(Cc::Nz) => 0x20,
            Some(Cc::Z) => 0x28,
            Some(Cc::Nc) => 0x30,
            Some(Cc::C) => 0x38,
        },
        e,
    ]
}

#[test]
fn jp_deref_hl_sets_pc_to_hl() {
    let mut bench = TestBench::default();
    let target = 0x1234;
    bench.cpu.data.h = high_byte(target);
    bench.cpu.data.l = low_byte(target);
    bench.trace_fetch(bench.cpu.data.pc, &[0xe9]);

    // PC is set in the last M-cycle of the intruction, so we need to fetch the next instruction to
    // observe the change in PC.
    bench.trace_fetch(target, &[NOP]);
    assert_eq!(bench.trace, bench.expected)
}

#[test]
fn call_jumps_to_target() {
    let mut bench = TestBench::default();
    let target = 0x1234;
    bench.trace_call(target);
    assert_eq!(bench.cpu.data.pc, target)
}

#[test]
fn call_decrements_sp_by_2() {
    let mut bench = TestBench::default();
    let sp = bench.cpu.data.sp;
    bench.trace_call(0x1234);
    assert_eq!(bench.cpu.data.sp, sp.wrapping_sub(2))
}

#[test]
fn call_bus_activity() {
    let mut bench = TestBench::default();
    bench.trace_call(0x1234);
    assert_eq!(bench.trace, bench.expected)
}

#[test]
fn ret_after_call() {
    let mut bench = TestBench::default();
    bench.trace_call(0x1234);
    bench.trace_ret(0x5678);
    assert_eq!(bench.trace, bench.expected)
}

impl TestBench {
    fn trace_call(&mut self, target: u16) {
        let sp = self.cpu.data.sp;
        self.trace_fetch(self.cpu.data.pc, &encode_call(target));
        self.trace_bus_no_op();
        self.trace_bus_write(sp.wrapping_sub(1), high_byte(self.cpu.data.pc));
        self.trace_bus_write(sp.wrapping_sub(2), low_byte(self.cpu.data.pc));
    }
}

fn encode_call(target: u16) -> Vec<u8> {
    vec![0xcd, low_byte(target), high_byte(target)]
}

#[test]
fn ret_jumps_to_target() {
    let mut bench = TestBench::default();
    let target = 0x5678;
    bench.trace_ret(target);
    assert_eq!(bench.cpu.data.pc, target)
}

#[test]
fn ret_increments_sp_by_2() {
    let mut bench = TestBench::default();
    let sp = bench.cpu.data.sp;
    bench.trace_ret(0x5678);
    assert_eq!(bench.cpu.data.sp, sp.wrapping_add(2))
}

#[test]
fn ret_after_ret() {
    let mut bench = TestBench::default();
    bench.trace_ret(0x5678);
    bench.trace_ret(0x9abc);
    assert_eq!(bench.trace, bench.expected)
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
