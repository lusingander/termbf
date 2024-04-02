use std::collections::HashMap;

pub struct Interpreter {
    memory: Vec<u8>,
    ptr: u8,

    tokens: Vec<Token>,
    jump_map: HashMap<Token, usize>,
    cur: usize,
    input: String,
    output: String,

    step_count: usize,
}

impl Interpreter {
    pub fn new(source: &str, input: &str) -> Interpreter {
        let tokens = tokenize(source);
        let jump_map = build_jump_map(&tokens);
        Interpreter {
            memory: vec![0; u8::MAX as usize],
            ptr: 0,
            tokens,
            jump_map,
            cur: 0,
            input: input.to_string(),
            output: String::new(),
            step_count: 0,
        }
    }

    pub fn input(&self) -> &str {
        &self.input
    }

    pub fn set_input(&mut self, input: String) {
        self.input = input;
    }

    pub fn output(&self) -> &str {
        &self.output
    }

    pub fn memory(&self) -> &Vec<u8> {
        &self.memory
    }

    pub fn current_ptr(&self) -> u8 {
        self.ptr
    }

    pub fn total_step_count(&self) -> usize {
        self.step_count
    }

    pub fn current_line_and_pos(&self) -> Option<(usize, usize)> {
        self.current_token().map(|t| (t.line, t.pos))
    }

    pub fn end(&self) -> bool {
        self.cur >= self.tokens.len()
    }

    pub fn running(&self) -> bool {
        self.cur > 0
    }

    pub fn step(&mut self) {
        if let Some(token) = self.current_token() {
            match token.tp {
                TokenType::Plus => {
                    let v = self.current_value();
                    *v = v.checked_add(1).unwrap();
                    self.cur += 1;
                }
                TokenType::Minus => {
                    let v = self.current_value();
                    *v = v.checked_sub(1).unwrap();
                    self.cur += 1;
                }
                TokenType::RightAngle => {
                    self.ptr = self.ptr.checked_add(1).unwrap();
                    self.cur += 1;
                }
                TokenType::LeftAngle => {
                    self.ptr = self.ptr.checked_sub(1).unwrap();
                    self.cur += 1;
                }
                TokenType::LeftSquare => {
                    let v = self.current_value();
                    if *v == 0 {
                        self.cur = self.jump_idx(&token);
                    } else {
                        self.cur += 1;
                    }
                }
                TokenType::RightSquare => {
                    let v = self.current_value();
                    if *v != 0 {
                        self.cur = self.jump_idx(&token);
                    } else {
                        self.cur += 1;
                    }
                }
                TokenType::Dot => {
                    let v = self.current_value();
                    let c = *v as char;
                    self.output.push(c);
                    self.cur += 1;
                }
                TokenType::Comma => {
                    let mut cs = self.input.chars();
                    let c = cs.next().unwrap_or(0 as char); // EOF: 0
                    self.input = cs.collect();
                    let v = self.current_value();
                    *v = c as u8;
                    self.cur += 1;
                }
            }
        }

        self.step_count += 1;
    }

    fn current_token(&self) -> Option<Token> {
        self.tokens.get(self.cur).copied()
    }

    fn current_value(&mut self) -> &mut u8 {
        self.memory.get_mut(self.ptr as usize).unwrap()
    }

    fn jump_idx(&self, token: &Token) -> usize {
        *self.jump_map.get(token).unwrap()
    }
}

fn tokenize(source: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    for (line, s) in source.lines().enumerate() {
        for (pos, c) in s.chars().enumerate() {
            if let Some(tp) = char_to_token_type(c) {
                let token = Token::new(tp, line, pos);
                tokens.push(token);
            }
        }
    }
    tokens
}

fn char_to_token_type(c: char) -> Option<TokenType> {
    match c {
        '+' => Some(TokenType::Plus),
        '-' => Some(TokenType::Minus),
        '>' => Some(TokenType::RightAngle),
        '<' => Some(TokenType::LeftAngle),
        '[' => Some(TokenType::LeftSquare),
        ']' => Some(TokenType::RightSquare),
        '.' => Some(TokenType::Dot),
        ',' => Some(TokenType::Comma),
        _ => None,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Token {
    tp: TokenType,
    line: usize,
    pos: usize,
}

impl Token {
    fn new(tp: TokenType, line: usize, pos: usize) -> Token {
        Token { tp, line, pos }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum TokenType {
    Plus,
    Minus,
    RightAngle,
    LeftAngle,
    LeftSquare,
    RightSquare,
    Dot,
    Comma,
}

fn build_jump_map(tokens: &[Token]) -> HashMap<Token, usize> {
    let mut m = HashMap::new();
    let mut stack: Vec<(&Token, usize)> = Vec::new();
    for (i, t) in tokens.iter().enumerate() {
        match t.tp {
            TokenType::LeftSquare => {
                stack.push((t, i));
            }
            TokenType::RightSquare => {
                if let Some((tt, ii)) = stack.pop() {
                    m.insert(*t, ii + 1);
                    m.insert(*tt, i + 1);
                }
            }
            _ => {}
        }
    }
    m
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenize() {
        let source = "
<>[] // xxx
abc .,
あいうえお
+- # foobar <>
";
        let actual = tokenize(source);
        let expected = vec![
            Token::new(TokenType::LeftAngle, 1, 0),
            Token::new(TokenType::RightAngle, 1, 1),
            Token::new(TokenType::LeftSquare, 1, 2),
            Token::new(TokenType::RightSquare, 1, 3),
            Token::new(TokenType::Dot, 2, 4),
            Token::new(TokenType::Comma, 2, 5),
            Token::new(TokenType::Plus, 4, 0),
            Token::new(TokenType::Minus, 4, 1),
            Token::new(TokenType::LeftAngle, 4, 12),
            Token::new(TokenType::RightAngle, 4, 13),
        ];
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_interpreter_hello_world() {
        let source = "++++++++[>++++[>++>+++>+++>+<<<<-]>+>+>->>+[<]<-]>>.>---.+++++++..+++.>>.<-.<.+++.------.--------.>>+.>++.";
        let output = run_interpreter(source, "");
        assert_eq!(output, "Hello World!\n")
    }

    #[test]
    fn test_interpreter_echo() {
        let source = ",[.,]";
        let output = run_interpreter(source, "Rust");
        assert_eq!(output, "Rust")
    }

    fn run_interpreter(source: &str, input: &str) -> String {
        let mut interpreter = Interpreter::new(source, input);
        while !interpreter.end() {
            interpreter.step()
        }
        interpreter.output
    }
}
