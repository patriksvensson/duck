use crate::query::lexer::{Token, TokenStream};
use crate::query::{Constant, Expression, Property};
use crate::DuckResult;

pub fn parse(stream: &mut TokenStream) -> DuckResult<Expression> {
    parse_or(stream)
}

fn parse_or(stream: &mut TokenStream) -> DuckResult<Expression> {
    if stream.current().is_none() {
        return Err(format_err!("Unexpected end of token stream"));
    }

    let mut expression = parse_and(stream)?;
    while let Some(token) = stream.current() {
        if token == &Token::Or {
            stream.move_next();
            expression = Expression::Or(Box::new(expression), Box::new(parse_and(stream)?));
        } else {
            break;
        }
    }

    Ok(expression)
}

fn parse_and(stream: &mut TokenStream) -> DuckResult<Expression> {
    if stream.current().is_none() {
        return Err(format_err!("Unexpected end of token stream"));
    }

    let mut expression = parse_predicate(stream)?;
    while let Some(token) = stream.current() {
        if token == &Token::And {
            stream.move_next();
            expression = Expression::And(Box::new(expression), Box::new(parse_predicate(stream)?));
        } else {
            break;
        }
    }

    Ok(expression)
}

fn parse_predicate(stream: &mut TokenStream) -> DuckResult<Expression> {
    if stream.current().is_none() {
        return Err(format_err!("Unexpected end of token stream"));
    }

    if let Some(token) = stream.current() {
        if token == &Token::Not {
            stream.move_next();
            return Ok(Expression::Not(Box::new(parse_predicate(stream)?)));
        }
    }

    Ok(parse_relation(stream)?)
}

fn parse_relation(stream: &mut TokenStream) -> DuckResult<Expression> {
    if stream.current().is_none() {
        return Err(format_err!("Unexpected end of token stream"));
    }

    let expression = parse_literal(stream)?;
    stream.move_next();

    if let Some(token) = stream.current() {
        if let Some(op) = token.get_operator() {
            stream.move_next();

            let left = expression;
            let right = parse_literal(stream)?;
            stream.move_next();

            return Ok(Expression::Relational(Box::new(left), Box::new(right), op));
        }
    }

    Ok(expression)
}

fn parse_literal(stream: &mut TokenStream) -> DuckResult<Expression> {
    match stream.current() {
        None => Err(format_err!("Unexpected end of token stream")),
        Some(token) => match token {
            Token::Word(word) => match &word[..] {
                "branch" => Ok(Expression::Property(Property::Branch)),
                "status" => Ok(Expression::Property(Property::Status)),
                "project" => Ok(Expression::Property(Property::Project)),
                "definition" => Ok(Expression::Property(Property::Definition)),
                "build" => Ok(Expression::Property(Property::Build)),
                "collector" => Ok(Expression::Property(Property::Collector)),
                "provider" => Ok(Expression::Property(Property::Provider)),
                _ => Err(format_err!("Unknown property '{}'", word)),
            },
            Token::Literal(literal) => Ok(Expression::Constant(Constant::Text(literal.clone()))),
            Token::Integer(number) => Ok(Expression::Constant(Constant::Number(*number))),
            Token::Status(status) => Ok(Expression::Constant(Constant::Status(status.clone()))),
            Token::True => Ok(Expression::Constant(Constant::Boolean(true))),
            Token::False => Ok(Expression::Constant(Constant::Boolean(false))),
            _ => Err(format_err!("Could not parse literal expression")),
        },
    }
}

///////////////////////////////////////////////////////////
// Tests

#[cfg(test)]
mod tests {
    use super::*;
    use crate::builds::BuildStatus;
    use crate::query::lexer;
    use crate::query::Operator;

    #[test]
    fn should_parse_expression() {
        // Given
        let query = "branch == 'master' and status != skipped";
        let tokens = &mut lexer::tokenize(&query[..]).unwrap();

        // When
        let expression = parse(tokens).unwrap();

        // Then
        assert_eq!(
            expression,
            Expression::And(
                Box::new(Expression::Relational(
                    Box::new(Expression::Property(Property::Branch)),
                    Box::new(Expression::Constant(Constant::Text("master".to_owned()))),
                    Operator::EqualTo
                )),
                Box::new(Expression::Relational(
                    Box::new(Expression::Property(Property::Status)),
                    Box::new(Expression::Constant(Constant::Status(BuildStatus::Skipped))),
                    Operator::NotEqualTo
                ))
            )
        )
    }
}
