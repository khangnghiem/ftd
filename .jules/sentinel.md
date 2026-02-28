## 2024-06-03 - Initial Learning

**Vulnerability:** Found unescaped HTML content generated in VS Code Webviews. In `fd-vscode/src/webview-html.ts`, the `renderProposals` function dynamically generates HTML for rename proposals without escaping the `p.oldId` and `p.newId` values. This is an XSS vulnerability, allowing malicious code embedded in node IDs to be executed in the Webview context.

**Learning:** When generating HTML dynamically for a Webview, we must sanitize or escape all user-provided or dynamically parsed strings, particularly strings derived from the source document (like node IDs) to prevent script injection.

**Prevention:** Always use a utility function like `escapeHtml` when embedding text into `.innerHTML`, or avoid `.innerHTML` and use `document.createElement()` with `.textContent` instead.
