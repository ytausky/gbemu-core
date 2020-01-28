use super::*;

pub(super) struct Microinstruction {
    data_select: DataSelect,
    word_select: WordSelect,
    byte_writeback: Option<ByteWriteback>,
    word_writeback: Option<WordWriteback>,
    bus_op_select: Option<BusOpSelect>,
    write_opcode: bool,
}

pub(super) enum DataSelect {
    R(R),
    SpH,
    SpL,
}

pub(super) enum WordSelect {
    Bc,
    De,
    Hl,
    Pc,
    Sp,
    AddrBuffer,
    C,
    DataBuf,
}

struct ByteWriteback {
    dest: ByteWritebackDest,
    src: ByteWritebackSrc,
}

pub(super) enum ByteWritebackDest {
    R(R),
    SpH,
    SpL,
    DataBuf,
    AddrH,
    AddrL,
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
    AddrBuffer,
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
            data_select: DataSelect::R(R::A),
            word_select: WordSelect::Pc,
            byte_writeback: None,
            word_writeback: None,
            bus_op_select: None,
            write_opcode: false,
        }
    }
}

impl Microinstruction {
    pub(super) fn read_immediate(&mut self) -> &mut Self {
        self.word_select = WordSelect::Pc;
        self.word_writeback = Some(WordWriteback {
            dest: WordWritebackDest::Pc,
            src: WordWritebackSrc::Inc,
        });
        self.bus_op_select = Some(BusOpSelect::Read);
        self
    }

    pub(super) fn pop_byte(&mut self) -> &mut Self {
        self.word_select = WordSelect::Sp;
        self.word_writeback = Some(WordWriteback {
            dest: WordWritebackDest::Sp,
            src: WordWritebackSrc::Inc,
        });
        self.bus_op_select = Some(BusOpSelect::Read);
        self
    }

    pub(super) fn bus_read(&mut self, addr_sel: WordSelect) -> &mut Self {
        self.bus_op_select = Some(BusOpSelect::Read);
        self.select_addr(addr_sel)
    }

    pub(super) fn bus_write(&mut self, addr: WordSelect, data: DataSelect) -> &mut Self {
        self.bus_op_select = Some(BusOpSelect::Write);
        self.select_addr(addr).select_data(data)
    }

    pub(super) fn select_data(&mut self, selector: DataSelect) -> &mut Self {
        self.data_select = selector;
        self
    }

    pub(super) fn select_addr(&mut self, selector: WordSelect) -> &mut Self {
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

    pub(super) fn write_a(&mut self) -> &mut Self {
        self.byte_writeback = Some(ByteWriteback {
            dest: ByteWritebackDest::R(R::A),
            src: ByteWritebackSrc::Bus,
        });
        self
    }

    pub(super) fn write_data_buf(&mut self) -> &mut Self {
        self.byte_writeback = Some(ByteWriteback {
            dest: ByteWritebackDest::DataBuf,
            src: ByteWritebackSrc::Bus,
        });
        self
    }

    pub(super) fn write_addr_l(&mut self) -> &mut Self {
        self.byte_writeback = Some(ByteWriteback {
            dest: ByteWritebackDest::AddrL,
            src: ByteWritebackSrc::Bus,
        });
        self
    }

    pub(super) fn write_addr_h(&mut self) -> &mut Self {
        self.byte_writeback = Some(ByteWriteback {
            dest: ByteWritebackDest::AddrH,
            src: ByteWritebackSrc::Bus,
        });
        self
    }

    pub(super) fn data_writeback(&mut self, dest: ByteWritebackDest) -> &mut Self {
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
        self.word_select = WordSelect::Pc;
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
            DataSelect::R(r) => *self.regs.select_r(r),
            DataSelect::SpH => high_byte(self.regs.sp),
            DataSelect::SpL => low_byte(self.regs.sp),
        };
        let addr = match microinstruction.word_select {
            WordSelect::Bc => self.regs.bc(),
            WordSelect::De => self.regs.de(),
            WordSelect::Hl => self.regs.hl(),
            WordSelect::Pc => self.regs.pc,
            WordSelect::Sp => self.regs.sp,
            WordSelect::AddrBuffer => self.state.addr,
            WordSelect::C => 0xff00 | u16::from(self.regs.c),
            WordSelect::DataBuf => 0xff00 | u16::from(self.state.data),
        };

        if *self.phase == Tock {
            if let Some(byte_writeback) = &microinstruction.byte_writeback {
                let byte = match byte_writeback.src {
                    ByteWritebackSrc::Bus => self.input.data.unwrap(),
                };
                match byte_writeback.dest {
                    ByteWritebackDest::R(r) => *self.regs.select_r_mut(r) = byte,
                    ByteWritebackDest::SpH => {
                        self.regs.sp = self.regs.sp & 0x00ff | u16::from(byte) << 8
                    }
                    ByteWritebackDest::SpL => {
                        self.regs.sp = self.regs.sp & 0xff00 | u16::from(byte)
                    }
                    ByteWritebackDest::DataBuf => self.state.data = byte,
                    ByteWritebackDest::AddrH => {
                        self.state.addr = self.state.addr & 0x00ff | u16::from(byte) << 8
                    }
                    ByteWritebackDest::AddrL => {
                        self.state.addr = self.state.addr & 0xff00 | u16::from(byte)
                    }
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
                    WordWritebackDest::AddrBuffer => self.state.addr = word,
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
    pub(super) fn low(self) -> ByteWritebackDest {
        match self {
            Dd::Bc => ByteWritebackDest::R(R::C),
            Dd::De => ByteWritebackDest::R(R::E),
            Dd::Hl => ByteWritebackDest::R(R::L),
            Dd::Sp => ByteWritebackDest::SpL,
        }
    }

    pub(super) fn high(self) -> ByteWritebackDest {
        match self {
            Dd::Bc => ByteWritebackDest::R(R::B),
            Dd::De => ByteWritebackDest::R(R::D),
            Dd::Hl => ByteWritebackDest::R(R::H),
            Dd::Sp => ByteWritebackDest::SpH,
        }
    }
}
