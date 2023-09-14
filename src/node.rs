#[derive(Debug, Clone)]
pub struct Node {
    pub kind: NodeKind,
}

impl Node {
    pub fn new(kind: NodeKind) -> Self {
        Self { kind }
    }
}

#[derive(Debug, Clone)]
pub enum NodeKind {
    Body(Vec<Node>),

    IntegerLiteral(i64),
    BooleanLiteral(bool),
    NullLiteral,

    Identifier(String),

    BinaryOperation {
        left: Box<Node>,
        op: BinaryOperator,
        right: Box<Node>,
    },

    If {
        condition: Box<Node>,
        if_true: Box<Node>,
    },

    Send {
        value: Box<Node>,
        channel: Box<Node>,
    },
    Receive {
        value: Box<Node>,
        channel: Box<Node>,
        bind_channel: bool,
    },
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum BinaryOperator {
    Add,
    Subtract,
    Multiply,
    Divide,
}

#[derive(Debug, Clone)]
pub struct Item {
    pub kind: ItemKind,
}

#[derive(Debug, Clone)]
pub enum ItemKind {
    TaskDefinition {
        name: String,
        body: Node,
    }
}
