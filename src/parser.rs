/*
Syntax example:

    task A
        123 -> B
        456 -> B
    

    task B
        val1 <- ?x
        val2 <- x
        val1 + val2 -> $out
    
*/

use crate::{tokenizer::{Token, TokenKind}, node::{Item, Node, NodeKind, ItemKind, BinaryOperator}};

pub struct Parser<'t> {
    tokens: &'t [Token],
    index: usize,

    pub items: Vec<Item>,
    pub errors: Vec<ParserError>,
}

#[derive(Debug, Clone)]
pub struct ParserError {
    message: String,
}

impl ParserError {
    fn new(s: impl Into<String>) -> Self {
        Self { message: s.into() }
    }
}

impl<'t> Parser<'t> {
    pub fn new(tokens: &'t [Token]) -> Self {
        Self {
            tokens,
            index: 0,
            items: vec![],
            errors: vec![],
        }
    }

    pub fn parse_top_level(&mut self) {
        self.parse_items();
    }

    fn parse_items(&mut self) {
        loop {
            match self.this().kind {
                TokenKind::KwTask => { self.parse_task(); },
                TokenKind::NewLine => self.advance(),
                TokenKind::EndOfFile => break,
                _ => {
                    self.push_unexpected_error();
                    break;
                }
            }
        }
    }

    fn parse_task(&mut self) -> Option<()> {
        // Skip keyword
        self.expect(TokenKind::KwTask)?;

        // Get name
        let TokenKind::Identifier(name) = &self.this().kind else {
            self.push_unexpected_error(); return None;
        };
        let name = name.to_string();
        self.advance();

        // Expect newline, then indentation
        self.expect(TokenKind::NewLine)?;
        self.expect(TokenKind::Indent)?;

        // Parse body
        let body = self.parse_body();

        self.items.push(Item {
            kind: ItemKind::TaskDefinition {
                name,
                body,
            }
        });
        Some(())
    }

    fn parse_body(&mut self) -> Node {
        // Build up a body until we hit a dedent
        // (If there is nested indentation, that should be handled by the child parser)
        let mut body_nodes = vec![];
        while self.this().kind != TokenKind::Dedent {
            if let Some(node) = self.parse_statement() {
                body_nodes.push(node);
            }
        }
        self.advance(); // skip the dedent

        Node::new(NodeKind::Body(body_nodes))
    }

    fn parse_statement(&mut self) -> Option<Node> {
        let stmt = match self.this().kind {
            TokenKind::KwIf => self.parse_if(),
            _ => self.parse_send_receive(),
        };

        while self.this().kind == TokenKind::NewLine {
            self.advance();
        }

        stmt
    }

    fn parse_if(&mut self) -> Option<Node> {
        // Skip keyword
        self.expect(TokenKind::KwIf)?;

        // Parse condition
        let condition = self.parse_expression()?;

        // Expect newline, then indentation
        self.expect(TokenKind::NewLine)?;
        self.expect(TokenKind::Indent)?;

        // Parse body
        let body = self.parse_body();

        Some(Node::new(NodeKind::If {
            condition: Box::new(condition),
            if_true: Box::new(body),
        }))
    }

    fn parse_send_receive(&mut self) -> Option<Node> {
        let left = self.parse_expression()?;

        match self.this().kind {
            TokenKind::SendArrow => {
                self.advance();
                let right = self.parse_atom()?;

                Some(Node::new(NodeKind::Send {
                    value: Box::new(left),
                    channel: Box::new(right),
                }))
            }

            TokenKind::ReceiveArrow => {
                self.advance();

                let mut bind_channel = false;
                if self.this().kind == TokenKind::QuestionMark {
                    bind_channel = true;
                    self.advance();
                }

                let right = self.parse_atom()?;

                Some(Node::new(NodeKind::Receive {
                    value: Box::new(left),
                    channel: Box::new(right),
                    bind_channel,
                }))
            }

            _ => Some(left),
        }
    }

    fn parse_expression(&mut self) -> Option<Node> {
        self.parse_mul_div()
    }

    fn parse_mul_div(&mut self) -> Option<Node> {
        let mut left = self.parse_add_sub()?;

        loop {
            match self.this().kind {
                TokenKind::Multiply => {
                    self.advance();
                    left = Node::new(NodeKind::BinaryOperation {
                        left: Box::new(left),
                        op: BinaryOperator::Multiply,
                        right: Box::new(self.parse_expression()?),
                    });
                },
                TokenKind::Divide  => {
                    self.advance();
                    left = Node::new(NodeKind::BinaryOperation {
                        left: Box::new(left),
                        op: BinaryOperator::Divide,
                        right: Box::new(self.parse_expression()?),
                    });
                },

                _ => break,
            }
        }

        Some(left)
    }

    fn parse_add_sub(&mut self) -> Option<Node> {
        let mut left = self.parse_comparison()?;

        loop {
            match self.this().kind {
                TokenKind::Add => {
                    self.advance();
                    left = Node::new(NodeKind::BinaryOperation {
                        left: Box::new(left),
                        op: BinaryOperator::Add,
                        right: Box::new(self.parse_expression()?),
                    });
                },
                TokenKind::Subtract => {
                    self.advance();
                    left = Node::new(NodeKind::BinaryOperation {
                        left: Box::new(left),
                        op: BinaryOperator::Subtract,
                        right: Box::new(self.parse_expression()?),
                    });
                },

                _ => break,
            }
        }

        Some(left)
    }

    fn parse_comparison(&mut self) -> Option<Node> {
        let mut left = self.parse_parens()?;

        loop {
            match self.this().kind {
                TokenKind::Equals => {
                    self.advance();
                    left = Node::new(NodeKind::BinaryOperation {
                        left: Box::new(left),
                        op: BinaryOperator::Equals,
                        right: Box::new(self.parse_expression()?),
                    });
                },
                TokenKind::LessThan => {
                    self.advance();
                    left = Node::new(NodeKind::BinaryOperation {
                        left: Box::new(left),
                        op: BinaryOperator::LessThan,
                        right: Box::new(self.parse_expression()?),
                    });
                },
                TokenKind::GreaterThan => {
                    self.advance();
                    left = Node::new(NodeKind::BinaryOperation {
                        left: Box::new(left),
                        op: BinaryOperator::GreaterThan,
                        right: Box::new(self.parse_expression()?),
                    });
                },

                _ => break,
            }
        }

        Some(left)
    }

    fn parse_parens(&mut self) -> Option<Node> {
        if self.this().kind == TokenKind::LeftParen {
            self.advance();
            let result = self.parse_expression()?;
            
            let TokenKind::RightParen = &self.this().kind else {
                self.push_unexpected_error(); return None;
            };
            self.advance();
    
            Some(result)
        } else {
            self.parse_atom()
        }
    }

    fn parse_atom(&mut self) -> Option<Node> {
        match &self.this().kind {
            TokenKind::Identifier(id) => {
                let x = Some(Node::new(NodeKind::Identifier(id.clone())));
                self.advance();
                x
            },

            TokenKind::IntegerLiteral(int) => {
                let x = Some(Node::new(NodeKind::IntegerLiteral(*int)));
                self.advance();
                x
            },
            TokenKind::KwTrue => {
                self.advance();
                Some(Node::new(NodeKind::BooleanLiteral(true)))
            },
            TokenKind::KwFalse => {
                self.advance();
                Some(Node::new(NodeKind::BooleanLiteral(false)))
            },
            TokenKind::KwNull => {
                self.advance();
                Some(Node::new(NodeKind::NullLiteral))
            }
            
            _ => {
                self.push_unexpected_error();
                self.advance();
                None
            }
        }
    }

    fn is_at_end(&self) -> bool {
        self.index >= self.tokens.len()
    }

    fn advance(&mut self) {
        self.index += 1;
    }

    fn this(&self) -> &Token {
        if self.is_at_end() {
            let last = self.tokens.last().unwrap();
            let TokenKind::EndOfFile = last.kind else { unreachable!() };
            last
        } else {
            &self.tokens[self.index]
        }
    }

    #[must_use]
    fn expect(&mut self, kind: TokenKind) -> Option<()> {
        if &self.this().kind != &kind {
            self.push_unexpected_error();
            return None;
        };
        self.advance();

        Some(())
    }

    fn push_unexpected_error(&mut self) {
        let token = self.this();
        self.errors.push(ParserError::new(format!("unexpected token {token:?}")));
    }
}
