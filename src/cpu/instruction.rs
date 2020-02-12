use super::*;

fn split_opcode(opcode: u8) -> (u8, u8, u8) {
    (opcode >> 6, (opcode >> 3) & 0b111, opcode & 0b111)
}

impl<'a> View<'a, InstructionExecutionState> {
    pub(super) fn step(&mut self, input: &Input) -> (Option<ModeTransition>, CpuOutput) {
        match self.data.phase {
            Tick => {
                let output = self.exec_instr();
                (None, output)
            }
            Tock => {
                self.state.bus_data = input.data;
                let transition = if self.state.m1 {
                    Some(if input.r#if & self.data.ie != 0x00 {
                        ModeTransition::Interrupt
                    } else {
                        self.data.pc += 1;
                        ModeTransition::Instruction(self.state.bus_data.unwrap())
                    })
                } else {
                    None
                };
                (transition, None)
            }
        }
    }

    fn exec_instr(&mut self) -> CpuOutput {
        match split_opcode(self.state.opcode) {
            (0b00, 0b000, 0b000) => self.nop(),
            (0b00, dest, 0b001) if dest & 0b001 == 0 => self.ld_dd_nn((dest >> 1).into()),
            (0b00, 0b000, 0b010) => self.ld_deref_bc_a(),
            (0b00, 0b110, 0b100) => self.inc_deref_hl(),
            (0b00, operand, 0b100) => self.inc_r(operand.into()),
            (0b00, 0b110, 0b110) => self.ld_deref_hl_n(),
            (0b00, dest, 0b110) => self.ld_r_n(dest.into()),
            (0b00, 0b001, 0b000) => self.ld_deref_nn_sp(),
            (0b00, 0b001, 0b010) => self.ld_a_deref_bc(),
            (0b00, 0b010, 0b010) => self.ld_deref_de_a(),
            (0b00, 0b011, 0b000) => self.jr_e(),
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
            (0b11, 0b000, 0b011) => self.jp_nn(),
            (0b11, cc, 0b010) if cc <= 0b011 => self.jp_cc_nn(cc.into()),
            (0b11, src, 0b101) if src & 0b001 == 0 => self.push_qq((src >> 1).into()),
            (0b11, op, 0b110) => self.alu_op_n(op.into()),
            (0b11, 0b001, 0b001) => self.ret(),
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
        }
    }

    fn nop(&mut self) -> Option<BusOp> {
        match self.data.m_cycle {
            M2 => self.execute_m1(),
            _ => unreachable!(),
        }
    }

    fn halt(&mut self) -> Option<BusOp> {
        unimplemented!()
    }

    fn ld_r_r(&mut self, dest: R, src: R) -> Option<BusOp> {
        match self.data.m_cycle {
            M2 => {
                self.data.write(dest, self.data.read(src));
                self.execute_m1()
            }
            _ => unreachable!(),
        }
    }

    fn ld_r_n(&mut self, dest: R) -> Option<BusOp> {
        match self.data.m_cycle {
            M2 => self.read_immediate(),
            M3 => {
                self.data.write(dest, self.state.bus_data.unwrap());
                self.execute_m1()
            }
            _ => unreachable!(),
        }
    }

    fn ld_r_deref_hl(&mut self, dest: R) -> Option<BusOp> {
        match self.data.m_cycle {
            M2 => Some(BusOp::Read(self.data.hl())),
            M3 => {
                self.data.write(dest, self.state.bus_data.unwrap());
                self.execute_m1()
            }
            _ => unreachable!(),
        }
    }

    fn ld_deref_hl_r(&mut self, src: R) -> Option<BusOp> {
        match self.data.m_cycle {
            M2 => Some(BusOp::Write(self.data.hl(), self.data.read(src))),
            M3 => self.execute_m1(),
            _ => unreachable!(),
        }
    }

    fn ld_deref_hl_n(&mut self) -> Option<BusOp> {
        match self.data.m_cycle {
            M2 => self.read_immediate(),
            M3 => Some(BusOp::Write(self.data.hl(), self.state.bus_data.unwrap())),
            M4 => self.execute_m1(),
            _ => unreachable!(),
        }
    }

    fn ld_a_deref_bc(&mut self) -> Option<BusOp> {
        match self.data.m_cycle {
            M2 => Some(BusOp::Read(self.data.bc())),
            M3 => {
                self.data.a = self.state.bus_data.unwrap();
                self.execute_m1()
            }
            _ => unreachable!(),
        }
    }

    fn ld_a_deref_de(&mut self) -> Option<BusOp> {
        match self.data.m_cycle {
            M2 => Some(BusOp::Read(self.data.de())),
            M3 => {
                self.data.a = self.state.bus_data.unwrap();
                self.execute_m1()
            }
            _ => unreachable!(),
        }
    }

    fn ld_a_deref_c(&mut self) -> Option<BusOp> {
        match self.data.m_cycle {
            M2 => Some(BusOp::Read(u16::from_be_bytes([0xff, self.data.c]))),
            M3 => {
                self.data.a = self.state.bus_data.unwrap();
                self.execute_m1()
            }
            _ => unreachable!(),
        }
    }

    fn ld_deref_c_a(&mut self) -> Option<BusOp> {
        match self.data.m_cycle {
            M2 => Some(BusOp::Write(
                u16::from_be_bytes([0xff, self.data.c]),
                self.data.a,
            )),
            M3 => self.execute_m1(),
            _ => unreachable!(),
        }
    }

    fn ld_a_deref_n(&mut self) -> Option<BusOp> {
        match self.data.m_cycle {
            M2 => self.read_immediate(),
            M3 => Some(BusOp::Read(u16::from_be_bytes([
                0xff,
                self.state.bus_data.unwrap(),
            ]))),
            M4 => {
                self.data.a = self.state.bus_data.unwrap();
                self.execute_m1()
            }
            _ => unreachable!(),
        }
    }

    fn ld_deref_n_a(&mut self) -> Option<BusOp> {
        match self.data.m_cycle {
            M2 => self.read_immediate(),
            M3 => Some(BusOp::Write(
                u16::from_be_bytes([0xff, self.state.bus_data.unwrap()]),
                self.data.a,
            )),
            M4 => self.execute_m1(),
            _ => unreachable!(),
        }
    }

    fn ld_a_deref_nn(&mut self) -> Option<BusOp> {
        match self.data.m_cycle {
            M2 => self.read_immediate(),
            M3 => {
                self.state.data = self.state.bus_data.unwrap();
                self.read_immediate()
            }
            M4 => Some(BusOp::Read(u16::from_be_bytes([
                self.state.bus_data.unwrap(),
                self.state.data,
            ]))),
            M5 => {
                self.data.a = self.state.bus_data.unwrap();
                self.execute_m1()
            }
            _ => unreachable!(),
        }
    }

    fn ld_deref_nn_a(&mut self) -> Option<BusOp> {
        match self.data.m_cycle {
            M2 => self.read_immediate(),
            M3 => {
                self.state.data = self.state.bus_data.unwrap();
                self.read_immediate()
            }
            M4 => Some(BusOp::Write(
                u16::from_be_bytes([self.state.bus_data.unwrap(), self.state.data]),
                self.data.a,
            )),
            M5 => self.execute_m1(),
            _ => unreachable!(),
        }
    }

    fn ld_a_deref_hli(&mut self) -> Option<BusOp> {
        match self.data.m_cycle {
            M2 => {
                let hl = self.data.hl();
                let incremented_hl = hl + 1;
                self.data.h = high_byte(incremented_hl);
                self.data.l = low_byte(incremented_hl);
                Some(BusOp::Read(hl))
            }
            M3 => {
                self.data.a = self.state.bus_data.unwrap();
                self.execute_m1()
            }
            _ => unreachable!(),
        }
    }

    fn ld_a_deref_hld(&mut self) -> Option<BusOp> {
        match self.data.m_cycle {
            M2 => {
                let hl = self.data.hl();
                let decremented_hl = hl - 1;
                self.data.h = high_byte(decremented_hl);
                self.data.l = low_byte(decremented_hl);
                Some(BusOp::Read(hl))
            }
            M3 => {
                self.data.a = self.state.bus_data.unwrap();
                self.execute_m1()
            }
            _ => unreachable!(),
        }
    }

    fn ld_deref_bc_a(&mut self) -> Option<BusOp> {
        match self.data.m_cycle {
            M2 => Some(BusOp::Write(self.data.bc(), self.data.a)),
            M3 => self.execute_m1(),
            _ => unreachable!(),
        }
    }

    fn ld_deref_de_a(&mut self) -> Option<BusOp> {
        match self.data.m_cycle {
            M2 => Some(BusOp::Write(self.data.de(), self.data.a)),
            M3 => self.execute_m1(),
            _ => unreachable!(),
        }
    }

    fn ld_deref_hli_a(&mut self) -> Option<BusOp> {
        match self.data.m_cycle {
            M2 => {
                let hl = self.data.hl();
                let incremented_hl = hl.wrapping_add(1);
                self.data.h = high_byte(incremented_hl);
                self.data.l = low_byte(incremented_hl);
                Some(BusOp::Write(hl, self.data.a))
            }
            M3 => self.execute_m1(),
            _ => unreachable!(),
        }
    }

    fn ld_deref_hld_a(&mut self) -> Option<BusOp> {
        match self.data.m_cycle {
            M2 => {
                let hl = self.data.hl();
                let decremented_hl = hl - 1;
                self.data.h = high_byte(decremented_hl);
                self.data.l = low_byte(decremented_hl);
                Some(BusOp::Write(hl, self.data.a))
            }
            M3 => self.execute_m1(),
            _ => unreachable!(),
        }
    }

    fn ld_dd_nn(&mut self, dd: Dd) -> Option<BusOp> {
        match self.data.m_cycle {
            M2 => self.read_immediate(),
            M3 => {
                self.data.write(dd.low(), self.state.bus_data.unwrap());
                self.read_immediate()
            }
            M4 => {
                self.data.write(dd.high(), self.state.bus_data.unwrap());
                self.execute_m1()
            }
            _ => unreachable!(),
        }
    }

    fn ld_sp_hl(&mut self) -> Option<BusOp> {
        match self.data.m_cycle {
            M2 => {
                self.data.sp = self.data.hl();
                None
            }
            M3 => self.execute_m1(),
            _ => unreachable!(),
        }
    }

    fn push_qq(&mut self, qq: Qq) -> Option<BusOp> {
        match self.data.m_cycle {
            M2 => None,
            M3 => self.push_byte(self.data.read(qq.high())),
            M4 => self.push_byte(self.data.read(qq.low())),
            M5 => self.execute_m1(),
            _ => unreachable!(),
        }
    }

    fn pop_qq(&mut self, qq: Qq) -> Option<BusOp> {
        match self.data.m_cycle {
            M2 => self.pop_byte(),
            M3 => {
                self.data.write(qq.low(), self.state.bus_data.unwrap());
                self.pop_byte()
            }
            M4 => {
                self.data.write(qq.high(), self.state.bus_data.unwrap());
                self.execute_m1()
            }
            _ => unreachable!(),
        }
    }

    fn ldhl_sp_e(&mut self) -> Option<BusOp> {
        match self.data.m_cycle {
            M2 => self.read_immediate(),
            M3 => {
                let e = self.state.bus_data.unwrap();
                let (l, flags) = alu::add(low_byte(self.data.sp), e, false);
                let (h, _) = alu::add(high_byte(self.data.sp), sign_extension(e), flags.cy);
                self.data.h = h;
                self.data.l = l;
                self.data.f = flags;
                self.data.f.z = false;
                None
            }
            M4 => self.execute_m1(),
            _ => unreachable!(),
        }
    }

    fn ld_deref_nn_sp(&mut self) -> Option<BusOp> {
        match self.data.m_cycle {
            M2 => self.read_immediate(),
            M3 => {
                self.state.addr = self.state.bus_data.unwrap() as u16;
                self.read_immediate()
            }
            M4 => {
                self.state.addr |= (self.state.bus_data.unwrap() as u16) << 8;
                Some(BusOp::Write(self.state.addr, low_byte(self.data.sp)))
            }
            M5 => Some(BusOp::Write(self.state.addr + 1, high_byte(self.data.sp))),
            M6 => self.execute_m1(),
            _ => unreachable!(),
        }
    }

    fn alu_op_r(&mut self, op: AluOp, r: R) -> Option<BusOp> {
        match self.data.m_cycle {
            M2 => {
                let (result, flags) = self.alu_op(op, self.data.a, self.data.read(r));
                self.data.a = result;
                self.data.f = flags;
                self.execute_m1()
            }
            _ => unreachable!(),
        }
    }

    fn alu_op_n(&mut self, op: AluOp) -> Option<BusOp> {
        match self.data.m_cycle {
            M2 => self.read_immediate(),
            M3 => {
                let (result, flags) = self.alu_op(op, self.data.a, self.state.bus_data.unwrap());
                self.data.a = result;
                self.data.f = flags;
                self.execute_m1()
            }
            _ => unreachable!(),
        }
    }

    fn alu_op_deref_hl(&mut self, op: AluOp) -> Option<BusOp> {
        match self.data.m_cycle {
            M2 => Some(BusOp::Read(self.data.hl())),
            M3 => {
                let (result, flags) = self.alu_op(op, self.data.a, self.state.bus_data.unwrap());
                self.data.a = result;
                self.data.f = flags;
                self.execute_m1()
            }
            _ => unreachable!(),
        }
    }

    fn inc_r(&mut self, r: R) -> Option<BusOp> {
        match self.data.m_cycle {
            M2 => {
                let (result, flags) = alu::add(self.data.read(r), 1, false);
                self.data.write(r, result);
                self.data.f.z = flags.z;
                self.data.f.n = flags.n;
                self.data.f.h = flags.h;
                self.execute_m1()
            }
            _ => unreachable!(),
        }
    }

    fn inc_deref_hl(&mut self) -> Option<BusOp> {
        match self.data.m_cycle {
            M2 => Some(BusOp::Read(self.data.hl())),
            M3 => {
                let (result, flags) = alu::add(self.state.bus_data.unwrap(), 1, false);
                self.data.f.z = flags.z;
                self.data.f.n = flags.n;
                self.data.f.h = flags.h;
                Some(BusOp::Write(self.data.hl(), result))
            }
            M4 => self.execute_m1(),
            _ => unreachable!(),
        }
    }

    fn jp_nn(&mut self) -> Option<BusOp> {
        match self.data.m_cycle {
            M2 => self.read_immediate(),
            M3 => {
                self.state.data = self.state.bus_data.unwrap();
                self.read_immediate()
            }
            M4 => {
                self.data.pc = u16::from_be_bytes([self.state.bus_data.unwrap(), self.state.data]);
                None
            }
            M5 => self.execute_m1(),
            _ => unreachable!(),
        }
    }

    fn jp_cc_nn(&mut self, cc: Cc) -> Option<BusOp> {
        match self.data.m_cycle {
            M2 => self.read_immediate(),
            M3 => {
                self.state.data = self.state.bus_data.unwrap();
                self.read_immediate()
            }
            M4 => {
                if self.evaluate_condition(cc) {
                    self.data.pc =
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

    fn jr_e(&mut self) -> Option<BusOp> {
        match self.data.m_cycle {
            M2 => self.read_immediate(),
            M3 => {
                let e = self.state.bus_data.unwrap() as i8;
                self.data.pc = self.data.pc.wrapping_add(e as i16 as u16);
                None
            }
            M4 => self.execute_m1(),
            _ => unreachable!(),
        }
    }

    fn jp_deref_hl(&mut self) -> Option<BusOp> {
        match self.data.m_cycle {
            M2 => {
                self.data.pc = self.data.hl();
                self.execute_m1()
            }
            _ => unreachable!(),
        }
    }

    fn ret(&mut self) -> Option<BusOp> {
        match self.data.m_cycle {
            M2 => self.pop_byte(),
            M3 => {
                self.state.data = self.state.bus_data.unwrap();
                self.pop_byte()
            }
            M4 => {
                self.data.pc = u16::from_be_bytes([self.state.bus_data.unwrap(), self.state.data]);
                None
            }
            M5 => self.execute_m1(),
            _ => unreachable!(),
        }
    }

    fn execute_m1(&mut self) -> Option<BusOp> {
        self.state.m1 = true;
        Some(BusOp::Read(self.data.pc))
    }

    fn read_immediate(&mut self) -> CpuOutput {
        let pc = self.data.pc;
        self.data.pc += 1;
        Some(BusOp::Read(pc))
    }

    fn push_byte(&mut self, data: u8) -> CpuOutput {
        self.data.sp -= 1;
        Some(BusOp::Write(self.data.sp, data))
    }

    fn pop_byte(&mut self) -> CpuOutput {
        let sp = self.data.sp;
        self.data.sp += 1;
        Some(BusOp::Read(sp))
    }

    fn alu_op(&self, op: AluOp, lhs: u8, rhs: u8) -> (u8, Flags) {
        match op {
            AluOp::Add => alu::add(lhs, rhs, false),
            AluOp::Adc => alu::add(lhs, rhs, self.data.f.cy),
            AluOp::Sub => alu::sub(lhs, rhs, false),
            AluOp::Sbc => alu::sub(lhs, rhs, self.data.f.cy),
            AluOp::And => alu::and(lhs, rhs),
            AluOp::Xor => alu::xor(lhs, rhs),
            AluOp::Or => alu::or(lhs, rhs),
            AluOp::Cp => {
                let (_, flags) = alu::sub(lhs, rhs, false);
                (lhs, flags)
            }
        }
    }

    fn evaluate_condition(&self, cc: Cc) -> bool {
        match cc {
            Cc::Nz => !self.data.f.z,
            Cc::Z => self.data.f.z,
            Cc::Nc => !self.data.f.cy,
            Cc::C => self.data.f.cy,
        }
    }
}
