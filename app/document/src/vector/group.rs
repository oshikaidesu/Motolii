
use crate::doc::vector::{RepeaterTransform, Shape};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ShapeNode {
    Leaf(Shape),
    Group(ShapeGroup),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ShapeGroup {
    pub transform: RepeaterTransform,
    pub children: Vec<ShapeNode>,
}
