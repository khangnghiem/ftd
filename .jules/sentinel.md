## 2025-02-15 - [XSS in Webview Node ID Injection]
**Vulnerability:** Node IDs parsed from `.fd` files were unsafely injected into Webview HTML via `.innerHTML` in `renderProposals` inside `fd-vscode/src/webview-html.ts` allowing script injection (XSS).
**Learning:** Even though `.fd` files are just design specifications, their contents (like node IDs) are effectively user inputs when rendered in VS Code Webviews. Using `.innerHTML` to render these fields directly opens up high-priority XSS vectors.
**Prevention:** When dynamically rendering user-provided or parsed strings into Webview HTML, always use DOM API creation with `.textContent` or explicitly sanitize using an `escapeHtml` utility before embedding into `.innerHTML`.
