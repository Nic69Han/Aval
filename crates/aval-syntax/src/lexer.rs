//! Hand-written lexer. Implements the newline rules of spec section 4.

use crate::diag::{Diag, Pos};

/// The 46 reserved words of spec section 2.
pub const KEYWORDS: [&str; 46] = [
    "fn", "let", "return", "type", "enum", "module", "import", "pub", "if", "else", "match", "for",
    "in", "par", "requires", "ensures", "result", "examples", "oracle", "effects", "alloc", "cost",
    "latency", "reuse", "none", "resource", "using", "unsafe", "true", "false", "and", "or", "not",
    "Int", "Float", "Bool", "String", "Byte", "List", "Set", "Map", "Queue", "Image", "Tensor",
    "Result", "Arena",
];

/// Reserved words that name types.
pub const TYPE_KEYWORDS: [&str; 13] = [
    "Int", "Float", "Bool", "String", "Byte", "List", "Set", "Map", "Queue", "Image", "Tensor",
    "Result", "Arena",
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TokKind {
    Ident,
    TypeIdent,
    Keyword,
    Int,
    Float,
    Str,
    Dur,
    Sym,
    Nl,
    Eof,
    Underscore,
}

#[derive(Clone, Debug)]
pub struct Token {
    pub kind: TokKind,
    /// Source text; for strings the decoded content.
    pub text: String,
    pub pos: Pos,
    pub start: usize,
    pub end: usize,
}

impl Token {
    /// Canonical form used by the spec-lock hash.
    pub fn canon(&self) -> String {
        let k = match self.kind {
            TokKind::Ident | TokKind::Underscore => 'I',
            TokKind::TypeIdent => 'T',
            TokKind::Keyword => 'K',
            TokKind::Int => 'N',
            TokKind::Float => 'F',
            TokKind::Str => 'S',
            TokKind::Dur => 'D',
            TokKind::Sym => 'P',
            TokKind::Nl => 'L',
            TokKind::Eof => 'E',
        };
        format!("{}{}:{}", k, self.text.len(), self.text)
    }
}

pub struct LexOut {
    pub tokens: Vec<Token>,
    pub errors: Vec<Diag>,
}

struct Lexer<'a> {
    src: &'a str,
    chars: Vec<(usize, char)>,
    j: usize,
    line: u32,
    col: u32,
    toks: Vec<Token>,
    errs: Vec<Diag>,
    stack: Vec<u8>,
    last_suppress: bool,
}

pub fn lex(src: &str) -> LexOut {
    let mut lx = Lexer {
        src,
        chars: src.char_indices().collect(),
        j: 0,
        line: 1,
        col: 1,
        toks: Vec::new(),
        errs: Vec::new(),
        stack: Vec::new(),
        last_suppress: false,
    };
    lx.run();
    LexOut {
        tokens: lx.toks,
        errors: lx.errs,
    }
}

fn is_word_start(c: char) -> bool {
    c.is_ascii_alphabetic() || c == '_'
}

fn is_word_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_'
}

impl<'a> Lexer<'a> {
    fn peek(&self, n: usize) -> Option<char> {
        self.chars.get(self.j + n).map(|c| c.1)
    }

    fn off(&self, j: usize) -> usize {
        self.chars.get(j).map(|c| c.0).unwrap_or(self.src.len())
    }

    fn bump(&mut self) -> Option<char> {
        let c = self.chars.get(self.j)?.1;
        self.j += 1;
        if c == '\n' {
            self.line += 1;
            self.col = 1;
        } else {
            self.col += 1;
        }
        Some(c)
    }

    fn pos(&self) -> Pos {
        Pos::new(self.line, self.col)
    }

    fn push(&mut self, kind: TokKind, text: String, pos: Pos, start: usize, suppress: bool) {
        let end = self.off(self.j);
        self.toks.push(Token {
            kind,
            text,
            pos,
            start,
            end,
        });
        self.last_suppress = suppress;
    }

    fn err(&mut self, pos: Pos, msg: &str) {
        self.errs.push(Diag::new("E001", pos, msg));
    }

    fn run(&mut self) {
        while let Some(c) = self.peek(0) {
            match c {
                ' ' | '\t' | '\r' => {
                    self.bump();
                }
                '\n' => {
                    self.newline();
                    self.bump();
                }
                '#' => {
                    while let Some(c) = self.peek(0) {
                        if c == '\n' {
                            break;
                        }
                        self.bump();
                    }
                }
                '"' => self.string(),
                c if c.is_ascii_digit() => self.number(),
                c if is_word_start(c) => self.word(),
                _ => self.symbol(c),
            }
        }
        let pos = self.pos();
        let at = self.src.len();
        if let Some(last) = self.toks.last() {
            if last.kind != TokKind::Nl {
                self.toks.push(Token {
                    kind: TokKind::Nl,
                    text: "NL".into(),
                    pos,
                    start: at,
                    end: at,
                });
            }
        }
        self.toks.push(Token {
            kind: TokKind::Eof,
            text: "EOF".into(),
            pos,
            start: at,
            end: at,
        });
    }

    fn newline(&mut self) {
        if self.stack.last() == Some(&b'p') {
            return;
        }
        match self.toks.last() {
            None => return,
            Some(t) if t.kind == TokKind::Nl => return,
            _ => {}
        }
        if self.last_suppress {
            return;
        }
        let pos = self.pos();
        let start = self.off(self.j);
        self.toks.push(Token {
            kind: TokKind::Nl,
            text: "NL".into(),
            pos,
            start,
            end: start + 1,
        });
    }

    fn word(&mut self) {
        let pos = self.pos();
        let sj = self.j;
        let start = self.off(sj);
        while matches!(self.peek(0), Some(c) if is_word_char(c)) {
            self.bump();
        }
        let text = &self.src[start..self.off(self.j)];
        let text = text.to_string();
        let first = text.chars().next().unwrap_or('_');
        if text == "_" {
            self.push(TokKind::Underscore, text, pos, start, false);
            return;
        }
        if first.is_ascii_uppercase() {
            if text.contains('_') {
                self.err(pos, "invalid identifier: PascalCase has no underscore");
                return;
            }
            let kind = if KEYWORDS.contains(&text.as_str()) {
                TokKind::Keyword
            } else {
                TokKind::TypeIdent
            };
            self.push(kind, text, pos, start, false);
        } else {
            if text.chars().any(|c| c.is_ascii_uppercase()) {
                self.err(pos, "invalid identifier: snake_case expected");
                return;
            }
            let kind = if KEYWORDS.contains(&text.as_str()) {
                TokKind::Keyword
            } else {
                TokKind::Ident
            };
            let suppress = matches!(text.as_str(), "and" | "or" | "in");
            self.push(kind, text, pos, start, suppress);
        }
    }

    fn consume_word_tail(&mut self) {
        while matches!(self.peek(0), Some(c) if is_word_char(c) || c == '.') {
            self.bump();
        }
    }

    fn number(&mut self) {
        let pos = self.pos();
        let start = self.off(self.j);
        while matches!(self.peek(0), Some(c) if c.is_ascii_digit()) {
            self.bump();
        }
        let mut is_float = false;
        if self.peek(0) == Some('.') {
            match self.peek(1) {
                Some(c) if c.is_ascii_digit() => {
                    self.bump();
                    while matches!(self.peek(0), Some(c) if c.is_ascii_digit()) {
                        self.bump();
                    }
                    is_float = true;
                    if matches!(self.peek(0), Some('e') | Some('E')) {
                        let k = if matches!(self.peek(1), Some('+') | Some('-')) {
                            2
                        } else {
                            1
                        };
                        if matches!(self.peek(k), Some(c) if c.is_ascii_digit()) {
                            for _ in 0..k {
                                self.bump();
                            }
                            while matches!(self.peek(0), Some(c) if c.is_ascii_digit()) {
                                self.bump();
                            }
                        } else {
                            self.err(pos, "malformed number: exponent needs digits");
                            self.consume_word_tail();
                            return;
                        }
                    }
                }
                Some('.') => {}
                Some(c) if is_word_start(c) => {}
                _ => {
                    self.bump();
                    self.err(pos, "malformed number: digits required after '.'");
                    return;
                }
            }
        }
        if matches!(self.peek(0), Some(c) if is_word_start(c)) {
            if !is_float {
                let mut k = 0;
                while matches!(self.peek(k), Some(c) if is_word_char(c)) {
                    k += 1;
                }
                let suffix: String = (0..k).filter_map(|i| self.peek(i)).collect();
                if matches!(suffix.as_str(), "ns" | "us" | "ms" | "s") {
                    for _ in 0..k {
                        self.bump();
                    }
                    let text = self.src[start..self.off(self.j)].to_string();
                    self.push(TokKind::Dur, text, pos, start, false);
                    return;
                }
            }
            self.err(pos, "malformed number");
            self.consume_word_tail();
            return;
        }
        let text = self.src[start..self.off(self.j)].to_string();
        if !is_float && text.parse::<i64>().is_err() {
            self.err(pos, "integer literal too large");
            return;
        }
        let kind = if is_float {
            TokKind::Float
        } else {
            TokKind::Int
        };
        self.push(kind, text, pos, start, false);
    }

    fn string(&mut self) {
        let pos = self.pos();
        let start = self.off(self.j);
        self.bump();
        let mut out = String::new();
        loop {
            match self.peek(0) {
                None | Some('\n') => {
                    self.err(pos, "unterminated string");
                    return;
                }
                Some('"') => {
                    self.bump();
                    break;
                }
                Some('\\') => {
                    let epos = self.pos();
                    self.bump();
                    match self.peek(0) {
                        Some('"') => out.push('"'),
                        Some('\\') => out.push('\\'),
                        Some('n') => out.push('\n'),
                        Some('t') => out.push('\t'),
                        Some('\n') | None => {
                            self.err(pos, "unterminated string");
                            return;
                        }
                        Some(_) => self.err(epos, "invalid escape sequence"),
                    }
                    self.bump();
                }
                Some(c) => {
                    out.push(c);
                    self.bump();
                }
            }
        }
        self.push(TokKind::Str, out, pos, start, false);
    }

    fn symbol(&mut self, c: char) {
        let pos = self.pos();
        let start = self.off(self.j);
        let c1 = self.peek(1);
        let c2 = self.peek(2);
        let (text, suppress): (&str, bool) = match (c, c1, c2) {
            ('.', Some('.'), Some('=')) => ("..=", true),
            ('.', Some('.'), _) => ("..", true),
            ('-', Some('>'), _) => ("->", true),
            ('=', Some('>'), _) => ("=>", true),
            ('=', Some('='), _) => ("==", true),
            ('!', Some('='), _) => ("!=", true),
            ('<', Some('='), _) => ("<=", true),
            ('>', Some('='), _) => (">=", true),
            ('(', _, _) => ("(", false),
            (')', _, _) => (")", false),
            ('[', _, _) => ("[", false),
            (']', _, _) => ("]", false),
            ('{', _, _) => ("{", true),
            ('}', _, _) => ("}", false),
            (',', _, _) => (",", true),
            (':', _, _) => (":", false),
            ('.', _, _) => (".", false),
            ('=', _, _) => ("=", true),
            ('<', _, _) => ("<", true),
            ('>', _, _) => (">", true),
            ('+', _, _) => ("+", true),
            ('-', _, _) => ("-", true),
            ('*', _, _) => ("*", true),
            ('/', _, _) => ("/", true),
            ('%', _, _) => ("%", true),
            ('?', _, _) => ("?", false),
            ('|', _, _) => ("|", false),
            _ => {
                self.bump();
                self.err(pos, &format!("invalid character `{}`", c));
                return;
            }
        };
        let mut suppress = suppress;
        match text {
            "(" | "[" => self.stack.push(b'p'),
            "{" => self.stack.push(b'b'),
            ")" | "]" => {
                while let Some(t) = self.stack.pop() {
                    if t == b'p' {
                        break;
                    }
                }
            }
            "}" => {
                while let Some(t) = self.stack.pop() {
                    if t == b'b' {
                        break;
                    }
                }
            }
            "<" => {
                let generic = match self.toks.last() {
                    Some(t) => {
                        (t.end == start && t.kind == TokKind::TypeIdent)
                            || (t.kind == TokKind::Keyword
                                && TYPE_KEYWORDS.contains(&t.text.as_str()))
                    }
                    None => false,
                };
                if generic {
                    self.stack.push(b'a');
                    suppress = false;
                }
            }
            ">" if self.stack.last() == Some(&b'a') => {
                self.stack.pop();
                suppress = false;
            }
            _ => {}
        }
        for _ in 0..text.len() {
            self.bump();
        }
        self.push(TokKind::Sym, text.to_string(), pos, start, suppress);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn kinds(src: &str) -> Vec<String> {
        lex(src)
            .tokens
            .iter()
            .map(|t| format!("{:?}:{}", t.kind, t.text))
            .collect()
    }

    #[test]
    fn keyword_count_is_46() {
        let mut k = KEYWORDS.to_vec();
        k.sort();
        k.dedup();
        assert_eq!(k.len(), 46);
    }

    #[test]
    fn keywords_and_identifiers() {
        let t = kinds("fn while Total x_1 _");
        assert_eq!(t[0], "Keyword:fn");
        assert_eq!(t[1], "Ident:while");
        assert_eq!(t[2], "TypeIdent:Total");
        assert_eq!(t[3], "Ident:x_1");
        assert_eq!(t[4], "Underscore:_");
    }

    #[test]
    fn float_needs_digits_after_dot() {
        assert_eq!(lex("2.\n").errors.len(), 1);
        assert!(lex("2.5 1.5e3").errors.is_empty());
        assert_eq!(lex("1..5").tokens.len(), 5); // 1 .. 5 NL EOF
    }

    #[test]
    fn durations() {
        let t = kinds("2ms 500us 1s 10ns");
        assert_eq!(t[0], "Dur:2ms");
        assert_eq!(t[3], "Dur:10ns");
        assert_eq!(lex("2xs").errors.len(), 1);
    }

    #[test]
    fn strings_and_errors() {
        let ok = lex("\"a\\n\\\"b\"");
        assert!(ok.errors.is_empty());
        assert_eq!(ok.tokens[0].text, "a\n\"b");
        let bad = lex("\"abc\n");
        assert_eq!(bad.errors.len(), 1);
        assert_eq!(bad.errors[0].pos, Pos::new(1, 1));
    }

    #[test]
    fn stray_character() {
        let o = lex("return 1 @ 2");
        assert_eq!(o.errors.len(), 1);
        assert_eq!(o.errors[0].pos, Pos::new(1, 10));
    }

    #[test]
    fn newline_rules() {
        // after an operator and inside brackets newlines vanish
        let t = kinds("1 +\n2\n[1,\n2\n]\n");
        assert!(!t[..3].iter().any(|s| s.starts_with("Nl")));
        let nls = t.iter().filter(|s| s.starts_with("Nl")).count();
        assert_eq!(nls, 2);
        // comments and blank lines collapse
        let t = kinds("a # c\n\n# x\nb\n");
        assert_eq!(t.iter().filter(|s| s.starts_with("Nl")).count(), 2);
    }

    #[test]
    fn space_before_generic_args() {
        let t = kinds("List <Int>\nx");
        assert!(t.iter().any(|s| s.starts_with("Nl")));
    }

    #[test]
    fn canonical_tokens_encode_length() {
        let a = lex("\"a K3:or\"").tokens[0].canon();
        let b = lex("\"a\" or")
            .tokens
            .iter()
            .map(|t| t.canon())
            .collect::<Vec<_>>()
            .join(" ");
        assert_ne!(a, b);
    }

    #[test]
    fn generic_close_keeps_newline() {
        let t = kinds("List<Int>\nx");
        assert!(t.iter().any(|s| s.starts_with("Nl")));
    }
}
