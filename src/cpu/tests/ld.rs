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
    cpu.data.write(src, data);
    cpu.test_simple_instr(&encode_ld_r_r(dest, src), &[]);
    assert_eq!(cpu.data.read(dest), data)
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
    assert_eq!(cpu.data.read(r), n)
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
    cpu.data.h = 0x12;
    cpu.data.l = 0x34;
    cpu.test_simple_instr(
        &encode_ld_r_deref_hl(dest),
        &[(input!(), bus_read(0x1234)), (input!(data: data), None)],
    );
    assert_eq!(cpu.data.read(dest), data)
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
    cpu.data.h = 0x12;
    cpu.data.l = 0x34;
    cpu.data.write(src, data);
    cpu.test_simple_instr(
        &encode_ld_deref_hl_r(src),
        &[(input!(), bus_write(cpu.data.hl(), data)), (input!(), None)],
    );
}

fn encode_ld_deref_hl_r(src: R) -> Vec<u8> {
    vec![0b01_110_000 | src.code()]
}

#[test]
fn ld_deref_hl_n() {
    let mut cpu = Cpu::default();
    cpu.data.h = 0x12;
    cpu.data.l = 0x34;
    let n = 0x42;
    cpu.test_simple_instr(
        &encode_ld_deref_hl_n(n),
        &[(input!(), bus_write(0x1234, n)), (input!(), None)],
    )
}

fn encode_ld_deref_hl_n(n: u8) -> Vec<u8> {
    vec![0b00_110_110, n]
}

#[test]
fn ld_a_deref_bc() {
    let mut cpu = Cpu::default();
    cpu.data.b = 0x12;
    cpu.data.c = 0x34;
    let value = 0x42;
    cpu.test_simple_instr(
        &[0b00_001_010],
        &[(input!(), bus_read(0x1234)), (input!(data: value), None)],
    );
    assert_eq!(cpu.data.a, value)
}

#[test]
fn ld_a_deref_de() {
    let mut cpu = Cpu::default();
    cpu.data.d = 0x12;
    cpu.data.e = 0x34;
    let value = 0x5f;
    cpu.test_simple_instr(
        &[0b00_011_010],
        &[(input!(), bus_read(0x1234)), (input!(data: value), None)],
    );
    assert_eq!(cpu.data.a, value)
}

#[test]
fn ld_a_deref_c() {
    let mut cpu = Cpu::default();
    cpu.data.c = 0x95;
    let value = 0x42;
    cpu.test_simple_instr(
        &[0b11_110_010],
        &[(input!(), bus_read(0xff95)), (input!(data: value), None)],
    );
    assert_eq!(cpu.data.a, value)
}

#[test]
fn ld_deref_c_a() {
    let mut cpu = Cpu::default();
    let value = 0x42;
    cpu.data.a = value;
    cpu.data.c = 0x9f;
    cpu.test_simple_instr(
        &[0b11_100_010],
        &[(input!(), bus_write(0xff9f, value)), (input!(), None)],
    )
}

#[test]
fn ld_a_deref_n() {
    let mut cpu = Cpu::default();
    let value = 0x42;
    cpu.test_simple_instr(
        &[0b11_110_000, 0x34],
        &[(input!(), bus_read(0xff34)), (input!(data: value), None)],
    );
    assert_eq!(cpu.data.a, value)
}

#[test]
fn ld_deref_n_a() {
    let mut cpu = Cpu::default();
    let value = 0x42;
    cpu.data.a = value;
    cpu.test_simple_instr(
        &[0b11_100_000, 0x34],
        &[(input!(), bus_write(0xff34, value)), (input!(), None)],
    )
}

#[test]
fn ld_a_deref_nn() {
    let mut cpu = Cpu::default();
    let value = 0x42;
    cpu.test_simple_instr(
        &[0b11_111_010, 0x00, 0x80],
        &[(input!(), bus_read(0x8000)), (input!(data: value), None)],
    );
    assert_eq!(cpu.data.a, value)
}

#[test]
fn ld_deref_nn_a() {
    let mut cpu = Cpu::default();
    let value = 0x42;
    cpu.data.a = value;
    cpu.test_simple_instr(
        &[0b11_101_010, 0x00, 0x80],
        &[(input!(), bus_write(0x8000, value)), (input!(), None)],
    )
}

#[test]
fn ld_a_deref_hli() {
    let mut cpu = Cpu::default();
    cpu.data.h = 0x01;
    cpu.data.l = 0xff;
    let value = 0x56;
    cpu.test_simple_instr(
        &[0b00_101_010],
        &[(input!(), bus_read(0x01ff)), (input!(data: value), None)],
    );
    assert_eq!(cpu.data.a, value);
    assert_eq!(cpu.data.hl(), 0x0200)
}

#[test]
fn ld_a_deref_hld() {
    let mut cpu = Cpu::default();
    cpu.data.h = 0x8a;
    cpu.data.l = 0x5c;
    let value = 0x3c;
    cpu.test_simple_instr(
        &[0b00_111_010],
        &[(input!(), bus_read(0x8a5c)), (input!(data: value), None)],
    );
    assert_eq!(cpu.data.a, value);
    assert_eq!(cpu.data.hl(), 0x8a5b)
}

#[test]
fn ld_deref_bc_a() {
    let mut cpu = Cpu::default();
    cpu.data.a = 0x3f;
    cpu.data.b = 0x02;
    cpu.data.c = 0x05;
    cpu.test_simple_instr(
        &[0b00_000_010],
        &[(input!(), bus_write(0x0205, 0x3f)), (input!(), None)],
    )
}

#[test]
fn ld_deref_de_a() {
    let mut cpu = Cpu::default();
    cpu.data.d = 0x02;
    cpu.data.e = 0x05;
    cpu.test_simple_instr(
        &[0b00_010_010],
        &[(input!(), bus_write(0x0205, 0x00)), (input!(), None)],
    )
}

#[test]
fn ld_deref_hli_a() {
    let mut cpu = Cpu::default();
    cpu.data.a = 0x56;
    cpu.data.h = 0xff;
    cpu.data.l = 0xff;
    cpu.test_simple_instr(
        &[0b00_100_010],
        &[(input!(), bus_write(0xffff, 0x56)), (input!(), None)],
    );
    assert_eq!(cpu.data.hl(), 0x0000)
}

#[test]
fn ld_deref_hld_a() {
    let mut cpu = Cpu::default();
    cpu.data.a = 0x05;
    cpu.data.h = 0x40;
    cpu.data.l = 0x00;
    cpu.test_simple_instr(
        &[0b00_110_010],
        &[(input!(), bus_write(0x4000, 0x05)), (input!(), None)],
    );
    assert_eq!(cpu.data.hl(), 0x3fff)
}

#[test]
fn ld_bc_nn() {
    let mut cpu = Cpu::default();
    cpu.test_simple_instr(&encode_ld_dd_nn(Dd::Bc, 0x3a5b), &[]);
    assert_eq!(cpu.data.bc(), 0x3a5b)
}

#[test]
fn ld_de_nn() {
    let mut cpu = Cpu::default();
    cpu.test_simple_instr(&encode_ld_dd_nn(Dd::De, 0x3a5b), &[]);
    assert_eq!(cpu.data.de(), 0x3a5b)
}

#[test]
fn ld_hl_nn() {
    let mut cpu = Cpu::default();
    cpu.test_simple_instr(&encode_ld_dd_nn(Dd::Hl, 0x3a5b), &[]);
    assert_eq!(cpu.data.hl(), 0x3a5b)
}

#[test]
fn ld_sp_nn() {
    let mut cpu = Cpu::default();
    cpu.test_simple_instr(&encode_ld_dd_nn(Dd::Sp, 0x3a5b), &[]);
    assert_eq!(cpu.data.sp, 0x3a5b)
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
    cpu.data.h = 0x12;
    cpu.data.l = 0x34;
    cpu.test_simple_instr(&[0b11_111_001], &[(input!(), None), (input!(), None)]);
    assert_eq!(cpu.data.sp, 0x1234)
}

#[test]
fn push_bc() {
    test_push_qq(Qq::Bc)
}

#[test]
fn push_de() {
    test_push_qq(Qq::De)
}

#[test]
fn push_hl() {
    test_push_qq(Qq::Hl)
}

#[test]
fn push_af() {
    test_push_qq(Qq::Af)
}

fn test_push_qq(qq: Qq) {
    let mut cpu = Cpu::default();
    cpu.data.write(qq.high(), 0x12);
    cpu.data.write(qq.low(), 0x34);
    cpu.data.sp = 0xfffe;
    cpu.test_simple_instr(
        &encode_push_qq(qq),
        &[
            (input!(), None),
            (input!(), None),
            (input!(), bus_write(0xfffd, cpu.data.read(qq.high()))),
            (input!(), None),
            (input!(), bus_write(0xfffc, cpu.data.read(qq.low()))),
            (input!(), None),
        ],
    );
    assert_eq!(cpu.data.sp, 0xfffc)
}

fn encode_push_qq(qq: Qq) -> Vec<u8> {
    vec![0b11_000_101 | qq.encode() << 4]
}

#[test]
fn pop_bc() {
    test_pop_qq(Qq::Bc)
}

#[test]
fn pop_de() {
    test_pop_qq(Qq::De)
}

#[test]
fn pop_hl() {
    test_pop_qq(Qq::Hl)
}

#[test]
fn pop_af() {
    test_pop_qq(Qq::Af)
}

fn test_pop_qq(qq: Qq) {
    let mut cpu = Cpu::default();
    cpu.data.sp = 0xfffc;
    cpu.test_simple_instr(
        &encode_pop_qq(qq),
        &[
            (input!(), bus_read(0xfffc)),
            (input!(data: 0x50), None),
            (input!(), bus_read(0xfffd)),
            (input!(data: 0x3c), None),
        ],
    );
    assert_eq!(cpu.data.read(qq.high()), 0x3c);
    assert_eq!(cpu.data.read(qq.low()), 0x50);
    assert_eq!(cpu.data.sp, 0xfffe)
}

fn encode_pop_qq(qq: Qq) -> Vec<u8> {
    vec![0b11_000_001 | qq.encode() << 4]
}

#[test]
fn ldhl_sp_e() {
    let mut cpu = Cpu::default();
    cpu.data.sp = 0xfff8;
    cpu.test_simple_instr(
        &encode_ldhl_sp_e(0x02),
        &[(input!(), None), (input!(), None)],
    );
    assert_eq!(cpu.data.hl(), 0xfffa);
    assert_eq!(cpu.data.f, flags!())
}

#[test]
fn ldhl_sp_e_with_negative_e() {
    let mut cpu = Cpu::default();
    cpu.test_simple_instr(&encode_ldhl_sp_e(-1), &[(input!(), None), (input!(), None)]);
    assert_eq!(cpu.data.hl(), 0xffff);
    assert_eq!(cpu.data.f, flags!())
}

#[test]
fn ldhl_sp_e_does_not_set_z() {
    let mut cpu = Cpu::default();
    cpu.data.sp = 0xffff;
    cpu.test_simple_instr(
        &encode_ldhl_sp_e(0x01),
        &[(input!(), None), (input!(), None)],
    );
    assert_eq!(cpu.data.hl(), 0x0000);
    assert_eq!(cpu.data.f, flags!(h, cy))
}

#[test]
fn ldhl_sp_e_sets_h() {
    let mut cpu = Cpu::default();
    cpu.data.sp = 0xffe8;
    cpu.test_simple_instr(
        &encode_ldhl_sp_e(0x09),
        &[(input!(), None), (input!(), None)],
    );
    assert_eq!(cpu.data.hl(), 0xfff1);
    assert_eq!(cpu.data.f, flags!(h))
}

#[test]
fn ldhl_sp_e_sets_cy() {
    let mut cpu = Cpu::default();
    cpu.data.sp = 0xfef1;
    cpu.test_simple_instr(
        &encode_ldhl_sp_e(0x10),
        &[(input!(), None), (input!(), None)],
    );
    assert_eq!(cpu.data.hl(), 0xff01);
    assert_eq!(cpu.data.f, flags!(cy))
}

fn encode_ldhl_sp_e(e: i8) -> Vec<u8> {
    vec![0b11_111_000, e as u8]
}

#[test]
fn ld_deref_nn_sp() {
    let mut cpu = Cpu::default();
    cpu.data.sp = 0xfff8;
    cpu.test_simple_instr(
        &encode_ld_deref_nn_sp(0xc100),
        &[
            (input!(), bus_write(0xc100, low_byte(cpu.data.sp))),
            (input!(), None),
            (input!(), bus_write(0xc101, high_byte(cpu.data.sp))),
            (input!(), None),
        ],
    )
}

fn encode_ld_deref_nn_sp(nn: u16) -> Vec<u8> {
    vec![0b00_001_000, low_byte(nn), high_byte(nn)]
}
