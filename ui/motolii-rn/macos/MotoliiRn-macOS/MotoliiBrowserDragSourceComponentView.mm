#import "MotoliiBrowserDragSourceComponentView.h"

#import "MotoliiHostBridge.h"

#import <AppKit/AppKit.h>
#import <React/renderer/components/MotoliiRnSpec/ComponentDescriptors.h>

#include <cerrno>
#include <cstdlib>

using namespace facebook::react;

namespace {

const uint8_t kCancelEscape = 1;
const uint8_t kCancelCaptureLost = 2;

uint64_t ParseHandle(const std::string &value)
{
  if (value.empty()) {
    return 0;
  }
  errno = 0;
  char *end = nullptr;
  unsigned long long parsed = std::strtoull(value.c_str(), &end, 10);
  if (errno != 0 || end == value.c_str() || *end != '\0' || parsed == 0) {
    return 0;
  }
  return static_cast<uint64_t>(parsed);
}

NSPoint TopDownLogicalWindowPoint(NSView *view, NSEvent *event)
{
  NSView *content = view.window.contentView;
  if (content == nil) {
    return NSMakePoint(NAN, NAN);
  }
  NSPoint point = [content convertPoint:event.locationInWindow fromView:nil];
  if (!content.isFlipped) {
    point.y = NSHeight(content.bounds) - point.y;
  }
  return point;
}

} // namespace

@implementation MotoliiBrowserDragSourceComponentView

+ (ComponentDescriptorProvider)componentDescriptorProvider
{
  return concreteComponentDescriptorProvider<MotoliiBrowserDragSourceViewComponentDescriptor>();
}

- (void)dealloc
{
  [self cancelCapture:kCancelCaptureLost];
  [self stopObservingWindow];
}

- (void)updateProps:(Props::Shared const &)props
           oldProps:(Props::Shared const &)oldProps
{
  const auto &newProps =
      *std::static_pointer_cast<const MotoliiBrowserDragSourceViewProps>(props);
  uint64_t nextHostHandle = ParseHandle(newProps.hostHandle);
  NSString *nextScopeRef = [NSString stringWithUTF8String:newProps.scopeRef.c_str()];
  NSString *nextItemId = [NSString stringWithUTF8String:newProps.itemId.c_str()];
  if (nextHostHandle != _hostHandle || ![_scopeRef isEqualToString:nextScopeRef] ||
      ![_itemId isEqualToString:nextItemId]) {
    [self cancelCapture:kCancelCaptureLost];
    _hostHandle = nextHostHandle;
    _scopeRef = [nextScopeRef copy];
    _itemId = [nextItemId copy];
  }
  [super updateProps:props oldProps:oldProps];
}

- (void)viewDidMoveToWindow
{
  [self cancelCapture:kCancelCaptureLost];
  [self stopObservingWindow];
  [super viewDidMoveToWindow];
  if (self.window != nil) {
    __weak MotoliiBrowserDragSourceComponentView *weakSelf = self;
    _windowResignObserver = [[NSNotificationCenter defaultCenter]
        addObserverForName:NSWindowDidResignKeyNotification
                    object:self.window
                     queue:nil
                usingBlock:^(__unused NSNotification *notification) {
                  [weakSelf cancelCapture:kCancelCaptureLost];
                }];
  }
}

- (void)prepareForRecycle
{
  [self cancelCapture:kCancelCaptureLost];
  [self stopObservingWindow];
  _hostHandle = 0;
  _scopeRef = nil;
  _itemId = nil;
  [super prepareForRecycle];
}

- (BOOL)acceptsFirstResponder
{
  return YES;
}

- (BOOL)resignFirstResponder
{
  [self cancelCapture:kCancelCaptureLost];
  return [super resignFirstResponder];
}

- (NSView *)hitTest:(NSPoint)point
{
  return NSPointInRect(point, self.bounds) ? self : nil;
}

- (void)mouseDown:(NSEvent *)event
{
  if (_captureGeneration != 0 || event.buttonNumber != 0 || _hostHandle == 0 ||
      _scopeRef.length == 0 || _itemId.length == 0) {
    return;
  }
  [self.window makeFirstResponder:self];
  NSPoint point = TopDownLogicalWindowPoint(self, event);
  NSData *scopeData = [_scopeRef dataUsingEncoding:NSUTF8StringEncoding];
  NSData *itemData = [_itemId dataUsingEncoding:NSUTF8StringEncoding];
  _captureGeneration = motolii_rn_browser_capture_begin(
      _hostHandle,
      static_cast<const uint8_t *>(scopeData.bytes),
      scopeData.length,
      static_cast<const uint8_t *>(itemData.bytes),
      itemData.length,
      point.x,
      point.y);
}

- (void)mouseDragged:(NSEvent *)event
{
  if (_captureGeneration == 0) {
    return;
  }
  NSPoint point = TopDownLogicalWindowPoint(self, event);
  if (!motolii_rn_browser_capture_move(
          _hostHandle, _captureGeneration, point.x, point.y)) {
    uint64_t generation = _captureGeneration;
    _captureGeneration = 0;
    motolii_rn_browser_capture_cancel(_hostHandle, generation, kCancelCaptureLost);
  }
}

- (void)mouseUp:(NSEvent *)event
{
  if (_captureGeneration == 0 || event.buttonNumber != 0) {
    return;
  }
  uint64_t generation = _captureGeneration;
  _captureGeneration = 0;
  NSPoint point = TopDownLogicalWindowPoint(self, event);
  if (!motolii_rn_browser_capture_release(
          _hostHandle, generation, point.x, point.y)) {
    motolii_rn_browser_capture_cancel(_hostHandle, generation, kCancelCaptureLost);
  }
}

- (void)keyDown:(NSEvent *)event
{
  if (event.keyCode == 53) {
    [self cancelCapture:kCancelEscape];
    return;
  }
  [super keyDown:event];
}

- (void)cancelCapture:(uint8_t)reason
{
  if (_captureGeneration == 0) {
    return;
  }
  uint64_t generation = _captureGeneration;
  _captureGeneration = 0;
  motolii_rn_browser_capture_cancel(_hostHandle, generation, reason);
}

- (void)stopObservingWindow
{
  if (_windowResignObserver != nil) {
    [[NSNotificationCenter defaultCenter] removeObserver:_windowResignObserver];
    _windowResignObserver = nil;
  }
}

@end
