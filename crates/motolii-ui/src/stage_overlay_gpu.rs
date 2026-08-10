use motolii_doc::LayerId;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct OverlayUploadKey {
    pub(crate) selected: Option<LayerId>,
    pub(crate) projection_generation: u64,
}

pub(crate) fn overlay_dirty(previous: Option<OverlayUploadKey>, next: OverlayUploadKey) -> bool {
    previous != Some(next)
}

pub(crate) fn overlay_dimensions_match(
    raster_width: u32,
    raster_height: u32,
    surface_width: u32,
    surface_height: u32,
) -> bool {
    raster_width != 0
        && raster_height != 0
        && raster_width == surface_width
        && raster_height == surface_height
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dirty_only_when_selection_or_generation_changes() {
        let key = OverlayUploadKey {
            selected: None,
            projection_generation: 3,
        };
        assert!(!overlay_dirty(Some(key), key));
        assert!(overlay_dirty(
            Some(key),
            OverlayUploadKey {
                selected: Some(LayerId::from_raw(1)),
                ..key
            }
        ));
        assert!(overlay_dirty(
            Some(key),
            OverlayUploadKey {
                projection_generation: 4,
                ..key
            }
        ));
    }

    #[test]
    fn dimensions_match_only_for_nonzero_surface_sized_raster() {
        assert!(overlay_dimensions_match(640, 360, 640, 360));
        assert!(!overlay_dimensions_match(0, 360, 640, 360));
        assert!(!overlay_dimensions_match(640, 359, 640, 360));
    }
}
