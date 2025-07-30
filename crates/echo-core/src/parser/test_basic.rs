#[cfg(test)]
mod tests {
    use crate::parser::simple_parser::parse_simple;

    #[test]
    fn test_simple_parser_works() {
        // Test that simple parser works
        let result = parse_simple("42");
        assert!(result.is_ok());
        
        let result = parse_simple("let x = 10;");
        assert!(result.is_ok());
    }
}