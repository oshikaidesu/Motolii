#[cfg(target_os = "macos")]
use motolii_ui::{motolii_rn_host_dispatch_intent_json, motolii_rn_host_projection_stamp};

use super::json_scan::{
    find_key_object, find_matching_brace, find_root_key_array, json_bool_value, json_string_value,
};
use super::parse_wire::parse_timeline_projection;
use super::slot::{slice_from_written, MAX_JSON_BYTES, MAX_SNAPSHOT_JSON_BYTES};
use super::types::{HostTerminalDiagnostic, HostTerminalResult};

#[cfg(target_os = "macos")]
pub(super) fn dispatch_intent_json_terminal(handle: u64, intent: &str) -> Option<HostTerminalResult> {
    let intent = intent_with_projection_generation(handle, intent);
    if intent.len() > MAX_JSON_BYTES {
        return None;
    }
    let mut out = [0u8; MAX_SNAPSHOT_JSON_BYTES];
    let written = unsafe {
        motolii_rn_host_dispatch_intent_json(
            handle,
            intent.as_ptr(),
            intent.len(),
            out.as_mut_ptr(),
            out.len(),
        )
    };
    if written <= 0 {
        return None;
    }
    let Some(response_bytes) = slice_from_written(&out, written) else {
        return None;
    };
    let Ok(response) = std::str::from_utf8(response_bytes) else {
        return None;
    };
    parse_terminal_result(response)
}

#[cfg(target_os = "macos")]
pub(super) fn intent_with_projection_generation(handle: u64, intent: &str) -> String {
    if intent.contains("\"projection_generation\"") {
        return intent.to_owned();
    }
    let mut _revision = 0u64;
    let mut generation = 0u64;
    if !unsafe { motolii_rn_host_projection_stamp(handle, &mut _revision, &mut generation) } {
        return intent.to_owned();
    }
    let Some(body) = intent.strip_suffix('}') else {
        return intent.to_owned();
    };
    format!(r#"{body},"projection_generation":"{generation}"}}"#)
}

pub(super) fn parse_terminal_result(response: &str) -> Option<HostTerminalResult> {
    let accepted = json_bool_value(response, "accepted")?;
    let projection = find_key_object(response, "snapshot").and_then(parse_timeline_projection);
    let diagnostics = parse_terminal_diagnostics(response)?;
    Some(HostTerminalResult {
        accepted,
        diagnostics,
        message: json_string_value(response, "message"),
        projection,
    })
}

pub(super) fn parse_terminal_diagnostics(response: &str) -> Option<Vec<HostTerminalDiagnostic>> {
    let Some(array) = find_root_key_array(response, "diagnostics") else {
        return Some(Vec::new());
    };
    let mut diagnostics = Vec::new();
    let mut rest = &array[1..array.len() - 1];
    loop {
        rest = rest.trim_start();
        if rest.is_empty() {
            break;
        }
        if let Some(next) = rest.strip_prefix(',') {
            rest = next;
            continue;
        }
        if !rest.starts_with('{') {
            return None;
        }
        let end = find_matching_brace(rest)?;
        let diagnostic = &rest[..=end];
        diagnostics.push(HostTerminalDiagnostic {
            reason: json_string_value(diagnostic, "reason")?,
            host_handle: json_string_value(diagnostic, "host_handle"),
            stage_handle: json_string_value(diagnostic, "stage_handle"),
            timeline_handle: json_string_value(diagnostic, "timeline_handle"),
            expected_projection_generation: json_string_value(
                diagnostic,
                "expected_projection_generation",
            ),
            actual_projection_generation: json_string_value(
                diagnostic,
                "actual_projection_generation",
            ),
        });
        rest = &rest[end + 1..];
    }
    Some(diagnostics)
}

pub(super) fn response_is_accepted(response: &str) -> bool {
    let Some(pos) = response.find("\"accepted\"") else {
        return false;
    };
    let Some(colon_pos) = response[pos..].find(':') else {
        return false;
    };
    response[pos + colon_pos + 1..]
        .trim_start()
        .starts_with("true")
}

pub(super) fn inject_host_handle(intent: &str, handle: u64) -> Result<String, ()> {
    const KEY: &str = "\"host_handle\"";
    // top-level fieldのみ。brace depthで nested `"host_handle"` への誤爆を防ぐ。
    let mut depth = 0i32;
    let mut in_string = false;
    let mut escape = false;
    let bytes = intent.as_bytes();
    let mut key_at = None;
    let mut root_open = None;
    let mut root_end = None;
    let mut i = 0usize;
    while i < bytes.len() {
        let b = bytes[i];
        if in_string {
            if escape {
                escape = false;
            } else if b == b'\\' {
                escape = true;
            } else if b == b'"' {
                in_string = false;
            }
            i += 1;
            continue;
        }
        match b {
            b'"' => {
                if depth == 1 && intent[i..].starts_with(KEY) {
                    let after = &intent[i + KEY.len()..];
                    if after.trim_start().starts_with(':') {
                        key_at = Some(i);
                        break;
                    }
                }
                in_string = true;
                i += 1;
            }
            b'{' => {
                if depth == 0 {
                    root_open = Some(i);
                }
                depth += 1;
                i += 1;
            }
            b'}' => {
                if depth == 1 {
                    root_end = Some(i);
                    break;
                }
                depth -= 1;
                i += 1;
            }
            _ => i += 1,
        }
    }
    let Some(key_at) = key_at else {
        let root_open = root_open.ok_or(())?;
        let root_end = root_end.ok_or(())?;
        let separator = if intent[root_open + 1..root_end].trim().is_empty() {
            ""
        } else {
            ","
        };
        let mut out = String::with_capacity(intent.len() + 40);
        out.push_str(&intent[..root_end]);
        out.push_str(separator);
        out.push_str(&format!(r#""host_handle":"{handle}""#));
        out.push_str(&intent[root_end..]);
        return Ok(out);
    };
    let after_key = &intent[key_at + KEY.len()..];
    let colon = after_key.find(':').ok_or(())?;
    let after_colon = &after_key[colon + 1..];
    let value_start_rel = after_colon.find('"').ok_or(())?;
    let value_body = &after_colon[value_start_rel + 1..];
    let value_end_rel = value_body.find('"').ok_or(())?;
    let abs_value_start = key_at + KEY.len() + colon + 1 + value_start_rel + 1;
    let abs_value_end = abs_value_start + value_end_rel;
    let mut out = String::with_capacity(intent.len() + 24);
    out.push_str(&intent[..abs_value_start]);
    out.push_str(&handle.to_string());
    out.push_str(&intent[abs_value_end..]);
    Ok(out)
}
