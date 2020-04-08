use crate::builds::Build;
use crate::query::{Constant, Expression, Operator, Property, Visitor};
use crate::DuckResult;

pub struct BuildEvaluator {}

impl Visitor<Build, Constant> for BuildEvaluator {
    fn or(&self, ctx: &Build, left: &Expression, right: &Expression) -> DuckResult<Constant> {
        let left = left.accept(ctx, self)?;
        let right = right.accept(ctx, self)?;
        match (left, right) {
            (Constant::Boolean(lhs), Constant::Boolean(rhs)) => Ok(Constant::Boolean(lhs || rhs)),
            _ => panic!("Cannot compare OR expression"),
        }
    }

    fn and(&self, ctx: &Build, left: &Expression, right: &Expression) -> DuckResult<Constant> {
        let left = left.accept(ctx, self)?;
        let right = right.accept(ctx, self)?;
        match (left, right) {
            (Constant::Boolean(lhs), Constant::Boolean(rhs)) => Ok(Constant::Boolean(lhs && rhs)),
            _ => panic!("Cannot compare AND expression"),
        }
    }

    fn not(&self, ctx: &Build, exp: &Expression) -> DuckResult<Constant> {
        match exp.accept(ctx, self)? {
            Constant::Boolean(e) => Ok(Constant::Boolean(!e)),
            _ => panic!("Can't negate type"),
        }
    }

    fn constant(&self, _ctx: &Build, constant: &Constant) -> DuckResult<Constant> {
        match constant {
            Constant::Boolean(b) => Ok(Constant::Boolean(*b)),
            Constant::Number(n) => Ok(Constant::Number(*n)),
            Constant::Text(t) => Ok(Constant::Text(t.clone())),
            Constant::Status(s) => Ok(Constant::Status(s.clone())),
        }
    }

    fn property(&self, ctx: &Build, property: &Property) -> DuckResult<Constant> {
        Ok(match property {
            Property::Branch => Constant::Text(ctx.branch.clone()),
            Property::Status => Constant::Status(ctx.status.clone()),
            Property::Definition => Constant::Text(ctx.definition_id.clone()),
            Property::Project => Constant::Text(ctx.project_id.clone()),
            Property::Build => Constant::Text(ctx.build_id.clone()),
            Property::Collector => Constant::Text(ctx.collector.clone()),
            Property::Provider => Constant::Text(ctx.provider.clone()),
        })
    }

    fn relational(
        &self,
        ctx: &Build,
        left: &Expression,
        right: &Expression,
        operator: &Operator,
    ) -> DuckResult<Constant> {
        let left = left.accept(ctx, self)?;
        let right = right.accept(ctx, self)?;

        match operator {
            Operator::EqualTo => match (left, right) {
                (Constant::Number(lhs), Constant::Number(rhs)) => Ok(Constant::Boolean(lhs == rhs)),
                (Constant::Boolean(lhs), Constant::Boolean(rhs)) => {
                    Ok(Constant::Boolean(lhs == rhs))
                }
                (Constant::Text(lhs), Constant::Text(rhs)) => Ok(Constant::Boolean(lhs == rhs)),
                (Constant::Status(lhs), Constant::Status(rhs)) => Ok(Constant::Boolean(lhs == rhs)),
                _ => panic!("Can't compare types"),
            },
            Operator::NotEqualTo => match (left, right) {
                (Constant::Number(lhs), Constant::Number(rhs)) => Ok(Constant::Boolean(lhs != rhs)),
                (Constant::Boolean(lhs), Constant::Boolean(rhs)) => {
                    Ok(Constant::Boolean(lhs != rhs))
                }
                (Constant::Text(lhs), Constant::Text(rhs)) => Ok(Constant::Boolean(lhs != rhs)),
                _ => panic!("Can't compare types"),
            },
            Operator::GreaterThan => match (left, right) {
                (Constant::Number(lhs), Constant::Number(rhs)) => Ok(Constant::Boolean(lhs > rhs)),
                _ => panic!("Can't compare types"),
            },
            Operator::GreaterThanOrEqualTo => match (left, right) {
                (Constant::Number(lhs), Constant::Number(rhs)) => Ok(Constant::Boolean(lhs >= rhs)),
                _ => panic!("Can't compare types"),
            },
            Operator::LessThan => match (left, right) {
                (Constant::Number(lhs), Constant::Number(rhs)) => Ok(Constant::Boolean(lhs < rhs)),
                _ => panic!("Can't compare types"),
            },
            Operator::LessThanOrEqualTo => match (left, right) {
                (Constant::Number(lhs), Constant::Number(rhs)) => Ok(Constant::Boolean(lhs <= rhs)),
                _ => panic!("Can't compare types"),
            },
        }
    }
}

///////////////////////////////////////////////////////////
// Tests

#[cfg(test)]
mod tests {
    use super::*;
    use crate::builds::{BuildBuilder, BuildStatus};
    use crate::query;
    use test_case::test_case;

    #[test_case("3 == 3", Constant::Boolean(true) ; "integer_equal_to_1")]
    #[test_case("3 == 2", Constant::Boolean(false) ; "integer_equal_to_2")]
    #[test_case("3 != 2", Constant::Boolean(true) ; "integer_not_equal_to_1")]
    #[test_case("3 != 3", Constant::Boolean(false) ; "integer_not_equal_to_2")]
    #[test_case("3 > 2", Constant::Boolean(true) ; "integer_greater_than_1")]
    #[test_case("3 > 3", Constant::Boolean(false) ; "integer_greater_than_2")]
    #[test_case("3 >= 2", Constant::Boolean(true) ; "integer_greater_than_or_equal_to_1")]
    #[test_case("3 >= 3", Constant::Boolean(true) ; "integer_greater_than_or_equal_to_2")]
    #[test_case("3 >= 4", Constant::Boolean(false) ; "integer_greater_than_or_equal_to_3")]
    #[test_case("2 < 3", Constant::Boolean(true) ; "integer_less_than_1")]
    #[test_case("3 < 3", Constant::Boolean(false) ; "integer_less_than_2")]
    #[test_case("2 <= 3", Constant::Boolean(true) ; "integer_less_than_or_equal_to_1")]
    #[test_case("2 <= 2", Constant::Boolean(true) ; "integer_less_than_or_equal_to_2")]
    #[test_case("2 <= 1", Constant::Boolean(false) ; "integer_less_than_or_equal_to_3")]
    #[test_case("!true", Constant::Boolean(false) ; "negated_true_1")]
    #[test_case("NOT true", Constant::Boolean(false) ; "negated_true_2")]
    #[test_case("!false", Constant::Boolean(true) ; "negated_false_1")]
    #[test_case("NOT false", Constant::Boolean(true) ; "negated_false_2")]
    #[test_case("true and true", Constant::Boolean(true) ; "and_1")]
    #[test_case("true and false", Constant::Boolean(false) ; "and_2")]
    #[test_case("true or true", Constant::Boolean(true) ; "or_1")]
    #[test_case("true or false", Constant::Boolean(true) ; "or_2")]
    #[test_case("false or false", Constant::Boolean(false) ; "or_3")]
    #[test_case("false or true", Constant::Boolean(true) ; "or_4")]
    fn should_evaluate_expression(expression: &str, expected: Constant) {
        // Given
        let build = BuildBuilder::dummy().build().unwrap();
        let evaluator = BuildEvaluator {};
        let expression = query::parse(expression).unwrap();

        // When
        let result = expression.accept(&build, &evaluator).unwrap();

        // Then
        assert_eq!(expected, result);
    }

    #[test_case("branch == 'develop'", Constant::Boolean(true))]
    #[test_case("status == queued", Constant::Boolean(true))]
    #[test_case("project == 'foo'", Constant::Boolean(true))]
    #[test_case("definition == 'bar'", Constant::Boolean(true))]
    #[test_case("build == '123'", Constant::Boolean(true))]
    #[test_case("collector == 'test'", Constant::Boolean(true))]
    #[test_case("provider == 'TeamCity'", Constant::Boolean(true))]
    fn should_evaluate_expression_with_property(expression: &str, expected: Constant) {
        // Given
        let evaluator = BuildEvaluator {};
        let expression = query::parse(expression).unwrap();
        let build = BuildBuilder::dummy()
            .branch("develop")
            .collector("test")
            .provider("TeamCity")
            .status(BuildStatus::Queued)
            .project_id("foo")
            .definition_id("bar")
            .build_id("123")
            .build()
            .unwrap();

        // When
        let result = expression.accept(&build, &evaluator).unwrap();

        // Then
        assert_eq!(expected, result);
    }
}
