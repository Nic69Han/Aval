//! Hand-written recursive-descent parser. Syntax errors carry the list of tokens that
//! would have been accepted at the failure point (`expected`).

use crate::ast::*;
use crate::diag::{Diag, Pos};
use crate::lexer::{lex, TokKind, Token, TYPE_KEYWORDS};

pub struct ParseOut {
    pub module: Module,
    pub diags: Vec<Diag>,
}

type PResult<T> = Result<T, Diag>;

const CLAUSES: [&str; 8] = [
    "requires", "ensures", "examples", "oracle", "effects", "alloc", "cost", "latency",
];

pub fn parse(src: &str) -> ParseOut {
    let lexed = lex(src);
    let mut module = Module {
        name: "?".into(),
        name_pos: Pos::new(1, 1),
        header_pos: Pos::new(1, 1),
        imports: Vec::new(),
        items: Vec::new(),
        broken: Vec::new(),
    };
    if !lexed.errors.is_empty() {
        return ParseOut {
            module,
            diags: lexed.errors,
        };
    }
    let mut p = Parser {
        src,
        toks: lexed.tokens,
        i: 0,
        diags: Vec::new(),
        cur_fn: None,
        cur_item: None,
        no_struct: false,
        next_id: 0,
        last_end: 0,
    };
    p.parse_module(&mut module);
    ParseOut {
        module,
        diags: p.diags,
    }
}

struct Parser<'a> {
    src: &'a str,
    toks: Vec<Token>,
    i: usize,
    diags: Vec<Diag>,
    cur_fn: Option<String>,
    cur_item: Option<String>,
    no_struct: bool,
    next_id: u32,
    last_end: usize,
}

impl<'a> Parser<'a> {
    // ---- token helpers -------------------------------------------------

    fn cur(&self) -> &Token {
        &self.toks[self.i.min(self.toks.len() - 1)]
    }

    fn nth(&self, n: usize) -> &Token {
        &self.toks[(self.i + n).min(self.toks.len() - 1)]
    }

    fn bump(&mut self) -> Token {
        let t = self.cur().clone();
        if self.i < self.toks.len() - 1 {
            self.i += 1;
        }
        if t.kind != TokKind::Nl {
            self.last_end = t.end;
        }
        t
    }

    fn is_sym(&self, s: &str) -> bool {
        let t = self.cur();
        t.kind == TokKind::Sym && t.text == s
    }

    fn is_kw(&self, s: &str) -> bool {
        let t = self.cur();
        t.kind == TokKind::Keyword && t.text == s
    }

    fn eat_sym(&mut self, s: &str) -> bool {
        if self.is_sym(s) {
            self.bump();
            true
        } else {
            false
        }
    }

    fn eat_kw(&mut self, s: &str) -> bool {
        if self.is_kw(s) {
            self.bump();
            true
        } else {
            false
        }
    }

    fn eat_nl(&mut self) {
        while self.cur().kind == TokKind::Nl {
            self.bump();
        }
    }

    fn fail(&self, expected: &[&str]) -> Diag {
        let t = self.cur();
        let what = match t.kind {
            TokKind::Eof => "end of file".to_string(),
            TokKind::Nl => "end of line".to_string(),
            TokKind::Str => "string literal".to_string(),
            _ => format!("`{}`", t.text),
        };
        let mut d = Diag::new("E002", t.pos, format!("unexpected {}", what));
        if let Some(f) = &self.cur_fn {
            d = d.with_fn(f);
        }
        d.with(
            "expected",
            serde_json::Value::Array(expected.iter().map(|s| (*s).into()).collect()),
        )
    }

    fn expect_sym(&mut self, s: &str) -> PResult<Token> {
        if self.is_sym(s) {
            Ok(self.bump())
        } else {
            Err(self.fail(&[s]))
        }
    }

    fn expect_kw(&mut self, s: &str) -> PResult<Token> {
        if self.is_kw(s) {
            Ok(self.bump())
        } else {
            Err(self.fail(&[s]))
        }
    }

    fn expect_nl(&mut self) -> PResult<()> {
        if self.cur().kind == TokKind::Nl {
            self.bump();
            Ok(())
        } else {
            Err(self.fail(&["NL"]))
        }
    }

    fn expect_ident(&mut self) -> PResult<(String, Pos)> {
        if self.cur().kind == TokKind::Ident {
            let t = self.bump();
            Ok((t.text, t.pos))
        } else {
            Err(self.fail(&["ident"]))
        }
    }

    fn expect_pascal(&mut self) -> PResult<(String, Pos)> {
        if self.cur().kind == TokKind::TypeIdent {
            let t = self.bump();
            Ok((t.text, t.pos))
        } else {
            Err(self.fail(&["Ident"]))
        }
    }

    fn mk(&mut self, kind: ExprKind, pos: Pos, start: usize) -> Expr {
        self.next_id += 1;
        Expr {
            id: self.next_id,
            pos,
            span: (start, self.last_end.max(start)),
            kind,
        }
    }

    // ---- module and items ---------------------------------------------

    fn parse_module(&mut self, module: &mut Module) {
        self.eat_nl();
        module.header_pos = self.cur().pos;
        if self.is_kw("module") {
            match self.parse_module_header() {
                Ok((name, pos)) => {
                    module.name = name;
                    module.name_pos = pos;
                }
                Err(d) => {
                    self.diags.push(d);
                    self.recover(0);
                }
            }
        } else {
            let d = self.fail(&["module"]);
            self.diags.push(d);
            if self.cur().kind != TokKind::Eof {
                self.recover(0);
            }
        }
        loop {
            self.eat_nl();
            if self.cur().kind == TokKind::Eof {
                break;
            }
            let start = self.i;
            self.cur_fn = None;
            self.cur_item = None;
            let r = if self.is_kw("import") {
                if module.items.is_empty() {
                    self.parse_import().map(|im| module.imports.push(im))
                } else {
                    Err(self.fail(&["pub", "fn", "type", "enum", "resource"]))
                }
            } else {
                self.parse_item().map(|it| module.items.push(it))
            };
            if let Err(d) = r {
                self.diags.push(d);
                if let Some(n) = self.cur_item.take() {
                    module.broken.push(n);
                }
                self.recover(start);
            }
        }
    }

    fn parse_module_header(&mut self) -> PResult<(String, Pos)> {
        self.expect_kw("module")?;
        let pos = self.cur().pos;
        let name = self.parse_path()?;
        self.expect_nl()?;
        Ok((name, pos))
    }

    fn parse_path(&mut self) -> PResult<String> {
        let (mut s, _) = self.expect_ident()?;
        while self.is_sym(".") {
            self.bump();
            let (n, _) = self.expect_ident()?;
            s.push('.');
            s.push_str(&n);
        }
        Ok(s)
    }

    /// Skip to the next line that starts an item.
    fn recover(&mut self, item_start: usize) {
        let mut j = if self.i > item_start {
            self.i
        } else {
            self.i + 1
        };
        while j < self.toks.len() {
            let t = &self.toks[j];
            if t.kind == TokKind::Eof {
                break;
            }
            let starts_item = t.kind == TokKind::Keyword
                && matches!(t.text.as_str(), "fn" | "pub" | "type" | "enum" | "resource");
            if starts_item && j > 0 && self.toks[j - 1].kind == TokKind::Nl {
                break;
            }
            j += 1;
        }
        self.i = j.min(self.toks.len() - 1);
    }

    fn parse_import(&mut self) -> PResult<Import> {
        let t = self.expect_kw("import")?;
        let path = self.parse_path()?;
        if self.eat_sym("{") {
            loop {
                self.expect_pascal()?;
                if !self.eat_sym(",") {
                    break;
                }
            }
            self.expect_sym("}")?;
        }
        self.expect_nl()?;
        Ok(Import { path, pos: t.pos })
    }

    fn parse_item(&mut self) -> PResult<Item> {
        let pos = self.cur().pos;
        let start_idx = self.i;
        let is_pub = self.eat_kw("pub");
        let kind = if self.is_kw("fn") {
            ItemKind::Fn(Box::new(self.parse_fn(start_idx + usize::from(is_pub))?))
        } else if self.is_kw("type") {
            ItemKind::Type(self.parse_type_decl("type")?)
        } else if self.is_kw("enum") {
            ItemKind::Enum(self.parse_enum_decl()?)
        } else if self.is_kw("resource") {
            ItemKind::Resource(self.parse_resource()?)
        } else if is_pub {
            return Err(self.fail(&["fn", "type", "enum", "resource"]));
        } else {
            return Err(self.fail(&["pub", "fn", "type", "enum", "resource"]));
        };
        Ok(Item { pos, is_pub, kind })
    }

    fn parse_generics(&mut self) -> PResult<Vec<String>> {
        let mut g = Vec::new();
        if self.eat_sym("<") {
            loop {
                g.push(self.expect_pascal()?.0);
                if !self.eat_sym(",") {
                    break;
                }
            }
            self.expect_sym(">")?;
        }
        Ok(g)
    }

    fn parse_field(&mut self) -> PResult<Field> {
        let (name, pos) = self.expect_ident()?;
        self.expect_sym(":")?;
        let ty = self.parse_type()?;
        Ok(Field { name, pos, ty })
    }

    fn parse_fields_braced(&mut self) -> PResult<Vec<Field>> {
        self.expect_sym("{")?;
        let mut fields = Vec::new();
        loop {
            self.eat_nl();
            fields.push(self.parse_field()?);
            self.eat_nl();
            if !self.eat_sym(",") {
                break;
            }
        }
        self.eat_nl();
        self.expect_sym("}")?;
        Ok(fields)
    }

    fn tokens_from(&self, start: usize) -> Vec<String> {
        self.toks[start..self.i]
            .iter()
            .filter(|t| t.kind != TokKind::Nl)
            .map(Token::canon)
            .collect()
    }

    fn parse_type_decl(&mut self, kw: &str) -> PResult<TypeDecl> {
        let start = self.i;
        self.expect_kw(kw)?;
        let (name, name_pos) = self.expect_pascal()?;
        self.cur_item = Some(name.clone());
        let generics = self.parse_generics()?;
        let fields = self.parse_fields_braced()?;
        let tokens = self.tokens_from(start);
        self.expect_nl()?;
        Ok(TypeDecl {
            name,
            name_pos,
            generics,
            fields,
            tokens,
        })
    }

    fn parse_resource(&mut self) -> PResult<TypeDecl> {
        let start = self.i;
        self.expect_kw("resource")?;
        let (name, name_pos) = self.expect_pascal()?;
        self.cur_item = Some(name.clone());
        let fields = if self.is_sym("{") {
            self.parse_fields_braced()?
        } else {
            Vec::new()
        };
        let tokens = self.tokens_from(start);
        self.expect_nl()?;
        Ok(TypeDecl {
            name,
            name_pos,
            generics: Vec::new(),
            fields,
            tokens,
        })
    }

    fn parse_enum_decl(&mut self) -> PResult<EnumDecl> {
        let start = self.i;
        self.expect_kw("enum")?;
        let (name, name_pos) = self.expect_pascal()?;
        self.cur_item = Some(name.clone());
        let generics = self.parse_generics()?;
        self.expect_sym("{")?;
        let mut variants = Vec::new();
        loop {
            self.eat_nl();
            let (vname, vpos) = self.expect_pascal()?;
            let mut payload = Vec::new();
            if self.eat_sym("(") {
                loop {
                    payload.push(self.parse_type()?);
                    if !self.eat_sym(",") {
                        break;
                    }
                }
                self.expect_sym(")")?;
            }
            variants.push(Variant {
                name: vname,
                pos: vpos,
                payload,
            });
            self.eat_nl();
            if !self.eat_sym(",") {
                break;
            }
        }
        self.eat_nl();
        self.expect_sym("}")?;
        let tokens = self.tokens_from(start);
        self.expect_nl()?;
        Ok(EnumDecl {
            name,
            name_pos,
            generics,
            variants,
            tokens,
        })
    }

    // ---- types ---------------------------------------------------------

    fn parse_type(&mut self) -> PResult<TypeExpr> {
        let t = self.cur().clone();
        let ok = t.kind == TokKind::TypeIdent
            || (t.kind == TokKind::Keyword && TYPE_KEYWORDS.contains(&t.text.as_str()));
        if !ok {
            return Err(self.fail(&["Ident"]));
        }
        self.bump();
        let mut args = Vec::new();
        if self.eat_sym("<") {
            loop {
                if self.cur().kind == TokKind::Int {
                    let n = self.bump();
                    args.push(TypeArg::Int(n.text.parse().unwrap_or(0)));
                } else if self.cur().kind == TokKind::Ident && self.nth(1).text == ":" {
                    let (n, _) = self.expect_ident()?;
                    self.bump();
                    let e = self.parse_expr()?;
                    args.push(TypeArg::Named(n, Box::new(e)));
                } else {
                    args.push(TypeArg::Type(self.parse_type()?));
                }
                if !self.eat_sym(",") {
                    break;
                }
            }
            self.expect_sym(">")?;
        }
        Ok(TypeExpr {
            name: t.text,
            args,
            pos: t.pos,
        })
    }

    // ---- functions -----------------------------------------------------

    fn clause_stage(&self) -> Option<usize> {
        let t = self.cur();
        if t.kind != TokKind::Keyword {
            return None;
        }
        CLAUSES.iter().position(|c| *c == t.text)
    }

    fn expected_clauses(stage: usize) -> Vec<&'static str> {
        let mut v: Vec<&'static str> = CLAUSES[stage.min(8)..].to_vec();
        v.push("{");
        v
    }

    fn parse_fn(&mut self, fn_idx: usize) -> PResult<FnDecl> {
        self.expect_kw("fn")?;
        let (name, name_pos) = self.expect_ident()?;
        self.cur_fn = Some(name.clone());
        self.cur_item = Some(name.clone());
        self.expect_sym("(")?;
        let mut params = Vec::new();
        if !self.is_sym(")") {
            loop {
                let (pname, ppos) = self.expect_ident()?;
                self.expect_sym(":")?;
                let ty = self.parse_type()?;
                params.push(Param {
                    name: pname,
                    pos: ppos,
                    ty,
                });
                if self.eat_sym(",") {
                    continue;
                }
                if self.is_sym(")") {
                    break;
                }
                return Err(self.fail(&[",", ")"]));
            }
        }
        self.expect_sym(")")?;
        self.expect_sym("->")?;
        let ret = self.parse_type()?;
        let mut spec_end = self.i;
        self.expect_nl()?;

        let mut f = FnDecl {
            name,
            name_pos,
            params,
            ret,
            requires: Vec::new(),
            ensures: Vec::new(),
            has_examples: false,
            examples: Vec::new(),
            oracle: None,
            effects: None,
            alloc: None,
            cost: None,
            latency: None,
            body: Body {
                lets: Vec::new(),
                terminal: Terminal::Return(Expr {
                    id: 0,
                    pos: name_pos,
                    span: (0, 0),
                    kind: ExprKind::Bool(true),
                }),
            },
            spec_tokens: Vec::new(),
            full_tokens: Vec::new(),
        };

        let mut stage = 0usize;
        while let Some(k) = self.clause_stage() {
            let repeatable = k <= 1;
            if k < stage {
                return Err(self.fail(&Self::expected_clauses(stage)));
            }
            stage = if repeatable { k } else { k + 1 };
            let kw_pos = self.cur().pos;
            self.bump();
            match k {
                0 | 1 => {
                    let clause = self.parse_clause()?;
                    if k == 0 {
                        f.requires.push(clause);
                    } else {
                        f.ensures.push(clause);
                    }
                    spec_end = self.i;
                }
                2 => {
                    self.expect_nl()?;
                    f.has_examples = true;
                    f.examples = self.parse_examples()?;
                    spec_end = self.i;
                    continue;
                }
                3 => {
                    let (n, p) = self.expect_ident()?;
                    f.oracle = Some((n, p));
                    spec_end = self.i;
                }
                4 => {
                    f.effects = Some(self.parse_effects(kw_pos)?);
                }
                5 => {
                    f.alloc = Some(self.parse_alloc(kw_pos)?);
                }
                6 => {
                    let t = self.cur().clone();
                    if !(t.kind == TokKind::TypeIdent && t.text == "O") {
                        return Err(self.fail(&["O"]));
                    }
                    self.bump();
                    self.expect_sym("(")?;
                    let save = std::mem::replace(&mut self.no_struct, false);
                    let e = self.parse_expr()?;
                    self.no_struct = save;
                    self.expect_sym(")")?;
                    f.cost = Some(CostClause {
                        pos: kw_pos,
                        expr: e,
                    });
                }
                _ => {
                    self.expect_sym("<")?;
                    if self.cur().kind != TokKind::Dur {
                        return Err(self.fail(&["duration"]));
                    }
                    let d = self.bump();
                    f.latency = Some((d.text, kw_pos));
                }
            }
            self.expect_nl()?;
        }
        if !self.is_sym("{") {
            return Err(self.fail(&Self::expected_clauses(stage)));
        }
        f.spec_tokens = self.toks[fn_idx..spec_end]
            .iter()
            .filter(|t| t.kind != TokKind::Nl)
            .map(Token::canon)
            .collect();
        f.body = self.parse_body()?;
        f.full_tokens = self.toks[fn_idx..self.i]
            .iter()
            .filter(|t| t.kind != TokKind::Nl)
            .map(Token::canon)
            .collect();
        self.expect_nl()?;
        Ok(f)
    }

    fn parse_clause(&mut self) -> PResult<Clause> {
        let start = self.cur().start;
        let expr = self.parse_expr()?;
        let text = self.src[start..self.last_end.max(start)].to_string();
        Ok(Clause { expr, text })
    }

    fn at_example_end(&self) -> bool {
        let t = self.cur();
        t.kind == TokKind::Eof
            || (t.kind == TokKind::Keyword && CLAUSES.contains(&t.text.as_str()))
            || (t.kind == TokKind::Sym && t.text == "{")
    }

    fn parse_examples(&mut self) -> PResult<Vec<Example>> {
        let mut out = Vec::new();
        if self.at_example_end() {
            return Err(self.fail(&["expr", "=>"]));
        }
        while !self.at_example_end() {
            let pos = self.cur().pos;
            let mut inputs = Vec::new();
            if !self.is_sym("=>") {
                loop {
                    inputs.push(self.parse_expr()?);
                    if !self.eat_sym(",") {
                        break;
                    }
                }
            }
            if !self.is_sym("=>") {
                return Err(self.fail(&["=>", ","]));
            }
            self.bump();
            let output = self.parse_expr()?;
            out.push(Example {
                pos,
                inputs,
                output,
            });
            if !self.at_example_end() {
                self.expect_nl()?;
            }
        }
        // the trailing NL of the last example is consumed by the loop above or by the
        // caller's position check; make sure the caller sees the next clause.
        Ok(out)
    }

    fn parse_effects(&mut self, pos: Pos) -> PResult<EffectsClause> {
        let mut names = Vec::new();
        if self.is_kw("none") {
            self.bump();
        } else if self.cur().kind == TokKind::Ident {
            loop {
                let (mut n, p) = self.expect_ident()?;
                while self.is_sym(".") {
                    self.bump();
                    let (m, _) = self.expect_ident()?;
                    n.push('.');
                    n.push_str(&m);
                }
                names.push((n, p));
                if !self.eat_sym(",") {
                    break;
                }
            }
        } else {
            return Err(self.fail(&["none", "ident"]));
        }
        Ok(EffectsClause { pos, names })
    }

    fn parse_alloc(&mut self, pos: Pos) -> PResult<AllocClause> {
        if self.eat_kw("none") {
            return Ok(AllocClause { pos, reuse: None });
        }
        if self.eat_kw("reuse") {
            self.expect_sym("(")?;
            let r = self.expect_ident()?;
            self.expect_sym(")")?;
            return Ok(AllocClause {
                pos,
                reuse: Some(r),
            });
        }
        Err(self.fail(&["none", "reuse"]))
    }

    fn parse_let(&mut self) -> PResult<Let> {
        self.expect_kw("let")?;
        let (name, pos) = self.expect_ident()?;
        self.expect_sym(":")?;
        let ty = self.parse_type()?;
        self.expect_sym("=")?;
        let value = self.parse_expr()?;
        self.expect_nl()?;
        Ok(Let {
            name,
            pos,
            ty,
            value,
        })
    }

    /// `{ let* terminal }` (the trailing NL is left to the caller).
    fn parse_body(&mut self) -> PResult<Body> {
        self.expect_sym("{")?;
        let save = std::mem::replace(&mut self.no_struct, false);
        let mut lets = Vec::new();
        while self.is_kw("let") {
            lets.push(self.parse_let()?);
        }
        let terminal = if self.is_kw("return") {
            self.bump();
            let e = self.parse_expr()?;
            if !self.is_sym("}") {
                self.expect_nl()?;
            }
            Terminal::Return(e)
        } else if self.is_kw("using") {
            self.bump();
            let (name, pos) = self.expect_ident()?;
            let body = self.parse_body()?;
            self.expect_nl()?;
            Terminal::Using {
                name,
                pos,
                body: Box::new(body),
            }
        } else {
            return Err(self.fail(&["let", "return", "using"]));
        };
        self.expect_sym("}")?;
        self.no_struct = save;
        Ok(Body { lets, terminal })
    }

    // ---- expressions ---------------------------------------------------

    pub fn parse_expr(&mut self) -> PResult<Expr> {
        self.parse_or()
    }

    fn bin(&mut self, op: BinOp, l: Expr, r: Expr) -> Expr {
        let (pos, start) = (l.pos, l.span.0);
        self.mk(ExprKind::Binary(op, Box::new(l), Box::new(r)), pos, start)
    }

    fn parse_or(&mut self) -> PResult<Expr> {
        let mut l = self.parse_and()?;
        while self.is_kw("or") {
            self.bump();
            let r = self.parse_and()?;
            l = self.bin(BinOp::Or, l, r);
        }
        Ok(l)
    }

    fn parse_and(&mut self) -> PResult<Expr> {
        let mut l = self.parse_not()?;
        while self.is_kw("and") {
            self.bump();
            let r = self.parse_not()?;
            l = self.bin(BinOp::And, l, r);
        }
        Ok(l)
    }

    fn parse_not(&mut self) -> PResult<Expr> {
        if self.is_kw("not") {
            let t = self.bump();
            let e = self.parse_cmp()?;
            return Ok(self.mk(ExprKind::Unary(UnOp::Not, Box::new(e)), t.pos, t.start));
        }
        self.parse_cmp()
    }

    fn parse_cmp(&mut self) -> PResult<Expr> {
        let l = self.parse_range()?;
        let t = self.cur().clone();
        let op = match (t.kind, t.text.as_str()) {
            (TokKind::Sym, "==") => Some(BinOp::Eq),
            (TokKind::Sym, "!=") => Some(BinOp::Ne),
            (TokKind::Sym, "<") => Some(BinOp::Lt),
            (TokKind::Sym, "<=") => Some(BinOp::Le),
            (TokKind::Sym, ">") => Some(BinOp::Gt),
            (TokKind::Sym, ">=") => Some(BinOp::Ge),
            (TokKind::Keyword, "in") => Some(BinOp::In),
            _ => None,
        };
        if let Some(op) = op {
            self.bump();
            // `a == not b` means `a == (not b)`; `not` is accepted here for `==`/`!=` only.
            let r = if matches!(op, BinOp::Eq | BinOp::Ne) && self.is_kw("not") {
                let t = self.bump();
                let e = self.parse_range()?;
                self.mk(ExprKind::Unary(UnOp::Not, Box::new(e)), t.pos, t.start)
            } else {
                self.parse_range()?
            };
            return Ok(self.bin(op, l, r));
        }
        Ok(l)
    }

    fn parse_range(&mut self) -> PResult<Expr> {
        let l = self.parse_add()?;
        let incl = if self.is_sym("..") {
            false
        } else if self.is_sym("..=") {
            true
        } else {
            return Ok(l);
        };
        self.bump();
        let r = self.parse_add()?;
        let (pos, start) = (l.pos, l.span.0);
        Ok(self.mk(ExprKind::Range(Box::new(l), Box::new(r), incl), pos, start))
    }

    fn parse_add(&mut self) -> PResult<Expr> {
        let mut l = self.parse_mul()?;
        loop {
            let op = if self.is_sym("+") {
                BinOp::Add
            } else if self.is_sym("-") {
                BinOp::Sub
            } else {
                break;
            };
            self.bump();
            let r = self.parse_mul()?;
            l = self.bin(op, l, r);
        }
        Ok(l)
    }

    fn parse_mul(&mut self) -> PResult<Expr> {
        let mut l = self.parse_unary()?;
        loop {
            let op = if self.is_sym("*") {
                BinOp::Mul
            } else if self.is_sym("/") {
                BinOp::Div
            } else if self.is_sym("%") {
                BinOp::Rem
            } else {
                break;
            };
            self.bump();
            let r = self.parse_unary()?;
            l = self.bin(op, l, r);
        }
        Ok(l)
    }

    fn parse_unary(&mut self) -> PResult<Expr> {
        if self.is_sym("-") {
            let t = self.bump();
            let e = self.parse_postfix()?;
            return Ok(self.mk(ExprKind::Unary(UnOp::Neg, Box::new(e)), t.pos, t.start));
        }
        self.parse_postfix()
    }

    fn parse_args(&mut self) -> PResult<Vec<Expr>> {
        self.expect_sym("(")?;
        let save = std::mem::replace(&mut self.no_struct, false);
        let mut args = Vec::new();
        if !self.is_sym(")") {
            loop {
                args.push(self.parse_expr()?);
                if self.eat_sym(",") {
                    continue;
                }
                if self.is_sym(")") {
                    break;
                }
                return Err(self.fail(&[",", ")"]));
            }
        }
        self.expect_sym(")")?;
        self.no_struct = save;
        Ok(args)
    }

    fn parse_postfix(&mut self) -> PResult<Expr> {
        let mut e = self.parse_primary()?;
        loop {
            if self.is_sym(".") {
                self.bump();
                let (name, _) = self.expect_ident()?;
                let name_end = self.last_end;
                let args = if self.is_sym("(") {
                    Some(self.parse_args()?)
                } else {
                    None
                };
                let (pos, start) = (e.pos, e.span.0);
                e = self.mk(
                    ExprKind::Method {
                        recv: Box::new(e),
                        name,
                        name_end,
                        args,
                    },
                    pos,
                    start,
                );
            } else if self.is_sym("[") {
                // A `[` on a later line is a new list literal: the lexer emits an NL token
                // there, so reaching this branch means the bracket is on the receiver's line.
                self.bump();
                let save = std::mem::replace(&mut self.no_struct, false);
                let idx = self.parse_expr()?;
                self.no_struct = save;
                self.expect_sym("]")?;
                let (pos, start) = (e.pos, e.span.0);
                e = self.mk(ExprKind::Index(Box::new(e), Box::new(idx)), pos, start);
            } else if self.is_sym("?") {
                self.bump();
                let (pos, start) = (e.pos, e.span.0);
                e = self.mk(ExprKind::Try(Box::new(e)), pos, start);
            } else {
                break;
            }
        }
        Ok(e)
    }

    fn parse_primary(&mut self) -> PResult<Expr> {
        let t = self.cur().clone();
        let (pos, start) = (t.pos, t.start);
        match t.kind {
            TokKind::Int => {
                self.bump();
                let n = t.text.parse::<u64>().unwrap_or(0);
                Ok(self.mk(ExprKind::Int(n), pos, start))
            }
            TokKind::Float => {
                self.bump();
                let n = t.text.parse::<f64>().unwrap_or(0.0);
                Ok(self.mk(ExprKind::Float(n), pos, start))
            }
            TokKind::Str => {
                self.bump();
                Ok(self.mk(ExprKind::Str(t.text), pos, start))
            }
            TokKind::Keyword => match t.text.as_str() {
                "true" | "false" => {
                    self.bump();
                    Ok(self.mk(ExprKind::Bool(t.text == "true"), pos, start))
                }
                "result" => {
                    self.bump();
                    Ok(self.mk(ExprKind::ResultKw, pos, start))
                }
                "if" => self.parse_if(),
                "match" => self.parse_match(),
                "for" => self.parse_for(false),
                "par" => {
                    self.bump();
                    self.parse_for_rest(true, pos, start)
                }
                _ => Err(self.fail(&["expr"])),
            },
            TokKind::Ident => {
                self.bump();
                if self.is_sym("(") {
                    let args = self.parse_args()?;
                    Ok(self.mk(ExprKind::Call(t.text, args), pos, start))
                } else {
                    Ok(self.mk(ExprKind::Var(t.text), pos, start))
                }
            }
            TokKind::TypeIdent => {
                self.bump();
                if self.is_sym("(") {
                    let args = self.parse_args()?;
                    Ok(self.mk(ExprKind::Ctor(t.text, Some(args)), pos, start))
                } else if self.is_sym("{") && !self.no_struct {
                    self.parse_struct_lit(t.text, pos, start)
                } else {
                    Ok(self.mk(ExprKind::Ctor(t.text, None), pos, start))
                }
            }
            TokKind::Sym => match t.text.as_str() {
                "(" => {
                    self.bump();
                    let save = std::mem::replace(&mut self.no_struct, false);
                    let e = self.parse_expr()?;
                    self.no_struct = save;
                    self.expect_sym(")")?;
                    // parentheses are transparent, but the source span includes them
                    let mut e = e;
                    e.span = (start, self.last_end);
                    Ok(e)
                }
                "[" => {
                    self.bump();
                    let save = std::mem::replace(&mut self.no_struct, false);
                    let mut items = Vec::new();
                    if !self.is_sym("]") {
                        loop {
                            items.push(self.parse_expr()?);
                            if self.eat_sym(",") {
                                continue;
                            }
                            if self.is_sym("]") {
                                break;
                            }
                            return Err(self.fail(&[",", "]"]));
                        }
                    }
                    self.expect_sym("]")?;
                    self.no_struct = save;
                    Ok(self.mk(ExprKind::List(items), pos, start))
                }
                "|" => {
                    self.bump();
                    let mut params = Vec::new();
                    loop {
                        params.push(self.expect_ident()?);
                        if !self.eat_sym(",") {
                            break;
                        }
                    }
                    self.expect_sym("|")?;
                    let body = self.parse_expr()?;
                    Ok(self.mk(ExprKind::Lambda(params, Box::new(body)), pos, start))
                }
                _ => Err(self.fail(&["expr"])),
            },
            _ => Err(self.fail(&["expr"])),
        }
    }

    fn parse_struct_lit(&mut self, name: String, pos: Pos, start: usize) -> PResult<Expr> {
        self.expect_sym("{")?;
        let save = std::mem::replace(&mut self.no_struct, false);
        let mut fields = Vec::new();
        loop {
            self.eat_nl();
            let (fname, fpos) = self.expect_ident()?;
            self.expect_sym(":")?;
            let e = self.parse_expr()?;
            fields.push((fname, fpos, e));
            self.eat_nl();
            if !self.eat_sym(",") {
                break;
            }
        }
        self.eat_nl();
        self.expect_sym("}")?;
        self.no_struct = save;
        Ok(self.mk(ExprKind::Struct(name, fields), pos, start))
    }

    /// `{ let* expr }` used by `if`, `else` and `for`.
    fn parse_block(&mut self) -> PResult<Block> {
        self.expect_sym("{")?;
        let save = std::mem::replace(&mut self.no_struct, false);
        let mut lets = Vec::new();
        while self.is_kw("let") {
            lets.push(self.parse_let()?);
        }
        let expr = self.parse_expr()?;
        self.eat_nl();
        self.expect_sym("}")?;
        self.no_struct = save;
        Ok(Block {
            lets,
            expr: Box::new(expr),
        })
    }

    fn parse_cond(&mut self) -> PResult<Expr> {
        let save = std::mem::replace(&mut self.no_struct, true);
        let e = self.parse_expr();
        self.no_struct = save;
        e
    }

    fn parse_if(&mut self) -> PResult<Expr> {
        let t = self.expect_kw("if")?;
        let cond = self.parse_cond()?;
        let then = self.parse_block()?;
        if !self.is_kw("else") {
            return Err(self.fail(&["else"]));
        }
        self.bump();
        let els = if self.is_kw("if") {
            let inner = self.parse_if()?;
            Block {
                lets: Vec::new(),
                expr: Box::new(inner),
            }
        } else {
            self.parse_block()?
        };
        Ok(self.mk(ExprKind::If(Box::new(cond), then, els), t.pos, t.start))
    }

    fn parse_match(&mut self) -> PResult<Expr> {
        let t = self.expect_kw("match")?;
        let scrut = self.parse_cond()?;
        self.expect_sym("{")?;
        let save = std::mem::replace(&mut self.no_struct, false);
        let mut arms = Vec::new();
        loop {
            self.eat_nl();
            if self.is_sym("}") {
                break;
            }
            let pat = self.parse_pattern()?;
            self.expect_sym("=>")?;
            let body = self.parse_expr()?;
            arms.push(Arm { pat, body });
            if self.cur().kind == TokKind::Nl {
                self.bump();
            } else if !self.is_sym("}") {
                return Err(self.fail(&["NL"]));
            }
        }
        self.expect_sym("}")?;
        self.no_struct = save;
        Ok(self.mk(ExprKind::Match(Box::new(scrut), arms), t.pos, t.start))
    }

    fn parse_for(&mut self, par: bool) -> PResult<Expr> {
        let t = self.cur().clone();
        self.parse_for_rest(par, t.pos, t.start)
    }

    fn parse_for_rest(&mut self, par: bool, pos: Pos, start: usize) -> PResult<Expr> {
        self.expect_kw("for")?;
        let (var, var_pos) = self.expect_ident()?;
        self.expect_kw("in")?;
        let iter = self.parse_cond()?;
        let body = self.parse_block()?;
        Ok(self.mk(
            ExprKind::For {
                par,
                var,
                var_pos,
                iter: Box::new(iter),
                body,
            },
            pos,
            start,
        ))
    }

    fn parse_pattern(&mut self) -> PResult<Pattern> {
        let t = self.cur().clone();
        let pos = t.pos;
        let kind = match t.kind {
            TokKind::Underscore => {
                self.bump();
                PatKind::Wild
            }
            TokKind::Int => {
                self.bump();
                PatKind::Int(t.text.parse().unwrap_or(0))
            }
            TokKind::Float => {
                self.bump();
                PatKind::Float(t.text.parse().unwrap_or(0.0))
            }
            TokKind::Str => {
                self.bump();
                PatKind::Str(t.text)
            }
            TokKind::Ident => {
                self.bump();
                PatKind::Bind(t.text)
            }
            TokKind::Keyword if t.text == "true" || t.text == "false" => {
                self.bump();
                PatKind::Bool(t.text == "true")
            }
            TokKind::Sym if t.text == "-" => {
                self.bump();
                let n = self.cur().clone();
                match n.kind {
                    TokKind::Int => {
                        self.bump();
                        PatKind::Int(-n.text.parse::<i64>().unwrap_or(0))
                    }
                    TokKind::Float => {
                        self.bump();
                        PatKind::Float(-n.text.parse::<f64>().unwrap_or(0.0))
                    }
                    _ => return Err(self.fail(&["int", "float"])),
                }
            }
            TokKind::TypeIdent => {
                self.bump();
                let mut subs = Vec::new();
                if self.eat_sym("(") {
                    loop {
                        subs.push(self.parse_pattern()?);
                        if !self.eat_sym(",") {
                            break;
                        }
                    }
                    self.expect_sym(")")?;
                }
                PatKind::Ctor(t.text, subs)
            }
            _ => return Err(self.fail(&["pattern"])),
        };
        Ok(Pattern { pos, kind })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn errs(src: &str) -> Vec<Diag> {
        parse(src).diags
    }

    fn expected(d: &Diag) -> Vec<String> {
        d.extra
            .iter()
            .find(|(k, _)| *k == "expected")
            .map(|(_, v)| {
                v.as_array()
                    .unwrap()
                    .iter()
                    .map(|s| s.as_str().unwrap().to_string())
                    .collect()
            })
            .unwrap_or_default()
    }

    const OK: &str = "module m\nfn f(x: Int) -> Int\n  requires x > 0\n  ensures result > 0\n  examples\n    1 => 1\n  effects none\n  cost O(1)\n  latency < 2ms\n{\n  return x\n}\n";

    fn body(e: &str) -> String {
        format!(
            "module m\nfn f(a: Bool, b: Bool, c: Bool) -> Bool\n{{\n  return {}\n}}\n",
            e
        )
    }

    #[test]
    fn not_is_accepted_after_equality_only() {
        assert!(errs(&body("a == not b")).is_empty());
        assert!(errs(&body("a != not b and c")).is_empty());
        assert!(errs(&body("not a == b")).is_empty());
        assert!(errs(&body("a < not b")).iter().any(|d| d.code == "E002"));
        assert!(errs(&body("a and (b == c) == not a")).is_empty());
    }

    #[test]
    fn not_operand_binds_to_the_operand_only() {
        let m = parse(&body("a == not b and c")).module;
        let dbg = format!("{:?}", m);
        let and = dbg.find("And").unwrap();
        let eq = dbg.find("Eq").unwrap();
        assert!(and < eq);
    }

    #[test]
    fn canonical_order_parses() {
        let o = parse(OK);
        assert!(o.diags.is_empty(), "{:?}", o.diags);
        let f = o.module.fns().next().unwrap().1;
        assert_eq!(f.ensures[0].text, "result > 0");
        assert_eq!(f.examples.len(), 1);
    }

    #[test]
    fn missing_module() {
        let d = errs("fn f() -> Int\n{\n  return 1\n}\n");
        assert_eq!(d.len(), 1);
        assert_eq!(expected(&d[0]), vec!["module"]);
        assert_eq!(d[0].pos, Pos::new(1, 1));
    }

    #[test]
    fn missing_arrow() {
        let d = errs("module m\nfn f(x: Int) Int\n{\n  return 1\n}\n");
        assert_eq!(d.len(), 1);
        assert!(expected(&d[0]).contains(&"->".to_string()));
        assert_eq!(d[0].pos, Pos::new(2, 14));
    }

    #[test]
    fn clause_order() {
        let d =
            errs("module m\nfn f(x: Int) -> Int\n  latency < 2ms\n  cost O(1)\n{\n  return 1\n}\n");
        assert_eq!(d.len(), 1);
        assert_eq!(d[0].pos, Pos::new(4, 3));
        assert_eq!(expected(&d[0]), vec!["{"]);
        let d = errs(
            "module m\nfn f(x: Int) -> Int\n  ensures x > 0\n  requires x > 0\n{\n  return 1\n}\n",
        );
        assert_eq!(d[0].pos, Pos::new(4, 3));
    }

    #[test]
    fn body_terminal() {
        let d = errs("module m\nfn f() -> Int\n{\n  let y: Int = 1\n}\n");
        assert!(expected(&d[0]).contains(&"return".to_string()));
        let d = errs("module m\nfn f() -> Int\n{\n  return 1\n  let y: Int = 2\n}\n");
        assert_eq!(d[0].pos, Pos::new(5, 3));
    }

    #[test]
    fn let_requires_type_and_if_requires_else() {
        let d = errs("module m\nfn f() -> Int\n{\n  let y = 1\n  return y\n}\n");
        assert!(expected(&d[0]).contains(&":".to_string()));
        let d = errs("module m\nfn f(x: Int) -> Int\n{\n  return if x > 0 { 1 }\n}\n");
        assert!(expected(&d[0]).contains(&"else".to_string()));
    }

    #[test]
    fn no_return_in_block_and_no_chain() {
        let d = errs(
            "module m\nfn f(x: Int) -> Int\n{\n  return if x > 0 { return 1 } else { 0 }\n}\n",
        );
        assert_eq!(d[0].pos, Pos::new(4, 21));
        assert_eq!(
            errs("module m\nfn f() -> Bool\n{\n  return 1 < 2 < 3\n}\n").len(),
            1
        );
    }

    #[test]
    fn struct_literal_rule() {
        let ok = "module m\ntype Point { x: Int, y: Int }\nfn f() -> Int\n{\n  return Point { x: 1, y: 2 }.y\n}\n";
        assert!(errs(ok).is_empty());
        let bad = "module m\ntype Point { x: Int, y: Int }\nfn f() -> Int\n{\n  return if Point { x: 1, y: 2 }.x > 0 { 1 } else { 0 }\n}\n";
        assert_eq!(errs(bad).len(), 1);
    }

    #[test]
    fn cost_needs_o() {
        let d = errs("module m\nfn f(xs: List<Int>) -> Int\n  cost xs.len()\n{\n  return 1\n}\n");
        assert_eq!(expected(&d[0]), vec!["O"]);
    }

    #[test]
    fn keyword_as_name_and_pascal_let() {
        let d = errs("module m\nfn match(x: Int) -> Int\n{\n  return 1\n}\n");
        assert_eq!(d.len(), 1);
        assert_eq!(d[0].pos, Pos::new(2, 4));
        let d = errs("module m\nfn f() -> Int\n{\n  let Total: Int = 1\n  return 1\n}\n");
        assert!(expected(&d[0]).contains(&"ident".to_string()));
    }

    #[test]
    fn recovery_reports_errors_in_two_items() {
        let src = "module m\nfn a() -> Int\n{\n  let y = 1\n  return 1\n}\nfn b() -> Int\n{\n  let z = 1\n  return 1\n}\n";
        assert_eq!(errs(src).len(), 2);
    }

    #[test]
    fn multiline_list_in_example_and_split_operator() {
        let src = "module m\nfn f(xs: List<Int>) -> Int\n  examples\n    [1,\n     2,\n     3] => 6\n{\n  return 1 +\n    2\n}\n";
        assert!(errs(src).is_empty(), "{:?}", errs(src));
    }

    #[test]
    fn spec_tokens_ignore_whitespace_and_comments() {
        let a = parse("module m\npub fn f(x: Int) -> Int\n  ensures result > 0\n  cost O(1)\n{\n  return x\n}\n");
        let b = parse("module m\npub fn f( x : Int ) -> Int # c\n  ensures   result > 0 # d\n  cost O(2)\n{\n  return 5\n}\n");
        let fa = a.module.fns().next().unwrap().1;
        let fb = b.module.fns().next().unwrap().1;
        assert_eq!(fa.spec_tokens, fb.spec_tokens);
    }

    #[test]
    fn match_and_lambda_and_for() {
        let src = "module m\nenum Shape { Circle(Float), Square(Float) }\nfn a(s: Shape, xs: List<Int>) -> Int\n{\n  let t: Int = xs.fold(0, |acc, x| acc + x)\n  let u: List<Int> = par for x in xs { x * 2 }\n  return match s {\n    Circle(r) => 1\n    _ => 2\n  }\n}\n";
        assert!(errs(src).is_empty(), "{:?}", errs(src));
    }
}
