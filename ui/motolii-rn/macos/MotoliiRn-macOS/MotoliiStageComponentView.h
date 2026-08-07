#import <React/RCTViewComponentView.h>
#import <React/renderer/components/MotoliiRnSpec/ComponentDescriptors.h>
#import <React/renderer/components/MotoliiRnSpec/Props.h>
#import <React/renderer/components/MotoliiRnSpec/RCTComponentViewHelpers.h>

#include <stdint.h>

@interface MotoliiStageComponentView : RCTViewComponentView <RCTMotoliiStageViewViewProtocol> {
 @private
  uint64_t _hostHandle;
  uint64_t _stageHandle;
  uint64_t _revision;
  BOOL _mounted;
  BOOL _focused;
  NSString *_snapshotText;
}
@end
