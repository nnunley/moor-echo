#[cfg(test)]
mod tests {
    use crate::parser::lambdamoo_db_parser::{
        LambdaMooDbParser, Rule, LambdaMooValue, PropertyValue,
        TYPE_INT, TYPE_OBJ, TYPE_STR, TYPE_ERR, TYPE_LIST,
        TYPE_CLEAR, TYPE_NONE, TYPE_CATCH, TYPE_FINALLY, TYPE_FLOAT
    };
    use pest::Parser;

    #[test]
    fn test_type_constants() {
        // Verify type constants match LambdaMOO structures.h
        assert_eq!(TYPE_INT, 0);
        assert_eq!(TYPE_OBJ, 1);
        assert_eq!(TYPE_STR, 2);
        assert_eq!(TYPE_ERR, 3);
        assert_eq!(TYPE_LIST, 4);
        assert_eq!(TYPE_CLEAR, 5);
        assert_eq!(TYPE_NONE, 6);
        assert_eq!(TYPE_CATCH, 7);
        assert_eq!(TYPE_FINALLY, 8);
        assert_eq!(TYPE_FLOAT, 9);
    }

    #[test]
    fn test_parse_simple_integer_value() {
        let input = "0\n42\n";
        let result = LambdaMooDbParser::parse(Rule::value, input);
        assert!(result.is_ok());
        
        let mut pairs = result.unwrap();
        let pair = pairs.next().unwrap();
        let value = LambdaMooDbParser::parse_value(pair).unwrap();
        
        match value {
            LambdaMooValue::Int(n) => assert_eq!(n, 42),
            _ => panic!("Expected Int value"),
        }
    }

    #[test]
    fn test_parse_string_value() {
        let input = "2\nhello world\n";
        let result = LambdaMooDbParser::parse(Rule::value, input);
        assert!(result.is_ok());
        
        let mut pairs = result.unwrap();
        let pair = pairs.next().unwrap();
        let value = LambdaMooDbParser::parse_value(pair).unwrap();
        
        match value {
            LambdaMooValue::Str(s) => assert_eq!(s, "hello world"),
            _ => panic!("Expected Str value"),
        }
    }

    #[test]
    fn test_parse_object_value() {
        let input = "1\n42\n";
        let result = LambdaMooDbParser::parse(Rule::value, input);
        assert!(result.is_ok());
        
        let mut pairs = result.unwrap();
        let pair = pairs.next().unwrap();
        let value = LambdaMooDbParser::parse_value(pair).unwrap();
        
        match value {
            LambdaMooValue::Obj(id) => assert_eq!(id, 42),
            _ => panic!("Expected Obj value"),
        }
    }

    #[test]
    fn test_parse_list_with_integers() {
        // List with 2 integers: [42, 99]
        let input = "4\n2\n0\n42\n0\n99\n";
        let result = LambdaMooDbParser::parse(Rule::value, input);
        assert!(result.is_ok());
        
        let mut pairs = result.unwrap();
        let pair = pairs.next().unwrap();
        let value = LambdaMooDbParser::parse_value(pair).unwrap();
        
        match value {
            LambdaMooValue::List(list) => {
                assert_eq!(list.len(), 2);
                match &list[0] {
                    LambdaMooValue::Int(n) => assert_eq!(*n, 42),
                    _ => panic!("Expected Int in list"),
                }
                match &list[1] {
                    LambdaMooValue::Int(n) => assert_eq!(*n, 99),
                    _ => panic!("Expected Int in list"),
                }
            }
            _ => panic!("Expected List value"),
        }
    }

    #[test]
    fn test_parse_list_with_strings() {
        // List with 2 strings: ["generics", "Generic objects"]
        let input = "4\n2\n2\ngenerics\n2\nGeneric objects\n";
        let result = LambdaMooDbParser::parse(Rule::value, input);
        assert!(result.is_ok());
        
        let mut pairs = result.unwrap();
        let pair = pairs.next().unwrap();
        let value = LambdaMooDbParser::parse_value(pair).unwrap();
        
        match value {
            LambdaMooValue::List(list) => {
                assert_eq!(list.len(), 2);
                match &list[0] {
                    LambdaMooValue::Str(s) => assert_eq!(s, "generics"),
                    _ => panic!("Expected Str in list"),
                }
                match &list[1] {
                    LambdaMooValue::Str(s) => assert_eq!(s, "Generic objects"),
                    _ => panic!("Expected Str in list"),
                }
            }
            _ => panic!("Expected List value"),
        }
    }

    #[test]
    fn test_parse_nested_list() {
        // List with 2 elements: [42, [1, 2]]
        let input = "4\n2\n0\n42\n4\n2\n0\n1\n0\n2\n";
        let result = LambdaMooDbParser::parse(Rule::value, input);
        assert!(result.is_ok());
        
        let mut pairs = result.unwrap();
        let pair = pairs.next().unwrap();
        let value = LambdaMooDbParser::parse_value(pair).unwrap();
        
        match value {
            LambdaMooValue::List(list) => {
                assert_eq!(list.len(), 2);
                match &list[0] {
                    LambdaMooValue::Int(n) => assert_eq!(*n, 42),
                    _ => panic!("Expected Int in list"),
                }
                match &list[1] {
                    LambdaMooValue::List(inner) => {
                        assert_eq!(inner.len(), 2);
                        match &inner[0] {
                            LambdaMooValue::Int(n) => assert_eq!(*n, 1),
                            _ => panic!("Expected Int in nested list"),
                        }
                        match &inner[1] {
                            LambdaMooValue::Int(n) => assert_eq!(*n, 2),
                            _ => panic!("Expected Int in nested list"),
                        }
                    }
                    _ => panic!("Expected List in list"),
                }
            }
            _ => panic!("Expected List value"),
        }
    }

    #[test]
    fn test_parse_property_value_with_simple_type() {
        // Property value: value=42, owner=2, perms=5
        let input = "0\n42\n2\n5\n";
        let result = LambdaMooDbParser::parse(Rule::propval, input);
        assert!(result.is_ok());
        
        let mut pairs = result.unwrap();
        let pair = pairs.next().unwrap();
        let propval = LambdaMooDbParser::parse_propval(pair).unwrap();
        
        match propval.value {
            LambdaMooValue::Int(n) => assert_eq!(n, 42),
            _ => panic!("Expected Int value"),
        }
        assert_eq!(propval.owner, 2);
        assert_eq!(propval.perms, 5);
    }

    #[test]
    fn test_parse_property_value_with_list() {
        // Property value with list: value=[2, 3], owner=4, perms=3
        let input = "4\n2\n0\n2\n0\n3\n4\n3\n";
        let result = LambdaMooDbParser::parse(Rule::propval, input);
        assert!(result.is_ok());
        
        let mut pairs = result.unwrap();
        let pair = pairs.next().unwrap();
        let propval = LambdaMooDbParser::parse_propval(pair).unwrap();
        
        match &propval.value {
            LambdaMooValue::List(list) => {
                assert_eq!(list.len(), 2);
                match &list[0] {
                    LambdaMooValue::Int(n) => assert_eq!(*n, 2),
                    _ => panic!("Expected Int in list"),
                }
                match &list[1] {
                    LambdaMooValue::Int(n) => assert_eq!(*n, 3),
                    _ => panic!("Expected Int in list"),
                }
            }
            _ => panic!("Expected List value"),
        }
        assert_eq!(propval.owner, 4);
        assert_eq!(propval.perms, 3);
    }

    #[test]
    fn test_parse_empty_list() {
        // Empty list: []
        let input = "4\n0\n";
        let result = LambdaMooDbParser::parse(Rule::value, input);
        assert!(result.is_ok());
        
        let mut pairs = result.unwrap();
        let pair = pairs.next().unwrap();
        let value = LambdaMooDbParser::parse_value(pair).unwrap();
        
        match value {
            LambdaMooValue::List(list) => {
                assert_eq!(list.len(), 0);
            }
            _ => panic!("Expected List value"),
        }
    }

    #[test]
    fn test_parse_property_values_section_with_simple_values() {
        // Property values section with 2 property values
        let input = "2\n0\n42\n2\n5\n1\n99\n3\n7\n";
        let result = LambdaMooDbParser::parse(Rule::property_values, input);
        assert!(result.is_ok());
        
        let mut pairs = result.unwrap();
        let pair = pairs.next().unwrap();
        let propvals = LambdaMooDbParser::parse_property_values(pair).unwrap();
        
        assert_eq!(propvals.len(), 2);
        
        // First property value
        match &propvals[0].value {
            LambdaMooValue::Int(n) => assert_eq!(*n, 42),
            _ => panic!("Expected Int value"),
        }
        assert_eq!(propvals[0].owner, 2);
        assert_eq!(propvals[0].perms, 5);
        
        // Second property value
        match &propvals[1].value {
            LambdaMooValue::Obj(id) => assert_eq!(*id, 99),
            _ => panic!("Expected Obj value"),
        }
        assert_eq!(propvals[1].owner, 3);
        assert_eq!(propvals[1].perms, 7);
    }

    #[test]
    fn test_parse_property_values_section_with_list() {
        // Property values section with 1 property that contains a list
        // The list property: ["hello", "world"], owner=2, perms=5
        let input = "1\n4\n2\n2\nhello\n2\nworld\n2\n5\n";
        let result = LambdaMooDbParser::parse(Rule::property_values, input);
        assert!(result.is_ok());
        
        let mut pairs = result.unwrap();
        let pair = pairs.next().unwrap();
        let propvals = LambdaMooDbParser::parse_property_values(pair).unwrap();
        
        assert_eq!(propvals.len(), 1);
        
        match &propvals[0].value {
            LambdaMooValue::List(list) => {
                assert_eq!(list.len(), 2);
                match &list[0] {
                    LambdaMooValue::Str(s) => assert_eq!(s, "hello"),
                    _ => panic!("Expected Str in list"),
                }
                match &list[1] {
                    LambdaMooValue::Str(s) => assert_eq!(s, "world"),
                    _ => panic!("Expected Str in list"),
                }
            }
            _ => panic!("Expected List value"),
        }
        assert_eq!(propvals[0].owner, 2);
        assert_eq!(propvals[0].perms, 5);
    }

    #[test]
    fn test_lambdacore_property_value_36() {
        // This is the exact property value #36 from LambdaCore that was causing issues
        // It's a list with 2 strings: ["generics", "Generic objects intended for use as the parents of new objects"]
        let input = "4\n2\n2\ngenerics\n2\nGeneric objects intended for use as the parents of new objects\n4\n3\n";
        let result = LambdaMooDbParser::parse(Rule::propval, input);
        assert!(result.is_ok());
        
        let mut pairs = result.unwrap();
        let pair = pairs.next().unwrap();
        let propval = LambdaMooDbParser::parse_propval(pair).unwrap();
        
        match &propval.value {
            LambdaMooValue::List(list) => {
                assert_eq!(list.len(), 2);
                match &list[0] {
                    LambdaMooValue::Str(s) => assert_eq!(s, "generics"),
                    _ => panic!("Expected Str in list"),
                }
                match &list[1] {
                    LambdaMooValue::Str(s) => assert_eq!(s, "Generic objects intended for use as the parents of new objects"),
                    _ => panic!("Expected Str in list"),
                }
            }
            _ => panic!("Expected List value"),
        }
        assert_eq!(propval.owner, 4);
        assert_eq!(propval.perms, 3);
    }
}