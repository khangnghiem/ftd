//! Integration tests: parse → layout resolution → verify bounds.
//!
//! Exercises the full `fd-core` pipeline: text → SceneGraph → resolved bounds.

use fd_core::id::NodeId;
use fd_core::layout::{Viewport, resolve_layout};
use fd_core::parser::parse_document;

const VIEWPORT: Viewport = Viewport {
    width: 800.0,
    height: 600.0,
};

// ─── center_in: canvas ───────────────────────────────────────────────────

#[test]
fn center_in_canvas_centers_node() {
    let input = include_str!("fixtures/nested_layout.fd");
    let graph = parse_document(input).unwrap();
    let bounds = resolve_layout(&graph, VIEWPORT);

    let id = NodeId::intern("sidebar");
    let idx = graph.index_of(id).expect("@sidebar not found");
    let b = bounds.get(&idx).expect("no bounds for @sidebar");

    // Centered horizontally: x + width/2 ≈ viewport.width / 2
    let center_x = b.x + b.width / 2.0;
    assert!(
        (center_x - 400.0).abs() < 1.0,
        "expected horizontal center ~400, got {center_x}"
    );

    // Centered vertically: y + height/2 ≈ viewport.height / 2
    let center_y = b.y + b.height / 2.0;
    assert!(
        (center_y - 300.0).abs() < 1.0,
        "expected vertical center ~300, got {center_y}"
    );
}

// ─── Column layout ──────────────────────────────────────────────────────

#[test]
fn column_layout_stacks_children_vertically() {
    let input = include_str!("fixtures/nested_layout.fd");
    let graph = parse_document(input).unwrap();
    let bounds = resolve_layout(&graph, VIEWPORT);

    let ids = ["nav_item_1", "nav_item_2", "nav_item_3"];
    let mut ys: Vec<f32> = ids
        .iter()
        .map(|name| {
            let idx = graph.index_of(NodeId::intern(name)).unwrap();
            bounds.get(&idx).expect("no bounds").y
        })
        .collect();

    // Sort and verify all y-values are distinct (children stacked, not overlapping)
    ys.sort_by(|a, b| a.partial_cmp(b).unwrap());
    assert!(
        ys[0] < ys[1] && ys[1] < ys[2],
        "column children should have distinct y-values, got {ys:?}"
    );
}

#[test]
fn column_layout_respects_gap() {
    let input = include_str!("fixtures/nested_layout.fd");
    let graph = parse_document(input).unwrap();
    let bounds = resolve_layout(&graph, VIEWPORT);

    // Collect all nav item bounds sorted by y-position
    let ids = ["nav_item_1", "nav_item_2", "nav_item_3"];
    let mut items: Vec<(f32, f32)> = ids
        .iter()
        .map(|name| {
            let idx = graph.index_of(NodeId::intern(name)).unwrap();
            let b = bounds.get(&idx).unwrap();
            (b.y, b.height)
        })
        .collect();
    items.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

    // Gap between consecutive items = 8px (from layout: column gap=8)
    let gap = items[1].0 - (items[0].0 + items[0].1);
    assert!(
        (gap - 8.0).abs() < 0.5,
        "expected gap ~8 between first two items, got {gap}"
    );
}

// ─── Node dimensions ────────────────────────────────────────────────────

#[test]
fn resolved_bounds_preserve_declared_size() {
    let input = include_str!("fixtures/minimal.fd");
    let graph = parse_document(input).unwrap();
    let bounds = resolve_layout(&graph, VIEWPORT);

    let idx = graph.index_of(NodeId::intern("box")).unwrap();
    let b = bounds.get(&idx).unwrap();

    assert_eq!(b.width, 100.0, "width should be 100");
    assert_eq!(b.height, 50.0, "height should be 50");
}

// ─── All nodes get bounds ───────────────────────────────────────────────

#[test]
fn all_nodes_have_bounds() {
    let input = include_str!("fixtures/login_form.fd");
    let graph = parse_document(input).unwrap();
    let bounds = resolve_layout(&graph, VIEWPORT);

    // Every non-root node should have resolved bounds
    for idx in graph.graph.node_indices() {
        if idx == graph.root {
            continue;
        }
        assert!(
            bounds.contains_key(&idx),
            "node index {idx:?} has no resolved bounds"
        );
    }
}

// ─── Multiple groups ────────────────────────────────────────────────────

#[test]
fn separate_groups_resolve_independently() {
    let input = include_str!("fixtures/nested_layout.fd");
    let graph = parse_document(input).unwrap();
    let bounds = resolve_layout(&graph, VIEWPORT);

    // Both groups should have bounds
    let sidebar_idx = graph.index_of(NodeId::intern("sidebar")).unwrap();
    let main_idx = graph.index_of(NodeId::intern("main_content")).unwrap();

    assert!(bounds.contains_key(&sidebar_idx), "sidebar has no bounds");
    assert!(bounds.contains_key(&main_idx), "main_content has no bounds");
}
