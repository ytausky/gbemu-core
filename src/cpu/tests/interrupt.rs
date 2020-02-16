use super::*;

#[test]
fn dispatch_interrupt_0() {
    let mut bench = TestBench::default();
    bench.cpu.data.sp = 0x2000;
    bench.cpu.data.ie = 0x01;
    bench.cpu.data.ime = true;
    bench.r#if = 0x01;
    bench.trace_fetch(bench.cpu.data.pc, &[NOP]);
    bench.trace_interrupt_dispatch(0x01);
    assert_eq!(bench.trace, bench.expected)
}

#[test]
fn interrupt_0_dispatch_jumps_to_0x0040() {
    let mut bench = TestBench::default();
    bench.cpu.data.sp = 0x2000;
    bench.cpu.data.ie = 0x01;
    bench.cpu.data.ime = true;
    bench.r#if = 0x01;
    bench.trace_fetch(bench.cpu.data.pc, &[NOP]);
    bench.trace_interrupt_dispatch(0x01);
    assert_eq!(bench.cpu.data.pc, 0x0040)
}

#[test]
fn dispatch_interrupt_1() {
    let mut bench = TestBench::default();
    bench.cpu.data.sp = 0x2000;
    bench.cpu.data.ie = 0x02;
    bench.cpu.data.ime = true;
    bench.r#if = 0x02;
    bench.trace_fetch(bench.cpu.data.pc, &[NOP]);
    bench.trace_interrupt_dispatch(0x02);
    assert_eq!(bench.trace, bench.expected)
}

#[test]
fn interrupt_1_dispatch_jumps_to_0x0048() {
    let mut bench = TestBench::default();
    bench.cpu.data.sp = 0x2000;
    bench.cpu.data.ie = 0x02;
    bench.cpu.data.ime = true;
    bench.r#if = 0x02;
    bench.trace_fetch(bench.cpu.data.pc, &[NOP]);
    bench.trace_interrupt_dispatch(0x02);
    assert_eq!(bench.cpu.data.pc, 0x0048)
}

#[test]
fn disabled_interrupt_0_does_not_cause_interrupt_dispatch() {
    let mut bench = TestBench::default();
    bench.cpu.data.ie = 0x00;
    bench.cpu.data.ime = true;
    bench.r#if = 0x01;
    bench.trace_fetch(bench.cpu.data.pc, &[NOP]);
    bench.trace_fetch(bench.cpu.data.pc, &[NOP]);
    assert_eq!(bench.trace, bench.expected)
}

#[test]
fn enabled_interrupt_not_dispatched_with_reset_ime() {
    let mut bench = TestBench::default();
    bench.cpu.data.ie = 0x01;
    bench.cpu.data.ime = false;
    bench.r#if = 0x01;
    bench.trace_fetch(bench.cpu.data.pc, &[NOP]);
    bench.trace_fetch(bench.cpu.data.pc, &[NOP]);
    assert_eq!(bench.trace, bench.expected)
}

#[test]
fn ie_is_checked_when_choosing_interrupt_vector() {
    let mut bench = TestBench::default();
    bench.cpu.data.sp = 0x8000;
    bench.cpu.data.ie = 0x02;
    bench.cpu.data.ime = true;
    bench.r#if = 0x03;
    bench.trace_fetch(bench.cpu.data.pc, &[NOP]);
    bench.trace_interrupt_dispatch(0x02);
    assert_eq!(bench.trace, bench.expected)
}

#[test]
fn halt_mode_canceled_and_interrupt_dispatched() {
    let mut bench = TestBench::default();
    bench.cpu.data.sp = 0x8000;
    bench.cpu.data.ie = 0x01;
    bench.cpu.data.ime = true;

    // Fetch HALT opcode
    bench.trace_fetch(bench.cpu.data.pc, &[HALT]);

    // Execute HALT
    bench.trace_bus_no_op();

    // Wait one M-cycle to avoid testing behavior immediately following HALT execution
    bench.trace_bus_no_op();

    // Request interrupt
    bench.r#if = 0x01;
    bench.trace_bus_no_op();

    bench.trace_interrupt_dispatch(0x01);
    assert_eq!(bench.trace, bench.expected)
}

const HALT: u8 = 0x76;

#[test]
fn reading_0xffff_returns_ie() {
    let mut bench = TestBench::default();
    bench.cpu.data.ie = 0x15;
    const LD_A_DEREF_N: u8 = 0xf0;
    let src = 0xffff;
    bench.trace_fetch(bench.cpu.data.pc, &[LD_A_DEREF_N, low_byte(src)]);

    // Nothing else in the system reacts to this read
    bench.trace_step(None, output!(bus: bus_read(src)));
    bench.trace_step(None, output!());

    bench.trace_fetch(bench.cpu.data.pc, &[NOP]);
    assert_eq!(bench.cpu.data.a, 0x15)
}

#[test]
fn read_memory_in_same_instruction_after_reading_0xffff() {
    let mut bench = TestBench::default();
    bench.cpu.data.sp = 0xffff;
    bench.cpu.data.ie = 0x15;
    const POP_BC: u8 = 0xc1;
    bench.trace_fetch(bench.cpu.data.pc, &[POP_BC]);
    bench.trace_bus_read(0xffff, 0xff);
    bench.trace_bus_read(0x0000, 0x42);
    bench.trace_fetch(bench.cpu.data.pc, &[NOP]);
    assert_eq!(bench.cpu.data.bc(), 0x4215)
}

#[test]
fn writing_0xffff_sets_5_lower_bits_of_ie() {
    let mut bench = TestBench::default();
    bench.cpu.data.ie = 0x00;
    bench.cpu.data.a = 0xff;
    const LD_DEREF_N_A: u8 = 0xe0;
    let dest = 0xffff;
    bench.trace_fetch(bench.cpu.data.pc, &[LD_DEREF_N_A, low_byte(dest)]);
    bench.trace_bus_write(dest, bench.cpu.data.a);
    assert_eq!(bench.cpu.data.ie, 0x1f)
}

#[test]
fn writing_0xffff_during_interrupt_dispatch_updates_ie() {
    let mut bench = TestBench::default();
    bench.cpu.data.pc = 0xff00;
    bench.cpu.data.sp = 0x0000;
    bench.cpu.data.ie = 0x01;
    bench.cpu.data.ime = true;
    bench.r#if = 0x01;
    bench.trace_fetch(bench.cpu.data.pc, &[NOP]);
    bench.trace_interrupt_dispatch(0x01);
    assert_eq!(bench.cpu.data.ie, 0x1f)
}

impl TestBench {
    fn trace_interrupt_dispatch(&mut self, ack: u8) {
        let pc = self.cpu.data.pc;
        let sp = self.cpu.data.sp;
        self.trace_bus_no_op();
        self.trace_bus_no_op();
        self.trace_bus_write(sp.wrapping_sub(1), high_byte(pc));
        self.trace_step(None, output!(bus: bus_write(sp.wrapping_sub(2), low_byte(pc))));
        self.trace_step(None, output!(ack: ack))
    }
}
