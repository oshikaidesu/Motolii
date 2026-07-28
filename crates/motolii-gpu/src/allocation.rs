#[derive(Debug, thiserror::Error, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AllocationEstimateError {
    #[error("allocation alignment must be greater than zero")]
    ZeroAlignment,
    #[error("resource size estimate overflowed u64")]
    Overflow,
}

pub(crate) fn estimate_buffer_bytes(
    size: u64,
    alignment: u64,
) -> Result<u64, AllocationEstimateError> {
    align_up(size, alignment)
}

pub(crate) fn estimate_copy_buffer_bytes(
    width: u32,
    height: u32,
    bytes_per_texel: u32,
    row_alignment: u32,
) -> Result<(u32, u32, u64), AllocationEstimateError> {
    let unpadded_row = width
        .checked_mul(bytes_per_texel)
        .ok_or(AllocationEstimateError::Overflow)?;
    let padded_row_u64 = estimate_buffer_bytes(u64::from(unpadded_row), u64::from(row_alignment))?;
    let padded_row =
        u32::try_from(padded_row_u64).map_err(|_| AllocationEstimateError::Overflow)?;
    let total = padded_row_u64
        .checked_mul(u64::from(height))
        .ok_or(AllocationEstimateError::Overflow)?;
    Ok((unpadded_row, padded_row, total))
}

fn align_up(value: u64, alignment: u64) -> Result<u64, AllocationEstimateError> {
    if alignment == 0 {
        return Err(AllocationEstimateError::ZeroAlignment);
    }
    let remainder = value % alignment;
    if remainder == 0 {
        return Ok(value);
    }
    value
        .checked_add(alignment - remainder)
        .ok_or(AllocationEstimateError::Overflow)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    struct TextureAllocationAlignment {
        row_bytes: u64,
        allocation_bytes: u64,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum TextureEstimateError {
        Allocation(AllocationEstimateError),
        ZeroTextureExtent,
        UnsupportedTextureFormat,
    }

    fn estimate_texture_bytes(
        desc: &wgpu::TextureDescriptor<'_>,
        alignment: TextureAllocationAlignment,
    ) -> Result<u64, TextureEstimateError> {
        if desc.size.width == 0
            || desc.size.height == 0
            || desc.size.depth_or_array_layers == 0
            || desc.mip_level_count == 0
            || desc.sample_count == 0
        {
            return Err(TextureEstimateError::ZeroTextureExtent);
        }

        let block_bytes = u64::from(
            desc.format
                .block_copy_size(None)
                .ok_or(TextureEstimateError::UnsupportedTextureFormat)?,
        );
        let (block_width, block_height) = desc.format.block_dimensions();
        let mut total = 0_u64;

        for mip in 0..desc.mip_level_count {
            let width = mip_extent(desc.size.width, mip);
            let height = mip_extent(desc.size.height, mip);
            let depth_or_layers = match desc.dimension {
                wgpu::TextureDimension::D3 => mip_extent(desc.size.depth_or_array_layers, mip),
                wgpu::TextureDimension::D1 | wgpu::TextureDimension::D2 => {
                    desc.size.depth_or_array_layers
                }
            };
            let blocks_per_row = u64::from(width.div_ceil(block_width));
            let block_rows = u64::from(height.div_ceil(block_height));
            let row_bytes =
                blocks_per_row
                    .checked_mul(block_bytes)
                    .ok_or(TextureEstimateError::Allocation(
                        AllocationEstimateError::Overflow,
                    ))?;
            let padded_row_bytes = align_up(row_bytes, alignment.row_bytes)
                .map_err(TextureEstimateError::Allocation)?;
            let mip_bytes = padded_row_bytes
                .checked_mul(block_rows)
                .and_then(|bytes| bytes.checked_mul(u64::from(depth_or_layers)))
                .and_then(|bytes| bytes.checked_mul(u64::from(desc.sample_count)))
                .ok_or(TextureEstimateError::Allocation(
                    AllocationEstimateError::Overflow,
                ))?;
            total = total
                .checked_add(mip_bytes)
                .ok_or(TextureEstimateError::Allocation(
                    AllocationEstimateError::Overflow,
                ))?;
        }

        align_up(total, alignment.allocation_bytes).map_err(TextureEstimateError::Allocation)
    }

    fn mip_extent(base: u32, mip: u32) -> u32 {
        base.checked_shr(mip).unwrap_or(0).max(1)
    }

    fn texture_desc(
        width: u32,
        height: u32,
        depth_or_array_layers: u32,
        mip_level_count: u32,
        sample_count: u32,
        dimension: wgpu::TextureDimension,
        format: wgpu::TextureFormat,
    ) -> wgpu::TextureDescriptor<'static> {
        wgpu::TextureDescriptor {
            label: None,
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers,
            },
            mip_level_count,
            sample_count,
            dimension,
            format,
            usage: wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        }
    }

    fn tight() -> TextureAllocationAlignment {
        TextureAllocationAlignment {
            row_bytes: 1,
            allocation_bytes: 1,
        }
    }

    #[test]
    fn texture_estimate_counts_format_mips_and_samples() {
        let rgba8 = texture_desc(
            8,
            8,
            1,
            4,
            1,
            wgpu::TextureDimension::D2,
            wgpu::TextureFormat::Rgba8Unorm,
        );
        let rgba16 = texture_desc(
            8,
            8,
            1,
            1,
            1,
            wgpu::TextureDimension::D2,
            wgpu::TextureFormat::Rgba16Float,
        );
        let msaa = texture_desc(
            8,
            8,
            1,
            1,
            4,
            wgpu::TextureDimension::D2,
            wgpu::TextureFormat::Rgba8Unorm,
        );

        assert_eq!(estimate_texture_bytes(&rgba8, tight()), Ok(340));
        assert_eq!(estimate_texture_bytes(&rgba16, tight()), Ok(512));
        assert_eq!(estimate_texture_bytes(&msaa, tight()), Ok(1_024));
    }

    #[test]
    fn texture_estimate_counts_array_layers_and_3d_mips_differently() {
        let array = texture_desc(
            4,
            4,
            4,
            3,
            1,
            wgpu::TextureDimension::D2,
            wgpu::TextureFormat::Rgba8Unorm,
        );
        let volume = texture_desc(
            4,
            4,
            4,
            3,
            1,
            wgpu::TextureDimension::D3,
            wgpu::TextureFormat::Rgba8Unorm,
        );

        assert_eq!(estimate_texture_bytes(&array, tight()), Ok(336));
        assert_eq!(estimate_texture_bytes(&volume, tight()), Ok(292));
    }

    #[test]
    fn texture_estimate_counts_compressed_blocks_and_alignment() {
        let compressed = texture_desc(
            8,
            8,
            1,
            1,
            1,
            wgpu::TextureDimension::D2,
            wgpu::TextureFormat::Bc1RgbaUnorm,
        );
        let aligned = TextureAllocationAlignment {
            row_bytes: 256,
            allocation_bytes: 1_024,
        };

        assert_eq!(estimate_texture_bytes(&compressed, tight()), Ok(32));
        assert_eq!(estimate_texture_bytes(&compressed, aligned), Ok(1_024));
    }

    #[test]
    fn estimates_reject_unknown_footprints_zero_alignment_and_overflow() {
        let depth24 = texture_desc(
            8,
            8,
            1,
            1,
            1,
            wgpu::TextureDimension::D2,
            wgpu::TextureFormat::Depth24Plus,
        );
        assert!(matches!(
            estimate_texture_bytes(&depth24, tight()),
            Err(TextureEstimateError::UnsupportedTextureFormat)
        ));
        assert_eq!(
            estimate_buffer_bytes(1, 0),
            Err(AllocationEstimateError::ZeroAlignment)
        );
        assert_eq!(
            estimate_copy_buffer_bytes(u32::MAX, 2, 4, 256),
            Err(AllocationEstimateError::Overflow)
        );
    }

    #[test]
    fn copy_buffer_estimate_returns_checked_row_and_total_sizes() {
        assert_eq!(
            estimate_copy_buffer_bytes(1_920, 1_080, 4, 256),
            Ok((7_680, 7_680, 8_294_400))
        );
        assert_eq!(
            estimate_copy_buffer_bytes(1_921, 2, 4, 256),
            Ok((7_684, 7_936, 15_872))
        );
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum Tier {
        Vram,
        Ram,
        Disk,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    struct Budgets {
        vram: u64,
        ram: u64,
        disk: u64,
        shared: Option<u64>,
    }

    #[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
    struct Usage {
        vram: u64,
        ram: u64,
        disk: u64,
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    struct AdmissionError {
        owner: &'static str,
        tier: Tier,
        requested: u64,
        used: u64,
        budget: u64,
    }

    #[derive(Debug)]
    struct ContractLedger {
        budgets: Budgets,
        usage: Usage,
    }

    impl ContractLedger {
        fn admit(
            &mut self,
            owner: &'static str,
            tier: Tier,
            requested: u64,
        ) -> Result<(), AdmissionError> {
            let (used, budget) = match tier {
                Tier::Vram => (self.usage.vram, self.budgets.vram),
                Tier::Ram => (self.usage.ram, self.budgets.ram),
                Tier::Disk => (self.usage.disk, self.budgets.disk),
            };
            if used.checked_add(requested).is_none_or(|next| next > budget) {
                return Err(AdmissionError {
                    owner,
                    tier,
                    requested,
                    used,
                    budget,
                });
            }
            if matches!(tier, Tier::Vram | Tier::Ram) {
                if let Some(shared_budget) = self.budgets.shared {
                    let shared_used = self.usage.vram + self.usage.ram;
                    if shared_used
                        .checked_add(requested)
                        .is_none_or(|next| next > shared_budget)
                    {
                        return Err(AdmissionError {
                            owner,
                            tier,
                            requested,
                            used: shared_used,
                            budget: shared_budget,
                        });
                    }
                }
            }
            match tier {
                Tier::Vram => self.usage.vram += requested,
                Tier::Ram => self.usage.ram += requested,
                Tier::Disk => self.usage.disk += requested,
            }
            Ok(())
        }

        fn release(&mut self, tier: Tier, bytes: u64) {
            match tier {
                Tier::Vram => self.usage.vram -= bytes,
                Tier::Ram => self.usage.ram -= bytes,
                Tier::Disk => self.usage.disk -= bytes,
            }
        }
    }

    #[test]
    fn hard_caps_reject_before_usage_exceeds_budget_and_report_context() {
        let mut ledger = ContractLedger {
            budgets: Budgets {
                vram: 1_024,
                ram: 2_048,
                disk: 4_096,
                shared: None,
            },
            usage: Usage::default(),
        };

        assert_eq!(ledger.admit("render-target", Tier::Vram, 768), Ok(()));
        assert_eq!(
            ledger.admit("decoder-surface", Tier::Vram, 512),
            Err(AdmissionError {
                owner: "decoder-surface",
                tier: Tier::Vram,
                requested: 512,
                used: 768,
                budget: 1_024,
            })
        );
        assert_eq!(ledger.usage.vram, 768);
        ledger.release(Tier::Vram, 768);
        assert_eq!(ledger.usage, Usage::default());
    }

    #[test]
    fn shared_memory_cap_rejects_even_when_separate_caps_fit() {
        let mut ledger = ContractLedger {
            budgets: Budgets {
                vram: 1_024,
                ram: 1_024,
                disk: 4_096,
                shared: Some(1_200),
            },
            usage: Usage::default(),
        };

        ledger.admit("hot-cache", Tier::Vram, 800).unwrap();
        assert_eq!(
            ledger.admit("warm-cache", Tier::Ram, 600),
            Err(AdmissionError {
                owner: "warm-cache",
                tier: Tier::Ram,
                requested: 600,
                used: 800,
                budget: 1_200,
            })
        );
        assert_eq!(ledger.usage.ram, 0);
        assert_eq!(ledger.admit("disk-cache", Tier::Disk, 600), Ok(()));
    }
}
