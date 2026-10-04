//! Datamosh is the usual H.264 edit: keep the first IDR, drop later ones, repeat P slices.
//! `h264-reader` finds the NALs. The bytes go back to VideoToolbox. No pixel readback.

use std::io::Read;
use std::path::Path;
use std::process::Command;

use h264_reader::annexb::AnnexBReader;
use h264_reader::nal::{Nal, RefNal, UnitType};
use h264_reader::push::NalInterest;

pub(super) struct Mosh {
    pub path: String,
    pub kept_idr: u32,
    pub dropped_idr: u32,
    pub repeated_p: u32,
}

pub(super) fn material(src: &str) -> Result<Mosh, String> {
    let annex = "/tmp/motolii-mosh-src.h264";
    let edited = "/tmp/motolii-mosh.h264";
    let out = "/tmp/motolii-mosh.mp4";
    ffmpeg(&[
        "-y", "-loglevel", "error", "-i", src, "-an", "-c:v", "libx264", "-bf", "0", "-g", "12",
        "-keyint_min", "12", "-sc_threshold", "0", "-pix_fmt", "yuv420p", "-f", "h264", annex,
    ])?;
    let bytes = std::fs::read(annex).map_err(|error| error.to_string())?;
    let (body, kept_idr, dropped_idr, repeated_p) = edit(&bytes)?;
    std::fs::write(edited, body).map_err(|error| error.to_string())?;
    ffmpeg(&["-y", "-loglevel", "error", "-fflags", "+genpts", "-i", edited, "-c", "copy", out])?;
    if !Path::new(out).exists() {
        return Err("moshed mp4 missing".into());
    }
    Ok(Mosh { path: out.into(), kept_idr, dropped_idr, repeated_p })
}

fn edit(annexb: &[u8]) -> Result<(Vec<u8>, u32, u32, u32), String> {
    let mut nals = Vec::new();
    let mut reader = AnnexBReader::accumulate(|nal: RefNal<'_>| {
        if !nal.is_complete() {
            return NalInterest::Buffer;
        }
        let kind = nal.header().ok().map(|header| header.nal_unit_type());
        let mut body = Vec::new();
        if nal.reader().read_to_end(&mut body).is_ok() && !body.is_empty() {
            nals.push((kind, body));
        }
        NalInterest::Ignore
    });
    reader.push(annexb);
    reader.push(&[0, 0, 0, 1]);
    let mut out = Vec::with_capacity(annexb.len() * 2);
    let mut kept_idr = 0u32;
    let mut dropped_idr = 0u32;
    let mut repeated_p = 0u32;
    for (kind, body) in nals {
        match kind {
            Some(UnitType::SliceLayerWithoutPartitioningIdr) => {
                if kept_idr == 0 {
                    kept_idr = 1;
                    push(&mut out, &body);
                } else {
                    dropped_idr += 1;
                }
            }
            Some(UnitType::SliceLayerWithoutPartitioningNonIdr) => {
                push(&mut out, &body);
                push(&mut out, &body);
                repeated_p += 1;
            }
            Some(_) => push(&mut out, &body),
            None => {}
        }
    }
    if kept_idr == 0 {
        return Err("datamosh found no IDR".into());
    }
    Ok((out, kept_idr, dropped_idr, repeated_p))
}

fn push(out: &mut Vec<u8>, nal: &[u8]) {
    out.extend_from_slice(&[0, 0, 0, 1]);
    out.extend_from_slice(nal);
}

fn ffmpeg(args: &[&str]) -> Result<(), String> {
    let done = Command::new("ffmpeg").args(args).status().map_err(|error| error.to_string())?;
    if done.success() {
        Ok(())
    } else {
        Err(format!("ffmpeg {args:?} -> {done}"))
    }
}
