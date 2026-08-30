
use crate::render::vector::{OpKind, Shape, ShapeOp};

#[derive(Debug, Clone, Copy, PartialEq, thiserror::Error)]
pub enum StackEditError {
    #[error("insert index {index} is out of range (stack has {len} entries)")]
    InsertOutOfRange { index: usize, len: usize },
    #[error("op index {index} is out of range (stack has {len} entries)")]
    OpOutOfRange { index: usize, len: usize },
}

pub fn insert_op(ops: &[ShapeOp], at: usize, op: ShapeOp) -> Result<Vec<ShapeOp>, StackEditError> {
    let len = ops.len();
    if at > len {
        return Err(StackEditError::InsertOutOfRange { index: at, len });
    }
    let mut out = ops.to_vec();
    out.insert(at, op);
    Ok(out)
}

pub fn remove_op(ops: &[ShapeOp], at: usize) -> Result<Vec<ShapeOp>, StackEditError> {
    let len = ops.len();
    if at >= len {
        return Err(StackEditError::OpOutOfRange { index: at, len });
    }
    let mut out = ops.to_vec();
    out.remove(at);
    Ok(out)
}

pub fn move_op(ops: &[ShapeOp], from: usize, to: usize) -> Result<Vec<ShapeOp>, StackEditError> {
    let len = ops.len();
    if from >= len {
        return Err(StackEditError::OpOutOfRange { index: from, len });
    }
    if to >= len {
        return Err(StackEditError::OpOutOfRange { index: to, len });
    }
    if from == to {
        return Ok(ops.to_vec());
    }
    let mut out = ops.to_vec();
    let moved = out.remove(from);
    out.insert(to, moved);
    Ok(out)
}

pub fn set_kind(ops: &[ShapeOp], at: usize, kind: OpKind) -> Result<Vec<ShapeOp>, StackEditError> {
    let len = ops.len();
    if at >= len {
        return Err(StackEditError::OpOutOfRange { index: at, len });
    }
    let mut out = ops.to_vec();
    out[at].kind = kind;
    Ok(out)
}

pub fn set_hidden(ops: &[ShapeOp], at: usize, hidden: bool) -> Result<Vec<ShapeOp>, StackEditError> {
    let len = ops.len();
    if at >= len {
        return Err(StackEditError::OpOutOfRange { index: at, len });
    }
    let mut out = ops.to_vec();
    out[at].hidden = hidden;
    Ok(out)
}

pub fn with_ops<F>(shape: &Shape, f: F) -> Result<Shape, StackEditError>
where
    F: FnOnce(&[ShapeOp]) -> Result<Vec<ShapeOp>, StackEditError>,
{
    let ops = f(&shape.ops)?;
    Ok(Shape {
        ops,
        ..shape.clone()
    })
}
