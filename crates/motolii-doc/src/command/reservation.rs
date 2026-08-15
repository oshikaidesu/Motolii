use crate::stable_id::StableIdReservation;
use crate::validate::stable_id_in_use;
use crate::Document;

use super::CommandError;

pub(super) struct ReservationCommit {
    advance_to: Option<u64>,
}

pub(super) fn swap_if_valid(doc: &mut Document, next: Document) -> Result<(), CommandError> {
    next.validate().map_err(CommandError::Validate)?;
    *doc = next;
    Ok(())
}

fn validate_reservation_shape(
    reservation: StableIdReservation,
    introduced: &[u64],
) -> Result<(), CommandError> {
    let before = reservation.before();
    let after = reservation.after();
    if before >= after {
        return Err(CommandError::InvalidStableIdReservationInterval { before, after });
    }
    let span = after
        .checked_sub(before)
        .ok_or(CommandError::InvalidStableIdReservationInterval { before, after })?;
    let introduced_len =
        u64::try_from(introduced.len()).map_err(|_| CommandError::StableIdReservationMismatch {
            before,
            after,
            introduced: introduced.to_vec(),
        })?;
    if introduced_len != span {
        return Err(CommandError::StableIdReservationMismatch {
            before,
            after,
            introduced: introduced.to_vec(),
        });
    }
    for (offset, &id) in introduced.iter().enumerate() {
        let offset =
            u64::try_from(offset).map_err(|_| CommandError::StableIdReservationMismatch {
                before,
                after,
                introduced: introduced.to_vec(),
            })?;
        let expected =
            before
                .checked_add(offset)
                .ok_or(CommandError::StableIdReservationMismatch {
                    before,
                    after,
                    introduced: introduced.to_vec(),
                })?;
        if id != expected {
            return Err(CommandError::StableIdReservationMismatch {
                before,
                after,
                introduced: introduced.to_vec(),
            });
        }
    }
    Ok(())
}

pub(super) fn validate_reservation_for_apply(
    doc: &Document,
    reservation: StableIdReservation,
    introduced: &[u64],
) -> Result<ReservationCommit, CommandError> {
    validate_reservation_shape(reservation, introduced)?;
    let before = reservation.before();
    let after = reservation.after();
    for &id in introduced {
        if stable_id_in_use(doc, id) {
            return Err(CommandError::StableIdCollision { id });
        }
    }
    let next = doc.next_stable_id.peek_next();
    let advance_to = if next == before {
        Some(after)
    } else if next >= after {
        None
    } else {
        return Err(CommandError::StableIdReservationCounterMismatch {
            next,
            before,
            after,
        });
    };
    Ok(ReservationCommit { advance_to })
}

pub(super) fn validate_reservation_for_undo(
    doc: &Document,
    reservation: StableIdReservation,
    introduced: &[u64],
) -> Result<(), CommandError> {
    validate_reservation_shape(reservation, introduced)?;
    let before = reservation.before();
    let after = reservation.after();
    let next = doc.next_stable_id.peek_next();
    if next < after {
        return Err(CommandError::StableIdReservationCounterMismatch {
            next,
            before,
            after,
        });
    }
    Ok(())
}

pub(crate) fn validate_reservation_closure(
    reservation: StableIdReservation,
    introduced: &[u64],
) -> Result<(), CommandError> {
    validate_reservation_shape(reservation, introduced)
}

pub(super) fn apply_reservation_commit(doc: &mut Document, commit: ReservationCommit) {
    if let Some(after) = commit.advance_to {
        doc.next_stable_id.commit_validated_reservation(after);
    }
}
