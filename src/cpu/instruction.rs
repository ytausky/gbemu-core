use super::*;

fn split_opcode(opcode: u8) -> (u8, u8, u8) {
    (opcode >> 6, (opcode >> 3) & 0b111, opcode & 0b111)
}

impl<'a> RunView<'a, InstructionExecutionState> {
    pub(super) fn step(&mut self, input: &Input) -> (Option<ModeTransition>, Output) {
        match self.basic.phase {
            Tick => {
                let output = self.exec_instr();
                (None, output)
            }
            Tock => {
                self.state.bus_data = if self.state.read_ie {
                    self.state.read_ie = false;
                    Some(self.basic.ie)
                } else {
                    input.data
                };
                let transition = if self.state.m1 {
                    Some(if self.basic.ime && input.r#if & self.basic.ie != 0x00 {
                        ModeTransition::Interrupt
                    } else {
                        self.basic.pc += 1;
                        ModeTransition::Instruction(self.state.bus_data.unwrap())
                    })
                } else if let Some(standby) = self.state.standby {
                    Some(standby.into())
                } else {
                    None
                };
                (transition, Default::default())
            }
        }
    }

    fn exec_instr(&mut self) -> Output {
        let bus = match split_opcode(self.state.opcode) {
            (0b00, 0b000, 0b000) => self.nop(),
            (0b00, cc, 0b000) if cc & 0b100 > 0 => self.jr(Some((cc & 0b011).into())),
            (0b00, dest, 0b001) if dest & 0b001 == 0 => self.ld_dd_nn((dest >> 1).into()),
            (0b00, 0b000, 0b010) => self.ld_deref_bc_a(),
            (0b00, 0b110, 0b100) => self.inc_deref_hl(),
            (0b00, operand, 0b100) => self.inc_r(operand.into()),
            (0b00, 0b110, 0b110) => self.ld_deref_hl_n(),
            (0b00, dest, 0b110) => self.ld_r_n(dest.into()),
            (0b00, 0b001, 0b000) => self.ld_deref_nn_sp(),
            (0b00, 0b001, 0b010) => self.ld_a_deref_bc(),
            (0b00, 0b010, 0b010) => self.ld_deref_de_a(),
            (0b00, 0b011, 0b000) => self.jr(None),
            (0b00, 0b011, 0b010) => self.ld_a_deref_de(),
            (0b00, 0b100, 0b010) => self.ld_deref_hli_a(),
            (0b00, 0b101, 0b010) => self.ld_a_deref_hli(),
            (0b00, 0b110, 0b010) => self.ld_deref_hld_a(),
            (0b00, 0b111, 0b010) => self.ld_a_deref_hld(),
            (0b01, 0b110, 0b110) => self.halt(),
            (0b01, dest, 0b110) => self.ld_r_deref_hl(dest.into()),
            (0b01, 0b110, src) => self.ld_deref_hl_r(src.into()),
            (0b01, dest, src) => self.ld_r_r(dest.into(), src.into()),
            (0b10, op, 0b110) => self.alu_op_deref_hl(op.into()),
            (0b10, op, src) => self.alu_op_r(op.into(), src.into()),
            (0b11, dest, 0b001) if dest & 0b001 == 0 => self.pop_qq((dest >> 1).into()),
            (0b11, 0b000, 0b011) => self.jp(None),
            (0b11, cc, 0b010) if cc <= 0b011 => self.jp(Some(cc.into())),
            (0b11, src, 0b101) if src & 0b001 == 0 => self.push_qq((src >> 1).into()),
            (0b11, op, 0b110) => self.alu_op_n(op.into()),
            (0b11, 0b001, 0b001) => self.ret(),
            (0b11, 0b001, 0b101) => self.call(),
            (0b11, 0b100, 0b000) => self.ld_deref_n_a(),
            (0b11, 0b100, 0b010) => self.ld_deref_c_a(),
            (0b11, 0b101, 0b001) => self.jp_deref_hl(),
            (0b11, 0b101, 0b010) => self.ld_deref_nn_a(),
            (0b11, 0b110, 0b000) => self.ld_a_deref_n(),
            (0b11, 0b110, 0b010) => self.ld_a_deref_c(),
            (0b11, 0b111, 0b000) => self.ldhl_sp_e(),
            (0b11, 0b111, 0b001) => self.ld_sp_hl(),
            (0b11, 0b111, 0b010) => self.ld_a_deref_nn(),
            _ => unimplemented!(),
        };
        Output { bus, ack: 0x00 }
    }

    fn nop(&mut self) -> Option<BusActivity> {
        match self.run.m_cycle {
            M2 => self.execute_m1(),
            _ => unreachable!(),
        }
    }

    fn halt(&mut self) -> Option<BusActivity> {
        match self.run.m_cycle {
            M2 => {
                self.state.standby = Some(Standby::Halt);
                None
            }
            _ => unreachable!(),
        }
    }

    fn ld_r_r(&mut self, dest: R, src: R) -> Option<BusActivity> {
        match self.run.m_cycle {
            M2 => {
                self.basic.write(dest, self.basic.read(src));
                self.execute_m1()
            }
            _ => unreachable!(),
        }
    }

    fn ld_r_n(&mut self, dest: R) -> Option<BusActivity> {
        match self.run.m_cycle {
            M2 => self.read_immediate(),
            M3 => {
                self.basic.write(dest, self.state.bus_data.unwrap());
                self.execute_m1()
            }
            _ => unreachable!(),
        }
    }

    fn ld_r_deref_hl(&mut self, dest: R) -> Option<BusActivity> {
        match self.run.m_cycle {
            M2 => self.bus_read(self.basic.hl()),
            M3 => {
                self.basic.write(dest, self.state.bus_data.unwrap());
                self.execute_m1()
            }
            _ => unreachable!(),
        }
    }

    fn ld_deref_hl_r(&mut self, src: R) -> Option<BusActivity> {
        match self.run.m_cycle {
            M2 => self.bus_write(self.basic.hl(), self.basic.read(src)),
            M3 => self.execute_m1(),
            _ => unreachable!(),
        }
    }

    fn ld_deref_hl_n(&mut self) -> Option<BusActivity> {
        match self.run.m_cycle {
            M2 => self.read_immediate(),
            M3 => self.bus_write(self.basic.hl(), self.state.bus_data.unwrap()),
            M4 => self.execute_m1(),
            _ => unreachable!(),
        }
    }

    fn ld_a_deref_bc(&mut self) -> Option<BusActivity> {
        match self.run.m_cycle {
            M2 => self.bus_read(self.basic.bc()),
            M3 => {
                self.basic.a = self.state.bus_data.unwrap();
                self.execute_m1()
            }
            _ => unreachable!(),
        }
    }

    fn ld_a_deref_de(&mut self) -> Option<BusActivity> {
        match self.run.m_cycle {
            M2 => self.bus_read(self.basic.de()),
            M3 => {
                self.basic.a = self.state.bus_data.unwrap();
                self.execute_m1()
            }
            _ => unreachable!(),
        }
    }

    fn ld_a_deref_c(&mut self) -> Option<BusActivity> {
        match self.run.m_cycle {
            M2 => self.bus_read(u16::from_be_bytes([0xff, self.basic.c])),
            M3 => {
                self.basic.a = self.state.bus_data.unwrap();
                self.execute_m1()
            }
            _ => unreachable!(),
        }
    }

    fn ld_deref_c_a(&mut self) -> Option<BusActivity> {
        match self.run.m_cycle {
            M2 => self.bus_write(u16::from_be_bytes([0xff, self.basic.c]), self.basic.a),
            M3 => self.execute_m1(),
            _ => unreachable!(),
        }
    }

    fn ld_a_deref_n(&mut self) -> Option<BusActivity> {
        match self.run.m_cycle {
            M2 => self.read_immediate(),
            M3 => self.bus_read(u16::from_be_bytes([0xff, self.state.bus_data.unwrap()])),
            M4 => {
                self.basic.a = self.state.bus_data.unwrap();
                self.execute_m1()
            }
            _ => unreachable!(),
        }
    }

    fn ld_deref_n_a(&mut self) -> Option<BusActivity> {
        match self.run.m_cycle {
            M2 => self.read_immediate(),
            M3 => self.bus_write(
                u16::from_be_bytes([0xff, self.state.bus_data.unwrap()]),
                self.basic.a,
            ),
            M4 => self.execute_m1(),
            _ => unreachable!(),
        }
    }

    fn ld_a_deref_nn(&mut self) -> Option<BusActivity> {
        match self.run.m_cycle {
            M2 => self.read_immediate(),
            M3 => {
                self.state.data = self.state.bus_data.unwrap();
                self.read_immediate()
            }
            M4 => self.bus_read(u16::from_be_bytes([
                self.state.bus_data.unwrap(),
                self.state.data,
            ])),
            M5 => {
                self.basic.a = self.state.bus_data.unwrap();
                self.execute_m1()
            }
            _ => unreachable!(),
        }
    }

    fn ld_deref_nn_a(&mut self) -> Option<BusActivity> {
        match self.run.m_cycle {
            M2 => self.read_immediate(),
            M3 => {
                self.state.data = self.state.bus_data.unwrap();
                self.read_immediate()
            }
            M4 => self.bus_write(
                u16::from_be_bytes([self.state.bus_data.unwrap(), self.state.data]),
                self.basic.a,
            ),
            M5 => self.execute_m1(),
            _ => unreachable!(),
        }
    }

    fn ld_a_deref_hli(&mut self) -> Option<BusActivity> {
        match self.run.m_cycle {
            M2 => {
                let hl = self.basic.hl();
                let incremented_hl = hl + 1;
                self.basic.h = high_byte(incremented_hl);
                self.basic.l = low_byte(incremented_hl);
                self.bus_read(hl)
            }
            M3 => {
                self.basic.a = self.state.bus_data.unwrap();
                self.execute_m1()
            }
            _ => unreachable!(),
        }
    }

    fn ld_a_deref_hld(&mut self) -> Option<BusActivity> {
        match self.run.m_cycle {
            M2 => {
                let hl = self.basic.hl();
                let decremented_hl = hl - 1;
                self.basic.h = high_byte(decremented_hl);
                self.basic.l = low_byte(decremented_hl);
                self.bus_read(hl)
            }
            M3 => {
                self.basic.a = self.state.bus_data.unwrap();
                self.execute_m1()
            }
            _ => unreachable!(),
        }
    }

    fn ld_deref_bc_a(&mut self) -> Option<BusActivity> {
        match self.run.m_cycle {
            M2 => self.bus_write(self.basic.bc(), self.basic.a),
            M3 => self.execute_m1(),
            _ => unreachable!(),
        }
    }

    fn ld_deref_de_a(&mut self) -> Option<BusActivity> {
        match self.run.m_cycle {
            M2 => self.bus_write(self.basic.de(), self.basic.a),
            M3 => self.execute_m1(),
            _ => unreachable!(),
        }
    }

    fn ld_deref_hli_a(&mut self) -> Option<BusActivity> {
        match self.run.m_cycle {
            M2 => {
                let hl = self.basic.hl();
                let incremented_hl = hl.wrapping_add(1);
                self.basic.h = high_byte(incremented_hl);
                self.basic.l = low_byte(incremented_hl);
                self.bus_write(hl, self.basic.a)
            }
            M3 => self.execute_m1(),
            _ => unreachable!(),
        }
    }

    fn ld_deref_hld_a(&mut self) -> Option<BusActivity> {
        match self.run.m_cycle {
            M2 => {
                let hl = self.basic.hl();
                let decremented_hl = hl - 1;
                self.basic.h = high_byte(decremented_hl);
                self.basic.l = low_byte(decremented_hl);
                self.bus_write(hl, self.basic.a)
            }
            M3 => self.execute_m1(),
            _ => unreachable!(),
        }
    }

    fn ld_dd_nn(&mut self, dd: Dd) -> Option<BusActivity> {
        match self.run.m_cycle {
            M2 => self.read_immediate(),
            M3 => {
                self.basic.write(dd.low(), self.state.bus_data.unwrap());
                self.read_immediate()
            }
            M4 => {
                self.basic.write(dd.high(), self.state.bus_data.unwrap());
                self.execute_m1()
            }
            _ => unreachable!(),
        }
    }

    fn ld_sp_hl(&mut self) -> Option<BusActivity> {
        match self.run.m_cycle {
            M2 => {
                self.basic.sp = self.basic.hl();
                None
            }
            M3 => self.execute_m1(),
            _ => unreachable!(),
        }
    }

    fn push_qq(&mut self, qq: Qq) -> Option<BusActivity> {
        match self.run.m_cycle {
            M2 => None,
            M3 => self.push_byte(self.basic.read(qq.high())),
            M4 => self.push_byte(self.basic.read(qq.low())),
            M5 => self.execute_m1(),
            _ => unreachable!(),
        }
    }

    fn pop_qq(&mut self, qq: Qq) -> Option<BusActivity> {
        match self.run.m_cycle {
            M2 => self.pop_byte(),
            M3 => {
                self.basic.write(qq.low(), self.state.bus_data.unwrap());
                self.pop_byte()
            }
            M4 => {
                self.basic.write(qq.high(), self.state.bus_data.unwrap());
                self.execute_m1()
            }
            _ => unreachable!(),
        }
    }

    fn ldhl_sp_e(&mut self) -> Option<BusActivity> {
        match self.run.m_cycle {
            M2 => self.read_immediate(),
            M3 => {
                let e = self.state.bus_data.unwrap();
                let (l, flags) = add(low_byte(self.basic.sp), e, false);
                let (h, _) = add(high_byte(self.basic.sp), sign_extension(e), flags.cy);
                self.basic.h = h;
                self.basic.l = l;
                self.basic.f = flags;
                self.basic.f.z = false;
                None
            }
            M4 => self.execute_m1(),
            _ => unreachable!(),
        }
    }

    fn ld_deref_nn_sp(&mut self) -> Option<BusActivity> {
        match self.run.m_cycle {
            M2 => self.read_immediate(),
            M3 => {
                self.state.addr = self.state.bus_data.unwrap() as u16;
                self.read_immediate()
            }
            M4 => {
                self.state.addr |= (self.state.bus_data.unwrap() as u16) << 8;
                self.bus_write(self.state.addr, low_byte(self.basic.sp))
            }
            M5 => self.bus_write(self.state.addr + 1, high_byte(self.basic.sp)),
            M6 => self.execute_m1(),
            _ => unreachable!(),
        }
    }

    fn alu_op_r(&mut self, op: AluOp, r: R) -> Option<BusActivity> {
        match self.run.m_cycle {
            M2 => {
                let (result, flags) = self.alu_op(op, self.basic.a, self.basic.read(r));
                self.basic.a = result;
                self.basic.f = flags;
                self.execute_m1()
            }
            _ => unreachable!(),
        }
    }

    fn alu_op_n(&mut self, op: AluOp) -> Option<BusActivity> {
        match self.run.m_cycle {
            M2 => self.read_immediate(),
            M3 => {
                let (result, flags) = self.alu_op(op, self.basic.a, self.state.bus_data.unwrap());
                self.basic.a = result;
                self.basic.f = flags;
                self.execute_m1()
            }
            _ => unreachable!(),
        }
    }

    fn alu_op_deref_hl(&mut self, op: AluOp) -> Option<BusActivity> {
        match self.run.m_cycle {
            M2 => self.bus_read(self.basic.hl()),
            M3 => {
                let (result, flags) = self.alu_op(op, self.basic.a, self.state.bus_data.unwrap());
                self.basic.a = result;
                self.basic.f = flags;
                self.execute_m1()
            }
            _ => unreachable!(),
        }
    }

    fn inc_r(&mut self, r: R) -> Option<BusActivity> {
        match self.run.m_cycle {
            M2 => {
                let (result, flags) = add(self.basic.read(r), 1, false);
                self.basic.write(r, result);
                self.basic.f.z = flags.z;
                self.basic.f.n = flags.n;
                self.basic.f.h = flags.h;
                self.execute_m1()
            }
            _ => unreachable!(),
        }
    }

    fn inc_deref_hl(&mut self) -> Option<BusActivity> {
        match self.run.m_cycle {
            M2 => self.bus_read(self.basic.hl()),
            M3 => {
                let (result, flags) = add(self.state.bus_data.unwrap(), 1, false);
                self.basic.f.z = flags.z;
                self.basic.f.n = flags.n;
                self.basic.f.h = flags.h;
                self.bus_write(self.basic.hl(), result)
            }
            M4 => self.execute_m1(),
            _ => unreachable!(),
        }
    }

    fn jp(&mut self, cc: Option<Cc>) -> Option<BusActivity> {
        match self.run.m_cycle {
            M2 => self.read_immediate(),
            M3 => {
                self.state.data = self.state.bus_data.unwrap();
                self.read_immediate()
            }
            M4 => {
                if cc.map(|cc| self.evaluate_condition(cc)).unwrap_or(true) {
                    self.basic.pc =
                        u16::from_be_bytes([self.state.bus_data.unwrap(), self.state.data]);
                    None
                } else {
                    self.execute_m1()
                }
            }
            M5 => self.execute_m1(),
            _ => unreachable!(),
        }
    }

    fn jr(&mut self, cc: Option<Cc>) -> Option<BusActivity> {
        match self.run.m_cycle {
            M2 => self.read_immediate(),
            M3 => {
                if cc.map(|cc| self.evaluate_condition(cc)).unwrap_or(true) {
                    let e = self.state.bus_data.unwrap() as i8;
                    self.basic.pc = self.basic.pc.wrapping_add(e as i16 as u16);
                    None
                } else {
                    self.execute_m1()
                }
            }
            M4 => self.execute_m1(),
            _ => unreachable!(),
        }
    }

    fn jp_deref_hl(&mut self) -> Option<BusActivity> {
        match self.run.m_cycle {
            M2 => {
                self.basic.pc = self.basic.hl();
                self.execute_m1()
            }
            _ => unreachable!(),
        }
    }

    fn call(&mut self) -> Option<BusActivity> {
        match self.run.m_cycle {
            M2 => self.read_immediate(),
            M3 => {
                self.state.data = self.state.bus_data.unwrap();
                self.read_immediate()
            }
            M4 => {
                self.state.addr =
                    u16::from_be_bytes([self.state.bus_data.unwrap(), self.state.data]);
                None
            }
            M5 => self.push_byte(high_byte(self.basic.pc)),
            M6 => {
                let pc = self.basic.pc;
                self.basic.pc = self.state.addr;
                self.push_byte(low_byte(pc))
            }
            M7 => self.execute_m1(),
            _ => unreachable!(),
        }
    }

    fn ret(&mut self) -> Option<BusActivity> {
        match self.run.m_cycle {
            M2 => self.pop_byte(),
            M3 => {
                self.state.data = self.state.bus_data.unwrap();
                self.pop_byte()
            }
            M4 => {
                self.basic.pc = u16::from_be_bytes([self.state.bus_data.unwrap(), self.state.data]);
                None
            }
            M5 => self.execute_m1(),
            _ => unreachable!(),
        }
    }

    fn execute_m1(&mut self) -> Option<BusActivity> {
        self.state.m1 = true;
        self.bus_read(self.basic.pc)
    }

    fn read_immediate(&mut self) -> Option<BusActivity> {
        let addr = self.basic.pc;
        self.basic.pc += 1;
        self.bus_read(addr)
    }

    fn push_byte(&mut self, data: u8) -> Option<BusActivity> {
        self.basic.sp -= 1;
        Some(BusActivity {
            addr: self.basic.sp,
            op: Some(BusOp::Write(data)),
        })
    }

    fn pop_byte(&mut self) -> Option<BusActivity> {
        let addr = self.basic.sp;
        self.basic.sp = self.basic.sp.wrapping_add(1);
        self.bus_read(addr)
    }

    fn bus_read(&mut self, addr: u16) -> Option<BusActivity> {
        if addr == 0xffff {
            self.state.read_ie = true;
        }
        Some(bus_read(addr))
    }

    fn alu_op(&self, op: AluOp, lhs: u8, rhs: u8) -> (u8, Flags) {
        match op {
            AluOp::Add => add(lhs, rhs, false),
            AluOp::Adc => add(lhs, rhs, self.basic.f.cy),
            AluOp::Sub => sub(lhs, rhs, false),
            AluOp::Sbc => sub(lhs, rhs, self.basic.f.cy),
            AluOp::And => and(lhs, rhs),
            AluOp::Xor => xor(lhs, rhs),
            AluOp::Or => or(lhs, rhs),
            AluOp::Cp => {
                let (_, flags) = sub(lhs, rhs, false);
                (lhs, flags)
            }
        }
    }

    fn evaluate_condition(&self, cc: Cc) -> bool {
        match cc {
            Cc::Nz => !self.basic.f.z,
            Cc::Z => self.basic.f.z,
            Cc::Nc => !self.basic.f.cy,
            Cc::C => self.basic.f.cy,
        }
    }
}

fn add(lhs: u8, rhs: u8, carry_in: bool) -> (u8, Flags) {
    let (partial, overflow1) = lhs.overflowing_add(rhs);
    let (result, overflow2) = partial.overflowing_add(carry_in.into());
    (
        result,
        Flags {
            z: result == 0,
            n: false,
            h: (lhs & 0x0f) + (rhs & 0x0f) + u8::from(carry_in) > 0x0f,
            cy: overflow1 | overflow2,
        },
    )
}

fn sub(lhs: u8, rhs: u8, carry_in: bool) -> (u8, Flags) {
    let carry_in = u8::from(carry_in);
    let (partial, overflow1) = lhs.overflowing_sub(rhs);
    let (result, overflow2) = partial.overflowing_sub(carry_in);
    (
        result,
        Flags {
            z: result == 0,
            n: true,
            h: (lhs & 0x0f).wrapping_sub(rhs & 0x0f).wrapping_sub(carry_in) > 0x0f,
            cy: overflow1 | overflow2,
        },
    )
}

fn and(lhs: u8, rhs: u8) -> (u8, Flags) {
    let result = lhs & rhs;
    (
        result,
        Flags {
            z: result == 0,
            h: true,
            ..Default::default()
        },
    )
}

fn xor(lhs: u8, rhs: u8) -> (u8, Flags) {
    let result = lhs ^ rhs;
    (
        result,
        Flags {
            z: result == 0,
            ..Default::default()
        },
    )
}

fn or(lhs: u8, rhs: u8) -> (u8, Flags) {
    let result = lhs | rhs;
    (
        result,
        Flags {
            z: result == 0,
            ..Default::default()
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn addition() {
        assert_eq!(add(0x12, 0x34, false), (0x46, flags!()))
    }

    #[test]
    fn addition_sets_h() {
        assert_eq!(add(0x08, 0x08, false), (0x10, flags!(h)))
    }

    #[test]
    fn addition_is_wrapping() {
        assert_eq!(add(0x80, 0x80, false), (0x00, flags!(z, cy)))
    }

    #[test]
    fn addition_with_carry_in() {
        assert_eq!(add(0xff, 0x00, true), (0x00, flags!(z, h, cy)))
    }

    #[test]
    fn subtraction_sets_n() {
        assert_eq!(sub(0x34, 0x12, false), (0x22, flags!(n)))
    }

    #[test]
    fn subtraction_sets_h() {
        assert_eq!(sub(0x10, 0x01, false), (0x0f, flags!(n, h)))
    }

    #[test]
    fn subtraction_is_wrapping() {
        assert_eq!(sub(0x00, 0x01, false), (0xff, flags!(n, h, cy)))
    }

    #[test]
    fn subtraction_with_carry_in() {
        assert_eq!(sub(0x00, 0x00, true), (0xff, flags!(n, h, cy)))
    }

    #[test]
    fn subtraction_sets_z() {
        assert_eq!(sub(0x07, 0x07, false), (0x00, flags!(z, n)))
    }
}
