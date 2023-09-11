#[derive(Debug, Clone)]
pub struct Node {
    pub kind: NodeKind,
}

#[derive(Debug, Clone)]
pub enum NodeKind {
    IntegerLiteral(i64),
    Identifier(String),

    BinaryOperation {
        left: Box<Node>,
        op: BinaryOperator,
        right: Box<Node>,
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