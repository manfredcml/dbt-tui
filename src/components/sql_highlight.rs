use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};

/// SQL keywords that should be highlighted
const SQL_KEYWORDS: &[&str] = &[
    "SELECT",
    "FROM",
    "WHERE",
    "JOIN",
    "INNER",
    "LEFT",
    "RIGHT",
    "OUTER",
    "FULL",
    "ON",
    "AS",
    "AND",
    "OR",
    "NOT",
    "IN",
    "EXISTS",
    "BETWEEN",
    "LIKE",
    "IS",
    "NULL",
    "GROUP",
    "BY",
    "HAVING",
    "ORDER",
    "ASC",
    "DESC",
    "LIMIT",
    "OFFSET",
    "INSERT",
    "INTO",
    "VALUES",
    "UPDATE",
    "SET",
    "DELETE",
    "CREATE",
    "TABLE",
    "ALTER",
    "DROP",
    "INDEX",
    "VIEW",
    "DATABASE",
    "SCHEMA",
    "WITH",
    "CASE",
    "WHEN",
    "THEN",
    "ELSE",
    "END",
    "DISTINCT",
    "UNION",
    "ALL",
    "INTERSECT",
    "EXCEPT",
    "COUNT",
    "SUM",
    "AVG",
    "MIN",
    "MAX",
    "CAST",
    "COALESCE",
];

/// Token types for SQL syntax
#[derive(Debug, PartialEq, Clone)]
enum Token {
    Keyword(String),
    String(String),
    Number(String),
    Comment(String),
    Identifier(String),
    Whitespace(String),
    Punctuation(String),
}

/// Simple SQL tokenizer
fn tokenize(sql: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut chars = sql.chars().peekable();
    let mut current = String::new();

    while let Some(&ch) = chars.peek() {
        match ch {
            // String literals with single quotes
            '\'' => {
                if !current.is_empty() {
                    tokens.push(classify_word(&current));
                    current.clear();
                }
                current.push(chars.next().unwrap());
                while let Some(&ch) = chars.peek() {
                    current.push(chars.next().unwrap());
                    if ch == '\'' {
                        break;
                    }
                }
                tokens.push(Token::String(current.clone()));
                current.clear();
            }
            // String literals with double quotes (identifiers in some SQL dialects)
            '"' => {
                if !current.is_empty() {
                    tokens.push(classify_word(&current));
                    current.clear();
                }
                current.push(chars.next().unwrap());
                while let Some(&ch) = chars.peek() {
                    current.push(chars.next().unwrap());
                    if ch == '"' {
                        break;
                    }
                }
                tokens.push(Token::String(current.clone()));
                current.clear();
            }
            // Single-line comments
            '-' if chars.clone().nth(1) == Some('-') => {
                if !current.is_empty() {
                    tokens.push(classify_word(&current));
                    current.clear();
                }
                while let Some(&ch) = chars.peek() {
                    current.push(chars.next().unwrap());
                    if ch == '\n' {
                        break;
                    }
                }
                tokens.push(Token::Comment(current.clone()));
                current.clear();
            }
            // Multi-line comments
            '/' if chars.clone().nth(1) == Some('*') => {
                if !current.is_empty() {
                    tokens.push(classify_word(&current));
                    current.clear();
                }
                current.push(chars.next().unwrap()); // '/'
                current.push(chars.next().unwrap()); // '*'
                let mut prev_char = '*';
                while let Some(&ch) = chars.peek() {
                    current.push(chars.next().unwrap());
                    if prev_char == '*' && ch == '/' {
                        break;
                    }
                    prev_char = ch;
                }
                tokens.push(Token::Comment(current.clone()));
                current.clear();
            }
            // Whitespace
            ' ' | '\t' | '\n' | '\r' => {
                if !current.is_empty() {
                    tokens.push(classify_word(&current));
                    current.clear();
                }
                current.push(chars.next().unwrap());
                while let Some(&ch) = chars.peek() {
                    if ch == ' ' || ch == '\t' || ch == '\n' || ch == '\r' {
                        current.push(chars.next().unwrap());
                    } else {
                        break;
                    }
                }
                tokens.push(Token::Whitespace(current.clone()));
                current.clear();
            }
            // Operators and punctuation
            '(' | ')' | ',' | ';' | '.' | '*' | '=' | '<' | '>' | '+' | '/' | '%' => {
                if !current.is_empty() {
                    tokens.push(classify_word(&current));
                    current.clear();
                }
                current.push(chars.next().unwrap());
                tokens.push(Token::Punctuation(current.clone()));
                current.clear();
            }
            // Everything else (identifiers, numbers, keywords)
            _ => {
                current.push(chars.next().unwrap());
            }
        }
    }

    if !current.is_empty() {
        tokens.push(classify_word(&current));
    }

    tokens
}

/// Classify a word as keyword, number, or identifier
fn classify_word(word: &str) -> Token {
    let upper = word.to_uppercase();

    if SQL_KEYWORDS.contains(&upper.as_str()) {
        Token::Keyword(word.to_string())
    } else if word.chars().all(|c| c.is_ascii_digit() || c == '.') {
        Token::Number(word.to_string())
    } else {
        Token::Identifier(word.to_string())
    }
}

/// Convert SQL string into highlighted ratatui Lines
pub fn highlight_sql(sql: &str) -> Vec<Line<'static>> {
    let tokens = tokenize(sql);
    let mut lines = Vec::new();
    let mut current_line_spans = Vec::new();

    for token in tokens {
        let (style, text) = match token {
            Token::Keyword(s) => (
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
                s,
            ),
            Token::String(s) => (Style::default().fg(Color::Green), s),
            Token::Number(s) => (Style::default().fg(Color::Magenta), s),
            Token::Comment(s) => (Style::default().fg(Color::DarkGray), s),
            Token::Identifier(s) => (Style::default().fg(Color::White), s),
            Token::Whitespace(s) => (Style::default(), s),
            Token::Punctuation(s) => (Style::default().fg(Color::Gray), s),
        };

        // Split by newlines to create proper Lines
        for (i, part) in text.split('\n').enumerate() {
            if i > 0 {
                // Push the current line and start a new one
                lines.push(Line::from(current_line_spans.clone()));
                current_line_spans.clear();
            }
            if !part.is_empty() {
                current_line_spans.push(Span::styled(part.to_string(), style));
            }
        }
    }

    // Push the last line if it has content
    if !current_line_spans.is_empty() {
        lines.push(Line::from(current_line_spans));
    }

    // If no lines were created, return at least one empty line
    if lines.is_empty() {
        lines.push(Line::from(""));
    }

    lines
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenize_simple_select() {
        let sql = "SELECT * FROM users";
        let tokens = tokenize(sql);

        assert_eq!(tokens[0], Token::Keyword("SELECT".to_string()));
        assert_eq!(tokens[1], Token::Whitespace(" ".to_string()));
        assert_eq!(tokens[2], Token::Punctuation("*".to_string()));
        assert_eq!(tokens[3], Token::Whitespace(" ".to_string()));
        assert_eq!(tokens[4], Token::Keyword("FROM".to_string()));
        assert_eq!(tokens[5], Token::Whitespace(" ".to_string()));
        assert_eq!(tokens[6], Token::Identifier("users".to_string()));
    }

    #[test]
    fn test_tokenize_with_string() {
        let sql = "WHERE name = 'John'";
        let tokens = tokenize(sql);

        // Expected: WHERE, " ", name, " ", =, " ", 'John'
        // Token indices: 0, 1, 2, 3, 4, 5, 6
        assert!(matches!(tokens[0], Token::Keyword(_)));
        assert!(tokens.iter().any(|t| matches!(t, Token::String(_))));

        // Find the string token
        let string_token = tokens.iter().find(|t| matches!(t, Token::String(_)));
        assert!(string_token.is_some());
        assert_eq!(string_token.unwrap(), &Token::String("'John'".to_string()));
    }

    #[test]
    fn test_tokenize_with_number() {
        let sql = "LIMIT 100";
        let tokens = tokenize(sql);

        assert_eq!(tokens[2], Token::Number("100".to_string()));
    }

    #[test]
    fn test_tokenize_with_comment() {
        let sql = "SELECT * -- get all columns\nFROM users";
        let tokens = tokenize(sql);

        assert!(tokens.iter().any(|t| matches!(t, Token::Comment(_))));
    }

    #[test]
    fn test_highlight_sql_returns_lines() {
        let sql = "SELECT id, name\nFROM users";
        let lines = highlight_sql(sql);

        assert!(lines.len() >= 2);
    }

    #[test]
    fn test_classify_keyword() {
        assert!(matches!(classify_word("SELECT"), Token::Keyword(_)));
        assert!(matches!(classify_word("select"), Token::Keyword(_)));
        assert!(matches!(classify_word("SeLeCt"), Token::Keyword(_)));
    }

    #[test]
    fn test_classify_number() {
        assert!(matches!(classify_word("123"), Token::Number(_)));
        assert!(matches!(classify_word("45.67"), Token::Number(_)));
    }

    #[test]
    fn test_classify_identifier() {
        assert!(matches!(classify_word("customer_id"), Token::Identifier(_)));
        assert!(matches!(classify_word("table1"), Token::Identifier(_)));
    }
}
