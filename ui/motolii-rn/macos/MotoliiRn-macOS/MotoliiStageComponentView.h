#import <React/RCTViewComponentView.h>
#import <React/renderer/components/MotoliiRnSpec/ComponentDescriptors.h>
#import <React/renderer/components/MotoliiRnSpec/Props.h>
#import <React/renderer/components/MotoliiRnSpec/RCTComponentViewHelpers.h>

#import <QuartzCore/CAMetalLayer.h>

#include <stdint.h>

@interface MotoliiStageComponentView : RCTViewComponentView <RCTMotoliiStageViewViewProtocol> {
 @private
  uint64_t _hostHandle;
  uint64_t _stageHandle;
  uint64_t _revision;
  BOOL _mounted;
  BOOL _focused;
  BOOL _surfaceAttached;
  BOOL _surfaceRecoveryPending;
  NSString *_snapshotText;
  CAMetalLayer *_metalLayer;
}
@end
