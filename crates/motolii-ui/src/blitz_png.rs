//! 8bit RGBA PNG を書くだけ。**encoder依存を足さないために手で書く。**
//!
//! zlibは **stored**(非圧縮)ブロックだけを使う。仕様上これも正当なdeflateストリームで、
//! 絵を見るための道具に圧縮率は要らない。
//!
//! 元は `blitz_dump/gpu.rs`(さらに元は `spikes/blitz-probe/src/bin/alpha_composite.rs`)に
//! 閉じていたが、`blitz_shell` からも同じものが要るのでlib側へ出した。**3つ目の写しを作らない。**

use std::io::Write;
use std::path::Path;

/// `rgba` は行頭パディング無しの `w * h * 4` バイト。
pub fn write_png(path: &Path, rgba: &[u8], w: u32, h: u32) {
    let mut raw = Vec::with_capacity(((w * 4 + 1) * h) as usize);
    for y in 0..h {
        raw.push(0u8); // filter: None
        let s = (y * w * 4) as usize;
        raw.extend_from_slice(&rgba[s..s + (w * 4) as usize]);
    }

    let mut z = vec![0x78u8, 0x01]; // zlib header, no compression
    let mut i = 0usize;
    while i < raw.len() {
        let n = (raw.len() - i).min(65535);
        let last = if i + n == raw.len() { 1u8 } else { 0u8 };
        z.push(last);
        z.extend_from_slice(&(n as u16).to_le_bytes());
        z.extend_from_slice(&(!(n as u16)).to_le_bytes());
        z.extend_from_slice(&raw[i..i + n]);
        i += n;
    }
    z.extend_from_slice(&adler32(&raw).to_be_bytes());

    let mut png = vec![0x89, b'P', b'N', b'G', 0x0d, 0x0a, 0x1a, 0x0a];
    let mut ihdr = Vec::new();
    ihdr.extend_from_slice(&w.to_be_bytes());
    ihdr.extend_from_slice(&h.to_be_bytes());
    ihdr.extend_from_slice(&[8, 6, 0, 0, 0]); // 8bit RGBA
    chunk(&mut png, b"IHDR", &ihdr);
    chunk(&mut png, b"IDAT", &z);
    chunk(&mut png, b"IEND", &[]);

    let mut file = std::fs::File::create(path).expect("png create");
    file.write_all(&png).expect("png write");
}

fn chunk(out: &mut Vec<u8>, kind: &[u8; 4], data: &[u8]) {
    out.extend_from_slice(&(data.len() as u32).to_be_bytes());
    let start = out.len();
    out.extend_from_slice(kind);
    out.extend_from_slice(data);
    let crc = crc32(&out[start..]);
    out.extend_from_slice(&crc.to_be_bytes());
}

fn crc32(data: &[u8]) -> u32 {
    let mut crc = 0xffff_ffffu32;
    for &b in data {
        crc ^= b as u32;
        for _ in 0..8 {
            crc = if crc & 1 != 0 {
                (crc >> 1) ^ 0xedb8_8320
            } else {
                crc >> 1
            };
        }
    }
    !crc
}

fn adler32(data: &[u8]) -> u32 {
    let (mut a, mut b) = (1u32, 0u32);
    for &x in data {
        a = (a + x as u32) % 65521;
        b = (b + a) % 65521;
    }
    (b << 16) | a
}
