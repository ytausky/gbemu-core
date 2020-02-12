use super::*;

impl<'a> View<'a, InterruptDispatchState> {
    pub(super) fn step(&mut self, input: &Input) -> (Option<ModeTransition>, CpuOutput) {
        match self.data.m_cycle {
            M2 => (None, None),
            M3 => (None, None),
            M4 => match self.data.phase {
                Tick => {
                    self.data.sp -= 1;
                    (
                        None,
                        Some(BusOp::Write(self.data.sp, high_byte(self.data.pc))),
                    )
                }
                Tock => (None, None),
            },
            M5 => match self.data.phase {
                Tick => {
                    self.data.sp -= 1;
                    (
                        None,
                        Some(BusOp::Write(self.data.sp, low_byte(self.data.pc))),
                    )
                }
                Tock => {
                    self.data.ime = false;
                    let n = input.r#if.trailing_zeros();
                    self.data.pc = 0x0040 + 8 * n as u16;
                    (Some(ModeTransition::Instruction(0x00)), None)
                }
            },
            _ => unreachable!(),
        }
    }
}
