//! Dependency-free Unicode table rendering with ANSI-aware width handling.

use crate::ansi::{self, AnsiColor, Attr, Color};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Attribute {
    Bold,
}

/// Border style. `Rounded` matches the historical comfy-table
/// `UTF8_ROUND_CORNERS` look (double-line header rule, a rule between every
/// row). `Square` matches a plain hand-rolled box: square corners, a single
/// rule after the header, and no rules between data rows.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TableStyle {
    #[default]
    Rounded,
    Square,
    /// Matches the `tabled` crate's default grid: plain ASCII `+-|`, with a
    /// rule after the header and after every data row.
    Ascii,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CellAlignment {
    #[default]
    Left,
    Center,
    Right,
}

#[derive(Debug, Clone)]
pub struct Cell {
    text: String,
    bold: bool,
    fg: Option<Color>,
}

impl Cell {
    pub fn new(value: impl fmt::Display) -> Self {
        Self {
            text: value.to_string(),
            bold: false,
            fg: None,
        }
    }
    pub fn add_attribute(mut self, attribute: Attribute) -> Self {
        self.bold = matches!(attribute, Attribute::Bold);
        self
    }
    pub fn fg(mut self, color: Color) -> Self {
        self.fg = Some(color);
        self
    }
}

impl<T: fmt::Display> From<T> for Cell {
    fn from(value: T) -> Self {
        Self::new(value)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Width {
    Fixed(u16),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColumnConstraint {
    LowerBoundary(Width),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContentArrangement {
    Dynamic,
}

#[derive(Debug, Clone, Copy)]
pub struct Modifier;

pub mod modifiers {
    use super::Modifier;
    pub const UTF8_ROUND_CORNERS: Modifier = Modifier;
}

#[derive(Debug, Clone, Default)]
pub struct Column {
    alignment: CellAlignment,
}

impl Column {
    pub fn set_cell_alignment(&mut self, alignment: CellAlignment) -> &mut Self {
        self.alignment = alignment;
        self
    }
}

#[derive(Debug, Clone, Default)]
pub struct Table {
    header: Vec<Cell>,
    rows: Vec<Vec<Cell>>,
    columns: Vec<Column>,
    minimum_widths: Vec<usize>,
    style: TableStyle,
}

impl Table {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_style(&mut self, style: TableStyle) -> &mut Self {
        self.style = style;
        self
    }

    pub fn set_header<T: Into<Cell>>(&mut self, cells: Vec<T>) -> &mut Self {
        self.header = cells.into_iter().map(Into::into).collect();
        self.ensure_columns();
        self
    }

    pub fn add_row<T: Into<Cell>>(&mut self, cells: Vec<T>) -> &mut Self {
        self.rows.push(cells.into_iter().map(Into::into).collect());
        self.ensure_columns();
        self
    }

    pub fn apply_modifier(&mut self, _: Modifier) -> &mut Self {
        self
    }
    pub fn set_content_arrangement(&mut self, _: ContentArrangement) -> &mut Self {
        self
    }

    pub fn set_constraints(&mut self, constraints: Vec<ColumnConstraint>) -> &mut Self {
        self.minimum_widths = constraints
            .into_iter()
            .map(|constraint| match constraint {
                ColumnConstraint::LowerBoundary(Width::Fixed(width)) => width as usize,
            })
            .collect();
        self.ensure_columns();
        self
    }

    pub fn column_mut(&mut self, index: usize) -> Option<&mut Column> {
        self.ensure_columns();
        self.columns.get_mut(index)
    }

    fn ensure_columns(&mut self) {
        let count = self
            .header
            .len()
            .max(self.rows.iter().map(Vec::len).max().unwrap_or(0));
        self.columns.resize_with(count, Column::default);
    }

    fn widths(&self) -> Vec<usize> {
        (0..self.columns.len())
            .map(|index| {
                let header = self
                    .header
                    .get(index)
                    .map_or(0, |cell| visible_width(&cell.text));
                let rows = self
                    .rows
                    .iter()
                    .filter_map(|row| row.get(index))
                    .map(|cell| visible_width(&cell.text))
                    .max()
                    .unwrap_or(0);
                header
                    .max(rows)
                    .max(self.minimum_widths.get(index).copied().unwrap_or(0))
            })
            .collect()
    }
}

impl fmt::Display for Table {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let widths = self.widths();
        if widths.is_empty() {
            return Ok(());
        }
        match self.style {
            TableStyle::Rounded => {
                border(formatter, '╭', '┬', '╮', '─', &widths)?;
                if !self.header.is_empty() {
                    row(formatter, &self.header, &self.columns, &widths, '│')?;
                    border(formatter, '╞', '╪', '╡', '═', &widths)?;
                }
                for (index, cells) in self.rows.iter().enumerate() {
                    row(formatter, cells, &self.columns, &widths, '│')?;
                    if index + 1 < self.rows.len() {
                        border(formatter, '├', '┼', '┤', '─', &widths)?;
                    }
                }
                border(formatter, '╰', '┴', '╯', '─', &widths)
            }
            TableStyle::Square => {
                border(formatter, '┌', '┬', '┐', '─', &widths)?;
                if !self.header.is_empty() {
                    row(formatter, &self.header, &self.columns, &widths, '│')?;
                    border(formatter, '├', '┼', '┤', '─', &widths)?;
                }
                for cells in &self.rows {
                    row(formatter, cells, &self.columns, &widths, '│')?;
                }
                border(formatter, '└', '┴', '┘', '─', &widths)
            }
            TableStyle::Ascii => {
                border(formatter, '+', '+', '+', '-', &widths)?;
                if !self.header.is_empty() {
                    row(formatter, &self.header, &self.columns, &widths, '|')?;
                    border(formatter, '+', '+', '+', '-', &widths)?;
                }
                for cells in &self.rows {
                    row(formatter, cells, &self.columns, &widths, '|')?;
                    border(formatter, '+', '+', '+', '-', &widths)?;
                }
                Ok(())
            }
        }
    }
}

fn border(
    f: &mut fmt::Formatter<'_>,
    left: char,
    join: char,
    right: char,
    fill: char,
    widths: &[usize],
) -> fmt::Result {
    write!(f, "{left}")?;
    for (index, width) in widths.iter().enumerate() {
        if index > 0 {
            write!(f, "{join}")?;
        }
        for _ in 0..width + 2 {
            write!(f, "{fill}")?;
        }
    }
    writeln!(f, "{right}")
}

fn row(
    f: &mut fmt::Formatter<'_>,
    cells: &[Cell],
    columns: &[Column],
    widths: &[usize],
    vertical: char,
) -> fmt::Result {
    write!(f, "{vertical}")?;
    for (index, width) in widths.iter().enumerate() {
        let cell = cells.get(index).cloned().unwrap_or_else(|| Cell::new(""));
        let visible = visible_width(&cell.text);
        let remaining = width.saturating_sub(visible);
        let alignment = columns
            .get(index)
            .map_or(CellAlignment::Left, |column| column.alignment);
        let (before, after) = match alignment {
            CellAlignment::Left => (0, remaining),
            CellAlignment::Center => (remaining / 2, remaining - remaining / 2),
            CellAlignment::Right => (remaining, 0),
        };
        let mut codes = String::new();
        if let Some(color) = cell.fg {
            codes.push_str(&ansi::fg(&AnsiColor::Standard(color, false)));
        }
        if cell.bold {
            codes.push_str(&ansi::sgr(Attr::Bold));
        }
        let text = if codes.is_empty() {
            cell.text
        } else {
            format!("{codes}{}{}", cell.text, ansi::reset())
        };
        write!(
            f,
            " {}{}{} {vertical}",
            " ".repeat(before),
            text,
            " ".repeat(after)
        )?;
    }
    writeln!(f)
}

fn visible_width(text: &str) -> usize {
    let mut width = 0;
    let mut escape = false;
    for character in text.chars() {
        if escape {
            if ('@'..='~').contains(&character) {
                escape = false;
            }
        } else if character == '\x1b' {
            escape = true;
        } else {
            width += 1;
        }
    }
    width
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_rounded_ansi_aware_table() {
        let mut table = Table::new();
        table.set_header(vec![
            Cell::new("Name").add_attribute(Attribute::Bold),
            Cell::new("State"),
        ]);
        table.add_row(vec![Cell::new("amber"), Cell::new("\x1b[32mok\x1b[0m")]);
        let output = table.to_string();
        assert!(output.starts_with('╭'));
        assert!(output.contains("amber"));
        assert!(output.ends_with("╯\n"));
    }

    #[test]
    fn renders_square_table_with_no_row_rules() {
        let mut table = Table::new();
        table.set_style(TableStyle::Square);
        table.set_header(vec![
            Cell::new("Name")
                .fg(Color::Cyan)
                .add_attribute(Attribute::Bold),
            Cell::new("State"),
        ]);
        table.add_row(vec![Cell::new("ami"), Cell::new("ok")]);
        table.add_row(vec![Cell::new("amber"), Cell::new("ok")]);
        let output = table.to_string();
        assert!(output.starts_with('┌'));
        assert!(output.contains("\x1b[36m\x1b[1mName\x1b[0m"));
        assert_eq!(output.matches('┼').count(), 1);
        assert!(output.ends_with("┘\n"));
    }

    #[test]
    fn renders_ascii_table_matching_tabled_default() {
        let mut table = Table::new();
        table.set_style(TableStyle::Ascii);
        table.set_header(vec![Cell::new("Tool"), Cell::new("Installed")]);
        table.add_row(vec![Cell::new("baby"), Cell::new("1.0")]);
        table.add_row(vec![Cell::new("amber"), Cell::new("2.0")]);
        assert_eq!(
            table.to_string(),
            "+-------+-----------+\n\
             | Tool  | Installed |\n\
             +-------+-----------+\n\
             | baby  | 1.0       |\n\
             +-------+-----------+\n\
             | amber | 2.0       |\n\
             +-------+-----------+\n"
        );
    }
}
