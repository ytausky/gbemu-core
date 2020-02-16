use super::*;

impl<'a> RunView<'a, InterruptDispatchState> {
    pub(super) fn step(&mut self, input: &Input) -> (Option<ModeTransition>, Output) {
        match self.run.m_cycle {
            M2 => (None, Default::default()),
            M3 => (None, Default::default()),
            M4 => match self.basic.phase {
                Tick => {
                    self.basic.sp = self.basic.sp.wrapping_sub(1);
                    (
                        None,
                        Output {
                            bus: self.bus_write(self.basic.sp, high_byte(self.basic.pc)),
                            ack: 0x00,
                        },
                    )
                }
                Tock => (None, Default::default()),
            },
            M5 => match self.basic.phase {
                Tick => {
                    self.basic.sp = self.basic.sp.wrapping_sub(1);
                    (
                        None,
                        Output {
                            bus: self.bus_write(self.basic.sp, low_byte(self.basic.pc)),
                            ack: 0x00,
                        },
                    )
                }
                Tock => {
                    self.basic.ime = false;
                    let n = (input.r#if & self.basic.ie).trailing_zeros();
                    self.basic.pc = 0x0040 + 8 * n as u16;
                    (
                        Some(ModeTransition::Instruction(NOP)),
                        Output {
                            bus: None,
                            ack: 1 << n,
                        },
                    )
                }
            },
            _ => unreachable!(),
        }
    }
}
