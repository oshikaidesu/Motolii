#import "MotoliiStageComponentView.h"

#import "MotoliiHostBridge.h"

#import <AppKit/AppKit.h>
#import <QuartzCore/CAMetalLayer.h>
#import <React/renderer/components/MotoliiRnSpec/ComponentDescriptors.h>

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

NSString *HandleString(uint64_t handle)
{
  return [NSString stringWithFormat:@"%llu", (unsigned long long)handle];
}

NSData *IntentData(
    NSString *kind,
    uint64_t hostHandle,
    uint64_t stageHandle,
    CGFloat width,
    CGFloat height,
    CGFloat scaleFactor,
    BOOL focused)
{
  NSMutableDictionary<NSString *, id> *payload = [@{
    @"version" : @1,
    @"direction" : @"rn-to-host",
    @"kind" : kind,
    @"host_handle" : HandleString(hostHandle),
    @"stage_handle" : HandleString(stageHandle),
  } mutableCopy];

  if ([kind isEqualToString:@"stage_resize"]) {
    payload[@"width"] = @((NSUInteger)MAX(0, ceil(width)));
    payload[@"height"] = @((NSUInteger)MAX(0, ceil(height)));
    payload[@"scale_factor"] = @(scaleFactor > 0 ? scaleFactor : 1.0);
  } else if ([kind isEqualToString:@"stage_focus"]) {
    payload[@"focused"] = @(focused);
  }

  return [NSJSONSerialization dataWithJSONObject:payload options:0 error:nil];
}

NSString *SnapshotTextFromJSON(NSData *data)
{
  if (data == nil) {
    return @"snapshot unavailable";
  }
  NSDictionary *response =
      [NSJSONSerialization JSONObjectWithData:data options:0 error:nil];
  if (![response isKindOfClass:[NSDictionary class]]) {
    return @"snapshot response invalid";
  }

  NSDictionary *snapshot = response[@"snapshot"];
  if (![snapshot isKindOfClass:[NSDictionary class]] &&
      [response[@"revision"] isKindOfClass:[NSString class]] &&
      [response[@"projection_generation"] isKindOfClass:[NSString class]]) {
    // read_snapshot returns WireProductSnapshot directly, while lifecycle
    // intents return the same snapshot inside WireIntentResponse.
    snapshot = response;
  }
  if (![snapshot isKindOfClass:[NSDictionary class]]) {
    NSArray *diagnostics = response[@"diagnostics"];
    NSDictionary *diagnostic = [diagnostics isKindOfClass:[NSArray class]] ? diagnostics.firstObject : nil;
    NSString *reason = [diagnostic isKindOfClass:[NSDictionary class]] ? diagnostic[@"reason"] : nil;
    return reason.length > 0 ? [NSString stringWithFormat:@"rejected: %@", reason]
                             : @"snapshot rejected";
  }

  NSString *revision = snapshot[@"revision"] ?: @"?";
  NSString *generation = snapshot[@"projection_generation"] ?: @"?";
  NSString *layer = snapshot[@"primary_layer_id"] ?: @"none";
  return [NSString stringWithFormat:@"revision %@\ngeneration %@\nprimary layer %@",
                                    revision,
                                    generation,
                                    layer];
}

BOOL BridgeAccepted(NSData *data)
{
  if (data == nil) {
    return NO;
  }
  NSDictionary *response =
      [NSJSONSerialization JSONObjectWithData:data options:0 error:nil];
  if (![response isKindOfClass:[NSDictionary class]]) {
    return NO;
  }
  return [response[@"accepted"] boolValue];
}

} // namespace

@implementation MotoliiStageComponentView

+ (ComponentDescriptorProvider)componentDescriptorProvider
{
  return concreteComponentDescriptorProvider<MotoliiStageViewComponentDescriptor>();
}

- (instancetype)initWithFrame:(CGRect)frame
{
  self = [super initWithFrame:frame];
  if (self) {
    _snapshotText = @"stage not mounted";
    _surfaceAttached = NO;
    _surfaceRecoveryPending = NO;
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
  [self deactivateStage];
}

- (void)updateProps:(Props::Shared const &)props
           oldProps:(Props::Shared const &)oldProps
{
  const auto &newProps = *std::static_pointer_cast<const MotoliiStageViewProps>(props);
  uint64_t nextHostHandle = ParseHandle(newProps.hostHandle);
  if (nextHostHandle != _hostHandle) {
    [self deactivateStage];
    _hostHandle = nextHostHandle;
    [self activateStageIfNeeded];
  }
  [super updateProps:props oldProps:oldProps];
}

- (void)viewDidMoveToWindow
{
  [super viewDidMoveToWindow];
  if (self.window != nil) {
    [self activateStageIfNeeded];
  } else {
    [self deactivateStage];
  }
}

- (void)prepareForRecycle
{
  [self deactivateStage];
  [super prepareForRecycle];
}

- (void)layout
{
  [super layout];
  if (self.window != nil) {
    _metalLayer.contentsScale = self.window.backingScaleFactor;
  }
  [self sendResizeIntent];
  [self sendPhysicalResize];
}

- (void)viewDidChangeBackingProperties
{
  [super viewDidChangeBackingProperties];
  if (self.window != nil) {
    _metalLayer.contentsScale = self.window.backingScaleFactor;
  }
  [self sendResizeIntent];
  [self sendPhysicalResize];
}

- (BOOL)acceptsFirstResponder
{
  return YES;
}

- (BOOL)becomeFirstResponder
{
  BOOL becameFirstResponder = [super becomeFirstResponder];
  if (becameFirstResponder) {
    _focused = YES;
    [self sendFocusIntent];
  }
  return becameFirstResponder;
}

- (BOOL)resignFirstResponder
{
  BOOL resigned = [super resignFirstResponder];
  if (resigned) {
    _focused = NO;
    [self sendFocusIntent];
  }
  return resigned;
}

- (void)mouseDown:(NSEvent *)event
{
  [self.window makeFirstResponder:self];
  [super mouseDown:event];
}

- (void)updateLayer
{
  if (![NSThread isMainThread]) {
    _snapshotText = @"gpu draw rejected off main thread";
    return;
  }

  if (_surfaceRecoveryPending) {
    // 一回のdisplay passにつきfresh attachは一度だけ試し、失敗時は再予約しない。
    _surfaceRecoveryPending = NO;
    [self attachSurfaceIfNeeded];
    if (!_surfaceAttached) {
      return;
    }
    [self sendPhysicalResize];
  }

  if (_stageHandle == 0 || !_surfaceAttached) {
    return;
  }

  uint8_t output[kBridgeBufferCapacity];
  int64_t drawResult = motolii_rn_stage_draw(
      _hostHandle, _stageHandle, output, sizeof(output));
  if (drawResult <= 0 || (uint64_t)drawResult > sizeof(output) ||
      !BridgeAccepted([NSData dataWithBytes:output length:(NSUInteger)drawResult])) {
    _snapshotText = @"gpu draw rejected";
    [self scheduleSurfaceRecovery];
  }
}

- (void)scheduleSurfaceRecovery
{
  if (_surfaceRecoveryPending) {
    return;
  }
  [self detachSurfaceIfNeeded];
  _surfaceRecoveryPending = YES;
  [self setNeedsDisplay:YES];
}

- (void)activateStageIfNeeded
{
  if (_hostHandle == 0 || _stageHandle != 0 || self.window == nil) {
    return;
  }

  uint64_t stageHandle = 0;
  uint8_t output[kBridgeBufferCapacity];
  int64_t result = motolii_rn_stage_register(
      _hostHandle, &stageHandle, output, sizeof(output));
  if (result <= 0 || (uint64_t)result > sizeof(output) || stageHandle == 0) {
    if (result > 0 && (uint64_t)result <= sizeof(output)) {
      NSData *response = [NSData dataWithBytes:output length:(NSUInteger)result];
      _snapshotText = SnapshotTextFromJSON(response);
    } else {
      _snapshotText = @"stage registration rejected";
    }
    [self setNeedsDisplay:YES];
    return;
  }

  NSData *response = [NSData dataWithBytes:output length:(NSUInteger)result];
  NSDictionary *payload =
      [NSJSONSerialization JSONObjectWithData:response options:0 error:nil];
  if (![payload isKindOfClass:[NSDictionary class]] || ![payload[@"accepted"] boolValue]) {
    _snapshotText = SnapshotTextFromJSON(response);
    [self setNeedsDisplay:YES];
    return;
  }

  _stageHandle = stageHandle;
  _mounted = YES;
  [self sendIntent:@"stage_mount"];
  [self attachSurfaceIfNeeded];
  [self sendResizeIntent];
  [self sendPhysicalResize];
  [self sendFocusIntent];
  [self readSnapshot];
  [self setNeedsDisplay:YES];
}

- (void)attachSurfaceIfNeeded
{
  if (_stageHandle == 0 || _surfaceAttached || _metalLayer == nil) {
    return;
  }
  if (![NSThread isMainThread]) {
    _snapshotText = @"gpu attach rejected off main thread";
    return;
  }
  uint8_t output[kBridgeBufferCapacity];
  int64_t attachResult = motolii_rn_stage_attach(
      _hostHandle, _stageHandle, (__bridge void *)_metalLayer, output, sizeof(output));
  if (attachResult <= 0 || (uint64_t)attachResult > sizeof(output)) {
    _snapshotText = @"gpu attach rejected";
    return;
  }
  NSData *response = [NSData dataWithBytes:output length:(NSUInteger)attachResult];
  if (!BridgeAccepted(response)) {
    _snapshotText = SnapshotTextFromJSON(response);
    return;
  }
  _surfaceAttached = YES;
}

- (void)detachSurfaceIfNeeded
{
  if (_stageHandle == 0 || !_surfaceAttached) {
    return;
  }
  if (![NSThread isMainThread]) {
    _snapshotText = @"gpu detach rejected off main thread";
    return;
  }
  uint8_t output[kBridgeBufferCapacity];
  int64_t detachResult = motolii_rn_stage_detach(
      _hostHandle, _stageHandle, output, sizeof(output));
  if (detachResult > 0 && (uint64_t)detachResult <= sizeof(output)) {
    NSData *response = [NSData dataWithBytes:output length:(NSUInteger)detachResult];
    if (!BridgeAccepted(response)) {
      _snapshotText = SnapshotTextFromJSON(response);
    }
  }
  _surfaceAttached = NO;
}

- (void)sendPhysicalResize
{
  if (_stageHandle == 0 || !_surfaceAttached || self.window == nil) {
    return;
  }
  if (![NSThread isMainThread]) {
    _snapshotText = @"gpu resize rejected off main thread";
    return;
  }
  CGFloat scaleFactor = self.window.backingScaleFactor > 0 ? self.window.backingScaleFactor : 1.0;
  NSUInteger width = (NSUInteger)MAX(0, ceil(self.bounds.size.width * scaleFactor));
  NSUInteger height = (NSUInteger)MAX(0, ceil(self.bounds.size.height * scaleFactor));
  uint8_t output[kBridgeBufferCapacity];
  int64_t resizeResult = motolii_rn_stage_resize_physical(
      _hostHandle, _stageHandle, (uint32_t)width, (uint32_t)height, output, sizeof(output));
  if (resizeResult <= 0 || (uint64_t)resizeResult > sizeof(output) ||
      !BridgeAccepted([NSData dataWithBytes:output length:(NSUInteger)resizeResult])) {
    _snapshotText = @"gpu resize rejected";
  }
}

- (void)deactivateStage
{
  if (_stageHandle == 0) {
    return;
  }

  [self detachSurfaceIfNeeded];
  if (_mounted) {
    [self sendIntent:@"stage_unmount"];
  }
  uint8_t output[kBridgeBufferCapacity];
  motolii_rn_stage_destroy(_stageHandle, output, sizeof(output));
  _mounted = NO;
  _focused = NO;
  _surfaceAttached = NO;
  _surfaceRecoveryPending = NO;
  _stageHandle = 0;
  _snapshotText = @"stage not mounted";
  [self setNeedsDisplay:YES];
}

- (void)sendResizeIntent
{
  if (_stageHandle == 0 || self.window == nil) {
    return;
  }
  CGFloat scaleFactor = self.window.backingScaleFactor;
  [self sendIntent:@"stage_resize"
             width:self.bounds.size.width
            height:self.bounds.size.height
       scaleFactor:scaleFactor
           focused:_focused];
}

- (void)sendFocusIntent
{
  if (_stageHandle == 0) {
    return;
  }
  [self sendIntent:@"stage_focus"
             width:self.bounds.size.width
            height:self.bounds.size.height
       scaleFactor:self.window.backingScaleFactor
           focused:_focused];
}

- (void)sendIntent:(NSString *)kind
{
  [self sendIntent:kind
             width:self.bounds.size.width
            height:self.bounds.size.height
       scaleFactor:self.window.backingScaleFactor
           focused:_focused];
}

- (void)sendIntent:(NSString *)kind
             width:(CGFloat)width
            height:(CGFloat)height
       scaleFactor:(CGFloat)scaleFactor
           focused:(BOOL)focused
{
  if (_hostHandle == 0 || _stageHandle == 0) {
    return;
  }

  NSData *request =
      IntentData(kind, _hostHandle, _stageHandle, width, height, scaleFactor, focused);
  if (request == nil) {
    _snapshotText = @"intent encoding failed";
    [self setNeedsDisplay:YES];
    return;
  }

  uint8_t output[kBridgeBufferCapacity];
  int64_t result = motolii_rn_host_dispatch_intent_json(
      _hostHandle,
      (const uint8_t *)request.bytes,
      request.length,
      output,
      sizeof(output));
  if (result <= 0 || (uint64_t)result > sizeof(output)) {
    _snapshotText = @"intent rejected by host bridge";
  } else {
    NSData *response = [NSData dataWithBytes:output length:(NSUInteger)result];
    _snapshotText = SnapshotTextFromJSON(response);
  }
  [self setNeedsDisplay:YES];
}

- (void)readSnapshot
{
  if (_hostHandle == 0) {
    return;
  }
  uint8_t output[kBridgeBufferCapacity];
  int64_t result =
      motolii_rn_host_read_snapshot_json(_hostHandle, output, sizeof(output));
  if (result <= 0 || (uint64_t)result > sizeof(output)) {
    _snapshotText = @"snapshot read rejected";
  } else {
    NSData *response = [NSData dataWithBytes:output length:(NSUInteger)result];
    _snapshotText = SnapshotTextFromJSON(response);
  }
  [self setNeedsDisplay:YES];
}

@end
