use super::*;

pub(super) struct Microinstruction {
    data_select: DataSel,
    word_select: AddrSel,
    computation: Option<Computation>,
    byte_writeback: Option<ByteWriteback>,
    word_writeback: Option<WordWriteback>,
    reset_z: bool,
    flag_mask: Flags,
    bus_op_select: Option<BusOpSelect>,
    condition: Option<Cc>,
    fetch: bool,
}

#[derive(Clone, Copy)]
pub(super) enum DataSel {
    R(R),
    F,
    SpH,
    SpL,
    DataBuf,
    AddrH,
    AddrL,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) enum AddrSel {
    Bc,
    De,
    Hl,
    Pc,
    Sp,
    AddrBuf,
    C,
    DataBuf,
}

enum Computation {
    Alu(AluComputation),
}

struct AluComputation {
    op: AluOp,
    lhs: AluOperand,
    rhs: AluOperand,
}

#[derive(Clone, Copy)]
pub(super) enum AluOperand {
    A,
    Bus,
    Data,
    One,
    SignExtension,
    SpH,
    SpL,
}

struct ByteWriteback {
    dest: DataSel,
    src: ByteWritebackSrc,
}

pub(super) enum ByteWritebackSrc {
    Bus,
    Computation,
    Data,
}

struct WordWriteback {
    dest: WordWritebackDest,
    src: WordWritebackSrc,
}

pub(super) enum WordWritebackDest {
    Hl,
    Pc,
    Sp,
    AddrBuf,
}

enum WordWritebackSrc {
    Addr,
    Inc,
    Dec,
}

#[derive(Clone, Copy, PartialEq)]
enum BusOpSelect {
    Read,
    Write,
}

impl Default for Microinstruction {
    fn default() -> Self {
        Self {
            data_select: DataSel::R(R::A),
            word_select: AddrSel::Pc,
            computation: None,
            byte_writeback: None,
            word_writeback: None,
            reset_z: false,
            flag_mask: Default::default(),
            bus_op_select: None,
            condition: None,
            fetch: false,
        }
    }
}

impl Microinstruction {
    pub(super) fn read_immediate(&mut self) -> &mut Self {
        self.word_select = AddrSel::Pc;
        self.word_writeback = Some(WordWriteback {
            dest: WordWritebackDest::Pc,
            src: WordWritebackSrc::Inc,
        });
        self.bus_op_select = Some(BusOpSelect::Read);
        self
    }

    pub(super) fn pop_byte(&mut self) -> &mut Self {
        self.word_select = AddrSel::Sp;
        self.word_writeback = Some(WordWriteback {
            dest: WordWritebackDest::Sp,
            src: WordWritebackSrc::Inc,
        });
        self.bus_op_select = Some(BusOpSelect::Read);
        self
    }

    pub(super) fn bus_read(&mut self, addr_sel: AddrSel) -> &mut Self {
        self.bus_op_select = Some(BusOpSelect::Read);
        self.select_addr(addr_sel)
    }

    pub(super) fn bus_write(&mut self, addr: AddrSel, data: DataSel) -> &mut Self {
        self.bus_op_select = Some(BusOpSelect::Write);
        self.select_addr(addr).select_data(data)
    }

    pub(super) fn select_data(&mut self, selector: DataSel) -> &mut Self {
        self.data_select = selector;
        self
    }

    pub(super) fn select_addr(&mut self, selector: AddrSel) -> &mut Self {
        self.word_select = selector;
        self
    }

    pub(super) fn alu_op(&mut self, op: AluOp, lhs: AluOperand, rhs: AluOperand) -> &mut Self {
        self.computation = Some(Computation::Alu(AluComputation { op, lhs, rhs }));
        self
    }

    pub(super) fn write_result(&mut self, dest: DataSel) -> &mut Self {
        self.byte_writeback = Some(ByteWriteback {
            dest,
            src: ByteWritebackSrc::Computation,
        });
        self
    }

    pub(super) fn write_flags(&mut self, mask: Flags) -> &mut Self {
        self.flag_mask = mask;
        self
    }

    pub(super) fn reset_z(&mut self) -> &mut Self {
        self.reset_z = true;
        self
    }

    pub(super) fn increment(&mut self, dest: WordWritebackDest) -> &mut Self {
        self.word_writeback = Some(WordWriteback {
            dest,
            src: WordWritebackSrc::Inc,
        });
        self
    }

    pub(super) fn decrement(&mut self, dest: WordWritebackDest) -> &mut Self {
        self.word_writeback = Some(WordWriteback {
            dest,
            src: WordWritebackSrc::Dec,
        });
        self
    }

    pub(super) fn write_data(&mut self, dest: DataSel, src: ByteWritebackSrc) -> &mut Self {
        self.byte_writeback = Some(ByteWriteback { dest, src });
        self
    }

    pub(super) fn write_pc(&mut self) -> &mut Self {
        self.word_writeback = Some(WordWriteback {
            dest: WordWritebackDest::Pc,
            src: WordWritebackSrc::Addr,
        });
        self
    }

    pub(super) fn fetch(&mut self) -> &mut Self {
        self.fetch = true;
        self
    }

    pub(super) fn fetch_if_not(&mut self, cc: Cc) -> &mut Self {
        self.condition = Some(cc);
        self
    }
}

impl<'a> InstrExecution<'a> {
    pub(super) fn execute_microinstruction(
        &mut self,
        microinstruction: &Microinstruction,
    ) -> CpuOutput {
        match *self.phase {
            Tick => self.execute_microinstruction_on_tick(microinstruction),
            Tock => self.execute_microinstruction_on_tock(microinstruction),
        }
    }

    fn execute_microinstruction_on_tick(
        &mut self,
        microinstruction: &Microinstruction,
    ) -> CpuOutput {
        let (data, addr) = (
            self.data(microinstruction.data_select),
            self.addr(microinstruction.word_select),
        );
        let effective_bus_op_sel = if self.should_fetch(microinstruction) {
            Some(BusOpSelect::Read)
        } else {
            microinstruction.bus_op_select
        };
        effective_bus_op_sel.map(|op| match op {
            BusOpSelect::Read => BusOp::Read(addr),
            BusOpSelect::Write => BusOp::Write(addr, data),
        })
    }

    fn execute_microinstruction_on_tock(
        &mut self,
        microinstruction: &Microinstruction,
    ) -> CpuOutput {
        let data = self.data(microinstruction.data_select);
        let mut updated_flags = Flags::default();
        let result = microinstruction.computation.as_ref().map(|computation| {
            let (result, flags) = self.perform_computation(computation, data);
            updated_flags = flags;
            result
        });

        if let Some(byte_writeback) = &microinstruction.byte_writeback {
            let byte = match byte_writeback.src {
                ByteWritebackSrc::Bus => self.input.data.unwrap(),
                ByteWritebackSrc::Computation => result.unwrap(),
                ByteWritebackSrc::Data => data,
            };
            match byte_writeback.dest {
                DataSel::R(r) => *self.regs.select_r_mut(r) = byte,
                DataSel::F => self.regs.f = byte.into(),
                DataSel::SpH => self.regs.sp = self.regs.sp & 0x00ff | u16::from(byte) << 8,
                DataSel::SpL => self.regs.sp = self.regs.sp & 0xff00 | u16::from(byte),
                DataSel::DataBuf => self.state.data = byte,
                DataSel::AddrH => self.state.addr = self.state.addr & 0x00ff | u16::from(byte) << 8,
                DataSel::AddrL => self.state.addr = self.state.addr & 0xff00 | u16::from(byte),
            }
        }

        let fetch = self.should_fetch(microinstruction);
        let effective_word_writeback = if fetch {
            &Some(WordWriteback {
                dest: WordWritebackDest::Pc,
                src: WordWritebackSrc::Inc,
            })
        } else {
            &microinstruction.word_writeback
        };

        if let Some(word_writeback) = effective_word_writeback {
            let addr = self.addr(microinstruction.word_select);
            let word = match word_writeback.src {
                WordWritebackSrc::Addr => self.state.addr,
                WordWritebackSrc::Inc => addr.wrapping_add(1),
                WordWritebackSrc::Dec => addr - 1,
            };
            match word_writeback.dest {
                WordWritebackDest::Hl => {
                    self.regs.h = high_byte(word);
                    self.regs.l = low_byte(word)
                }
                WordWritebackDest::Pc => self.regs.pc = word,
                WordWritebackDest::Sp => self.regs.sp = word,
                WordWritebackDest::AddrBuf => self.state.addr = word,
            }
        }

        if microinstruction.reset_z {
            updated_flags.z = false;
        }
        self.regs.f =
            self.regs.f & !microinstruction.flag_mask | updated_flags & microinstruction.flag_mask;

        if fetch {
            self.mode_transition = Some(ModeTransition::Run(Opcode(self.input.data.unwrap())))
        }

        None
    }

    fn data(&self, data_sel: DataSel) -> u8 {
        match data_sel {
            DataSel::R(r) => *self.regs.select_r(r),
            DataSel::F => self.regs.f.into(),
            DataSel::SpH => high_byte(self.regs.sp),
            DataSel::SpL => low_byte(self.regs.sp),
            DataSel::DataBuf => self.state.data,
            DataSel::AddrH => high_byte(self.state.addr),
            DataSel::AddrL => low_byte(self.state.addr),
        }
    }

    fn addr(&self, addr_sel: AddrSel) -> u16 {
        match addr_sel {
            AddrSel::Bc => self.regs.bc(),
            AddrSel::De => self.regs.de(),
            AddrSel::Hl => self.regs.hl(),
            AddrSel::Pc => self.regs.pc,
            AddrSel::Sp => self.regs.sp,
            AddrSel::AddrBuf => self.state.addr,
            AddrSel::C => 0xff00 | u16::from(self.regs.c),
            AddrSel::DataBuf => 0xff00 | u16::from(self.state.data),
        }
    }

    fn should_fetch(&self, microinstruction: &Microinstruction) -> bool {
        microinstruction
            .condition
            .map(|cc| !self.evaluate_condition(cc))
            .unwrap_or(microinstruction.fetch)
    }

    fn evaluate_condition(&self, cc: Cc) -> bool {
        match cc {
            Cc::Nz => !self.regs.f.z,
            Cc::Z => self.regs.f.z,
            Cc::Nc => !self.regs.f.cy,
            Cc::C => self.regs.f.cy,
        }
    }

    fn perform_computation(&self, computation: &Computation, data: u8) -> (u8, Flags) {
        match computation {
            Computation::Alu(computation) => self.perform_alu_computation(computation, data),
        }
    }

    fn perform_alu_computation(&self, computation: &AluComputation, data: u8) -> (u8, Flags) {
        let lhs = self.alu_operand(computation.lhs, data);
        let rhs = self.alu_operand(computation.rhs, data);
        self.alu_op(computation.op, lhs, rhs)
    }

    fn alu_operand(&self, operand: AluOperand, data: u8) -> u8 {
        match operand {
            AluOperand::A => self.regs.a,
            AluOperand::Bus => self.input.data.unwrap(),
            AluOperand::Data => data,
            AluOperand::One => 0x01,
            AluOperand::SignExtension => sign_extension(data),
            AluOperand::SpH => high_byte(self.regs.sp),
            AluOperand::SpL => low_byte(self.regs.sp),
        }
    }

    fn alu_op(&self, op: AluOp, lhs: u8, rhs: u8) -> (u8, Flags) {
        match op {
            AluOp::Add => alu::add(lhs, rhs, false),
            AluOp::Adc => alu::add(lhs, rhs, self.regs.f.cy),
            AluOp::Sub => alu::sub(lhs, rhs, false),
            AluOp::Sbc => alu::sub(lhs, rhs, self.regs.f.cy),
            AluOp::And => alu::and(lhs, rhs),
            AluOp::Xor => alu::xor(lhs, rhs),
            AluOp::Or => alu::or(lhs, rhs),
            AluOp::Cp => {
                let (_, flags) = alu::sub(lhs, rhs, false);
                (lhs, flags)
            }
        }
    }
}

impl Dd {
    pub(super) fn high(self) -> DataSel {
        match self {
            Dd::Bc => DataSel::R(R::B),
            Dd::De => DataSel::R(R::D),
            Dd::Hl => DataSel::R(R::H),
            Dd::Sp => DataSel::SpH,
        }
    }

    pub(super) fn low(self) -> DataSel {
        match self {
            Dd::Bc => DataSel::R(R::C),
            Dd::De => DataSel::R(R::E),
            Dd::Hl => DataSel::R(R::L),
            Dd::Sp => DataSel::SpL,
        }
    }
}

impl Qq {
    pub(super) fn high(self) -> DataSel {
        match self {
            Qq::Bc => DataSel::R(R::B),
            Qq::De => DataSel::R(R::D),
            Qq::Hl => DataSel::R(R::H),
            Qq::Af => DataSel::R(R::A),
        }
    }

    pub(super) fn low(self) -> DataSel {
        match self {
            Qq::Bc => DataSel::R(R::C),
            Qq::De => DataSel::R(R::E),
            Qq::Hl => DataSel::R(R::L),
            Qq::Af => DataSel::F,
        }
    }
}
