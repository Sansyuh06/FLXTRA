//! JavaScript AST

use serde::{Deserialize, Serialize};

/// AST Node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AstNode {
    Program(Vec<AstNode>),
    
    // Statements
    VariableDeclaration {
        kind: VarKind,
        declarations: Vec<VariableDeclarator>,
    },
    FunctionDeclaration {
        name: String,
        params: Vec<String>,
        body: Box<AstNode>,
    },
    ExpressionStatement(Box<AstNode>),
    BlockStatement(Vec<AstNode>),
    ReturnStatement(Option<Box<AstNode>>),
    IfStatement {
        test: Box<AstNode>,
        consequent: Box<AstNode>,
        alternate: Option<Box<AstNode>>,
    },
    WhileStatement {
        test: Box<AstNode>,
        body: Box<AstNode>,
    },
    ForStatement {
        init: Option<Box<AstNode>>,
        test: Option<Box<AstNode>>,
        update: Option<Box<AstNode>>,
        body: Box<AstNode>,
    },
    BreakStatement,
    ContinueStatement,
    TryStatement {
        block: Box<AstNode>,
        handler: Option<CatchClause>,
        finalizer: Option<Box<AstNode>>,
    },
    ThrowStatement(Box<AstNode>),
    EmptyStatement,
    
    // Expressions
    Identifier(String),
    Literal(Literal),
    BinaryExpression {
        operator: BinaryOp,
        left: Box<AstNode>,
        right: Box<AstNode>,
    },
    UnaryExpression {
        operator: UnaryOp,
        argument: Box<AstNode>,
        prefix: bool,
    },
    AssignmentExpression {
        operator: AssignOp,
        left: Box<AstNode>,
        right: Box<AstNode>,
    },
    MemberExpression {
        object: Box<AstNode>,
        property: Box<AstNode>,
        computed: bool,
    },
    CallExpression {
        callee: Box<AstNode>,
        arguments: Vec<AstNode>,
    },
    NewExpression {
        callee: Box<AstNode>,
        arguments: Vec<AstNode>,
    },
    ArrayExpression(Vec<AstNode>),
    ObjectExpression(Vec<Property>),
    ConditionalExpression {
        test: Box<AstNode>,
        consequent: Box<AstNode>,
        alternate: Box<AstNode>,
    },
    ArrowFunctionExpression {
        params: Vec<String>,
        body: Box<AstNode>,
    },
    FunctionExpression {
        name: Option<String>,
        params: Vec<String>,
        body: Box<AstNode>,
    },
    LogicalExpression {
        operator: LogicalOp,
        left: Box<AstNode>,
        right: Box<AstNode>,
    },
    UpdateExpression {
        operator: UpdateOp,
        argument: Box<AstNode>,
        prefix: bool,
    },
    ThisExpression,
    SequenceExpression(Vec<AstNode>),
}

/// Variable declaration kind
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VarKind {
    Var,
    Let,
    Const,
}

/// Variable declarator
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariableDeclarator {
    pub name: String,
    pub init: Option<Box<AstNode>>,
}

/// Catch clause
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CatchClause {
    pub param: Option<String>,
    pub body: Box<AstNode>,
}

/// Object property
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Property {
    pub key: String,
    pub value: Box<AstNode>,
    pub computed: bool,
    pub shorthand: bool,
}

/// Literal value
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Literal {
    Number(f64),
    String(String),
    Boolean(bool),
    Null,
    Undefined,
    Regex { pattern: String, flags: String },
}

/// Binary operators
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BinaryOp {
    Add,        // +
    Sub,        // -
    Mul,        // *
    Div,        // /
    Mod,        // %
    Pow,        // **
    Eq,         // ==
    StrictEq,   // ===
    NotEq,      // !=
    StrictNotEq,// !==
    Lt,         // <
    LtEq,       // <=
    Gt,         // >
    GtEq,       // >=
    BitAnd,     // &
    BitOr,      // |
    BitXor,     // ^
    Shl,        // <<
    Shr,        // >>
    UShr,       // >>>
    In,
    InstanceOf,
}

/// Unary operators
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UnaryOp {
    Neg,        // -
    Pos,        // +
    Not,        // !
    BitNot,     // ~
    TypeOf,
    Void,
    Delete,
}

/// Assignment operators
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AssignOp {
    Assign,     // =
    AddAssign,  // +=
    SubAssign,  // -=
    MulAssign,  // *=
    DivAssign,  // /=
    ModAssign,  // %=
    PowAssign,  // **=
    AndAssign,  // &=
    OrAssign,   // |=
    XorAssign,  // ^=
    ShlAssign,  // <<=
    ShrAssign,  // >>=
    UShrAssign, // >>>=
}

/// Logical operators
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LogicalOp {
    And,        // &&
    Or,         // ||
    NullCoalesce, // ??
}

/// Update operators
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UpdateOp {
    Increment,  // ++
    Decrement,  // --
}
