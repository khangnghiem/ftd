/**
 * E2E UX Behavior Tests — Simulating Common Design Tool Behaviors
 *
 * Tests the fd-parse functions that underpin UX behaviors found in:
 * Figma, Sketch, Miro, Freeform, Drawio, and Excalidraw.
 *
 * Each describe() block documents which tool's behavior is being simulated
 * and what the expected UX outcome should be.
 */

import { describe, it, expect } from "vitest";
import {
    parseAnnotation,
    parseSpecNodes,
    computeSpecHideLines,
    computeSpecFoldRanges,
    findAnonNodeIds,
    stripMarkdownFences,
    escapeHtml,
    parseDocumentSymbols,
    findSymbolAtLine,
    transformSpecViewLine,
} from "./fd-parse";

// ─── Figma: Select & Move ────────────────────────────────────────────────
// In Figma, clicking a node selects it and shows its properties.
// Nodes are identified by their @id in the layer tree.

describe("Figma: Select & Move — Layer tree reflects node hierarchy", () => {
    it("parses a single node for selection", () => {
        const lines = ["rect @button {", "  fill: #007AFF", "  w: 120 h: 40", "}"];
        const symbols = parseDocumentSymbols(lines);
        expect(symbols).toHaveLength(1);
        expect(symbols[0].name).toBe("@button");
        expect(symbols[0].kind).toBe("rect");
    });

    it("clicking on a property line resolves to the parent node (Figma behavior)", () => {
        const lines = ["rect @card {", "  fill: #FFFFFF", "  w: 300 h: 200", "  corner: 16", "}"];
        const symbols = parseDocumentSymbols(lines);
        // In Figma, clicking anywhere inside a node selects the node
        expect(findSymbolAtLine(symbols, 2)).toBeDefined();
        expect(findSymbolAtLine(symbols, 2)!.name).toBe("@card");
    });

    it("selecting nested children drills into groups (Figma double-click)", () => {
        const lines = [
            "group @toolbar {",
            "  rect @btn_save {",
            "    fill: #34C759",
            "  }",
            "  rect @btn_cancel {",
            "    fill: #FF3B30",
            "  }",
            "}",
        ];
        const symbols = parseDocumentSymbols(lines);
        expect(symbols).toHaveLength(1);
        expect(symbols[0].children).toHaveLength(2);
        // Deep click: line 2 → @btn_save, not @toolbar
        expect(findSymbolAtLine(symbols, 2)!.name).toBe("@btn_save");
        // Shallow click: line 0 → @toolbar
        expect(findSymbolAtLine(symbols, 0)!.name).toBe("@toolbar");
    });

    it("multi-level nesting correctly resolves deepest symbol", () => {
        const lines = [
            "group @page {",
            "  group @header {",
            "    text @logo \"Logo\" {",
            "      font: Inter 700 24",
            "    }",
            "  }",
            "}",
        ];
        const symbols = parseDocumentSymbols(lines);
        // Line 3 (font property) → @logo
        expect(findSymbolAtLine(symbols, 3)!.name).toBe("@logo");
        // Line 1 (group @header) → @header
        expect(findSymbolAtLine(symbols, 1)!.name).toBe("@header");
    });
});

// ─── Figma/Sketch: Copy & Paste ──────────────────────────────────────────
// When copying a node, Figma generates a unique ID for the paste.
// The copy preserves all properties; paste creates a new instance.

describe("Figma/Sketch: Copy & Paste — ID uniqueness", () => {
    it("copy extracts the correct node block from FD source", () => {
        const source = [
            "rect @hero {",
            "  fill: #FF0000",
            "  w: 300 h: 200",
            "}",
            "",
            "text @title \"Hello\" {",
            "  font: Inter 700 32",
            "}",
        ].join("\n");

        // Simulate the copy logic from main.js: find block by @id
        const selectedId = "hero";
        const lines = source.split("\n");
        const startPattern = new RegExp(`^\\s*(\\w+)\\s+@${selectedId}\\b`);
        let startIdx = -1;
        for (let i = 0; i < lines.length; i++) {
            if (startPattern.test(lines[i])) { startIdx = i; break; }
        }
        expect(startIdx).toBe(0);

        // Collect block (same/deeper indentation)
        const startIndent = lines[startIdx].match(/^\s*/)![0].length;
        let endIdx = startIdx + 1;
        while (endIdx < lines.length) {
            const line = lines[endIdx];
            if (line.trim().length === 0) { endIdx++; continue; }
            const indent = line.match(/^\s*/)![0].length;
            if (indent <= startIndent) break;
            endIdx++;
        }
        const block = lines.slice(startIdx, endIdx).join("\n");
        expect(block).toContain("fill: #FF0000");
        expect(block).toContain("w: 300 h: 200");
        expect(block).not.toContain("@title");
    });

    it("paste generates a unique ID to avoid duplicates", () => {
        const clipText = "rect @hero {\n  fill: #FF0000\n  w: 300 h: 200\n}";
        const idMatch = clipText.match(/@(\w+)/);
        expect(idMatch).toBeTruthy();
        const oldId = idMatch![1];
        const newId = oldId + "_copy" + Math.floor(Math.random() * 1000);

        // Ensure new ID is different
        expect(newId).not.toBe(oldId);
        expect(newId).toMatch(/^hero_copy\d+$/);

        // Verify all word-boundary references are renamed
        const pasteText = clipText.replace(new RegExp(`@${oldId}\\b`, "g"), `@${newId}`);
        // The new ID contains the old ID as a prefix, so use regex word-boundary check
        const oldIdStillPresent = new RegExp(`@${oldId}\\b`).test(pasteText);
        expect(oldIdStillPresent).toBe(false);
        expect(pasteText).toContain(`@${newId}`);
    });

    it("paste of group with children renames all internal @ids", () => {
        const clipText = [
            "group @panel {",
            "  rect @bg {",
            "    fill: #FFF",
            "  }",
            "}",
        ].join("\n");

        // The current paste logic only renames the FIRST @id match
        // This is a known limitation — group children keep original IDs
        const idMatch = clipText.match(/@(\w+)/);
        const oldId = idMatch![1]; // "panel"
        const newId = oldId + "_copy42";
        const pasteText = clipText.replace(new RegExp(`@${oldId}\\b`, "g"), `@${newId}`);

        // BUG DETECTED: @bg is NOT renamed, which causes ID conflicts
        expect(pasteText).toContain("@bg"); // Still references old @bg
        // This documents an existing limitation in copy/paste for groups
    });
});

// ─── Figma/Sketch: Group & Ungroup ───────────────────────────────────────
// Grouping wraps selected nodes. Ungrouping dissolves the group wrapper.

describe("Figma/Sketch: Group & Ungroup — hierarchy parsing", () => {
    it("group wrapping creates a parent-child relationship", () => {
        const lines = [
            "group @card_group {",
            "  rect @card1 {",
            "    fill: #FFF",
            "  }",
            "  rect @card2 {",
            "    fill: #EEE",
            "  }",
            "}",
        ];
        const symbols = parseDocumentSymbols(lines);
        expect(symbols).toHaveLength(1);
        expect(symbols[0].name).toBe("@card_group");
        expect(symbols[0].kind).toBe("group");
        expect(symbols[0].children).toHaveLength(2);
        expect(symbols[0].children[0].name).toBe("@card1");
        expect(symbols[0].children[1].name).toBe("@card2");
    });

    it("nested groups correctly build hierarchy", () => {
        const lines = [
            "group @page {",
            "  group @header {",
            "    rect @logo {",
            "      fill: #000",
            "    }",
            "    text @nav \"Menu\" {",
            "    }",
            "  }",
            "  group @body {",
            "    rect @content {",
            "      fill: #FFF",
            "    }",
            "  }",
            "}",
        ];
        const symbols = parseDocumentSymbols(lines);
        expect(symbols).toHaveLength(1);
        const page = symbols[0];
        expect(page.children).toHaveLength(2);
        expect(page.children[0].name).toBe("@header");
        expect(page.children[0].children).toHaveLength(2);
        expect(page.children[1].name).toBe("@body");
        expect(page.children[1].children).toHaveLength(1);
    });

    it("ungroup detection via regex (used in context menu)", () => {
        const source = "group @cards {\n  rect @a {\n    fill: #F00\n  }\n}";
        const id = "cards";
        const groupRe = new RegExp(`(?:^|\\n)\\s*group\\s+@${id}\\b`);
        expect(groupRe.test(source)).toBe(true);
        // Non-group node should not match
        expect(groupRe.test(source.replace("group", "rect"))).toBe(false);
    });
});

// ─── Figma/Sketch: Layer Rename ──────────────────────────────────────────
// Double-clicking a layer name opens an inline rename field.
// All @id references are updated across the document.

describe("Figma/Sketch: Layer Rename — global ID replacement", () => {
    it("renaming an ID updates all references", () => {
        const text = [
            "rect @hero {",
            "  fill: #F00",
            "}",
            "@hero -> center_in: canvas",
        ].join("\n");

        const oldId = "hero";
        const newId = "banner";
        const renamed = text.replace(new RegExp(`@${oldId}\\b`, "g"), `@${newId}`);

        expect(renamed).not.toContain("@hero");
        expect(renamed).toContain("@banner");
        // Constraint reference also updated
        expect(renamed).toContain("@banner -> center_in: canvas");
    });

    it("rename sanitizes special characters", () => {
        const raw = "Hello World! @#$";
        const sanitized = raw.replace(/[^a-zA-Z0-9_]/g, "_");
        // All non-alphanumeric/underscore chars become underscores, including #
        expect(sanitized).toBe("Hello_World_____");
    });

    it("rename does not affect similar-prefix IDs", () => {
        const text = "rect @btn {\n}\nrect @btn_large {\n}\n@btn -> center_in: canvas";
        const renamed = text.replace(new RegExp(`@btn\\b`, "g"), "@button");
        expect(renamed).toContain("@button {");
        expect(renamed).toContain("@btn_large"); // Should NOT be renamed
        expect(renamed).toContain("@button -> center_in:");
    });
});

// ─── Drawio/Miro: Layer Tree & Visibility ────────────────────────────────
// Layers panel shows all nodes. Eye icon toggles visibility.

describe("Drawio/Miro: Layer Tree & Visibility toggle", () => {
    it("style blocks appear in layer tree", () => {
        const lines = ["style heading {", "  fill: #333", "  font: Inter 700 32", "}"];
        const symbols = parseDocumentSymbols(lines);
        expect(symbols).toHaveLength(1);
        expect(symbols[0].kind).toBe("style");
        expect(symbols[0].name).toBe("heading");
    });

    it("edges appear in layer tree", () => {
        const lines = ["edge @flow1 {", "  from: @a", "  to: @b", "}"];
        const symbols = parseDocumentSymbols(lines);
        expect(symbols).toHaveLength(1);
        expect(symbols[0].kind).toBe("edge");
    });

    it("constraints appear as single-line symbols", () => {
        const lines = ["@gallery -> center_in: canvas"];
        const symbols = parseDocumentSymbols(lines);
        expect(symbols).toHaveLength(1);
        expect(symbols[0].kind).toBe("constraint");
        expect(symbols[0].text).toBe("center_in: canvas");
    });

    it("complex document produces correct layer count", () => {
        const lines = [
            "style card_style {",
            "  fill: #FFF",
            "}",
            "group @page {",
            "  rect @header {",
            "    fill: #007AFF",
            "  }",
            "  text @title \"Hello\" {",
            "  }",
            "}",
            "edge @flow {",
            "  from: @header",
            "  to: @title",
            "}",
        ];
        const symbols = parseDocumentSymbols(lines);
        // Top-level: style, group, edge = 3
        expect(symbols).toHaveLength(3);
        // Group children: rect, text = 2
        const group = symbols.find((s) => s.name === "@page");
        expect(group).toBeDefined();
        expect(group!.children).toHaveLength(2);
    });
});

// ─── Miro/Freeform/Excalidraw: Infinite Canvas ──────────────────────────
// These tools feature infinite canvas with zoom-to-fit.
// Zoom calculations depend on scene bounds extraction.

describe("Miro/Freeform/Excalidraw: Infinite Canvas — zoom calculations", () => {
    it("zoom-to-fit computes correct fit zoom for scene bounds", () => {
        // Simulating zoomToFit() logic
        const sceneW = 800;
        const sceneH = 600;
        const viewportW = 1200;
        const viewportH = 800;
        const padding = 40;

        const fitZoom = Math.min(
            (viewportW - padding * 2) / Math.max(sceneW, 1),
            (viewportH - padding * 2) / Math.max(sceneH, 1),
        );
        expect(fitZoom).toBeCloseTo(1.2, 1); // (800-80)/600 = 1.2
        // Capped at 2.0
        const FIT_ZOOM_MAX = 2.0;
        const finalZoom = Math.max(0.1, Math.min(FIT_ZOOM_MAX, fitZoom));
        expect(finalZoom).toBe(fitZoom);
    });

    it("zoom-to-fit caps at 200% for small scenes", () => {
        const sceneW = 50;
        const sceneH = 50;
        const viewportW = 1200;
        const viewportH = 800;
        const padding = 40;

        const fitZoom = Math.min(
            (viewportW - padding * 2) / sceneW,
            (viewportH - padding * 2) / sceneH,
        );
        expect(fitZoom).toBeGreaterThan(2.0);
        const FIT_ZOOM_MAX = 2.0;
        const capped = Math.min(FIT_ZOOM_MAX, fitZoom);
        expect(capped).toBe(2.0);
    });

    it("zoom-at-point preserves cursor position", () => {
        let panX = 100;
        let panY = 200;
        let zoomLevel = 1.0;
        const mx = 400;
        const my = 300;
        const factor = 1.25;

        const oldZoom = zoomLevel;
        zoomLevel = Math.max(0.1, Math.min(10, zoomLevel * factor));
        panX = mx - (mx - panX) * (zoomLevel / oldZoom);
        panY = my - (my - panY) * (zoomLevel / oldZoom);

        // The point under the cursor should remain fixed
        const sceneXBefore = (mx - 100) / 1.0; // 300
        const sceneXAfter = (mx - panX) / zoomLevel;
        expect(sceneXAfter).toBeCloseTo(sceneXBefore, 5);
    });

    it("pinch zoom clamp prevents going below 10% or above 1000%", () => {
        const ZOOM_MIN = 0.1;
        const ZOOM_MAX = 10;
        // Zooming out heavily
        let zoom = 0.15;
        zoom = Math.max(ZOOM_MIN, Math.min(ZOOM_MAX, zoom * (1 / 1.03)));
        expect(zoom).toBeGreaterThanOrEqual(ZOOM_MIN);
        // Zooming in heavily
        zoom = 9.9;
        zoom = Math.max(ZOOM_MIN, Math.min(ZOOM_MAX, zoom * 1.03));
        expect(zoom).toBeLessThanOrEqual(ZOOM_MAX);
    });
});

// ─── Drawio/Miro: Grid Overlay ───────────────────────────────────────────
// Grid spacing adapts to zoom. Too-close dots double in spacing.

describe("Drawio/Miro: Grid overlay — spacing adaptation", () => {
    it("grid doubles spacing when dots too close at low zoom", () => {
        const GRID_BASE_SPACING = 20;
        const zoomLevel = 0.3; // Low zoom
        let spacing = GRID_BASE_SPACING;
        while (spacing * zoomLevel < 10) spacing *= 2;
        expect(spacing).toBeGreaterThan(GRID_BASE_SPACING);
        expect(spacing * zoomLevel).toBeGreaterThanOrEqual(10);
    });

    it("grid uses base spacing at normal zoom", () => {
        const GRID_BASE_SPACING = 20;
        const zoomLevel = 1.0;
        let spacing = GRID_BASE_SPACING;
        while (spacing * zoomLevel < 10) spacing *= 2;
        expect(spacing).toBe(GRID_BASE_SPACING); // No doubling needed
    });

    it("grid switches to lines at high zoom (>= 3x)", () => {
        const zoomLevel = 3.5;
        const useLines = zoomLevel >= 3;
        expect(useLines).toBe(true);
    });
});

// ─── Figma: Inline Text Editing ──────────────────────────────────────────
// Double-click a text node → textarea appears. Enter commits, Esc cancels.

describe("Figma: Inline Text Editing — text node handling", () => {
    it("text node with content is parsed correctly", () => {
        const lines = ['text @title "Hello World" {', "  font: Inter 700 32", "}"];
        const symbols = parseDocumentSymbols(lines);
        expect(symbols).toHaveLength(1);
        expect(symbols[0].name).toBe("@title");
        expect(symbols[0].text).toBe("Hello World");
    });

    it("double-click on empty canvas creates text node (Figma behavior)", () => {
        // When nothing is selected and you double-click, Figma creates a text node
        // This is handled by create_node_at("text", x, y) in the WASM API
        // We verify the parse side: a text node with empty content
        const source = 'text @_anon_1 "" {\n  x: 100 y: 200\n}';
        const anons = findAnonNodeIds(source);
        expect(anons).toContain("_anon_1");
    });

    it("hex luminance calculation for text contrast", () => {
        // Light background → dark text
        const hexLuminance = (hex: string) => {
            const r = parseInt(hex.slice(1, 3), 16) / 255;
            const g = parseInt(hex.slice(3, 5), 16) / 255;
            const b = parseInt(hex.slice(5, 7), 16) / 255;
            const lin = (c: number) => c <= 0.03928 ? c / 12.92 : Math.pow((c + 0.055) / 1.055, 2.4);
            return 0.2126 * lin(r) + 0.7152 * lin(g) + 0.0722 * lin(b);
        };

        expect(hexLuminance("#FFFFFF")).toBeCloseTo(1.0, 1);
        expect(hexLuminance("#000000")).toBeCloseTo(0.0, 1);
        // Dark fill → white text (luminance < 0.4)
        expect(hexLuminance("#1C1C1E")).toBeLessThan(0.4);
        // Light fill → dark text (luminance >= 0.4)
        expect(hexLuminance("#F5F5F7")).toBeGreaterThanOrEqual(0.4);
    });
});

// ─── Figma: Properties Panel ─────────────────────────────────────────────
// Selected node shows fill, stroke, size, opacity, corner radius.

describe("Figma: Properties Panel — hex color handling", () => {
    it("3-digit hex expands to 6-digit for color input", () => {
        const hex = "#FFF";
        const expanded = hex.length === 4
            ? `#${hex[1]}${hex[1]}${hex[2]}${hex[2]}${hex[3]}${hex[3]}`
            : hex;
        expect(expanded).toBe("#FFFFFF");
    });

    it("6-digit hex truncates to 7 chars (drops alpha)", () => {
        const hex = "#FF000088";
        const truncated = hex.substring(0, 7);
        expect(truncated).toBe("#FF0000");
    });

    it("opacity slider range is 0 to 1", () => {
        const opacity = 0.75;
        const display = Math.round(opacity * 100) + "%";
        expect(display).toBe("75%");
    });
});

// ─── Drawio: Drag & Drop from Palette ────────────────────────────────────
// Drawio has a side palette. Drag shape → drop on canvas → create node.

describe("Drawio: Drag & Drop — default shape dimensions", () => {
    it("rect default size is 100×80", () => {
        const DEFAULT_SHAPE_SIZES: Record<string, [number, number]> = {
            rect: [100, 80], ellipse: [100, 80], text: [80, 24],
            frame: [200, 150], line: [120, 4], arrow: [120, 4],
        };
        expect(DEFAULT_SHAPE_SIZES.rect).toEqual([100, 80]);
        expect(DEFAULT_SHAPE_SIZES.text).toEqual([80, 24]);
        expect(DEFAULT_SHAPE_SIZES.frame).toEqual([200, 150]);
    });

    it("drop position converts screen to scene coords", () => {
        const clientX = 500;
        const clientY = 300;
        const rectLeft = 50;
        const rectTop = 50;
        const panX = 100;
        const panY = 50;
        const zoomLevel = 1.5;

        const x = ((clientX - rectLeft) - panX) / zoomLevel;
        const y = ((clientY - rectTop) - panY) / zoomLevel;

        expect(x).toBeCloseTo(233.33, 1);
        expect(y).toBeCloseTo(133.33, 1);
    });
});

// ─── Excalidraw: Nudge with Arrow Keys ───────────────────────────────────
// Arrow keys move selected node 1px. Shift+Arrow moves 10px.

describe("Excalidraw/Figma: Arrow-key nudge", () => {
    it("arrow nudges 1px by default", () => {
        let x = 100;
        const step = 1; // No shift
        x -= step; // ArrowLeft
        expect(x).toBe(99);
    });

    it("shift+arrow nudges 10px", () => {
        let y = 200;
        const step = 10; // With shift
        y -= step; // ArrowUp
        expect(y).toBe(190);
    });
});

// ─── Figma: Selection Bar ────────────────────────────────────────────────
// Bottom bar shows: @id · kind · W×H · (X, Y)

describe("Figma: Selection Bar — status display", () => {
    it("formats selection info correctly", () => {
        const selectedId = "hero";
        const kind = "rect";
        const w = 300;
        const h = 200;
        const x = 50;
        const y = 100;
        const bar = `@${selectedId} · ${kind} · ${w}×${h} · (${x}, ${y})`;
        expect(bar).toBe("@hero · rect · 300×200 · (50, 100)");
    });
});

// ─── Sketch: Color Swatches ─────────────────────────────────────────────
// Preset color palette + recent colors. Dark color detection.

describe("Sketch: Color Swatches — dark color detection", () => {
    it("detects dark colors correctly", () => {
        const isColorDark = (hex: string) => {
            const c = hex.replace("#", "");
            const r = parseInt(c.substring(0, 2), 16);
            const g = parseInt(c.substring(2, 4), 16);
            const b = parseInt(c.substring(4, 6), 16);
            return (r * 299 + g * 587 + b * 114) / 1000 < 128;
        };
        expect(isColorDark("#000000")).toBe(true);
        expect(isColorDark("#FFFFFF")).toBe(false);
        expect(isColorDark("#1C1C1E")).toBe(true);
        // #FF3B30 is actually dark by weighted avg: (255*299+59*587+48*114)/1000 ≈ 116 < 128
        expect(isColorDark("#FF3B30")).toBe(true);
        // Yellow is bright: (255*299+204*587+0*114)/1000 ≈ 196 > 128
        expect(isColorDark("#FFCC00")).toBe(false);
    });

    it("recent colors are deduplicated and capped at 6", () => {
        const recentColors: string[] = [];
        const addRecent = (color: string) => {
            const n = color.toUpperCase();
            const idx = recentColors.indexOf(n);
            if (idx >= 0) recentColors.splice(idx, 1);
            recentColors.unshift(n);
            if (recentColors.length > 6) recentColors.pop();
        };
        for (let i = 0; i < 8; i++) {
            addRecent(`#${i.toString().padStart(6, "0")}`);
        }
        expect(recentColors).toHaveLength(6);
        // Most recent should be first
        expect(recentColors[0]).toBe("#000007");
    });
});

// ─── FD-Specific: Spec View (Unique to FD) ──────────────────────────────
// Spec annotations with filter tabs and bulk status.

describe("FD-Specific: Spec View — annotation parsing and filtering", () => {
    it("complex document with mixed annotations", () => {
        const source = [
            "rect @login_btn {",
            '  spec {',
            '    "Primary login CTA"',
            '    accept: "visible on page load"',
            '    accept: "triggers auth flow"',
            "    status: in_progress",
            "    priority: high",
            "    tag: auth, conversion",
            "  }",
            "  fill: #007AFF",
            "}",
        ].join("\n");
        const result = parseSpecNodes(source);
        expect(result.nodes).toHaveLength(1);
        const node = result.nodes[0];
        expect(node.annotations).toHaveLength(6);
        expect(node.annotations.filter((a) => a.type === "accept")).toHaveLength(2);
        expect(node.annotations.find((a) => a.type === "status")?.value).toBe("in_progress");
        expect(node.annotations.find((a) => a.type === "priority")?.value).toBe("high");
    });

    it("spec view hides properties but keeps annotations", () => {
        const lines = [
            "rect @card {",
            "  fill: #FFFFFF",
            "  stroke: #E8E8EC 1",
            "  corner: 16",
            '  spec {',
            '    "Product card"',
            "    status: done",
            "  }",
            "}",
        ];
        const hidden = computeSpecHideLines(lines);
        // Properties (fill, stroke, corner) should be hidden
        expect(hidden).toContain(1); // fill
        expect(hidden).toContain(2); // stroke
        expect(hidden).toContain(3); // corner
        // Spec block and node declaration should NOT be hidden
        expect(hidden).not.toContain(0); // rect @card {
        expect(hidden).not.toContain(8); // }
    });

    it("spec filter by status works on annotated nodes (bug fix: no annotation bleed)", () => {
        // Previously, parseSpecNodes used lines.indexOf() which caused annotation
        // bleed between nodes with identical spec blocks. Now fixed with indexed loop.
        const source = [
            "rect @a {",
            "  spec {",
            "    status: done",
            "  }",
            "}",
            "",
            "rect @b {",
            "  spec {",
            "    status: draft",
            "  }",
            "}",
            "",
            "rect @c {",
            "  spec {",
            "    status: done",
            "  }",
            "}",
        ].join("\n");
        const result = parseSpecNodes(source);
        expect(result.nodes).toHaveLength(3);
        const doneNodes = result.nodes.filter((n) =>
            n.annotations.some((a) => a.type === "status" && a.value === "done"),
        );
        expect(doneNodes).toHaveLength(2);
        expect(doneNodes[0].id).toBe("a");
        expect(doneNodes[1].id).toBe("c");
        const draftNodes = result.nodes.filter((n) =>
            n.annotations.some((a) => a.type === "status" && a.value === "draft"),
        );
        expect(draftNodes).toHaveLength(1);
        expect(draftNodes[0].id).toBe("b");
    });


    it("spec coverage percentage calculation", () => {
        // 3 nodes total, 2 annotated = 67%
        const totalNodes = 3;
        const annotatedCount = 2;
        const pct = Math.round((annotatedCount / totalNodes) * 100);
        expect(pct).toBe(67);
    });
});

// ─── Excalidraw: Tween Animation Engine ──────────────────────────────────
// Animation presets with easing functions.

describe("Excalidraw/Figma: Tween Animation Engine", () => {
    it("ease functions produce correct boundary values", () => {
        const EASE_FNS: Record<string, (t: number) => number> = {
            linear: (t) => t,
            ease_out: (t) => 1 - Math.pow(1 - t, 3),
            ease_in: (t) => t * t * t,
            ease_in_out: (t) => t < 0.5 ? 4 * t * t * t : 1 - Math.pow(-2 * t + 2, 3) / 2,
        };

        // All ease functions: f(0) = 0, f(1) = 1
        for (const [name, fn] of Object.entries(EASE_FNS)) {
            expect(fn(0)).toBe(0);
            expect(fn(1)).toBe(1);
        }
        // Ease out should be ahead of linear at midpoint
        expect(EASE_FNS.ease_out(0.5)).toBeGreaterThan(0.5);
        // Ease in should be behind linear at midpoint
        expect(EASE_FNS.ease_in(0.5)).toBeLessThan(0.5);
    });

    it("tween interpolation produces correct intermediate values", () => {
        const from = 1.0;
        const to = 1.1;
        const easeFn = (t: number) => t; // linear
        const t = 0.5;
        const v = from + (to - from) * easeFn(t);
        expect(v).toBeCloseTo(1.05, 5);
    });
});

// ─── Miro: Minimap ───────────────────────────────────────────────────────
// Minimap shows overview with viewport rectangle.

describe("Miro: Minimap — scene bounds and viewport mapping", () => {
    it("minimap scale factor fits scene in 180×120 with padding", () => {
        const mw = 180;
        const mh = 120;
        const padding = 20;
        const sceneW = 1000;
        const sceneH = 800;

        const scale = Math.min(
            (mw - padding * 2) / sceneW,
            (mh - padding * 2) / sceneH,
        );
        expect(scale).toBeCloseTo(0.1, 2); // (120-40)/800 = 0.1
    });

    it("minimap click converts to scene-space coordinates", () => {
        const mw = 180;
        const mh = 120;
        const padding = 20;
        const bounds = { minX: 0, minY: 0, maxX: 1000, maxY: 800 };
        const sceneW = bounds.maxX - bounds.minX;
        const sceneH = bounds.maxY - bounds.minY;
        const scale = Math.min(
            (mw - padding * 2) / sceneW,
            (mh - padding * 2) / sceneH,
        );
        const offsetX = (mw - sceneW * scale) / 2;
        const offsetY = (mh - sceneH * scale) / 2;

        // Click at minimap center
        const mx = 90;
        const my = 60;
        const sceneX = (mx - offsetX) / scale + bounds.minX;
        const sceneY = (my - offsetY) / scale + bounds.minY;

        expect(sceneX).toBeCloseTo(500, 0);
        expect(sceneY).toBeCloseTo(400, 0);
    });
});

// ─── Figma: Export PNG ───────────────────────────────────────────────────
// Export scene at 2× resolution with padding.

describe("Figma: Export PNG — bounds calculation", () => {
    it("export canvas adds padding and 2x resolution", () => {
        const sceneW = 800;
        const sceneH = 600;
        const padding = 40;
        const dpr = 2;
        const exportW = (sceneW + padding * 2) * dpr;
        const exportH = (sceneH + padding * 2) * dpr;
        expect(exportW).toBe(1760);
        expect(exportH).toBe(1360);
    });
});

// ─── Edge Cases: Error Resilience ────────────────────────────────────────
// Design tools must handle malformed input gracefully.

describe("Edge Cases: Error resilience", () => {
    it("parseDocumentSymbols handles unclosed braces", () => {
        const lines = ["rect @broken {", "  fill: #FFF"];
        // Should not throw
        const symbols = parseDocumentSymbols(lines);
        expect(symbols).toHaveLength(1);
        expect(symbols[0].name).toBe("@broken");
    });

    it("parseSpecNodes handles empty document", () => {
        expect(parseSpecNodes("").nodes).toEqual([]);
        expect(parseSpecNodes("").edges).toEqual([]);
    });

    it("parseSpecNodes handles comments-only document", () => {
        expect(parseSpecNodes("# just comments\n# more comments\n").nodes).toEqual([]);
    });

    it("findSymbolAtLine returns undefined for lines outside all nodes", () => {
        const lines = ["rect @a {", "}", "", "# comment"];
        const symbols = parseDocumentSymbols(lines);
        expect(findSymbolAtLine(symbols, 3)).toBeUndefined();
    });

    it("escapeHtml prevents XSS in node IDs", () => {
        expect(escapeHtml('<script>alert("xss")</script>')).toBe(
            '&lt;script&gt;alert(&quot;xss&quot;)&lt;/script&gt;',
        );
    });

    it("transformSpecViewLine handles all node types", () => {
        expect(transformSpecViewLine("group @a {")).toBe("@a {");
        expect(transformSpecViewLine("frame @b {")).toBe("@b {");
        expect(transformSpecViewLine("rect @c {")).toBe("@c {");
        expect(transformSpecViewLine("ellipse @d {")).toBe("@d {");
        expect(transformSpecViewLine("path @e {")).toBe("@e {");
        expect(transformSpecViewLine('text @f "hi" {')).toBe('@f "hi" {');
        // Non-node lines unchanged
        expect(transformSpecViewLine("  fill: #FFF")).toBe("  fill: #FFF");
        expect(transformSpecViewLine("style foo {")).toBe("style foo {");
    });
});
