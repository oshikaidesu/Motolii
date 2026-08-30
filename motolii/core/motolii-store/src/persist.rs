
use std::path::{Path, PathBuf};

use crate::components::{archetype_composition, archetype_layer};
use crate::{Document, Intent, Revision, StoreError};

impl Document {
    pub fn flattened(&self) -> Result<Self, StoreError> {
        let view = self.view();
        let mut out = Self::new();
        let at = 1;

        for (component, json) in view.track_json_components(&Document::composition_path())? {
            out.copy_track_json(
                Document::composition_path(),
                component,
                archetype_composition(),
                json,
                at,
            )?;
        }

        for layer in view.layers() {
            out.write(Intent::AddLayer(layer), at)?;
            for (component, json) in view.track_json_components(&layer.entity_path())? {
                out.copy_track_json(layer.entity_path(), component, archetype_layer(), json, at)?;
            }
        }

        out.mark_undo_floor();
        Ok(out)
    }

    pub fn save(&self, path: impl AsRef<Path>) -> Result<(), StoreError> {
        let flat = self.flattened()?;
        let file = std::fs::File::create(path.as_ref())
            .map_err(|e| StoreError::Io(e.to_string()))?;
        Self::encode_flattened_to(&flat, file)
    }

    fn encode_flattened_to(flat: &Self, file: std::fs::File) -> Result<(), StoreError> {
        let mut encoder = re_log_encoding::rrd::Encoder::new_eager(
            re_build_info::CrateVersion::LOCAL,
            re_log_encoding::rrd::EncodingOptions::PROTOBUF_UNCOMPRESSED,
            std::io::BufWriter::new(file),
        )
        .map_err(|e| StoreError::Io(e.to_string()))?;

        for message in flat.db.to_messages(None) {
            let message = message.map_err(|e| StoreError::Io(e.to_string()))?;
            encoder
                .append(&message)
                .map_err(|e| StoreError::Io(e.to_string()))?;
        }
        encoder.finish().map_err(|e| StoreError::Io(e.to_string()))?;
        Ok(())
    }

    fn save_atomic(&self, path: &Path) -> Result<(), StoreError> {
        let flat = self.flattened()?;
        let dir = path.parent().filter(|p| !p.as_os_str().is_empty());
        let dir = dir.unwrap_or_else(|| Path::new("."));
        let file_name = path
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| StoreError::Io("保存先のファイル名を取得できない".into()))?;
        let tmp_path = dir.join(format!(".{file_name}.tmp"));

        let file = std::fs::File::create(&tmp_path).map_err(|e| StoreError::Io(e.to_string()))?;
        if let Err(err) = Self::encode_flattened_to(&flat, file) {
            let _ = std::fs::remove_file(&tmp_path);
            return Err(err);
        }

        std::fs::rename(&tmp_path, path).map_err(|e| StoreError::Io(e.to_string()))
    }

    pub fn load(path: impl AsRef<Path>) -> Result<Self, StoreError> {
        let file = std::fs::File::open(path.as_ref()).map_err(|e| StoreError::Io(e.to_string()))?;
        let decoder = re_log_encoding::rrd::DecoderApp::decode_eager(std::io::BufReader::new(file))
            .map_err(|e| StoreError::Io(e.to_string()))?;

        let mut out: Option<Self> = None;
        for message in decoder {
            let message = message.map_err(|e| StoreError::Io(e.to_string()))?;
            let doc = match &mut out {
                Some(doc) => doc,
                None => {
                    out = Some(Self::with_store_id(message.store_id().clone()));
                    out.as_mut().expect("直前に入れた")
                }
            };
            doc.db
                .add_log_msg(&message)
                .map_err(|e| StoreError::Ingest(e.to_string()))?;
        }

        let mut out = out.ok_or_else(|| StoreError::Io("空の file(メッセージが1つも無い)".into()))?;
        out.rebuild_head_from_store();
        out.mark_undo_floor();
        Ok(out)
    }

    pub fn auto_save(
        &self,
        project_path: Option<&Path>,
        since: &Revision,
        config: &AutoSaveConfig,
    ) -> Result<Option<PathBuf>, StoreError> {
        let Some(project_path) = project_path else {
            return Ok(None);
        };
        if self.revision() == *since {
            return Ok(None);
        }

        let dir = Self::auto_save_dir(project_path);
        std::fs::create_dir_all(&dir).map_err(|e| StoreError::Io(e.to_string()))?;

        let stem = Self::auto_save_stem(project_path);
        let ext = Self::auto_save_ext(project_path);
        let generations = config.generations.max(1);

        let mut existing = Self::existing_generations(&dir, &stem, &ext)?;
        let next_seq = existing.last().map(|(seq, _)| seq + 1).unwrap_or(1);
        let target = Self::generation_path(&dir, &stem, &ext, next_seq);

        self.save_atomic(&target)?;

        existing.push((next_seq, target.clone()));
        if existing.len() > generations {
            let overflow = existing.len() - generations;
            for (_, stale) in existing.iter().take(overflow) {
                let _ = std::fs::remove_file(stale);
            }
        }

        Ok(Some(target))
    }

    pub fn auto_save_dir(project_path: &Path) -> PathBuf {
        let stem = Self::auto_save_stem(project_path);
        let parent = project_path
            .parent()
            .filter(|p| !p.as_os_str().is_empty())
            .unwrap_or_else(|| Path::new("."));
        parent.join(format!("{stem} auto-save"))
    }

    fn auto_save_stem(project_path: &Path) -> String {
        project_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("untitled")
            .to_owned()
    }

    fn auto_save_ext(project_path: &Path) -> String {
        project_path
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| format!(".{e}"))
            .unwrap_or_default()
    }

    fn generation_path(dir: &Path, stem: &str, ext: &str, seq: u64) -> PathBuf {
        dir.join(format!("{stem}.autosave-{seq}{ext}"))
    }

    fn generation_prefix(stem: &str) -> String {
        format!("{stem}.autosave-")
    }

    fn parse_generation_seq(file_name: &str, stem: &str, ext: &str) -> Option<u64> {
        let prefix = Self::generation_prefix(stem);
        let rest = file_name.strip_prefix(prefix.as_str())?;
        let rest = if ext.is_empty() {
            rest
        } else {
            rest.strip_suffix(ext)?
        };
        rest.parse::<u64>().ok()
    }

    fn existing_generations(
        dir: &Path,
        stem: &str,
        ext: &str,
    ) -> Result<Vec<(u64, PathBuf)>, StoreError> {
        let mut out = Vec::new();
        if !dir.exists() {
            return Ok(out);
        }
        for entry in std::fs::read_dir(dir).map_err(|e| StoreError::Io(e.to_string()))? {
            let entry = entry.map_err(|e| StoreError::Io(e.to_string()))?;
            let Some(name) = entry.file_name().to_str().map(str::to_owned) else {
                continue;
            };
            if let Some(seq) = Self::parse_generation_seq(&name, stem, ext) {
                out.push((seq, entry.path()));
            }
        }
        out.sort_by_key(|(seq, _)| *seq);
        Ok(out)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AutoSaveConfig {
    pub interval_secs: u64,
    pub generations: usize,
}

impl AutoSaveConfig {
    pub const DEFAULT_INTERVAL_SECS: u64 = 5 * 60;
    pub const DEFAULT_GENERATIONS: usize = 5;
}

impl Default for AutoSaveConfig {
    fn default() -> Self {
        Self {
            interval_secs: Self::DEFAULT_INTERVAL_SECS,
            generations: Self::DEFAULT_GENERATIONS,
        }
    }
}
