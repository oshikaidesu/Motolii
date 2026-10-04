//! One composition at time `t`. The camera and the placements are arguments.
//! Drawing them is not.

#[derive(Clone, Copy)]
pub(super) struct Frame {
    pub yaw: f32,
    pub text: [f32; 3],
    pub glass: [f32; 3],
}

/// `t` is a frame index. The camera and the text card both read it.
/// Text moves through the video card (z = 0), so front and back change with time.
pub(super) fn frame(t: u32) -> Frame {
    let seconds = t as f32 / 30.0;
    Frame {
        yaw: seconds * 0.45,
        text: [0.05, 0.72, (seconds * 0.8).sin() * 1.05],
        glass: [0.42, -0.05, 1.15],
    }
}
