use super::*;

pub(super) struct Microinstruction {
    data_select: DataSel,
    word_select: AddrSel,
    byte_writeback: Option<ByteWriteback>,
    word_writeback: Option<WordWriteback>,
    bus_op_select: Option<BusOpSelect>,
    write_opcode: bool,
}

pub(super) enum DataSel {
    R(R),
    F,
    SpH,
    SpL,
    DataBuf,
    AddrH,
    AddrL,
}

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

struct ByteWriteback {
    dest: DataSel,
    src: ByteWritebackSrc,
}

enum ByteWritebackSrc {
    Bus,
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
            byte_writeback: None,
            word_writeback: None,
            bus_op_select: None,
            write_opcode: false,
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

    pub(super) fn write_data(&mut self, dest: DataSel) -> &mut Self {
        self.byte_writeback = Some(ByteWriteback {
            dest,
            src: ByteWritebackSrc::Bus,
        });
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
        self.word_select = AddrSel::Pc;
        self.word_writeback = Some(WordWriteback {
            dest: WordWritebackDest::Pc,
            src: WordWritebackSrc::Inc,
        });
        self.bus_op_select = Some(BusOpSelect::Read);
        self.write_opcode = true;
        self
    }
}

impl<'a> InstrExecution<'a> {
    pub(super) fn execute_microinstruction(
        &mut self,
        microinstruction: &Microinstruction,
    ) -> CpuOutput {
        let data = match microinstruction.data_select {
            DataSel::R(r) => *self.regs.select_r(r),
            DataSel::F => self.regs.f.into(),
            DataSel::SpH => high_byte(self.regs.sp),
            DataSel::SpL => low_byte(self.regs.sp),
            DataSel::DataBuf => self.state.data,
            DataSel::AddrH => high_byte(self.state.addr),
            DataSel::AddrL => low_byte(self.state.addr),
        };
        let addr = match microinstruction.word_select {
            AddrSel::Bc => self.regs.bc(),
            AddrSel::De => self.regs.de(),
            AddrSel::Hl => self.regs.hl(),
            AddrSel::Pc => self.regs.pc,
            AddrSel::Sp => self.regs.sp,
            AddrSel::AddrBuf => self.state.addr,
            AddrSel::C => 0xff00 | u16::from(self.regs.c),
            AddrSel::DataBuf => 0xff00 | u16::from(self.state.data),
        };

        if *self.phase == Tock {
            if let Some(byte_writeback) = &microinstruction.byte_writeback {
                let byte = match byte_writeback.src {
                    ByteWritebackSrc::Bus => self.input.data.unwrap(),
                };
                match byte_writeback.dest {
                    DataSel::R(r) => *self.regs.select_r_mut(r) = byte,
                    DataSel::F => self.regs.f = byte.into(),
                    DataSel::SpH => self.regs.sp = self.regs.sp & 0x00ff | u16::from(byte) << 8,
                    DataSel::SpL => self.regs.sp = self.regs.sp & 0xff00 | u16::from(byte),
                    DataSel::DataBuf => self.state.data = byte,
                    DataSel::AddrH => {
                        self.state.addr = self.state.addr & 0x00ff | u16::from(byte) << 8
                    }
                    DataSel::AddrL => self.state.addr = self.state.addr & 0xff00 | u16::from(byte),
                }
            }
            if let Some(word_writeback) = &microinstruction.word_writeback {
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

            if microinstruction.write_opcode {
                self.mode_transition = Some(ModeTransition::Run(Opcode(self.input.data.unwrap())))
            }
        }

        microinstruction
            .bus_op_select
            .map(|op| match op {
                BusOpSelect::Read => BusOp::Read(addr),
                BusOpSelect::Write => BusOp::Write(addr, data),
            })
            .and_then(|op| if *self.phase == Tick { Some(op) } else { None })
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
