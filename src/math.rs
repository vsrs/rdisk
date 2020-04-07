use num_traits::PrimInt;

pub fn ceil<T: PrimInt + Copy>(numerator: T, denominator: T) -> T {
    (numerator + (denominator - T::one())) / denominator
}

pub fn round_up<T: PrimInt + Copy>(value: T, unit: T) -> T {
    ceil(value, unit) * unit
}

pub fn round_down<T: PrimInt + Copy>(value: T, unit: T) -> T {
    (value / unit) * unit
}

pub fn bound_to(data_size: u64, offset: u64, len: usize) -> Option<usize> {
    let total = offset + len as u64;
    if total < data_size {
        return Some(len);
    }

    let overrun = total - data_size;
    if overrun <= len as u64 {
        return Some(len - overrun as usize);
    }

    None
}

pub fn rest(data_size: u64, offset: u64, len: usize) -> usize {
    match bound_to(data_size, offset, len) {
        Some(sz) => sz,
        None => 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ceil_test() {
        assert_eq!(4, ceil(100, 25));
        assert_eq!(4, ceil(100, 33));
    }

    #[test]
    fn round_up_test() {
        assert_eq!(100, round_up(100, 25));
        assert_eq!(132, round_up(100, 33));
    }

    #[test]
    fn round_down_test() {
        assert_eq!(100, round_down(100, 25));
        assert_eq!(99, round_down(100, 33));
    }

    #[test]
    fn bound_to_test() {
        assert_eq!(Some(0), bound_to(512, 512, 10));
        assert_eq!(Some(1), bound_to(512, 511, 10));
        assert_eq!(Some(10), bound_to(512, 502, 10));
        assert_eq!(Some(9), bound_to(512, 503, 10));
        assert_eq!(Some(10), bound_to(512, 0, 10));
        assert_eq!(None, bound_to(512, 513, 10));
    }

    #[test]
    fn rest_test() {
        assert_eq!(0, rest(512, 512, 10));
        assert_eq!(1, rest(512, 511, 10));
        assert_eq!(10, rest(512, 502, 10));
        assert_eq!(9, rest(512, 503, 10));
        assert_eq!(10, rest(512, 0, 10));
        assert_eq!(0, rest(512, 513, 10));
    }
}
