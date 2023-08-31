/**
returns (g,x,y) such that g = ax + by
*/
#[allow(clippy::many_single_char_names, reason = "this is a math function")]
#[allow(clippy::arithmetic_side_effects, reason = "this is a math function")]
pub fn extended_euclid(a: i32, b: i32) -> (i32, i32, i32) {
    match b {
        0_i32 => (a, 1_i32, 0_i32),
        c => {
            let (gcd, x, y) = extended_euclid(c, a % c);
            (gcd, y, x - (a / c) * y)
        }
    }
}

pub fn try_inverse(a: i32, m: i32) -> Option<i32> {
    let (gcd, x, _y) = extended_euclid(a, m);
    (gcd == 1_i32).then_some(x.rem_euclid(m))
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn eudlid_algo() {
        assert_eq!(extended_euclid(2, 3), (1_i32, -1_i32, 1_i32));
        assert_eq!(extended_euclid(4, 6), (2_i32, -1_i32, 1_i32));
        assert_eq!(extended_euclid(6, 27), (3_i32, -4_i32, 1_i32));
    }

    #[test]
    fn modulo_inverse() {
        assert_eq!(try_inverse(5, 6), Some(5_i32));
        assert_eq!(try_inverse(9, 11), Some(5_i32));
        assert_eq!(try_inverse(4, 6), None);
    }
}
