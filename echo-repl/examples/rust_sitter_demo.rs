use rust_sitter::sitter;

#[sitter(grammar = "expression")]
#[derive(Debug, PartialEq)]
pub enum Expression {
    #[sitter(pattern = r"\d+", transform = |v| v.parse().unwrap())]
    Number(u64),
    #[sitter(prec_left = 1, non_assoc)]
    Add(Box<Expression>, #[sitter(leaf)] char, Box<Expression>),
    #[sitter(prec_left = 2, non_assoc)]
    Mul(Box<Expression>, #[sitter(leaf)] char, Box<Expression>),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_expression_parser() {
        let input = "1 + 2 * 3";
        let expected = Expression::Add(
            Box::new(Expression::Number(1)),
            '+',
            Box::new(Expression::Mul(
                Box::new(Expression::Number(2)),
                '*',
                Box::new(Expression::Number(3)),
            )),
        );

        let result = rust_sitter::parse::<Expression>(input).unwrap();

        assert_eq!(result, expected);
    }
}
