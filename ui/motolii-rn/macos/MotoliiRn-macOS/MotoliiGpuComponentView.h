#import <React/RCTViewComponentView.h>

@interface MotoliiGpuComponentView : RCTViewComponentView
@end

@interface MotoliiTimelineComponentView : RCTViewComponentView
@end

/// 実機windowのkeyDownを既存 host_key_event へ載せる。TextInput/IME は通す。
void MotoliiInstallProductKeymapMonitor(void);
