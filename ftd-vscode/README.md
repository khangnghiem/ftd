# FTD — Fast Draft

> A token-efficient file format and interactive canvas for drawing, design, and animation. Built in Rust + WASM.

## Features

### 🎨 Syntax Highlighting

Rich TextMate-based syntax highlighting for `.ftd` files:

- **Node keywords** — `group`, `rect`, `ellipse`, `text`, `path`
- **Style blocks** — `style accent { fill: #6C5CE7 }`
- **Node IDs** — `@login_form`, `@button`
- **Hex colors** — `#FF5733`, `#1A1A2E`
- **Properties** — `w:`, `h:`, `fill:`, `font:`, `layout:`
- **Animations** — `anim :hover { ... }`
- **Comments** — `# This is a comment`

### ✅ Live Parser Validation

Real-time error detection powered by the Rust parser compiled to WASM:

- Instant feedback as you type
- Parse errors shown as VS Code diagnostics (red squiggles)
- Error messages in the Problems panel

### 🖼️ Interactive Canvas (Preview)

GPU-style canvas that renders your `.ftd` designs:

- Bidirectional sync — edit text or canvas, both stay in sync
- Select and move elements visually
- Rectangle tool for quick prototyping
- Keyboard shortcuts (V = Select, R = Rect, Cmd+Z = Undo)

### 🌳 Tree Preview

View the parsed scene graph as a structural tree:

- Command: `FTD: Show Tree Preview`
- See node hierarchy, types, and properties at a glance

## Example

```ftd
# FTD v1

style accent {
  fill: #6C5CE7
}

rect @button {
  w: 200 h: 48
  corner: 10
  use: accent

  text @label "Sign In" {
    font: "Inter" 600 16
    fill: #FFFFFF
  }
}

@button -> center_in: canvas
```

## Requirements

- VS Code ≥ 1.85.0

## Links

- [GitHub Repository](https://github.com/khangnghiem/ftd)
- [FTD Format Specification](https://github.com/khangnghiem/ftd/blob/main/REQUIREMENTS.md)

## License

MIT
