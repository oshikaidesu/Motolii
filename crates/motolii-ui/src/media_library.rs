//! Workspace 所有の media library。一覧は実ファイル。モック配列は正本にしない。
//!
//! place は既存 `place_media` が Document へ書く。窓同士は繋がない。

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

const MAX_ITEMS: usize = 256;
const MAX_DIRECTORIES: usize = 64;

/// compile 時 worktree の Starter Media。実行時に別 dummy を増やさない。
pub fn default_media_library_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../docs/mocks-ui/starter-media/media")
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegisteredFolder {
    pub id: String,
    pub name: String,
    pub path: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MediaLibrary {
    roots: Vec<RegisteredFolder>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LibraryProjection {
    pub root: Option<RegisteredFolder>,
    pub directories: Vec<LibraryDirectory>,
    pub tags: Vec<LibraryTagCount>,
    pub items: Vec<LibraryItem>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LibraryDirectory {
    pub id: String,
    pub name: String,
    pub path: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LibraryTagCount {
    pub id: String,
    pub label: String,
    pub count: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LibraryItem {
    pub id: String,
    pub name: String,
    pub kind: &'static str,
    pub directory: String,
    pub tags: Vec<String>,
}

/// UI の TAGS / DIRECTORIES / TYPE はここへ写す。`project()` / `resolve()` は全件のまま。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LibraryFilter {
    All,
    Tag(String),
    Directory(String),
    Kind(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedLibraryFile {
    pub id: String,
    pub name: String,
    pub kind: &'static str,
    pub path: PathBuf,
    pub asset_type: &'static str,
}

impl MediaLibrary {
    pub fn with_default_root() -> Self {
        Self::with_root(default_media_library_root())
    }

    pub fn with_root(path: PathBuf) -> Self {
        let path = fs::canonicalize(&path).unwrap_or(path);
        let name = path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("media")
            .to_owned();
        Self {
            roots: vec![RegisteredFolder {
                id: "root-0".to_owned(),
                name,
                path,
            }],
        }
    }

    pub fn project(&self) -> LibraryProjection {
        let Some(root) = self.roots.first() else {
            return LibraryProjection {
                root: None,
                directories: Vec::new(),
                tags: Vec::new(),
                items: Vec::new(),
            };
        };
        let scanned = scan_root(root);
        let mut tag_counts: BTreeMap<String, u32> = BTreeMap::new();
        for item in &scanned.items {
            for tag in &item.tags {
                *tag_counts.entry(tag.clone()).or_insert(0) += 1;
            }
        }
        let tags = tag_counts
            .into_iter()
            .map(|(id, count)| LibraryTagCount {
                label: default_tag_label(&id),
                id,
                count,
            })
            .collect();
        LibraryProjection {
            root: Some(root.clone()),
            directories: scanned.directories,
            tags,
            items: scanned.items,
        }
    }

    pub fn filtered_items(&self, filter: &LibraryFilter) -> Vec<LibraryItem> {
        let items = self.project().items;
        match filter {
            LibraryFilter::All => items,
            LibraryFilter::Tag(id) => items
                .into_iter()
                .filter(|item| item.tags.iter().any(|tag| tag == id))
                .collect(),
            LibraryFilter::Directory(path) => items
                .into_iter()
                .filter(|item| item.directory == *path)
                .collect(),
            LibraryFilter::Kind(kind) => {
                items.into_iter().filter(|item| item.kind == kind).collect()
            }
        }
    }

    pub fn resolve(&self, id: &str) -> Option<ResolvedLibraryFile> {
        let projection = self.project();
        let root = projection.root.as_ref()?;
        let item = projection.items.iter().find(|item| item.id == id)?;
        let asset_type = asset_type_for_name(&item.name)?;
        let mut path = root.path.clone();
        if !item.directory.is_empty() {
            path.push(&item.directory);
        }
        path.push(&item.name);
        if !path.is_file() {
            return None;
        }
        Some(ResolvedLibraryFile {
            id: item.id.clone(),
            name: item.name.clone(),
            kind: item.kind,
            path,
            asset_type,
        })
    }
}

struct Scanned {
    directories: Vec<LibraryDirectory>,
    items: Vec<LibraryItem>,
}

fn scan_root(root: &RegisteredFolder) -> Scanned {
    let Ok(canonical) = root.path.canonicalize() else {
        return Scanned {
            directories: Vec::new(),
            items: Vec::new(),
        };
    };
    if !canonical.is_dir() {
        return Scanned {
            directories: Vec::new(),
            items: Vec::new(),
        };
    }

    let mut directories = vec![LibraryDirectory {
        id: format!("{}:", root.id),
        name: root.name.clone(),
        path: String::new(),
    }];
    let mut items = Vec::new();
    let mut seen_dirs = BTreeSet::from([String::new()]);
    walk(
        root,
        &canonical,
        Path::new(""),
        &mut directories,
        &mut items,
        &mut seen_dirs,
    );
    Scanned { directories, items }
}

fn walk(
    root: &RegisteredFolder,
    abs: &Path,
    rel: &Path,
    directories: &mut Vec<LibraryDirectory>,
    items: &mut Vec<LibraryItem>,
    seen_dirs: &mut BTreeSet<String>,
) {
    if items.len() >= MAX_ITEMS {
        return;
    }
    let Ok(entries) = fs::read_dir(abs) else {
        return;
    };
    let mut ordered: Vec<_> = entries.flatten().collect();
    ordered.sort_by_key(|entry| entry.file_name());
    for entry in ordered {
        if items.len() >= MAX_ITEMS {
            break;
        }
        let name = entry.file_name();
        let Some(name) = name.to_str() else {
            continue;
        };
        if name.starts_with('.') {
            continue;
        }
        let child_rel = if rel.as_os_str().is_empty() {
            PathBuf::from(name)
        } else {
            rel.join(name)
        };
        let rel_key = unix_rel(&child_rel);
        let Ok(file_type) = entry.file_type() else {
            continue;
        };
        if file_type.is_dir() {
            if directories.len() < MAX_DIRECTORIES && seen_dirs.insert(rel_key.clone()) {
                directories.push(LibraryDirectory {
                    id: format!("{}:{rel_key}", root.id),
                    name: name.to_owned(),
                    path: rel_key,
                });
            }
            walk(
                root,
                &entry.path(),
                &child_rel,
                directories,
                items,
                seen_dirs,
            );
            continue;
        }
        if !file_type.is_file() {
            continue;
        }
        let Some(kind) = media_kind(name) else {
            continue;
        };
        let directory = unix_rel(rel);
        let id = format!("{}:{rel_key}", root.id);
        let tags = derived_tags(kind, name, &directory);
        items.push(LibraryItem {
            id,
            name: name.to_owned(),
            kind,
            directory,
            tags,
        });
    }
}

fn derived_tags(kind: &str, name: &str, directory: &str) -> Vec<String> {
    let mut tags = Vec::new();
    push_tag(&mut tags, kind.to_owned());
    if let Some(ext) = Path::new(name)
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.to_ascii_lowercase())
    {
        push_tag(&mut tags, ext);
    }
    if let Some(parent) = directory.rsplit('/').next().filter(|part| !part.is_empty()) {
        push_tag(&mut tags, parent.to_owned());
    }
    tags
}

fn push_tag(tags: &mut Vec<String>, tag: String) {
    if tag.is_empty() || tags.iter().any(|existing| existing == &tag) {
        return;
    }
    tags.push(tag);
}

fn default_tag_label(id: &str) -> String {
    match id {
        "video" => "Video".to_owned(),
        "audio" => "Audio".to_owned(),
        "image" => "Image".to_owned(),
        _ => id.to_owned(),
    }
}

fn unix_rel(path: &Path) -> String {
    path.iter()
        .map(|part| part.to_string_lossy())
        .collect::<Vec<_>>()
        .join("/")
}

fn media_kind(name: &str) -> Option<&'static str> {
    asset_type_for_name(name).map(|asset_type| {
        if asset_type.starts_with("video/") {
            "video"
        } else if asset_type.starts_with("audio/") {
            "audio"
        } else {
            "image"
        }
    })
}

pub fn asset_type_for_name(name: &str) -> Option<&'static str> {
    let ext = Path::new(name)
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.to_ascii_lowercase())?;
    match ext.as_str() {
        "mp4" | "m4v" => Some("video/mp4"),
        "mov" => Some("video/quicktime"),
        "mkv" => Some("video/x-matroska"),
        "webm" => Some("video/webm"),
        "wav" => Some("audio/wav"),
        "mp3" => Some("audio/mpeg"),
        "aac" | "m4a" => Some("audio/mp4"),
        "aiff" | "aif" => Some("audio/aiff"),
        "flac" => Some("audio/flac"),
        "png" => Some("image/png"),
        "jpg" | "jpeg" => Some("image/jpeg"),
        "svg" => Some("image/svg+xml"),
        "webp" => Some("image/webp"),
        "gif" => Some("image/gif"),
        "exr" => Some("image/x-exr"),
        "tif" | "tiff" => Some("image/tiff"),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_root_lists_starter_media_files_not_a_fake_grid() {
        let library = MediaLibrary::with_default_root();
        let projection = library.project();
        let root = projection.root.expect("starter media root");
        let canonical = root.path.canonicalize().expect("starter media exists");
        assert!(
            canonical.ends_with("docs/mocks-ui/starter-media/media"),
            "{canonical:?}"
        );
        let names: Vec<_> = projection
            .items
            .iter()
            .map(|item| item.name.as_str())
            .collect();
        assert_eq!(
            names,
            [
                "starter-clip.mp4",
                "starter-mark.svg",
                "starter-still.png",
                "starter-tone.wav"
            ]
        );
        assert_eq!(
            projection
                .items
                .iter()
                .map(|item| item.kind)
                .collect::<Vec<_>>(),
            ["video", "image", "image", "audio"]
        );
        assert_eq!(projection.directories.len(), 1);
        assert_eq!(projection.directories[0].name, "media");
        assert_eq!(
            projection
                .items
                .iter()
                .map(|item| item.tags.as_slice())
                .collect::<Vec<_>>(),
            [
                ["video", "mp4"].as_slice(),
                ["image", "svg"].as_slice(),
                ["image", "png"].as_slice(),
                ["audio", "wav"].as_slice(),
            ]
        );
        let tag_ids: Vec<_> = projection
            .tags
            .iter()
            .map(|tag| (tag.id.as_str(), tag.label.as_str(), tag.count))
            .collect();
        assert_eq!(
            tag_ids,
            [
                ("audio", "Audio", 1),
                ("image", "Image", 2),
                ("mp4", "mp4", 1),
                ("png", "png", 1),
                ("svg", "svg", 1),
                ("video", "Video", 1),
                ("wav", "wav", 1),
            ]
        );
        assert!(projection.tags.iter().all(|tag| tag.id != "interview"));
        assert!(projection
            .items
            .iter()
            .all(|item| !item.name.starts_with("asset-")));
        for item in &projection.items {
            let resolved = library.resolve(&item.id).expect("listed file resolves");
            assert!(resolved.path.is_file(), "{:?}", resolved.path);
        }
        let video = library.filtered_items(&LibraryFilter::Tag("video".to_owned()));
        assert_eq!(
            video
                .iter()
                .map(|item| item.name.as_str())
                .collect::<Vec<_>>(),
            ["starter-clip.mp4"]
        );
        assert_eq!(video[0].tags, ["video", "mp4"]);
        let images = library.filtered_items(&LibraryFilter::Kind("image".to_owned()));
        assert_eq!(
            images
                .iter()
                .map(|item| item.name.as_str())
                .collect::<Vec<_>>(),
            ["starter-mark.svg", "starter-still.png"]
        );
        let before = library.project();
        assert!(library
            .filtered_items(&LibraryFilter::Tag("absent".to_owned()))
            .is_empty());
        assert!(library
            .filtered_items(&LibraryFilter::Tag("interview".to_owned()))
            .is_empty());
        assert_eq!(library.project(), before);
        let clip = library
            .resolve("root-0:starter-clip.mp4")
            .expect("starter clip");
        assert_eq!(clip.kind, "video");
        assert_eq!(clip.asset_type, "video/mp4");
        assert!(clip.path.ends_with("starter-clip.mp4"));
        assert!(clip.path.is_file());
        assert!(library.resolve("root-0:starter-tone.wav").is_some());
        assert!(library.resolve("root-0:missing.mp4").is_none());
    }

    #[test]
    fn missing_root_projects_empty_not_mock_assets() {
        let library = MediaLibrary::with_root(PathBuf::from("/no/such/motolii-media-library"));
        let projection = library.project();
        assert!(projection.items.is_empty());
        assert!(projection.directories.is_empty());
        assert!(projection.tags.is_empty());
        assert!(library.filtered_items(&LibraryFilter::All).is_empty());
        assert!(library
            .filtered_items(&LibraryFilter::Tag("video".to_owned()))
            .is_empty());
        assert!(library.resolve("root-0:hero.mp4").is_none());
    }

    #[test]
    fn nested_directory_lists_real_files_not_hand_tags() {
        let dir = std::env::temp_dir().join(format!(
            "motolii-media-lib-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        ));
        fs::create_dir_all(dir.join("b-roll")).expect("subdir");
        fs::write(dir.join("hero.mp4"), b"mp4").expect("file");
        fs::write(dir.join("b-roll").join("cut.wav"), b"wav").expect("nested");
        let library = MediaLibrary::with_root(dir.clone());
        let projection = library.project();
        let b_roll_tag = library.filtered_items(&LibraryFilter::Tag("b-roll".to_owned()));
        let b_roll_dir = library.filtered_items(&LibraryFilter::Directory("b-roll".to_owned()));
        let before = library.project();
        assert_eq!(library.project(), before);
        let hero = library.resolve("root-0:hero.mp4").expect("hero");
        assert!(hero.path.is_file());
        let _ = fs::remove_dir_all(&dir);
        let dirs: Vec<_> = projection
            .directories
            .iter()
            .map(|directory| directory.path.as_str())
            .collect();
        assert_eq!(dirs, ["", "b-roll"]);
        assert!(projection.tags.iter().all(|tag| tag.id != "interview"));
        let hero_item = projection
            .items
            .iter()
            .find(|item| item.name == "hero.mp4")
            .expect("hero");
        assert_eq!(hero_item.tags, ["video", "mp4"]);
        let cut = projection
            .items
            .iter()
            .find(|item| item.name == "cut.wav")
            .expect("cut");
        assert_eq!(cut.directory, "b-roll");
        assert_eq!(cut.tags, ["audio", "wav", "b-roll"]);
        assert_eq!(
            projection
                .items
                .iter()
                .filter(|item| item.tags.iter().any(|tag| tag == "video"))
                .map(|item| item.name.as_str())
                .collect::<Vec<_>>(),
            ["hero.mp4"]
        );
        assert_eq!(
            b_roll_tag
                .iter()
                .map(|item| item.name.as_str())
                .collect::<Vec<_>>(),
            ["cut.wav"]
        );
        assert_eq!(
            b_roll_dir
                .iter()
                .map(|item| item.name.as_str())
                .collect::<Vec<_>>(),
            ["cut.wav"]
        );
        assert!(library.resolve("root-0:hero.mp4").is_none());
    }
}
