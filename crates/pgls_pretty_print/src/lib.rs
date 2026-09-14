mod codegen;
pub mod emitter;
pub mod nodes;
pub mod normalize;
pub mod renderer;

pub use crate::codegen::token_kind::TokenKind;
pub use crate::normalize::normalize_ast;
pub use crate::renderer::{IndentStyle, KeywordCase, RenderConfig};
use pgls_query::NodeEnum;
use thiserror::Error;

/// Error type for formatting operations.
#[derive(Debug, Error)]
pub enum FormatError {
    /// Parsing failed - the formatted output is not valid SQL
    #[error("Failed to parse formatted output: {message}")]
    ParseError { message: String },

    /// Rendering failed
    #[error("Failed to render: {message}")]
    RenderError { message: String },

    /// Beta safety check failed - formatted output has different semantics
    #[error(
        "Formatter (beta): This statement type is not fully supported yet. \
         Formatting may alter semantics. Please report: \
         https://github.com/supabase/postgres-language-server/issues\n\
         Details: {message}"
    )]
    BetaUnsupported { message: String },
}

/// How an explicit cast is spelled.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum CastStyle {
    /// `CAST(expr AS type)`.
    #[default]
    Cast,
    /// `expr::type`.
    Operator,
}

/// Where the body of a clause starts.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum ClauseBodyStyle {
    /// The body starts on the line after the keyword.
    #[default]
    Break,
    /// The first element of the body stays on the keyword line: `FROM plan`.
    Compact,
}

/// Configuration for the SQL formatter.
/// Where a comma sits when a list breaks across lines.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum CommaStyle {
    /// `a,` at the end of the line.
    #[default]
    Trailing,
    /// `, a` at the start of the continuation line.
    Leading,
}

/// Where a boolean operator sits when a condition breaks across lines.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum LogicalOperatorPlacement {
    /// `a = 1 AND` at the end of the line.
    #[default]
    Trailing,
    /// `AND a = 1` at the start of the continuation line.
    Leading,
}

#[derive(Debug, Clone)]
pub struct FormatConfig {
    /// Maximum line width before breaking. Default: 100.
    pub line_width: usize,
    /// Number of spaces (or tab width) for indentation. Default: 2.
    pub indent_size: usize,
    /// Whether to use spaces or tabs for indentation. Default: Spaces.
    pub indent_style: IndentStyle,
    /// Casing for SQL keywords (SELECT, FROM, WHERE). Default: Lower.
    pub keyword_case: KeywordCase,
    /// Casing for constants (NULL, TRUE, FALSE). Default: Lower.
    pub constant_case: KeywordCase,
    /// Casing for data types (text, varchar, int). Default: Lower.
    pub type_case: KeywordCase,
    /// Where a comma sits when a list breaks. Default: Trailing.
    pub comma_style: CommaStyle,
    /// Where a boolean operator sits when a condition breaks. Default: Trailing.
    pub logical_operator_placement: LogicalOperatorPlacement,
    /// How an explicit cast is spelled. Default: Cast.
    pub cast_style: CastStyle,
    /// Where the body of a clause starts. Default: Break.
    pub clause_body_style: ClauseBodyStyle,
    /// Put the terminating semicolon on its own line when the statement spans several lines.
    /// Default: false.
    pub isolate_semicolon: bool,
}

impl Default for FormatConfig {
    fn default() -> Self {
        Self {
            line_width: 100,
            indent_size: 2,
            indent_style: IndentStyle::Spaces,
            keyword_case: KeywordCase::default(),
            constant_case: KeywordCase::default(),
            type_case: KeywordCase::default(),
            comma_style: CommaStyle::default(),
            logical_operator_placement: LogicalOperatorPlacement::default(),
            cast_style: CastStyle::default(),
            clause_body_style: ClauseBodyStyle::default(),
            isolate_semicolon: false,
        }
    }
}

impl From<FormatConfig> for RenderConfig {
    fn from(config: FormatConfig) -> Self {
        let FormatConfig {
            line_width,
            indent_size,
            indent_style,
            keyword_case,
            constant_case,
            type_case,
            comma_style: _,
            logical_operator_placement: _,
            cast_style: _,
            clause_body_style: _,
            isolate_semicolon,
        } = config;

        Self {
            max_line_length: line_width,
            indent_size,
            indent_style,
            keyword_case,
            constant_case,
            type_case,
            isolate_semicolon,
        }
    }
}

/// Result of formatting a SQL statement.
#[derive(Debug)]
pub struct FormatResult {
    /// The formatted SQL string.
    pub formatted: String,
}

/// Format a single SQL statement from its AST.
///
/// This function takes an already-parsed AST, formats it according to the
/// configuration, and returns the formatted output.
///
/// During beta, this function performs semantic verification by
/// comparing normalized ASTs. If the formatted output would have
/// different semantics, it returns a `BetaUnsupported` error.
///
/// # Arguments
///
/// * `ast` - The parsed AST of the SQL statement
/// * `config` - Formatting configuration options
///
/// # Returns
///
/// * `Ok(FormatResult)` - The formatted SQL
/// * `Err(FormatError)` - If formatting fails or beta safety check fails
pub fn format_statement(
    ast: &NodeEnum,
    config: &FormatConfig,
) -> Result<FormatResult, FormatError> {
    // Emit layout events from AST
    let mut emitter = emitter::EventEmitter::new(config.clone());
    nodes::emit_node_enum(ast, &mut emitter);

    // Render to string
    let render_config = RenderConfig {
        max_line_length: config.line_width,
        indent_size: config.indent_size,
        indent_style: config.indent_style.clone(),
        keyword_case: config.keyword_case.clone(),
        constant_case: config.constant_case.clone(),
        type_case: config.type_case.clone(),
        isolate_semicolon: config.isolate_semicolon,
    };

    let mut output = String::new();
    let mut renderer = renderer::Renderer::new(&mut output, render_config);
    renderer
        .render(emitter.events)
        .map_err(|e| FormatError::RenderError {
            message: e.to_string(),
        })?;

    // BETA: Verify formatted output parses to semantically equivalent AST
    let parsed_output = pgls_query::parse(&output).map_err(|e| FormatError::ParseError {
        message: format!("Formatted SQL failed to parse: {e}"),
    })?;

    let mut output_ast = parsed_output
        .into_root()
        .ok_or_else(|| FormatError::ParseError {
            message: "No root node in parsed output (expected single statement)".to_string(),
        })?;

    let mut original_ast = ast.clone();

    // Normalize both ASTs for semantic comparison
    normalize::normalize_ast(&mut output_ast);
    normalize::normalize_ast(&mut original_ast);

    if output_ast != original_ast {
        return Err(FormatError::BetaUnsupported {
            message: "Normalized ASTs differ after formatting".to_string(),
        });
    }

    Ok(FormatResult { formatted: output })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_simple_select() {
        let sql = "SELECT id, name FROM users WHERE active = true";
        let parsed = pgls_query::parse(sql).unwrap();
        let ast = parsed.into_root().unwrap();

        let config = FormatConfig::default();
        let result = format_statement(&ast, &config).unwrap();

        assert!(!result.formatted.is_empty());
        // Default keyword_case is Lower, so check for lowercase
        assert!(result.formatted.contains("select"));
    }

    #[test]
    fn test_format_config_defaults() {
        let config = FormatConfig::default();
        assert_eq!(config.line_width, 100);
        assert_eq!(config.indent_size, 2);
    }
}
