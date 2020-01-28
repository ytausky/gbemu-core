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
        (u16::from(*self.select_r(h)) << 8) + u16::from(*self.select_r(l))
    }

    pub(super) fn read_dd(&self, dd: Dd) -> u16 {
        match dd {
            Dd::Bc => self.bc(),
            Dd::De => self.de(),
            Dd::Hl => self.hl(),
            Dd::Sp => self.sp,
        }
    }

    pub(super) fn select_r(&self, r: R) -> &u8 {
        match r {
            R::A => &self.a,
            R::B => &self.b,
            R::C => &self.c,
            R::D => &self.d,
            R::E => &self.e,
            R::H => &self.h,
            R::L => &self.l,
        }
    }

    pub(super) fn select_r_mut(&mut self, r: R) -> &mut u8 {
        match r {
            R::A => &mut self.a,
            R::B => &mut self.b,
            R::C => &mut self.c,
            R::D => &mut self.d,
            R::E => &mut self.e,
            R::H => &mut self.h,
            R::L => &mut self.l,
        }
    }

    pub(super) fn read_qq_h(&self, qq: Qq) -> u8 {
        match qq {
            Qq::Bc => self.b,
            Qq::De => self.d,
            Qq::Hl => self.h,
            Qq::Af => self.a,
        }
    }

    pub(super) fn read_qq_l(&self, qq: Qq) -> u8 {
        match qq {
            Qq::Bc => self.c,
            Qq::De => self.e,
            Qq::Hl => self.l,
            Qq::Af => (&self.f).into(),
        }
    }

    pub(super) fn write_qq_h(&mut self, qq: Qq, data: u8) {
        match qq {
            Qq::Bc => self.b = data,
            Qq::De => self.d = data,
            Qq::Hl => self.h = data,
            Qq::Af => self.a = data,
        }
    }

    pub(super) fn write_qq_l(&mut self, qq: Qq, data: u8) {
        match qq {
            Qq::Bc => self.c = data,
            Qq::De => self.e = data,
            Qq::Hl => self.l = data,
            Qq::Af => self.f = data.into(),
        }
    }
}

impl From<&Flags> for u8 {
    fn from(flags: &Flags) -> Self {
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
