# Lessons Learned

Engineering lessons discovered through building FD.

---

## Layout Solver: Bounds ≠ Visual Position

**Date**: 2026-02-27
**Context**: Text nodes inside shapes (rect/ellipse/frame) appeared at the top-left instead of centered.

**Root cause**: The layout solver placed text children at the parent's origin with their _intrinsic_ size (e.g., 60×14 for a short label). The renderer correctly centered text _within its own bounds_, but those bounds were a tiny rectangle at the parent's corner — not spanning the full parent.

**Fix**: In `LayoutMode::Free`, when a shape parent has exactly one text child (no explicit position), expand the text bounds to fill the parent. The renderer's existing center/middle alignment then handles the visual centering.

**Lesson**: In a layout-then-render pipeline, the renderer can only center text within the bounds the layout gives it. If the bounds are wrong, alignment defaults are irrelevant. Always verify the _bounds_ passed to the renderer, not just the renderer's alignment logic.

---

## Multi-Layer Defaults: Model vs Renderer vs UI

**Date**: 2026-02-27
**Context**: The properties panel showed `textAlign: center` and `textVAlign: middle` as defaults, but the text wasn't visually centered.

**Root cause**: Defaults existed in 3 places:

1. **Renderer** (`render2d.rs`): defaults `center`/`middle` when `in_shape` is true ✅
2. **Properties panel** (`main.js`): defaults `center`/`middle` for display ✅
3. **Layout solver** (`layout.rs`): no text-in-shape awareness ❌

**Lesson**: When a feature spans multiple layers (model → layout → renderer → UI), ensure each layer agrees on behavior. A default in the UI (panel) or renderer is useless if the layout solver doesn't produce the right geometry.

---

## Layer Panel Skips Selection Highlight on Canvas Click

**Date**: 2026-02-28
**Context**: Clicking a node on canvas did not highlight it in the Layers panel. Reported multiple times.

**Root cause**: `refreshLayersPanel()` in `main.js` uses a generation-counter optimization: `if (sceneGeneration === lastLayerGeneration && selectedId === lastLayerSelectedId) return;`. When the user clicks a node on canvas, `sceneGeneration` doesn't change (no structural edit), so when the _selection_ changes but the _scene_ doesn't, the function skips the entire DOM update — including the `.selected` CSS class toggle.

**Fix**: Added a separate code path: when `sceneGeneration` matches but `selectedId` differs, update `.selected` class on existing layer items without a full DOM rebuild.

**Lesson**: Optimization shortcuts that skip DOM updates must account for all change dimensions. Selection changes and structural changes are independent — caching on one dimension (generation) can silently skip the other (selection state).

---

## Smart Guides: 1px Threshold Too Tight + Scoping Concerns

**Date**: 2026-02-28
**Context**: Smart guides (snap alignment lines) stopped appearing when dragging nodes, especially text outside parent shapes.

**Root cause**: The `compute_smart_guides()` function in `lib.rs` used a `snap_threshold` of 1.0px — guides only appeared at near-pixel-perfect alignment, making them practically invisible during normal drag operations. Additionally, while the function iterates all nodes (not just siblings), the tight threshold meant guides disappeared before the user could see them.

**Fix**: Increased `snap_threshold` from 1.0 to 5.0 pixels, matching industry-standard snap distances (Figma uses ~5px).

**Lesson**: Snap thresholds should match user interaction precision, not render precision. A 1px snap window is mathematically correct but practically useless at standard zoom levels. Always test snap features by dragging — not by computing distances in code.

---

## Group Detach: "Chasing Envelope" Bug

**Date**: 2026-02-28
**Context**: Dragging a child node outside a group never detached it, despite the detach logic being correct in unit tests.

**Root cause**: `handle_child_group_relationship` called `expand_group_to_children` every frame when the child partially overlapped the parent. This grew the parent to contain the child — so next frame, the child was always inside the expanded group. The group **chased** the child indefinitely. Unit tests passed because they used a single large `MoveNode(dx:500)` jump, bypassing intermediate frames.

**Fix**: Skip group expansion during continuous drag. Check overlap against the parent's **current stored bounds** without expanding. The group bounds stay stable; when the child fully exits, it detaches.

**Lesson**: When a per-frame mutation (drag) modifies both A and B, and then checks A against B, ensure neither mutation feeds back into the other's state. The expand-then-compare loop created an implicit dependency where the group (B) always contained the child (A), making the check tautological. Unit tests that use large single-step inputs miss frame-by-frame feedback bugs — always write tests that simulate real gestures (many small increments).

---

## VS Code Webview Context Menu Interception

**Date**: 2026-03-01
**Context**: Added a custom right-click context menu to items in the Layers panel inside the FD custom editor extension. The `contextmenu` event handler fired in regular browsers but failed to show the custom menu inside VS Code.
**Root cause**: VS Code webviews run inside an iframe hierarchy where the host application (VS Code itself) aggressively intercepts right-click (`contextmenu`) events to display its own native developer/extension menus. Even using `e.stopPropagation()`, `e.stopImmediatePropagation()`, and `true` (capture phase) on the webview DOM cannot consistently beat the host iframe interception.
**Fix**: Pivoted to a standard VS Code UI pattern — added an explicit `⋮` (more actions) button to each layer item that appears on hover. Clicking the button safely triggers the custom context menu without competing with the host's right-click capture.
**Lesson**: Never rely on native `contextmenu` events inside VS Code webviews for critical functionality. Always provide an explicit UI button (like a `⋮` or `⚙` icon) as an alternative or primary interaction method for webview-level context menus.

---

## Continuous Drag State Truncation

**Date**: 2026-03-01
**Context**: Fixing the group detach bug, but the UI still didn't reflect the detach despite the Rust core correctly executing the structural reparenting on the first frame of exiting the group.
**Root cause**: The `last_detach` flag in the Rust `SyncEngine` was unconditionally overwritten on every frame (`MoveNode` mutation). As the user continued dragging the detached node outside the group, the overlap check correctly evaluated to `None` (since the node was already detached), which overwrote `last_detach` with `None`. By the time the user released the mouse (`pointerup`), the UI read `None` instead of the original detach event.
**Fix**: Changed the update logic to accumulate the state: `if let Some(info) = check_detach() { self.last_detach = Some(info); }`. The accumulated state is then taken (`.take()`) when the UI finally reads it on `pointerup`.
**Lesson**: When bridging continuous events (like 60fps drag frames) to discrete event handlers (like `pointerup` UI syncs), ensure that one-shot trigger states (like "did detach") accumulate and persist rather than getting overwritten by the steady state of subsequent frames.

---

## Git: Never Push Directly to Main

**Date**: 2026-03-01
**Context**: After merging a PR locally with `git merge --no-ff`, attempted `git push origin main` to push the merge commit.

**Root cause**: The `.githooks/pre-push` hook blocks all direct pushes to the `main` branch. This is by design (configured via `git config core.hooksPath .githooks`). The local merge succeeded, but the push was rejected — leaving local main ahead of remote with no way to sync without force-push.

**Fix**: Never run `git push origin main`. Instead, merge PRs via `gh pr merge <number> --merge --delete-branch` (GitHub CLI) or the GitHub web UI. Then sync local main with `git pull origin main`. If local main diverges, reset with `git reset --hard origin/main` before pulling.

**Lesson**: In this repo, the merge workflow is: create branch → push branch → create PR → merge PR via `gh pr merge` → `git pull origin main` locally. Never attempt `git checkout main && git merge && git push` — the pre-push hook will always reject it.

---

## Editor: Text Reparent Blocked by `&& changed` Gate + Animation Picker Race

**Date**: 2026-03-01
**Context**: Dragging a text node onto a rect/group/frame to reparent it (R3.38 text-consume) silently did nothing — no error, no visual feedback, text stayed at root.

**Root cause**: Two bugs compounded:

1. **`&& changed` gate** (`main.js` line 724): The `evaluateTextAdoption()` call sits inside `if (isDraggingNode && draggedNodeId && changed)`. The `changed` flag comes from `handle_pointer_move()` (WASM). On the last frame of a drag, when the pointer slows down or rests over the target, the WASM reports `changed = false` (no position delta). The `else` branch (line 766) then executes `textDropTarget = null`, erasing the adoption target right before `pointerup`.

2. **Animation picker intercepts** (`main.js` line 849 vs 861): In `pointerup`, the animation drop handler (`if (animDropTargetId && ...)`) fires _before_ the text reparent handler. When dragging text onto a node, `animDropTargetId` is set for that same node (because any node under the cursor gets flagged as an animation drop target). `openAnimPicker()` fires, stealing the interaction. Even if `textDropTarget` survived bug #1, the animation picker already consumed the gesture.

**Fix**: (1) Move `evaluateTextAdoption()` outside the `&& changed` gate — text adoption should evaluate on every pointer-move frame regardless of WASM position change. (2) In `pointerup`, skip the animation drop handler when `textDropTarget` is set (text reparent takes priority over animation binding).

**Lesson**: When gating side-effect evaluations on a `changed` flag from a lower layer (WASM), distinguish between "model changed" (position moved) and "interaction continues" (still dragging). Adoption detection depends on _cursor position vs target bounds_, not on _model state change_. Similarly, when multiple drop-zone handlers compete in the same `pointerup`, priority must be explicit — the first `if` to fire wins and silently blocks everything below it.

---

## Renderer: Hit Radius Must Match Visual Handle Size

**Date**: 2026-03-01
**Context**: Users reported that 8-point resize handles on selected nodes were visible but couldn't be grabbed. Resize didn't work at all.

**Root cause**: The hit test radius for resize handles was 5px in scene-space (`lib.rs:hit_test_resize_handle` and `main.js:getResizeHandleCursor`), while the visual handle size was 7px (`render2d.rs:draw_selection_handles`). At any zoom level below ~1.5×, the hit area was smaller than a finger/cursor — making handles practically unusable. The WASM hit test and JS cursor feedback both used the same tight radius, so neither layer compensated.

**Fix**: Increased hit radius from 5px to 8px in both `lib.rs:hit_test_resize_handle` (Rust/WASM side) and `main.js:getResizeHandleCursor` (JS cursor feedback side). The hit area now exceeds the visual handle, matching Figma/Sketch behavior.

**Lesson**: Hit test radii for interactive handles should be **at least 1.5× the visual radius** to account for cursor imprecision and zoom levels. When the same hit test exists in two layers (WASM + JS), update **both** — mismatched radii cause cursor feedback to not match actual interaction.

---

## WASM: Text Intrinsic Sizing Padding Accumulates Visually

**Date**: 2026-03-01
**Context**: Users reported text node boundaries extending well beyond the visible text content. The bounding box was noticeably larger than the text.

**Root cause**: `update_text_metrics()` in `lib.rs` added 8px padding per side (16px total per axis). The JS `measureText()` already returns a width that includes some internal glyph spacing. Combined with the 16px padding, the resulting bounds exceeded the visual text by ~20px — visible as an oversized selection rectangle.

**Fix**: Reduced padding from 8px to 4px per side. Total overhead is now 8px per axis, which tightly wraps the text without clipping.

**Lesson**: When bridging text measurement across JS→WASM, account for the fact that `measureText().width` already includes glyph-level spacing. Additional padding should be minimal (2–4px per side) — it's a safety margin, not a design element. Test visual fit by selecting a text node and comparing the selection rectangle to the rendered text.

---

## Editor: Feature Removal Requires Full Call-Chain Cleanup

**Date**: 2026-03-01
**Context**: Removing the animation-picker-on-drag feature (bug #4). Initial attempt only removed the `openAnimPicker()` call in `pointerup`, but the glow ring kept rendering and the drop detection kept running.

**Root cause**: The animation drop feature had 3 code sites: (1) drop-zone detection in `pointermove` (L724-738), (2) glow ring rendering in `render()` (L441-457), (3) picker trigger in `pointerup` (L855-863). Removing only the trigger left the detection and rendering running — wasting CPU cycles and causing a purple glow ring with no purpose.

**Fix**: Removed all 3 code sites together: detection, rendering, and trigger. Also cleaned up the state variables (`animDropTargetId`, `animDropTargetBounds`) in the reset block at L901-904.

**Lesson**: When removing a feature, trace its full call chain: **detect → render → trigger → cleanup**. Search for all variable names associated with the feature (e.g., `animDropTargetId`, `animDropTargetBounds`) and remove every read/write site. A partial removal leaves orphaned state and wasted computation.
