//! `OutputStream::open_negotiated_shared` が Transport とカウンタを共有することを検証。

use std::sync::Arc;

use cpal::traits::HostTrait;
use motolii_audio::{
    channel, negotiate_output, DeviceWaitLatency, OutputStream, PcmCache, PcmFormat,
    PlaybackCounters,
};
use motolii_core::{Fps, Quality};
use motolii_transport::Transport;

fn tiny_cache() -> PcmCache {
    PcmCache::from_interleaved(
        vec![0.1, -0.1],
        PcmFormat {
            channels: 1,
            sample_rate: 48_000,
        },
    )
    .unwrap()
}

#[test]
fn output_stream_shares_counters_and_device_wait_with_transport() {
    let host = cpal::default_host();
    let Some(device) = host.default_output_device() else {
        return;
    };

    let _cache = tiny_cache();
    let format = _cache.format();
    let counters = Arc::new(PlaybackCounters::default());
    let device_wait = Arc::new(DeviceWaitLatency::default());
    let (prod, cons) = channel(1, 256).unwrap();

    let Ok(negotiated) = negotiate_output(&device, format) else {
        return;
    };
    let Ok(output) = OutputStream::open_negotiated_shared(
        &device,
        &negotiated,
        cons,
        Arc::clone(&counters),
        Some(Arc::clone(&device_wait)),
    ) else {
        return;
    };

    assert!(Arc::ptr_eq(&counters, &output.counters()));

    let transport = Transport::new(
        Arc::clone(&counters),
        Arc::clone(&device_wait),
        Fps::try_new(30, 1).unwrap(),
        negotiated.device_sample_rate,
        Quality::DRAFT,
        false,
    )
    .unwrap();

    while prod.push_frames(&[0.5]) == 0 {
        std::hint::spin_loop();
    }
    std::thread::sleep(std::time::Duration::from_millis(50));

    assert!(
        transport.supplied_frames() > 0 || counters.frames_supplied() > 0,
        "shared counters must advance via output callback"
    );

    device_wait.set_wait_frames(96);
    assert_eq!(transport.device_wait().wait_frames(), 96);
}
