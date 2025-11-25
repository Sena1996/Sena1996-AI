//! Unicode Table Builder
//!
//! Creates beautifully formatted tables using Unicode box-drawing characters

/// Table builder for creating Unicode tables
#[derive(Debug, Default)]
pub struct TableBuilder {
    title: Option<String>,
    rows: Vec<Vec<String>>,
    min_col_width: usize,
    padding: usize,
}

impl TableBuilder {
    /// Create a new table builder
    pub fn new() -> Self {
        Self {
            title: None,
            rows: Vec::new(),
            min_col_width: 10,
            padding: 1,
        }
    }

    /// Set the table title
    pub fn title(mut self, title: &str) -> Self {
        self.title = Some(title.to_string());
        self
    }

    /// Add a row to the table
    pub fn row(mut self, cells: Vec<String>) -> Self {
        self.rows.push(cells);
        self
    }

    /// Set minimum column width
    pub fn min_width(mut self, width: usize) -> Self {
        self.min_col_width = width;
        self
    }

    /// Build the table string
    pub fn build(&self) -> String {
        if self.rows.is_empty() {
            return String::new();
        }

        // Calculate column widths
        let col_count = self.rows.iter().map(|r| r.len()).max().unwrap_or(0);
        let mut col_widths = vec![self.min_col_width; col_count];

        for row in &self.rows {
            for (i, cell) in row.iter().enumerate() {
                if i < col_widths.len() {
                    col_widths[i] = col_widths[i].max(cell.len() + self.padding * 2);
                }
            }
        }

        let total_width: usize = col_widths.iter().sum::<usize>() + col_count + 1;

        let mut output = String::new();

        // Title box if present
        if let Some(title) = &self.title {
            output.push_str(&self.build_title_box(title, total_width));
            output.push('\n');
        }

        // Top border
        output.push_str(&self.build_border(&col_widths, '┌', '─', '┬', '┐'));
        output.push('\n');

        // Rows
        for (i, row) in self.rows.iter().enumerate() {
            output.push_str(&self.build_row(row, &col_widths));
            output.push('\n');

            // Separator after first row (header)
            if i == 0 && self.rows.len() > 1 {
                output.push_str(&self.build_border(&col_widths, '├', '─', '┼', '┤'));
                output.push('\n');
            }
        }

        // Bottom border
        output.push_str(&self.build_border(&col_widths, '└', '─', '┴', '┘'));

        output
    }

    fn build_title_box(&self, title: &str, width: usize) -> String {
        let box_width = width.max(title.len() + 8);
        let title_padding = (box_width - title.len() - 2) / 2;

        let mut output = String::new();

        // Top
        output.push('╔');
        output.push_str(&"═".repeat(box_width - 2));
        output.push_str("╗\n");

        // Empty line
        output.push('║');
        output.push_str(&" ".repeat(box_width - 2));
        output.push_str("║\n");

        // Title line
        output.push('║');
        output.push_str(&" ".repeat(title_padding));
        output.push_str("📊 ");
        output.push_str(title);
        let remaining = box_width - 2 - title_padding - title.len() - 3;
        output.push_str(&" ".repeat(remaining));
        output.push_str("║\n");

        // Empty line
        output.push('║');
        output.push_str(&" ".repeat(box_width - 2));
        output.push_str("║\n");

        // Bottom
        output.push('╚');
        output.push_str(&"═".repeat(box_width - 2));
        output.push('╝');

        output
    }

    fn build_border(&self, widths: &[usize], left: char, fill: char, mid: char, right: char) -> String {
        let mut output = String::new();
        output.push(left);

        for (i, width) in widths.iter().enumerate() {
            output.push_str(&fill.to_string().repeat(*width));
            if i < widths.len() - 1 {
                output.push(mid);
            }
        }

        output.push(right);
        output
    }

    fn build_row(&self, cells: &[String], widths: &[usize]) -> String {
        let mut output = String::new();
        output.push('│');

        for (i, width) in widths.iter().enumerate() {
            let cell = cells.get(i).cloned().unwrap_or_default();
            let padding_left = self.padding;
            let content_width = width - self.padding * 2;
            let display = if cell.len() > content_width {
                format!("{}...", &cell[..content_width.saturating_sub(3)])
            } else {
                cell.clone()
            };
            let padding_right = width - padding_left - display.len();

            output.push_str(&" ".repeat(padding_left));
            output.push_str(&display);
            output.push_str(&" ".repeat(padding_right));
            output.push('│');
        }

        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_table() {
        let table = TableBuilder::new().build();
        assert!(table.is_empty());
    }

    #[test]
    fn test_single_row() {
        let table = TableBuilder::new()
            .row(vec!["Header".to_string()])
            .build();

        assert!(table.contains("┌"));
        assert!(table.contains("└"));
        assert!(table.contains("Header"));
    }

    #[test]
    fn test_with_title() {
        let table = TableBuilder::new()
            .title("Test Table")
            .row(vec!["Col1".to_string(), "Col2".to_string()])
            .build();

        assert!(table.contains("╔"));
        assert!(table.contains("Test Table"));
        assert!(table.contains("Col1"));
    }

    #[test]
    fn test_multiple_rows() {
        let table = TableBuilder::new()
            .row(vec!["Name".to_string(), "Value".to_string()])
            .row(vec!["foo".to_string(), "bar".to_string()])
            .row(vec!["baz".to_string(), "qux".to_string()])
            .build();

        assert!(table.contains("├"));  // Row separator
        assert!(table.contains("foo"));
        assert!(table.contains("qux"));
    }
}
