/// Verb parsing utilities for MOO-style verb name matching
/// 
/// Supports:
/// - Multiple verb names separated by spaces (e.g., "l look")
/// - Pattern matching with * (e.g., "foo*bar", "pronoun_*")
/// - Exact and prefix matching

/// Parse a verb name specification into individual verb names/patterns
pub fn parse_verb_names(verb_spec: &str) -> Vec<String> {
    verb_spec.split_whitespace()
        .map(|s| s.to_string())
        .collect()
}

/// Check if a command matches a verb name pattern
/// 
/// Rules:
/// - Exact match always wins
/// - "foo*bar" matches "foo", "foob", "fooba", "foobar"
/// - "foo*" matches any string starting with "foo"
/// - "*" matches anything
pub fn matches_verb_pattern(pattern: &str, command: &str) -> bool {
    // Exact match
    if pattern == command {
        return true;
    }
    
    // Handle patterns with *
    if pattern.contains('*') {
        match_star_pattern(pattern, command)
    } else {
        false
    }
}

/// Match a pattern containing * against a command
fn match_star_pattern(pattern: &str, command: &str) -> bool {
    // Special case: "*" matches everything
    if pattern == "*" {
        return true;
    }
    
    // Find the position of the star
    if let Some(star_pos) = pattern.find('*') {
        let prefix = &pattern[..star_pos];
        let suffix = &pattern[star_pos + 1..];
        
        // Command must start with prefix
        if !command.starts_with(prefix) {
            return false;
        }
        
        // If there's no suffix, we just need the prefix match
        if suffix.is_empty() {
            return true;
        }
        
        // For patterns like "foo*bar", we need to check if command
        // is between "foo" and "foobar" in length and content
        let min_len = prefix.len();
        let max_len = prefix.len() + suffix.len();
        
        if command.len() < min_len || command.len() > max_len {
            return false;
        }
        
        // Check if the command could be formed by the pattern
        // For "foo*bar" pattern:
        // - "foo" matches (no suffix needed)
        // - "foob" matches (partial suffix)
        // - "fooba" matches (partial suffix)
        // - "foobar" matches (full suffix)
        if command.len() == prefix.len() {
            // Just the prefix, no suffix
            return true;
        }
        
        // Check if the remaining part of command is a prefix of suffix
        let remaining = &command[prefix.len()..];
        suffix.starts_with(remaining)
    } else {
        false
    }
}

/// Find the best matching verb from a list of verb definitions
/// Returns the matching verb name pattern and the specific name that matched
pub fn find_matching_verb<'a>(
    verb_defs: &'a [(String, Vec<String>)], // (original_spec, parsed_names)
    command: &str,
) -> Option<(&'a str, &'a str)> {
    let mut best_match: Option<(&'a str, &'a str)> = None;
    let mut best_score = 0;
    
    for (original_spec, patterns) in verb_defs {
        for pattern in patterns {
            if matches_verb_pattern(pattern, command) {
                let score = score_match(pattern, command);
                if score > best_score {
                    best_score = score;
                    best_match = Some((original_spec.as_str(), pattern.as_str()));
                }
            }
        }
    }
    
    best_match
}

/// Score a match - higher scores are better matches
/// Exact matches get highest score, then shorter patterns
fn score_match(pattern: &str, command: &str) -> usize {
    if pattern == command {
        // Exact match is best
        1000
    } else if !pattern.contains('*') {
        // Non-pattern exact match (shouldn't happen)
        900
    } else if pattern == "*" {
        // Wildcard match is lowest priority
        1
    } else {
        // Score based on how specific the pattern is
        // Longer non-* parts score higher
        let non_star_len = pattern.chars().filter(|&c| c != '*').count();
        non_star_len + 10
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_verb_names() {
        assert_eq!(parse_verb_names("look"), vec!["look"]);
        assert_eq!(parse_verb_names("l look"), vec!["l", "look"]);
        assert_eq!(parse_verb_names("get take"), vec!["get", "take"]);
        assert_eq!(parse_verb_names("pronoun_*"), vec!["pronoun_*"]);
    }

    #[test]
    fn test_exact_match() {
        assert!(matches_verb_pattern("look", "look"));
        assert!(!matches_verb_pattern("look", "loo"));
        assert!(!matches_verb_pattern("look", "looks"));
    }

    #[test]
    fn test_star_pattern() {
        // Test "foo*bar" pattern
        assert!(matches_verb_pattern("foo*bar", "foo"));
        assert!(matches_verb_pattern("foo*bar", "foob"));
        assert!(matches_verb_pattern("foo*bar", "fooba"));
        assert!(matches_verb_pattern("foo*bar", "foobar"));
        assert!(!matches_verb_pattern("foo*bar", "f"));
        assert!(!matches_verb_pattern("foo*bar", "foobars"));
        assert!(!matches_verb_pattern("foo*bar", "foobarx"));
        
        // Test "foo*" pattern
        assert!(matches_verb_pattern("foo*", "foo"));
        assert!(matches_verb_pattern("foo*", "foobar"));
        assert!(matches_verb_pattern("foo*", "foo123"));
        assert!(!matches_verb_pattern("foo*", "fo"));
        
        // Test "*" pattern
        assert!(matches_verb_pattern("*", "anything"));
        assert!(matches_verb_pattern("*", ""));
        
        // Test "pronoun_*" pattern
        assert!(matches_verb_pattern("pronoun_*", "pronoun_"));
        assert!(matches_verb_pattern("pronoun_*", "pronoun_sub"));
        assert!(matches_verb_pattern("pronoun_*", "pronoun_obj"));
        assert!(!matches_verb_pattern("pronoun_*", "pronoun"));
    }

    #[test]
    fn test_find_matching_verb() {
        let verbs = vec![
            ("l look".to_string(), vec!["l".to_string(), "look".to_string()]),
            ("get take".to_string(), vec!["get".to_string(), "take".to_string()]),
            ("foo*bar".to_string(), vec!["foo*bar".to_string()]),
            ("*".to_string(), vec!["*".to_string()]),
        ];
        
        // Exact matches
        assert_eq!(find_matching_verb(&verbs, "look"), Some(("l look", "look")));
        assert_eq!(find_matching_verb(&verbs, "l"), Some(("l look", "l")));
        assert_eq!(find_matching_verb(&verbs, "get"), Some(("get take", "get")));
        
        // Pattern matches
        assert_eq!(find_matching_verb(&verbs, "foo"), Some(("foo*bar", "foo*bar")));
        assert_eq!(find_matching_verb(&verbs, "foob"), Some(("foo*bar", "foo*bar")));
        
        // Wildcard as last resort
        assert_eq!(find_matching_verb(&verbs, "unknown"), Some(("*", "*")));
        
        // No match if no wildcard
        let verbs_no_wildcard = vec![
            ("look".to_string(), vec!["look".to_string()]),
        ];
        assert_eq!(find_matching_verb(&verbs_no_wildcard, "unknown"), None);
    }
    
    #[test]
    fn test_score_match() {
        // Exact match scores highest
        assert!(score_match("look", "look") > score_match("loo*", "look"));
        
        // More specific patterns score higher
        assert!(score_match("pronoun_*", "pronoun_sub") > score_match("*", "pronoun_sub"));
        assert!(score_match("foo*bar", "foob") > score_match("foo*", "foob"));
    }
}