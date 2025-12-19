//! Sample data output model for dbt show command

use super::run::RunStatus;

/// Output from a dbt show command
#[derive(Debug, Clone, Default)]
pub struct SampleDataOutput {
    pub model_name: String,
    pub status: RunStatus,
    pub raw_output: String,
    pub headers: Vec<String>,
    pub rows: Vec<Vec<String>>,
    pub error_message: Option<String>,
}

impl SampleDataOutput {
    pub fn new(model_name: String) -> Self {
        Self {
            model_name,
            status: RunStatus::Running,
            ..Default::default()
        }
    }

    /// Parse the dbt show output into headers and rows
    ///
    /// dbt show outputs data in a pipe-delimited table format:
    /// ```text
    /// | column1 | column2 | column3 |
    /// |---------|---------|---------|
    /// | value1  | value2  | value3  |
    /// ```
    pub fn parse_output(&mut self) {
        self.headers.clear();
        self.rows.clear();

        let mut headers_found = false;

        for line in self.raw_output.lines() {
            let trimmed = line.trim();

            // Skip empty lines
            if trimmed.is_empty() {
                continue;
            }

            // Skip separator lines (only -, |, +, whitespace)
            if trimmed
                .chars()
                .all(|c| c == '-' || c == '|' || c == '+' || c.is_whitespace())
            {
                continue;
            }

            // Check if this is a data line (contains |)
            if trimmed.contains('|') {
                let cells: Vec<String> = trimmed
                    .split('|')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect();

                if cells.is_empty() {
                    continue;
                }

                if !headers_found {
                    self.headers = cells;
                    headers_found = true;
                } else {
                    self.rows.push(cells);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_dbt_show_output() {
        let mut output = SampleDataOutput::new("test_model".to_string());
        output.raw_output = r#"
| id | name    | value |
|----|---------|-------|
| 1  | Alice   | 100   |
| 2  | Bob     | 200   |
"#
        .to_string();

        output.parse_output();

        assert_eq!(output.headers, vec!["id", "name", "value"]);
        assert_eq!(output.rows.len(), 2);
        assert_eq!(output.rows[0], vec!["1", "Alice", "100"]);
        assert_eq!(output.rows[1], vec!["2", "Bob", "200"]);
    }

    #[test]
    fn test_parse_empty_output() {
        let mut output = SampleDataOutput::new("test_model".to_string());
        output.raw_output = String::new();

        output.parse_output();

        assert!(output.headers.is_empty());
        assert!(output.rows.is_empty());
    }

    #[test]
    fn test_parse_single_row() {
        let mut output = SampleDataOutput::new("test_model".to_string());
        output.raw_output = r#"
| col1 | col2 |
|------|------|
| a    | b    |
"#
        .to_string();

        output.parse_output();

        assert_eq!(output.headers, vec!["col1", "col2"]);
        assert_eq!(output.rows.len(), 1);
        assert_eq!(output.rows[0], vec!["a", "b"]);
    }

    #[test]
    fn test_parse_headers_only() {
        let mut output = SampleDataOutput::new("test_model".to_string());
        output.raw_output = r#"
| col1 | col2 |
|------|------|
"#
        .to_string();

        output.parse_output();

        assert_eq!(output.headers, vec!["col1", "col2"]);
        assert!(output.rows.is_empty());
    }
}
