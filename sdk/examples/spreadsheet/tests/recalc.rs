use nieuw_os_calculator::decimal::{Kind, SCALE};
use nieuw_os_spreadsheet::{Address, CellError, Error, Sheet, MAX_CELLS, MAX_DEPENDENCY_DEPTH};

fn address(input: &str) -> Address {
    Address::parse(input).unwrap()
}
fn set(sheet: &mut Sheet, at: &str, source: &str) {
    sheet.set(address(at), source).unwrap();
}
fn raw(sheet: &Sheet, at: &str) -> i128 {
    sheet.value(address(at)).unwrap().value.raw()
}

#[test]
fn dependency_diamond_recalculates_after_edit_and_retains_source() {
    let mut sheet = Sheet::default();
    set(&mut sheet, "A1", "10");
    set(&mut sheet, "A2", "=A1*2");
    set(&mut sheet, "B1", "=A1+3");
    set(&mut sheet, "C1", "=A2+B1");
    assert_eq!(raw(&sheet, "C1"), 33 * SCALE);
    set(&mut sheet, "A1", "20");
    assert_eq!(raw(&sheet, "C1"), 63 * SCALE);
    assert_eq!(sheet.input(address("C1")), Some("=A2+B1"));
    set(&mut sheet, "A1", "");
    assert!(matches!(
        sheet.value(address("C1")),
        Err(Error::Cell {
            kind: CellError::MissingReference,
            ..
        })
    ));
    set(&mut sheet, "A1", "-2");
    assert_eq!(raw(&sheet, "C1"), -3 * SCALE);
}

#[test]
fn references_are_substituted_as_values_without_changing_precedence() {
    let mut sheet = Sheet::default();
    set(&mut sheet, "A1", "-2");
    set(&mut sheet, "B2", "=3*A1--1");
    assert_eq!(raw(&sheet, "B2"), -5 * SCALE);
    set(&mut sheet, "B2", "=A1*A1+(12,5/2)");
    assert_eq!(raw(&sheet, "B2"), 10 * SCALE + SCALE / 4);
    set(&mut sheet, "B2", "=A1+A1+A1");
    assert_eq!(raw(&sheet, "B2"), -6 * SCALE);
}

#[test]
fn cycle_missing_and_division_errors_are_not_zero_and_repair_recalculates() {
    let mut sheet = Sheet::default();
    set(&mut sheet, "A1", "=B1");
    set(&mut sheet, "B1", "=A1");
    assert!(matches!(
        sheet.value(address("A1")),
        Err(Error::Cell {
            kind: CellError::Cycle,
            ..
        })
    ));
    set(&mut sheet, "B1", "2");
    assert_eq!(raw(&sheet, "A1"), 2 * SCALE);
    set(&mut sheet, "B1", "=3/0");
    assert!(
        matches!(sheet.value(address("A1")), Err(Error::Cell { kind: CellError::Arithmetic(e), .. }) if e.kind == Kind::DivideByZero)
    );
    set(&mut sheet, "B1", "=Z64");
    assert_eq!(
        sheet.value(address("A1")),
        Err(Error::Cell {
            address: address("Z64"),
            kind: CellError::MissingReference
        })
    );
    set(&mut sheet, "Z64", "4");
    assert_eq!(raw(&sheet, "A1"), 4 * SCALE);
}

#[test]
fn rounded_values_remain_visible_through_dependencies() {
    let mut sheet = Sheet::default();
    set(&mut sheet, "A1", "=1/3");
    set(&mut sheet, "B1", "=A1*3");
    let result = sheet.value(address("B1")).unwrap();
    assert!(result.rounded);
    assert_eq!(result.value.raw(), SCALE - 1);
    set(&mut sheet, "A1", "=1/2");
    assert!(!sheet.value(address("B1")).unwrap().rounded);
    assert_eq!(raw(&sheet, "B1"), SCALE * 3 / 2);
}

#[test]
fn canonical_addresses_and_malformed_sources_fail_explicitly() {
    for bad in [
        "", "AA1", "a1", "A0", "A01", "A65", "A999", "1A", "A-1", " A1", "A1 ", "漢1",
    ] {
        assert_eq!(Address::parse(bad), Err(Error::InvalidAddress), "{bad}");
    }
    assert_eq!((address("Z64").column(), address("Z64").row()), (25, 64));
    let mut sheet = Sheet::default();
    for source in ["=A0", "=a1", "=AA1", "=SUM(A1)", "=A999"] {
        set(&mut sheet, "B1", source);
        assert!(matches!(
            sheet.value(address("B1")),
            Err(Error::Cell {
                kind: CellError::InvalidReference,
                ..
            })
        ));
        assert_eq!(sheet.input(address("B1")), Some(source));
    }
    for source in ["text", "1+2", "1 2", "--1", " "] {
        set(&mut sheet, "B1", source);
        assert!(matches!(
            sheet.value(address("B1")),
            Err(Error::Cell {
                kind: CellError::InvalidNumber,
                ..
            })
        ));
    }
    for source in ["=1+", "=", "=1;2", "99999999999", "=9999999999*2"] {
        set(&mut sheet, "B1", source);
        assert!(sheet.value(address("B1")).is_err());
    }
}

#[test]
fn failed_admissions_are_transactional_and_clearing_releases_capacity() {
    let mut sheet = Sheet::default();
    for row in 1..=MAX_CELLS {
        set(&mut sheet, &format!("A{row}"), "1");
    }
    assert_eq!(sheet.set(address("B1"), "2"), Err(Error::CellLimit));
    for bad in ["1".repeat(257), "\0".into(), "漢".into(), "\n".into()] {
        assert_eq!(sheet.set(address("A1"), &bad), Err(Error::InputLimit));
        assert_eq!(sheet.input(address("A1")), Some("1"));
    }
    set(&mut sheet, "A1", "2");
    assert_eq!(raw(&sheet, "A1"), 2 * SCALE);
    set(&mut sheet, "A2", "");
    set(&mut sheet, "B1", "3");
    assert_eq!(raw(&sheet, "B1"), 3 * SCALE);
}

#[test]
fn reference_depth_and_expanded_expression_have_exact_hard_limits() {
    let mut sheet = Sheet::default();
    for row in 1..=MAX_DEPENDENCY_DEPTH {
        set(&mut sheet, &format!("A{row}"), &format!("=A{}", row + 1));
    }

    set(&mut sheet, &format!("A{}", MAX_DEPENDENCY_DEPTH + 1), "1");
    assert!(matches!(
        sheet.value(address("A1")),
        Err(Error::Cell {
            kind: CellError::DependencyDepth,
            ..
        })
    ));
    set(&mut sheet, &format!("A{MAX_DEPENDENCY_DEPTH}"), "1");
    assert_eq!(raw(&sheet, "A1"), SCALE);
    set(&mut sheet, "B1", "9999999999.123456789");
    set(&mut sheet, "C1", &format!("={}", ["B1"; 20].join("+")));
    assert!(matches!(
        sheet.value(address("C1")),
        Err(Error::Cell {
            kind: CellError::ExpandedLimit,
            ..
        })
    ));
}

#[test]
fn repeated_dependency_dag_is_memoized_without_reinterpreting_results_as_formulas() {
    let mut sheet = Sheet::default();
    set(&mut sheet, "A1", "1");
    for row in 2..=30 {
        set(
            &mut sheet,
            &format!("A{row}"),
            &format!("=A{}+A{}", row - 1, row - 1),
        );
    }
    assert_eq!(raw(&sheet, "A30"), (1i128 << 29) * SCALE);
    set(&mut sheet, "A1", "2");
    assert_eq!(raw(&sheet, "A30"), (1i128 << 30) * SCALE);
    set(&mut sheet, "B1", &"0".repeat(256));
    assert_eq!(raw(&sheet, "B1"), 0);
    assert_eq!(sheet.input(address("B1")).unwrap().len(), 256);
}
