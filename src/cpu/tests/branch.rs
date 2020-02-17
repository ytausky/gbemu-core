use super::*;

use std::convert::TryFrom;

#[test]
fn jp_nn_jumps_to_target() {
    let mut bench = TestBench::default();
    bench.assert_branching_branch_jumps_to_target(&Branch {
        kind: BranchKind::Jp,
        condition: None,
        target: 0x1234,
    })
}

#[test]
fn ret_after_jp_nn() {
    let mut bench = TestBench::default();
    bench.trace_branching_branch(&Branch {
        kind: BranchKind::Call,
        condition: None,
        target: 0x1234,
    });
    bench.trace_ret(0x5678);
    assert_eq!(bench.trace, bench.expected)
}

#[test]
fn branching_jp_nz_nn_jumps_to_target() {
    let mut bench = TestBench::default();
    bench.assert_branching_branch_jumps_to_target(&Branch {
        kind: BranchKind::Jp,
        condition: Some(Cc::Nz),
        target: 0x1234,
    })
}

#[test]
fn ret_after_branching_jp_nz_nn() {
    let mut bench = TestBench::default();
    bench.trace_branching_branch(&Branch {
        kind: BranchKind::Call,
        condition: Some(Cc::Nz),
        target: 0x1234,
    });
    bench.trace_ret(0x5678);
    assert_eq!(bench.trace, bench.expected)
}

#[test]
fn non_branching_jp_nz_nn_does_not_jump_to_target() {
    let mut bench = TestBench::default();
    bench.assert_non_branching_branch_does_not_jump_to_target(&Branch {
        kind: BranchKind::Jp,
        condition: Some(Cc::Nz),
        target: 0x1234,
    })
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
    bench.assert_branching_branch_jumps_to_target(&Branch {
        kind: BranchKind::Jp,
        condition: Some(Cc::Z),
        target: 0x1234,
    })
}

#[test]
fn non_branching_jp_z_nn_does_not_jump_to_target() {
    let mut bench = TestBench::default();
    bench.assert_non_branching_branch_does_not_jump_to_target(&Branch {
        kind: BranchKind::Jp,
        condition: Some(Cc::Z),
        target: 0x1234,
    })
}

#[test]
fn branching_jp_nc_nn_jumps_to_target() {
    let mut bench = TestBench::default();
    bench.assert_branching_branch_jumps_to_target(&Branch {
        kind: BranchKind::Jp,
        condition: Some(Cc::Nc),
        target: 0x1234,
    })
}

#[test]
fn non_branching_jp_nc_nn_does_not_jump_to_target() {
    let mut bench = TestBench::default();
    bench.assert_non_branching_branch_does_not_jump_to_target(&Branch {
        kind: BranchKind::Jp,
        condition: Some(Cc::Nc),
        target: 0x1234,
    })
}

#[test]
fn branching_jp_c_nn_jumps_to_target() {
    let mut bench = TestBench::default();
    bench.assert_branching_branch_jumps_to_target(&Branch {
        kind: BranchKind::Jp,
        condition: Some(Cc::C),
        target: 0x1234,
    })
}

#[test]
fn non_branching_jp_c_nn_does_not_jump_to_target() {
    let mut bench = TestBench::default();
    bench.assert_non_branching_branch_does_not_jump_to_target(&Branch {
        kind: BranchKind::Jp,
        condition: Some(Cc::C),
        target: 0x1234,
    })
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
    bench.assert_branching_branch_jumps_to_target(&Branch {
        kind: BranchKind::Jr,
        condition: Some(Cc::Nz),
        target: 0x0042,
    })
}

#[test]
fn ret_after_branching_jr_nz_e() {
    let mut bench = TestBench::default();
    bench.trace_branching_branch(&Branch {
        kind: BranchKind::Jr,
        condition: Some(Cc::Nz),
        target: 0x0042,
    });
    bench.trace_ret(0x5678);
    assert_eq!(bench.trace, bench.expected)
}

#[test]
fn non_branching_jr_nz_e_does_not_jump_to_target() {
    let mut bench = TestBench::default();
    bench.assert_non_branching_branch_does_not_jump_to_target(&Branch {
        kind: BranchKind::Jr,
        condition: Some(Cc::Nz),
        target: 0x0042,
    })
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
    bench.assert_branching_branch_jumps_to_target(&Branch {
        kind: BranchKind::Jr,
        condition: Some(Cc::Z),
        target: 0x0042,
    })
}

#[test]
fn non_branching_jr_z_e_does_not_jump_to_target() {
    let mut bench = TestBench::default();
    bench.assert_non_branching_branch_does_not_jump_to_target(&Branch {
        kind: BranchKind::Jr,
        condition: Some(Cc::Z),
        target: 0x0042,
    })
}

#[test]
fn branching_jr_nc_e_jumps_to_target() {
    let mut bench = TestBench::default();
    bench.assert_branching_branch_jumps_to_target(&Branch {
        kind: BranchKind::Jr,
        condition: Some(Cc::Nc),
        target: 0x0042,
    })
}

#[test]
fn non_branching_jr_nc_e_does_not_jump_to_target() {
    let mut bench = TestBench::default();
    bench.assert_non_branching_branch_does_not_jump_to_target(&Branch {
        kind: BranchKind::Jr,
        condition: Some(Cc::Nc),
        target: 0x0042,
    })
}

#[test]
fn branching_jr_c_e_jumps_to_target() {
    let mut bench = TestBench::default();
    bench.assert_branching_branch_jumps_to_target(&Branch {
        kind: BranchKind::Jr,
        condition: Some(Cc::C),
        target: 0x0042,
    })
}

#[test]
fn non_branching_jr_c_e_does_not_jump_to_target() {
    let mut bench = TestBench::default();
    bench.assert_non_branching_branch_does_not_jump_to_target(&Branch {
        kind: BranchKind::Jr,
        condition: Some(Cc::C),
        target: 0x0042,
    })
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
    bench.assert_branching_branch_jumps_to_target(&Branch {
        kind: BranchKind::Call,
        condition: None,
        target: 0x1234,
    })
}

#[test]
fn call_decrements_sp_by_2() {
    let mut bench = TestBench::default();
    let sp = bench.cpu.data.sp;
    bench.trace_branching_branch(&Branch {
        kind: BranchKind::Call,
        condition: None,
        target: 0x1234,
    });
    assert_eq!(bench.cpu.data.sp, sp.wrapping_sub(2))
}

#[test]
fn call_bus_activity() {
    let mut bench = TestBench::default();
    bench.trace_branching_branch(&Branch {
        kind: BranchKind::Call,
        condition: None,
        target: 0x1234,
    });
    assert_eq!(bench.trace, bench.expected)
}

#[test]
fn ret_after_call() {
    let mut bench = TestBench::default();
    bench.trace_branching_branch(&Branch {
        kind: BranchKind::Call,
        condition: None,
        target: 0x1234,
    });
    bench.trace_ret(0x5678);
    assert_eq!(bench.trace, bench.expected)
}

#[test]
fn branching_call_nz_nn_jumps_to_target() {
    let mut bench = TestBench::default();
    bench.assert_branching_branch_jumps_to_target(&Branch {
        kind: BranchKind::Call,
        condition: Some(Cc::Nz),
        target: 0x1234,
    })
}

#[test]
fn non_branching_call_nz_nn_does_not_jump_to_target() {
    let mut bench = TestBench::default();
    bench.assert_non_branching_branch_does_not_jump_to_target(&Branch {
        kind: BranchKind::Call,
        condition: Some(Cc::Nz),
        target: 0x1234,
    })
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

struct Branch {
    kind: BranchKind,
    condition: Option<Cc>,
    target: u16,
}

#[derive(Clone, Copy, PartialEq)]
enum BranchKind {
    Jp,
    Jr,
    Call,
    Ret,
}

impl TestBench {
    fn assert_branching_branch_jumps_to_target(&mut self, branch: &Branch) {
        if let Some(cc) = branch.condition {
            self.set_condition_flag(cc)
        }
        self.trace_branching_branch(branch);
        self.trace_nop();
        assert_eq!(self.cpu.data.pc, branch.target.wrapping_add(1))
    }

    fn assert_non_branching_branch_does_not_jump_to_target(&mut self, branch: &Branch) {
        let pc = self.cpu.data.pc;
        self.set_condition_flag(!branch.condition.unwrap());
        self.trace_non_branching_branch(branch);
        self.trace_nop();
        assert_eq!(
            self.cpu.data.pc,
            pc.wrapping_add(self.encode_branch(branch).len() as u16)
                .wrapping_add(1)
        )
    }

    fn trace_branching_branch(&mut self, branch: &Branch) {
        let encoding = self.encode_branch(branch);
        let next_instruction = self.cpu.data.pc.wrapping_add(encoding.len() as u16);
        let sp = self.cpu.data.sp;
        self.trace_fetch(self.cpu.data.pc, &encoding);
        match branch.kind {
            BranchKind::Jp | BranchKind::Jr => self.trace_bus_no_op(),
            BranchKind::Call => {
                self.trace_bus_no_op();
                self.trace_bus_write(sp.wrapping_sub(1), high_byte(next_instruction));
                self.trace_bus_write(sp.wrapping_sub(2), low_byte(next_instruction));
            }
            BranchKind::Ret => {
                self.trace_bus_read(sp, low_byte(branch.target));
                self.trace_bus_read(sp.wrapping_add(1), high_byte(branch.target));
                self.trace_bus_no_op()
            }
        }
    }

    fn trace_non_branching_branch(&mut self, branch: &Branch) {
        self.trace_fetch(self.cpu.data.pc, &self.encode_branch(branch));
        if branch.kind == BranchKind::Ret {
            self.trace_bus_no_op()
        }
    }

    fn encode_branch(&self, branch: &Branch) -> Vec<u8> {
        match branch.kind {
            BranchKind::Jp => encode_jp(branch.condition, branch.target),
            BranchKind::Jr => encode_jr(
                branch.condition,
                i8::try_from(branch.target.wrapping_sub(self.cpu.data.pc).wrapping_sub(2)).unwrap()
                    as u8,
            ),
            BranchKind::Call => encode_call(branch.condition, branch.target),
            BranchKind::Ret => unimplemented!(),
        }
    }

    fn set_condition_flag(&mut self, cc: Cc) {
        match cc {
            Cc::Nz => self.cpu.data.f.z = false,
            Cc::Z => self.cpu.data.f.z = true,
            Cc::Nc => self.cpu.data.f.cy = false,
            Cc::C => self.cpu.data.f.cy = true,
        }
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

fn encode_call(cc: Option<Cc>, target: u16) -> Vec<u8> {
    vec![
        match cc {
            None => 0xcd,
            Some(Cc::Nz) => 0xc4,
            _ => unimplemented!(),
        },
        low_byte(target),
        high_byte(target),
    ]
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
