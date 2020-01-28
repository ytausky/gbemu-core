use super::R;

pub struct Microinstruction {
    pub(super) data_select: DataSelect,
    pub word_select: WordSelect,
    pub byte_writeback: Option<ByteWriteback>,
    pub word_writeback: Option<WordWriteback>,
    pub bus_op_select: Option<BusOpSelect>,
    pub write_opcode: bool,
}

pub(super) enum DataSelect {
    R(R),
    SpH,
    SpL,
}

pub enum WordSelect {
    AddrBuffer,
    Pc,
    Sp,
}

pub struct ByteWriteback {
    pub dest: ByteWritebackDest,
    pub src: ByteWritebackSrc,
}

pub enum ByteWritebackDest {
    AddrH,
    AddrL,
}

pub enum ByteWritebackSrc {
    Bus,
}

pub struct WordWriteback {
    pub dest: WordWritebackDest,
    pub src: WordWritebackSrc,
}

pub enum WordWritebackDest {
    AddrBuffer,
    Pc,
    Sp,
}

pub enum WordWritebackSrc {
    Addr,
    Inc,
}

#[derive(Clone, Copy, PartialEq)]
pub enum BusOpSelect {
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
    pub fn read_immediate(&mut self) -> &mut Self {
        self.word_select = WordSelect::Pc;
        self.word_writeback = Some(WordWriteback {
            dest: WordWritebackDest::Pc,
            src: WordWritebackSrc::Inc,
        });
        self.bus_op_select = Some(BusOpSelect::Read);
        self
    }

    pub fn pop_byte(&mut self) -> &mut Self {
        self.word_select = WordSelect::Sp;
        self.word_writeback = Some(WordWriteback {
            dest: WordWritebackDest::Sp,
            src: WordWritebackSrc::Inc,
        });
        self.bus_op_select = Some(BusOpSelect::Read);
        self
    }

    pub(super) fn bus_write(&mut self, addr: WordSelect, data: DataSelect) -> &mut Self {
        self.bus_op_select = Some(BusOpSelect::Write);
        self.select_addr(addr).select_data(data)
    }

    pub(super) fn select_data(&mut self, selector: DataSelect) -> &mut Self {
        self.data_select = selector;
        self
    }

    pub fn select_addr(&mut self, selector: WordSelect) -> &mut Self {
        self.word_select = selector;
        self
    }

    pub fn increment(&mut self, dest: WordWritebackDest) -> &mut Self {
        self.word_writeback = Some(WordWriteback {
            dest,
            src: WordWritebackSrc::Inc,
        });
        self
    }

    pub fn write_addr_l(&mut self) -> &mut Self {
        self.byte_writeback = Some(ByteWriteback {
            dest: ByteWritebackDest::AddrL,
            src: ByteWritebackSrc::Bus,
        });
        self
    }

    pub fn write_addr_h(&mut self) -> &mut Self {
        self.byte_writeback = Some(ByteWriteback {
            dest: ByteWritebackDest::AddrH,
            src: ByteWritebackSrc::Bus,
        });
        self
    }

    pub fn write_pc(&mut self) -> &mut Self {
        self.word_writeback = Some(WordWriteback {
            dest: WordWritebackDest::Pc,
            src: WordWritebackSrc::Addr,
        });
        self
    }

    pub fn fetch(&mut self) -> &mut Self {
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
