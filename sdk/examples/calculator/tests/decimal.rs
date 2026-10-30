use nieuw_os_calculator::decimal::{evaluate, Kind, Value, MAX_RAW, SCALE};
fn raw(text: &str, value: i128, rounded: bool) {
    let r = evaluate(text).unwrap();
    assert_eq!((r.value.raw(), r.rounded), (value, rounded), "{text}");
}
#[test]
fn precedence_parentheses_unary_comma_and_exact_integer_math() {
    raw("2+3*4", 14 * SCALE, false);
    raw("(2+3)*4", 20 * SCALE, false);
    raw("--(3-5)/-2", SCALE, false);
    raw(" 1,25 + .75 ", 2 * SCALE, false);
    raw("000001.2500000000", 1_250_000_000, false);
    raw("2.", 2 * SCALE, false);
    raw("10/4*2", 5 * SCALE, false);
    raw("1-2-3", -4 * SCALE, false);
}
#[test]
fn signed_halftoeven_applies_to_literals_products_and_divisions() {
    for (s, v) in [
        ("0.0000000005", 0),
        ("-0.0000000005", 0),
        ("0.0000000015", 2),
        ("-0.0000000015", -2),
        ("1.00000000250000000000", 1_000_000_002),
        ("1.00000000250000000001", 1_000_000_003),
        ("0.000000001*0.5", 0),
        ("0.000000003*0.5", 2),
        ("-0.000000003*0.5", -2),
        ("0.000000001/2", 0),
        ("-0.000000003/2", -2),
        ("1/3", 333_333_333),
    ] {
        raw(s, v, true);
    }
    assert_eq!(
        evaluate("-0.0000000005").unwrap().value.format().as_str(),
        "0"
    );
}
#[test]
fn full_raw_products_fit_i128_and_do_not_reject_small_valid_results() {
    assert!(MAX_RAW.checked_mul(MAX_RAW).is_some());
    for sign in ["", "-"] {
        raw(
            &format!("{sign}9999999999.999999999*0.000000001"),
            if sign.is_empty() {
                10 * SCALE
            } else {
                -10 * SCALE
            },
            true,
        );
        raw(
            &format!("{sign}9999999999.999999999/1"),
            if sign.is_empty() { MAX_RAW } else { -MAX_RAW },
            false,
        );
    }
    for text in [
        "9999999999.999999999+0.000000001",
        "-9999999999.999999999-0.000000001",
        "9999999999*9999999999",
        "9999999999/0.000000001",
        "9999999999.9999999995",
    ] {
        assert_eq!(evaluate(text).unwrap_err().kind, Kind::Range, "{text}");
    }
}
#[test]
fn input_nesting_syntax_and_zero_division_are_bounded_without_panics() {
    assert_eq!(evaluate(&"1".repeat(257)).unwrap_err().kind, Kind::TooLong);
    raw(
        &format!("{}1{}", "(".repeat(32), ")".repeat(32)),
        SCALE,
        false,
    );
    assert_eq!(
        evaluate(&format!("{}1{}", "(".repeat(33), ")".repeat(33)))
            .unwrap_err()
            .kind,
        Kind::Nesting
    );
    assert_eq!(
        evaluate(&format!("{}1", "-".repeat(33))).unwrap_err().kind,
        Kind::Nesting
    );
    for text in ["", " "] {
        assert_eq!(evaluate(text).unwrap_err().kind, Kind::Empty);
    }
    for text in ["1/0", "1/(3-3)", "-1/-0"] {
        assert_eq!(evaluate(text).unwrap_err().kind, Kind::DivideByZero);
    }
    for text in [".", "1..2", "1,2.3", "2(3)", "(1", "1+", "1 2", ")1("] {
        assert_eq!(evaluate(text).unwrap_err().kind, Kind::Syntax, "{text}");
    }
    for text in ["∞", "2^3", "NaN", "1e2"] {
        assert_eq!(evaluate(text).unwrap_err().kind, Kind::Unsupported);
    }
    let alphabet = b"0123+-*/()., ";
    let mut seed = 7u64;
    for length in 0..256 {
        let s: String = (0..length)
            .map(|_| {
                seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
                alphabet[(seed >> 32) as usize % alphabet.len()] as char
            })
            .collect();
        let _ = evaluate(&s);
    }
}
#[test]
fn decimal_formatter_is_exact_localized_and_canonical_at_every_bound() {
    for (raw, text) in [
        (0, "0"),
        (1, "0,000000001"),
        (-1, "-0,000000001"),
        (SCALE, "1"),
        (1_250_000_000, "1,25"),
        (MAX_RAW, "9999999999,999999999"),
        (-MAX_RAW, "-9999999999,999999999"),
    ] {
        assert_eq!(Value::from_raw(raw).unwrap().format().as_str(), text);
    }
    assert!(Value::from_raw(MAX_RAW + 1).is_none());
    assert!(Value::from_raw(i128::MIN).is_none());
}
