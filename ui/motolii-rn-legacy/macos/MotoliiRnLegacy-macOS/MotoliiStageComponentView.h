#import <React/RCTViewComponentView.h>
#import <React/renderer/components/MotoliiRnLegacySpec/ComponentDescriptors.h>
#import <React/renderer/components/MotoliiRnLegacySpec/Props.h>
#import <React/renderer/components/MotoliiRnLegacySpec/RCTComponentViewHelpers.h>

#import <QuartzCore/CAMetalLayer.h>

#include <stdint.h>

@interface MotoliiStageComponentView : RCTViewComponentView <RCTMotoliiStageViewViewProtocol> {
 @private
  uint64_t _hostHandle;
  uint64_t _stageHandle;
  uint64_t _revision;
  uint64_t _pointerSequence;
  BOOL _mounted;
  BOOL _focused;
  BOOL _surfaceAttached;
  BOOL _surfaceRecoveryPending;
  NSString *_snapshotText;
  CAMetalLayer *_metalLayer;
}
@end
