//! Numeric spreadsheet consumer of the existing decimal calculator.
//! No file, network, macro, native launch or resource authority.
#![no_std]
#![forbid(unsafe_code)]
extern crate alloc;

use alloc::string::String;
use core::fmt;
use nieuw_os_calculator::decimal::{self, Evaluation};

pub const MAX_CELLS: usize = 64;
pub const MAX_INPUT: usize = decimal::MAX_INPUT;
pub const MAX_DEPENDENCY_DEPTH: usize = 32;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Address {
    column: u8,
    row: u8,
}

impl Address {
    /// Strict canonical A1..Z64. No ranges, absolute refs, names or lowercase.
    pub fn parse(input: &str) -> Result<Self, Error> {
        let bytes = input.as_bytes();
        if !(2..=3).contains(&bytes.len())
            || !bytes[0].is_ascii_uppercase()
            || !(b'1'..=b'9').contains(&bytes[1])
            || !bytes[1..].iter().all(u8::is_ascii_digit)
        {
            return Err(Error::InvalidAddress);
        }
        let row = bytes[1..]
            .iter()
            .fold(0, |row, digit| row * 10 + digit - b'0');
        if row > 64 {
            return Err(Error::InvalidAddress);
        }
        Ok(Self {
            column: bytes[0] - b'A',
            row,
        })
    }

    pub const fn column(self) -> u8 {
        self.column
    }
    pub const fn row(self) -> u8 {
        self.row
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CellError {
    Cycle,
    DependencyDepth,
    MissingReference,
    InvalidReference,
    InvalidNumber,
    ExpandedLimit,
    Arithmetic(decimal::Error),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    InvalidAddress,
    InputLimit,
    CellLimit,
    OutOfMemory,
    Cell { address: Address, kind: CellError },
}

impl fmt::Display for Address {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}", char::from(b'A' + self.column), self.row)
    }
}

impl fmt::Display for CellError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Cycle => f.write_str("circulaire celverwijzing"),
            Self::DependencyDepth => f.write_str("maximale afhankelijkheidsdiepte bereikt"),
            Self::MissingReference => f.write_str("verwezen cel ontbreekt"),
            Self::InvalidReference => f.write_str("ongeldige celverwijzing"),
            Self::InvalidNumber => f.write_str("ongeldig getal"),
            Self::ExpandedLimit => f.write_str("uitgebreide formule is te lang"),
            Self::Arithmetic(error) => write!(
                f,
                "{} (positie {} in de geëvalueerde formule)",
                error.label(),
                error.position
            ),
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidAddress => f.write_str("ongeldig celadres (gebruik A1 t/m Z64)"),
            Self::InputLimit => f.write_str("celinvoer is ongeldig of te lang"),
            Self::CellLimit => f.write_str("maximaal 64 gevulde cellen"),
            Self::OutOfMemory => f.write_str("onvoldoende geheugen"),
            Self::Cell { address, kind } => write!(f, "Cel {address}: {kind}"),
        }
    }
}

struct Cell {
    address: Address,
    input: String,
}

pub struct Sheet {
    cells: [Option<Cell>; MAX_CELLS],
}

impl Default for Sheet {
    fn default() -> Self {
        Self {
            cells: [const { None }; MAX_CELLS],
        }
    }
}

impl Sheet {
    fn index(&self, address: Address) -> Option<usize> {
        self.cells
            .iter()
            .position(|cell| cell.as_ref().is_some_and(|cell| cell.address == address))
    }

    pub fn input(&self, address: Address) -> Option<&str> {
        self.index(address)
            .and_then(|index| self.cells[index].as_ref().map(|cell| cell.input.as_str()))
    }

    /// A failed admission/allocation leaves the previous cell intact.
    /// Empty input clears. Malformed bounded source is retained for correction.
    pub fn set(&mut self, address: Address, input: &str) -> Result<(), Error> {
        if input.len() > MAX_INPUT
            || !input.is_ascii()
            || input.bytes().any(|b| b.is_ascii_control())
        {
            return Err(Error::InputLimit);
        }
        let existing = self.index(address);
        if input.is_empty() {
            if let Some(index) = existing {
                self.cells[index] = None;
            }
            return Ok(());
        }
        let index = existing
            .or_else(|| self.cells.iter().position(Option::is_none))
            .ok_or(Error::CellLimit)?;
        let mut source = String::new();
        source
            .try_reserve_exact(input.len())
            .map_err(|_| Error::OutOfMemory)?;
        source.push_str(input);
        self.cells[index] = Some(Cell {
            address,
            input: source,
        });
        Ok(())
    }

    /// Recalculate against the current sheet. Memoization lives only for this
    /// call, so edits, clears and error repairs cannot retain stale results.
    pub fn value(&self, address: Address) -> Result<Evaluation, Error> {
        let index = self.index(address).ok_or(Error::Cell {
            address,
            kind: CellError::MissingReference,
        })?;
        Recalculation {
            sheet: self,
            visiting: [false; MAX_CELLS],
            cache: [None; MAX_CELLS],
        }
        .resolve(index, 0)
    }
}

struct Recalculation<'a> {
    sheet: &'a Sheet,
    visiting: [bool; MAX_CELLS],
    cache: [Option<Result<Evaluation, Error>>; MAX_CELLS],
}

impl Recalculation<'_> {
    fn resolve(&mut self, index: usize, depth: usize) -> Result<Evaluation, Error> {
        if let Some(result) = self.cache[index] {
            return result;
        }
        let cell = self.sheet.cells[index]
            .as_ref()
            .expect("only occupied indices are resolved");
        let failure = |kind| Error::Cell {
            address: cell.address,
            kind,
        };
        if self.visiting[index] {
            return Err(failure(CellError::Cycle));
        }
        if depth >= MAX_DEPENDENCY_DEPTH {
            return Err(failure(CellError::DependencyDepth));
        }
        self.visiting[index] = true;
        let result = self.evaluate(index, depth);
        self.visiting[index] = false;
        self.cache[index] = Some(result);
        result
    }

    fn evaluate(&mut self, index: usize, depth: usize) -> Result<Evaluation, Error> {
        let cell = self.sheet.cells[index].as_ref().expect("occupied cell");
        let failure = |kind| Error::Cell {
            address: cell.address,
            kind,
        };
        let input = cell.input.trim();
        let Some(formula) = input.strip_prefix('=') else {
            let bytes = input.as_bytes();
            let number = if matches!(bytes.first(), Some(b'+' | b'-')) {
                &bytes[1..]
            } else {
                bytes
            };
            if number.is_empty()
                || !number
                    .iter()
                    .all(|b| b.is_ascii_digit() || matches!(b, b'.' | b','))
            {
                return Err(failure(CellError::InvalidNumber));
            }
            return decimal::evaluate(input).map_err(|error| failure(CellError::Arithmetic(error)));
        };
        let mut expression = String::new();
        expression
            .try_reserve_exact(MAX_INPUT)
            .map_err(|_| Error::OutOfMemory)?;
        let mut at = 0;
        let mut rounded = false;
        while at < formula.len() {
            if formula.as_bytes()[at].is_ascii_alphabetic() {
                let begin = at;
                while at < formula.len() && formula.as_bytes()[at].is_ascii_alphabetic() {
                    at += 1;
                }
                while at < formula.len() && formula.as_bytes()[at].is_ascii_digit() {
                    at += 1;
                }
                let address = Address::parse(&formula[begin..at])
                    .map_err(|_| failure(CellError::InvalidReference))?;
                let dependency = self.sheet.index(address).ok_or(Error::Cell {
                    address,
                    kind: CellError::MissingReference,
                })?;
                let value = self.resolve(dependency, depth + 1)?;
                rounded |= value.rounded;
                let formatted = value.value.format();
                if expression.len() + formatted.as_str().len() + 2 > MAX_INPUT {
                    return Err(failure(CellError::ExpandedLimit));
                }
                expression.push('(');
                expression.push_str(formatted.as_str());
                expression.push(')');
            } else {
                if expression.len() == MAX_INPUT {
                    return Err(failure(CellError::ExpandedLimit));
                }
                expression.push(char::from(formula.as_bytes()[at]));
                at += 1;
            }
        }
        let mut value = decimal::evaluate(&expression)
            .map_err(|error| failure(CellError::Arithmetic(error)))?;
        value.rounded |= rounded;
        Ok(value)
    }
}
