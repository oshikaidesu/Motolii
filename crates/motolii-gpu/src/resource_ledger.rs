//! K1a Host-private ResourceLedger: hard-budget admission policy only.
//!
//! allocator report / wgpu / eviction / cache には結合しない。注入budgetと
//! 事前予約が正本で、未使用分の即時返却と Drop/明示release でaccountingを閉じる。

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// 注入するhard cap。`shared_memory_bytes` は RAM+VRAM 合算上限(任意)。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResourceBudgets {
    pub vram_bytes: u64,
    pub ram_bytes: u64,
    pub disk_bytes: u64,
    pub shared_memory_bytes: Option<u64>,
}

/// 予算階層。store/allocator 本体は後続adapterが持つ。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ResourceTier {
    Vram,
    Ram,
    Disk,
}

/// resident=後続evict対象候補、pinned=解放まで退避不可。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ResidentClass {
    Resident,
    Pinned,
}

/// 診断・owner集計用のHost-privateラベル。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ResourceOwner(Arc<str>);

impl ResourceOwner {
    pub fn new(name: impl Into<String>) -> Self {
        Self(Arc::from(name.into()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// 予約の用途ラベル(診断用)。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourcePurpose(Arc<str>);

impl ResourcePurpose {
    pub fn new(name: impl Into<String>) -> Self {
        Self(Arc::from(name.into()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// wgpu/backend型を持たないchecked見積り入力。
///
/// `None` の寄与は上限不明として admission 前に拒否する。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResourceEstimateParts {
    pub format_bytes: Option<u64>,
    pub dimensions_or_count: Option<u64>,
    pub mip_levels: Option<u32>,
    pub sample_count: Option<u32>,
    pub alignment_bytes: Option<u64>,
    pub overhead_bytes: Option<u64>,
}

/// 見積り失敗。台帳は変異しない。
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum EstimateError {
    #[error("resource size estimate is unknown or unbounded")]
    Unknown,
    #[error("resource size estimate overflowed")]
    Overflow,
}

impl ResourceEstimateParts {
    /// format×要素×sample をmip連鎖で合算し、各levelをalignmentへ切り上げてoverheadを足す。
    ///
    /// 各mipは供給されたbase要素数上界をそのまま使う。形状を仮定した `/4` 縮小は
    /// 1×N 等で過小見積りになるため、K1aでは常に保守上界を返す。
    pub fn estimate_reserved_bytes(self) -> Result<u64, EstimateError> {
        let format_bytes = self.format_bytes.ok_or(EstimateError::Unknown)?;
        let elements = self.dimensions_or_count.ok_or(EstimateError::Unknown)?;
        let mip_levels = self.mip_levels.ok_or(EstimateError::Unknown)?;
        let sample_count = self.sample_count.ok_or(EstimateError::Unknown)?;
        let alignment = self.alignment_bytes.ok_or(EstimateError::Unknown)?;
        let overhead = self.overhead_bytes.ok_or(EstimateError::Unknown)?;

        if mip_levels == 0 || sample_count == 0 || alignment == 0 {
            return Err(EstimateError::Unknown);
        }

        let mut total = 0u64;
        for _level in 0..mip_levels {
            let raw = format_bytes
                .checked_mul(elements)
                .and_then(|v| v.checked_mul(u64::from(sample_count)))
                .ok_or(EstimateError::Overflow)?;
            let aligned = align_up(raw, alignment).ok_or(EstimateError::Overflow)?;
            total = total.checked_add(aligned).ok_or(EstimateError::Overflow)?;
        }
        total.checked_add(overhead).ok_or(EstimateError::Overflow)
    }
}

fn align_up(value: u64, alignment: u64) -> Option<u64> {
    if alignment == 0 {
        return None;
    }
    let rem = value % alignment;
    if rem == 0 {
        Some(value)
    } else {
        value.checked_add(alignment - rem)
    }
}

/// 拒否診断。要求元・要求量・現使用量・予算を必ず持つ。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BudgetDiagnostic {
    pub owner: ResourceOwner,
    pub requested_bytes: u64,
    pub used_bytes: u64,
    pub budget_bytes: u64,
}

/// admission拒否。失敗時にaccountingは変えない。
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum AdmissionError {
    #[error("resource estimate is unknown or unbounded")]
    UnknownEstimate { diagnostic: BudgetDiagnostic },
    #[error("resource estimate overflowed")]
    EstimateOverflow { diagnostic: BudgetDiagnostic },
    #[error("tier hard cap exceeded for {tier:?}")]
    TierCapExceeded {
        tier: ResourceTier,
        diagnostic: BudgetDiagnostic,
    },
    #[error("shared memory hard cap exceeded")]
    SharedCapExceeded { diagnostic: BudgetDiagnostic },
    /// 超過を阻む生存量が全てpinnedで、後続evictでも空けられない。
    #[error("hard cap exceeded while blocking live set is entirely pinned")]
    FullPin {
        tier: Option<ResourceTier>,
        diagnostic: BudgetDiagnostic,
    },
    /// hard-cap合算がchecked加算で溢れた。台帳は変えない。
    #[error("hard-cap accounting arithmetic overflowed")]
    AccountingOverflow { diagnostic: BudgetDiagnostic },
}

/// 実使用確定の拒否。accountingは変えない。
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum UsageError {
    #[error("actual usage {used} exceeds reservation {reserved}")]
    ExceedsReservation { used: u64, reserved: u64 },
    /// 未使用返却後に、現在のaccounted量を超えて再取得しようとした。
    #[error("actual usage {used} exceeds current accounted {accounted}")]
    ExceedsAccounted { used: u64, accounted: u64 },
}

#[derive(Debug)]
struct LiveEntry {
    owner: ResourceOwner,
    tier: ResourceTier,
    class: ResidentClass,
    /// 現在台帳に載せているbytes(予約残またはcommit後の実使用)。
    bytes: u64,
}

#[derive(Debug, Default)]
struct LedgerState {
    next_id: u64,
    entries: HashMap<u64, LiveEntry>,
}

impl LedgerState {
    /// 診断向け合算。admission判定には使わず、溢れた場合はu64::MAXへ飽和するだけ。
    fn sum_saturating(&self, mut pred: impl FnMut(&LiveEntry) -> bool) -> u64 {
        self.entries
            .values()
            .filter(|e| pred(e))
            .map(|e| e.bytes)
            .fold(0u64, |a, b| a.saturating_add(b))
    }

    /// admission正本の合算。溢れは拒否側へ返す。
    fn sum_checked(&self, mut pred: impl FnMut(&LiveEntry) -> bool) -> Option<u64> {
        let mut total = 0u64;
        for entry in self.entries.values() {
            if pred(entry) {
                total = total.checked_add(entry.bytes)?;
            }
        }
        Some(total)
    }

    fn tier_live(&self, tier: ResourceTier) -> u64 {
        self.sum_saturating(|e| e.tier == tier)
    }

    fn tier_live_checked(&self, tier: ResourceTier) -> Option<u64> {
        self.sum_checked(|e| e.tier == tier)
    }

    fn tier_pinned_checked(&self, tier: ResourceTier) -> Option<u64> {
        self.sum_checked(|e| e.tier == tier && e.class == ResidentClass::Pinned)
    }

    fn shared_live(&self) -> u64 {
        self.tier_live(ResourceTier::Vram)
            .saturating_add(self.tier_live(ResourceTier::Ram))
    }

    fn shared_live_checked(&self) -> Option<u64> {
        let vram = self.tier_live_checked(ResourceTier::Vram)?;
        let ram = self.tier_live_checked(ResourceTier::Ram)?;
        vram.checked_add(ram)
    }

    fn shared_pinned_checked(&self) -> Option<u64> {
        let vram = self.tier_pinned_checked(ResourceTier::Vram)?;
        let ram = self.tier_pinned_checked(ResourceTier::Ram)?;
        vram.checked_add(ram)
    }

    fn owner_live(&self, owner: &ResourceOwner) -> u64 {
        self.sum_saturating(|e| &e.owner == owner)
    }

    fn total_live(&self) -> u64 {
        self.sum_saturating(|_| true)
    }
}

/// Host内で唯一のhard-budget admission policy owner。
#[derive(Debug, Clone)]
pub struct ResourceLedger {
    budgets: ResourceBudgets,
    state: Arc<Mutex<LedgerState>>,
}

impl ResourceLedger {
    pub fn new(budgets: ResourceBudgets) -> Self {
        Self {
            budgets,
            state: Arc::new(Mutex::new(LedgerState::default())),
        }
    }

    pub fn budgets(&self) -> ResourceBudgets {
        self.budgets
    }

    /// 診断用。合算溢れは飽和し、admission正本には使わない。
    pub fn tier_live_bytes(&self, tier: ResourceTier) -> u64 {
        self.state
            .lock()
            .expect("resource ledger poisoned")
            .tier_live(tier)
    }

    /// 診断用。合算溢れは飽和し、admission正本には使わない。
    pub fn owner_live_bytes(&self, owner: &ResourceOwner) -> u64 {
        self.state
            .lock()
            .expect("resource ledger poisoned")
            .owner_live(owner)
    }

    /// 診断用。合算溢れは飽和し、admission正本には使わない。
    pub fn total_live_bytes(&self) -> u64 {
        self.state
            .lock()
            .expect("resource ledger poisoned")
            .total_live()
    }

    fn tier_cap(&self, tier: ResourceTier) -> u64 {
        match tier {
            ResourceTier::Vram => self.budgets.vram_bytes,
            ResourceTier::Ram => self.budgets.ram_bytes,
            ResourceTier::Disk => self.budgets.disk_bytes,
        }
    }

    /// 既にboundedな予約上限bytesでadmissionする。
    pub fn admit(
        &self,
        owner: ResourceOwner,
        tier: ResourceTier,
        purpose: ResourcePurpose,
        class: ResidentClass,
        reserved_bytes: u64,
    ) -> Result<AdmissionPermit, AdmissionError> {
        let mut state = self.state.lock().expect("resource ledger poisoned");
        self.try_reserve_locked(&mut state, &owner, tier, reserved_bytes)?;

        let id = state.next_id;
        state.next_id = state.next_id.checked_add(1).expect("permit id exhausted");
        state.entries.insert(
            id,
            LiveEntry {
                owner: owner.clone(),
                tier,
                class,
                bytes: reserved_bytes,
            },
        );

        Ok(AdmissionPermit {
            ledger: self.state.clone(),
            id,
            owner,
            tier,
            purpose,
            class,
            reserved_bytes,
            accounted_bytes: reserved_bytes,
            released: false,
        })
    }

    /// 見積り境界を通してからadmissionする。失敗時は台帳を変えない。
    pub fn admit_estimated(
        &self,
        owner: ResourceOwner,
        tier: ResourceTier,
        purpose: ResourcePurpose,
        class: ResidentClass,
        parts: ResourceEstimateParts,
    ) -> Result<AdmissionPermit, AdmissionError> {
        let tier_cap = self.tier_cap(tier);
        let used = self.tier_live_bytes(tier);
        match parts.estimate_reserved_bytes() {
            Ok(reserved) => self.admit(owner, tier, purpose, class, reserved),
            Err(EstimateError::Unknown) => Err(AdmissionError::UnknownEstimate {
                diagnostic: BudgetDiagnostic {
                    owner,
                    requested_bytes: 0,
                    used_bytes: used,
                    budget_bytes: tier_cap,
                },
            }),
            Err(EstimateError::Overflow) => Err(AdmissionError::EstimateOverflow {
                diagnostic: BudgetDiagnostic {
                    owner,
                    requested_bytes: 0,
                    used_bytes: used,
                    budget_bytes: tier_cap,
                },
            }),
        }
    }

    fn try_reserve_locked(
        &self,
        state: &mut LedgerState,
        owner: &ResourceOwner,
        tier: ResourceTier,
        reserved_bytes: u64,
    ) -> Result<(), AdmissionError> {
        let tier_cap = self.tier_cap(tier);
        let tier_used =
            state
                .tier_live_checked(tier)
                .ok_or_else(|| AdmissionError::AccountingOverflow {
                    diagnostic: BudgetDiagnostic {
                        owner: owner.clone(),
                        requested_bytes: reserved_bytes,
                        used_bytes: state.tier_live(tier),
                        budget_bytes: tier_cap,
                    },
                })?;
        let Some(tier_next) = tier_used.checked_add(reserved_bytes) else {
            return Err(AdmissionError::AccountingOverflow {
                diagnostic: BudgetDiagnostic {
                    owner: owner.clone(),
                    requested_bytes: reserved_bytes,
                    used_bytes: tier_used,
                    budget_bytes: tier_cap,
                },
            });
        };
        if tier_next > tier_cap {
            let diagnostic = BudgetDiagnostic {
                owner: owner.clone(),
                requested_bytes: reserved_bytes,
                used_bytes: tier_used,
                budget_bytes: tier_cap,
            };
            let tier_pinned = state.tier_pinned_checked(tier).ok_or_else(|| {
                AdmissionError::AccountingOverflow {
                    diagnostic: BudgetDiagnostic {
                        owner: owner.clone(),
                        requested_bytes: reserved_bytes,
                        used_bytes: tier_used,
                        budget_bytes: tier_cap,
                    },
                }
            })?;
            if tier_used > 0 && tier_pinned == tier_used {
                return Err(AdmissionError::FullPin {
                    tier: Some(tier),
                    diagnostic,
                });
            }
            return Err(AdmissionError::TierCapExceeded { tier, diagnostic });
        }

        if matches!(tier, ResourceTier::Vram | ResourceTier::Ram) {
            if let Some(shared_cap) = self.budgets.shared_memory_bytes {
                let shared_used = state.shared_live_checked().ok_or_else(|| {
                    AdmissionError::AccountingOverflow {
                        diagnostic: BudgetDiagnostic {
                            owner: owner.clone(),
                            requested_bytes: reserved_bytes,
                            used_bytes: state.shared_live(),
                            budget_bytes: shared_cap,
                        },
                    }
                })?;
                let Some(shared_next) = shared_used.checked_add(reserved_bytes) else {
                    return Err(AdmissionError::AccountingOverflow {
                        diagnostic: BudgetDiagnostic {
                            owner: owner.clone(),
                            requested_bytes: reserved_bytes,
                            used_bytes: shared_used,
                            budget_bytes: shared_cap,
                        },
                    });
                };
                if shared_next > shared_cap {
                    let diagnostic = BudgetDiagnostic {
                        owner: owner.clone(),
                        requested_bytes: reserved_bytes,
                        used_bytes: shared_used,
                        budget_bytes: shared_cap,
                    };
                    let shared_pinned = state.shared_pinned_checked().ok_or_else(|| {
                        AdmissionError::AccountingOverflow {
                            diagnostic: BudgetDiagnostic {
                                owner: owner.clone(),
                                requested_bytes: reserved_bytes,
                                used_bytes: shared_used,
                                budget_bytes: shared_cap,
                            },
                        }
                    })?;
                    if shared_used > 0 && shared_pinned == shared_used {
                        return Err(AdmissionError::FullPin {
                            tier: None,
                            diagnostic,
                        });
                    }
                    return Err(AdmissionError::SharedCapExceeded { diagnostic });
                }
            }
        }

        Ok(())
    }
}

/// 予約済みadmission許可。実使用は予約以下に限り、未使用は即時返却する。
#[derive(Debug)]
pub struct AdmissionPermit {
    ledger: Arc<Mutex<LedgerState>>,
    id: u64,
    owner: ResourceOwner,
    tier: ResourceTier,
    purpose: ResourcePurpose,
    class: ResidentClass,
    reserved_bytes: u64,
    accounted_bytes: u64,
    released: bool,
}

impl AdmissionPermit {
    pub fn owner(&self) -> &ResourceOwner {
        &self.owner
    }

    pub fn tier(&self) -> ResourceTier {
        self.tier
    }

    pub fn purpose(&self) -> &ResourcePurpose {
        &self.purpose
    }

    pub fn class(&self) -> ResidentClass {
        self.class
    }

    pub fn reserved_bytes(&self) -> u64 {
        self.reserved_bytes
    }

    pub fn accounted_bytes(&self) -> u64 {
        self.accounted_bytes
    }

    /// 実使用を現accounted以下かつ予約以下で確定し、未使用bytesを直ちに台帳へ返す。
    ///
    /// 一度返却したbytesの再取得は許可しない。増加要求は台帳を変えず拒否する。
    pub fn commit_usage(&mut self, used: u64) -> Result<(), UsageError> {
        if self.released {
            return Ok(());
        }
        if used > self.reserved_bytes {
            return Err(UsageError::ExceedsReservation {
                used,
                reserved: self.reserved_bytes,
            });
        }
        if used > self.accounted_bytes {
            return Err(UsageError::ExceedsAccounted {
                used,
                accounted: self.accounted_bytes,
            });
        }
        if used == self.accounted_bytes {
            return Ok(());
        }
        let mut state = self.ledger.lock().expect("resource ledger poisoned");
        let entry = state
            .entries
            .get_mut(&self.id)
            .expect("live permit missing from ledger");
        entry.bytes = used;
        self.accounted_bytes = used;
        Ok(())
    }

    /// 残accountingを明示解放する。
    pub fn release(mut self) {
        self.release_remaining();
    }

    fn release_remaining(&mut self) {
        if self.released {
            return;
        }
        self.released = true;
        let mut state = self.ledger.lock().expect("resource ledger poisoned");
        state.entries.remove(&self.id);
        self.accounted_bytes = 0;
    }
}

impl Drop for AdmissionPermit {
    fn drop(&mut self) {
        self.release_remaining();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn budgets(vram: u64, ram: u64, disk: u64, shared: Option<u64>) -> ResourceBudgets {
        ResourceBudgets {
            vram_bytes: vram,
            ram_bytes: ram,
            disk_bytes: disk,
            shared_memory_bytes: shared,
        }
    }

    fn owner(name: &str) -> ResourceOwner {
        ResourceOwner::new(name)
    }

    fn purpose(name: &str) -> ResourcePurpose {
        ResourcePurpose::new(name)
    }

    fn known_parts(
        format_bytes: u64,
        count: u64,
        mip: u32,
        samples: u32,
        align: u64,
        overhead: u64,
    ) -> ResourceEstimateParts {
        ResourceEstimateParts {
            format_bytes: Some(format_bytes),
            dimensions_or_count: Some(count),
            mip_levels: Some(mip),
            sample_count: Some(samples),
            alignment_bytes: Some(align),
            overhead_bytes: Some(overhead),
        }
    }

    #[test]
    fn each_tier_cap_refuses_excess_without_mutation() {
        let ledger = ResourceLedger::new(budgets(100, 200, 300, None));
        let a = ledger
            .admit(
                owner("a"),
                ResourceTier::Vram,
                purpose("tex"),
                ResidentClass::Resident,
                100,
            )
            .unwrap();
        assert_eq!(ledger.tier_live_bytes(ResourceTier::Vram), 100);

        let err = ledger
            .admit(
                owner("b"),
                ResourceTier::Vram,
                purpose("tex"),
                ResidentClass::Resident,
                1,
            )
            .unwrap_err();
        match err {
            AdmissionError::TierCapExceeded { tier, diagnostic } => {
                assert_eq!(tier, ResourceTier::Vram);
                assert_eq!(diagnostic.owner.as_str(), "b");
                assert_eq!(diagnostic.requested_bytes, 1);
                assert_eq!(diagnostic.used_bytes, 100);
                assert_eq!(diagnostic.budget_bytes, 100);
            }
            other => panic!("expected TierCapExceeded, got {other:?}"),
        }
        assert_eq!(ledger.tier_live_bytes(ResourceTier::Vram), 100);
        assert_eq!(ledger.owner_live_bytes(&owner("b")), 0);

        drop(a);
        let _ram = ledger
            .admit(
                owner("ram"),
                ResourceTier::Ram,
                purpose("buf"),
                ResidentClass::Resident,
                200,
            )
            .unwrap();
        assert!(ledger
            .admit(
                owner("ram2"),
                ResourceTier::Ram,
                purpose("buf"),
                ResidentClass::Resident,
                1,
            )
            .is_err());
        let _disk = ledger
            .admit(
                owner("disk"),
                ResourceTier::Disk,
                purpose("cache"),
                ResidentClass::Resident,
                300,
            )
            .unwrap();
        assert!(ledger
            .admit(
                owner("disk2"),
                ResourceTier::Disk,
                purpose("cache"),
                ResidentClass::Resident,
                1,
            )
            .is_err());
    }

    #[test]
    fn shared_ram_vram_aggregate_cap() {
        let ledger = ResourceLedger::new(budgets(100, 100, 1000, Some(150)));
        let _v = ledger
            .admit(
                owner("v"),
                ResourceTier::Vram,
                purpose("tex"),
                ResidentClass::Resident,
                100,
            )
            .unwrap();
        // tier RAM capは空いているが共有合算150を超える。
        let err = ledger
            .admit(
                owner("r"),
                ResourceTier::Ram,
                purpose("pcm"),
                ResidentClass::Resident,
                60,
            )
            .unwrap_err();
        match err {
            AdmissionError::SharedCapExceeded { diagnostic } => {
                assert_eq!(diagnostic.owner.as_str(), "r");
                assert_eq!(diagnostic.requested_bytes, 60);
                assert_eq!(diagnostic.used_bytes, 100);
                assert_eq!(diagnostic.budget_bytes, 150);
            }
            other => panic!("expected SharedCapExceeded, got {other:?}"),
        }
        assert_eq!(ledger.tier_live_bytes(ResourceTier::Ram), 0);
        assert_eq!(ledger.owner_live_bytes(&owner("r")), 0);

        let ok = ledger
            .admit(
                owner("r"),
                ResourceTier::Ram,
                purpose("pcm"),
                ResidentClass::Resident,
                50,
            )
            .unwrap();
        assert_eq!(ok.reserved_bytes(), 50);
        // diskは共有合算の対象外。
        let _d = ledger
            .admit(
                owner("d"),
                ResourceTier::Disk,
                purpose("obj"),
                ResidentClass::Resident,
                1000,
            )
            .unwrap();
    }

    #[test]
    fn boundary_equality_is_admitted() {
        let ledger = ResourceLedger::new(budgets(64, 64, 64, Some(100)));
        let p = ledger
            .admit(
                owner("eq"),
                ResourceTier::Vram,
                purpose("tex"),
                ResidentClass::Resident,
                64,
            )
            .unwrap();
        assert_eq!(p.reserved_bytes(), 64);
        assert_eq!(ledger.tier_live_bytes(ResourceTier::Vram), 64);
        drop(p);
        let _r = ledger
            .admit(
                owner("eq"),
                ResourceTier::Ram,
                purpose("buf"),
                ResidentClass::Resident,
                36,
            )
            .unwrap();
        let _v = ledger
            .admit(
                owner("eq"),
                ResourceTier::Vram,
                purpose("tex"),
                ResidentClass::Resident,
                64,
            )
            .unwrap();
        assert_eq!(ledger.tier_live_bytes(ResourceTier::Vram), 64);
        assert_eq!(ledger.tier_live_bytes(ResourceTier::Ram), 36);
    }

    #[test]
    fn estimate_includes_alignment_mip_sample_overhead() {
        // 各mipはbase要素上界を使う: 4*16*2=128 → align 256; ×2 mips +overhead 16 = 528
        let parts = known_parts(4, 16, 2, 2, 256, 16);
        assert_eq!(parts.estimate_reserved_bytes().unwrap(), 528);

        let ledger = ResourceLedger::new(budgets(528, 0, 0, None));
        let p = ledger
            .admit_estimated(
                owner("est"),
                ResourceTier::Vram,
                purpose("tex"),
                ResidentClass::Resident,
                parts,
            )
            .unwrap();
        assert_eq!(p.reserved_bytes(), 528);
        assert!(ledger
            .admit_estimated(
                owner("est2"),
                ResourceTier::Vram,
                purpose("tex"),
                ResidentClass::Resident,
                known_parts(4, 16, 2, 2, 256, 16),
            )
            .is_err());
    }

    #[test]
    fn thin_resource_mip_estimate_is_conservative() {
        // 旧 `/4` は 1×16×3mip を 16+4+1=21 と過小見積りする。保守上界は base×mips=48。
        let parts = known_parts(1, 16, 3, 1, 1, 0);
        assert_eq!(parts.estimate_reserved_bytes().unwrap(), 48);
        assert!(parts.estimate_reserved_bytes().unwrap() > 21);

        let ledger = ResourceLedger::new(budgets(48, 0, 0, None));
        let p = ledger
            .admit_estimated(
                owner("thin"),
                ResourceTier::Vram,
                purpose("tex"),
                ResidentClass::Resident,
                parts,
            )
            .unwrap();
        assert_eq!(p.reserved_bytes(), 48);
        assert_eq!(ledger.tier_live_bytes(ResourceTier::Vram), 48);
    }

    #[test]
    fn unknown_and_overflow_refuse_with_zero_mutation() {
        let ledger = ResourceLedger::new(budgets(1024, 1024, 1024, None));
        let before = ledger.total_live_bytes();

        let unknown = ResourceEstimateParts {
            format_bytes: None,
            dimensions_or_count: Some(8),
            mip_levels: Some(1),
            sample_count: Some(1),
            alignment_bytes: Some(1),
            overhead_bytes: Some(0),
        };
        match ledger
            .admit_estimated(
                owner("u"),
                ResourceTier::Vram,
                purpose("x"),
                ResidentClass::Resident,
                unknown,
            )
            .unwrap_err()
        {
            AdmissionError::UnknownEstimate { diagnostic } => {
                assert_eq!(diagnostic.owner.as_str(), "u");
                assert_eq!(diagnostic.used_bytes, 0);
                assert_eq!(diagnostic.budget_bytes, 1024);
            }
            other => panic!("expected UnknownEstimate, got {other:?}"),
        }

        let overflow = known_parts(u64::MAX, 4, 1, 2, 1, 0);
        match ledger
            .admit_estimated(
                owner("o"),
                ResourceTier::Ram,
                purpose("x"),
                ResidentClass::Resident,
                overflow,
            )
            .unwrap_err()
        {
            AdmissionError::EstimateOverflow { diagnostic } => {
                assert_eq!(diagnostic.owner.as_str(), "o");
                assert_eq!(diagnostic.budget_bytes, 1024);
            }
            other => panic!("expected EstimateOverflow, got {other:?}"),
        }

        assert_eq!(ledger.total_live_bytes(), before);
        assert_eq!(ledger.owner_live_bytes(&owner("u")), 0);
        assert_eq!(ledger.owner_live_bytes(&owner("o")), 0);
    }

    #[test]
    fn full_pin_typed_refusal_when_blocking_live_set_is_pinned() {
        let ledger = ResourceLedger::new(budgets(100, 100, 100, Some(150)));
        let _pin = ledger
            .admit(
                owner("pin"),
                ResourceTier::Vram,
                purpose("ws"),
                ResidentClass::Pinned,
                100,
            )
            .unwrap();
        match ledger
            .admit(
                owner("more"),
                ResourceTier::Vram,
                purpose("ws"),
                ResidentClass::Resident,
                1,
            )
            .unwrap_err()
        {
            AdmissionError::FullPin {
                tier: Some(ResourceTier::Vram),
                diagnostic,
            } => {
                assert_eq!(diagnostic.requested_bytes, 1);
                assert_eq!(diagnostic.used_bytes, 100);
                assert_eq!(diagnostic.budget_bytes, 100);
            }
            other => panic!("expected FullPin tier, got {other:?}"),
        }
        assert_eq!(ledger.owner_live_bytes(&owner("more")), 0);

        // shared: VRAM pinned 100 + RAM request 60 > 150, and shared live is all pinned.
        match ledger
            .admit(
                owner("more"),
                ResourceTier::Ram,
                purpose("ws"),
                ResidentClass::Resident,
                60,
            )
            .unwrap_err()
        {
            AdmissionError::FullPin {
                tier: None,
                diagnostic,
            } => {
                assert_eq!(diagnostic.budget_bytes, 150);
                assert_eq!(diagnostic.used_bytes, 100);
            }
            other => panic!("expected FullPin shared, got {other:?}"),
        }

        // unpinned live → 通常のTierCapExceeded(後続evict余地あり)。
        let ledger2 = ResourceLedger::new(budgets(100, 0, 0, None));
        let _res = ledger2
            .admit(
                owner("r"),
                ResourceTier::Vram,
                purpose("cache"),
                ResidentClass::Resident,
                100,
            )
            .unwrap();
        match ledger2
            .admit(
                owner("x"),
                ResourceTier::Vram,
                purpose("cache"),
                ResidentClass::Resident,
                1,
            )
            .unwrap_err()
        {
            AdmissionError::TierCapExceeded { .. } => {}
            other => panic!("expected TierCapExceeded, got {other:?}"),
        }
    }

    #[test]
    fn permit_shrink_returns_unused_immediately() {
        let ledger = ResourceLedger::new(budgets(100, 0, 0, None));
        let mut p = ledger
            .admit(
                owner("s"),
                ResourceTier::Vram,
                purpose("tex"),
                ResidentClass::Resident,
                80,
            )
            .unwrap();
        assert_eq!(ledger.tier_live_bytes(ResourceTier::Vram), 80);
        p.commit_usage(50).unwrap();
        assert_eq!(p.accounted_bytes(), 50);
        assert_eq!(ledger.tier_live_bytes(ResourceTier::Vram), 50);
        assert_eq!(
            p.commit_usage(81).unwrap_err(),
            UsageError::ExceedsReservation {
                used: 81,
                reserved: 80
            }
        );
        assert_eq!(ledger.tier_live_bytes(ResourceTier::Vram), 50);

        let _second = ledger
            .admit(
                owner("t"),
                ResourceTier::Vram,
                purpose("tex"),
                ResidentClass::Resident,
                50,
            )
            .unwrap();
        assert_eq!(ledger.tier_live_bytes(ResourceTier::Vram), 100);
    }

    #[test]
    fn shrink_forbids_reacquisition_above_accounted() {
        let ledger = ResourceLedger::new(budgets(100, 0, 0, None));
        let mut first = ledger
            .admit(
                owner("s"),
                ResourceTier::Vram,
                purpose("tex"),
                ResidentClass::Resident,
                80,
            )
            .unwrap();
        first.commit_usage(50).unwrap();
        assert_eq!(ledger.tier_live_bytes(ResourceTier::Vram), 50);

        let _second = ledger
            .admit(
                owner("t"),
                ResourceTier::Vram,
                purpose("tex"),
                ResidentClass::Resident,
                50,
            )
            .unwrap();
        assert_eq!(ledger.tier_live_bytes(ResourceTier::Vram), 100);

        assert_eq!(
            first.commit_usage(80).unwrap_err(),
            UsageError::ExceedsAccounted {
                used: 80,
                accounted: 50
            }
        );
        assert_eq!(first.accounted_bytes(), 50);
        assert_eq!(ledger.tier_live_bytes(ResourceTier::Vram), 100);

        // 再縮小は許可する。
        first.commit_usage(40).unwrap();
        assert_eq!(first.accounted_bytes(), 40);
        assert_eq!(ledger.tier_live_bytes(ResourceTier::Vram), 90);
        first.commit_usage(40).unwrap();
        assert_eq!(ledger.tier_live_bytes(ResourceTier::Vram), 90);
    }

    #[test]
    fn admission_checked_add_overflow_refuses_without_mutation() {
        let ledger = ResourceLedger::new(budgets(u64::MAX, u64::MAX, 0, Some(u64::MAX)));
        let live = ledger
            .admit(
                owner("max"),
                ResourceTier::Vram,
                purpose("tex"),
                ResidentClass::Resident,
                u64::MAX,
            )
            .unwrap();
        assert_eq!(ledger.tier_live_bytes(ResourceTier::Vram), u64::MAX);

        match ledger
            .admit(
                owner("one"),
                ResourceTier::Vram,
                purpose("tex"),
                ResidentClass::Resident,
                1,
            )
            .unwrap_err()
        {
            AdmissionError::AccountingOverflow { diagnostic } => {
                assert_eq!(diagnostic.owner.as_str(), "one");
                assert_eq!(diagnostic.requested_bytes, 1);
                assert_eq!(diagnostic.used_bytes, u64::MAX);
                assert_eq!(diagnostic.budget_bytes, u64::MAX);
            }
            other => panic!("expected AccountingOverflow, got {other:?}"),
        }
        assert_eq!(ledger.tier_live_bytes(ResourceTier::Vram), u64::MAX);
        assert_eq!(ledger.owner_live_bytes(&owner("one")), 0);

        // shared: VRAM live MAX + RAM request 1 が checked 加算で溢れる。
        match ledger
            .admit(
                owner("ram"),
                ResourceTier::Ram,
                purpose("buf"),
                ResidentClass::Resident,
                1,
            )
            .unwrap_err()
        {
            AdmissionError::AccountingOverflow { diagnostic } => {
                assert_eq!(diagnostic.owner.as_str(), "ram");
                assert_eq!(diagnostic.requested_bytes, 1);
                assert_eq!(diagnostic.used_bytes, u64::MAX);
                assert_eq!(diagnostic.budget_bytes, u64::MAX);
            }
            other => panic!("expected shared AccountingOverflow, got {other:?}"),
        }
        assert_eq!(ledger.tier_live_bytes(ResourceTier::Ram), 0);
        assert_eq!(ledger.owner_live_bytes(&owner("ram")), 0);
        drop(live);
    }

    #[test]
    fn explicit_release_and_drop_return_to_zero() {
        let ledger = ResourceLedger::new(budgets(100, 100, 100, None));
        let p = ledger
            .admit(
                owner("a"),
                ResourceTier::Ram,
                purpose("pcm"),
                ResidentClass::Pinned,
                40,
            )
            .unwrap();
        let q = ledger
            .admit(
                owner("b"),
                ResourceTier::Ram,
                purpose("pcm"),
                ResidentClass::Resident,
                30,
            )
            .unwrap();
        assert_eq!(ledger.tier_live_bytes(ResourceTier::Ram), 70);
        p.release();
        assert_eq!(ledger.tier_live_bytes(ResourceTier::Ram), 30);
        assert_eq!(ledger.owner_live_bytes(&owner("a")), 0);
        drop(q);
        assert_eq!(ledger.tier_live_bytes(ResourceTier::Ram), 0);
        assert_eq!(ledger.total_live_bytes(), 0);
    }

    #[test]
    fn per_owner_totals_and_diagnostic_fields() {
        let ledger = ResourceLedger::new(budgets(100, 100, 100, None));
        let a1 = ledger
            .admit(
                owner("alpha"),
                ResourceTier::Vram,
                purpose("a"),
                ResidentClass::Resident,
                30,
            )
            .unwrap();
        let a2 = ledger
            .admit(
                owner("alpha"),
                ResourceTier::Ram,
                purpose("a"),
                ResidentClass::Resident,
                20,
            )
            .unwrap();
        let _b = ledger
            .admit(
                owner("beta"),
                ResourceTier::Disk,
                purpose("b"),
                ResidentClass::Resident,
                10,
            )
            .unwrap();
        assert_eq!(ledger.owner_live_bytes(&owner("alpha")), 50);
        assert_eq!(ledger.owner_live_bytes(&owner("beta")), 10);

        let err = ledger
            .admit(
                owner("alpha"),
                ResourceTier::Vram,
                purpose("a"),
                ResidentClass::Resident,
                80,
            )
            .unwrap_err();
        let AdmissionError::TierCapExceeded { diagnostic, .. } = err else {
            panic!("expected TierCapExceeded");
        };
        assert_eq!(diagnostic.owner.as_str(), "alpha");
        assert_eq!(diagnostic.requested_bytes, 80);
        assert_eq!(diagnostic.used_bytes, 30);
        assert_eq!(diagnostic.budget_bytes, 100);

        drop(a1);
        drop(a2);
        assert_eq!(ledger.owner_live_bytes(&owner("alpha")), 0);
    }

    #[test]
    fn decision_identical_without_allocator_report_coupling() {
        // allocator report入力路を持たないこと自体が「Noneでも同じ判定」の契約。
        let ledger = ResourceLedger::new(budgets(16, 16, 16, None));
        let p = ledger
            .admit(
                owner("n"),
                ResourceTier::Disk,
                purpose("tmp"),
                ResidentClass::Resident,
                16,
            )
            .unwrap();
        assert!(ledger
            .admit(
                owner("n2"),
                ResourceTier::Disk,
                purpose("tmp"),
                ResidentClass::Resident,
                1,
            )
            .is_err());
        p.release();
        assert_eq!(ledger.total_live_bytes(), 0);
    }
}
