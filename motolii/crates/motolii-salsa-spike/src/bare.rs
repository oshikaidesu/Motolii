//! Same arithmetic as the salsa spike, without a dependency ledger.
//! Release size of this binary is the baseline for the salsa delta.

use std::time::Instant;

const LAYERS: i32 = 1000;
const FRAMES: i32 = 30;

struct Layer {
    x: i32,
    y: i32,
    opacity: u16,
    parent: Option<usize>,
}

fn main() {
    let mut layers = Vec::with_capacity(LAYERS as usize);
    for i in 0..LAYERS {
        let parent = (i > 0 && i % 10 == 0).then_some((i as usize) - 1);
        layers.push(Layer { x: i, y: 0, opacity: 255, parent });
    }
    let started = Instant::now();
    let mut sum = 0i64;
    let mut frames = 0u64;
    for t in 0..FRAMES {
        for layer in &layers {
            let (px, py) = match layer.parent {
                Some(parent) => (layers[parent].x + t, layers[parent].y),
                None => (0, 0),
            };
            sum += (layer.x + px + t) as i64 + layer.y as i64 + py as i64;
            sum += i64::from(layer.opacity);
        }
        frames += 1;
    }
    let elapsed = started.elapsed();
    println!("bare frames={frames} sum={sum} us={}", elapsed.as_micros());
}
