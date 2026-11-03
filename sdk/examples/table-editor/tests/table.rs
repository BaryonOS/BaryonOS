use nieuw_os_table_editor::{Error, Limits, Table};

fn table(input: &[u8]) -> Table {
    Table::import(input, Limits::default()).unwrap()
}

#[test]
fn import_edit_export_reopen_preserves_unicode_quotes_newlines_and_empty_cells() {
    let input =
        "Naam,Notitie,Waarde\r\nZoë,\"een, twee\",12\r\nSimon,\"regel 1\r\nregel \"\"2\"\"\",\r\n";
    let mut document = table(input.as_bytes());
    assert_eq!((document.rows(), document.columns()), (3, 3));
    assert_eq!(document.cell(1, 0), Some("Zoë"));
    assert_eq!(document.cell(2, 1), Some("regel 1\r\nregel \"2\""));
    assert_eq!(document.cell(2, 2), Some(""));
    document.set_cell(1, 2, "€ 42,50").unwrap();
    document.set_cell(2, 0, "李").unwrap();
    let exported = document.export().unwrap();
    assert_eq!(exported, "Naam,Notitie,Waarde\r\nZoë,\"een, twee\",\"€ 42,50\"\r\n李,\"regel 1\r\nregel \"\"2\"\"\",\"\"\r\n".as_bytes());
    let reopened = table(&exported);
    for row in 0..3 {
        for column in 0..3 {
            assert_eq!(reopened.cell(row, column), document.cell(row, column));
        }
    }
}

#[test]
fn rfc4180_example_and_header_are_literal_rows_not_inferred_schema() {
    let document = table(b"field_name,field_name,field_name\r\n aaa ,bbb,ccc\r\nzzz,yyy,xxx");
    assert_eq!(document.cell(0, 0), Some("field_name"));
    assert_eq!(document.cell(1, 0), Some(" aaa "));
    assert_eq!(
        document.export().unwrap(),
        b"field_name,field_name,field_name\r\n aaa ,bbb,ccc\r\nzzz,yyy,xxx\r\n"
    );
}

#[test]
fn empty_input_empty_cell_trailing_comma_and_blank_record_remain_distinct() {
    for (input, rows, columns, expected) in [
        ("", 0, 0, ""),
        ("\u{feff}", 0, 0, ""),
        ("\n", 1, 1, "\"\"\r\n"),
        ("\"\"", 1, 1, "\"\"\r\n"),
        (",", 1, 2, "\"\",\"\"\r\n"),
        ("a,\n", 1, 2, "a,\"\"\r\n"),
        ("a\n\n", 2, 1, "a\r\n\"\"\r\n"),
        ("\u{feff}a,b\nc,d\n", 2, 2, "a,b\r\nc,d\r\n"),
    ] {
        let document = table(input.as_bytes());
        assert_eq!(
            (document.rows(), document.columns()),
            (rows, columns),
            "{input:?}"
        );
        assert_eq!(document.export().unwrap(), expected.as_bytes(), "{input:?}");
    }
}

#[test]
fn malformed_input_is_rejected_without_partial_table() {
    for (input, error) in [
        (b"\"unterminated".as_slice(), Error::UnterminatedQuote),
        (b"\"a\"x", Error::TrailingQuotedText),
        (b"\"a\" ", Error::TrailingQuotedText),
        (b"a\"b", Error::UnexpectedQuote),
        (b"\"a\"\"b", Error::UnterminatedQuote),
        (b"a\rb", Error::BareCarriageReturn),
        (b"a\r", Error::BareCarriageReturn),
        (b"a,b\nc", Error::RaggedRow),
        (b"a\nb,c", Error::RaggedRow),
        (b"a\0b", Error::InvalidText),
        (b"\xff", Error::InvalidUtf8),
        (b"\xc3", Error::InvalidUtf8),
    ] {
        assert_eq!(
            Table::import(input, Limits::default()).unwrap_err(),
            error,
            "{input:?}"
        );
    }
}

#[test]
fn every_resource_bound_is_enforced_at_its_exact_threshold() {
    let defaults = Limits::default();
    for (input, limits, error) in [
        (
            "ab",
            Limits {
                input_bytes: 1,
                ..defaults
            },
            Error::InputLimit,
        ),
        (
            "ab",
            Limits {
                field_bytes: 1,
                ..defaults
            },
            Error::FieldLimit,
        ),
        (
            "a,b",
            Limits {
                text_bytes: 1,
                ..defaults
            },
            Error::TextLimit,
        ),
        (
            "a,b",
            Limits {
                columns: 1,
                ..defaults
            },
            Error::ColumnLimit,
        ),
        (
            "a\nb",
            Limits {
                rows: 1,
                ..defaults
            },
            Error::RowLimit,
        ),
        (
            "\"a\"\"b\"",
            Limits {
                field_bytes: 2,
                ..defaults
            },
            Error::FieldLimit,
        ),
        (
            "é",
            Limits {
                field_bytes: 1,
                ..defaults
            },
            Error::FieldLimit,
        ),
    ] {
        assert_eq!(Table::import(input.as_bytes(), limits).unwrap_err(), error);
    }
    let document = Table::import(
        b"\"a\"\"b\"",
        Limits {
            input_bytes: 6,
            field_bytes: 3,
            text_bytes: 3,
            rows: 1,
            columns: 1,
            output_bytes: 8,
        },
    )
    .unwrap();
    assert_eq!(document.export().unwrap(), b"\"a\"\"b\"\r\n");
    assert_eq!(
        Table::import(
            b"",
            Limits {
                rows: 0,
                ..defaults
            }
        )
        .unwrap_err(),
        Error::InvalidLimits
    );
    assert_eq!(
        Table::import(
            b"",
            Limits {
                columns: 257,
                ..defaults
            }
        )
        .unwrap_err(),
        Error::InvalidLimits
    );
    assert_eq!(
        Table::import(
            b"",
            Limits {
                input_bytes: usize::MAX,
                ..defaults
            }
        )
        .unwrap_err(),
        Error::InvalidLimits
    );
    let document = Table::import(
        b"a",
        Limits {
            output_bytes: 2,
            ..defaults
        },
    )
    .unwrap();
    assert_eq!(document.export(), Err(Error::OutputLimit));
}

#[test]
fn failed_edit_is_transactional_and_same_size_replacement_does_not_consume_budget() {
    let mut document = Table::import(
        b"a,b",
        Limits {
            field_bytes: 2,
            text_bytes: 2,
            ..Limits::default()
        },
    )
    .unwrap();
    let original = document.export().unwrap();
    for (row, column, value, error) in [
        (2, 0, "x", Error::CellOutOfBounds),
        (0, 2, "x", Error::CellOutOfBounds),
        (0, 0, "xx", Error::TextLimit),
        (0, 0, "\0", Error::InvalidText),
    ] {
        assert_eq!(document.set_cell(row, column, value), Err(error));
        assert_eq!(document.export().unwrap(), original);
    }
    for _ in 0..10 {
        document.set_cell(0, 0, "z").unwrap();
    }
    assert_eq!(document.export().unwrap(), b"z,b\r\n");
    document.set_cell(0, 0, "").unwrap();
    document.set_cell(0, 1, "xy").unwrap();
    assert_eq!(document.cell(0, 1), Some("xy"));
    assert_eq!(document.set_cell(0, 1, "xyz"), Err(Error::TextLimit));
    let mut document = Table::import(
        b"a",
        Limits {
            field_bytes: 1,
            ..Limits::default()
        },
    )
    .unwrap();
    assert_eq!(document.set_cell(0, 0, "xx"), Err(Error::FieldLimit));
    assert_eq!(document.cell(0, 0), Some("a"));
}

#[test]
fn formula_like_cells_are_never_evaluated_or_silently_modified() {
    let document = table(b"=SUM(A1:A2),+1,-2,@command,001,1e99,2026-11-03");
    assert_eq!(
        document.export().unwrap(),
        b"=SUM(A1:A2),+1,-2,@command,001,1e99,2026-11-03\r\n"
    );
}

#[test]
fn exhaustive_small_text_roundtrips_preserve_all_cell_boundaries() {
    let alphabet = ["", "a", ",", "\"", "\n", "\r", "é", " ", "\u{feff}"];
    for first in alphabet {
        for second in alphabet {
            let mut document = table(b"x,y\nz,w");
            document.set_cell(0, 0, first).unwrap();
            document.set_cell(1, 1, second).unwrap();
            let exported = document.export().unwrap();
            let reopened = table(&exported);
            assert_eq!((reopened.rows(), reopened.columns()), (2, 2));
            for row in 0..2 {
                for column in 0..2 {
                    assert_eq!(reopened.cell(row, column), document.cell(row, column));
                }
            }
            assert_eq!(reopened.export().unwrap(), exported);
        }
    }
}

#[test]
fn leading_bom_cell_text_survives_edit_export_and_reopen() {
    for input in ["x", "\u{feff}\u{feff}x", "\"\u{feff}x\""] {
        let mut document = table(input.as_bytes());
        document.set_cell(0, 0, "\u{feff}x").unwrap();
        let exported = document.export().unwrap();
        assert_eq!(exported, "\"\u{feff}x\"\r\n".as_bytes());
        let reopened = table(&exported);
        assert_eq!(reopened.cell(0, 0), Some("\u{feff}x"));
        assert_eq!(reopened.export().unwrap(), exported);
    }
    let document = table("\u{feff}\u{feff}x".as_bytes());
    let reopened = table(&document.export().unwrap());
    assert_eq!(reopened.cell(0, 0), Some("\u{feff}x"));
}

#[test]
fn malformed_small_byte_sequences_never_panic_or_change_accepted_values_on_reopen() {
    let alphabet = [b'a', b'"', b',', b'\n', b'\r', 0xff];
    for mut combination in 0usize..alphabet.len().pow(5) {
        let mut input = [0; 5];
        for byte in &mut input {
            *byte = alphabet[combination % alphabet.len()];
            combination /= alphabet.len();
        }
        if let Ok(document) = Table::import(&input, Limits::default()) {
            let exported = document.export().unwrap();
            let reopened = table(&exported);
            assert_eq!(reopened.export().unwrap(), exported);
            assert_eq!(
                (document.rows(), document.columns()),
                (reopened.rows(), reopened.columns())
            );
            for row in 0..document.rows() {
                for column in 0..document.columns() {
                    assert_eq!(document.cell(row, column), reopened.cell(row, column));
                }
            }
        }
    }
}
