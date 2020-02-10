use super::{Dd, Qq, R};

use std::ops::{BitAnd, BitOr, Not};

#[derive(Default)]
pub struct Regs {
    pub a: u8,
    pub f: Flags,
    pub b: u8,
    pub c: u8,
    pub d: u8,
    pub e: u8,
    pub h: u8,
    pub l: u8,
    pub pc: u16,
    pub sp: u16,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Flags {
    pub z: bool,
    pub n: bool,
    pub h: bool,
    pub cy: bool,
}

pub(super) enum RegSelect {
    R(R),
    F,
    SpH,
    SpL,
}

impl Regs {
    pub(super) fn bc(&self) -> u16 {
        self.pair(R::B, R::C)
    }

    pub(super) fn de(&self) -> u16 {
        self.pair(R::D, R::E)
    }

    pub(super) fn hl(&self) -> u16 {
        self.pair(R::H, R::L)
    }

    fn pair(&self, h: R, l: R) -> u16 {
        u16::from_be_bytes([self.read(h), self.read(l)])
    }

    pub(super) fn read(&self, reg_select: impl Into<RegSelect>) -> u8 {
        match reg_select.into() {
            RegSelect::R(R::A) => self.a,
            RegSelect::R(R::B) => self.b,
            RegSelect::R(R::C) => self.c,
            RegSelect::R(R::D) => self.d,
            RegSelect::R(R::E) => self.e,
            RegSelect::R(R::H) => self.h,
            RegSelect::R(R::L) => self.l,
            RegSelect::F => self.f.into(),
            RegSelect::SpH => (self.sp >> 8) as u8,
            RegSelect::SpL => (self.sp & 0x00ff) as u8,
        }
    }

    pub(super) fn write(&mut self, reg_select: impl Into<RegSelect>, data: u8) {
        match reg_select.into() {
            RegSelect::R(R::A) => self.a = data,
            RegSelect::R(R::B) => self.b = data,
            RegSelect::R(R::C) => self.c = data,
            RegSelect::R(R::D) => self.d = data,
            RegSelect::R(R::E) => self.e = data,
            RegSelect::R(R::H) => self.h = data,
            RegSelect::R(R::L) => self.l = data,
            RegSelect::F => self.f = data.into(),
            RegSelect::SpH => self.sp = self.sp & 0x00ff | u16::from(data) << 8,
            RegSelect::SpL => self.sp = self.sp & 0xff00 | u16::from(data),
        }
    }
}

impl From<Flags> for u8 {
    fn from(flags: Flags) -> Self {
        (if flags.z { 0x80 } else { 0x00 })
            | if flags.n { 0x40 } else { 0x00 }
            | if flags.h { 0x20 } else { 0x00 }
            | if flags.cy { 0x10 } else { 0x00 }
    }
}

impl From<u8> for Flags {
    fn from(flags: u8) -> Self {
        Flags {
            z: flags & 0x80 > 0,
            n: flags & 0x40 > 0,
            h: flags & 0x20 > 0,
            cy: flags & 0x10 > 0,
        }
    }
}

impl BitAnd for Flags {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        Flags {
            z: self.z & rhs.z,
            n: self.n & rhs.n,
            h: self.h & rhs.h,
            cy: self.cy & rhs.cy,
        }
    }
}

impl BitOr for Flags {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Flags {
            z: self.z | rhs.z,
            n: self.n | rhs.n,
            h: self.h | rhs.h,
            cy: self.cy | rhs.cy,
        }
    }
}

impl Not for Flags {
    type Output = Self;

    fn not(self) -> Self::Output {
        Flags {
            z: !self.z,
            n: !self.n,
            h: !self.h,
            cy: !self.cy,
        }
    }
}

impl From<R> for RegSelect {
    fn from(r: R) -> Self {
        RegSelect::R(r)
    }
}

impl Dd {
    pub(super) fn high(self) -> RegSelect {
        match self {
            Dd::Bc => RegSelect::R(R::B),
            Dd::De => RegSelect::R(R::D),
            Dd::Hl => RegSelect::R(R::H),
            Dd::Sp => RegSelect::SpH,
        }
    }

    pub(super) fn low(self) -> RegSelect {
        match self {
            Dd::Bc => RegSelect::R(R::C),
            Dd::De => RegSelect::R(R::E),
            Dd::Hl => RegSelect::R(R::L),
            Dd::Sp => RegSelect::SpL,
        }
    }
}

impl Qq {
    pub(super) fn high(self) -> RegSelect {
        match self {
            Qq::Bc => RegSelect::R(R::B),
            Qq::De => RegSelect::R(R::D),
            Qq::Hl => RegSelect::R(R::H),
            Qq::Af => RegSelect::R(R::A),
        }
    }

    pub(super) fn low(self) -> RegSelect {
        match self {
            Qq::Bc => RegSelect::R(R::C),
            Qq::De => RegSelect::R(R::E),
            Qq::Hl => RegSelect::R(R::L),
            Qq::Af => RegSelect::F,
        }
    }
}
