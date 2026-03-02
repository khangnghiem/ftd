---
description: Autonomous overnight QA — exhaustive human behavior + accessibility testing, fix as you go
---

# QA Workflow

> Autonomous overnight QA session. Systematically tests every human interaction pattern,
> accessibility behavior, and edge case on the FD Canvas — inspired by Figma, Sketch,
> Miro, Freeform, and Excalidraw. Fixes bugs inline before moving on.
> Time budget: ~6–8 hours. **195+ checks across 33 phases.**

// turbo-all

---

## Prerequisites

1. **Sync + branch**:

   ```bash
   git fetch origin main
   git checkout -b test/qa-$(date +%Y%m%d)
   ```

2. **Rust CI must pass first**:

   ```bash
   cargo check --workspace && cargo test --workspace && cargo clippy --workspace -- -D warnings
   ```

3. **Build WASM**:

   ```bash
   wasm-pack build crates/fd-wasm --target web --out-dir ../../fd-vscode/webview/wasm
   ```

4. **Open Codespace** via browser subagent (reuse existing tab if open):

   ```
   Navigate to: https://github.com/codespaces
   Click on the available codespace for khangnghiem/fast-draft
   ```

5. Maintain ≤2 editor panels. Open `examples/demo.fd` → activate Design View.

---

## Fix-As-You-Go Rule

> [!IMPORTANT]
> When a check **fails**, do NOT just log it — fix it immediately:
>
> 1. Screenshot the failure
> 2. Document expected vs actual
> 3. Fix the bug (edit Rust/JS/CSS as needed)
> 4. Re-run `cargo check && cargo test && cargo clippy`
> 5. Rebuild WASM if Rust changed
> 6. Re-test the failing check → confirm PASS
> 7. Continue to next check
>
> Commit fixes periodically (every ~5 fixes) so progress isn't lost.

---

## Phase 1: Cold Start & First Impression (5 checks)

| #   | Action                                                    | Expected Result                            |
| --- | --------------------------------------------------------- | ------------------------------------------ |
| 1.1 | Open `examples/demo.fd` → Toggle Design View              | Canvas renders with shapes visible         |
| 1.2 | Open `examples/onboarding.fd` → Toggle Design View        | Complex nested file renders correctly      |
| 1.3 | Open `examples/landing_page.fd` → Toggle Design View      | Multi-component page renders               |
| 1.4 | Open `examples/constraints.fd` → verify constraint layout | Constraint-based positions resolve         |
| 1.5 | Open `examples/animations.fd` → verify animation nodes    | Animation declarations parse, nodes render |

---

## Phase 2: Drawing Tools — Basic (7 checks)

| #   | Action                                       | Expected Result                              |
| --- | -------------------------------------------- | -------------------------------------------- |
| 2.1 | Press R → drag small rect (50×50)            | Rect appears with default fill/stroke/corner |
| 2.2 | Press R → drag large rect (400×200)          | Scales correctly, no visual artifacts        |
| 2.3 | Press O → drag ellipse                       | Ellipse appears with default fill/stroke     |
| 2.4 | Press T → click canvas → type "Hello" → blur | Text node created, code appears              |
| 2.5 | Press A → drag from node A to node B         | Arrow/edge connects them                     |
| 2.6 | Press P → draw freehand path                 | Smooth pen path renders                      |
| 2.7 | Press F → drag frame                         | Frame container created with clip behavior   |

---

## Phase 3: Drawing Tool Modifiers (8 checks)

| #   | Action                                 | Expected Result                     |
| --- | -------------------------------------- | ----------------------------------- |
| 3.1 | R + Shift+drag                         | Perfect square (constrained aspect) |
| 3.2 | O + Shift+drag                         | Perfect circle                      |
| 3.3 | R + Alt+drag from existing node        | Clone + drag                        |
| 3.4 | While in Rect tool, ⌘+click node       | Temporary Select (can move it)      |
| 3.5 | While in Rect tool, ⌘+drag empty space | Marquee select (temp Select)        |
| 3.6 | While in Rect tool, Space+drag         | Canvas pans                         |
| 3.7 | R + Shift+Alt+drag                     | Constrained clone                   |
| 3.8 | R + start drag → press Escape mid-drag | Shape NOT created (cancelled)       |

---

## Phase 4: Tool Locking / Sticky Mode (5 checks)

| #   | Action                              | Expected Result               |
| --- | ----------------------------------- | ----------------------------- |
| 4.1 | Press R twice quickly (R R)         | Tool locks (🔒 indicator)     |
| 4.2 | While locked → draw 3 rects         | Tool stays on R after each    |
| 4.3 | While locked → press V              | Lock releases, back to Select |
| 4.4 | While locked → press Escape         | Lock releases                 |
| 4.5 | Double-click Rect button in toolbar | Tool locks (same as R R)      |

---

## Phase 5: Toolbar Drag-to-Create (6 checks)

| #   | Action                                           | Expected Result                                |
| --- | ------------------------------------------------ | ---------------------------------------------- |
| 5.1 | Drag Rect FROM toolbar onto empty canvas         | Ghost preview (dashed) → shape created at drop |
| 5.2 | Drag Ellipse FROM toolbar onto empty canvas      | Same ghost + create behavior                   |
| 5.3 | Drag Text FROM toolbar onto existing rect        | Text reparents inside rect, auto-centers       |
| 5.4 | Drag shape FROM toolbar → drop near node (≤40px) | Snap (20px gap) + auto-edge                    |
| 5.5 | Drag FROM toolbar → drop → immediately ⌘Z        | Shape removed (undo works)                     |
| 5.6 | Drag Text FROM toolbar near edge (≤30px)         | Text becomes edge label                        |

---

## Phase 6: Selection — Single Node (7 checks)

| #   | Action                               | Expected Result                           |
| --- | ------------------------------------ | ----------------------------------------- |
| 6.1 | Click a node                         | 8 blue resize handles appear              |
| 6.2 | Click empty space                    | Selection clears                          |
| 6.3 | Click node A → click node B          | Only B selected                           |
| 6.4 | Select node → Delete                 | Removed from canvas AND code              |
| 6.5 | Select node → Backspace              | Same as Delete                            |
| 6.6 | Click a group                        | Group selected (not child)                |
| 6.7 | Click group → click same group again | Drill-down into child (Figma deep-select) |

---

## Phase 7: Selection — Multi-Node (8 checks)

| #   | Action                               | Expected Result                        |
| --- | ------------------------------------ | -------------------------------------- |
| 7.1 | Drag on empty space                  | Marquee selection box appears          |
| 7.2 | Marquee over 3 nodes                 | All 3 selected (combined bounding box) |
| 7.3 | Shift+click A, Shift+click B         | Both selected (additive)               |
| 7.4 | Shift+click already-selected node    | Deselects (toggle)                     |
| 7.5 | ⌘A                                   | All nodes selected                     |
| 7.6 | Multi-select 3 → Delete              | All 3 removed                          |
| 7.7 | Multi-select → drag                  | All move together, same speed          |
| 7.8 | Marquee → hold Shift → marquee again | Adds to existing selection             |

---

## Phase 8: Move & Position (6 checks)

| #   | Action                        | Expected Result              |
| --- | ----------------------------- | ---------------------------- |
| 8.1 | Select → drag to new position | x/y update in code           |
| 8.2 | Select → Arrow Up             | Nudges 1px up                |
| 8.3 | Select → Arrow Right          | Nudges 1px right             |
| 8.4 | Select → Shift+Arrow Up       | Nudges 10px up               |
| 8.5 | Select → Shift+Arrow Right    | Nudges 10px right            |
| 8.6 | Drag node near another        | Snap/alignment guides appear |

---

## Phase 9: Resize (6 checks)

| #   | Action                       | Expected Result                           |
| --- | ---------------------------- | ----------------------------------------- |
| 9.1 | Drag BOTTOM-RIGHT handle     | Width/height increase, code updates       |
| 9.2 | Drag TOP-LEFT handle         | Position AND size change                  |
| 9.3 | Drag RIGHT-MIDDLE handle     | Only width changes                        |
| 9.4 | Drag BOTTOM-MIDDLE handle    | Only height changes                       |
| 9.5 | Hold Shift while resizing    | Proportional resize                       |
| 9.6 | Resize to very small (10×10) | Minimum size works, handles don't overlap |

---

## Phase 10: Copy / Paste / Duplicate (6 checks)

| #    | Action                   | Expected Result                         |
| ---- | ------------------------ | --------------------------------------- |
| 10.1 | ⌘C → ⌘V                  | Copy appears with NEW @id, offset +20px |
| 10.2 | ⌘X                       | Node removed from canvas                |
| 10.3 | ⌘V after cut             | Cut node pasted back with new @id       |
| 10.4 | ⌘D                       | Duplicate (same as ⌘C→⌘V)               |
| 10.5 | Multi-select 3 → ⌘C → ⌘V | All 3 copied with new @ids              |
| 10.6 | Alt+drag a node          | Clone created at drop position          |

---

## Phase 11: Undo / Redo Deep Dive (8 checks)

| #    | Action                 | Expected Result                      |
| ---- | ---------------------- | ------------------------------------ |
| 11.1 | Draw rect → ⌘Z         | Rect disappears from canvas AND code |
| 11.2 | After undo → ⌘⇧Z       | Rect reappears (redo)                |
| 11.3 | Move node → ⌘Z         | Returns to original position         |
| 11.4 | Delete node → ⌘Z       | Node restored                        |
| 11.5 | Change fill → ⌘Z       | Original color restored              |
| 11.6 | Group 2 nodes → ⌘Z     | Ungroups                             |
| 11.7 | 5 actions → ⌘Z ×5      | All 5 undone in reverse order        |
| 11.8 | After undo, new action | Redo stack cleared                   |

---

## Phase 12: Z-Order Operations (6 checks)

| #    | Action                                     | Expected Result           |
| ---- | ------------------------------------------ | ------------------------- |
| 12.1 | Create 3 overlapping rects (A, B, C)       | Layered visually in order |
| 12.2 | Select backmost → ⌘]                       | Moves forward one step    |
| 12.3 | Select backmost → ⌘⇧]                      | Jumps to front            |
| 12.4 | Select frontmost → ⌘[                      | Moves back one step       |
| 12.5 | Select frontmost → ⌘⇧[                     | Jumps to very back        |
| 12.6 | Verify Layers panel matches visual z-order | Order consistent          |

---

## Phase 13: Inline Text Editing (7 checks)

| #    | Action                               | Expected Result                         |
| ---- | ------------------------------------ | --------------------------------------- |
| 13.1 | Double-click text node               | Inline textarea opens with current text |
| 13.2 | Edit text → click away               | Text updates on canvas AND code         |
| 13.3 | Double-click → Escape                | Edit cancelled, original text preserved |
| 13.4 | Double-click → select all → type new | Full replacement works                  |
| 13.5 | Text with font/color → double-click  | Style preserved during editing          |
| 13.6 | Double-click text inside group       | Inline editor opens (no layout break)   |
| 13.7 | Edit text → make much longer         | Text node width adjusts                 |

---

## Phase 14: Text Reparenting (6 checks)

| #    | Action                           | Expected Result                |
| ---- | -------------------------------- | ------------------------------ |
| 14.1 | Drag text onto a rect            | Text reparents inside rect     |
| 14.2 | After reparent                   | Text auto-centered in rect     |
| 14.3 | Check code after reparent        | Text nested as child of rect   |
| 14.4 | Drag reparented text OUT of rect | Text detaches                  |
| 14.5 | After detach                     | Teal glow ring animation plays |
| 14.6 | Check code after detach          | Text back at top level         |

---

## Phase 15: Group & Ungroup (7 checks)

| #    | Action                               | Expected Result                    |
| ---- | ------------------------------------ | ---------------------------------- |
| 15.1 | Select 2 → right-click → Group       | Group wrapper created              |
| 15.2 | Check Layers panel                   | Shows group hierarchy              |
| 15.3 | Check code                           | Children nested inside group block |
| 15.4 | Select group → right-click → Ungroup | Group dissolved                    |
| 15.5 | After ungroup                        | Children are top-level nodes       |
| 15.6 | Select 3 → ⌘G                        | Keyboard shortcut groups           |
| 15.7 | Select group → ⌘⇧G                   | Keyboard shortcut ungroups         |

---

## Phase 16: Frame Operations (5 checks)

| #    | Action                                   | Expected Result                    |
| ---- | ---------------------------------------- | ---------------------------------- |
| 16.1 | Select 2 → right-click → Frame Selection | Frame container wraps them         |
| 16.2 | Check frame sizing                       | Independent size (not auto-shrink) |
| 16.3 | Draw nodes partially outside frame       | Frame clips children visually      |
| 16.4 | Move node out of frame                   | Detaches from frame in code        |
| 16.5 | Drag node into existing frame            | Reparents inside                   |

---

## Phase 17: Navigation — Pan & Zoom (10 checks)

| #     | Action                          | Expected Result              |
| ----- | ------------------------------- | ---------------------------- |
| 17.1  | Space+drag                      | Canvas pans (hand cursor)    |
| 17.2  | Release Space                   | Back to current tool         |
| 17.3  | ⌘+scroll up                     | Zoom in (centered on cursor) |
| 17.4  | ⌘+scroll down                   | Zoom out                     |
| 17.5  | Trackpad pinch                  | Smooth zoom                  |
| 17.6  | Click 100% in minimap zoom pill | Reset to 100%                |
| 17.7  | Click + in minimap zoom pill    | Zoom step in                 |
| 17.8  | Click - in minimap zoom pill    | Zoom step out                |
| 17.9  | ⌘0                              | Zoom-to-fit                  |
| 17.10 | ⌘1                              | Zoom-to-selection            |

---

## Phase 18: Minimap (4 checks)

| #    | Action                               | Expected Result                    |
| ---- | ------------------------------------ | ---------------------------------- |
| 18.1 | Verify minimap shows canvas overview | Overview of all content visible    |
| 18.2 | Drag viewport in minimap             | Canvas pans accordingly            |
| 18.3 | Zoom way out                         | Minimap viewport indicator grows   |
| 18.4 | Zoom way in                          | Minimap viewport indicator shrinks |

---

## Phase 19: Layers Panel (8 checks)

| #    | Action                        | Expected Result                      |
| ---- | ----------------------------- | ------------------------------------ |
| 19.1 | Press L                       | Layers panel toggles visibility      |
| 19.2 | Click layer item              | Node selects on canvas + camera pans |
| 19.3 | Double-click layer name       | Inline rename opens                  |
| 19.4 | Rename layer                  | @id updates in code                  |
| 19.5 | Click ⋮ on layer item         | Context menu opens                   |
| 19.6 | Drag layer items to reorder   | z-order changes on canvas            |
| 19.7 | Expand/collapse group in tree | Children show/hide                   |
| 19.8 | Verify hierarchy matches code | Layer tree = code nesting            |

---

## Phase 20: Properties Panel (8 checks)

| #    | Action               | Expected Result                               |
| ---- | -------------------- | --------------------------------------------- |
| 20.1 | Select rect          | Shows fill, stroke, w, h, x, y, corner radius |
| 20.2 | Change fill color    | Canvas updates immediately                    |
| 20.3 | Change stroke color  | Canvas + code update                          |
| 20.4 | Change width value   | Canvas + code update                          |
| 20.5 | Change corner radius | Visual + code update                          |
| 20.6 | Select ellipse       | Relevant props shown (no corner radius)       |
| 20.7 | Select text          | Shows font, font size, text fill              |
| 20.8 | Select nothing       | Panel empty or shows canvas-level info        |

---

## Phase 21: Context Menu (10 checks)

| #     | Action                             | Expected Result                     |
| ----- | ---------------------------------- | ----------------------------------- |
| 21.1  | Right-click single node            | Full context menu appears           |
| 21.2  | Right-click → Cut                  | Node removed                        |
| 21.3  | Right-click → Paste                | Pasted with new @id                 |
| 21.4  | Right-click → Copy                 | Node still exists (non-destructive) |
| 21.5  | Right-click → Duplicate            | Copy at +20px offset                |
| 21.6  | Right-click → Delete               | Removed from canvas + code          |
| 21.7  | Right-click → Copy as PNG          | No crash                            |
| 21.8  | Right-click → Bring to Front       | z-order change                      |
| 21.9  | Right-click → Send to Back         | z-order change                      |
| 21.10 | Right-click with <2 nodes selected | "Group" greyed out                  |

---

## Phase 22: Bidi Sync — Canvas → Code (6 checks)

| #    | Action                     | Expected Result                |
| ---- | -------------------------- | ------------------------------ |
| 22.1 | Draw rect on canvas        | FD code appears with @id, w, h |
| 22.2 | Move node on canvas        | x/y update in code             |
| 22.3 | Resize node on canvas      | w/h update in code             |
| 22.4 | Delete node on canvas      | Code block disappears          |
| 22.5 | Group nodes on canvas      | Code shows group nesting       |
| 22.6 | Change fill via Properties | `fill:` line updates in code   |

---

## Phase 23: Bidi Sync — Code → Canvas (6 checks)

| #    | Action                                            | Expected Result              |
| ---- | ------------------------------------------------- | ---------------------------- |
| 23.1 | Type `rect @manual_test { w: 100 h: 80 }` in code | Rect appears on canvas       |
| 23.2 | Edit `fill:` value in code                        | Canvas color updates         |
| 23.3 | Delete node's code block                          | Node disappears from canvas  |
| 23.4 | Add `x: 200 y: 300` in code                       | Position change on canvas    |
| 23.5 | Nest node inside group in code                    | Layers panel shows hierarchy |
| 23.6 | Add `corner: 20` in code                          | Visual corner radius update  |

---

## Phase 24: Zen Mode (4 checks)

| #    | Action                      | Expected Result                    |
| ---- | --------------------------- | ---------------------------------- |
| 24.1 | Click 🧘 toggle             | All panels hide except core tools  |
| 24.2 | Draw + select + move in Zen | Everything still works             |
| 24.3 | Press L in Zen              | Layers panel toggles independently |
| 24.4 | Click 🔧 toggle             | Full UI restored                   |

---

## Phase 25: Grid & Alignment (3 checks)

| #    | Action                  | Expected Result      |
| ---- | ----------------------- | -------------------- |
| 25.1 | Press G                 | Grid overlay toggles |
| 25.2 | With grid on, drag node | Snapping to grid     |
| 25.3 | Toggle grid off         | Grid disappears      |

---

## Phase 26: Keyboard Shortcuts — Full Sweep (12 checks)

| #     | Action | Expected Result                        |
| ----- | ------ | -------------------------------------- |
| 26.1  | V      | Select tool active (toolbar highlight) |
| 26.2  | R      | Rect tool active                       |
| 26.3  | O      | Ellipse tool active                    |
| 26.4  | T      | Text tool active                       |
| 26.5  | A      | Arrow tool active                      |
| 26.6  | P      | Pen tool active                        |
| 26.7  | F      | Frame tool active                      |
| 26.8  | Tab    | Toggles between last two tools         |
| 26.9  | ?      | Keyboard shortcuts help overlay        |
| 26.10 | 0      | Zoom resets to 100%                    |
| 26.11 | ⌘+     | Zoom in                                |
| 26.12 | ⌘-     | Zoom out                               |

---

## Phase 27: Theme (3 checks)

| #    | Action             | Expected Result                            |
| ---- | ------------------ | ------------------------------------------ |
| 27.1 | Click theme toggle | Canvas switches to dark mode               |
| 27.2 | Inspect all UI     | Panels, toolbar, minimap respect new theme |
| 27.3 | Toggle back        | Light mode restored cleanly                |

---

## Phase 28: Animations (3 checks)

| #    | Action                               | Expected Result              |
| ---- | ------------------------------------ | ---------------------------- |
| 28.1 | Open `examples/animations.fd`        | Animation declarations parse |
| 28.2 | Hover nodes with `:hover` animations | Hover state triggers         |
| 28.3 | Check `:enter` animations            | Play on canvas load          |

---

## Phase 29: Edge / Arrow (5 checks)

| #    | Action                         | Expected Result                    |
| ---- | ------------------------------ | ---------------------------------- |
| 29.1 | A → drag from node A to node B | Arrow/edge connects them           |
| 29.2 | Move node A                    | Edge endpoint follows              |
| 29.3 | Move node B                    | Other endpoint follows             |
| 29.4 | Delete an edge                 | Arrow removed from canvas AND code |
| 29.5 | Select edge → check Properties | Edge-specific properties shown     |

---

## Phase 30: Stress Tests (8 checks)

| #    | Action                                   | Expected Result                      |
| ---- | ---------------------------------------- | ------------------------------------ |
| 30.1 | Draw 20 shapes rapidly                   | No lag, no duplicates, all in code   |
| 30.2 | Select all → drag                        | Smooth movement (no jitter)          |
| 30.3 | ⌘Z rapidly 20 times                      | Undo stack doesn't crash             |
| 30.4 | ⌘V rapidly 10 times                      | 10 paste ops complete without errors |
| 30.5 | Zoom to 400% → draw → zoom out           | Shapes at correct positions          |
| 30.6 | Zoom to 10%                              | All content visible and selectable   |
| 30.7 | Cycle all 7 tools 3 times rapidly        | No stuck state                       |
| 30.8 | Open/close Layers panel 10 times rapidly | No UI glitch                         |

---

## Phase 31: Export & Clipboard (3 checks)

| #    | Action                             | Expected Result             |
| ---- | ---------------------------------- | --------------------------- |
| 31.1 | Select → right-click → Copy as PNG | Exports to clipboard        |
| 31.2 | ⌘A → right-click → Copy as PNG     | Full canvas export          |
| 31.3 | Verify PNG quality                 | Respects current zoom/scale |

---

## Phase 32: File Switching Stability (4 checks)

| #    | Action                           | Expected Result      |
| ---- | -------------------------------- | -------------------- |
| 32.1 | Open `demo.fd` → draw 2 shapes   | Shapes created       |
| 32.2 | Switch to `onboarding.fd`        | Canvas loads clean   |
| 32.3 | Switch back to `demo.fd`         | 2 shapes still there |
| 32.4 | Open 3 .fd tabs → switch between | No canvas corruption |

---

## Phase 33: Multimodal Workflows (5 checks)

| #    | Action                                                              | Expected Result                  |
| ---- | ------------------------------------------------------------------- | -------------------------------- |
| 33.1 | **Wireframe**: 2 input rects + 1 button + 3 labels → group → rename | Code structure correct           |
| 33.2 | **Flowchart**: 4 rects → connect with arrows                        | Edge connections in code         |
| 33.3 | **Annotation**: rect → reparent text inside → use as annotation     | Nesting correct                  |
| 33.4 | **Components**: Duplicate group 3× → arrange side by side           | Independent copies               |
| 33.5 | **Cleanup**: Select all → delete                                    | Canvas AND code completely empty |

---

## Reporting

After ALL 33 phases, produce a summary report:

```
Phase 01: Cold Start              ✅/⚠️/❌ X/5
Phase 02: Drawing Tools Basic      ✅/⚠️/❌ X/7
Phase 03: Drawing Tool Modifiers   ✅/⚠️/❌ X/8
Phase 04: Tool Locking             ✅/⚠️/❌ X/5
Phase 05: Toolbar Drag-to-Create   ✅/⚠️/❌ X/6
Phase 06: Selection Single         ✅/⚠️/❌ X/7
Phase 07: Selection Multi          ✅/⚠️/❌ X/8
Phase 08: Move & Position          ✅/⚠️/❌ X/6
Phase 09: Resize                   ✅/⚠️/❌ X/6
Phase 10: Copy/Paste/Duplicate     ✅/⚠️/❌ X/6
Phase 11: Undo/Redo               ✅/⚠️/❌ X/8
Phase 12: Z-Order                  ✅/⚠️/❌ X/6
Phase 13: Inline Text Editing      ✅/⚠️/❌ X/7
Phase 14: Text Reparenting         ✅/⚠️/❌ X/6
Phase 15: Group/Ungroup            ✅/⚠️/❌ X/7
Phase 16: Frame Operations         ✅/⚠️/❌ X/5
Phase 17: Navigation               ✅/⚠️/❌ X/10
Phase 18: Minimap                  ✅/⚠️/❌ X/4
Phase 19: Layers Panel             ✅/⚠️/❌ X/8
Phase 20: Properties Panel         ✅/⚠️/❌ X/8
Phase 21: Context Menu             ✅/⚠️/❌ X/10
Phase 22: Bidi Sync Canvas→Code    ✅/⚠️/❌ X/6
Phase 23: Bidi Sync Code→Canvas    ✅/⚠️/❌ X/6
Phase 24: Zen Mode                 ✅/⚠️/❌ X/4
Phase 25: Grid & Alignment         ✅/⚠️/❌ X/3
Phase 26: Keyboard Shortcuts       ✅/⚠️/❌ X/12
Phase 27: Theme                    ✅/⚠️/❌ X/3
Phase 28: Animations               ✅/⚠️/❌ X/3
Phase 29: Edge/Arrow               ✅/⚠️/❌ X/5
Phase 30: Stress Tests             ✅/⚠️/❌ X/8
Phase 31: Export                   ✅/⚠️/❌ X/3
Phase 32: File Switching           ✅/⚠️/❌ X/4
Phase 33: Multimodal Workflows     ✅/⚠️/❌ X/5
─────────────────────────────────────
TOTAL                              X/195
BUGS FOUND & FIXED                 N
COMMITS MADE                       N
```

For every failure found and fixed, document:

1. **What failed** — expected vs actual
2. **Root cause** — file + line
3. **Fix applied** — diff summary
4. **Severity** — 🔴 Critical / 🟡 Major / 🟢 Minor

---

## Commit & PR

After all phases complete:

```bash
git add -A
git commit -m "test(qa): overnight QA — N/195 checks, M bugs fixed"
git push -u origin HEAD
```

Create PR using GitKraken MCP with full report in the body.
