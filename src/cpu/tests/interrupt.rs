use super::*;

#[test]
fn dispatch_interrupt_0() {
    let mut cpu = Cpu::default();
    cpu.data.pc = 0x3000;
    cpu.data.sp = 0x2000;
    cpu.data.ie = 0x01;
    cpu.assert_interrupt_dispatch(0x0000)
}

#[test]
fn dispatch_interrupt_1() {
    let mut cpu = Cpu::default();
    cpu.data.pc = 0x3000;
    cpu.data.sp = 0x2000;
    cpu.data.ie = 0x02;
    cpu.assert_interrupt_dispatch(0x0001)
}

#[test]
fn disabled_interrupt_0_does_not_cause_interrupt_dispatch() {
    let mut cpu = Cpu::default();
    cpu.assert_no_interrupt_dispatch(0x01)
}

#[test]
fn enabled_interrupt_not_dispatched_with_reset_ime() {
    let mut cpu = Cpu::default();
    cpu.data.ie = 0x01;
    cpu.data.ime = false;
    cpu.assert_no_interrupt_dispatch(0x01)
}

#[ignore]
#[test]
fn ld_deref_0xffff_sets_5_lower_bits_of_ie() {
    let mut cpu = Cpu::default();
    cpu.data.ie = 0x00;
    cpu.data.a = 0xff;
    cpu.test_simple_instr(
        &[0xe0, 0xff],
        &[
            (Input::with_data(None), Some(BusOp::Write(0xffff, 0xff))),
            (Input::with_data(None), None),
        ],
    );
    assert_eq!(cpu.data.ie, 0x1f)
}

impl Cpu {
    fn assert_interrupt_dispatch(&mut self, n: u16) {
        let pc = self.data.pc;
        let sp = self.data.sp;
        let r#if = 0x01 << n;
        let input = Input { data: None, r#if };
        self.data.ime = true;
        assert_eq!(self.step(&input), Some(BusOp::Read(pc)));
        assert_eq!(
            self.step(&Input {
                data: Some(0x00),
                r#if
            }),
            None
        );
        assert_eq!(self.step(&input), None);
        assert_eq!(self.step(&input), None);
        assert_eq!(self.step(&input), None);
        assert_eq!(self.step(&input), None);
        assert_eq!(self.step(&input), Some(BusOp::Write(sp - 1, high_byte(pc))));
        assert_eq!(self.step(&input), None);
        assert_eq!(self.step(&input), Some(BusOp::Write(sp - 2, low_byte(pc))));
        assert_eq!(self.step(&input), None);
        assert!(!self.data.ime);
        assert_eq!(
            self.step(&Input::with_data(None)),
            Some(BusOp::Read(0x0040 + 8 * n))
        );
        assert_eq!(self.step(&Input::with_data(Some(0x00))), None);
    }

    fn assert_no_interrupt_dispatch(&mut self, r#if: u8) {
        let pc = self.data.pc;
        assert_eq!(
            self.step(&Input { data: None, r#if }),
            Some(BusOp::Read(pc))
        );
        assert_eq!(
            self.step(&Input {
                data: Some(0x00),
                r#if
            }),
            None
        );
        assert_eq!(
            self.step(&Input { data: None, r#if }),
            Some(BusOp::Read(pc + 1))
        );
        assert_eq!(
            self.step(&Input {
                data: Some(0x00),
                r#if
            }),
            None
        );
    }
}
