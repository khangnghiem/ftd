//! Document formatting pipeline: parse → transforms → emit.
//!
//! Combines lint auto-fixes, transform passes, and canonical emit into a
//! single idempotent entry point consumed by the LSP `textDocument/formatting`
//! handler and the VS Code extension.

use crate::emitter::emit_document;
use crate::parser::parse_document;
use crate::transform::dedup_use_styles;

// ─── Config ───────────────────────────────────────────────────────────────

/// Configuration for `format_document`.
///
/// All transforms default to a safe, non-destructive subset.
/// Opt-in to structural transforms via explicit `true` fields.
#[derive(Debug, Clone)]
pub struct FormatConfig {
    /// Remove duplicate `use:` references on each node. Default: **true**.
    pub dedup_use: bool,

    /// Promote repeated identical inline styles to top-level `style {}` blocks.
    /// This is *structurally destructive* (adds new style names, rewrites nodes),
    /// so it defaults to **false** — users must explicitly opt-in.
    pub hoist_styles: bool,
}

impl Default for FormatConfig {
    fn default() -> Self {
        Self {
            dedup_use: true,
            hoist_styles: false,
        }
    }
}

// ─── Pipeline ─────────────────────────────────────────────────────────────

/// Parse an FD document, apply configured transforms, and re-emit canonical text.
///
/// The output is idempotent: `format_document(format_document(s, c), c) == format_document(s, c)`.
///
/// # Errors
/// Returns the parser error string if the input is not valid FD syntax.
pub fn format_document(text: &str, config: &FormatConfig) -> Result<String, String> {
    let mut scene = parse_document(text)?;

    if config.dedup_use {
        dedup_use_styles(&mut scene);
    }

    if config.hoist_styles {
        crate::transform::hoist_styles(&mut scene);
    }

    Ok(emit_document(&scene))
}

// ─── Tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_document_default_is_idempotent() {
        let input = r#"
# FD v1

style accent {
  fill: #6C5CE7
  corner: 10
}

rect @primary_btn {
  w: 200 h: 48
  use: accent
}
"#;
        let config = FormatConfig::default();
        let first = format_document(input, &config).expect("first format failed");
        let second = format_document(&first, &config).expect("second format failed");
        assert_eq!(first, second, "format must be idempotent");
    }

    #[test]
    fn format_document_dedupes_use_styles() {
        let input = r#"
style card {
  fill: #FFF
}
rect @box {
  w: 100 h: 50
  use: card
  use: card
}
"#;
        let config = FormatConfig::default();
        let output = format_document(input, &config).expect("format failed");
        // Should appear only once
        let use_count = output.matches("use: card").count();
        assert_eq!(use_count, 1, "duplicate use: should be removed");
    }

    #[test]
    fn format_document_preserves_comments() {
        let input = r#"
# This is a section header
rect @box {
  w: 100 h: 50
  fill: #FF0000
}
"#;
        let config = FormatConfig::default();
        let output = format_document(input, &config).expect("format failed");
        assert!(
            output.contains("# This is a section header"),
            "comments must survive formatting"
        );
    }
}
