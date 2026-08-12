#import "MotoliiGpuComponentView.h"

#import <QuartzCore/CAMetalLayer.h>
#import <cstring>
#import <cstdint>
#import <react/renderer/components/MotoliiNativeSpec/ComponentDescriptors.h>
#import <react/renderer/components/MotoliiNativeSpec/EventEmitters.h>
#import <react/renderer/components/MotoliiNativeSpec/Props.h>
#import <react/renderer/components/MotoliiNativeSpec/RCTComponentViewHelpers.h>

using namespace facebook::react;

extern "C" void *motolii_macos_renderer_create_ca_layer(void *layer, uint32_t width, uint32_t height);
extern "C" bool motolii_macos_renderer_resize(void *handle, uint32_t width, uint32_t height);
extern "C" bool motolii_macos_renderer_render(void *handle);
extern "C" bool motolii_macos_stage_renderer_pointer(void *handle, uint32_t phase, double x, double y);
extern "C" bool motolii_macos_stage_renderer_set_created_item(void *handle, const char *itemId);
extern "C" bool motolii_rnapp_stage_mount(double width, double height, double scale_factor);
extern "C" bool motolii_rnapp_stage_resize(double width, double height, double scale_factor);
extern "C" bool motolii_rnapp_stage_unmount(void);
extern "C" bool motolii_rnapp_stage_pointer(const char *phase, double view_local_x, double view_local_y);
typedef struct {
  double x;
  double y;
  double z;
  double rotationX;
  double rotationY;
  double rotationZ;
} MotoliiStageTransform;
extern "C" bool motolii_macos_stage_renderer_get_transform(
    void *handle, MotoliiStageTransform *transform);
extern "C" bool motolii_macos_stage_renderer_set_transform(
    void *handle, MotoliiStageTransform transform);
extern "C" void motolii_macos_renderer_destroy(void *handle);
extern "C" void *motolii_macos_timeline_renderer_create_ca_layer(void *layer, uint32_t width, uint32_t height);
extern "C" bool motolii_macos_timeline_renderer_set_state(void *handle, int32_t selectedObjectIndex, double playhead);

typedef struct {
  int32_t objectIndex;
  double time;
} MotoliiTimelineFeedback;

typedef struct {
  uint64_t frameCount;
  uint64_t lastCpuUs;
  uint64_t maxCpuUs;
  uint64_t vertexBytes;
  uint64_t overlayUploads;
  uint64_t overlayLastUs;
  uint64_t pointerDowns;
  uint64_t pointerMoves;
  uint64_t pointerUps;
} MotoliiRenderStats;

extern "C" bool motolii_macos_timeline_renderer_hit_test(
    void *handle, double x, double y, MotoliiTimelineFeedback *feedback);
extern "C" int32_t motolii_macos_timeline_renderer_hover_cursor(void *handle, double x, double y);
extern "C" int32_t motolii_macos_stage_renderer_hover_cursor(void *handle, double x, double y);
extern "C" bool motolii_macos_timeline_renderer_pointer(
    void *handle, uint32_t phase, double x, double y, uint32_t modifiers,
    MotoliiTimelineFeedback *feedback);
extern "C" bool motolii_macos_timeline_renderer_scroll(
    void *handle, double deltaX, double deltaY, double magnification, uint32_t modifiers, double x,
    double y);
extern "C" bool motolii_macos_renderer_get_stats(void *handle, MotoliiRenderStats *stats);
extern "C" bool motolii_rnapp_host_keymap(const uint8_t *kind_utf8, size_t kind_len);
extern "C" bool motolii_macos_timeline_renderer_keymap_delete(void *handle);

/// Rust cursor code → NSCursor。0 arrow / 1 resizeLR / 2 openHand / 3 closedHand / 4 pointingHand
static void MotoliiApplyCursor(int32_t code)
{
  NSCursor *cursor = [NSCursor arrowCursor];
  switch (code) {
    case 1:
      cursor = [NSCursor resizeLeftRightCursor];
      break;
    case 2:
      cursor = [NSCursor openHandCursor];
      break;
    case 3:
      cursor = [NSCursor closedHandCursor];
      break;
    case 4:
      cursor = [NSCursor pointingHandCursor];
      break;
    default:
      break;
  }
  [cursor set];
}

/// Cmd+Z / Shift+Cmd+Z / Delete|Backspace だけをhostへ転送。認識した鍵はYES(superへ送らない)。
static BOOL MotoliiDispatchKeymap(NSEvent *event)
{
  NSEventModifierFlags mods = event.modifierFlags & NSEventModifierFlagDeviceIndependentFlagsMask;
  NSString *chars = event.charactersIgnoringModifiers ?: @"";

  if (mods == NSEventModifierFlagCommand && [chars isEqualToString:@"z"]) {
    (void)motolii_rnapp_host_keymap((const uint8_t *)"undo", 4);
    return YES;
  }

  if (mods == (NSEventModifierFlagCommand | NSEventModifierFlagShift)
      && [chars isEqualToString:@"z"]) {
    (void)motolii_rnapp_host_keymap((const uint8_t *)"redo", 4);
    return YES;
  }

  if (mods == 0 && (event.keyCode == 51 || event.keyCode == 117)) {
    (void)motolii_rnapp_host_keymap((const uint8_t *)"delete_layer", 12);
    return YES;
  }
  return NO;
}

@interface MotoliiMetalView : NSView
@end

@interface MotoliiTimelineMetalView : MotoliiMetalView
@property(nonatomic, copy) void (^timelinePointerHandler)(uint32_t phase, CGFloat x, CGFloat y, uint32_t modifiers);
@property(nonatomic, copy) void (^timelineScrollHandler)(
    CGFloat deltaX, CGFloat deltaY, CGFloat magnification, uint32_t modifiers, CGFloat x, CGFloat y);
@property(nonatomic, copy) void (^timelineDeleteHandler)(void);
@property(nonatomic, copy) void (^timelineHoverHandler)(CGFloat x, CGFloat y);
@property(nonatomic, strong) NSTrackingArea *trackingArea;
@property(nonatomic, assign) BOOL timelineGestureActive;
@end

@interface MotoliiStageMetalView : MotoliiMetalView
@property(nonatomic, copy) void (^stagePointerHandler)(uint32_t phase, CGFloat x, CGFloat y);
@property(nonatomic, copy) void (^stageHoverHandler)(CGFloat x, CGFloat y);
@property(nonatomic, strong) NSTrackingArea *trackingArea;
@property(nonatomic, assign) BOOL stageGestureActive;
@end

@implementation MotoliiStageMetalView
- (BOOL)acceptsFirstResponder { return YES; }
- (void)updateTrackingAreas
{
  [super updateTrackingAreas];
  if (self.trackingArea) {
    [self removeTrackingArea:self.trackingArea];
    self.trackingArea = nil;
  }
  NSTrackingArea *area = [[NSTrackingArea alloc]
      initWithRect:self.bounds
           options:(NSTrackingMouseEnteredAndExited | NSTrackingMouseMoved |
                    NSTrackingActiveInKeyWindow | NSTrackingInVisibleRect)
             owner:self
          userInfo:nil];
  self.trackingArea = area;
  [self addTrackingArea:area];
}
- (void)emitPhase:(uint32_t)phase event:(NSEvent *)event
{
  if (self.stageGestureActive && (phase == 2 || phase == 3)) {
    self.stageGestureActive = NO;
  }

  NSPoint point = [self convertPoint:event.locationInWindow fromView:nil];
  if (!self.stagePointerHandler) {
    return;
  }
  if (phase == 0) {
    if (self.stageGestureActive) {
      return;
    }
    self.stageGestureActive = YES;
    self.stagePointerHandler(phase, point.x, NSHeight(self.bounds) - point.y);
    return;
  }
  if (!self.stageGestureActive) {
    return;
  }
  self.stagePointerHandler(phase, point.x, NSHeight(self.bounds) - point.y);
}
- (void)mouseDown:(NSEvent *)event
{
  [self.window makeFirstResponder:self];
  [self emitPhase:0 event:event];
}
- (void)mouseDragged:(NSEvent *)event
{
  [self emitPhase:1 event:event];
  if (self.stageHoverHandler) {
    NSPoint point = [self convertPoint:event.locationInWindow fromView:nil];
    self.stageHoverHandler(point.x, NSHeight(self.bounds) - point.y);
  }
}
- (void)mouseUp:(NSEvent *)event
{
  [self emitPhase:2 event:event];
  // gesture終了後、現在位置でcursorを再計算(closedHand残留を防ぐ)。
  if (self.stageHoverHandler) {
    NSPoint point = [self convertPoint:event.locationInWindow fromView:nil];
    self.stageHoverHandler(point.x, NSHeight(self.bounds) - point.y);
  }
}
- (void)mouseMoved:(NSEvent *)event
{
  if (!self.stageHoverHandler) {
    return;
  }
  NSPoint point = [self convertPoint:event.locationInWindow fromView:nil];
  self.stageHoverHandler(point.x, NSHeight(self.bounds) - point.y);
}
// AppKitはdrag中のmouseDragged/Upをview外でも配送する。mouseExited即cancelしない。
- (void)mouseExited:(NSEvent *)event
{
  (void)event;
  MotoliiApplyCursor(0);
}

- (void)keyDown:(NSEvent *)event
{
  if (!MotoliiDispatchKeymap(event)) {
    [super keyDown:event];
  }
}

- (void)viewDidMoveToWindow
{
  [super viewDidMoveToWindow];
  // window喪失時だけcancelを維持。
  if (!self.window && self.stageGestureActive && self.stagePointerHandler) {
    self.stagePointerHandler(3, 0, 0);
    self.stageGestureActive = NO;
  }
}
@end

@implementation MotoliiTimelineMetalView

- (BOOL)acceptsFirstResponder
{
  return YES;
}

- (void)updateTrackingAreas
{
  [super updateTrackingAreas];
  if (self.trackingArea) {
    [self removeTrackingArea:self.trackingArea];
    self.trackingArea = nil;
  }
  NSTrackingArea *area = [[NSTrackingArea alloc]
      initWithRect:self.bounds
           options:(NSTrackingMouseEnteredAndExited | NSTrackingMouseMoved |
                    NSTrackingActiveInKeyWindow | NSTrackingInVisibleRect)
             owner:self
          userInfo:nil];
  self.trackingArea = area;
  [self addTrackingArea:area];
}

- (void)emitPhase:(uint32_t)phase event:(NSEvent *)event
{
  NSPoint point = [self convertPoint:event.locationInWindow fromView:nil];
  uint32_t modifiers = (event.modifierFlags & NSEventModifierFlagCommand) ? 1u : 0u;
  if (self.timelinePointerHandler) {
    self.timelinePointerHandler(phase, point.x, NSHeight(self.bounds) - point.y, modifiers);
  }
  if (phase == 0) {
    self.timelineGestureActive = YES;
  } else if (phase == 2 || phase == 3) {
    self.timelineGestureActive = NO;
  }
}

- (void)mouseDown:(NSEvent *)event
{
  [self.window makeFirstResponder:self];
  [self emitPhase:0 event:event];
}

- (void)mouseDragged:(NSEvent *)event
{
  [self emitPhase:1 event:event];
  if (self.timelineHoverHandler) {
    NSPoint point = [self convertPoint:event.locationInWindow fromView:nil];
    self.timelineHoverHandler(point.x, NSHeight(self.bounds) - point.y);
  }
}

- (void)mouseUp:(NSEvent *)event
{
  [self emitPhase:2 event:event];
  // gesture終了後、現在位置でcursorを再計算(closedHand残留を防ぐ)。
  if (self.timelineHoverHandler) {
    NSPoint point = [self convertPoint:event.locationInWindow fromView:nil];
    self.timelineHoverHandler(point.x, NSHeight(self.bounds) - point.y);
  }
}

- (void)mouseMoved:(NSEvent *)event
{
  if (!self.timelineHoverHandler) {
    return;
  }
  NSPoint point = [self convertPoint:event.locationInWindow fromView:nil];
  self.timelineHoverHandler(point.x, NSHeight(self.bounds) - point.y);
}

- (void)mouseExited:(NSEvent *)event
{
  (void)event;
  MotoliiApplyCursor(0);
}

- (void)keyDown:(NSEvent *)event
{
  NSEventModifierFlags mods = event.modifierFlags & NSEventModifierFlagDeviceIndependentFlagsMask;
  if (mods == 0 && (event.keyCode == 51 || event.keyCode == 117) && self.timelineDeleteHandler) {
    self.timelineDeleteHandler();
    return;
  }
  if (!MotoliiDispatchKeymap(event)) {
    [super keyDown:event];
  }
}

- (void)emitScrollDeltaX:(CGFloat)deltaX
                  deltaY:(CGFloat)deltaY
           magnification:(CGFloat)magnification
               modifiers:(uint32_t)modifiers
                   event:(NSEvent *)event
{
  NSPoint point = [self convertPoint:event.locationInWindow fromView:nil];
  if (self.timelineScrollHandler) {
    self.timelineScrollHandler(
        deltaX, deltaY, magnification, modifiers, point.x, NSHeight(self.bounds) - point.y);
  }
}

- (void)scrollWheel:(NSEvent *)event
{
  uint32_t modifiers = (event.modifierFlags & NSEventModifierFlagCommand) ? 1u : 0u;
  [self emitScrollDeltaX:event.scrollingDeltaX
                  deltaY:event.scrollingDeltaY
           magnification:0.0
               modifiers:modifiers
                   event:event];
}

- (void)magnifyWithEvent:(NSEvent *)event
{
  [self emitScrollDeltaX:0.0
                  deltaY:0.0
           magnification:event.magnification
               modifiers:0u
                   event:event];
}

- (void)viewDidMoveToWindow
{
  [super viewDidMoveToWindow];
  if (!self.window && self.timelineGestureActive && self.timelinePointerHandler) {
    self.timelinePointerHandler(3, 0, 0, 0);
    self.timelineGestureActive = NO;
  }
}

@end

@implementation MotoliiMetalView

- (CALayer *)makeBackingLayer
{
  CAMetalLayer *layer = [CAMetalLayer layer];
  layer.framebufferOnly = YES;
  layer.opaque = YES;
  layer.pixelFormat = MTLPixelFormatBGRA8Unorm_sRGB;
  return layer;
}

@end

@interface MotoliiTimelineComponentView () <RCTMotoliiTimelineViewViewProtocol>
@end

@implementation MotoliiTimelineComponentView {
  MotoliiTimelineMetalView *_timelineView;
  NSTimer *_timelineTimer;
  void *_timelineRenderer;
  int32_t _selectedObjectIndex;
  double _playhead;
}

+ (ComponentDescriptorProvider)componentDescriptorProvider
{
  return concreteComponentDescriptorProvider<MotoliiTimelineViewComponentDescriptor>();
}

- (instancetype)init
{
  if (self = [super init]) {
    _props = MotoliiTimelineViewShadowNode::defaultSharedProps();
    _selectedObjectIndex = 1;
    _playhead = 0.54;
    _timelineView = [MotoliiTimelineMetalView new];
    _timelineView.wantsLayer = YES;
    _timelineView.autoresizingMask = NSViewWidthSizable | NSViewHeightSizable;
    __weak MotoliiTimelineComponentView *weakSelf = self;
    _timelineView.timelinePointerHandler = ^(uint32_t phase, CGFloat x, CGFloat y, uint32_t modifiers) {
      MotoliiTimelineComponentView *strongSelf = weakSelf;
      if (!strongSelf || !strongSelf->_timelineRenderer) {
        return;
      }
      CAMetalLayer *layer = (CAMetalLayer *)strongSelf->_timelineView.layer;
      CGFloat scale = layer.contentsScale ?: 1.0;
      MotoliiTimelineFeedback feedback = {};
      if (!motolii_macos_timeline_renderer_pointer(
              strongSelf->_timelineRenderer, phase, x * scale, y * scale, modifiers, &feedback)) {
        return;
      }
      if (!strongSelf->_eventEmitter) {
        return;
      }
      auto emitter = std::static_pointer_cast<const MotoliiTimelineViewEventEmitter>(
          strongSelf->_eventEmitter);
      MotoliiTimelineViewEventEmitter::OnTimelineFeedback event = {
          .objectIndex = feedback.objectIndex,
          .time = feedback.time,
      };
      emitter->onTimelineFeedback(event);
    };
    _timelineView.timelineDeleteHandler = ^{
      MotoliiTimelineComponentView *strongSelf = weakSelf;
      if (!strongSelf || !strongSelf->_timelineRenderer) {
        return;
      }
      (void)motolii_macos_timeline_renderer_keymap_delete(strongSelf->_timelineRenderer);
    };
    _timelineView.timelineScrollHandler =
        ^(CGFloat deltaX, CGFloat deltaY, CGFloat magnification, uint32_t modifiers, CGFloat x,
          CGFloat y) {
          MotoliiTimelineComponentView *strongSelf = weakSelf;
          if (!strongSelf || !strongSelf->_timelineRenderer) {
            return;
          }
          CAMetalLayer *layer = (CAMetalLayer *)strongSelf->_timelineView.layer;
          CGFloat scale = layer.contentsScale ?: 1.0;
          motolii_macos_timeline_renderer_scroll(
              strongSelf->_timelineRenderer, deltaX * scale, deltaY * scale, magnification,
              modifiers, x * scale, y * scale);
        };
    _timelineView.timelineHoverHandler = ^(CGFloat x, CGFloat y) {
      MotoliiTimelineComponentView *strongSelf = weakSelf;
      if (!strongSelf || !strongSelf->_timelineRenderer) {
        return;
      }
      CAMetalLayer *layer = (CAMetalLayer *)strongSelf->_timelineView.layer;
      CGFloat scale = layer.contentsScale ?: 1.0;
      int32_t code = motolii_macos_timeline_renderer_hover_cursor(
          strongSelf->_timelineRenderer, x * scale, y * scale);
      MotoliiApplyCursor(code);
    };
    [self addSubview:_timelineView];
  }
  return self;
}

- (void)updateProps:(const Props::Shared &)props oldProps:(const Props::Shared &)oldProps
{
  const auto &newProps = static_cast<const MotoliiTimelineViewProps &>(*props);
  _selectedObjectIndex = newProps.selectedObjectIndex;
  _playhead = newProps.playhead;
  if (_timelineRenderer) {
    motolii_macos_timeline_renderer_set_state(
        _timelineRenderer, _selectedObjectIndex, _playhead);
  }
  [super updateProps:props oldProps:oldProps];
}

- (void)viewDidMoveToWindow
{
  [super viewDidMoveToWindow];
  if (self.window) {
    [self startTimelineRendererIfNeeded];
  } else {
    [self stopTimelineRenderer];
  }
}

- (void)layout
{
  [super layout];
  _timelineView.frame = self.bounds;
  [self resizeTimelineRenderer];
}

- (void)prepareForRecycle
{
  [self stopTimelineRenderer];
  [super prepareForRecycle];
}

- (void)dealloc
{
  [self stopTimelineRenderer];
}

- (void)startTimelineRendererIfNeeded
{
  if (_timelineRenderer || !_timelineView.layer || NSWidth(self.bounds) <= 0 || NSHeight(self.bounds) <= 0) {
    return;
  }
  CAMetalLayer *layer = (CAMetalLayer *)_timelineView.layer;
  CGFloat scale = self.window.backingScaleFactor ?: 1.0;
  layer.contentsScale = scale;
  layer.drawableSize = CGSizeMake(NSWidth(self.bounds) * scale, NSHeight(self.bounds) * scale);
  _timelineRenderer = motolii_macos_timeline_renderer_create_ca_layer(
      (__bridge void *)layer,
      (uint32_t)MAX(1.0, layer.drawableSize.width),
      (uint32_t)MAX(1.0, layer.drawableSize.height));
  if (!_timelineRenderer) {
    NSLog(@"[MotoliiTimelineProbe] renderer creation failed");
    return;
  }
  motolii_macos_timeline_renderer_set_state(
      _timelineRenderer, _selectedObjectIndex, _playhead);
  __weak MotoliiTimelineComponentView *weakSelf = self;
  _timelineTimer = [NSTimer scheduledTimerWithTimeInterval:(1.0 / 60.0)
                                                   repeats:YES
                                                     block:^(__unused NSTimer *timer) {
    MotoliiTimelineComponentView *strongSelf = weakSelf;
    if (!strongSelf || !strongSelf->_timelineRenderer) {
      return;
    }
    motolii_macos_renderer_render(strongSelf->_timelineRenderer);
    MotoliiRenderStats stats = {};
    if (motolii_macos_renderer_get_stats(strongSelf->_timelineRenderer, &stats) &&
        stats.frameCount > 0 && stats.frameCount % 120 == 0) {
      NSLog(@"[MotoliiTimelineProbe] frames=%llu cpu=%lluus max=%lluus vertices=%llubytes",
            stats.frameCount, stats.lastCpuUs, stats.maxCpuUs, stats.vertexBytes);
    }
  }];
  NSLog(@"[MotoliiTimelineProbe] renderer mounted %.0fx%.0f @ %.1fx",
        layer.drawableSize.width, layer.drawableSize.height, scale);
}

- (void)resizeTimelineRenderer
{
  if (!self.window) {
    return;
  }
  [self startTimelineRendererIfNeeded];
  if (!_timelineRenderer) {
    return;
  }
  CAMetalLayer *layer = (CAMetalLayer *)_timelineView.layer;
  CGFloat scale = self.window.backingScaleFactor ?: 1.0;
  CGSize newSize = CGSizeMake(MAX(1.0, NSWidth(self.bounds) * scale),
                              MAX(1.0, NSHeight(self.bounds) * scale));
  layer.contentsScale = scale;
  layer.drawableSize = newSize;
  motolii_macos_renderer_resize(
      _timelineRenderer, (uint32_t)newSize.width, (uint32_t)newSize.height);
}

- (void)stopTimelineRenderer
{
  if (_timelineRenderer && _timelineView.timelineGestureActive) {
    if (_timelineView.timelinePointerHandler) {
      _timelineView.timelinePointerHandler(3, 0, 0, 0);
    } else {
      MotoliiTimelineFeedback feedback = {};
      motolii_macos_timeline_renderer_pointer(_timelineRenderer, 3, 0.0, 0.0, 0u, &feedback);
    }
    _timelineView.timelineGestureActive = NO;
  }
  [_timelineTimer invalidate];
  _timelineTimer = nil;
  if (_timelineRenderer) {
    motolii_macos_renderer_destroy(_timelineRenderer);
    _timelineRenderer = nullptr;
    NSLog(@"[MotoliiTimelineProbe] renderer unmounted");
  }
}

@end

@interface MotoliiGpuComponentView () <RCTMotoliiGpuViewViewProtocol>
@end

@implementation MotoliiGpuComponentView {
  MotoliiStageMetalView *_metalView;
  NSTimer *_frameTimer;
  void *_renderer;
  std::string _createdItemId;
  std::string _draggedItemId;
  MotoliiStageTransform _lastTransform;
  BOOL _hasLastTransform;
  std::string _lastSentCreatedItemId;
  MotoliiStageTransform _lastSentTransform;
  BOOL _hasLastSentTransform;
  id _dropMonitor;
}

+ (ComponentDescriptorProvider)componentDescriptorProvider
{
  return concreteComponentDescriptorProvider<MotoliiGpuViewComponentDescriptor>();
}

- (instancetype)init
{
  if (self = [super init]) {
    _props = MotoliiGpuViewShadowNode::defaultSharedProps();
    _metalView = [MotoliiStageMetalView new];
    _metalView.wantsLayer = YES;
    _metalView.autoresizingMask = NSViewWidthSizable | NSViewHeightSizable;
    [_metalView setAccessibilityElement:YES];
    [_metalView setAccessibilityLabel:@"Rerun Spatial Stage metrics"];
    [_metalView setAccessibilityIdentifier:@"rerun-spatial-stage-metrics"];
    __weak MotoliiGpuComponentView *weakSelf = self;
    _metalView.stagePointerHandler = ^(uint32_t phase, CGFloat x, CGFloat y) {
      MotoliiGpuComponentView *strongSelf = weakSelf;
      if (!strongSelf || !strongSelf->_renderer) return;
      // host seat へは view-local logical。sequence は bridge が採番。
      const char *phaseName = "cancel";
      if (phase == 0) phaseName = "down";
      else if (phase == 1) phaseName = "drag";
      else if (phase == 2) phaseName = "up";
      (void)motolii_rnapp_stage_pointer(phaseName, x, y);
      // host 投影 active 時は renderer_core 側で gizmo probe を止める。
      CAMetalLayer *layer = (CAMetalLayer *)strongSelf->_metalView.layer;
      CGFloat scale = layer.contentsScale ?: 1.0;
      motolii_macos_stage_renderer_pointer(strongSelf->_renderer, phase, x * scale, y * scale);
    };
    _metalView.stageHoverHandler = ^(CGFloat x, CGFloat y) {
      MotoliiGpuComponentView *strongSelf = weakSelf;
      if (!strongSelf || !strongSelf->_renderer) {
        return;
      }
      CAMetalLayer *layer = (CAMetalLayer *)strongSelf->_metalView.layer;
      CGFloat scale = layer.contentsScale ?: 1.0;
      int32_t code =
          motolii_macos_stage_renderer_hover_cursor(strongSelf->_renderer, x * scale, y * scale);
      MotoliiApplyCursor(code);
    };
    [self addSubview:_metalView];
  }
  return self;
}

- (void)finishBrowserDrop:(NSEvent *)event
{
  if (_dropMonitor) {
    [NSEvent removeMonitor:_dropMonitor];
    _dropMonitor = nil;
  }
  if (!_eventEmitter) {
    return;
  }

  NSPoint point = [_metalView convertPoint:event.locationInWindow fromView:nil];
  double x = -1.0;
  double y = -1.0;
  double canonicalX = 0.0;
  double canonicalY = 0.0;
  CGFloat logicalW = NSWidth(_metalView.bounds);
  CGFloat logicalH = NSHeight(_metalView.bounds);
  if (NSPointInRect(point, _metalView.bounds) && logicalW > 0 && logicalH > 0) {
    x = point.x / logicalW;
    y = 1.0 - point.y / logicalH;
    // host 正準: cx=(nx-0.5)*(w/h), cy=0.5-ny（logical viewport）
    canonicalX = (x - 0.5) * (logicalW / logicalH);
    canonicalY = 0.5 - y;
  }
  auto emitter = std::static_pointer_cast<const MotoliiGpuViewEventEmitter>(_eventEmitter);
  MotoliiGpuViewEventEmitter::OnStageDrop drop = {
      .x = x,
      .y = y,
      .canonicalX = canonicalX,
      .canonicalY = canonicalY,
  };
  emitter->onStageDrop(drop);
}

- (void)emitStageTransformIfChanged
{
  if (!_renderer || !_eventEmitter) return;
  MotoliiStageTransform transform = {};
  if (!motolii_macos_stage_renderer_get_transform(_renderer, &transform)) return;
  if (_hasLastTransform && std::memcmp(&_lastTransform, &transform, sizeof(transform)) == 0) return;
  _lastTransform = transform;
  _hasLastTransform = YES;
  auto emitter = std::static_pointer_cast<const MotoliiGpuViewEventEmitter>(_eventEmitter);
  MotoliiGpuViewEventEmitter::OnStageTransform event = {
      .x = transform.x,
      .y = transform.y,
      .z = transform.z,
      .rotationX = transform.rotationX,
      .rotationY = transform.rotationY,
      .rotationZ = transform.rotationZ,
  };
  emitter->onStageTransform(event);
}

- (void)updateProps:(const Props::Shared &)props oldProps:(const Props::Shared &)oldProps
{
  const auto &newProps = static_cast<const MotoliiGpuViewProps &>(*props);
  _createdItemId = newProps.createdItemId;
  _draggedItemId = newProps.draggedItemId;
  if (_renderer) {
    if (_createdItemId != _lastSentCreatedItemId) {
      motolii_macos_stage_renderer_set_created_item(_renderer, _createdItemId.c_str());
      _lastSentCreatedItemId = _createdItemId;
    }
    MotoliiStageTransform transform = {
        .x = newProps.transformX,
        .y = newProps.transformY,
        .z = newProps.transformZ,
        .rotationX = newProps.rotationX,
        .rotationY = newProps.rotationY,
        .rotationZ = newProps.rotationZ,
    };
    if (!_hasLastSentTransform ||
        std::memcmp(&_lastSentTransform, &transform, sizeof(transform)) != 0) {
      motolii_macos_stage_renderer_set_transform(_renderer, transform);
      _lastSentTransform = transform;
      _hasLastSentTransform = YES;
    }
  }
  if (!_draggedItemId.empty() && !_dropMonitor) {
    __weak MotoliiGpuComponentView *weakSelf = self;
    _dropMonitor = [NSEvent addLocalMonitorForEventsMatchingMask:NSEventMaskLeftMouseUp handler:^NSEvent *(NSEvent *event) {
      MotoliiGpuComponentView *strongSelf = weakSelf;
      [strongSelf finishBrowserDrop:event];
      return event;
    }];
  } else if (_draggedItemId.empty() && _dropMonitor) {
    [NSEvent removeMonitor:_dropMonitor];
    _dropMonitor = nil;
  }
  [super updateProps:props oldProps:oldProps];
}

- (void)viewDidMoveToWindow
{
  [super viewDidMoveToWindow];
  if (self.window) {
    [self startRendererIfNeeded];
  } else {
    [self stopRenderer];
  }
}

- (void)layout
{
  [super layout];
  _metalView.frame = self.bounds;
  [self resizeRenderer];
}

- (void)prepareForRecycle
{
  if (_dropMonitor) {
    [NSEvent removeMonitor:_dropMonitor];
    _dropMonitor = nil;
  }
  [self stopRenderer];
  [super prepareForRecycle];
}

- (void)dealloc
{
  if (_dropMonitor) {
    [NSEvent removeMonitor:_dropMonitor];
  }
  [self stopRenderer];
}

- (void)startRendererIfNeeded
{
  if (_renderer || !_metalView.layer || NSWidth(self.bounds) <= 0 || NSHeight(self.bounds) <= 0) {
    return;
  }

  CAMetalLayer *layer = (CAMetalLayer *)_metalView.layer;
  CGFloat scale = self.window.backingScaleFactor ?: 1.0;
  layer.contentsScale = scale;
  layer.drawableSize = CGSizeMake(NSWidth(self.bounds) * scale, NSHeight(self.bounds) * scale);
  _renderer = motolii_macos_renderer_create_ca_layer(
      (__bridge void *)layer,
      (uint32_t)MAX(1.0, layer.drawableSize.width),
      (uint32_t)MAX(1.0, layer.drawableSize.height));
  if (!_renderer) {
    NSLog(@"[MotoliiRerunStage] renderer creation failed");
    return;
  }
  motolii_macos_stage_renderer_set_created_item(_renderer, _createdItemId.c_str());
  _lastSentCreatedItemId = _createdItemId;
  const auto &props = static_cast<const MotoliiGpuViewProps &>(*_props);
  MotoliiStageTransform transform = {
      .x = props.transformX,
      .y = props.transformY,
      .z = props.transformZ,
      .rotationX = props.rotationX,
      .rotationY = props.rotationY,
      .rotationZ = props.rotationZ,
  };
  motolii_macos_stage_renderer_set_transform(_renderer, transform);
  _lastSentTransform = transform;
  _hasLastSentTransform = YES;

  // host stage seat: logical bounds + contentsScale
  (void)motolii_rnapp_stage_mount(NSWidth(self.bounds), NSHeight(self.bounds), scale);

  __weak MotoliiGpuComponentView *weakSelf = self;
  _frameTimer = [NSTimer scheduledTimerWithTimeInterval:(1.0 / 60.0)
                                                repeats:YES
                                                  block:^(__unused NSTimer *timer) {
    MotoliiGpuComponentView *strongSelf = weakSelf;
    if (strongSelf && strongSelf->_renderer) {
      motolii_macos_renderer_render(strongSelf->_renderer);
      [strongSelf emitStageTransformIfChanged];
      MotoliiRenderStats stats = {};
      if (motolii_macos_renderer_get_stats(strongSelf->_renderer, &stats) &&
          stats.frameCount > 0 && stats.frameCount % 120 == 0) {
        [strongSelf->_metalView setAccessibilityValue:
            [NSString stringWithFormat:@"frames=%llu uploads=%llu overlay=%lluus pointer=%llu/%llu/%llu",
                stats.frameCount, stats.overlayUploads, stats.overlayLastUs,
                stats.pointerDowns, stats.pointerMoves, stats.pointerUps]];
        NSLog(@"[MotoliiRerunStage] frames=%llu cpu=%lluus max=%lluus overlay_uploads=%llu overlay=%lluus pointer=%llu/%llu/%llu",
              stats.frameCount, stats.lastCpuUs, stats.maxCpuUs,
              stats.overlayUploads, stats.overlayLastUs,
              stats.pointerDowns, stats.pointerMoves, stats.pointerUps);
      }
    }
  }];
  NSLog(@"[MotoliiRerunStage] renderer mounted %.0fx%.0f @ %.1fx",
        layer.drawableSize.width, layer.drawableSize.height, scale);
}

- (void)resizeRenderer
{
  if (!self.window) {
    return;
  }
  [self startRendererIfNeeded];
  if (!_renderer) {
    return;
  }
  CAMetalLayer *layer = (CAMetalLayer *)_metalView.layer;
  CGFloat scale = self.window.backingScaleFactor ?: 1.0;
  CGSize oldSize = layer.drawableSize;
  CGSize newSize = CGSizeMake(MAX(1.0, NSWidth(self.bounds) * scale),
                              MAX(1.0, NSHeight(self.bounds) * scale));
  layer.contentsScale = scale;
  layer.drawableSize = newSize;
  motolii_macos_renderer_resize(_renderer,
                                (uint32_t)layer.drawableSize.width,
                                (uint32_t)layer.drawableSize.height);
  (void)motolii_rnapp_stage_resize(NSWidth(self.bounds), NSHeight(self.bounds), scale);
  if (!CGSizeEqualToSize(oldSize, newSize)) {
    NSLog(@"[MotoliiRerunStage] renderer resized %.0fx%.0f @ %.1fx",
          newSize.width, newSize.height, scale);
  }
}

- (void)stopRenderer
{
  [_frameTimer invalidate];
  _frameTimer = nil;
  if (_renderer) {
    (void)motolii_rnapp_stage_unmount();
    motolii_macos_renderer_destroy(_renderer);
    _renderer = nullptr;
    NSLog(@"[MotoliiRerunStage] renderer unmounted");
  }
}

@end
