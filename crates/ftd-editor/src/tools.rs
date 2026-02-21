//! Tool system for canvas interactions.
//!
//! Each tool translates user input events into `GraphMutation` commands
//! that are applied via the `SyncEngine`.

use crate::input::InputEvent;
use crate::sync::GraphMutation;
use ftd_core::id::NodeId;
use ftd_core::model::*;

/// The active tool determines how input events are interpreted.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolKind {
    Select,
    Rect,
    Ellipse,
    Pen,
    Text,
}

/// Trait for tools that handle input and produce mutations.
pub trait Tool {
    fn kind(&self) -> ToolKind;

    /// Handle an input event, returning zero or more mutations.
    fn handle(&mut self, event: &InputEvent, hit_node: Option<NodeId>) -> Vec<GraphMutation>;
}

// ─── Select Tool ─────────────────────────────────────────────────────────

pub struct SelectTool {
    /// Currently selected node.
    pub selected: Option<NodeId>,
    /// Drag state.
    dragging: bool,
    last_x: f32,
    last_y: f32,
}

impl Default for SelectTool {
    fn default() -> Self {
        Self::new()
    }
}

impl SelectTool {
    pub fn new() -> Self {
        Self {
            selected: None,
            dragging: false,
            last_x: 0.0,
            last_y: 0.0,
        }
    }
}

impl Tool for SelectTool {
    fn kind(&self) -> ToolKind {
        ToolKind::Select
    }

    fn handle(&mut self, event: &InputEvent, hit_node: Option<NodeId>) -> Vec<GraphMutation> {
        match event {
            InputEvent::PointerDown { x, y, .. } => {
                self.selected = hit_node;
                self.dragging = self.selected.is_some();
                self.last_x = *x;
                self.last_y = *y;
                vec![]
            }
            InputEvent::PointerMove { x, y, .. } => {
                if self.dragging {
                    if let Some(id) = self.selected {
                        let dx = x - self.last_x;
                        let dy = y - self.last_y;
                        self.last_x = *x;
                        self.last_y = *y;
                        return vec![GraphMutation::MoveNode { id, dx, dy }];
                    }
                }
                vec![]
            }
            InputEvent::PointerUp { .. } => {
                self.dragging = false;
                vec![]
            }
            _ => vec![],
        }
    }
}

// ─── Rect Tool ───────────────────────────────────────────────────────────

pub struct RectTool {
    drawing: bool,
    start_x: f32,
    start_y: f32,
    current_id: Option<NodeId>,
}

impl Default for RectTool {
    fn default() -> Self {
        Self::new()
    }
}

impl RectTool {
    pub fn new() -> Self {
        Self {
            drawing: false,
            start_x: 0.0,
            start_y: 0.0,
            current_id: None,
        }
    }
}

impl Tool for RectTool {
    fn kind(&self) -> ToolKind {
        ToolKind::Rect
    }

    fn handle(&mut self, event: &InputEvent, _hit_node: Option<NodeId>) -> Vec<GraphMutation> {
        match event {
            InputEvent::PointerDown { x, y, .. } => {
                self.drawing = true;
                self.start_x = *x;
                self.start_y = *y;
                let id = NodeId::anonymous();
                self.current_id = Some(id);

                let node = SceneNode::new(
                    id,
                    NodeKind::Rect {
                        width: 0.0,
                        height: 0.0,
                    },
                );
                vec![GraphMutation::AddNode {
                    parent_id: NodeId::intern("root"),
                    node: Box::new(node),
                }]
            }
            InputEvent::PointerMove { x, y, .. } => {
                if self.drawing {
                    if let Some(id) = self.current_id {
                        let w = (x - self.start_x).abs();
                        let h = (y - self.start_y).abs();
                        return vec![GraphMutation::ResizeNode {
                            id,
                            width: w,
                            height: h,
                        }];
                    }
                }
                vec![]
            }
            InputEvent::PointerUp { .. } => {
                self.drawing = false;
                self.current_id = None;
                vec![]
            }
            _ => vec![],
        }
    }
}

// ─── Pen Tool (freehand) ─────────────────────────────────────────────────

pub struct PenTool {
    drawing: bool,
    points: Vec<(f32, f32)>,
    current_id: Option<NodeId>,
}

impl Default for PenTool {
    fn default() -> Self {
        Self::new()
    }
}

impl PenTool {
    pub fn new() -> Self {
        Self {
            drawing: false,
            points: Vec::new(),
            current_id: None,
        }
    }
}

impl Tool for PenTool {
    fn kind(&self) -> ToolKind {
        ToolKind::Pen
    }

    fn handle(&mut self, event: &InputEvent, _hit_node: Option<NodeId>) -> Vec<GraphMutation> {
        match event {
            InputEvent::PointerDown { x, y, .. } => {
                self.drawing = true;
                self.points.clear();
                self.points.push((*x, *y));
                let id = NodeId::anonymous();
                self.current_id = Some(id);

                let path = NodeKind::Path {
                    commands: vec![PathCmd::MoveTo(*x, *y)],
                };
                let node = SceneNode::new(id, path);
                vec![GraphMutation::AddNode {
                    parent_id: NodeId::intern("root"),
                    node: Box::new(node),
                }]
            }
            InputEvent::PointerMove { x, y, .. } => {
                if self.drawing {
                    self.points.push((*x, *y));
                }
                vec![]
            }
            InputEvent::PointerUp { .. } => {
                if !self.drawing {
                    return vec![];
                }
                self.drawing = false;

                let mut mutations = Vec::new();

                if let Some(id) = self.current_id.take() {
                    // Build finalized path commands from accumulated points
                    let commands = build_path_commands(&self.points);
                    let mut node = SceneNode::new(id, NodeKind::Path { commands });
                    node.style.stroke = Some(Stroke {
                        paint: Paint::Solid(Color::from_hex("#333333").expect("hardcoded color")),
                        width: 2.0,
                        ..Stroke::default()
                    });

                    // Remove placeholder and add finalized node
                    mutations.push(GraphMutation::RemoveNode { id });
                    mutations.push(GraphMutation::AddNode {
                        parent_id: NodeId::intern("root"),
                        node: Box::new(node),
                    });
                }

                self.points.clear();
                mutations
            }
            _ => vec![],
        }
    }
}

/// Build path commands from a sequence of points.
/// Uses `MoveTo` for the first point and `LineTo` for subsequent ones.
/// Points too close together (< 2px apart) are skipped to reduce noise.
fn build_path_commands(points: &[(f32, f32)]) -> Vec<PathCmd> {
    if points.is_empty() {
        return vec![];
    }

    let mut commands = vec![PathCmd::MoveTo(points[0].0, points[0].1)];
    let mut last = points[0];

    for &(x, y) in &points[1..] {
        let dist_sq = (x - last.0).powi(2) + (y - last.1).powi(2);
        if dist_sq < 4.0 {
            continue; // Skip points too close (< 2px)
        }
        commands.push(PathCmd::LineTo(x, y));
        last = (x, y);
    }

    commands
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::input::InputEvent;

    #[test]
    fn select_tool_drag() {
        let mut tool = SelectTool::new();
        let target = NodeId::intern("box1");

        // Press on a node
        let mutations = tool.handle(
            &InputEvent::PointerDown {
                x: 100.0,
                y: 100.0,
                pressure: 1.0,
            },
            Some(target),
        );
        assert!(mutations.is_empty()); // Press alone doesn't mutate
        assert_eq!(tool.selected, Some(target));

        // Drag
        let mutations = tool.handle(
            &InputEvent::PointerMove {
                x: 110.0,
                y: 105.0,
                pressure: 1.0,
            },
            None,
        );
        assert_eq!(mutations.len(), 1);
        match &mutations[0] {
            GraphMutation::MoveNode { id, dx, dy } => {
                assert_eq!(*id, target);
                assert!((dx - 10.0).abs() < 0.01);
                assert!((dy - 5.0).abs() < 0.01);
            }
            _ => panic!("expected MoveNode"),
        }
    }

    #[test]
    fn select_tool_deselect() {
        let mut tool = SelectTool::new();
        let target = NodeId::intern("box2");

        // Press on a node to select
        tool.handle(
            &InputEvent::PointerDown {
                x: 50.0,
                y: 50.0,
                pressure: 1.0,
            },
            Some(target),
        );
        assert_eq!(tool.selected, Some(target));

        // Press on empty space to deselect
        tool.handle(
            &InputEvent::PointerDown {
                x: 300.0,
                y: 300.0,
                pressure: 1.0,
            },
            None,
        );
        assert_eq!(tool.selected, None);
    }

    #[test]
    fn rect_tool_draw() {
        let mut tool = RectTool::new();

        // Start drawing
        let mutations = tool.handle(
            &InputEvent::PointerDown {
                x: 10.0,
                y: 10.0,
                pressure: 1.0,
            },
            None,
        );
        assert_eq!(mutations.len(), 1);
        match &mutations[0] {
            GraphMutation::AddNode { node, .. } => {
                assert!(matches!(node.kind, NodeKind::Rect { .. }));
            }
            _ => panic!("expected AddNode"),
        }

        // Drag to resize
        let mutations = tool.handle(
            &InputEvent::PointerMove {
                x: 110.0,
                y: 60.0,
                pressure: 1.0,
            },
            None,
        );
        assert_eq!(mutations.len(), 1);
        match &mutations[0] {
            GraphMutation::ResizeNode { width, height, .. } => {
                assert_eq!(*width, 100.0);
                assert_eq!(*height, 50.0);
            }
            _ => panic!("expected ResizeNode"),
        }

        // Release
        let mutations = tool.handle(&InputEvent::PointerUp { x: 110.0, y: 60.0 }, None);
        assert!(mutations.is_empty());
    }

    #[test]
    fn pen_tool_draw() {
        let mut tool = PenTool::new();

        // Start drawing
        let mutations = tool.handle(
            &InputEvent::PointerDown {
                x: 0.0,
                y: 0.0,
                pressure: 1.0,
            },
            None,
        );
        assert_eq!(mutations.len(), 1, "PointerDown should add a path node");

        // Accumulate points
        tool.handle(
            &InputEvent::PointerMove {
                x: 50.0,
                y: 25.0,
                pressure: 1.0,
            },
            None,
        );
        tool.handle(
            &InputEvent::PointerMove {
                x: 100.0,
                y: 50.0,
                pressure: 1.0,
            },
            None,
        );

        // Release — should finalize path
        let mutations = tool.handle(&InputEvent::PointerUp { x: 100.0, y: 50.0 }, None);
        assert_eq!(
            mutations.len(),
            2,
            "PointerUp should remove placeholder and add finalized"
        );

        // First mutation: remove placeholder
        assert!(matches!(&mutations[0], GraphMutation::RemoveNode { .. }));

        // Second mutation: add final path with commands
        match &mutations[1] {
            GraphMutation::AddNode { node, .. } => {
                if let NodeKind::Path { commands } = &node.kind {
                    assert!(commands.len() >= 2, "path should have MoveTo + LineTo(s)");
                    assert!(
                        matches!(commands[0], PathCmd::MoveTo(0.0, 0.0)),
                        "first command should be MoveTo"
                    );
                } else {
                    panic!("expected Path node");
                }
                // Should have a stroke style
                assert!(node.style.stroke.is_some(), "pen path should have stroke");
            }
            _ => panic!("expected AddNode"),
        }
    }

    #[test]
    fn build_path_commands_filters_close_points() {
        let points = vec![
            (0.0, 0.0),
            (0.5, 0.5), // too close, should be skipped
            (10.0, 10.0),
            (10.1, 10.1), // too close, should be skipped
            (50.0, 50.0),
        ];

        let cmds = build_path_commands(&points);
        assert_eq!(cmds.len(), 3); // MoveTo + 2 LineTo (skipped 2 close points)
    }
}
