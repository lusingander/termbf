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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
}
