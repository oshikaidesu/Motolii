use rand_pcg::{rand_core::Rng, Pcg32};
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub struct InstanceId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SlotInput {
    pub source_id: u64,
    pub parent_id: u64,
    pub depth: u16,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct InstanceSample {
    pub source_id: u64,
    pub instance_id: InstanceId,
    pub channels: [f32; 3],
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum EvaluatorError {
    #[error("source id must be non-zero")]
    InvalidSource,
    #[error("parent id must be non-zero for nested input")]
    InvalidParent,
}

pub fn evaluate(
    user_seed: u64,
    inputs: &[SlotInput],
) -> Result<Vec<InstanceSample>, EvaluatorError> {
    let mut samples = Vec::with_capacity(inputs.len());
    for input in inputs {
        if input.source_id == 0 {
            return Err(EvaluatorError::InvalidSource);
        }
        if input.depth > 0 && input.parent_id == 0 {
            return Err(EvaluatorError::InvalidParent);
        }
        let slot_key = slot_key(user_seed, *input);
        let instance_id = InstanceId(slot_key);
        let channels = [
            channel_value(slot_key, 0),
            channel_value(slot_key, 1),
            channel_value(slot_key, 2),
        ];
        samples.push(InstanceSample {
            source_id: input.source_id,
            instance_id,
            channels,
        });
    }
    Ok(samples)
}

pub fn slot_key(user_seed: u64, input: SlotInput) -> u64 {
    let mut value = user_seed ^ input.source_id.rotate_left(17);
    value = mix(value ^ input.parent_id.rotate_left(31));
    mix(value ^ u64::from(input.depth).wrapping_mul(0xD6E8_FEB8_6659_FD93))
}

fn channel_value(slot_key: u64, channel: u64) -> f32 {
    let mut rng = Pcg32::new(slot_key, channel.wrapping_add(1));
    (rng.next_u32() as f32) / (u32::MAX as f32)
}

fn mix(mut value: u64) -> u64 {
    value = (value ^ (value >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    value = (value ^ (value >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    value ^ (value >> 31)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    fn by_source(samples: &[InstanceSample]) -> BTreeMap<u64, InstanceSample> {
        samples
            .iter()
            .copied()
            .map(|sample| (sample.source_id, sample))
            .collect()
    }

    #[test]
    fn count_growth_and_reorder_preserve_instance_identity_and_channels() {
        let base = [
            SlotInput {
                source_id: 11,
                parent_id: 1,
                depth: 0,
            },
            SlotInput {
                source_id: 22,
                parent_id: 1,
                depth: 0,
            },
        ];
        let grown = [
            SlotInput {
                source_id: 99,
                parent_id: 1,
                depth: 0,
            },
            base[1],
            base[0],
        ];
        let first = by_source(&evaluate(77, &base).unwrap());
        let second = by_source(&evaluate(77, &grown).unwrap());
        assert_eq!(first[&11], second[&11]);
        assert_eq!(first[&22], second[&22]);
    }

    #[test]
    fn nested_parent_context_changes_identity_without_index_dependency() {
        let a = SlotInput {
            source_id: 11,
            parent_id: 1,
            depth: 1,
        };
        let b = SlotInput {
            source_id: 11,
            parent_id: 2,
            depth: 1,
        };
        let first = evaluate(77, &[a]).unwrap()[0];
        let second = evaluate(77, &[b]).unwrap()[0];
        assert_ne!(first.instance_id, second.instance_id);
        assert_ne!(first.channels, second.channels);
    }

    #[test]
    fn thread_order_is_irrelevant_and_invalid_input_is_typed() {
        let inputs = vec![
            SlotInput {
                source_id: 11,
                parent_id: 1,
                depth: 0,
            },
            SlotInput {
                source_id: 22,
                parent_id: 1,
                depth: 0,
            },
            SlotInput {
                source_id: 33,
                parent_id: 1,
                depth: 0,
            },
        ];
        let expected = by_source(&evaluate(77, &inputs).unwrap());
        let actual = std::thread::scope(|scope| {
            scope
                .spawn(|| by_source(&evaluate(77, &inputs).unwrap()))
                .join()
                .unwrap()
        });
        assert_eq!(expected, actual);
        assert_eq!(
            evaluate(
                77,
                &[SlotInput {
                    source_id: 0,
                    parent_id: 1,
                    depth: 0
                }]
            ),
            Err(EvaluatorError::InvalidSource)
        );
        assert_eq!(
            evaluate(
                77,
                &[SlotInput {
                    source_id: 1,
                    parent_id: 0,
                    depth: 1
                }]
            ),
            Err(EvaluatorError::InvalidParent)
        );
    }

    #[test]
    fn stable_golden_vector_is_pcg32_and_mixer_specific() {
        let sample = evaluate(
            77,
            &[SlotInput {
                source_id: 11,
                parent_id: 1,
                depth: 0,
            }],
        )
        .unwrap()[0];
        assert_eq!(sample.instance_id, InstanceId(237_390_148_889_641_753));
        assert_eq!(sample.channels, [0.942_798_3, 0.320_141_14, 0.077_010_944]);
    }
}
