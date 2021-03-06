use num_bigint::{BigInt, ToBigInt};
use num_traits::{Float, Signed, ToPrimitive, Zero};

pub fn ufrexp(value: f64) -> (f64, i32) {
    if 0.0 == value {
        (0.0, 0i32)
    } else {
        let bits = value.to_bits();
        let exponent: i32 = ((bits >> 52) & 0x7ff) as i32 - 1022;
        let mantissa_bits = bits & (0x000f_ffff_ffff_ffff) | (1022 << 52);
        (f64::from_bits(mantissa_bits), exponent)
    }
}

pub fn eq_int(value: f64, other: &BigInt) -> bool {
    if let (Some(self_int), Some(other_float)) = (value.to_bigint(), other.to_f64()) {
        value == other_float && self_int == *other
    } else {
        false
    }
}

pub fn lt_int(value: f64, other_int: &BigInt) -> bool {
    match (value.to_bigint(), other_int.to_f64()) {
        (Some(self_int), Some(other_float)) => value < other_float || self_int < *other_int,
        // finite float, other_int too big for float,
        // the result depends only on other_int’s sign
        (Some(_), None) => other_int.is_positive(),
        // infinite float must be bigger or lower than any int, depending on its sign
        _ if value.is_infinite() => value.is_sign_negative(),
        // NaN, always false
        _ => false,
    }
}

pub fn gt_int(value: f64, other_int: &BigInt) -> bool {
    match (value.to_bigint(), other_int.to_f64()) {
        (Some(self_int), Some(other_float)) => value > other_float || self_int > *other_int,
        // finite float, other_int too big for float,
        // the result depends only on other_int’s sign
        (Some(_), None) => other_int.is_negative(),
        // infinite float must be bigger or lower than any int, depending on its sign
        _ if value.is_infinite() => value.is_sign_positive(),
        // NaN, always false
        _ => false,
    }
}

pub fn parse_str(literal: &str) -> Option<f64> {
    if literal.starts_with('_') || literal.ends_with('_') {
        return None;
    }

    let mut buf = String::with_capacity(literal.len());
    let mut last_tok: Option<char> = None;
    for c in literal.chars() {
        if !(c.is_ascii_alphanumeric() || c == '_' || c == '+' || c == '-' || c == '.') {
            return None;
        }

        if !c.is_ascii_alphanumeric() {
            if let Some(l) = last_tok {
                if !l.is_ascii_alphanumeric() && !(c == '.' && (l == '-' || l == '+')) {
                    return None;
                }
            }
        }

        if c != '_' {
            buf.push(c);
        }
        last_tok = Some(c);
    }

    if let Ok(f) = lexical_core::parse(buf.as_bytes()) {
        Some(f)
    } else {
        None
    }
}

pub fn is_integer(v: f64) -> bool {
    (v - v.round()).abs() < std::f64::EPSILON
}

pub fn to_string(value: f64) -> String {
    let lit = format!("{:e}", value);
    if let Some(position) = lit.find('e') {
        let significand = &lit[..position];
        let exponent = &lit[position + 1..];
        let exponent = exponent.parse::<i32>().unwrap();
        if exponent < 16 && exponent > -5 {
            if is_integer(value) {
                format!("{:.1?}", value)
            } else {
                value.to_string()
            }
        } else {
            format!("{}e{:+#03}", significand, exponent)
        }
    } else {
        value.to_string()
    }
}

pub fn from_hex(s: &str) -> Option<f64> {
    if let Ok(f) = hexf_parse::parse_hexf64(s, false) {
        return Some(f);
    }
    match s.to_ascii_lowercase().as_str() {
        "nan" | "+nan" | "-nan" => Some(f64::NAN),
        "inf" | "infinity" | "+inf" | "+infinity" => Some(f64::INFINITY),
        "-inf" | "-infinity" => Some(f64::NEG_INFINITY),
        value => {
            let mut hex = String::with_capacity(value.len());
            let has_0x = value.contains("0x");
            let has_p = value.contains('p');
            let has_dot = value.contains('.');
            let mut start = 0;

            if !has_0x && value.starts_with('-') {
                hex.push_str("-0x");
                start += 1;
            } else if !has_0x {
                hex.push_str("0x");
                if value.starts_with('+') {
                    start += 1;
                }
            }

            for (index, ch) in value.chars().enumerate() {
                if ch == 'p' {
                    if has_dot {
                        hex.push('p');
                    } else {
                        hex.push_str(".p");
                    }
                } else if index >= start {
                    hex.push(ch);
                }
            }

            if !has_p && has_dot {
                hex.push_str("p0");
            } else if !has_p && !has_dot {
                hex.push_str(".p0")
            }

            hexf_parse::parse_hexf64(hex.as_str(), false).ok()
        }
    }
}

pub fn to_hex(value: f64) -> String {
    let (mantissa, exponent, sign) = value.integer_decode();
    let sign_fmt = if sign < 0 { "-" } else { "" };
    match value {
        value if value.is_zero() => format!("{}0x0.0p+0", sign_fmt),
        value if value.is_infinite() => format!("{}inf", sign_fmt),
        value if value.is_nan() => "nan".to_owned(),
        _ => {
            const BITS: i16 = 52;
            const FRACT_MASK: u64 = 0xf_ffff_ffff_ffff;
            format!(
                "{}0x{:x}.{:013x}p{:+}",
                sign_fmt,
                mantissa >> BITS,
                mantissa & FRACT_MASK,
                exponent + BITS
            )
        }
    }
}

pub fn div(v1: f64, v2: f64) -> Option<f64> {
    if v2 != 0.0 {
        Some(v1 / v2)
    } else {
        None
    }
}

pub fn mod_(v1: f64, v2: f64) -> Option<f64> {
    if v2 != 0.0 {
        Some(v1 % v2)
    } else {
        None
    }
}

pub fn floordiv(v1: f64, v2: f64) -> Option<f64> {
    if v2 != 0.0 {
        Some((v1 / v2).floor())
    } else {
        None
    }
}

pub fn divmod(v1: f64, v2: f64) -> Option<(f64, f64)> {
    if v2 != 0.0 {
        let mut m = v1 % v2;
        let mut d = (v1 - m) / v2;
        if v2.is_sign_negative() != m.is_sign_negative() {
            m += v2;
            d -= 1.0;
        }
        Some((d, m))
    } else {
        None
    }
}

// nextafter algorithm based off of https://gitlab.com/bronsonbdevost/next_afterf
#[allow(clippy::float_cmp)]
pub fn nextafter(x: f64, y: f64) -> f64 {
    if x == y {
        y
    } else if x.is_nan() || y.is_nan() {
        f64::NAN
    } else if x >= f64::INFINITY {
        f64::MAX
    } else if x <= f64::NEG_INFINITY {
        f64::MIN
    } else if x == 0.0 {
        f64::from_bits(1).copysign(y)
    } else {
        // next x after 0 if y is farther from 0 than x, otherwise next towards 0
        // the sign is a separate bit in floats, so bits+1 moves away from 0 no matter the float
        let b = x.to_bits();
        let bits = if (y > x) == (x > 0.0) { b + 1 } else { b - 1 };
        let ret = f64::from_bits(bits);
        if ret == 0.0 {
            ret.copysign(x)
        } else {
            ret
        }
    }
}

pub fn ulp(x: f64) -> f64 {
    if x.is_nan() {
        return x;
    }
    let x = x.abs();
    let x2 = nextafter(x, f64::INFINITY);
    if x2.is_infinite() {
        // special case: x is the largest positive representable float
        let x2 = nextafter(x, f64::NEG_INFINITY);
        x - x2
    } else {
        x2 - x
    }
}

#[test]
fn test_to_hex() {
    use rand::Rng;
    for _ in 0..20000 {
        let bytes = rand::thread_rng().gen::<[u64; 1]>();
        let f = f64::from_bits(bytes[0]);
        if !f.is_finite() {
            continue;
        }
        let hex = to_hex(f);
        // println!("{} -> {}", f, hex);
        let roundtrip = hexf_parse::parse_hexf64(&hex, false).unwrap();
        // println!("  -> {}", roundtrip);
        assert!(f == roundtrip, "{} {} {}", f, hex, roundtrip);
    }
}
