pub struct Tokenizer<'s> {
    input: &'s [char],
    index: usize,

    indent_level: usize,
    indent_size: usize,
    indent_format: IndentFormat,

    pub tokens: Vec<Token>,
    pub errors: Vec<TokenizerError>,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum IndentFormat {
    Spaces,
    Tabs,
}

#[derive(Debug, Clone)]
pub struct Token {
    pub kind: TokenKind,
}

impl Token {
    pub fn new(kind: TokenKind) -> Self {
        Self { kind }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenKind {
    IntegerLiteral(i64),
    Identifier(String),

    Add,
    Subtract,
    Multiply,
    Divide,

    Equals,
    GreaterThan,
    LessThan,

    Range,

    SendArrow,
    ReceiveArrow,
    QuestionMark,

    LeftParen,
    RightParen,

    LeftBrace,
    RightBrace,
    Comma,

    Assign,

    KwTask,
    KwIf,
    KwWhile,
    KwLoop,
    KwTrue,
    KwFalse,
    KwNull,
    KwExit,

    Indent,
    Dedent,
    NewLine,

    EndOfFile,
}

#[derive(Debug, Clone)]
pub struct TokenizerError {
    message: String,
}

impl TokenizerError {
    fn new(s: impl Into<String>) -> Self {
        Self { message: s.into() }
    }
}

impl<'s> Tokenizer<'s> {
    pub fn new(input: &'s [char]) -> Self {
        Self {
            input,
            index: 0,

            indent_level: 0,
            indent_size: 0,
            indent_format: IndentFormat::Spaces,

            tokens: vec![],
            errors: vec![],
        }
    }

    pub fn tokenize(&mut self) {
        while !self.is_at_end() {
            if self.this() == '#' {
                // This is a line comment - consume until the end
                self.advance();
                while self.this() != '\n' && self.this() != '\0' {
                    self.advance();
                }
            } else if let Some(id) = self.try_get_identifier() {
                if let Some(kw) = Self::try_convert_to_keyword(&id) {
                    self.tokens.push(Token::new(kw))
                } else {
                    self.tokens.push(Token::new(TokenKind::Identifier(id)))
                }
            } else if self.this() == '\n' {
                self.tokens.push(Token::new(TokenKind::NewLine));
                self.advance();

                // Get the indentation on the next line
                match self.consume_all_indentation() {
                    Ok(new_indent_level) => {
                        // If it's increased by 1, emit an "indent" token
                        if new_indent_level == self.indent_level + 1 {
                            self.tokens.push(Token::new(TokenKind::Indent));
                        }
                        // If it's decreased by any amount, emit that number of "dedent" tokens
                        else if new_indent_level < self.indent_level {
                            let number_of_dedents = self.indent_level - new_indent_level;
                            for _ in 0..number_of_dedents {
                                self.tokens.push(Token::new(TokenKind::Dedent));
                            }
                        }
                        // If it's the same, nothing to do
                        else if new_indent_level == self.indent_level {
                            // Nothing!
                        }
                        // Anything else isn't something we expect!
                        else {
                            self.errors.push(TokenizerError::new("indentation increased too much"))
                        }

                        self.indent_level = new_indent_level;
                    },
                    Err(e) => self.errors.push(e),
                };
            } else if self.this() == '<' && self.next() == '-' {
                self.advance();
                self.advance();
                self.tokens.push(Token::new(TokenKind::ReceiveArrow));
            } else if self.this() == '-' && self.next() == '>' {
                self.advance();
                self.advance();
                self.tokens.push(Token::new(TokenKind::SendArrow));
            } else if self.this().is_ascii_digit() || self.this() == '-' {
                // Parse the number into a character list
                let mut buffer = vec![self.this()];
                self.advance();

                while self.this().is_ascii_digit() {
                    buffer.push(self.this());
                    self.advance();
                }

                // Convert into an actual integer
                let buffer_str: String = buffer.iter().collect();
                let int = buffer_str.parse::<i64>().unwrap();
                self.tokens.push(Token::new(TokenKind::IntegerLiteral(int)))
            } else if self.this().is_whitespace() {
                self.advance(); // Skip whitespace
            } else {
                // Easy single-character cases
                match self.this() {
                    '?' => self.tokens.push(Token::new(TokenKind::QuestionMark)),

                    '+' => self.tokens.push(Token::new(TokenKind::Add)),
                    '-' => self.tokens.push(Token::new(TokenKind::Subtract)),
                    '*' => self.tokens.push(Token::new(TokenKind::Multiply)),
                    '/' => self.tokens.push(Token::new(TokenKind::Divide)),

                    '(' => self.tokens.push(Token::new(TokenKind::LeftParen)),
                    ')' => self.tokens.push(Token::new(TokenKind::RightParen)),

                    '[' => self.tokens.push(Token::new(TokenKind::LeftBrace)),
                    ']' => self.tokens.push(Token::new(TokenKind::RightBrace)),
                    ',' => self.tokens.push(Token::new(TokenKind::Comma)),

                    '=' if self.next() == '=' => {
                        self.advance();
                        self.tokens.push(Token::new(TokenKind::Equals))
                    },
                    '=' => self.tokens.push(Token::new(TokenKind::Assign)),
                    '>' => self.tokens.push(Token::new(TokenKind::GreaterThan)),
                    '<' => self.tokens.push(Token::new(TokenKind::LessThan)),

                    '.' if self.next() == '.' => {
                        self.advance();
                        self.tokens.push(Token::new(TokenKind::Range))
                    },

                    _ => self.push_unexpected_error(),
                }
                self.advance();
            }
        }

        self.tokens.push(Token::new(TokenKind::EndOfFile))
    }

    fn this(&self) -> char {
        if self.is_at_end() {
            '\0'
        } else {
            self.input[self.index]
        }
    }

    fn next(&self) -> char {
        if self.next_is_at_end() {
            '\0'
        } else {
            self.input[self.index + 1]
        }
    }

    fn is_at_end(&self) -> bool {
        self.index >= self.input.len()
    }

    fn next_is_at_end(&self) -> bool {
        self.index + 1 >= self.input.len()
    }

    fn advance(&mut self) {
        self.index += 1;
    }

    fn try_get_identifier(&mut self) -> Option<String> {
        if self.this().is_alphabetic() || self.this() == '_' || self.this() == '$' {
            // Looks like an identifier! Let's go...
            let mut buffer = vec![self.this()];
            self.advance();

            while self.this().is_alphanumeric() || self.this() == '_' {
                buffer.push(self.this());
                self.advance();
            }

            Some(buffer.iter().collect())
        } else {
            None
        }
    }

    fn try_convert_to_keyword(s: &str) -> Option<TokenKind> {
        match s {
            "task" => Some(TokenKind::KwTask),
            "true" => Some(TokenKind::KwTrue),
            "false" => Some(TokenKind::KwFalse),
            "null" => Some(TokenKind::KwNull),
            "if" => Some(TokenKind::KwIf),
            "while" => Some(TokenKind::KwWhile),
            "loop" => Some(TokenKind::KwLoop),
            "exit" => Some(TokenKind::KwExit),
            _ => None,
        }
    }

    fn consume_all_indentation(&mut self) -> Result<usize, TokenizerError> {
        // If lines are blank, ignore them and move onto the next one
        while self.this() == '\n' {
            self.advance();
        }

        // Try consuming a single indentation character first, to get the baseline format
        let Some(given_format) = self.consume_one_indentation() else {
            // There's no indentation - return nothing
            return Ok(0)
        };

        // Have we already decided on an expected indent format?
        let mut set_indent_size = false;
        if self.indent_size > 0 {
            // Yes - check this matches the expected format
            if self.indent_format != given_format {
                return Err(TokenizerError::new("indentation format mismatch"))
            }
        } else {
            // No - we've got one now!
            self.indent_format = given_format;
            set_indent_size = true;
        }

        // Remember, we already consumed a character to check there was any indentation at all
        let mut current_indent_size = 1;
        loop {
            let this_indent = self.consume_one_indentation();

            // Check if the indentation is over
            if this_indent.is_none() {
                // Is there any content on this line, or does it end immediately after this
                // indentation we consumed?
                if self.this() == '\n' {
                    // It's a blank line; ignore everything we gathered and try again on the next
                    self.advance();
                    return self.consume_all_indentation();
                }

                if set_indent_size {
                    self.indent_size = current_indent_size;
                }

                // Convert "size" (number of chars) into "level" (number of full indents)
                if current_indent_size % self.indent_size != 0 {
                    return Err(TokenizerError::new("incomplete indentation"))
                }
                let indent_level = current_indent_size / self.indent_size;
                return Ok(indent_level)
            }

            if this_indent.unwrap() != self.indent_format {
                return Err(TokenizerError::new("indentation mismatch"))
            }
            
            current_indent_size += 1;
        }
    }

    fn consume_one_indentation(&mut self) -> Option<IndentFormat> {
        match self.this() {
            '\t' => {
                self.advance();
                Some(IndentFormat::Tabs)
            }
            ' ' => {
                self.advance();
                Some(IndentFormat::Spaces)
            }
            _ => None,
        }
    }

    fn push_unexpected_error(&mut self) {
        let c = self.this();
        self.errors.push(TokenizerError::new(format!("unexpected char {c:?}")));
    }
}
