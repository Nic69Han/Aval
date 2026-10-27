//! Abstract syntax tree.

use crate::diag::Pos;

#[derive(Clone, Debug)]
pub struct Module {
    pub name: String,
    pub name_pos: Pos,
    /// Position of the `module` keyword (or of the first token when it is missing).
    pub header_pos: Pos,
    pub imports: Vec<Import>,
    pub items: Vec<Item>,
    /// Names of items that failed to parse (later stages stay quiet about them).
    pub broken: Vec<String>,
}

#[derive(Clone, Debug)]
pub struct Import {
    pub path: String,
    pub pos: Pos,
}

#[derive(Clone, Debug)]
pub struct Item {
    /// Position of the first token of the item (`pub` if present).
    pub pos: Pos,
    pub is_pub: bool,
    pub kind: ItemKind,
}

#[derive(Clone, Debug)]
pub enum ItemKind {
    Fn(Box<FnDecl>),
    Type(TypeDecl),
    Enum(EnumDecl),
    Resource(TypeDecl),
}

#[derive(Clone, Debug)]
pub struct TypeExpr {
    pub name: String,
    pub args: Vec<TypeArg>,
    pub pos: Pos,
}

#[derive(Clone, Debug)]
pub enum TypeArg {
    Type(TypeExpr),
    Int(u64),
    Named(String, Box<Expr>),
}

#[derive(Clone, Debug)]
pub struct Field {
    pub name: String,
    pub pos: Pos,
    pub ty: TypeExpr,
}

#[derive(Clone, Debug)]
pub struct TypeDecl {
    pub name: String,
    pub name_pos: Pos,
    pub generics: Vec<String>,
    pub fields: Vec<Field>,
    /// Canonical tokens of the whole declaration (for the spec lock).
    pub tokens: Vec<String>,
}

#[derive(Clone, Debug)]
pub struct Variant {
    pub name: String,
    pub pos: Pos,
    pub payload: Vec<TypeExpr>,
}

#[derive(Clone, Debug)]
pub struct EnumDecl {
    pub name: String,
    pub name_pos: Pos,
    pub generics: Vec<String>,
    pub variants: Vec<Variant>,
    /// Canonical tokens of the whole declaration (for the spec lock).
    pub tokens: Vec<String>,
}

#[derive(Clone, Debug)]
pub struct Param {
    pub name: String,
    pub pos: Pos,
    pub ty: TypeExpr,
}

/// A spec clause expression with its source text.
#[derive(Clone, Debug)]
pub struct Clause {
    pub expr: Expr,
    pub text: String,
}

#[derive(Clone, Debug)]
pub struct Example {
    pub pos: Pos,
    pub inputs: Vec<Expr>,
    pub output: Expr,
}

#[derive(Clone, Debug)]
pub struct EffectsClause {
    pub pos: Pos,
    pub names: Vec<(String, Pos)>,
}

#[derive(Clone, Debug)]
pub struct AllocClause {
    pub pos: Pos,
    pub reuse: Option<(String, Pos)>,
}

#[derive(Clone, Debug)]
pub struct CostClause {
    pub pos: Pos,
    pub expr: Expr,
}

#[derive(Clone, Debug)]
pub struct FnDecl {
    pub name: String,
    pub name_pos: Pos,
    pub params: Vec<Param>,
    pub ret: TypeExpr,
    pub requires: Vec<Clause>,
    pub ensures: Vec<Clause>,
    pub has_examples: bool,
    pub examples: Vec<Example>,
    pub oracle: Option<(String, Pos)>,
    pub effects: Option<EffectsClause>,
    pub alloc: Option<AllocClause>,
    pub cost: Option<CostClause>,
    pub latency: Option<(String, Pos)>,
    pub body: Body,
    /// Canonical tokens of the signature and the spec clauses (for the spec lock).
    pub spec_tokens: Vec<String>,
    /// Canonical tokens of the whole function, body included.
    pub full_tokens: Vec<String>,
}

#[derive(Clone, Debug)]
pub struct Let {
    pub name: String,
    pub pos: Pos,
    pub ty: TypeExpr,
    pub value: Expr,
}

#[derive(Clone, Debug)]
pub struct Body {
    pub lets: Vec<Let>,
    pub terminal: Terminal,
}

#[derive(Clone, Debug)]
pub enum Terminal {
    Return(Expr),
    Using {
        name: String,
        pos: Pos,
        body: Box<Body>,
    },
}

#[derive(Clone, Debug)]
pub struct Block {
    pub lets: Vec<Let>,
    pub expr: Box<Expr>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Rem,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    And,
    Or,
    In,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UnOp {
    Neg,
    Not,
}

#[derive(Clone, Debug)]
pub struct Expr {
    /// Unique within a module; used for side tables.
    pub id: u32,
    pub pos: Pos,
    pub span: (usize, usize),
    pub kind: ExprKind,
}

#[derive(Clone, Debug)]
pub enum ExprKind {
    Int(u64),
    Float(f64),
    Str(String),
    Bool(bool),
    ResultKw,
    Var(String),
    Call(String, Vec<Expr>),
    Ctor(String, Option<Vec<Expr>>),
    Struct(String, Vec<(String, Pos, Expr)>),
    /// `recv.name` (args = None) or `recv.name(args)`.
    Method {
        recv: Box<Expr>,
        name: String,
        name_end: usize,
        args: Option<Vec<Expr>>,
    },
    Try(Box<Expr>),
    /// `recv[idx]`; a slice when `idx` is a `Range` expression.
    Index(Box<Expr>, Box<Expr>),
    Unary(UnOp, Box<Expr>),
    Binary(BinOp, Box<Expr>, Box<Expr>),
    Range(Box<Expr>, Box<Expr>, bool),
    List(Vec<Expr>),
    If(Box<Expr>, Block, Block),
    Match(Box<Expr>, Vec<Arm>),
    For {
        par: bool,
        var: String,
        var_pos: Pos,
        iter: Box<Expr>,
        body: Block,
    },
    Lambda(Vec<(String, Pos)>, Box<Expr>),
}

#[derive(Clone, Debug)]
pub struct Arm {
    pub pat: Pattern,
    pub body: Expr,
}

#[derive(Clone, Debug)]
pub struct Pattern {
    pub pos: Pos,
    pub kind: PatKind,
}

#[derive(Clone, Debug)]
pub enum PatKind {
    Wild,
    Int(i64),
    Float(f64),
    Str(String),
    Bool(bool),
    Bind(String),
    Ctor(String, Vec<Pattern>),
}

impl Block {
    fn walk<'e>(&'e self, f: &mut dyn FnMut(&'e Expr)) {
        for l in &self.lets {
            l.value.walk(f);
        }
        self.expr.walk(f);
    }
}

impl Expr {
    /// Pre-order traversal of this expression and all sub-expressions.
    pub fn walk<'e>(&'e self, f: &mut dyn FnMut(&'e Expr)) {
        f(self);
        match &self.kind {
            ExprKind::Int(_)
            | ExprKind::Float(_)
            | ExprKind::Str(_)
            | ExprKind::Bool(_)
            | ExprKind::ResultKw
            | ExprKind::Var(_) => {}
            ExprKind::Call(_, args) => args.iter().for_each(|a| a.walk(f)),
            ExprKind::Ctor(_, args) => {
                if let Some(a) = args {
                    a.iter().for_each(|a| a.walk(f))
                }
            }
            ExprKind::Struct(_, fields) => fields.iter().for_each(|(_, _, e)| e.walk(f)),
            ExprKind::Method { recv, args, .. } => {
                recv.walk(f);
                if let Some(a) = args {
                    a.iter().for_each(|a| a.walk(f))
                }
            }
            ExprKind::Try(e) | ExprKind::Unary(_, e) => e.walk(f),
            ExprKind::Binary(_, a, b) | ExprKind::Range(a, b, _) | ExprKind::Index(a, b) => {
                a.walk(f);
                b.walk(f)
            }
            ExprKind::List(es) => es.iter().for_each(|a| a.walk(f)),
            ExprKind::If(c, t, e) => {
                c.walk(f);
                t.walk(f);
                e.walk(f)
            }
            ExprKind::Match(s, arms) => {
                s.walk(f);
                arms.iter().for_each(|a| a.body.walk(f))
            }
            ExprKind::For { iter, body, .. } => {
                iter.walk(f);
                body.walk(f)
            }
            ExprKind::Lambda(_, b) => b.walk(f),
        }
    }
}

impl Body {
    pub fn walk<'e>(&'e self, f: &mut dyn FnMut(&'e Expr)) {
        for l in &self.lets {
            l.value.walk(f);
        }
        match &self.terminal {
            Terminal::Return(e) => e.walk(f),
            Terminal::Using { body, .. } => body.walk(f),
        }
    }
}

impl FnDecl {
    /// Visit every expression of the function: clauses, examples and body.
    pub fn walk_all<'e>(&'e self, f: &mut dyn FnMut(&'e Expr)) {
        for c in self.requires.iter().chain(self.ensures.iter()) {
            c.expr.walk(f);
        }
        for ex in &self.examples {
            ex.inputs.iter().for_each(|e| e.walk(f));
            ex.output.walk(f);
        }
        self.body.walk(f);
    }
}

impl Module {
    pub fn fns(&self) -> impl Iterator<Item = (&Item, &FnDecl)> {
        self.items.iter().filter_map(|it| match &it.kind {
            ItemKind::Fn(f) => Some((it, &**f)),
            _ => None,
        })
    }
}
