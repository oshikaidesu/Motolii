
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MatteMode {
    Alpha,
    InvertedAlpha,
    Luma,
    InvertedLuma,
}

pub(crate) fn matte_mode_index(mode: MatteMode) -> u32 {
    match mode {
        MatteMode::Alpha => 0,
        MatteMode::InvertedAlpha => 1,
        MatteMode::Luma => 2,
        MatteMode::InvertedLuma => 3,
    }
}
