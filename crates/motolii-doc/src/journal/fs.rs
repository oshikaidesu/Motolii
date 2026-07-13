//! 擬似FS traitと故障注入(監査S11)。
//!
//! `SaveAbortAfter`だけではD1d完了にしない — 並べ替え・部分write・rename未永続・
//! ENOSPC・checkpoint中kill・recovery中再crashをここへ集約する。

use std::collections::BTreeMap;
use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use thiserror::Error;

/// ジャーナル/checkpointが観測するFS操作種別(順序テストの正本)。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FsOpKind {
    WriteCreate,
    Append,
    SyncFile,
    SyncDir,
    Rename,
    Read,
    CreateDir,
    NoteStage,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FsOp {
    pub kind: FsOpKind,
    pub path: PathBuf,
    pub detail: String,
}

/// checkpoint/commitの耐久段。kill注入の照準。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DurabilityStage {
    JournalAppend,
    JournalFsync,
    MainTempWrite,
    MainTempFsync,
    MainRename,
    MainDirFsync,
    CheckpointAppend,
    CheckpointFsync,
    CatalogWrite,
    CatalogFsync,
    /// recoveryがrecovered成果物を書く直前
    RecoveryWrite,
    RecoveryFsync,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum FaultPlan {
    #[default]
    None,
    /// 指定段の完了直後に型付きabortを返す(電源断/kill相当)。
    KillAfter(DurabilityStage),
    /// 次のAppend/WriteCreateを`max_bytes`で打ち切り、残りを捨てる。
    PartialWrite { max_bytes: usize },
    /// 次のAppend/WriteCreateをENOSPCで失敗させる。
    Enospc,
    /// Renameはメモリ上成功するが、続くSyncDir前にcrashすると失われる。
    RenameNotDurable,
    /// Appendをvolatileバッファに溜め、SyncFileまで永続しない。
    ReorderPendingAppend,
}

#[derive(Debug, Error)]
pub enum FsError {
    #[error(transparent)]
    Io(#[from] io::Error),
    #[error("fault injection aborted after {0:?}")]
    Aborted(DurabilityStage),
}

impl FsError {
    pub fn is_enospc(&self) -> bool {
        matches!(
            self,
            FsError::Io(e) if e.raw_os_error() == Some(28) || e.kind() == io::ErrorKind::StorageFull
        )
    }
}

/// ジャーナル経路が使う最小FS契約。
pub trait JournalFs {
    fn create_dir_all(&mut self, path: &Path) -> Result<(), FsError>;
    fn write_create(&mut self, path: &Path, bytes: &[u8]) -> Result<(), FsError>;
    fn append(&mut self, path: &Path, bytes: &[u8]) -> Result<(), FsError>;
    fn sync_file(&mut self, path: &Path) -> Result<(), FsError>;
    fn sync_dir(&mut self, path: &Path) -> Result<(), FsError>;
    fn rename(&mut self, from: &Path, to: &Path) -> Result<(), FsError>;
    fn read(&mut self, path: &Path) -> Result<Vec<u8>, FsError>;
    fn exists(&mut self, path: &Path) -> bool;
    fn metadata_len(&mut self, path: &Path) -> Result<u64, FsError>;
    /// 耐久段の完了通知。FaultInjectingFsがKillAfterを発火する。
    fn note_stage(&mut self, stage: DurabilityStage) -> Result<(), FsError> {
        let _ = stage;
        Ok(())
    }
}

/// 実FS実装。
#[derive(Debug, Default)]
pub struct StdFs;

impl JournalFs for StdFs {
    fn create_dir_all(&mut self, path: &Path) -> Result<(), FsError> {
        fs::create_dir_all(path)?;
        Ok(())
    }

    fn write_create(&mut self, path: &Path, bytes: &[u8]) -> Result<(), FsError> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let mut f = File::create(path)?;
        f.write_all(bytes)?;
        f.flush()?;
        Ok(())
    }

    fn append(&mut self, path: &Path, bytes: &[u8]) -> Result<(), FsError> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let mut f = OpenOptions::new().create(true).append(true).open(path)?;
        f.write_all(bytes)?;
        f.flush()?;
        Ok(())
    }

    fn sync_file(&mut self, path: &Path) -> Result<(), FsError> {
        let f = OpenOptions::new().write(true).open(path)?;
        f.sync_all()?;
        Ok(())
    }

    fn sync_dir(&mut self, path: &Path) -> Result<(), FsError> {
        #[cfg(unix)]
        {
            let f = File::open(path)?;
            f.sync_all()?;
        }
        #[cfg(not(unix))]
        {
            let _ = path;
        }
        Ok(())
    }

    fn rename(&mut self, from: &Path, to: &Path) -> Result<(), FsError> {
        fs::rename(from, to)?;
        Ok(())
    }

    fn read(&mut self, path: &Path) -> Result<Vec<u8>, FsError> {
        let mut f = File::open(path)?;
        let mut buf = Vec::new();
        f.read_to_end(&mut buf)?;
        Ok(buf)
    }

    fn exists(&mut self, path: &Path) -> bool {
        path.exists()
    }

    fn metadata_len(&mut self, path: &Path) -> Result<u64, FsError> {
        Ok(fs::metadata(path)?.len())
    }
}

/// 操作列を記録するラッパ(commit/checkpoint順序の固定用)。
pub struct RecordingFs<F> {
    inner: F,
    log: Arc<Mutex<Vec<FsOp>>>,
}

impl<F> RecordingFs<F> {
    pub fn new(inner: F) -> (Self, Arc<Mutex<Vec<FsOp>>>) {
        let log = Arc::new(Mutex::new(Vec::new()));
        (
            Self {
                inner,
                log: Arc::clone(&log),
            },
            log,
        )
    }

    fn record(&self, kind: FsOpKind, path: &Path, detail: impl Into<String>) {
        self.log.lock().expect("fs log").push(FsOp {
            kind,
            path: path.to_path_buf(),
            detail: detail.into(),
        });
    }
}

impl<F: JournalFs> JournalFs for RecordingFs<F> {
    fn create_dir_all(&mut self, path: &Path) -> Result<(), FsError> {
        self.record(FsOpKind::CreateDir, path, "");
        self.inner.create_dir_all(path)
    }

    fn write_create(&mut self, path: &Path, bytes: &[u8]) -> Result<(), FsError> {
        self.record(FsOpKind::WriteCreate, path, format!("{}B", bytes.len()));
        self.inner.write_create(path, bytes)
    }

    fn append(&mut self, path: &Path, bytes: &[u8]) -> Result<(), FsError> {
        self.record(FsOpKind::Append, path, format!("{}B", bytes.len()));
        self.inner.append(path, bytes)
    }

    fn sync_file(&mut self, path: &Path) -> Result<(), FsError> {
        self.record(FsOpKind::SyncFile, path, "");
        self.inner.sync_file(path)
    }

    fn sync_dir(&mut self, path: &Path) -> Result<(), FsError> {
        self.record(FsOpKind::SyncDir, path, "");
        self.inner.sync_dir(path)
    }

    fn rename(&mut self, from: &Path, to: &Path) -> Result<(), FsError> {
        self.record(
            FsOpKind::Rename,
            to,
            format!("from={}", from.display()),
        );
        self.inner.rename(from, to)
    }

    fn read(&mut self, path: &Path) -> Result<Vec<u8>, FsError> {
        self.record(FsOpKind::Read, path, "");
        self.inner.read(path)
    }

    fn exists(&mut self, path: &Path) -> bool {
        self.inner.exists(path)
    }

    fn metadata_len(&mut self, path: &Path) -> Result<u64, FsError> {
        self.inner.metadata_len(path)
    }

    fn note_stage(&mut self, stage: DurabilityStage) -> Result<(), FsError> {
        self.record(FsOpKind::NoteStage, Path::new(""), format!("{stage:?}"));
        self.inner.note_stage(stage)
    }
}

/// メモリ上の耐久ストア + 故障注入。
#[derive(Debug, Default)]
pub struct FaultInjectingFs {
    durable: BTreeMap<PathBuf, Vec<u8>>,
    volatile_files: BTreeMap<PathBuf, Vec<u8>>,
    pending_renames: Vec<(PathBuf, PathBuf)>,
    pending_appends: BTreeMap<PathBuf, Vec<u8>>,
    plan: FaultPlan,
    partial_armed: bool,
    enospc_armed: bool,
    kill_stage: Option<DurabilityStage>,
    rename_not_durable: bool,
    reorder: bool,
    last_completed_stage: Option<DurabilityStage>,
}

impl FaultInjectingFs {
    pub fn new(plan: FaultPlan) -> Self {
        let mut s = Self {
            plan: plan.clone(),
            ..Self::default()
        };
        match plan {
            FaultPlan::None => {}
            FaultPlan::KillAfter(stage) => s.kill_stage = Some(stage),
            FaultPlan::PartialWrite { .. } => s.partial_armed = true,
            FaultPlan::Enospc => s.enospc_armed = true,
            FaultPlan::RenameNotDurable => s.rename_not_durable = true,
            FaultPlan::ReorderPendingAppend => s.reorder = true,
        }
        s
    }

    pub fn seed_from_disk(&mut self, root: &Path) -> io::Result<()> {
        if !root.exists() {
            return Ok(());
        }
        for entry in walkdir_files(root)? {
            let bytes = fs::read(&entry)?;
            self.durable.insert(entry, bytes);
        }
        Ok(())
    }

    pub fn flush_durable_to_disk(&self) -> io::Result<()> {
        for (path, bytes) in &self.durable {
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::write(path, bytes)?;
        }
        Ok(())
    }

    pub fn durable_get(&self, path: &Path) -> Option<&[u8]> {
        self.durable.get(path).map(|v| v.as_slice())
    }

    pub fn complete_stage(&mut self, stage: DurabilityStage) -> Result<(), FsError> {
        self.last_completed_stage = Some(stage);
        if self.kill_stage == Some(stage) {
            self.crash();
            return Err(FsError::Aborted(stage));
        }
        Ok(())
    }

    /// プロセスクラッシュ: volatileと未dirsyncのrenameを破棄。
    pub fn crash(&mut self) {
        self.volatile_files.clear();
        self.pending_appends.clear();
        self.pending_renames.clear();
    }

    fn view(&self, path: &Path) -> Option<Vec<u8>> {
        if let Some(v) = self.volatile_files.get(path) {
            return Some(v.clone());
        }
        self.durable.get(path).cloned()
    }

    fn apply_write_limit<'a>(&mut self, bytes: &'a [u8]) -> Result<&'a [u8], FsError> {
        if self.enospc_armed {
            self.enospc_armed = false;
            return Err(FsError::Io(io::Error::new(
                io::ErrorKind::StorageFull,
                "ENOSPC fault injection",
            )));
        }
        if self.partial_armed {
            self.partial_armed = false;
            if let FaultPlan::PartialWrite { max_bytes } = self.plan {
                let n = max_bytes.min(bytes.len());
                return Ok(&bytes[..n]);
            }
        }
        Ok(bytes)
    }
}

impl JournalFs for FaultInjectingFs {
    fn create_dir_all(&mut self, _path: &Path) -> Result<(), FsError> {
        Ok(())
    }

    fn write_create(&mut self, path: &Path, bytes: &[u8]) -> Result<(), FsError> {
        let bytes = self.apply_write_limit(bytes)?;
        self.volatile_files
            .insert(path.to_path_buf(), bytes.to_vec());
        Ok(())
    }

    fn append(&mut self, path: &Path, bytes: &[u8]) -> Result<(), FsError> {
        let bytes = self.apply_write_limit(bytes)?;
        if self.reorder {
            self.pending_appends
                .entry(path.to_path_buf())
                .or_default()
                .extend_from_slice(bytes);
            return Ok(());
        }
        let mut cur = self.view(path).unwrap_or_default();
        cur.extend_from_slice(bytes);
        self.volatile_files.insert(path.to_path_buf(), cur);
        Ok(())
    }

    fn sync_file(&mut self, path: &Path) -> Result<(), FsError> {
        if let Some(pending) = self.pending_appends.remove(path) {
            let mut cur = self.view(path).unwrap_or_default();
            cur.extend_from_slice(&pending);
            self.volatile_files.insert(path.to_path_buf(), cur);
        }
        if let Some(bytes) = self.volatile_files.remove(path) {
            self.durable.insert(path.to_path_buf(), bytes);
        }
        Ok(())
    }

    fn sync_dir(&mut self, _path: &Path) -> Result<(), FsError> {
        let pending = std::mem::take(&mut self.pending_renames);
        for (from, to) in pending {
            if let Some(bytes) = self.durable.remove(&from) {
                self.durable.insert(to, bytes);
            } else if let Some(bytes) = self.volatile_files.remove(&from) {
                self.durable.insert(to, bytes);
            }
        }
        Ok(())
    }

    fn rename(&mut self, from: &Path, to: &Path) -> Result<(), FsError> {
        if self.rename_not_durable {
            self.pending_renames
                .push((from.to_path_buf(), to.to_path_buf()));
            if let Some(bytes) = self.view(from) {
                self.volatile_files.insert(to.to_path_buf(), bytes);
                self.volatile_files.remove(from);
            }
            return Ok(());
        }
        if let Some(bytes) = self.volatile_files.remove(from) {
            self.volatile_files.insert(to.to_path_buf(), bytes);
        } else if let Some(bytes) = self.durable.remove(from) {
            self.durable.insert(to.to_path_buf(), bytes);
        } else {
            return Err(FsError::Io(io::Error::new(
                io::ErrorKind::NotFound,
                "rename source missing",
            )));
        }
        Ok(())
    }

    fn read(&mut self, path: &Path) -> Result<Vec<u8>, FsError> {
        self.view(path).ok_or_else(|| {
            FsError::Io(io::Error::new(
                io::ErrorKind::NotFound,
                format!("missing {}", path.display()),
            ))
        })
    }

    fn exists(&mut self, path: &Path) -> bool {
        self.view(path).is_some() || self.pending_renames.iter().any(|(_, to)| to == path)
    }

    fn metadata_len(&mut self, path: &Path) -> Result<u64, FsError> {
        Ok(self.view(path).map(|b| b.len() as u64).unwrap_or(0))
    }

    fn note_stage(&mut self, stage: DurabilityStage) -> Result<(), FsError> {
        self.complete_stage(stage)
    }
}

fn walkdir_files(root: &Path) -> io::Result<Vec<PathBuf>> {
    let mut out = Vec::new();
    fn walk(dir: &Path, out: &mut Vec<PathBuf>) -> io::Result<()> {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                walk(&path, out)?;
            } else {
                out.push(path);
            }
        }
        Ok(())
    }
    if root.is_file() {
        out.push(root.to_path_buf());
    } else {
        walk(root, &mut out)?;
    }
    Ok(out)
}
