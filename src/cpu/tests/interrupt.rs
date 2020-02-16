use super::*;

#[test]
fn dispatch_interrupt_0() {
    let mut cpu = Cpu::default();
    cpu.data.pc = 0x3000;
    cpu.data.sp = 0x2000;
    cpu.data.ie = 0x01;
    cpu.assert_fetch_and_interrupt_dispatch(0x01, 0)
}

#[test]
fn dispatch_interrupt_1() {
    let mut cpu = Cpu::default();
    cpu.data.pc = 0x3000;
    cpu.data.sp = 0x2000;
    cpu.data.ie = 0x02;
    cpu.assert_fetch_and_interrupt_dispatch(0x02, 1)
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
fn ie_is_checked_when_choosing_interrupt_vector() {
    let mut cpu = Cpu::default();
    cpu.data.sp = 0x8000;
    cpu.data.ie = 0x02;
    cpu.assert_fetch_and_interrupt_dispatch(0x03, 1)
}

#[test]
fn halt_mode_canceled_and_interrupt_dispatched() {
    let mut cpu = Cpu::default();
    cpu.data.sp = 0x8000;
    cpu.data.ie = 0x01;
    cpu.data.ime = true;

    // Fetch HALT opcode
    assert_eq!(cpu.step(&input!()), output!(bus: bus_read(0x0000)));
    assert_eq!(cpu.step(&input!(data: HALT)), output!());

    // Execute HALT
    assert_eq!(cpu.step(&input!()), output!());
    assert_eq!(cpu.step(&input!()), output!());

    // Wait one M-cycle to avoid testing behavior immediately following HALT execution
    assert_eq!(cpu.step(&input!()), output!());
    assert_eq!(cpu.step(&input!()), output!());

    // Request interrupt
    assert_eq!(cpu.step(&input!(if: 0x01)), output!());
    assert_eq!(cpu.step(&input!(if: 0x01)), output!());

    cpu.assert_interrupt_dispatch(0x01, 0);
}

const HALT: u8 = 0x76;

#[test]
fn reading_0xffff_returns_ie() {
    let mut cpu = Cpu::default();
    cpu.data.ie = 0x15;
    cpu.test_simple_instr(
        &[0xf0, 0xff],
        &[
            (input!(), output!(bus: bus_read(0xffff))),
            (input!(), output!()),
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
            (input!(), output!(bus: bus_read(0xffff))),
            (input!(data: 0xff), output!()),
            (input!(), output!(bus: bus_read(0x0000))),
            (input!(data: 0x42), output!()),
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
            (input!(), output!(bus: bus_write(0xffff, 0xff))),
            (input!(), output!()),
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
    cpu.assert_fetch_and_interrupt_dispatch(0x01, 0);
    assert_eq!(cpu.data.ie, 0x1f)
}

impl Cpu {
    fn assert_fetch_and_interrupt_dispatch(&mut self, r#if: u8, n: u16) {
        self.data.ime = true;
        assert_eq!(
            self.step(&input!(if: r#if)),
            output!(bus: bus_read(self.data.pc))
        );
        assert_eq!(self.step(&input!(data: NOP, if: r#if)), output!());
        self.assert_interrupt_dispatch(r#if, n)
    }

    fn assert_interrupt_dispatch(&mut self, r#if: u8, n: u16) {
        let pc = self.data.pc;
        let sp = self.data.sp;
        assert_eq!(self.step(&input!(if: r#if)), output!());
        assert_eq!(self.step(&input!(if: r#if)), output!());
        assert_eq!(self.step(&input!(if: r#if)), output!());
        assert_eq!(self.step(&input!(if: r#if)), output!());
        assert_eq!(
            self.step(&input!(if: r#if)),
            output!(bus: bus_write(sp.wrapping_sub(1), high_byte(pc)))
        );
        assert_eq!(self.step(&input!(if: r#if)), output!());
        assert_eq!(
            self.step(&input!(if: r#if)),
            output!(bus: bus_write(sp.wrapping_sub(2), low_byte(pc)))
        );
        assert_eq!(self.step(&input!(if: r#if)), output!(ack: 1 << n));
        assert!(!self.data.ime);
        assert_eq!(self.step(&input!()), output!(bus: bus_read(0x0040 + 8 * n)));
        assert_eq!(self.step(&input!(data: 0x00)), output!());
    }

    fn assert_no_interrupt_dispatch(&mut self, r#if: u8) {
        let pc = self.data.pc;
        assert_eq!(self.step(&input!(if: r#if)), output!(bus: bus_read(pc)));
        assert_eq!(self.step(&input!(data: 0x00, if: r#if)), output!());
        assert_eq!(self.step(&input!(if: r#if)), output!(bus: bus_read(pc + 1)));
        assert_eq!(self.step(&input!(data: 0x00, if: r#if)), output!());
    }
}
