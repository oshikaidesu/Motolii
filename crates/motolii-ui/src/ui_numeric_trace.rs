//! UI境界の数値変換をTransient診断として記録する。

pub(crate) fn emit(arguments: std::fmt::Arguments<'_>) {
    if enabled() {
        eprintln!("[motolii-ui-trace] {arguments}");
    }
}

fn enabled() -> bool {
    cfg!(debug_assertions)
        || std::env::var_os("MOTOLII_UI_TRACE").is_some()
        || std::env::var_os("MOTOLII_UI_NUMERIC_TRACE").is_some()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn debug_ui_always_enables_numeric_trace() {
        assert!(enabled());
    }

    /// Web窓(wry)の host runtime 3本は 2026-08-16 に凍結・撤去したので、
    /// この検査に残っているのは pointer 境界だけ。
    #[test]
    fn pointer_trace_schema_keeps_every_numeric_boundary() {
        let pointer_source = include_str!("host_pointer_capture.rs");
        for field in [
            "generation=",
            "raw_x=",
            "raw_y=",
            "content_height=",
            "content_is_flipped=",
            "logical_x=",
            "logical_y=",
        ] {
            assert!(
                pointer_source.contains(field),
                "pointer numeric trace is missing {field}"
            );
        }
    }
}
