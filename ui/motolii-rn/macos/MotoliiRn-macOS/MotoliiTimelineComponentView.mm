#import "MotoliiTimelineComponentView.h"

#import "MotoliiHostBridge.h"

#import <AppKit/AppKit.h>

#include <cerrno>
#include <cstdlib>

using namespace facebook::react;

namespace {

const size_t kBridgeBufferCapacity = 16 * 1024;

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

BOOL Accepted(int64_t result, const uint8_t *output, size_t capacity)
{
  if (result <= 0 || (uint64_t)result > capacity) {
    return NO;
  }
  NSData *data = [NSData dataWithBytes:output length:(NSUInteger)result];
  NSDictionary *response = [NSJSONSerialization JSONObjectWithData:data options:0 error:nil];
  return [response isKindOfClass:[NSDictionary class]] && [response[@"accepted"] boolValue];
}

} // namespace

@interface MotoliiTimelineComponentView ()
- (void)activateTimelineIfNeeded;
- (void)resizeTimeline;
- (void)deactivateTimeline;
@end

@implementation MotoliiTimelineComponentView

+ (ComponentDescriptorProvider)componentDescriptorProvider
{
  return concreteComponentDescriptorProvider<MotoliiTimelineViewComponentDescriptor>();
}

- (instancetype)initWithFrame:(CGRect)frame
{
  self = [super initWithFrame:frame];
  if (self) {
    self.wantsLayer = YES;
    _metalLayer = (CAMetalLayer *)self.layer;
  }
  return self;
}

- (CALayer *)makeBackingLayer
{
  CAMetalLayer *layer = [CAMetalLayer layer];
  layer.contentsScale = 1.0;
  layer.pixelFormat = MTLPixelFormatBGRA8Unorm;
  return layer;
}

- (BOOL)wantsUpdateLayer
{
  return YES;
}

- (void)dealloc
{
  [self deactivateTimeline];
}

- (void)updateProps:(Props::Shared const &)props
           oldProps:(Props::Shared const &)oldProps
{
  const auto &newProps = *std::static_pointer_cast<const MotoliiTimelineViewProps>(props);
  const auto &previousProps =
      *std::static_pointer_cast<const MotoliiTimelineViewProps>(oldProps);
  bool refreshRequested = newProps.refreshToken != previousProps.refreshToken;
  uint64_t nextHostHandle = ParseHandle(newProps.hostHandle);
  if (nextHostHandle != _hostHandle) {
    [self deactivateTimeline];
    _hostHandle = nextHostHandle;
    [self activateTimelineIfNeeded];
  }
  [super updateProps:props oldProps:oldProps];
  if (refreshRequested) {
    [self setNeedsDisplay:YES];
  }
}

- (void)viewDidMoveToWindow
{
  [super viewDidMoveToWindow];
  if (self.window != nil) {
    [self activateTimelineIfNeeded];
  } else {
    [self deactivateTimeline];
  }
}

- (void)prepareForRecycle
{
  [self deactivateTimeline];
  [super prepareForRecycle];
}

- (void)layout
{
  [super layout];
  [self resizeTimeline];
}

- (void)viewDidChangeBackingProperties
{
  [super viewDidChangeBackingProperties];
  [self resizeTimeline];
}

- (void)updateLayer
{
  if (_hostHandle == 0 || _timelineHandle == 0 || !_surfaceAttached) {
    return;
  }
  uint8_t output[kBridgeBufferCapacity];
  motolii_rn_timeline_draw(
      _hostHandle, _timelineHandle, output, sizeof(output));
}

- (void)activateTimelineIfNeeded
{
  if (_hostHandle == 0 || _timelineHandle != 0 || self.window == nil) {
    return;
  }
  uint8_t output[kBridgeBufferCapacity];
  uint64_t timelineHandle = 0;
  int64_t registered = motolii_rn_timeline_register(
      _hostHandle, &timelineHandle, output, sizeof(output));
  if (!Accepted(registered, output, sizeof(output)) || timelineHandle == 0) {
    return;
  }
  _timelineHandle = timelineHandle;
  int64_t attached = motolii_rn_timeline_attach(
      _hostHandle,
      _timelineHandle,
      (__bridge void *)_metalLayer,
      output,
      sizeof(output));
  if (!Accepted(attached, output, sizeof(output))) {
    motolii_rn_timeline_destroy(_timelineHandle, output, sizeof(output));
    _timelineHandle = 0;
    return;
  }
  _surfaceAttached = YES;
  [self resizeTimeline];
}

- (void)resizeTimeline
{
  if (self.window == nil) {
    return;
  }
  CGFloat scale = self.window.backingScaleFactor > 0 ? self.window.backingScaleFactor : 1.0;
  _metalLayer.contentsScale = scale;
  CGSize size = CGSizeMake(
      MAX(0.0, NSWidth(self.bounds) * scale),
      MAX(0.0, NSHeight(self.bounds) * scale));
  _metalLayer.drawableSize = size;
  if (_hostHandle == 0 || _timelineHandle == 0 || !_surfaceAttached) {
    [self activateTimelineIfNeeded];
    return;
  }
  uint8_t output[kBridgeBufferCapacity];
  int64_t resized = motolii_rn_timeline_resize_physical(
      _hostHandle,
      _timelineHandle,
      (uint32_t)size.width,
      (uint32_t)size.height,
      output,
      sizeof(output));
  if (Accepted(resized, output, sizeof(output))) {
    [self setNeedsDisplay:YES];
  }
}

- (void)deactivateTimeline
{
  if (_timelineHandle == 0) {
    _surfaceAttached = NO;
    return;
  }
  uint8_t output[kBridgeBufferCapacity];
  if (_surfaceAttached && _hostHandle != 0) {
    motolii_rn_timeline_detach(
        _hostHandle, _timelineHandle, output, sizeof(output));
  }
  motolii_rn_timeline_destroy(_timelineHandle, output, sizeof(output));
  _timelineHandle = 0;
  _surfaceAttached = NO;
}

@end
