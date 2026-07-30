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

    #[test]
    fn place_trace_schema_keeps_every_numeric_boundary() {
        let pointer_source = include_str!("host_pointer_capture.rs");
        let product_source = include_str!("product_runtime.rs");
        let browser_source = include_str!("browser_host_runtime.rs");
        let inspector_source = include_str!("inspector_host_runtime.rs");

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
        for field in [
            "layout_epoch=",
            "stage_x=",
            "stage_y=",
            "stage_width=",
            "stage_height=",
            "ndc_x=",
            "ndc_y=",
            "canonical_x=",
            "canonical_y=",
            "kind=startup",
            "kind=layout",
            "kind=browser-intent",
            "kind=document-publish",
            "kind=stage-submit",
            "kind=stage-result",
            "kind=timeline-projection",
            "kind=timeline-hit",
            "kind=window",
            "kind=surface",
            "kind=failure",
        ] {
            assert!(
                product_source.contains(field),
                "product numeric trace is missing {field}"
            );
        }
        for field in [
            "event=load-started",
            "event=load-finished",
            "event=ipc-accepted",
            "event=bounds",
            "event=focus",
        ] {
            assert!(
                browser_source.contains(field),
                "Browser trace is missing {field}"
            );
        }
        for field in ["surface=inspector", "event=bounds", "event=publish"] {
            assert!(
                inspector_source.contains(field),
                "Inspector trace is missing {field}"
            );
        }
    }
}
