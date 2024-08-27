use crate::token::{LiteralType, Token, TokenType};

// The scanner will scan through the input text and produce a list of tokens
pub struct Scanner {
    // Source string stored as bytes to save converting at every step
    source: Vec<u8>,
    pub tokens: Vec<Token>,
    start: u32,
    current: u32,
    line: u32,
    pub(crate) had_error: bool,
}

impl Scanner {
    pub fn new(source: Vec<u8>) -> Scanner {
        Scanner {
            source,
            tokens: Vec::new(),
            start: 0,
            current: 0,
            line: 1,
            had_error: false,
        }
    }

    // Scan all tokens in the source
    pub fn scan_tokens(&mut self) {
        while !self.is_at_end() {
            self.start = self.current;
            self.scan_token();
        }

        self.tokens.push(Token::new(TokenType::Eof, String::default(), LiteralType::Empty, self.line))
    }

    // Debug function to print the stored tokens
    pub fn print_tokens(&self) {
        self.tokens.clone().into_iter().for_each(|t| println!("-- {}", t.to_string()))
    }

    // Check if we are at the end of the source
    fn is_at_end(&self) -> bool {
        self.current >= self.source.len() as u32
    }

    // Scan the next token in the source
    pub fn scan_token(&mut self) {
        let c: u8 = self.advance();

        match c {
            // Single character token types
            b'(' => self.add_token(TokenType::LeftParen, None),
            b')' => self.add_token(TokenType::RightParen, None),
            b'{' => self.add_token(TokenType::LeftBrace, None),
            b'}' => self.add_token(TokenType::RightBrace, None),
            b',' => self.add_token(TokenType::Comma, None),
            b'.' => self.add_token(TokenType::Dot, None),
            b'-' => self.add_token(TokenType::Minus, None),
            b'+' => self.add_token(TokenType::Plus, None),
            b';' => self.add_token(TokenType::Semicolon, None),
            b'*' => self.add_token(TokenType::Star, None),

            // One or Two Character Tokens
            b'!' => {
                if self.match_char(b'=') {
                    self.add_token(TokenType::BangEqual, None)
                } else {
                    self.add_token(TokenType::Bang, None)
                }
            },
            b'=' => {
                if self.match_char(b'=') {
                    self.add_token(TokenType::EqualEqual, None)
                } else {
                    self.add_token(TokenType::Equal, None)
                }
            },
            b'<' => {
                if self.match_char(b'=') {
                    self.add_token(TokenType::LessEqual, None)
                } else {
                    self.add_token(TokenType::Less, None)
                }
            },
            b'>' => {
                if self.match_char(b'=') {
                    self.add_token(TokenType::GreaterEqual, None)
                } else {
                    self.add_token(TokenType::Greater, None)
                }
            },
            b'/' => {
                if self.match_char(b'/') {
                    // A comment goes until the end of the line
                    while self.peek() != b'\n' && !self.is_at_end() {
                        self.advance();
                    }
                } else {
                    self.add_token(TokenType::Slash, None)
                }
            },

            // Ignore whitespace
            b' ' => (),
            b'\r' => (),
            b'\t' => (),
            b'\n' => self.line += 1,

            // Handle strings
            b'"' => self.string(),

            // Numbers
            f if self.is_digit(f) => self.number(),
            f if self.is_alpha(f) => self.identifier(),

            _ => {
                self.error(self.line, "Unexpected character.")
            }
        }
    }

    // Return the current character and advance the current pointer
    fn advance(&mut self) -> u8 {
        let c: u8 = self.source[self.current as usize];
        self.current += 1;
        c
    }

    // Peek at the current character without advancing the current pointer
    fn peek(&self) -> u8 {
        if self.is_at_end() {
            return b'\0';
        }
        self.source[self.current as usize]
    }

    // Peek the next character without advancing
    fn peek_next(&self) -> u8 {
        if self.current + 1 >= self.source.len() as u32 {
            return b'\0';
        }
        return self.source[(self.current + 1) as usize]
    }

    // Add a token to the list of tokens
    fn add_token(&mut self, token_type: TokenType, literal: Option<LiteralType>) {
        let text: String = self.range_to_string(self.start, self.current);
        let literal = match literal {
            Some(l) => l,
            None => LiteralType::Empty
        };
        self.tokens.push(Token::new(token_type, text, literal, self.line));
    }

    // Is the character a digit?
    fn is_digit(&self, c: u8) -> bool {
        c.is_ascii_digit()
    }

    // Is the character alpha?
    fn is_alpha(&self, c: u8) -> bool {
        c.is_ascii_alphabetic() || c == b'_'
    }

    // Is the character alphaNumeric?
    fn is_alpha_numeric(&self, c: u8) -> bool {
        self.is_alpha(c) || self.is_digit(c)
    }

    // Match the current character. If it is equal to expected, advance current and return true
    // Otherwise return false
    fn match_char(&mut self, expected: u8) -> bool {
        if self.is_at_end() {
            return false;
        }

        if self.source[self.current as usize] != expected {
            return false;
        }

        self.current += 1;
        true
    }

    // Handle strings
    fn string(&mut self) {
        // Run until eof or closing character
        while self.peek() != b'"' && !self.is_at_end() {
            if self.peek() == b'\n' {
                self.line +=1;
            }
            self.advance();
        }

        if self.is_at_end() {
            self.error(self.line, "Unterminated string.");
        }

        // To advance past the closing "
        self.advance();

        // Trim surrounding quotes
        let value: String = self.range_to_string(self.start + 1, self.current - 1);
        self.add_token(TokenType::String, Some(LiteralType::String(value)));
    }

    // Handle floating point numbers and decimals. Does not support decimal at the start or end
    fn number(&mut self) {
        while self.is_digit(self.peek()) {
            self.advance();
        }

        // Check if we have a decimal, then consume the trailing digits
        if self.peek() == b'.' && self.is_digit(self.peek_next()) {
            // Consume the decimal
            self.advance();

            while self.is_digit(self.peek()) {
                self.advance();
            }
        }
        let literal_str = self.range_to_string(self.start, self.current);
        let literal: LiteralType = LiteralType::Number(literal_str.parse::<f64>().unwrap());
        self.add_token(TokenType::Number, Some(literal));
    }

    // Handle identifiers
    fn identifier(&mut self) {
        while self.is_alpha_numeric(self.peek()) {
            self.advance();
        }

        let text = self.range_to_string(self.start, self.current);
        let token_type = self.get_identifier_type(&text);
        self.add_token(token_type, None)
    }

    fn get_identifier_type(&self, text: &str) -> TokenType {
        match text {
            "and" => TokenType::And,
            "class" => TokenType::Class,
            "else" => TokenType::Else,
            "false" => TokenType::False,
            "true" => TokenType::True,
            "funk" => TokenType::Funk,
            "for" => TokenType::For,
            "if" => TokenType::If,
            "null" => TokenType::Null,
            "or" => TokenType::Or,
            "print" => TokenType::Print,
            "return" => TokenType::Return,
            "super" => TokenType::Super,
            "this" => TokenType::This,
            "var" => TokenType::Var,
            "while" => TokenType::While,
            _ => TokenType::Identifier
        }
    }

    // Convert a start and end range into a string from the source byte array
    fn range_to_string(&mut self, start: u32, end: u32) -> String {
        // Safe to unwrap as we know it is utf-8
        String::from_utf8(self.source[start as usize .. end as usize].to_vec()).unwrap()
    }

    // Report an error to the user
    pub fn error(&mut self, line: u32, message: &str) {
        self.report(line, "", message);
    }

    // Report an error to the user with a location
    pub fn report(&mut self, line: u32, location: &str, message: &str) {
        self.had_error = true;
        eprintln!("[line {}] Error {}: {}", line, location, message);
    }
}