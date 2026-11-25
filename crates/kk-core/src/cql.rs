//! CQL: lexer, parser and validation. Pure; the server turns the AST into SQL.
//!
//! Grammar: `expr [ORDER BY field [ASC|DESC] {, ...}]`, `AND` binds tighter than
//! `OR`, `NOT` and parentheses group, a clause is `field op value`,
//! `field [NOT] IN (v, ...)`. Errors carry the character offset.

use std::fmt;

use chrono::{Datelike, Days, Months, NaiveDate, NaiveDateTime, NaiveTime, Weekday};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Op {
    Eq,
    Ne,
    Contains,
    NotContains,
    Gt,
    Ge,
    Lt,
    Le,
    In,
    NotIn,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Str(String),
    Func(String, Vec<String>),
}

#[derive(Clone, Debug, PartialEq)]
pub enum Expr {
    And(Vec<Expr>),
    Or(Vec<Expr>),
    Not(Box<Expr>),
    Clause {
        field: Field,
        op: Op,
        values: Vec<Value>,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Field {
    Type,
    Space,
    Title,
    Text,
    Label,
    Creator,
    Created,
    LastModified,
    Id,
    Ancestor,
    Parent,
    Status,
    UserFullname,
    UserAccountId,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Kind {
    Text,
    Keyword,
    Date,
    Id,
    User,
}

impl Field {
    fn kind(self) -> Kind {
        match self {
            Self::Title | Self::Text | Self::UserFullname => Kind::Text,
            Self::Created | Self::LastModified => Kind::Date,
            Self::Id | Self::Ancestor | Self::Parent => Kind::Id,
            Self::Creator => Kind::User,
            _ => Kind::Keyword,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::Type => "type",
            Self::Space => "space",
            Self::Title => "title",
            Self::Text => "text",
            Self::Label => "label",
            Self::Creator => "creator",
            Self::Created => "created",
            Self::LastModified => "lastModified",
            Self::Id => "id",
            Self::Ancestor => "ancestor",
            Self::Parent => "parent",
            Self::Status => "status",
            Self::UserFullname => "user.fullname",
            Self::UserAccountId => "user.accountid",
        }
    }
}

fn field_named(name: &str) -> Option<Field> {
    Some(match name.to_ascii_lowercase().as_str() {
        "type" => Field::Type,
        "space" | "space.key" => Field::Space,
        "title" => Field::Title,
        "text" => Field::Text,
        "label" => Field::Label,
        "creator" => Field::Creator,
        "created" => Field::Created,
        "lastmodified" => Field::LastModified,
        "id" | "content" => Field::Id,
        "ancestor" => Field::Ancestor,
        "parent" => Field::Parent,
        "status" => Field::Status,
        "user.fullname" => Field::UserFullname,
        "user.accountid" => Field::UserAccountId,
        _ => return None,
    })
}

/// Fields the grammar names but this build cannot evaluate.
const UNSUPPORTED: &[&str] = &[
    "favourite",
    "favouriteof",
    "favorite",
    "favoriteof",
    "watcher",
    "mention",
    "contributor",
    "macro",
    "instancetype",
    "space.type",
    "container",
    "creator.accountid",
];

#[derive(Clone, Debug, PartialEq)]
pub struct Query {
    pub expr: Expr,
    /// `(field, descending)`.
    pub order: Vec<(Field, bool)>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CqlError {
    Syntax { pos: usize, msg: String },
    UnknownField { pos: usize, name: String },
    Unsupported { pos: usize, name: String },
    BadOperator { pos: usize, field: &'static str },
    BadValue { pos: usize, msg: String },
    UnknownFunction { pos: usize, name: String },
}

impl fmt::Display for CqlError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Syntax { pos, msg } => write!(f, "Could not parse CQL at position {pos}: {msg}"),
            Self::UnknownField { pos, name } => {
                write!(f, "Field '{name}' does not exist (position {pos})")
            }
            Self::Unsupported { pos, name } => {
                write!(f, "Field '{name}' is not supported (position {pos})")
            }
            Self::BadOperator { pos, field } => write!(
                f,
                "Operator is not valid for field '{field}' (position {pos})"
            ),
            Self::BadValue { pos, msg } => write!(f, "{msg} (position {pos})"),
            Self::UnknownFunction { pos, name } => {
                write!(f, "Function '{name}' does not exist (position {pos})")
            }
        }
    }
}

impl std::error::Error for CqlError {}

#[derive(Clone, Debug, PartialEq)]
enum Tok {
    Word(String),
    Str(String),
    Op(Op),
    LParen,
    RParen,
    Comma,
}

fn lex(src: &str) -> Result<Vec<(Tok, usize)>, CqlError> {
    let cs: Vec<char> = src.chars().collect();
    let (mut i, mut out) = (0, Vec::new());
    let err = |pos, msg: &str| CqlError::Syntax {
        pos,
        msg: msg.into(),
    };
    while i < cs.len() {
        let c = cs[i];
        let at = i;
        let next = cs.get(i + 1).copied();
        match c {
            c if c.is_whitespace() => i += 1,
            '(' | ')' | ',' => {
                out.push((
                    match c {
                        '(' => Tok::LParen,
                        ')' => Tok::RParen,
                        _ => Tok::Comma,
                    },
                    at,
                ));
                i += 1;
            }
            '"' | '\'' => {
                let mut s = String::new();
                i += 1;
                loop {
                    match cs.get(i) {
                        None => return Err(err(at, "unterminated string")),
                        Some('\\') => {
                            s.push(
                                *cs.get(i + 1)
                                    .ok_or_else(|| err(at, "unterminated string"))?,
                            );
                            i += 2;
                        }
                        Some(&q) if q == c => {
                            i += 1;
                            break;
                        }
                        Some(&ch) => {
                            s.push(ch);
                            i += 1;
                        }
                    }
                }
                out.push((Tok::Str(s), at));
            }
            '=' | '~' | '!' | '<' | '>' => {
                let (op, n) = match (c, next) {
                    ('=', _) => (Op::Eq, 1),
                    ('~', _) => (Op::Contains, 1),
                    ('!', Some('=')) => (Op::Ne, 2),
                    ('!', Some('~')) => (Op::NotContains, 2),
                    ('<', Some('=')) => (Op::Le, 2),
                    ('>', Some('=')) => (Op::Ge, 2),
                    ('<', _) => (Op::Lt, 1),
                    ('>', _) => (Op::Gt, 1),
                    _ => return Err(err(at, "unexpected '!'")),
                };
                out.push((Tok::Op(op), at));
                i += n;
            }
            _ => {
                let mut w = String::new();
                while let Some(&ch) = cs.get(i) {
                    if ch.is_whitespace() || "(),=~!<>\"'".contains(ch) {
                        break;
                    }
                    w.push(ch);
                    i += 1;
                }
                out.push((Tok::Word(w), at));
            }
        }
    }
    Ok(out)
}

struct P<'a> {
    toks: &'a [(Tok, usize)],
    i: usize,
    end: usize,
}

impl P<'_> {
    fn pos(&self) -> usize {
        self.toks.get(self.i).map_or(self.end, |t| t.1)
    }

    fn peek(&self) -> Option<&Tok> {
        self.toks.get(self.i).map(|t| &t.0)
    }

    fn kw(&self, k: &str) -> bool {
        matches!(self.peek(), Some(Tok::Word(w)) if w.eq_ignore_ascii_case(k))
    }

    fn bad(&self, msg: &str) -> CqlError {
        CqlError::Syntax {
            pos: self.pos(),
            msg: msg.into(),
        }
    }

    fn or(&mut self) -> Result<Expr, CqlError> {
        let mut v = vec![self.and()?];
        while self.kw("or") {
            self.i += 1;
            v.push(self.and()?);
        }
        Ok(if v.len() == 1 {
            v.remove(0)
        } else {
            Expr::Or(v)
        })
    }

    fn and(&mut self) -> Result<Expr, CqlError> {
        let mut v = vec![self.not()?];
        while self.kw("and") {
            self.i += 1;
            v.push(self.not()?);
        }
        Ok(if v.len() == 1 {
            v.remove(0)
        } else {
            Expr::And(v)
        })
    }

    fn not(&mut self) -> Result<Expr, CqlError> {
        if self.kw("not") {
            self.i += 1;
            return Ok(Expr::Not(Box::new(self.not()?)));
        }
        if self.peek() == Some(&Tok::LParen) {
            self.i += 1;
            let e = self.or()?;
            if self.peek() != Some(&Tok::RParen) {
                return Err(self.bad("expected ')'"));
            }
            self.i += 1;
            return Ok(e);
        }
        self.clause()
    }

    fn field(&mut self) -> Result<Field, CqlError> {
        let pos = self.pos();
        let Some(Tok::Word(w)) = self.peek() else {
            return Err(self.bad("expected a field name"));
        };
        let w = w.clone();
        self.i += 1;
        if let Some(f) = field_named(&w) {
            return Ok(f);
        }
        let l = w.to_ascii_lowercase();
        Err(if UNSUPPORTED.contains(&l.as_str()) {
            CqlError::Unsupported { pos, name: w }
        } else {
            CqlError::UnknownField { pos, name: w }
        })
    }

    fn value(&mut self) -> Result<Value, CqlError> {
        let pos = self.pos();
        match self.peek().cloned() {
            Some(Tok::Str(s)) => {
                self.i += 1;
                Ok(Value::Str(s))
            }
            Some(Tok::Word(w)) => {
                self.i += 1;
                if self.peek() != Some(&Tok::LParen) {
                    return Ok(Value::Str(w));
                }
                self.i += 1;
                let mut args = Vec::new();
                while self.peek() != Some(&Tok::RParen) {
                    match self.peek().cloned() {
                        Some(Tok::Str(s) | Tok::Word(s)) => args.push(s),
                        _ => return Err(self.bad("expected a function argument or ')'")),
                    }
                    self.i += 1;
                    if self.peek() == Some(&Tok::Comma) {
                        self.i += 1;
                    }
                }
                self.i += 1;
                if !FUNCS.contains(&w.to_ascii_lowercase().as_str()) {
                    return Err(CqlError::UnknownFunction { pos, name: w });
                }
                Ok(Value::Func(w, args))
            }
            _ => Err(self.bad("expected a value")),
        }
    }

    fn clause(&mut self) -> Result<Expr, CqlError> {
        let field = self.field()?;
        let pos = self.pos();
        let op = match self.peek().cloned() {
            Some(Tok::Op(o)) => {
                self.i += 1;
                o
            }
            Some(Tok::Word(_)) if self.kw("in") => {
                self.i += 1;
                Op::In
            }
            Some(Tok::Word(_)) if self.kw("not") => {
                self.i += 1;
                if !self.kw("in") {
                    return Err(self.bad("expected IN after NOT"));
                }
                self.i += 1;
                Op::NotIn
            }
            _ => return Err(self.bad("expected an operator")),
        };
        let allowed: &[Op] = match field.kind() {
            Kind::Text => &[Op::Eq, Op::Ne, Op::Contains, Op::NotContains],
            Kind::Date => &[Op::Eq, Op::Ne, Op::Gt, Op::Ge, Op::Lt, Op::Le],
            _ => &[Op::Eq, Op::Ne, Op::In, Op::NotIn],
        };
        if !allowed.contains(&op) {
            return Err(CqlError::BadOperator {
                pos,
                field: field.name(),
            });
        }
        let vpos = self.pos();
        let values = if matches!(op, Op::In | Op::NotIn) {
            if self.peek() != Some(&Tok::LParen) {
                return Err(self.bad("expected '(' after IN"));
            }
            self.i += 1;
            let mut v = vec![self.value()?];
            while self.peek() == Some(&Tok::Comma) {
                self.i += 1;
                v.push(self.value()?);
            }
            if self.peek() != Some(&Tok::RParen) {
                return Err(self.bad("expected ')'"));
            }
            self.i += 1;
            v
        } else {
            vec![self.value()?]
        };
        for v in &values {
            check_value(field, v, vpos)?;
        }
        Ok(Expr::Clause { field, op, values })
    }
}

const FUNCS: &[&str] = &[
    "currentuser",
    "now",
    "startofday",
    "startofweek",
    "startofmonth",
    "startofyear",
    "endofday",
    "endofweek",
    "endofmonth",
    "endofyear",
];

fn check_value(field: Field, v: &Value, pos: usize) -> Result<(), CqlError> {
    let bad = |msg: String| CqlError::BadValue { pos, msg };
    match (field.kind(), v) {
        (Kind::Date, v) => date_value(v, chrono::DateTime::UNIX_EPOCH.naive_utc(), pos).map(|_| ()),
        (Kind::Id, Value::Str(s)) if s.parse::<i64>().is_err() => {
            Err(bad(format!("'{s}' is not a valid content id")))
        }
        (Kind::Id, Value::Func(..)) => Err(bad("functions are not valid for ids".into())),
        (Kind::User, Value::Func(n, _)) if !n.eq_ignore_ascii_case("currentuser") => {
            Err(bad(format!("function {n} does not return a user")))
        }
        (Kind::Text | Kind::Keyword, Value::Func(n, _))
            if !(field == Field::UserAccountId && n.eq_ignore_ascii_case("currentuser")) =>
        {
            Err(bad(format!(
                "function {n} is not valid for field '{}'",
                field.name()
            )))
        }
        _ => Ok(()),
    }
}

pub fn parse(src: &str) -> Result<Query, CqlError> {
    let toks = lex(src)?;
    let mut p = P {
        toks: &toks,
        i: 0,
        end: src.chars().count(),
    };
    if toks.is_empty() {
        return Err(p.bad("empty query"));
    }
    let expr = if p.kw("order") { None } else { Some(p.or()?) };
    let mut order = Vec::new();
    if p.kw("order") {
        p.i += 1;
        if !p.kw("by") {
            return Err(p.bad("expected BY after ORDER"));
        }
        p.i += 1;
        loop {
            let pos = p.pos();
            let f = p.field()?;
            if !matches!(
                f,
                Field::Title | Field::Created | Field::LastModified | Field::Id
            ) {
                return Err(CqlError::BadValue {
                    pos,
                    msg: format!("cannot order by '{}'", f.name()),
                });
            }
            let desc = if p.kw("desc") {
                p.i += 1;
                true
            } else {
                if p.kw("asc") {
                    p.i += 1;
                }
                false
            };
            order.push((f, desc));
            if p.peek() == Some(&Tok::Comma) {
                p.i += 1;
            } else {
                break;
            }
        }
    }
    if p.i < toks.len() {
        return Err(p.bad("unexpected input"));
    }
    // An ORDER BY-only query matches everything: `id != 0` is true for every row.
    let expr = expr.unwrap_or(Expr::Clause {
        field: Field::Id,
        op: Op::Ne,
        values: vec![Value::Str("0".into())],
    });
    Ok(Query { expr, order })
}

/// Resolves a date literal, relative offset or date function against `now` (UTC).
pub fn date_value(v: &Value, now: NaiveDateTime, pos: usize) -> Result<NaiveDateTime, CqlError> {
    let bad = |m: &str| CqlError::BadValue { pos, msg: m.into() };
    match v {
        Value::Func(name, args) => {
            let off = match args.first() {
                Some(a) => offset(now, a).ok_or_else(|| bad("invalid date offset"))?,
                None => now,
            };
            let l = name.to_ascii_lowercase();
            let day = off.date();
            let mid = |d: NaiveDate| d.and_time(NaiveTime::MIN);
            let end = |d: NaiveDate| d.and_hms_opt(23, 59, 59).unwrap();
            let week = day.week(Weekday::Mon);
            let month_first = day.with_day(1).unwrap();
            let month_last = month_first + Months::new(1) - Days::new(1);
            Ok(match l.as_str() {
                "now" => off,
                "startofday" => mid(day),
                "endofday" => end(day),
                "startofweek" => mid(week.first_day()),
                "endofweek" => end(week.last_day()),
                "startofmonth" => mid(month_first),
                "endofmonth" => end(month_last),
                "startofyear" => mid(NaiveDate::from_ymd_opt(day.year(), 1, 1).unwrap()),
                "endofyear" => end(NaiveDate::from_ymd_opt(day.year(), 12, 31).unwrap()),
                _ => {
                    return Err(CqlError::UnknownFunction {
                        pos,
                        name: name.clone(),
                    })
                }
            })
        }
        Value::Str(s) => {
            if let Some(t) = offset(now, s) {
                return Ok(t);
            }
            for f in ["%Y-%m-%d %H:%M", "%Y/%m/%d %H:%M"] {
                if let Ok(t) = NaiveDateTime::parse_from_str(s, f) {
                    return Ok(t);
                }
            }
            for f in ["%Y-%m-%d", "%Y/%m/%d"] {
                if let Ok(d) = NaiveDate::parse_from_str(s, f) {
                    return Ok(d.and_time(NaiveTime::MIN));
                }
            }
            Err(bad(&format!("'{s}' is not a valid date")))
        }
    }
}

/// `-4w`, `+2d`, `1M`: units y, M, w, d, h, m.
fn offset(now: NaiveDateTime, s: &str) -> Option<NaiveDateTime> {
    let unit = s.chars().last()?;
    let num = &s[..s.len() - unit.len_utf8()];
    let n: i64 = num.strip_prefix('+').unwrap_or(num).parse().ok()?;
    let m = n.unsigned_abs();
    let back = n < 0;
    let mo = |k: u64| {
        let k = u32::try_from(k).ok()?;
        if back {
            now.checked_sub_months(Months::new(k))
        } else {
            now.checked_add_months(Months::new(k))
        }
    };
    let dur =
        |secs: i64| now.checked_add_signed(chrono::Duration::try_seconds(secs.checked_mul(n)?)?);
    match unit {
        'y' => mo(m.checked_mul(12)?),
        'M' => mo(m),
        'w' => dur(7 * 86400),
        'd' => dur(86400),
        'h' => dur(3600),
        'm' => dur(60),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn clause(f: Field, op: Op, v: &str) -> Expr {
        Expr::Clause {
            field: f,
            op,
            values: vec![Value::Str(v.into())],
        }
    }

    fn at(s: &str) -> NaiveDateTime {
        NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M").unwrap()
    }

    #[test]
    fn and_binds_tighter_than_or() {
        let q = parse("type = page OR space = ENG AND title ~ \"x\"").unwrap();
        assert_eq!(
            q.expr,
            Expr::Or(vec![
                clause(Field::Type, Op::Eq, "page"),
                Expr::And(vec![
                    clause(Field::Space, Op::Eq, "ENG"),
                    clause(Field::Title, Op::Contains, "x")
                ]),
            ])
        );
    }

    #[test]
    fn keywords_case_insensitive_not_in_and_order_by() {
        let q = parse("not (type in (page, 'comment')) and label NOT IN (a,b) order by lastModified desc, title").unwrap();
        assert_eq!(
            q.order,
            vec![(Field::LastModified, true), (Field::Title, false)]
        );
        let Expr::And(v) = q.expr else { panic!() };
        assert!(matches!(&v[0], Expr::Not(_)));
        assert!(matches!(&v[1], Expr::Clause { op: Op::NotIn, values, .. } if values.len() == 2));
    }

    #[test]
    fn string_escapes() {
        let q = parse(r#"title = "a \"b\" c""#).unwrap();
        assert_eq!(q.expr, clause(Field::Title, Op::Eq, "a \"b\" c"));
    }

    #[test]
    fn order_by_only_matches_everything() {
        assert!(parse("order by created").is_ok());
    }

    #[test]
    fn errors_carry_positions_and_kinds() {
        assert!(matches!(
            parse("type = = page"),
            Err(CqlError::Syntax { pos: 7, .. })
        ));
        assert!(matches!(
            parse("foo = 1"),
            Err(CqlError::UnknownField { pos: 0, .. })
        ));
        assert!(matches!(
            parse("type = page AND watcher = currentUser()"),
            Err(CqlError::Unsupported { pos: 16, .. })
        ));
        assert!(matches!(
            parse("title > 3"),
            Err(CqlError::BadOperator { pos: 6, .. })
        ));
        assert!(matches!(
            parse("created > \"nope\""),
            Err(CqlError::BadValue { .. })
        ));
        assert!(matches!(
            parse("created > bogus(1)"),
            Err(CqlError::UnknownFunction { .. })
        ));
        assert!(matches!(parse("id = abc"), Err(CqlError::BadValue { .. })));
        assert!(matches!(
            parse("(type = page"),
            Err(CqlError::Syntax { .. })
        ));
        assert!(matches!(
            parse("type = page extra"),
            Err(CqlError::Syntax { .. })
        ));
        assert!(matches!(
            parse("title = \"open"),
            Err(CqlError::Syntax { pos: 8, .. })
        ));
        assert!(parse("").is_err());
        assert!(parse("order by space").is_err());
    }

    #[test]
    fn dates_literals_offsets_and_functions() {
        let now = at("2026-11-25 10:30"); // Wednesday
        let d = |s: &str| date_value(&Value::Str(s.into()), now, 0).unwrap();
        assert_eq!(d("2026-01-02"), at("2026-01-02 00:00"));
        assert_eq!(d("2026/01/02 04:05"), at("2026-01-02 04:05"));
        assert_eq!(d("-1w"), at("2026-11-18 10:30"));
        assert_eq!(d("+2d"), at("2026-11-27 10:30"));
        assert_eq!(d("-1M"), at("2026-10-25 10:30"));
        let f = |n: &str, a: &[&str]| {
            date_value(
                &Value::Func(n.into(), a.iter().map(|s| s.to_string()).collect()),
                now,
                0,
            )
            .unwrap()
        };
        assert_eq!(f("startOfWeek", &[]), at("2026-11-23 00:00"));
        assert_eq!(
            f("endOfMonth", &[]),
            at("2026-11-30 23:59") + chrono::Duration::seconds(59)
        );
        assert_eq!(f("startOfYear", &["-1y"]), at("2025-01-01 00:00"));
        assert_eq!(f("now", &[]), now);
    }
}
