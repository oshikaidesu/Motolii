#import <React/RCTViewComponentView.h>
#import <React/renderer/components/MotoliiRnSpec/ComponentDescriptors.h>
#import <React/renderer/components/MotoliiRnSpec/Props.h>
#import <React/renderer/components/MotoliiRnSpec/RCTComponentViewHelpers.h>

#import <QuartzCore/CAMetalLayer.h>

@interface MotoliiTimelineComponentView
    : RCTViewComponentView <RCTMotoliiTimelineViewViewProtocol> {
 @private
  uint64_t _hostHandle;
  uint64_t _timelineHandle;
  BOOL _surfaceAttached;
  CAMetalLayer *_metalLayer;
}
@end
