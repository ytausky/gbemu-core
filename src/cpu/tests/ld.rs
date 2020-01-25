use super::*;

#[test]
fn ld_a_a() {
    test_ld_r_r(R::A, R::A)
}

#[test]
fn ld_a_b() {
    test_ld_r_r(R::A, R::B)
}

#[test]
fn ld_a_c() {
    test_ld_r_r(R::A, R::C)
}

#[test]
fn ld_a_d() {
    test_ld_r_r(R::A, R::D)
}

#[test]
fn ld_a_e() {
    test_ld_r_r(R::A, R::E)
}

#[test]
fn ld_a_h() {
    test_ld_r_r(R::A, R::H)
}

#[test]
fn ld_a_l() {
    test_ld_r_r(R::A, R::L)
}

#[test]
fn ld_b_a() {
    test_ld_r_r(R::B, R::A)
}

#[test]
fn ld_b_b() {
    test_ld_r_r(R::B, R::B)
}

#[test]
fn ld_b_c() {
    test_ld_r_r(R::B, R::C)
}

#[test]
fn ld_b_d() {
    test_ld_r_r(R::B, R::D)
}

#[test]
fn ld_b_e() {
    test_ld_r_r(R::B, R::E)
}

#[test]
fn ld_b_h() {
    test_ld_r_r(R::B, R::H)
}

#[test]
fn ld_b_l() {
    test_ld_r_r(R::B, R::L)
}

#[test]
fn ld_c_a() {
    test_ld_r_r(R::C, R::A)
}

#[test]
fn ld_c_b() {
    test_ld_r_r(R::C, R::B)
}

#[test]
fn ld_c_c() {
    test_ld_r_r(R::C, R::C)
}

#[test]
fn ld_c_d() {
    test_ld_r_r(R::C, R::D)
}

#[test]
fn ld_c_e() {
    test_ld_r_r(R::C, R::E)
}

#[test]
fn ld_c_h() {
    test_ld_r_r(R::C, R::H)
}

#[test]
fn ld_c_l() {
    test_ld_r_r(R::C, R::L)
}

#[test]
fn ld_d_a() {
    test_ld_r_r(R::D, R::A)
}

#[test]
fn ld_d_b() {
    test_ld_r_r(R::D, R::B)
}

#[test]
fn ld_d_c() {
    test_ld_r_r(R::D, R::C)
}

#[test]
fn ld_d_d() {
    test_ld_r_r(R::D, R::D)
}

#[test]
fn ld_d_e() {
    test_ld_r_r(R::D, R::E)
}

#[test]
fn ld_d_h() {
    test_ld_r_r(R::D, R::H)
}

#[test]
fn ld_d_l() {
    test_ld_r_r(R::D, R::L)
}

#[test]
fn ld_e_a() {
    test_ld_r_r(R::E, R::A)
}

#[test]
fn ld_e_b() {
    test_ld_r_r(R::E, R::B)
}

#[test]
fn ld_e_c() {
    test_ld_r_r(R::E, R::C)
}

#[test]
fn ld_e_d() {
    test_ld_r_r(R::E, R::D)
}

#[test]
fn ld_e_e() {
    test_ld_r_r(R::E, R::E)
}

#[test]
fn ld_e_h() {
    test_ld_r_r(R::E, R::H)
}

#[test]
fn ld_e_l() {
    test_ld_r_r(R::E, R::L)
}

#[test]
fn ld_h_a() {
    test_ld_r_r(R::H, R::A)
}

#[test]
fn ld_h_b() {
    test_ld_r_r(R::H, R::B)
}

#[test]
fn ld_h_c() {
    test_ld_r_r(R::H, R::C)
}

#[test]
fn ld_h_d() {
    test_ld_r_r(R::H, R::D)
}

#[test]
fn ld_h_e() {
    test_ld_r_r(R::H, R::E)
}

#[test]
fn ld_h_h() {
    test_ld_r_r(R::H, R::H)
}

#[test]
fn ld_h_l() {
    test_ld_r_r(R::H, R::L)
}

#[test]
fn ld_l_a() {
    test_ld_r_r(R::L, R::A)
}

#[test]
fn ld_l_b() {
    test_ld_r_r(R::L, R::B)
}

#[test]
fn ld_l_c() {
    test_ld_r_r(R::L, R::C)
}

#[test]
fn ld_l_d() {
    test_ld_r_r(R::L, R::D)
}

#[test]
fn ld_l_e() {
    test_ld_r_r(R::L, R::E)
}

#[test]
fn ld_l_h() {
    test_ld_r_r(R::L, R::H)
}

#[test]
fn ld_l_l() {
    test_ld_r_r(R::L, R::L)
}

fn test_ld_r_r(dest: R, src: R) {
    let mut cpu = Cpu::default();
    let data = 0x42;
    *cpu.regs.select_r_mut(src) = data;
    cpu.test_simple_instr(&encode_ld_r_r(dest, src), &[]);
    assert_eq!(*cpu.regs.select_r(dest), data)
}

fn encode_ld_r_r(dest: R, src: R) -> Vec<u8> {
    vec![0b01_000_000 | (dest.code() << 3) | src.code()]
}

#[test]
fn ld_a_n() {
    test_ld_r_n(R::A)
}

#[test]
fn ld_b_n() {
    test_ld_r_n(R::B)
}

#[test]
fn ld_c_n() {
    test_ld_r_n(R::C)
}

#[test]
fn ld_d_n() {
    test_ld_r_n(R::D)
}

#[test]
fn ld_e_n() {
    test_ld_r_n(R::E)
}

#[test]
fn ld_h_n() {
    test_ld_r_n(R::H)
}

#[test]
fn ld_l_n() {
    test_ld_r_n(R::L)
}

fn test_ld_r_n(r: R) {
    let mut cpu = Cpu::default();
    let n = 0x42;
    cpu.test_simple_instr(&encode_ld_r_n(r, n), &[]);
    assert_eq!(*cpu.regs.select_r(r), n)
}

fn encode_ld_r_n(r: R, n: u8) -> Vec<u8> {
    vec![0b00_000_110 | r.code() << 3, n]
}

#[test]
fn ld_a_deref_hl() {
    test_ld_r_deref_hl(R::A)
}

#[test]
fn ld_b_deref_hl() {
    test_ld_r_deref_hl(R::B)
}

#[test]
fn ld_c_deref_hl() {
    test_ld_r_deref_hl(R::C)
}

#[test]
fn ld_d_deref_hl() {
    test_ld_r_deref_hl(R::D)
}

#[test]
fn ld_e_deref_hl() {
    test_ld_r_deref_hl(R::E)
}

#[test]
fn ld_h_deref_hl() {
    test_ld_r_deref_hl(R::H)
}

#[test]
fn ld_l_deref_hl() {
    test_ld_r_deref_hl(R::L)
}

fn test_ld_r_deref_hl(dest: R) {
    let mut cpu = Cpu::default();
    let data = 0x42;
    cpu.regs.h = 0x12;
    cpu.regs.l = 0x34;
    cpu.test_simple_instr(
        &encode_ld_r_deref_hl(dest),
        &[
            (Input::with_data(None), Some(BusOp::Read(0x1234))),
            (Input::with_data(Some(data)), None),
        ],
    );
    assert_eq!(*cpu.regs.select_r(dest), data)
}

fn encode_ld_r_deref_hl(dest: R) -> Vec<u8> {
    vec![0b01_000_110 | (dest.code() << 3)]
}

#[test]
fn ld_deref_hl_a() {
    test_ld_deref_hl_r(R::A)
}

#[test]
fn ld_deref_hl_b() {
    test_ld_deref_hl_r(R::B)
}

#[test]
fn ld_deref_hl_c() {
    test_ld_deref_hl_r(R::C)
}

#[test]
fn ld_deref_hl_d() {
    test_ld_deref_hl_r(R::D)
}

#[test]
fn ld_deref_hl_e() {
    test_ld_deref_hl_r(R::E)
}

#[test]
fn ld_deref_hl_h() {
    test_ld_deref_hl_r(R::H)
}

#[test]
fn ld_deref_hl_l() {
    test_ld_deref_hl_r(R::L)
}

fn test_ld_deref_hl_r(src: R) {
    let mut cpu = Cpu::default();
    let data = 0x42;
    cpu.regs.h = 0x12;
    cpu.regs.l = 0x34;
    *cpu.regs.select_r_mut(src) = data;
    cpu.test_simple_instr(
        &encode_ld_deref_hl_r(src),
        &[
            (
                Input::with_data(None),
                Some(BusOp::Write(cpu.regs.hl(), data)),
            ),
            (Input::with_data(None), None),
        ],
    );
}

fn encode_ld_deref_hl_r(src: R) -> Vec<u8> {
    vec![0b01_110_000 | src.code()]
}

#[test]
fn ld_deref_hl_n() {
    let mut cpu = Cpu::default();
    cpu.regs.h = 0x12;
    cpu.regs.l = 0x34;
    let n = 0x42;
    cpu.test_simple_instr(
        &encode_ld_deref_hl_n(n),
        &[
            (Input::with_data(None), Some(BusOp::Write(0x1234, n))),
            (Input::with_data(None), None),
        ],
    )
}

fn encode_ld_deref_hl_n(n: u8) -> Vec<u8> {
    vec![0b00_110_110, n]
}

#[test]
fn ld_a_deref_bc() {
    let mut cpu = Cpu::default();
    cpu.regs.b = 0x12;
    cpu.regs.c = 0x34;
    let value = 0x42;
    cpu.test_simple_instr(
        &[0b00_001_010],
        &[
            (Input::with_data(None), Some(BusOp::Read(0x1234))),
            (Input::with_data(Some(value)), None),
        ],
    );
    assert_eq!(cpu.regs.a, value)
}

#[test]
fn ld_a_deref_de() {
    let mut cpu = Cpu::default();
    cpu.regs.d = 0x12;
    cpu.regs.e = 0x34;
    let value = 0x5f;
    cpu.test_simple_instr(
        &[0b00_011_010],
        &[
            (Input::with_data(None), Some(BusOp::Read(0x1234))),
            (Input::with_data(Some(value)), None),
        ],
    );
    assert_eq!(cpu.regs.a, value)
}

#[test]
fn ld_a_deref_c() {
    let mut cpu = Cpu::default();
    cpu.regs.c = 0x95;
    let value = 0x42;
    cpu.test_simple_instr(
        &[0b11_110_010],
        &[
            (Input::with_data(None), Some(BusOp::Read(0xff95))),
            (Input::with_data(Some(value)), None),
        ],
    );
    assert_eq!(cpu.regs.a, value)
}

#[test]
fn ld_deref_c_a() {
    let mut cpu = Cpu::default();
    let value = 0x42;
    cpu.regs.a = value;
    cpu.regs.c = 0x9f;
    cpu.test_simple_instr(
        &[0b11_100_010],
        &[
            (Input::with_data(None), Some(BusOp::Write(0xff9f, value))),
            (Input::with_data(None), None),
        ],
    )
}

#[test]
fn ld_a_deref_n() {
    let mut cpu = Cpu::default();
    let value = 0x42;
    cpu.test_simple_instr(
        &[0b11_110_000, 0x34],
        &[
            (Input::with_data(None), Some(BusOp::Read(0xff34))),
            (Input::with_data(Some(value)), None),
        ],
    );
    assert_eq!(cpu.regs.a, value)
}

#[test]
fn ld_deref_n_a() {
    let mut cpu = Cpu::default();
    let value = 0x42;
    cpu.regs.a = value;
    cpu.test_simple_instr(
        &[0b11_100_000, 0x34],
        &[
            (Input::with_data(None), Some(BusOp::Write(0xff34, value))),
            (Input::with_data(None), None),
        ],
    )
}

#[test]
fn ld_a_deref_nn() {
    let mut cpu = Cpu::default();
    let value = 0x42;
    cpu.test_simple_instr(
        &[0b11_111_010, 0x00, 0x80],
        &[
            (Input::with_data(None), Some(BusOp::Read(0x8000))),
            (Input::with_data(Some(value)), None),
        ],
    );
    assert_eq!(cpu.regs.a, value)
}

#[test]
fn ld_deref_nn_a() {
    let mut cpu = Cpu::default();
    let value = 0x42;
    cpu.regs.a = value;
    cpu.test_simple_instr(
        &[0b11_101_010, 0x00, 0x80],
        &[
            (Input::with_data(None), Some(BusOp::Write(0x8000, value))),
            (Input::with_data(None), None),
        ],
    )
}

#[test]
fn ld_a_deref_hli() {
    let mut cpu = Cpu::default();
    cpu.regs.h = 0x01;
    cpu.regs.l = 0xff;
    let value = 0x56;
    cpu.test_simple_instr(
        &[0b00_101_010],
        &[
            (Input::with_data(None), Some(BusOp::Read(0x01ff))),
            (Input::with_data(Some(value)), None),
        ],
    );
    assert_eq!(cpu.regs.a, value);
    assert_eq!(cpu.regs.hl(), 0x0200)
}

#[test]
fn ld_a_deref_hld() {
    let mut cpu = Cpu::default();
    cpu.regs.h = 0x8a;
    cpu.regs.l = 0x5c;
    let value = 0x3c;
    cpu.test_simple_instr(
        &[0b00_111_010],
        &[
            (Input::with_data(None), Some(BusOp::Read(0x8a5c))),
            (Input::with_data(Some(value)), None),
        ],
    );
    assert_eq!(cpu.regs.a, value);
    assert_eq!(cpu.regs.hl(), 0x8a5b)
}

#[test]
fn ld_deref_bc_a() {
    let mut cpu = Cpu::default();
    cpu.regs.a = 0x3f;
    cpu.regs.b = 0x02;
    cpu.regs.c = 0x05;
    cpu.test_simple_instr(
        &[0b00_000_010],
        &[
            (Input::with_data(None), Some(BusOp::Write(0x0205, 0x3f))),
            (Input::with_data(None), None),
        ],
    )
}

#[test]
fn ld_deref_de_a() {
    let mut cpu = Cpu::default();
    cpu.regs.d = 0x02;
    cpu.regs.e = 0x05;
    cpu.test_simple_instr(
        &[0b00_010_010],
        &[
            (Input::with_data(None), Some(BusOp::Write(0x0205, 0x00))),
            (Input::with_data(None), None),
        ],
    )
}

#[test]
fn ld_deref_hli_a() {
    let mut cpu = Cpu::default();
    cpu.regs.a = 0x56;
    cpu.regs.h = 0xff;
    cpu.regs.l = 0xff;
    cpu.test_simple_instr(
        &[0b00_100_010],
        &[
            (Input::with_data(None), Some(BusOp::Write(0xffff, 0x56))),
            (Input::with_data(None), None),
        ],
    );
    assert_eq!(cpu.regs.hl(), 0x0000)
}

#[test]
fn ld_deref_hld_a() {
    let mut cpu = Cpu::default();
    cpu.regs.a = 0x05;
    cpu.regs.h = 0x40;
    cpu.regs.l = 0x00;
    cpu.test_simple_instr(
        &[0b00_110_010],
        &[
            (Input::with_data(None), Some(BusOp::Write(0x4000, 0x05))),
            (Input::with_data(None), None),
        ],
    );
    assert_eq!(cpu.regs.hl(), 0x3fff)
}

#[test]
fn ld_bc_nn() {
    let mut cpu = Cpu::default();
    cpu.test_simple_instr(&encode_ld_dd_nn(Dd::Bc, 0x3a5b), &[]);
    assert_eq!(cpu.regs.bc(), 0x3a5b)
}

#[test]
fn ld_de_nn() {
    let mut cpu = Cpu::default();
    cpu.test_simple_instr(&encode_ld_dd_nn(Dd::De, 0x3a5b), &[]);
    assert_eq!(cpu.regs.de(), 0x3a5b)
}

#[test]
fn ld_hl_nn() {
    let mut cpu = Cpu::default();
    cpu.test_simple_instr(&encode_ld_dd_nn(Dd::Hl, 0x3a5b), &[]);
    assert_eq!(cpu.regs.hl(), 0x3a5b)
}

#[test]
fn ld_sp_nn() {
    let mut cpu = Cpu::default();
    cpu.test_simple_instr(&encode_ld_dd_nn(Dd::Sp, 0x3a5b), &[]);
    assert_eq!(cpu.regs.sp, 0x3a5b)
}

fn encode_ld_dd_nn(dd: Dd, nn: u16) -> Vec<u8> {
    vec![
        0b00_000_001 | dd.encode() << 4,
        (nn & 0x00ff) as u8,
        (nn >> 8) as u8,
    ]
}

#[test]
fn ld_sp_hl() {
    let mut cpu = Cpu::default();
    cpu.regs.h = 0x12;
    cpu.regs.l = 0x34;
    cpu.test_simple_instr(
        &[0b11_111_001],
        &[
            (Input::with_data(None), None),
            (Input::with_data(None), None),
        ],
    );
    assert_eq!(cpu.regs.sp, 0x1234)
}
