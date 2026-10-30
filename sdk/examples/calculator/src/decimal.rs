//! Bounded decimal arithmetic: 9 fractional places, half to even. Every
//! literal and operation result must fit +/-9_999_999_999.999999999. With
//! |raw|<=10^19-1, |a*b|<10^38<i128::MAX and |a*10^9|<10^28.
//! Checks remain explicit. No float, allocator, libc, host or prepared answer.
pub const SCALE: i128 = 1_000_000_000;
pub const MAX_RAW: i128 = 9_999_999_999_999_999_999;
pub const MAX_INPUT: usize = 256;
pub const MAX_DEPTH: usize = 32;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Empty,
    Syntax,
    DivideByZero,
    Range,
    TooLong,
    Nesting,
    Unsupported,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Error {
    pub kind: Kind,
    pub position: usize,
}
impl Error {
    const fn new(kind: Kind, position: usize) -> Self {
        Self { kind, position }
    }
    pub const fn label(self) -> &'static str {
        match self.kind {
            Kind::Empty => "Voer een berekening in.",
            Kind::Syntax => "Controleer de berekening.",
            Kind::DivideByZero => "Delen door nul kan niet.",
            Kind::Range => "Getal buiten bereik: maximaal 10 gehele cijfers.",
            Kind::TooLong => "Maximaal 256 tekens. Je invoer blijft behouden.",
            Kind::Nesting => "Maximaal 32 geneste tekens. Je invoer blijft behouden.",
            Kind::Unsupported => "Gebruik cijfers, + - * / ( ) en een komma of punt.",
        }
    }
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Value(i128);
impl Value {
    pub const fn raw(self) -> i128 {
        self.0
    }
    pub const fn from_raw(raw: i128) -> Option<Self> {
        if raw < -MAX_RAW || raw > MAX_RAW {
            None
        } else {
            Some(Self(raw))
        }
    }
    pub fn format(self) -> Formatted {
        let mut result = Formatted {
            bytes: [0; 32],
            length: 0,
        };
        let mut number = self.0.unsigned_abs();
        if self.0 < 0 {
            result.push(b'-');
        }
        let integer = number / SCALE as u128;
        let mut reverse = [0; 10];
        let mut length = 0;
        let mut quotient = integer;
        loop {
            reverse[length] = b'0' + (quotient % 10) as u8;
            length += 1;
            quotient /= 10;
            if quotient == 0 {
                break;
            }
        }
        for at in (0..length).rev() {
            result.push(reverse[at]);
        }
        number %= SCALE as u128;
        if number != 0 {
            result.push(b',');
            let mut fraction = [b'0'; 9];
            for at in (0..9).rev() {
                fraction[at] += (number % 10) as u8;
                number /= 10;
            }
            let mut end = 9;
            while fraction[end - 1] == b'0' {
                end -= 1;
            }
            for byte in &fraction[..end] {
                result.push(*byte);
            }
        }
        result
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Formatted {
    bytes: [u8; 32],
    length: usize,
}
impl Formatted {
    fn push(&mut self, byte: u8) {
        self.bytes[self.length] = byte;
        self.length += 1;
    }
    pub fn as_str(&self) -> &str {
        core::str::from_utf8(&self.bytes[..self.length]).expect("decimal formatter emits ASCII")
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Evaluation {
    pub value: Value,
    pub rounded: bool,
}

fn bounded(raw: i128, at: usize) -> Result<Value, Error> {
    Value::from_raw(raw).ok_or(Error::new(Kind::Range, at))
}
fn divided(numerator: i128, denominator: i128, at: usize) -> Result<Evaluation, Error> {
    if denominator == 0 {
        return Err(Error::new(Kind::DivideByZero, at));
    }
    let negative = (numerator < 0) != (denominator < 0);
    let n = numerator.checked_abs().ok_or(Error::new(Kind::Range, at))?;
    let d = denominator
        .checked_abs()
        .ok_or(Error::new(Kind::Range, at))?;
    let mut q = n / d;
    let remainder = n % d;
    let twice = remainder
        .checked_mul(2)
        .ok_or(Error::new(Kind::Range, at))?;
    if twice > d || twice == d && q % 2 != 0 {
        q = q.checked_add(1).ok_or(Error::new(Kind::Range, at))?;
    }
    let raw = if negative { -q } else { q };
    Ok(Evaluation {
        value: bounded(raw, at)?,
        rounded: remainder != 0,
    })
}

pub fn evaluate(input: &str) -> Result<Evaluation, Error> {
    if input.len() > MAX_INPUT {
        return Err(Error::new(Kind::TooLong, MAX_INPUT));
    }
    for (at, byte) in input.bytes().enumerate() {
        if !byte.is_ascii()
            || !matches!(
                byte,
                b'0'..=b'9' | b'+' | b'-' | b'*' | b'/' | b'(' | b')' | b'.' | b',' | b' ' | b'\t'
            )
        {
            return Err(Error::new(Kind::Unsupported, at));
        }
    }
    let mut parser = Parser {
        bytes: input.as_bytes(),
        at: 0,
        depth: 0,
    };
    parser.space();
    if parser.at == parser.bytes.len() {
        return Err(Error::new(Kind::Empty, parser.at));
    }
    let answer = parser.sum()?;
    parser.space();
    if parser.at != parser.bytes.len() {
        return Err(Error::new(Kind::Syntax, parser.at));
    }
    Ok(answer)
}
struct Parser<'a> {
    bytes: &'a [u8],
    at: usize,
    depth: usize,
}
impl Parser<'_> {
    fn space(&mut self) {
        while self
            .bytes
            .get(self.at)
            .is_some_and(|b| *b == b' ' || *b == b'\t')
        {
            self.at += 1;
        }
    }
    fn sum(&mut self) -> Result<Evaluation, Error> {
        let mut left = self.product()?;
        loop {
            self.space();
            let at = self.at;
            let op = match self.bytes.get(at) {
                Some(b'+' | b'-') => self.bytes[at],
                _ => break,
            };
            self.at += 1;
            let right = self.product()?;
            let raw = if op == b'+' {
                left.value.0.checked_add(right.value.0)
            } else {
                left.value.0.checked_sub(right.value.0)
            }
            .ok_or(Error::new(Kind::Range, at))?;
            left = Evaluation {
                value: bounded(raw, at)?,
                rounded: left.rounded || right.rounded,
            };
        }
        Ok(left)
    }
    fn product(&mut self) -> Result<Evaluation, Error> {
        let mut left = self.unary()?;
        loop {
            self.space();
            let at = self.at;
            let op = match self.bytes.get(at) {
                Some(b'*' | b'/') => self.bytes[at],
                _ => break,
            };
            self.at += 1;
            let right = self.unary()?;
            let n = left
                .value
                .0
                .checked_mul(if op == b'*' { right.value.0 } else { SCALE })
                .ok_or(Error::new(Kind::Range, at))?;
            let mut result = divided(n, if op == b'*' { SCALE } else { right.value.0 }, at)?;
            result.rounded |= left.rounded || right.rounded;
            left = result;
        }
        Ok(left)
    }
    fn nested(&mut self) -> Result<(), Error> {
        if self.depth == MAX_DEPTH {
            return Err(Error::new(Kind::Nesting, self.at));
        }
        self.depth += 1;
        Ok(())
    }
    fn unary(&mut self) -> Result<Evaluation, Error> {
        self.space();
        if let Some(op @ (b'+' | b'-')) = self.bytes.get(self.at).copied() {
            self.nested()?;
            self.at += 1;
            let mut result = self.unary()?;
            self.depth -= 1;
            if op == b'-' {
                result.value.0 = -result.value.0;
            }
            return Ok(result);
        }
        if self.bytes.get(self.at) == Some(&b'(') {
            self.nested()?;
            self.at += 1;
            let result = self.sum()?;
            self.space();
            if self.bytes.get(self.at) != Some(&b')') {
                return Err(Error::new(Kind::Syntax, self.at));
            }
            self.at += 1;
            self.depth -= 1;
            return Ok(result);
        }
        self.literal()
    }
    fn literal(&mut self) -> Result<Evaluation, Error> {
        self.space();
        let start = self.at;
        let mut integer = 0i128;
        let mut digits = 0;
        while let Some(b @ b'0'..=b'9') = self.bytes.get(self.at).copied() {
            integer = integer
                .checked_mul(10)
                .and_then(|n| n.checked_add(i128::from(b - b'0')))
                .filter(|n| *n <= MAX_RAW / SCALE)
                .ok_or(Error::new(Kind::Range, start))?;
            self.at += 1;
            digits += 1;
        }
        let mut fraction = 0i128;
        let mut places = 0;
        let mut first_discarded = 0;
        let mut later_nonzero = false;
        if matches!(self.bytes.get(self.at), Some(b'.' | b',')) {
            self.at += 1;
            while let Some(b @ b'0'..=b'9') = self.bytes.get(self.at).copied() {
                if places < 9 {
                    fraction = fraction * 10 + i128::from(b - b'0');
                } else if places == 9 {
                    first_discarded = b - b'0';
                } else {
                    later_nonzero |= b != b'0';
                }
                places += 1;
                self.at += 1;
                digits += 1;
            }
        }
        if digits == 0 {
            return Err(Error::new(Kind::Syntax, start));
        }
        for _ in places..9 {
            fraction *= 10;
        }
        let mut raw = integer * SCALE + fraction;
        if first_discarded > 5 || first_discarded == 5 && (later_nonzero || raw % 2 != 0) {
            raw += 1;
        }
        Ok(Evaluation {
            value: bounded(raw, start)?,
            rounded: first_discarded != 0 || later_nonzero,
        })
    }
}
