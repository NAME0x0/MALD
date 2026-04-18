/// Tiling layout system inspired by tmux/vim splits.
///
/// The layout is a binary tree where each internal node is a split (horizontal or vertical)
/// and each leaf is a pane with content.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Horizontal,
    Vertical,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PaneId(pub usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PaneContent {
    Editor,
    Terminal,
    AiChat,
    FileTree,
    Backlinks,
    Graph,
    Search,
    Tasks,
    Home,
}

#[derive(Debug, Clone)]
pub enum PaneNode {
    Leaf {
        id: PaneId,
        content: PaneContent,
    },
    Split {
        direction: Direction,
        ratio: f32, // 0.0..1.0, portion of space for `left`/`top`
        first: Box<PaneNode>,
        second: Box<PaneNode>,
    },
}

pub struct LayoutState {
    pub root: PaneNode,
    pub focused: PaneId,
    next_id: usize,
}

impl Default for LayoutState {
    fn default() -> Self {
        Self::new()
    }
}

impl LayoutState {
    pub fn new() -> Self {
        Self {
            root: PaneNode::Leaf {
                id: PaneId(0),
                content: PaneContent::Home,
            },
            focused: PaneId(0),
            next_id: 1,
        }
    }

    pub fn default_editor_layout() -> Self {
        let mut state = Self {
            root: PaneNode::Split {
                direction: Direction::Horizontal,
                ratio: 0.2,
                first: Box::new(PaneNode::Leaf {
                    id: PaneId(0),
                    content: PaneContent::FileTree,
                }),
                second: Box::new(PaneNode::Leaf {
                    id: PaneId(1),
                    content: PaneContent::Editor,
                }),
            },
            focused: PaneId(1),
            next_id: 2,
        };
        state.next_id = 2;
        state
    }

    fn alloc_id(&mut self) -> PaneId {
        let id = PaneId(self.next_id);
        self.next_id += 1;
        id
    }

    /// Split the focused pane in the given direction.
    pub fn split(&mut self, direction: Direction, content: PaneContent) {
        let new_id = self.alloc_id();
        self.root = split_node(self.root.clone(), self.focused, direction, new_id, content);
    }

    /// Close the focused pane and focus its sibling.
    pub fn close_focused(&mut self) {
        if let Some((new_root, new_focus)) = close_node(&self.root, self.focused) {
            self.root = new_root;
            self.focused = new_focus;
        }
        // If only one pane, don't close
    }

    /// Resize the split containing the focused pane.
    pub fn resize_focused(&mut self, ratio: f32) {
        resize_node(&mut self.root, self.focused, ratio.clamp(0.1, 0.9));
    }

    /// Collect all leaf pane IDs.
    pub fn pane_ids(&self) -> Vec<PaneId> {
        let mut ids = Vec::new();
        collect_ids(&self.root, &mut ids);
        ids
    }

    /// Get the content type for a given pane.
    pub fn pane_content(&self, id: PaneId) -> Option<PaneContent> {
        find_content(&self.root, id)
    }

    /// Compute (x, y, w, h) rectangles for each pane given total area.
    pub fn compute_rects(
        &self,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
    ) -> Vec<(PaneId, PaneContent, f32, f32, f32, f32)> {
        let mut rects = Vec::new();
        compute_rects_inner(&self.root, x, y, w, h, &mut rects);
        rects
    }
}

fn split_node(
    node: PaneNode,
    target: PaneId,
    direction: Direction,
    new_id: PaneId,
    content: PaneContent,
) -> PaneNode {
    match node {
        PaneNode::Leaf { id, content: c } if id == target => PaneNode::Split {
            direction,
            ratio: 0.5,
            first: Box::new(PaneNode::Leaf { id, content: c }),
            second: Box::new(PaneNode::Leaf {
                id: new_id,
                content,
            }),
        },
        PaneNode::Leaf { .. } => node,
        PaneNode::Split {
            direction: d,
            ratio,
            first,
            second,
        } => PaneNode::Split {
            direction: d,
            ratio,
            first: Box::new(split_node(*first, target, direction, new_id, content)),
            second: Box::new(split_node(*second, target, direction, new_id, content)),
        },
    }
}

fn close_node(node: &PaneNode, target: PaneId) -> Option<(PaneNode, PaneId)> {
    match node {
        PaneNode::Leaf { .. } => None, // can't close the last pane
        PaneNode::Split { first, second, .. } => {
            // Check if first child is the target leaf
            if let PaneNode::Leaf { id, .. } = first.as_ref() {
                if *id == target {
                    let sibling = *second.clone();
                    let focus = first_leaf_id(&sibling);
                    return Some((sibling, focus));
                }
            }
            // Check if second child is the target leaf
            if let PaneNode::Leaf { id, .. } = second.as_ref() {
                if *id == target {
                    let sibling = *first.clone();
                    let focus = first_leaf_id(&sibling);
                    return Some((sibling, focus));
                }
            }
            // Recurse into children
            if let Some((new_first, focus)) = close_node(first, target) {
                return Some((
                    PaneNode::Split {
                        direction: match node {
                            PaneNode::Split { direction, .. } => *direction,
                            _ => unreachable!(),
                        },
                        ratio: match node {
                            PaneNode::Split { ratio, .. } => *ratio,
                            _ => unreachable!(),
                        },
                        first: Box::new(new_first),
                        second: second.clone(),
                    },
                    focus,
                ));
            }
            if let Some((new_second, focus)) = close_node(second, target) {
                return Some((
                    PaneNode::Split {
                        direction: match node {
                            PaneNode::Split { direction, .. } => *direction,
                            _ => unreachable!(),
                        },
                        ratio: match node {
                            PaneNode::Split { ratio, .. } => *ratio,
                            _ => unreachable!(),
                        },
                        first: first.clone(),
                        second: Box::new(new_second),
                    },
                    focus,
                ));
            }
            None
        }
    }
}

fn resize_node(node: &mut PaneNode, target: PaneId, new_ratio: f32) {
    if let PaneNode::Split {
        ratio,
        first,
        second,
        ..
    } = node
    {
        let contains_target = |n: &PaneNode| -> bool {
            let mut ids = Vec::new();
            collect_ids(n, &mut ids);
            ids.contains(&target)
        };
        if contains_target(first) || contains_target(second) {
            *ratio = new_ratio;
        }
        resize_node(first, target, new_ratio);
        resize_node(second, target, new_ratio);
    }
}

fn collect_ids(node: &PaneNode, ids: &mut Vec<PaneId>) {
    match node {
        PaneNode::Leaf { id, .. } => ids.push(*id),
        PaneNode::Split { first, second, .. } => {
            collect_ids(first, ids);
            collect_ids(second, ids);
        }
    }
}

fn find_content(node: &PaneNode, target: PaneId) -> Option<PaneContent> {
    match node {
        PaneNode::Leaf { id, content } if *id == target => Some(*content),
        PaneNode::Leaf { .. } => None,
        PaneNode::Split { first, second, .. } => {
            find_content(first, target).or_else(|| find_content(second, target))
        }
    }
}

fn first_leaf_id(node: &PaneNode) -> PaneId {
    match node {
        PaneNode::Leaf { id, .. } => *id,
        PaneNode::Split { first, .. } => first_leaf_id(first),
    }
}

fn compute_rects_inner(
    node: &PaneNode,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    out: &mut Vec<(PaneId, PaneContent, f32, f32, f32, f32)>,
) {
    match node {
        PaneNode::Leaf { id, content } => {
            out.push((*id, *content, x, y, w, h));
        }
        PaneNode::Split {
            direction,
            ratio,
            first,
            second,
        } => match direction {
            Direction::Horizontal => {
                let w1 = w * ratio;
                let w2 = w - w1;
                compute_rects_inner(first, x, y, w1, h, out);
                compute_rects_inner(second, x + w1, y, w2, h, out);
            }
            Direction::Vertical => {
                let h1 = h * ratio;
                let h2 = h - h1;
                compute_rects_inner(first, x, y, w, h1, out);
                compute_rects_inner(second, x, y + h1, w, h2, out);
            }
        },
    }
}
