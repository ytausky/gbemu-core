use super::*;

impl<'a> RunView<'a, InterruptDispatchState> {
    pub(super) fn step(&mut self, input: &Input) -> (Option<ModeTransition>, CpuOutput) {
        match self.run.m_cycle {
            M2 => (None, None),
            M3 => (None, None),
            M4 => match self.basic.phase {
                Tick => {
                    self.basic.sp -= 1;
                    (
                        None,
                        Some(BusOp::Write(self.basic.sp, high_byte(self.basic.pc))),
                    )
                }
                Tock => (None, None),
            },
            M5 => match self.basic.phase {
                Tick => {
                    self.basic.sp -= 1;
                    (
                        None,
                        Some(BusOp::Write(self.basic.sp, low_byte(self.basic.pc))),
                    )
                }
                Tock => {
                    self.basic.ime = false;
                    let n = input.r#if.trailing_zeros();
                    self.basic.pc = 0x0040 + 8 * n as u16;
                    (Some(ModeTransition::Instruction(NOP)), None)
                }
            },
            _ => unreachable!(),
        }
    }
}
