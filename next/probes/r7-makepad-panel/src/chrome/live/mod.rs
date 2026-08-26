//! Ableton Live chrome。見た目だけ。Document / DAW 意味は持たない。
//!
//! 正本: 利用者添付の Live 12 実画面
//!   `assets/image-bd719b8e-8793-4d66-a7e9-1080b06b7deb.png`
//! 名称併記: https://www.ableton.com/en/live-manual/12/
//! 色は画像から画素。Theme .ask / 記憶のネオンは使わない。
//!
//! ホスト向け公開型（この1枚に見える粒）:
//! `LiveFace` `LiveFaceBar` `LiveFacePanel` `LiveFaceDevice` `LiveFaceRecess`
//! `LiveFaceArea` `LiveFaceHighlight` `LiveInk` `LiveInkPanel` `LiveInkDisabled`
//! `LiveLink` `LiveTap` `LiveTempo` `LiveTimeSignature` `LiveMetronome`
//! `LivePlay` `LiveStop` `LiveRecord` `LiveArrangementPosition` `LiveQuantization`
//! `LiveMidi` `LiveCpu` `LiveTransport`
//! `LiveBrowserSearch` `LiveBrowserLabel` `LiveBrowserLabelOn`
//! `LiveFilterTag` `LiveFilterTagOn` `LiveBrowserItem` `LiveBrowserItemOn`
//! `LiveBrowserCollection`
//! `LiveArrangementRuler` `LiveArrangementClip` `LivePlayhead` `LiveArrangementLane`
//! `LiveMeter` `LiveVolume` `LivePan` `LiveTrackActivator` `LiveSolo`
//! `LiveTrackTitleBar` `LiveTrackMixer`
//! `LiveDeviceActivator` `LiveFold` `LiveDeviceTitleBar` `LiveDeviceKnob`
//! `LiveDeviceParam` `LiveSliderVertical` `LiveSegmentButton` `LiveSegmentButtonOn`
//! `LiveVisualizerWave` `LiveVisualizerEq` `LiveDeviceChain`
//!
//! Session グリッドはこの1枚に無い。旧型は色だけ画面パレットへ寄せて残す:
//! `LiveClipLaunch` `LiveClipStop` `LiveClipSlotEmpty` `LiveClipSlot`
//! `LiveClipSlotPlaying` `LiveGroupSlot` `LiveSceneLaunch` `LiveClipTitleBar` `LiveArm`
//!
//! 結線はホスト。`set_type_default()` は使わない。`ScrollYView` は書かない。
use makepad_widgets::*;

pub mod arrangement;
pub mod browser;
pub mod device;
pub mod mixer;
pub mod session;
pub mod theme;
pub mod transport;

pub fn script_mod(vm: &mut ScriptVm) {
    theme::script_mod(vm);
    session::script_mod(vm);
    mixer::script_mod(vm);
    device::script_mod(vm);
    browser::script_mod(vm);
    transport::script_mod(vm);
    arrangement::script_mod(vm);
}
