use std::sync::mpsc::channel;

use block2::RcBlock;
use objc2::ClassType;
use objc2_av_foundation::{
    AVAssetReader, AVAssetReaderTrackOutput, AVAsynchronousKeyValueLoading, AVMediaTypeVideo,
    AVURLAsset,
};
use objc2_core_foundation::CFRetained;
use objc2_core_media::CMSampleBuffer;
use objc2_core_video::{
    kCVPixelBufferIOSurfacePropertiesKey, kCVPixelBufferMetalCompatibilityKey,
    kCVPixelBufferPixelFormatTypeKey, kCVPixelFormatType_420YpCbCr8BiPlanarVideoRange,
    CVPixelBuffer, CVPixelBufferGetHeight, CVPixelBufferGetPixelFormatType, CVPixelBufferGetWidth,
};
use objc2_foundation::{ns_string, NSArray, NSDictionary, NSNumber, NSString, NSURL};

pub(super) struct VideoToolbox {
    _asset: objc2::rc::Retained<AVURLAsset>,
    _reader: objc2::rc::Retained<AVAssetReader>,
    output: objc2::rc::Retained<AVAssetReaderTrackOutput>,
    pub width: usize,
    pub height: usize,
    pub fps: f32,
}

pub(super) struct Sample {
    pub(super) buffer: objc2::rc::Retained<CMSampleBuffer>,
    pub(super) pixel: CFRetained<CVPixelBuffer>,
}

impl VideoToolbox {
    pub(super) fn open(path: &NSString) -> Result<Self, String> {
        let url = NSURL::fileURLWithPath(path);
        let asset = unsafe { AVURLAsset::URLAssetWithURL_options(&url, None) };
        let (sender, receiver) = channel();
        let block = RcBlock::new(move || {
            let _ = sender.send(());
        });
        unsafe {
            asset.loadValuesAsynchronouslyForKeys_completionHandler(
                &NSArray::from_slice(&[ns_string!("tracks")]),
                Some(&block),
            );
        }
        receiver
            .recv()
            .map_err(|_| "asset load closed".to_string())?;

        let media = unsafe { AVMediaTypeVideo }.expect("AVMediaTypeVideo");
        #[allow(deprecated)]
        let tracks = unsafe { asset.tracksWithMediaType(media) };
        let track = tracks
            .firstObject()
            .ok_or_else(|| format!("no video track in {}", path))?;
        let fps = unsafe { track.nominalFrameRate() }.max(1.0);

        let format = NSNumber::numberWithUnsignedInt(
            kCVPixelFormatType_420YpCbCr8BiPlanarVideoRange,
        );
        let metal = NSNumber::numberWithBool(true);
        let iosurface = NSDictionary::<NSString, objc2::runtime::AnyObject>::new();
        let format_key: &NSString = unsafe { &*kCVPixelBufferPixelFormatTypeKey }.as_ref();
        let metal_key: &NSString = unsafe { &*kCVPixelBufferMetalCompatibilityKey }.as_ref();
        let io_key: &NSString = unsafe { &*kCVPixelBufferIOSurfacePropertiesKey }.as_ref();
        let settings = NSDictionary::from_slices(
            &[format_key, metal_key, io_key],
            &[
                format.as_ref(),
                metal.as_ref(),
                iosurface.as_ref(),
            ],
        );

        let reader = unsafe { AVAssetReader::assetReaderWithAsset_error(&asset) }
            .map_err(|error| error.localizedDescription().to_string())?;
        let output = unsafe {
            AVAssetReaderTrackOutput::assetReaderTrackOutputWithTrack_outputSettings(
                &track,
                Some(&settings),
            )
        };
        unsafe { output.setAlwaysCopiesSampleData(false) };
        unsafe { reader.addOutput(&output) };
        if !unsafe { reader.startReading() } {
            let why = unsafe { reader.error() }
                .map(|error| error.localizedDescription().to_string())
                .unwrap_or_else(|| "startReading failed".to_string());
            return Err(why);
        }

        Ok(Self {
            _asset: asset,
            _reader: reader,
            output,
            width: 0,
            height: 0,
            fps,
        })
    }

    pub(super) fn next_frame(&mut self) -> Result<Option<Sample>, String> {
        let Some(buffer) = (unsafe { self.output.as_super().copyNextSampleBuffer() }) else {
            return Ok(None);
        };
        let pixel = unsafe { buffer.image_buffer() }.ok_or("sample has no image buffer")?;
        let format = CVPixelBufferGetPixelFormatType(&pixel);
        if format != kCVPixelFormatType_420YpCbCr8BiPlanarVideoRange {
            return Err(format!("pixel format {format:#x} is not 420v"));
        }
        self.width = CVPixelBufferGetWidth(&pixel);
        self.height = CVPixelBufferGetHeight(&pixel);
        Ok(Some(Sample { buffer, pixel }))
    }

    pub(super) fn drain(&mut self) -> Result<Vec<Sample>, String> {
        let mut frames = Vec::new();
        while let Some(sample) = self.next_frame()? {
            frames.push(sample);
            if frames.len() == 240 {
                eprintln!("A1: video frame cache stopped at 240");
                break;
            }
        }
        if frames.is_empty() {
            return Err("video has no frame".to_string());
        }
        Ok(frames)
    }
}
