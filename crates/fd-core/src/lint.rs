//! Lint diagnostics for FD documents.
//!
//! Reports structural issues without modifying the document.
//! Results feed into `textDocument/publishDiagnostics` in the LSP server.

use crate::id::NodeId;
use crate::model::SceneGraph;
use std::collections::HashSet;

// ─── Diagnostic types ────────────────────────────────────────────────────

/// Severity of a lint finding.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LintSeverity {
    /// Should be fixed — likely a mistake.
    Warning,
    /// Informational — style suggestion.
    Info,
}

/// A single lint diagnostic for a scene node.
#[derive(Debug, Clone)]
pub struct LintDiagnostic {
    /// The node this diagnostic refers to.
    pub node_id: NodeId,
    /// Human-readable message.
    pub message: String,
    /// Severity level.
    pub severity: LintSeverity,
    /// Short rule identifier (e.g. "anonymous-id", "duplicate-use").
    pub rule: &'static str,
}

// ─── Public API ───────────────────────────────────────────────────────────

/// Run all lint rules over the scene graph and return diagnostics.
#[must_use]
pub fn lint_document(graph: &SceneGraph) -> Vec<LintDiagnostic> {
    let mut diags = Vec::new();
    lint_anonymous_ids(graph, &mut diags);
    lint_duplicate_use(graph, &mut diags);
    lint_unused_styles(graph, &mut diags);
    diags
}

// ─── Rules ────────────────────────────────────────────────────────────────

/// Warn on any node whose ID matches auto-generated `_kind_N` pattern.
fn lint_anonymous_ids(graph: &SceneGraph, diags: &mut Vec<LintDiagnostic>) {
    for idx in graph.graph.node_indices() {
        let node = &graph.graph[idx];
        if is_anonymous_id(node.id.as_str()) {
            diags.push(LintDiagnostic {
                node_id: node.id,
                message: format!(
                    "Anonymous node `@{}` — consider giving it a semantic name like `@button_primary`.",
                    node.id.as_str()
                ),
                severity: LintSeverity::Warning,
                rule: "anonymous-id",
            });
        }
    }
}

/// Check if an ID matches the auto-generated `_kind_N` pattern.
fn is_anonymous_id(id: &str) -> bool {
    let prefixes = [
        "_rect_",
        "_ellipse_",
        "_text_",
        "_group_",
        "_path_",
        "_frame_",
        "_generic_",
        "_edge_",
    ];
    prefixes.iter().any(|p| id.starts_with(p))
}

/// Warn when the same style name appears more than once in a node's `use:` list.
fn lint_duplicate_use(graph: &SceneGraph, diags: &mut Vec<LintDiagnostic>) {
    for idx in graph.graph.node_indices() {
        let node = &graph.graph[idx];
        let mut seen = HashSet::new();
        for style_id in &node.use_styles {
            if !seen.insert(style_id) {
                diags.push(LintDiagnostic {
                    node_id: node.id,
                    message: format!(
                        "Duplicate `use: {}` on `@{}` — remove the extra reference.",
                        style_id.as_str(),
                        node.id.as_str()
                    ),
                    severity: LintSeverity::Warning,
                    rule: "duplicate-use",
                });
            }
        }
    }
}

/// Info when a top-level `style {}` block is defined but never referenced.
fn lint_unused_styles(graph: &SceneGraph, diags: &mut Vec<LintDiagnostic>) {
    // Collect all style IDs referenced by any node or edge.
    let mut referenced: HashSet<NodeId> = HashSet::new();

    for idx in graph.graph.node_indices() {
        for style_id in &graph.graph[idx].use_styles {
            referenced.insert(*style_id);
        }
    }
    for edge in &graph.edges {
        for style_id in &edge.use_styles {
            referenced.insert(*style_id);
        }
    }

    // Report any defined-but-unreferenced style.
    for style_id in graph.styles.keys() {
        if !referenced.contains(style_id) {
            diags.push(LintDiagnostic {
                node_id: *style_id,
                message: format!("Style `{}` is defined but never used.", style_id.as_str()),
                severity: LintSeverity::Info,
                rule: "unused-style",
            });
        }
    }
}

// ─── Tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse_document;

    #[test]
    fn lint_anonymous_ids() {
        // Auto-generated IDs come from nodes without an explicit @id (e.g. _rect_0)
        let input = "rect { w: 100 h: 50 }\n";
        let graph = parse_document(input).unwrap();
        let diags = lint_document(&graph);
        assert!(
            diags.iter().any(|d| d.rule == "anonymous-id"),
            "expected anonymous-id diagnostic"
        );
    }

    #[test]
    fn lint_duplicate_use() {
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
        let graph = parse_document(input).unwrap();
        let diags = lint_document(&graph);
        assert!(
            diags.iter().any(|d| d.rule == "duplicate-use"),
            "expected duplicate-use diagnostic"
        );
    }

    #[test]
    fn lint_unused_style() {
        let input = r#"
style ghost {
  opacity: 0.5
}
rect @box {
  w: 100 h: 50
}
"#;
        let graph = parse_document(input).unwrap();
        let diags = lint_document(&graph);
        assert!(
            diags.iter().any(|d| d.rule == "unused-style"),
            "expected unused-style diagnostic"
        );
    }

    #[test]
    fn lint_clean_document_no_diags() {
        let input = r#"
style card {
  fill: #FFF
}
rect @primary_btn {
  w: 200 h: 48
  use: card
}
"#;
        let graph = parse_document(input).unwrap();
        let diags = lint_document(&graph);
        assert!(
            diags.is_empty(),
            "clean document should have no diagnostics"
        );
    }
}
