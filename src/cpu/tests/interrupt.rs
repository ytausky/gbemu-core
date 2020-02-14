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

#[test]
fn reading_0xffff_returns_ie() {
    let mut cpu = Cpu::default();
    cpu.data.ie = 0x15;
    cpu.test_simple_instr(
        &[0xf0, 0xff],
        &[
            (Input::with_data(None), bus_read(0xffff)),
            (Input::with_data(None), None),
        ],
    );
    assert_eq!(cpu.data.a, 0x15)
}

#[test]
fn read_memory_in_same_instruction_after_reading_0xffff() {
    let mut cpu = Cpu::default();
    cpu.data.sp = 0xffff;
    cpu.data.ie = 0x15;
    const POP_BC: u8 = 0xc1;
    cpu.test_simple_instr(
        &[POP_BC],
        &[
            (Input::with_data(None), bus_read(0xffff)),
            (Input::with_data(None), None),
            (Input::with_data(None), bus_read(0x0000)),
            (Input::with_data(Some(0x42)), None),
        ],
    );
    assert_eq!(cpu.data.bc(), 0x4215)
}

#[test]
fn writing_0xffff_sets_5_lower_bits_of_ie() {
    let mut cpu = Cpu::default();
    cpu.data.ie = 0x00;
    cpu.data.a = 0xff;
    cpu.test_simple_instr(
        &[0xe0, 0xff],
        &[
            (Input::with_data(None), bus_write(0xffff, 0xff)),
            (Input::with_data(None), None),
        ],
    );
    assert_eq!(cpu.data.ie, 0x1f)
}

#[test]
fn writing_0xffff_during_interrupt_dispatch_updates_ie() {
    let mut cpu = Cpu::default();
    cpu.data.pc = 0xff00;
    cpu.data.sp = 0x0000;
    cpu.data.ie = 0x01;
    cpu.assert_interrupt_dispatch(0);
    assert_eq!(cpu.data.ie, 0x1f)
}

impl Cpu {
    fn assert_interrupt_dispatch(&mut self, n: u16) {
        let pc = self.data.pc;
        let sp = self.data.sp;
        let r#if = 0x01 << n;
        let input = Input { data: None, r#if };
        self.data.ime = true;
        assert_eq!(self.step(&input), bus_read(pc));
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
        assert_eq!(
            self.step(&input),
            bus_write(sp.wrapping_sub(1), high_byte(pc))
        );
        assert_eq!(self.step(&input), None);
        assert_eq!(
            self.step(&input),
            bus_write(sp.wrapping_sub(2), low_byte(pc))
        );
        assert_eq!(self.step(&input), None);
        assert!(!self.data.ime);
        assert_eq!(self.step(&Input::with_data(None)), bus_read(0x0040 + 8 * n));
        assert_eq!(self.step(&Input::with_data(Some(0x00))), None);
    }

    fn assert_no_interrupt_dispatch(&mut self, r#if: u8) {
        let pc = self.data.pc;
        assert_eq!(self.step(&Input { data: None, r#if }), bus_read(pc));
        assert_eq!(
            self.step(&Input {
                data: Some(0x00),
                r#if
            }),
            None
        );
        assert_eq!(self.step(&Input { data: None, r#if }), bus_read(pc + 1));
        assert_eq!(
            self.step(&Input {
                data: Some(0x00),
                r#if
            }),
            None
        );
    }
}
