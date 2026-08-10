//! M4-K1c着手前の階層転送・single-flight負例。

use std::collections::HashMap;

use motolii_gpu::{
    AdmissionError, AdmissionPermit, BudgetDiagnostic, ResidentClass, ResourceBudgets,
    ResourceLedger, ResourceOwner, ResourcePurpose, ResourceTier,
};

fn cache_entry() -> ResourceOwner {
    ResourceOwner::new("cache-entry")
}

fn prefetch() -> ResourceOwner {
    ResourceOwner::new("prefetch")
}

fn cache_purpose() -> ResourcePurpose {
    ResourcePurpose::new("cache-entry")
}

fn prefetch_purpose() -> ResourcePurpose {
    ResourcePurpose::new("prefetch")
}

#[derive(Debug)]
struct Resident {
    tier: ResourceTier,
    bytes: u64,
    _grant: AdmissionPermit,
}

#[derive(Debug)]
struct Transfer {
    source: Resident,
    destination: Resident,
}

fn resident(
    ledger: &ResourceLedger,
    tier: ResourceTier,
    bytes: u64,
    pinned: bool,
) -> Result<Resident, AdmissionError> {
    let class = if pinned {
        ResidentClass::Pinned
    } else {
        ResidentClass::Resident
    };
    Ok(Resident {
        tier,
        bytes,
        _grant: ledger.admit(cache_entry(), tier, cache_purpose(), class, bytes)?,
    })
}

fn begin_transfer(
    ledger: &ResourceLedger,
    source: Resident,
    destination_tier: ResourceTier,
) -> Result<Transfer, (Resident, AdmissionError)> {
    match resident(ledger, destination_tier, source.bytes, false) {
        Ok(destination) => Ok(Transfer {
            source,
            destination,
        }),
        Err(error) => Err((source, error)),
    }
}

impl Transfer {
    fn commit(self) -> Resident {
        let Transfer {
            source,
            destination,
        } = self;
        drop(source);
        destination
    }

    fn abort(self) -> Resident {
        let Transfer {
            source,
            destination,
        } = self;
        drop(destination);
        source
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Candidate {
    id: u64,
    last_used: u64,
    pinned: bool,
    in_use: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct AllCandidatesPinned;

fn lru_evictable(candidates: &[Candidate]) -> Result<u64, AllCandidatesPinned> {
    candidates
        .iter()
        .filter(|candidate| !candidate.pinned && !candidate.in_use)
        .min_by_key(|candidate| candidate.last_used)
        .map(|candidate| candidate.id)
        .ok_or(AllCandidatesPinned)
}

struct InFlight {
    id: u64,
    grant: AdmissionPermit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Completion {
    Current,
    StaleDiscarded,
}

struct SingleFlight {
    generation: u64,
    next_id: u64,
    jobs: HashMap<(&'static str, u64), InFlight>,
}

impl SingleFlight {
    fn new() -> Self {
        Self {
            generation: 1,
            next_id: 1,
            jobs: HashMap::new(),
        }
    }

    fn request(
        &mut self,
        ledger: &ResourceLedger,
        key: &'static str,
        bytes: u64,
    ) -> Result<(u64, bool), AdmissionError> {
        let job_key = (key, self.generation);
        if let Some(job) = self.jobs.get(&job_key) {
            return Ok((job.id, false));
        }
        let grant = ledger.admit(
            prefetch(),
            ResourceTier::Ram,
            prefetch_purpose(),
            ResidentClass::Pinned,
            bytes,
        )?;
        let id = self.next_id;
        self.next_id += 1;
        self.jobs.insert(job_key, InFlight { id, grant });
        Ok((id, true))
    }

    fn advance_generation(&mut self) {
        self.generation += 1;
    }

    fn complete(&mut self, key: &'static str, generation: u64) -> Option<Completion> {
        let job = self.jobs.remove(&(key, generation))?;
        let _grant_lived_until_completion = job.grant;
        if generation == self.generation {
            Some(Completion::Current)
        } else {
            Some(Completion::StaleDiscarded)
        }
    }

    fn cancel(&mut self, key: &'static str, generation: u64) -> bool {
        self.jobs.remove(&(key, generation)).is_some()
    }
}

fn budgets(vram: u64, ram: u64, shared: Option<u64>) -> ResourceBudgets {
    ResourceBudgets {
        vram_bytes: vram,
        ram_bytes: ram,
        disk_bytes: 10_000,
        shared_memory_bytes: shared,
    }
}

#[test]
fn destination_refusal_keeps_source_resident_and_accounted() {
    let ledger = ResourceLedger::new(budgets(100, 60, None));
    let source = resident(&ledger, ResourceTier::Vram, 80, true).unwrap();

    let (source, error) = match begin_transfer(&ledger, source, ResourceTier::Ram) {
        Ok(_) => panic!("destination must not fit"),
        Err(result) => result,
    };

    assert_eq!(
        error,
        AdmissionError::TierCapExceeded {
            tier: ResourceTier::Ram,
            diagnostic: BudgetDiagnostic {
                owner: cache_entry(),
                requested_bytes: 80,
                used_bytes: 0,
                budget_bytes: 60,
            },
        }
    );
    assert_eq!(source.tier, ResourceTier::Vram);
    assert_eq!(ledger.tier_live_bytes(ResourceTier::Vram), 80);
    assert_eq!(ledger.tier_live_bytes(ResourceTier::Ram), 0);
    drop(source);
    assert_eq!(ledger.tier_live_bytes(ResourceTier::Vram), 0);
}

#[test]
fn transfer_keeps_both_tiers_until_commit_and_abort_keeps_source() {
    let ledger = ResourceLedger::new(budgets(100, 100, None));
    let source = resident(&ledger, ResourceTier::Vram, 80, true).unwrap();
    let transfer = begin_transfer(&ledger, source, ResourceTier::Ram).unwrap();
    assert_eq!(ledger.tier_live_bytes(ResourceTier::Vram), 80);
    assert_eq!(ledger.tier_live_bytes(ResourceTier::Ram), 80);

    let source = transfer.abort();
    assert_eq!(source.tier, ResourceTier::Vram);
    assert_eq!(ledger.tier_live_bytes(ResourceTier::Vram), 80);
    assert_eq!(ledger.tier_live_bytes(ResourceTier::Ram), 0);

    let transfer = begin_transfer(&ledger, source, ResourceTier::Ram).unwrap();
    let destination = transfer.commit();
    assert_eq!(destination.tier, ResourceTier::Ram);
    assert_eq!(ledger.tier_live_bytes(ResourceTier::Vram), 0);
    assert_eq!(ledger.tier_live_bytes(ResourceTier::Ram), 80);
    drop(destination);
    assert_eq!(ledger.tier_live_bytes(ResourceTier::Ram), 0);
}

#[test]
fn uma_shared_cap_counts_transfer_double_residency() {
    let ledger = ResourceLedger::new(budgets(100, 100, Some(120)));
    let source = resident(&ledger, ResourceTier::Vram, 80, true).unwrap();
    let (source, error) = match begin_transfer(&ledger, source, ResourceTier::Ram) {
        Ok(_) => panic!("shared cap must include source and destination"),
        Err(result) => result,
    };
    assert_eq!(
        error,
        AdmissionError::FullPin {
            tier: None,
            diagnostic: BudgetDiagnostic {
                owner: cache_entry(),
                requested_bytes: 80,
                used_bytes: 80,
                budget_bytes: 120,
            },
        }
    );
    assert_eq!(ledger.tier_live_bytes(ResourceTier::Vram), 80);
    drop(source);
}

#[test]
fn lru_skips_in_use_and_pinned_and_rejects_when_none_are_evictable() {
    let candidates = [
        Candidate {
            id: 1,
            last_used: 1,
            pinned: true,
            in_use: false,
        },
        Candidate {
            id: 2,
            last_used: 2,
            pinned: false,
            in_use: true,
        },
        Candidate {
            id: 3,
            last_used: 3,
            pinned: false,
            in_use: false,
        },
    ];
    assert_eq!(lru_evictable(&candidates), Ok(3));
    assert_eq!(lru_evictable(&candidates[..2]), Err(AllCandidatesPinned));
}

#[test]
fn duplicate_demand_is_single_flight_and_stale_completion_releases_reservation() {
    let ledger = ResourceLedger::new(budgets(100, 100, None));
    let mut flights = SingleFlight::new();
    let (first, created) = flights.request(&ledger, "frame-42", 60).unwrap();
    let (duplicate, duplicate_created) = flights.request(&ledger, "frame-42", 60).unwrap();
    assert_eq!(first, duplicate);
    assert!(created);
    assert!(!duplicate_created);
    assert_eq!(ledger.tier_live_bytes(ResourceTier::Ram), 60);

    flights.advance_generation();
    assert_eq!(
        flights.complete("frame-42", 1),
        Some(Completion::StaleDiscarded)
    );
    assert_eq!(ledger.tier_live_bytes(ResourceTier::Ram), 0);
}

#[test]
fn cancellation_releases_inflight_reservation_without_result() {
    let ledger = ResourceLedger::new(budgets(100, 100, None));
    let mut flights = SingleFlight::new();
    flights.request(&ledger, "frame-7", 60).unwrap();
    assert_eq!(ledger.tier_live_bytes(ResourceTier::Ram), 60);

    assert!(flights.cancel("frame-7", 1));
    assert_eq!(ledger.tier_live_bytes(ResourceTier::Ram), 0);
    assert!(!flights.cancel("frame-7", 1));
}
