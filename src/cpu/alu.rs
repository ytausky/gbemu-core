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
