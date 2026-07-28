//! M4-K1a: 製品GPU割当のowner棚卸しをraw callsiteの増減に追従させる。

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct InventoryEntry {
    relative_path: &'static str,
    raw_allocation_calls: usize,
    owner_seats: &'static [OwnerSeat],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct OwnerSeat {
    name: &'static str,
    lifetime: &'static str,
    peak_multiplicity: &'static str,
}

const INVENTORY: &[InventoryEntry] = &[
    InventoryEntry {
        relative_path: "crates/motolii-gpu/src/pipeline_cache.rs",
        raw_allocation_calls: 2,
        owner_seats: &[OwnerSeat {
            name: "pipeline-uniform",
            lifetime: "pipeline-cache-entry",
            peak_multiplicity: "dynamic-by-cache-key",
        }],
    },
    InventoryEntry {
        relative_path: "crates/motolii-gpu/src/transfer.rs",
        raw_allocation_calls: 2,
        owner_seats: &[
            OwnerSeat {
                name: "source-upload",
                lifetime: "caller-owned",
                peak_multiplicity: "dynamic-by-live-caller",
            },
            OwnerSeat {
                name: "copy-out-staging",
                lifetime: "downloader-size-generation",
                peak_multiplicity: "one-per-downloader",
            },
        ],
    },
    InventoryEntry {
        relative_path: "crates/motolii-gpu/src/yuv.rs",
        raw_allocation_calls: 3,
        owner_seats: &[OwnerSeat {
            name: "decode-materialization-pool",
            lifetime: "converter-size-generation",
            peak_multiplicity: "three-planes-two-outputs-one-uniform",
        }],
    },
    InventoryEntry {
        relative_path: "crates/motolii-nodes/src/lib.rs",
        raw_allocation_calls: 5,
        owner_seats: &[
            OwnerSeat {
                name: "render-target",
                lifetime: "render-session-graph-liveness",
                peak_multiplicity: "dynamic-by-live-branch",
            },
            OwnerSeat {
                name: "node-uniform",
                lifetime: "node-or-render-call",
                peak_multiplicity: "dynamic-by-active-node",
            },
        ],
    },
    InventoryEntry {
        relative_path: "crates/motolii-render/src/lib.rs",
        raw_allocation_calls: 1,
        owner_seats: &[OwnerSeat {
            name: "rendered-frame",
            lifetime: "consumer-handle",
            peak_multiplicity: "dynamic-by-in-flight-generation",
        }],
    },
    InventoryEntry {
        relative_path: "crates/motolii-ui/src/display_slot.rs",
        raw_allocation_calls: 1,
        owner_seats: &[OwnerSeat {
            name: "preview-display",
            lifetime: "host-and-ui-registration",
            peak_multiplicity: "dynamic-by-display-generation",
        }],
    },
];

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("crates/motolii-testkit -> workspace root")
        .to_path_buf()
}

fn rust_sources_below(root: &Path, relative_dir: &str) -> Vec<PathBuf> {
    let mut pending = vec![root.join(relative_dir)];
    let mut sources = Vec::new();
    while let Some(dir) = pending.pop() {
        for entry in fs::read_dir(&dir)
            .unwrap_or_else(|error| panic!("read_dir {} failed: {error}", dir.display()))
        {
            let entry = entry.expect("directory entry");
            let file_type = entry.file_type().expect("file type");
            if file_type.is_dir() {
                pending.push(entry.path());
            } else if entry.path().extension().is_some_and(|ext| ext == "rs") {
                sources.push(entry.path());
            }
        }
    }
    sources.sort();
    sources
}

fn raw_allocation_count(source: &str) -> usize {
    [
        ".create_texture(",
        ".create_buffer(",
        ".create_buffer_init(",
    ]
    .into_iter()
    .map(|needle| source.matches(needle).count())
    .sum()
}

#[test]
fn every_product_raw_gpu_allocation_callsite_has_an_owner_seat() {
    let root = workspace_root();
    let expected: BTreeMap<_, _> = INVENTORY
        .iter()
        .map(|entry| (entry.relative_path, entry))
        .collect();
    let mut observed = BTreeMap::new();

    for source_path in rust_sources_below(&root, "crates") {
        if source_path.starts_with(root.join("crates/motolii-testkit")) {
            continue;
        }
        let source = fs::read_to_string(&source_path)
            .unwrap_or_else(|error| panic!("read {} failed: {error}", source_path.display()));
        let count = raw_allocation_count(&source);
        if count == 0 {
            continue;
        }
        let relative = source_path
            .strip_prefix(&root)
            .expect("source below workspace")
            .to_string_lossy()
            .replace('\\', "/");
        observed.insert(relative, count);
    }

    assert_eq!(
        observed.len(),
        expected.len(),
        "raw GPU allocation file set changed; classify its owner before accepting it"
    );
    for (relative, count) in observed {
        let entry = expected
            .get(relative.as_str())
            .unwrap_or_else(|| panic!("unowned raw GPU allocation file: {relative}"));
        assert_eq!(
            count, entry.raw_allocation_calls,
            "raw GPU allocation count changed in {relative}; update the owner inventory"
        );
        assert!(
            !entry.owner_seats.is_empty(),
            "{relative} must have at least one owner seat"
        );
        for seat in entry.owner_seats {
            assert!(!seat.name.is_empty(), "{relative} has an empty owner name");
            assert!(
                !seat.lifetime.is_empty(),
                "{relative}/{} has no lifetime class",
                seat.name
            );
            assert!(
                !seat.peak_multiplicity.is_empty(),
                "{relative}/{} has no peak multiplicity",
                seat.name
            );
        }
    }
}

#[test]
fn product_has_no_unaccounted_native_or_external_texture_import() {
    let root = workspace_root();
    let forbidden = [
        "wgpu_hal",
        "as_hal::<",
        "create_texture_from_hal",
        "IOSurface",
        "ID3D11Texture2D",
        "DMA_BUF",
        "Dmabuf",
        "ExternalTexture",
    ];
    let mut violations = Vec::new();

    for source_path in rust_sources_below(&root, "crates") {
        if source_path.starts_with(root.join("crates/motolii-testkit")) {
            continue;
        }
        let source = fs::read_to_string(&source_path)
            .unwrap_or_else(|error| panic!("read {} failed: {error}", source_path.display()));
        for token in forbidden {
            if source.contains(token) {
                violations.push(format!("{}: {token}", source_path.display()));
            }
        }
    }

    assert!(
        violations.is_empty(),
        "native/external GPU memory must have a measurable ResourceLedger entry before product admission:\n{}",
        violations.join("\n")
    );
}

#[test]
fn inventory_keeps_distinct_lifetime_seats() {
    let seats: Vec<_> = INVENTORY
        .iter()
        .flat_map(|entry| entry.owner_seats.iter().map(|seat| seat.name))
        .collect();
    for required in [
        "source-upload",
        "decode-materialization-pool",
        "render-target",
        "rendered-frame",
        "preview-display",
        "copy-out-staging",
        "pipeline-uniform",
        "node-uniform",
    ] {
        assert!(seats.contains(&required), "missing owner seat: {required}");
    }
}
