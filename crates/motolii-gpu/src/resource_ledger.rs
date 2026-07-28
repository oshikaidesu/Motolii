use std::collections::{BTreeMap, HashMap};
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum MemoryTier {
    Vram,
    Ram,
    Disk,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ResourceOwner {
    label: &'static str,
}

impl ResourceOwner {
    pub const fn new(label: &'static str) -> Self {
        Self { label }
    }

    pub const fn label(self) -> &'static str {
        self.label
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResourceBudgets {
    pub vram_bytes: u64,
    pub ram_bytes: u64,
    pub disk_bytes: u64,
    pub shared_vram_ram_bytes: Option<u64>,
}

impl ResourceBudgets {
    fn for_tier(self, tier: MemoryTier) -> u64 {
        match tier {
            MemoryTier::Vram => self.vram_bytes,
            MemoryTier::Ram => self.ram_bytes,
            MemoryTier::Disk => self.disk_bytes,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResourceRequest {
    pub owner: ResourceOwner,
    pub tier: MemoryTier,
    pub bytes: u64,
    pub pinned: bool,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct TierUsage {
    pub resident_bytes: u64,
    pub pinned_bytes: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OwnerUsage {
    pub owner: ResourceOwner,
    pub tier: MemoryTier,
    pub usage: TierUsage,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceSnapshot {
    pub vram: TierUsage,
    pub ram: TierUsage,
    pub disk: TierUsage,
    pub owners: Vec<OwnerUsage>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BudgetCap {
    Tier(MemoryTier),
    SharedVramRam,
}

#[derive(Debug, thiserror::Error, Clone, PartialEq, Eq)]
pub enum ResourceAdmissionError {
    #[error("resource owner label must not be empty")]
    EmptyOwner,
    #[error("resource request must be greater than zero bytes")]
    ZeroBytes,
    #[error(
        "resource budget exceeded for {owner} at {tier:?}: requested={requested_bytes}, used={used_bytes}, budget={budget_bytes}, cap={cap:?}"
    )]
    BudgetExceeded {
        owner: &'static str,
        tier: MemoryTier,
        requested_bytes: u64,
        used_bytes: u64,
        budget_bytes: u64,
        cap: BudgetCap,
    },
    #[error("resource accounting overflow for {owner} at {tier:?}")]
    AccountingOverflow {
        owner: &'static str,
        tier: MemoryTier,
    },
    #[error("resource allocation id exhausted")]
    AllocationIdExhausted,
    #[error("resource ledger accounting is corrupted")]
    AccountingCorrupted,
    #[error("resource ledger lock is poisoned")]
    LockPoisoned,
}

#[derive(Debug, Clone)]
pub struct ResourceLedger {
    budgets: ResourceBudgets,
    state: Arc<Mutex<LedgerState>>,
}

#[derive(Debug)]
pub struct ResourceGrant {
    allocation_id: u64,
    state: Arc<Mutex<LedgerState>>,
}

#[derive(Debug, Default)]
struct LedgerState {
    next_allocation_id: u64,
    corrupted: bool,
    usage: TierTotals,
    owners: BTreeMap<(ResourceOwner, MemoryTier), TierUsage>,
    allocations: HashMap<u64, AllocationRecord>,
}

#[derive(Debug, Clone, Copy, Default)]
struct TierTotals {
    vram: TierUsage,
    ram: TierUsage,
    disk: TierUsage,
}

impl TierTotals {
    fn get(self, tier: MemoryTier) -> TierUsage {
        match tier {
            MemoryTier::Vram => self.vram,
            MemoryTier::Ram => self.ram,
            MemoryTier::Disk => self.disk,
        }
    }

    fn get_mut(&mut self, tier: MemoryTier) -> &mut TierUsage {
        match tier {
            MemoryTier::Vram => &mut self.vram,
            MemoryTier::Ram => &mut self.ram,
            MemoryTier::Disk => &mut self.disk,
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct AllocationRecord {
    owner: ResourceOwner,
    tier: MemoryTier,
    bytes: u64,
    pinned: bool,
}

impl ResourceLedger {
    pub fn new(budgets: ResourceBudgets) -> Self {
        Self {
            budgets,
            state: Arc::new(Mutex::new(LedgerState {
                next_allocation_id: 1,
                ..LedgerState::default()
            })),
        }
    }

    pub const fn budgets(&self) -> ResourceBudgets {
        self.budgets
    }

    pub fn admit(&self, request: ResourceRequest) -> Result<ResourceGrant, ResourceAdmissionError> {
        let mut grants = self.admit_batch(&[request])?;
        grants
            .pop()
            .ok_or(ResourceAdmissionError::AccountingCorrupted)
    }

    pub fn admit_batch(
        &self,
        requests: &[ResourceRequest],
    ) -> Result<Vec<ResourceGrant>, ResourceAdmissionError> {
        let mut state = self
            .state
            .lock()
            .map_err(|_| ResourceAdmissionError::LockPoisoned)?;
        if state.corrupted {
            return Err(ResourceAdmissionError::AccountingCorrupted);
        }
        let mut candidate_usage = state.usage;
        let mut candidate_owners = state.owners.clone();
        for &request in requests {
            validate_request(request)?;
            check_budget(self.budgets, candidate_usage, request)?;
            add_usage(&mut candidate_usage, request)?;
            add_owner_usage(&mut candidate_owners, request)?;
        }

        let request_count = u64::try_from(requests.len())
            .map_err(|_| ResourceAdmissionError::AllocationIdExhausted)?;
        let next_allocation_id = state
            .next_allocation_id
            .checked_add(request_count)
            .ok_or(ResourceAdmissionError::AllocationIdExhausted)?;
        let allocation_ids = state.next_allocation_id..next_allocation_id;
        let mut grants = Vec::with_capacity(requests.len());
        for (allocation_id, &request) in allocation_ids.zip(requests) {
            state.allocations.insert(
                allocation_id,
                AllocationRecord {
                    owner: request.owner,
                    tier: request.tier,
                    bytes: request.bytes,
                    pinned: request.pinned,
                },
            );
            grants.push(ResourceGrant {
                allocation_id,
                state: Arc::clone(&self.state),
            });
        }
        state.next_allocation_id = next_allocation_id;
        state.usage = candidate_usage;
        state.owners = candidate_owners;
        Ok(grants)
    }

    pub fn snapshot(&self) -> Result<ResourceSnapshot, ResourceAdmissionError> {
        let state = self
            .state
            .lock()
            .map_err(|_| ResourceAdmissionError::LockPoisoned)?;
        if state.corrupted {
            return Err(ResourceAdmissionError::AccountingCorrupted);
        }
        Ok(ResourceSnapshot {
            vram: state.usage.vram,
            ram: state.usage.ram,
            disk: state.usage.disk,
            owners: state
                .owners
                .iter()
                .map(|(&(owner, tier), &usage)| OwnerUsage { owner, tier, usage })
                .collect(),
        })
    }
}

fn validate_request(request: ResourceRequest) -> Result<(), ResourceAdmissionError> {
    if request.owner.label().is_empty() {
        return Err(ResourceAdmissionError::EmptyOwner);
    }
    if request.bytes == 0 {
        return Err(ResourceAdmissionError::ZeroBytes);
    }
    Ok(())
}

fn check_budget(
    budgets: ResourceBudgets,
    usage: TierTotals,
    request: ResourceRequest,
) -> Result<(), ResourceAdmissionError> {
    let tier_usage = usage.get(request.tier);
    let tier_budget = budgets.for_tier(request.tier);
    let next_tier = tier_usage.resident_bytes.checked_add(request.bytes).ok_or(
        ResourceAdmissionError::AccountingOverflow {
            owner: request.owner.label(),
            tier: request.tier,
        },
    )?;
    if next_tier > tier_budget {
        return Err(ResourceAdmissionError::BudgetExceeded {
            owner: request.owner.label(),
            tier: request.tier,
            requested_bytes: request.bytes,
            used_bytes: tier_usage.resident_bytes,
            budget_bytes: tier_budget,
            cap: BudgetCap::Tier(request.tier),
        });
    }

    if matches!(request.tier, MemoryTier::Vram | MemoryTier::Ram) {
        if let Some(shared_budget) = budgets.shared_vram_ram_bytes {
            let shared_used = usage
                .vram
                .resident_bytes
                .checked_add(usage.ram.resident_bytes)
                .ok_or(ResourceAdmissionError::AccountingOverflow {
                    owner: request.owner.label(),
                    tier: request.tier,
                })?;
            let next_shared = shared_used.checked_add(request.bytes).ok_or(
                ResourceAdmissionError::AccountingOverflow {
                    owner: request.owner.label(),
                    tier: request.tier,
                },
            )?;
            if next_shared > shared_budget {
                return Err(ResourceAdmissionError::BudgetExceeded {
                    owner: request.owner.label(),
                    tier: request.tier,
                    requested_bytes: request.bytes,
                    used_bytes: shared_used,
                    budget_bytes: shared_budget,
                    cap: BudgetCap::SharedVramRam,
                });
            }
        }
    }
    Ok(())
}

fn add_owner_usage(
    owners: &mut BTreeMap<(ResourceOwner, MemoryTier), TierUsage>,
    request: ResourceRequest,
) -> Result<(), ResourceAdmissionError> {
    let owner_usage = owners.entry((request.owner, request.tier)).or_default();
    add_tier_usage(owner_usage, request.bytes, request.pinned).map_err(|_| {
        ResourceAdmissionError::AccountingOverflow {
            owner: request.owner.label(),
            tier: request.tier,
        }
    })
}

impl Drop for ResourceGrant {
    fn drop(&mut self) {
        let mut state = match self.state.lock() {
            Ok(state) => state,
            Err(poisoned) => poisoned.into_inner(),
        };
        let Some(record) = state.allocations.remove(&self.allocation_id) else {
            state.corrupted = true;
            return;
        };
        let totals_valid = subtract_usage(&mut state.usage, record);
        let key = (record.owner, record.tier);
        let owner_valid = if let Some(owner_usage) = state.owners.get_mut(&key) {
            let valid = subtract_tier_usage(owner_usage, record.bytes, record.pinned);
            if valid && *owner_usage == TierUsage::default() {
                state.owners.remove(&key);
            }
            valid
        } else {
            false
        };
        if !totals_valid || !owner_valid {
            state.corrupted = true;
        }
    }
}

fn add_usage(
    totals: &mut TierTotals,
    request: ResourceRequest,
) -> Result<(), ResourceAdmissionError> {
    add_tier_usage(totals.get_mut(request.tier), request.bytes, request.pinned).map_err(|_| {
        ResourceAdmissionError::AccountingOverflow {
            owner: request.owner.label(),
            tier: request.tier,
        }
    })
}

fn add_tier_usage(usage: &mut TierUsage, bytes: u64, pinned: bool) -> Result<(), ()> {
    let resident = usage.resident_bytes.checked_add(bytes).ok_or(())?;
    let pinned_bytes = if pinned {
        usage.pinned_bytes.checked_add(bytes).ok_or(())?
    } else {
        usage.pinned_bytes
    };
    usage.resident_bytes = resident;
    usage.pinned_bytes = pinned_bytes;
    Ok(())
}

fn subtract_usage(totals: &mut TierTotals, record: AllocationRecord) -> bool {
    subtract_tier_usage(totals.get_mut(record.tier), record.bytes, record.pinned)
}

fn subtract_tier_usage(usage: &mut TierUsage, bytes: u64, pinned: bool) -> bool {
    let Some(resident_bytes) = usage.resident_bytes.checked_sub(bytes) else {
        return false;
    };
    let pinned_bytes = if pinned {
        let Some(pinned_bytes) = usage.pinned_bytes.checked_sub(bytes) else {
            return false;
        };
        pinned_bytes
    } else {
        usage.pinned_bytes
    };
    if pinned_bytes > resident_bytes {
        return false;
    }
    usage.resident_bytes = resident_bytes;
    usage.pinned_bytes = pinned_bytes;
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    const RENDER_TARGET: ResourceOwner = ResourceOwner::new("render-target");
    const DECODE_POOL: ResourceOwner = ResourceOwner::new("decode-pool");

    fn budgets(shared: Option<u64>) -> ResourceBudgets {
        ResourceBudgets {
            vram_bytes: 1_024,
            ram_bytes: 2_048,
            disk_bytes: 4_096,
            shared_vram_ram_bytes: shared,
        }
    }

    #[test]
    fn grant_drop_releases_exact_admitted_bytes() {
        let ledger = ResourceLedger::new(budgets(None));
        let grant = ledger
            .admit(ResourceRequest {
                owner: RENDER_TARGET,
                tier: MemoryTier::Vram,
                bytes: 768,
                pinned: true,
            })
            .unwrap();
        assert_eq!(
            ledger.snapshot().unwrap().vram,
            TierUsage {
                resident_bytes: 768,
                pinned_bytes: 768,
            }
        );

        drop(grant);
        assert_eq!(ledger.snapshot().unwrap().vram, TierUsage::default());
        assert!(ledger.snapshot().unwrap().owners.is_empty());
    }

    #[test]
    fn tier_cap_rejects_before_mutating_usage_with_full_context() {
        let ledger = ResourceLedger::new(budgets(None));
        let _grant = ledger
            .admit(ResourceRequest {
                owner: RENDER_TARGET,
                tier: MemoryTier::Vram,
                bytes: 768,
                pinned: false,
            })
            .unwrap();
        assert_eq!(
            ledger
                .admit(ResourceRequest {
                    owner: DECODE_POOL,
                    tier: MemoryTier::Vram,
                    bytes: 512,
                    pinned: false,
                })
                .unwrap_err(),
            ResourceAdmissionError::BudgetExceeded {
                owner: "decode-pool",
                tier: MemoryTier::Vram,
                requested_bytes: 512,
                used_bytes: 768,
                budget_bytes: 1_024,
                cap: BudgetCap::Tier(MemoryTier::Vram),
            }
        );
        assert_eq!(ledger.snapshot().unwrap().vram.resident_bytes, 768);
    }

    #[test]
    fn shared_cap_rejects_even_when_each_tier_fits() {
        let ledger = ResourceLedger::new(budgets(Some(1_200)));
        let _vram = ledger
            .admit(ResourceRequest {
                owner: RENDER_TARGET,
                tier: MemoryTier::Vram,
                bytes: 800,
                pinned: false,
            })
            .unwrap();
        assert_eq!(
            ledger
                .admit(ResourceRequest {
                    owner: DECODE_POOL,
                    tier: MemoryTier::Ram,
                    bytes: 600,
                    pinned: false,
                })
                .unwrap_err(),
            ResourceAdmissionError::BudgetExceeded {
                owner: "decode-pool",
                tier: MemoryTier::Ram,
                requested_bytes: 600,
                used_bytes: 800,
                budget_bytes: 1_200,
                cap: BudgetCap::SharedVramRam,
            }
        );
        assert_eq!(ledger.snapshot().unwrap().ram, TierUsage::default());
    }

    #[test]
    fn owner_snapshot_separates_tier_resident_and_pinned_usage() {
        let ledger = ResourceLedger::new(budgets(None));
        let _vram = ledger
            .admit(ResourceRequest {
                owner: RENDER_TARGET,
                tier: MemoryTier::Vram,
                bytes: 256,
                pinned: false,
            })
            .unwrap();
        let _ram = ledger
            .admit(ResourceRequest {
                owner: RENDER_TARGET,
                tier: MemoryTier::Ram,
                bytes: 512,
                pinned: true,
            })
            .unwrap();

        assert_eq!(
            ledger.snapshot().unwrap().owners,
            vec![
                OwnerUsage {
                    owner: RENDER_TARGET,
                    tier: MemoryTier::Vram,
                    usage: TierUsage {
                        resident_bytes: 256,
                        pinned_bytes: 0,
                    },
                },
                OwnerUsage {
                    owner: RENDER_TARGET,
                    tier: MemoryTier::Ram,
                    usage: TierUsage {
                        resident_bytes: 512,
                        pinned_bytes: 512,
                    },
                },
            ]
        );
    }

    #[test]
    fn invalid_requests_are_typed_and_do_not_mutate_usage() {
        let ledger = ResourceLedger::new(budgets(None));
        assert_eq!(
            ledger
                .admit(ResourceRequest {
                    owner: ResourceOwner::new(""),
                    tier: MemoryTier::Vram,
                    bytes: 1,
                    pinned: false,
                })
                .unwrap_err(),
            ResourceAdmissionError::EmptyOwner
        );
        assert_eq!(
            ledger
                .admit(ResourceRequest {
                    owner: RENDER_TARGET,
                    tier: MemoryTier::Vram,
                    bytes: 0,
                    pinned: false,
                })
                .unwrap_err(),
            ResourceAdmissionError::ZeroBytes
        );
        assert_eq!(ledger.snapshot().unwrap().vram, TierUsage::default());
    }

    #[test]
    fn batch_rejection_keeps_previous_generation_and_adds_nothing() {
        let ledger = ResourceLedger::new(budgets(None));
        let old_generation = ledger
            .admit(ResourceRequest {
                owner: DECODE_POOL,
                tier: MemoryTier::Vram,
                bytes: 400,
                pinned: true,
            })
            .unwrap();

        assert_eq!(
            ledger
                .admit_batch(&[
                    ResourceRequest {
                        owner: DECODE_POOL,
                        tier: MemoryTier::Vram,
                        bytes: 400,
                        pinned: true,
                    },
                    ResourceRequest {
                        owner: DECODE_POOL,
                        tier: MemoryTier::Vram,
                        bytes: 400,
                        pinned: true,
                    },
                ])
                .unwrap_err(),
            ResourceAdmissionError::BudgetExceeded {
                owner: "decode-pool",
                tier: MemoryTier::Vram,
                requested_bytes: 400,
                used_bytes: 800,
                budget_bytes: 1_024,
                cap: BudgetCap::Tier(MemoryTier::Vram),
            }
        );
        assert_eq!(
            ledger.snapshot().unwrap().vram,
            TierUsage {
                resident_bytes: 400,
                pinned_bytes: 400,
            }
        );

        drop(old_generation);
        assert_eq!(ledger.snapshot().unwrap().vram, TierUsage::default());
    }

    #[test]
    fn admitted_batch_releases_each_record_exactly_once() {
        let ledger = ResourceLedger::new(budgets(None));
        let mut grants = ledger
            .admit_batch(&[
                ResourceRequest {
                    owner: RENDER_TARGET,
                    tier: MemoryTier::Vram,
                    bytes: 256,
                    pinned: false,
                },
                ResourceRequest {
                    owner: DECODE_POOL,
                    tier: MemoryTier::Ram,
                    bytes: 512,
                    pinned: true,
                },
            ])
            .unwrap();
        assert_eq!(ledger.snapshot().unwrap().vram.resident_bytes, 256);
        assert_eq!(ledger.snapshot().unwrap().ram.resident_bytes, 512);

        drop(grants.pop());
        assert_eq!(ledger.snapshot().unwrap().ram, TierUsage::default());
        assert_eq!(ledger.snapshot().unwrap().vram.resident_bytes, 256);

        drop(grants);
        assert_eq!(ledger.snapshot().unwrap().vram, TierUsage::default());
    }

    #[test]
    fn release_inconsistency_fails_closed() {
        let ledger = ResourceLedger::new(budgets(None));
        let grant = ledger
            .admit(ResourceRequest {
                owner: RENDER_TARGET,
                tier: MemoryTier::Vram,
                bytes: 256,
                pinned: true,
            })
            .unwrap();
        ledger.state.lock().unwrap().usage.vram = TierUsage::default();

        drop(grant);

        assert_eq!(
            ledger.snapshot().unwrap_err(),
            ResourceAdmissionError::AccountingCorrupted
        );
        assert_eq!(
            ledger
                .admit(ResourceRequest {
                    owner: RENDER_TARGET,
                    tier: MemoryTier::Vram,
                    bytes: 1,
                    pinned: false,
                })
                .unwrap_err(),
            ResourceAdmissionError::AccountingCorrupted
        );
    }
}
