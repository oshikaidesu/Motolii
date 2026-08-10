#import <React/RCTViewComponentView.h>
#import <React/renderer/components/MotoliiRnSpec/ComponentDescriptors.h>
#import <React/renderer/components/MotoliiRnSpec/Props.h>
#import <React/renderer/components/MotoliiRnSpec/RCTComponentViewHelpers.h>

#include <stdint.h>

@interface MotoliiBrowserDragSourceComponentView
    : RCTViewComponentView <RCTMotoliiBrowserDragSourceViewViewProtocol> {
 @private
  uint64_t _hostHandle;
  uint64_t _captureGeneration;
  NSString *_scopeRef;
  NSString *_itemId;
  id _windowResignObserver;
}
@end
