//! Minimal JSON value parser/serializer (cgap-mcp internal; zero-dep law).

#[derive(Debug, Clone, PartialEq)]
pub enum Json {
    Null,
    Bool(bool),
    Num(f64),
    Str(String),
    Arr(Vec<Json>),
    Obj(Vec<(String, Json)>),
}

impl Json {
    pub fn get(&self, key: &str) -> Option<&Json> {
        match self {
            Json::Obj(m) => m.iter().find(|(k, _)| k == key).map(|(_, v)| v),
            _ => None,
        }
    }
    pub fn is_null(&self) -> bool {
        matches!(self, Json::Null)
    }
    pub fn as_str(&self) -> Option<&str> {
        if let Json::Str(s) = self {
            Some(s)
        } else {
            None
        }
    }
    pub fn obj(pairs: Vec<(&str, Json)>) -> Json {
        Json::Obj(pairs.into_iter().map(|(k, v)| (k.to_string(), v)).collect())
    }
    pub fn s(v: &str) -> Json {
        Json::Str(v.to_string())
    }
}

pub fn parse(text: &str) -> Result<Json, String> {
    let b: Vec<char> = text.chars().collect();
    let mut p = Parser { b, i: 0 };
    p.skip_ws();
    let v = p.value()?;
    p.skip_ws();
    if p.i != p.b.len() {
        return Err("trailing characters after JSON value".into());
    }
    Ok(v)
}

struct Parser {
    b: Vec<char>,
    i: usize,
}

impl Parser {
    fn skip_ws(&mut self) {
        while self.i < self.b.len() && self.b[self.i].is_whitespace() {
            self.i += 1;
        }
    }
    fn peek(&self) -> Option<char> {
        self.b.get(self.i).copied()
    }
    fn eat(&mut self, c: char) -> Result<(), String> {
        if self.peek() == Some(c) {
            self.i += 1;
            Ok(())
        } else {
            Err(format!("expected '{}'", c))
        }
    }
    fn next_char(&mut self) -> Result<char, String> {
        match self.peek() {
            Some(c) => {
                self.i += 1;
                Ok(c)
            }
            None => Err("unexpected end of input".into()),
        }
    }
    fn value(&mut self) -> Result<Json, String> {
        match self.peek() {
            Some('{') => self.object(),
            Some('[') => self.array(),
            Some('"') => Ok(Json::Str(self.string()?)),
            Some('t') => self.lit("true", Json::Bool(true)),
            Some('f') => self.lit("false", Json::Bool(false)),
            Some('n') => self.lit("null", Json::Null),
            Some(_) => self.number(),
            None => Err("unexpected end of input".into()),
        }
    }
    fn lit(&mut self, s: &str, v: Json) -> Result<Json, String> {
        for c in s.chars() {
            self.eat(c)?;
        }
        Ok(v)
    }
    fn object(&mut self) -> Result<Json, String> {
        self.eat('{')?;
        let mut m = Vec::new();
        self.skip_ws();
        if self.peek() == Some('}') {
            self.i += 1;
            return Ok(Json::Obj(m));
        }
        loop {
            self.skip_ws();
            let k = self.string()?;
            self.skip_ws();
            self.eat(':')?;
            self.skip_ws();
            let v = self.value()?;
            m.push((k, v));
            self.skip_ws();
            match self.peek() {
                Some(',') => self.i += 1,
                Some('}') => {
                    self.i += 1;
                    return Ok(Json::Obj(m));
                }
                _ => return Err("expected ',' or '}' in object".into()),
            }
        }
    }
    fn array(&mut self) -> Result<Json, String> {
        self.eat('[')?;
        let mut a = Vec::new();
        self.skip_ws();
        if self.peek() == Some(']') {
            self.i += 1;
            return Ok(Json::Arr(a));
        }
        loop {
            self.skip_ws();
            a.push(self.value()?);
            self.skip_ws();
            match self.peek() {
                Some(',') => self.i += 1,
                Some(']') => {
                    self.i += 1;
                    return Ok(Json::Arr(a));
                }
                _ => return Err("expected ',' or ']' in array".into()),
            }
        }
    }
    fn string(&mut self) -> Result<String, String> {
        self.eat('"')?;
        let mut out = String::new();
        loop {
            match self.next_char()? {
                '"' => return Ok(out),
                '\\' => out.push(self.escape()?),
                c => out.push(c),
            }
        }
    }
    fn escape(&mut self) -> Result<char, String> {
        match self.next_char()? {
            'n' => Ok('\n'),
            't' => Ok('\t'),
            'r' => Ok('\r'),
            'b' => Ok('\u{8}'),
            'f' => Ok('\u{c}'),
            '"' => Ok('"'),
            '\\' => Ok('\\'),
            '/' => Ok('/'),
            'u' => self.unicode_escape(),
            c => Err(format!("invalid escape '\\{}'", c)),
        }
    }
    fn unicode_escape(&mut self) -> Result<char, String> {
        let mut v: u32 = 0;
        for _ in 0..4 {
            let c = self.next_char()?;
            let d = c.to_digit(16).ok_or_else(|| format!("bad hex '{}'", c))?;
            v = v * 16 + d;
        }
        char::from_u32(v).ok_or_else(|| "invalid unicode scalar".into())
    }
    fn number(&mut self) -> Result<Json, String> {
        let start = self.i;
        while let Some(c) = self.peek() {
            if c.is_ascii_digit() || matches!(c, '-' | '+' | '.' | 'e' | 'E') {
                self.i += 1;
            } else {
                break;
            }
        }
        let s: String = self.b[start..self.i].iter().collect();
        s.parse::<f64>()
            .map(Json::Num)
            .map_err(|_| format!("bad number '{}'", s))
    }
}

pub fn to_string(v: &Json) -> String {
    let mut s = String::new();
    write(v, &mut s);
    s
}

fn write(v: &Json, out: &mut String) {
    match v {
        Json::Null => out.push_str("null"),
        Json::Bool(b) => out.push_str(if *b { "true" } else { "false" }),
        Json::Num(n) => {
            if n.fract() == 0.0 && n.abs() < 9.0e15 {
                out.push_str(&format!("{}", *n as i64));
            } else {
                out.push_str(&format!("{}", n));
            }
        }
        Json::Str(s) => write_str(s, out),
        Json::Arr(a) => {
            out.push('[');
            for (i, x) in a.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                write(x, out);
            }
            out.push(']');
        }
        Json::Obj(m) => {
            out.push('{');
            for (i, (k, x)) in m.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                write_str(k, out);
                out.push(':');
                write(x, out);
            }
            out.push('}');
        }
    }
}

fn write_str(s: &str, out: &mut String) {
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
}
