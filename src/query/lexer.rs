use std::cmp::PartialEq;
use std::iter::Peekable;
use std::str::Chars;

use crate::builds::BuildStatus;
use crate::query::Operator;
use crate::DuckResult;

pub struct TokenStream {
    tokens: Vec<Token>,
    position: usize,
}

impl TokenStream {
    pub fn new(tokens: Vec<Token>) -> TokenStream {
        Self {
            tokens,
            position: 0,
        }
    }

    #[cfg(test)]
    pub fn get_tokens(&self) -> Vec<Token> {
        return self.tokens.clone();
    }

    pub fn current(&self) -> Option<&Token> {
        if self.position >= self.tokens.len() {
            return None;
        }
        Some(&self.tokens[self.position])
    }

    pub fn move_next(&mut self) -> bool {
        if self.position >= self.tokens.len() {
            return false;
        }
        self.position += 1;
        true
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    Word(String),
    Literal(String),
    Integer(i64),
    Status(BuildStatus),
    Operator(Operator),
    Or,
    And,
    Not,
    True,
    False,
    LParen,
    RParen,
}

impl Token {
    pub fn is_operator(&self) -> bool {
        match &self {
            Token::Operator(_) => true,
            _ => false,
        }
    }

    pub fn get_operator(&self) -> Option<Operator> {
        match &self {
            Token::Operator(op) => Some(op.clone()),
            _ => None,
        }
    }
}

///////////////////////////////////////////////////////////
// Lexer

pub fn tokenize(text: &str) -> DuckResult<TokenStream> {
    let mut result: Vec<Token> = Vec::new();
    let mut stream = text.chars().peekable();
    loop {
        match stream.peek() {
            None => break,
            Some(&character) => match character {
                ' ' => {
                    stream.next();
                }
                'a'..='z' | 'A'..='Z' => {
                    let word = read_word(&mut stream)?;
                    match &word.to_lowercase()[..] {
                        "or" => result.push(Token::Or),
                        "and" => result.push(Token::And),
                        "not" => result.push(Token::Not),
                        "true" => result.push(Token::True),
                        "false" => result.push(Token::False),
                        "success" => result.push(Token::Status(BuildStatus::Success)),
                        "canceled" => result.push(Token::Status(BuildStatus::Canceled)),
                        "cancelled" => result.push(Token::Status(BuildStatus::Canceled)),
                        "failed" => result.push(Token::Status(BuildStatus::Failed)),
                        "running" => result.push(Token::Status(BuildStatus::Running)),
                        "skipped" => result.push(Token::Status(BuildStatus::Skipped)),
                        "queued" => result.push(Token::Status(BuildStatus::Queued)),
                        _ => result.push(Token::Word(word)),
                    }
                }
                '0'..='9' => {
                    let integer = read_integer(&mut stream)?;
                    result.push(Token::Integer(integer))
                }
                '=' | '!' | '>' | '<' | '&' | '|' => {
                    let symbols = read_symbols(&mut stream)?;
                    match &symbols[..] {
                        "!" => result.push(Token::Not),
                        "&&" => result.push(Token::And),
                        "||" => result.push(Token::Or),
                        "==" => result.push(Token::Operator(Operator::EqualTo)),
                        "!=" => result.push(Token::Operator(Operator::NotEqualTo)),
                        ">" => result.push(Token::Operator(Operator::GreaterThan)),
                        ">=" => result.push(Token::Operator(Operator::GreaterThanOrEqualTo)),
                        "<" => result.push(Token::Operator(Operator::LessThan)),
                        "<=" => result.push(Token::Operator(Operator::LessThanOrEqualTo)),
                        _ => return Err(format_err!("Unexpected operator '{}'.", symbols)),
                    }
                }
                '\'' => {
                    stream.next();
                    let literal = read_literal(&mut stream)?;
                    result.push(Token::Literal(literal))
                }
                '(' => {
                    result.push(Token::LParen);
                    stream.next();
                }
                ')' => {
                    result.push(Token::RParen);
                    stream.next();
                }
                _ => return Err(format_err!("Unexpected token '{}'.", character)),
            },
        }
    }
    Ok(TokenStream::new(result))
}

fn read_word(stream: &mut Peekable<Chars>) -> DuckResult<String> {
    let mut accumulator: Vec<char> = Vec::new();
    loop {
        match stream.peek() {
            None => break,
            Some(&character) => match character {
                'a'..='z' | 'A'..='Z' => {
                    accumulator.push(character);
                    stream.next();
                }
                _ => break,
            },
        }
    }
    Ok(accumulator.into_iter().collect())
}

fn read_literal(stream: &mut Peekable<Chars>) -> DuckResult<String> {
    let mut accumulator: Vec<char> = Vec::new();
    loop {
        match stream.peek() {
            None => return Err(format_err!("Unexpected end of string.")),
            Some(&character) => match character {
                '\'' => {
                    stream.next();
                    break;
                }
                _ => {
                    accumulator.push(character);
                    stream.next();
                }
            },
        }
    }
    Ok(accumulator.into_iter().collect())
}

fn read_integer(stream: &mut Peekable<Chars>) -> DuckResult<i64> {
    let mut accumulator: Vec<char> = Vec::new();
    loop {
        match stream.peek() {
            None => break,
            Some(&character) => match character {
                '0'..='9' => {
                    accumulator.push(character);
                    stream.next();
                }
                _ => break,
            },
        }
    }
    let result: String = accumulator.into_iter().collect();
    Ok(result.parse::<i64>()?)
}

fn read_symbols(stream: &mut Peekable<Chars>) -> DuckResult<String> {
    let mut accumulator: Vec<char> = Vec::new();
    loop {
        match stream.peek() {
            None => break,
            Some(&character) => match character {
                '=' | '!' | '>' | '<' | '&' | '|' => {
                    accumulator.push(character);
                    stream.next();
                }
                _ => break,
            },
        }
    }
    Ok(accumulator.into_iter().collect())
}

///////////////////////////////////////////////////////////
// Tests

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_recognize_word() {
        // Given, When
        let tokens = tokenize("foo baR BAZ Qux").unwrap().get_tokens();

        // Then
        assert_eq!(4, tokens.len());
        assert_eq!(tokens[0], Token::Word("foo".to_owned()));
        assert_eq!(tokens[1], Token::Word("baR".to_owned()));
        assert_eq!(tokens[2], Token::Word("BAZ".to_owned()));
        assert_eq!(tokens[3], Token::Word("Qux".to_owned()));
    }

    #[test]
    fn should_recognize_string_literals() {
        // Given, When
        let tokens = tokenize("'hello' 'world'").unwrap().get_tokens();

        // Then
        assert_eq!(2, tokens.len());
        assert_eq!(tokens[0], Token::Literal("hello".to_owned()));
        assert_eq!(tokens[1], Token::Literal("world".to_owned()));
    }

    #[test]
    fn should_recognize_integers() {
        // Given, When
        let tokens = tokenize("3 99 89").unwrap().get_tokens();

        // Then
        assert_eq!(3, tokens.len());
        assert_eq!(tokens[0], Token::Integer(3));
        assert_eq!(tokens[1], Token::Integer(99));
        assert_eq!(tokens[2], Token::Integer(89));
    }

    #[test]
    fn should_recognize_keywords() {
        // Given, When
        let tokens = tokenize("OR AND NOT TRUE FALSE ! && ||")
            .unwrap()
            .get_tokens();

        // Then
        assert_eq!(8, tokens.len());
        assert_eq!(tokens[0], Token::Or);
        assert_eq!(tokens[1], Token::And);
        assert_eq!(tokens[2], Token::Not);
        assert_eq!(tokens[3], Token::True);
        assert_eq!(tokens[4], Token::False);
        assert_eq!(tokens[5], Token::Not);
        assert_eq!(tokens[6], Token::And);
        assert_eq!(tokens[7], Token::Or);
    }

    #[test]
    fn should_recognize_scopes() {
        // Given, When
        let tokens = tokenize("(( ) )()").unwrap().get_tokens();

        // Then
        assert_eq!(6, tokens.len());
        assert_eq!(tokens[0], Token::LParen);
        assert_eq!(tokens[1], Token::LParen);
        assert_eq!(tokens[2], Token::RParen);
        assert_eq!(tokens[3], Token::RParen);
        assert_eq!(tokens[4], Token::LParen);
        assert_eq!(tokens[5], Token::RParen);
    }

    #[test]
    fn should_recognize_statuses() {
        // Given, When
        let tokens = tokenize("success failed canceled cancelled queued running skipped")
            .unwrap()
            .get_tokens();

        // Then
        assert_eq!(7, tokens.len());
        assert_eq!(tokens[0], Token::Status(BuildStatus::Success));
        assert_eq!(tokens[1], Token::Status(BuildStatus::Failed));
        assert_eq!(tokens[2], Token::Status(BuildStatus::Canceled));
        assert_eq!(tokens[3], Token::Status(BuildStatus::Canceled));
        assert_eq!(tokens[4], Token::Status(BuildStatus::Queued));
        assert_eq!(tokens[5], Token::Status(BuildStatus::Running));
        assert_eq!(tokens[6], Token::Status(BuildStatus::Skipped));
    }

    #[test]
    fn should_recognize_operators() {
        // Given, When
        let tokens = tokenize("== != > >= < <=").unwrap().get_tokens();

        // Then
        assert_eq!(6, tokens.len());
        assert_eq!(tokens[0], Token::Operator(Operator::EqualTo));
        assert_eq!(tokens[1], Token::Operator(Operator::NotEqualTo));
        assert_eq!(tokens[2], Token::Operator(Operator::GreaterThan));
        assert_eq!(tokens[3], Token::Operator(Operator::GreaterThanOrEqualTo));
        assert_eq!(tokens[4], Token::Operator(Operator::LessThan));
        assert_eq!(tokens[5], Token::Operator(Operator::LessThanOrEqualTo));
    }

    #[test]
    fn should_tokenize_expression_correctly() {
        // Given, When
        let tokens = tokenize("branch == 'master' and status != 'skipped'")
            .unwrap()
            .get_tokens();

        // Then
        assert_eq!(7, tokens.len());
        assert_eq!(tokens[0], Token::Word("branch".to_owned()));
        assert_eq!(tokens[1], Token::Operator(Operator::EqualTo));
        assert_eq!(tokens[2], Token::Literal("master".to_owned()));
        assert_eq!(tokens[3], Token::And);
        assert_eq!(tokens[4], Token::Word("status".to_owned()));
        assert_eq!(tokens[5], Token::Operator(Operator::NotEqualTo));
        assert_eq!(tokens[6], Token::Literal("skipped".to_owned()));
    }
}
