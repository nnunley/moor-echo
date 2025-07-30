// Tests for verb execution from stored objects

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use tempfile::tempdir;

    use crate::{
        evaluator::{create_evaluator, Value},
        parser::create_parser,
        storage::Storage,
    };

    // Helper function to parse Echo code
    fn parse_echo(code: &str) -> anyhow::Result<crate::ast::EchoAst> {
        let mut parser = create_parser("echo")?;
        parser.parse(code)
    }

    #[test]
    fn test_simple_verb_execution() {
        let temp_dir = tempdir().expect("Failed to create temp directory");
        let storage = Arc::new(Storage::new(temp_dir.path()).expect("Failed to create storage"));
        let mut evaluator = create_evaluator(storage).expect("Failed to create evaluator");

        // Create a player
        let player_id = evaluator
            .create_player("test_player")
            .expect("Failed to create player");
        evaluator
            .switch_player(player_id)
            .expect("Failed to switch player");

        // Define an object with a verb
        let code = r#"
            object greeter
                verb greet {}
                    return "Hello, World!"
                endverb
            endobject
        "#;

        let ast = parse_echo(code).expect("Failed to parse");
        let result = evaluator.eval(&ast).expect("Failed to evaluate");

        // The object should be created
        if let Value::Object(_obj_id) = result {
            // Call the verb
            let call_code = "#0.greeter:greet()";
            let call_ast = parse_echo(call_code).expect("Failed to parse method call");
            let greeting = evaluator.eval(&call_ast).expect("Failed to call verb");

            assert_eq!(greeting, Value::String("Hello, World!".to_string()));
        } else {
            panic!("Object creation failed");
        }
    }

    #[test]
    fn test_verb_with_multiple_params() {
        let temp_dir = tempdir().expect("Failed to create temp directory");
        let storage = Arc::new(Storage::new(temp_dir.path()).expect("Failed to create storage"));
        let mut evaluator = create_evaluator(storage).expect("Failed to create evaluator");

        // Create a player
        let player_id = evaluator
            .create_player("test_player")
            .expect("Failed to create player");
        evaluator
            .switch_player(player_id)
            .expect("Failed to switch player");

        // Define an object with a verb that takes multiple parameters
        let code = r#"
            object calculator
                verb add {a, b, c}
                    return a + b + c
                endverb
            endobject
        "#;

        let ast = parse_echo(code).expect("Failed to parse");
        evaluator.eval(&ast).expect("Failed to evaluate");

        // Call the verb
        let call_code = "#0.calculator:add(10, 20, 30)";
        let call_ast = parse_echo(call_code).expect("Failed to parse method call");
        let result = evaluator.eval(&call_ast).expect("Failed to call verb");

        assert_eq!(result, Value::Integer(60));
    }

    #[test]
    fn test_verb_with_this_reference() {
        let temp_dir = tempdir().expect("Failed to create temp directory");
        let storage = Arc::new(Storage::new(temp_dir.path()).expect("Failed to create storage"));
        let mut evaluator = create_evaluator(storage).expect("Failed to create evaluator");

        // Create a player
        let player_id = evaluator
            .create_player("test_player")
            .expect("Failed to create player");
        evaluator
            .switch_player(player_id)
            .expect("Failed to switch player");

        // Define an object with a property and a verb that accesses it
        let code = r#"
            object counter
                property count = 42
                
                verb get_count {}
                    return this.count
                endverb
                
                verb increment {}
                    this.count = this.count + 1
                    return this.count
                endverb
            endobject
        "#;

        let ast = parse_echo(code).expect("Failed to parse");
        evaluator.eval(&ast).expect("Failed to evaluate");

        // Call get_count
        let call_code = "#0.counter:get_count()";
        let call_ast = parse_echo(call_code).expect("Failed to parse method call");
        let count = evaluator.eval(&call_ast).expect("Failed to call verb");
        assert_eq!(count, Value::Integer(42));

        // Call increment
        let inc_code = "#0.counter:increment()";
        let inc_ast = parse_echo(inc_code).expect("Failed to parse method call");
        let new_count = evaluator.eval(&inc_ast).expect("Failed to call verb");
        assert_eq!(new_count, Value::Integer(43));

        // Verify the property was updated
        let verify_code = "#0.counter.count";
        let verify_ast = parse_echo(verify_code).expect("Failed to parse property access");
        let final_count = evaluator
            .eval(&verify_ast)
            .expect("Failed to access property");
        assert_eq!(final_count, Value::Integer(43));
    }

    #[test]
    fn test_verb_with_default_parameters() {
        let temp_dir = tempdir().expect("Failed to create temp directory");
        let storage = Arc::new(Storage::new(temp_dir.path()).expect("Failed to create storage"));
        let mut evaluator = create_evaluator(storage).expect("Failed to create evaluator");

        // Create a player
        let player_id = evaluator
            .create_player("test_player")
            .expect("Failed to create player");
        evaluator
            .switch_player(player_id)
            .expect("Failed to switch player");

        // Define an object with a verb that has default parameters
        let code = r#"
            object formatter
                verb format {text, ?prefix = ">>> ", ?suffix = " <<<"}
                    return prefix + text + suffix
                endverb
            endobject
        "#;

        let ast = parse_echo(code).expect("Failed to parse");
        evaluator.eval(&ast).expect("Failed to evaluate");

        // Call with all parameters
        let call1_code = r#"#0.formatter:format("Hello", "[", "]")"#;
        let call1_ast = parse_echo(call1_code).expect("Failed to parse");
        let result1 = evaluator.eval(&call1_ast).expect("Failed to call verb");
        assert_eq!(result1, Value::String("[Hello]".to_string()));

        // Call with only required parameter (uses defaults)
        let call2_code = r#"#0.formatter:format("World")"#;
        let call2_ast = parse_echo(call2_code).expect("Failed to parse");
        let result2 = evaluator.eval(&call2_ast).expect("Failed to call verb");
        assert_eq!(result2, Value::String(">>> World <<<".to_string()));
    }

    #[test]
    fn test_verb_execution_error() {
        let temp_dir = tempdir().expect("Failed to create temp directory");
        let storage = Arc::new(Storage::new(temp_dir.path()).expect("Failed to create storage"));
        let mut evaluator = create_evaluator(storage).expect("Failed to create evaluator");

        // Create a player
        let player_id = evaluator
            .create_player("test_player")
            .expect("Failed to create player");
        evaluator
            .switch_player(player_id)
            .expect("Failed to switch player");

        // Define an object with a verb
        let code = r#"
            object test_obj
                verb needs_args {x, y}
                    return x + y
                endverb
            endobject
        "#;

        let ast = parse_echo(code).expect("Failed to parse");
        evaluator.eval(&ast).expect("Failed to evaluate");

        // Try to call with missing arguments
        let call_code = "#0.test_obj:needs_args(5)";
        let call_ast = parse_echo(call_code).expect("Failed to parse method call");
        let result = evaluator.eval(&call_ast);

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Missing required argument"));
    }
}
