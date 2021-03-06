use super::*;

#[test]
fn dispatch_interrupt_0() {
    let mut bench = TestBench::default();
    bench.trace_interrupt_request_and_dispatch(0);
    assert_eq!(bench.trace, bench.expected)
}

#[test]
fn interrupt_0_dispatch_jumps_to_0x0040() {
    let mut bench = TestBench::default();
    bench.trace_interrupt_request_and_dispatch(0);
    assert_eq!(bench.cpu.data.pc, 0x0040)
}

#[test]
fn interrupt_0_dispatch_resets_ime() {
    let mut bench = TestBench::default();
    bench.trace_interrupt_request_and_dispatch(0);
    assert!(!bench.cpu.data.ime)
}

#[test]
fn dispatch_interrupt_1() {
    let mut bench = TestBench::default();
    bench.trace_interrupt_request_and_dispatch(1);
    assert_eq!(bench.trace, bench.expected)
}

#[test]
fn interrupt_1_dispatch_jumps_to_0x0048() {
    let mut bench = TestBench::default();
    bench.trace_interrupt_request_and_dispatch(1);
    assert_eq!(bench.cpu.data.pc, 0x0048)
}

#[test]
fn disabled_interrupt_0_does_not_cause_interrupt_dispatch() {
    let mut bench = TestBench::default();
    bench.cpu.data.ie = 0x1e;
    bench.r#if = 0x01;
    bench.trace_fetch(bench.cpu.data.pc, &[NOP]);
    bench.trace_fetch(bench.cpu.data.pc, &[NOP]);
    assert_eq!(bench.trace, bench.expected)
}

#[test]
fn enabled_interrupt_not_dispatched_with_reset_ime() {
    let mut bench = TestBench::default();
    bench.cpu.data.ime = false;
    bench.r#if = 0x01;
    bench.trace_fetch(bench.cpu.data.pc, &[NOP]);
    bench.trace_fetch(bench.cpu.data.pc, &[NOP]);
    assert_eq!(bench.trace, bench.expected)
}

#[test]
fn ie_is_checked_when_choosing_interrupt_vector() {
    let mut bench = TestBench::default();
    bench.cpu.data.ie = 0x1e;
    bench.r#if = 0x03;
    bench.trace_fetch(bench.cpu.data.pc, &[NOP]);
    bench.trace_interrupt_dispatch(1);
    assert_eq!(bench.trace, bench.expected)
}

#[test]
fn halt_mode_canceled_and_interrupt_dispatched_when_ime_is_set() {
    let mut bench = TestBench::default();
    bench.request_interrupt_in_halt_mode(0);
    bench.trace_interrupt_dispatch(0);
    assert_eq!(bench.trace, bench.expected)
}

#[test]
fn halt_mode_canceled_and_interrupt_not_dispatched_when_ime_is_reset() {
    let mut bench = TestBench::default();
    bench.cpu.data.ime = false;
    bench.request_interrupt_in_halt_mode(0);
    bench.trace_fetch(bench.cpu.data.pc, &[NOP]);
    bench.trace_fetch(bench.cpu.data.pc, &[NOP]);
    assert_eq!(bench.trace, bench.expected)
}

#[test]
fn reading_0xffff_returns_ie() {
    let mut bench = TestBench::default();
    bench.cpu.data.ie = 0x15;
    const LD_A_DEREF_N: u8 = 0xf0;
    let src = 0xffff;
    bench.trace_fetch(bench.cpu.data.pc, &[LD_A_DEREF_N, low_byte(src)]);
    bench.trace_open_bus_read(src);
    bench.trace_fetch(bench.cpu.data.pc, &[NOP]);
    assert_eq!(bench.cpu.data.a, 0x15)
}

#[test]
fn read_memory_in_same_instruction_after_reading_0xffff() {
    let mut bench = TestBench::default();
    bench.cpu.data.sp = 0xffff;
    const POP_BC: u8 = 0xc1;
    bench.trace_fetch(bench.cpu.data.pc, &[POP_BC]);
    bench.trace_open_bus_read(0xffff);
    bench.trace_bus_read(0x0000, 0x42);
    bench.trace_fetch(bench.cpu.data.pc, &[NOP]);
    assert_eq!(bench.cpu.data.bc(), 0x421f)
}

#[test]
fn writing_0xffff_updates_ie() {
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
fn writing_pc_h_to_0xffff_during_interrupt_dispatch_updates_ie() {
    let mut bench = TestBench::default();
    bench.cpu.data.pc = 0xf500;
    bench.cpu.data.sp = 0x0000;
    bench.trace_interrupt_request_and_dispatch(0);
    assert_eq!(bench.cpu.data.ie, 0x15)
}

#[test]
fn writing_pc_l_to_0xffff_during_interrupt_dispatch_updates_ie() {
    let mut bench = TestBench::default();
    bench.cpu.data.pc = 0x00f5;
    bench.cpu.data.sp = 0x0001;
    bench.trace_interrupt_request_and_dispatch(0);
    assert_eq!(bench.cpu.data.ie, 0x15)
}

impl TestBench {
    fn trace_open_bus_read(&mut self, addr: u16) {
        self.trace_step(None, output!(bus: bus_read(addr)));
        self.trace_step(None, output!());
    }

    fn trace_interrupt_request_and_dispatch(&mut self, n: u32) {
        self.r#if = 1 << n;
        self.trace_fetch(self.cpu.data.pc, &[NOP]);
        self.trace_interrupt_dispatch(n);
    }

    fn trace_interrupt_dispatch(&mut self, n: u32) {
        let pc = self.cpu.data.pc;
        let sp = self.cpu.data.sp;
        self.trace_bus_no_op();
        self.trace_bus_no_op();
        self.trace_bus_write(sp.wrapping_sub(1), high_byte(pc));
        self.trace_step(
            None,
            output!(bus: bus_write(sp.wrapping_sub(2), low_byte(pc))),
        );
        self.trace_step(None, output!(ack: 1 << n))
    }

    fn request_interrupt_in_halt_mode(&mut self, n: u32) {
        // Fetch HALT opcode
        self.trace_fetch(self.cpu.data.pc, &[HALT]);

        // Execute HALT
        self.trace_bus_no_op();

        // Wait one M-cycle to avoid testing behavior immediately following HALT execution
        self.trace_bus_no_op();

        // Request interrupt
        self.r#if = 1 << n;
        self.trace_bus_no_op();
    }
}

const HALT: u8 = 0x76;
