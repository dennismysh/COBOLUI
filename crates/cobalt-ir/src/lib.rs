//! COBALT Intermediate Representation
//!
//! Typed IR produced by the COBOL parser. This is the platform-neutral
//! representation that renderers consume.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Top-level application structure produced by parsing a COBALT project.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CobaltApp {
    pub screens: Vec<Screen>,
    pub state: StateMap,
    pub handlers: Vec<Handler>,
    pub paragraphs: HashMap<String, Paragraph>,
}

/// A single screen (mapped from a level-01 SCREEN SECTION entry).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Screen {
    pub name: String,
    pub root: Node,
}

/// A node in the UI tree. Group levels become Containers; leaf levels
/// become Text, Numeric, or Button depending on their clauses.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Node {
    Container {
        name: String,
        children: Vec<Node>,
        style: Style,
    },
    Text {
        name: String,
        pic: PicClause,
        value: Option<String>,
        binding: Option<String>,
        style: Style,
    },
    Numeric {
        name: String,
        pic: PicClause,
        value: Option<String>,
        binding: Option<String>,
        style: Style,
    },
    Button {
        name: String,
        label: String,
        action: Option<String>,
        navigate: Option<String>,
        style: Style,
    },
}

impl Node {
    pub fn name(&self) -> &str {
        match self {
            Node::Container { name, .. }
            | Node::Text { name, .. }
            | Node::Numeric { name, .. }
            | Node::Button { name, .. } => name,
        }
    }
}

/// PIC clause parsed from `PIC X(n)`, `PIC 9(n)`, or `PIC A(n)`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PicClause {
    pub kind: PicKind,
    pub width: usize,
    pub decimals: Option<usize>,
}

/// The type indicator in a PIC clause.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PicKind {
    /// `PIC X` — alphanumeric
    Alphanumeric,
    /// `PIC 9` — numeric
    Numeric,
    /// `PIC A` — alphabetic
    Alphabetic,
}

/// Visual style properties for a node.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Style {
    /// Foreground color index (maps to theme palette).
    pub fg_color: Option<u8>,
    /// Background color index (maps to theme palette).
    pub bg_color: Option<u8>,
}

/// A field in WORKING-STORAGE that holds application state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateField {
    pub name: String,
    pub pic: PicClause,
    pub default_value: Option<String>,
    /// Level-88 condition names: (name, value).
    pub conditions: Vec<(String, String)>,
}

/// Application state: a map from variable name to its definition.
pub type StateMap = HashMap<String, StateField>;

/// A handler mapping from a paragraph name in PROCEDURE DIVISION.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Handler {
    pub name: String,
    pub paragraph_name: String,
}

/// A parsed COBOL statement.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Statement {
    /// MOVE source TO target
    Move { source: Expr, target: String },
    /// ADD source TO target
    Add { source: Expr, target: String },
    /// SUBTRACT source FROM target
    Subtract { source: Expr, target: String },
    /// MULTIPLY source BY target
    Multiply { source: Expr, target: String },
    /// DIVIDE source INTO target
    Divide { source: Expr, target: String },
    /// DISPLAY values
    Display { values: Vec<Expr> },
    /// IF condition THEN statements ELSE statements END-IF
    If {
        condition: Condition,
        then_body: Vec<Statement>,
        else_body: Vec<Statement>,
    },
    /// PERFORM paragraph-name
    Perform { paragraph: String },
    /// STRING sources DELIMITED BY delim INTO target
    StringConcat {
        sources: Vec<(Expr, Expr)>,
        into: String,
    },
    /// EVALUATE subject WHEN value ... WHEN OTHER ... END-EVALUATE
    Evaluate {
        subject: Expr,
        whens: Vec<WhenClause>,
        other: Vec<Statement>,
    },
    /// PERFORM paragraph UNTIL condition
    PerformUntil {
        paragraph: String,
        condition: Condition,
    },
    /// COMPUTE target = arithmetic-expression
    Compute {
        target: String,
        expression: ArithExpr,
    },
    /// ACCEPT target FROM source
    Accept {
        target: String,
        source: AcceptSource,
    },
    /// SET condition-name TO TRUE/FALSE
    Set {
        condition: String,
        value: bool,
    },
    /// STOP RUN
    StopRun,
}

/// A WHEN clause inside an EVALUATE statement.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhenClause {
    pub value: Expr,
    pub body: Vec<Statement>,
}

/// Arithmetic expression tree for COMPUTE statements.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ArithExpr {
    Num(f64),
    Var(String),
    BinOp {
        left: Box<ArithExpr>,
        op: ArithOp,
        right: Box<ArithExpr>,
    },
}

/// Arithmetic operators for COMPUTE expressions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ArithOp {
    Add,
    Subtract,
    Multiply,
    Divide,
}

/// Source for ACCEPT statement.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AcceptSource {
    Date,
    Time,
    DayOfWeek,
}

/// An expression (literal or variable reference).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Expr {
    Literal(String),
    NumericLiteral(f64),
    Variable(String),
}

/// A condition for IF statements.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Condition {
    Compare {
        left: Expr,
        op: CompareOp,
        right: Expr,
    },
    ConditionName(String),
}

/// Comparison operators.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CompareOp {
    Equal,
    NotEqual,
    GreaterThan,
    LessThan,
    GreaterOrEqual,
    LessOrEqual,
}

/// A parsed paragraph with its statements.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Paragraph {
    pub name: String,
    pub statements: Vec<Statement>,
}

/// An event record populated by the runtime's event loop.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventRecord {
    pub event_type: String,
    pub target: String,
    pub payload: String,
}

/// Runtime state values (the live data at render time).
pub type RuntimeState = HashMap<String, String>;
