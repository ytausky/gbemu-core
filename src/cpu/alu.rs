use super::Flags;

pub fn add(lhs: u8, rhs: u8, carry_in: bool) -> (u8, Flags) {
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

pub fn sub(lhs: u8, rhs: u8, carry_in: bool) -> (u8, Flags) {
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
