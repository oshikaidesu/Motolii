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
extern "C" bool motolii_macos_stage_renderer_pointer(
    void *handle, uint32_t phase, uint32_t button, uint32_t modifiers, double x, double y);
extern "C" bool motolii_macos_stage_renderer_scroll(
    void *handle, double deltaX, double deltaY, double magnification, uint32_t modifiers, double x,
    double y);
extern "C" bool motolii_macos_stage_renderer_set_created_item(void *handle, const char *itemId);
extern "C" bool MotoliiEnsureProductHost(void);
extern "C" bool motolii_rnapp_stage_mount(double width, double height, double scale_factor);
extern "C" bool motolii_rnapp_stage_resize(double width, double height, double scale_factor);
extern "C" bool motolii_rnapp_stage_unmount(void);
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
extern "C" int64_t motolii_macos_renderer_take_host_terminal(
    void *handle, bool *accepted, uint8_t *message, size_t messageCap);
extern "C" int32_t motolii_rnapp_host_key_event(
    uint16_t keyCode, uint32_t modifierBits, const uint8_t *charsUtf8, size_t charsLen,
    bool isRepeat, bool timelineFocused);
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

/// RN TextInput / フィールドエディタ / IME preedit。これ以外は製品keymapへ通す。
static BOOL MotoliiResponderIsTextInput(NSResponder *responder)
{
  if (responder == nil) {
    return NO;
  }
  if ([responder isKindOfClass:[NSTextView class]] || [responder isKindOfClass:[NSTextField class]]) {
    return YES;
  }
  Class rctField = NSClassFromString(@"RCTUITextField");
  Class rctView = NSClassFromString(@"RCTUITextView");
  Class rctInput = NSClassFromString(@"RCTTextInputComponentView");
  if ((rctField && [responder isKindOfClass:rctField]) || (rctView && [responder isKindOfClass:rctView]) ||
      (rctInput && [responder isKindOfClass:rctInput])) {
    return YES;
  }
  if ([responder conformsToProtocol:@protocol(NSTextInputClient)] &&
      [responder respondsToSelector:@selector(hasMarkedText)] &&
      [(id<NSTextInputClient>)responder hasMarkedText]) {
    return YES;
  }
  return NO;
}

static uint32_t MotoliiStageModifierBits(NSEventModifierFlags modifierFlags)
{
  NSEventModifierFlags flags = modifierFlags & NSEventModifierFlagDeviceIndependentFlagsMask;
  uint32_t bits = 0;
  if (flags & NSEventModifierFlagShift) bits |= 1u;
  if (flags & NSEventModifierFlagControl) bits |= 2u;
  if (flags & NSEventModifierFlagOption) bits |= 4u;
  if (flags & NSEventModifierFlagCommand) bits |= 8u;
  return bits;
}

/// 物理キーは表へ渡す。Space/Delete 定数で kind を焼かない。
static int32_t MotoliiDispatchKeymap(NSEvent *event, BOOL timelineFocused)
{
  // IME/TextInput は既存 host_key_event へ送らない。set_ime_gate FFI は未公開。
  if (MotoliiResponderIsTextInput(event.window.firstResponder)) {
    return 0;
  }
  // dispatchIntent と同じく key 経路でも slot を起こす。AppDelegate の ensure 失敗をキーで再試行する。
  (void)MotoliiEnsureProductHost();
  NSEventModifierFlags flags = event.modifierFlags & NSEventModifierFlagDeviceIndependentFlagsMask;
  uint32_t bits = 0;
  if (flags & NSEventModifierFlagShift) {
    bits |= 1u;
  }
  if (flags & NSEventModifierFlagControl) {
    bits |= 2u;
  }
  if (flags & NSEventModifierFlagOption) {
    bits |= 4u;
  }
  if (flags & NSEventModifierFlagCommand) {
    bits |= 8u;
  }
  NSString *chars = event.charactersIgnoringModifiers ?: @"";
  const char *utf8 = chars.UTF8String ?: "";
  size_t len = strlen(utf8);
  return motolii_rnapp_host_key_event(
      event.keyCode, bits, (const uint8_t *)utf8, len, event.isARepeat, timelineFocused);
}

@interface MotoliiMetalView : NSView
- (BOOL)cancelActiveGesture;
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
@property(nonatomic, copy) void (^stagePointerHandler)(
    uint32_t phase, uint32_t button, uint32_t modifiers, CGFloat x, CGFloat y);
@property(nonatomic, copy) void (^stageScrollHandler)(
    CGFloat deltaX, CGFloat deltaY, CGFloat magnification, uint32_t modifiers, CGFloat x,
    CGFloat y);
@property(nonatomic, copy) void (^stageHoverHandler)(CGFloat x, CGFloat y);
@property(nonatomic, strong) NSTrackingArea *trackingArea;
@property(nonatomic, assign) BOOL stageGestureActive;
@property(nonatomic, assign) uint32_t stageGestureButton;
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
- (void)emitPhase:(uint32_t)phase button:(uint32_t)button event:(NSEvent *)event
{
  NSPoint point = [self convertPoint:event.locationInWindow fromView:nil];
  if (!self.stagePointerHandler) {
    return;
  }
  if (phase == 0) {
    if (self.stageGestureActive) {
      return;
    }
    self.stageGestureActive = YES;
    self.stageGestureButton = button;
    self.stagePointerHandler(
        phase, button, MotoliiStageModifierBits(event.modifierFlags), point.x,
        NSHeight(self.bounds) - point.y);
    return;
  }
  if (!self.stageGestureActive || self.stageGestureButton != button) {
    return;
  }
  if (phase == 2 || phase == 3) {
    self.stageGestureActive = NO;
  }
  self.stagePointerHandler(
      phase, button, MotoliiStageModifierBits(event.modifierFlags), point.x,
      NSHeight(self.bounds) - point.y);
}
- (BOOL)cancelActiveGesture
{
  if (!self.stageGestureActive) {
    return NO;
  }
  self.stageGestureActive = NO;
  if (self.stagePointerHandler) {
    self.stagePointerHandler(3, self.stageGestureButton, 0, 0, 0);
  }
  return YES;
}
- (void)mouseDown:(NSEvent *)event
{
  [self.window makeFirstResponder:self];
  [self emitPhase:0 button:0 event:event];
}
- (void)mouseDragged:(NSEvent *)event
{
  [self emitPhase:1 button:0 event:event];
  if (self.stageHoverHandler) {
    NSPoint point = [self convertPoint:event.locationInWindow fromView:nil];
    self.stageHoverHandler(point.x, NSHeight(self.bounds) - point.y);
  }
}
- (void)mouseUp:(NSEvent *)event
{
  [self emitPhase:2 button:0 event:event];
  // gesture終了後、現在位置でcursorを再計算(closedHand残留を防ぐ)。
  if (self.stageHoverHandler) {
    NSPoint point = [self convertPoint:event.locationInWindow fromView:nil];
    self.stageHoverHandler(point.x, NSHeight(self.bounds) - point.y);
  }
}
- (void)rightMouseDown:(NSEvent *)event
{
  [self.window makeFirstResponder:self];
  [self emitPhase:0 button:1 event:event];
}
- (void)rightMouseDragged:(NSEvent *)event { [self emitPhase:1 button:1 event:event]; }
- (void)rightMouseUp:(NSEvent *)event { [self emitPhase:2 button:1 event:event]; }
- (void)otherMouseDown:(NSEvent *)event
{
  if (event.buttonNumber != 2) return;
  [self.window makeFirstResponder:self];
  [self emitPhase:0 button:2 event:event];
}
- (void)otherMouseDragged:(NSEvent *)event
{
  if (event.buttonNumber == 2) [self emitPhase:1 button:2 event:event];
}
- (void)otherMouseUp:(NSEvent *)event
{
  if (event.buttonNumber == 2) [self emitPhase:2 button:2 event:event];
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

- (void)emitScrollDeltaX:(CGFloat)deltaX
                  deltaY:(CGFloat)deltaY
           magnification:(CGFloat)magnification
                   event:(NSEvent *)event
{
  NSPoint point = [self convertPoint:event.locationInWindow fromView:nil];
  if (self.stageScrollHandler) {
    self.stageScrollHandler(
        deltaX, deltaY, magnification, MotoliiStageModifierBits(event.modifierFlags), point.x,
        NSHeight(self.bounds) - point.y);
  }
}

- (void)scrollWheel:(NSEvent *)event
{
  [self emitScrollDeltaX:event.scrollingDeltaX
                  deltaY:event.scrollingDeltaY
           magnification:0.0
                   event:event];
}

- (void)magnifyWithEvent:(NSEvent *)event
{
  [self emitScrollDeltaX:0.0
                  deltaY:0.0
           magnification:event.magnification
                   event:event];
}

- (void)keyDown:(NSEvent *)event
{
  if (event.keyCode == 53 && [self cancelActiveGesture]) {
    return;
  }
  if (MotoliiDispatchKeymap(event, NO) == 0) {
    [super keyDown:event];
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
  if (phase == 0) {
    if (self.timelineGestureActive) {
      return;
    }
    self.timelineGestureActive = YES;
  } else if (!self.timelineGestureActive) {
    return;
  } else if (phase == 2 || phase == 3) {
    self.timelineGestureActive = NO;
  }
  if (self.timelinePointerHandler) {
    self.timelinePointerHandler(phase, point.x, NSHeight(self.bounds) - point.y, modifiers);
  }
}

- (BOOL)cancelActiveGesture
{
  if (!self.timelineGestureActive) {
    return NO;
  }
  self.timelineGestureActive = NO;
  if (self.timelinePointerHandler) {
    self.timelinePointerHandler(3, 0, 0, 0);
  }
  return YES;
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
  if (event.keyCode == 53 && [self cancelActiveGesture]) {
    return;
  }
  int32_t result = MotoliiDispatchKeymap(event, YES);
  if (result == 2 && self.timelineDeleteHandler) {
    self.timelineDeleteHandler();
    return;
  }
  if (result == 0) {
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

@end

@implementation MotoliiMetalView

- (BOOL)cancelActiveGesture
{
  return NO;
}

- (BOOL)resignFirstResponder
{
  BOOL resigned = [super resignFirstResponder];
  if (resigned) {
    [self cancelActiveGesture];
  }
  return resigned;
}

- (void)windowDidResignKey:(NSNotification *)notification
{
  (void)notification;
  [self cancelActiveGesture];
}

- (void)viewDidMoveToWindow
{
  [super viewDidMoveToWindow];
  NSNotificationCenter *center = [NSNotificationCenter defaultCenter];
  [center removeObserver:self name:NSWindowDidResignKeyNotification object:nil];
  if (self.window) {
    [center addObserver:self
               selector:@selector(windowDidResignKey:)
                   name:NSWindowDidResignKeyNotification
                 object:self.window];
  } else {
    [self cancelActiveGesture];
  }
}

- (void)dealloc
{
  [[NSNotificationCenter defaultCenter] removeObserver:self];
}

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
- (void)emitHostTerminalIfPending;
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
    // 製品は未選択・t=0。fixture clip 1 / 0.54 を native 初期値に焼かない。
    _selectedObjectIndex = -1;
    _playhead = 0;
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
      bool hasFeedback = motolii_macos_timeline_renderer_pointer(
          strongSelf->_timelineRenderer, phase, x * scale, y * scale, modifiers, &feedback);
      [strongSelf emitHostTerminalIfPending];
      if (!hasFeedback) {
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

- (void)emitHostTerminalIfPending
{
  if (!_timelineRenderer || !_eventEmitter) {
    return;
  }
  bool accepted = false;
  uint8_t message[4096] = {};
  int64_t length = motolii_macos_renderer_take_host_terminal(
      _timelineRenderer, &accepted, message, sizeof(message));
  if (length < 0) {
    return;
  }
  auto emitter = std::static_pointer_cast<const MotoliiTimelineViewEventEmitter>(_eventEmitter);
  MotoliiTimelineViewEventEmitter::OnHostTerminal event = {
      .accepted = accepted,
      .message = std::string((const char *)message, (size_t)length),
  };
  emitter->onHostTerminal(event);
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

- (BOOL)acceptsFirstResponder
{
  return YES;
}

- (void)keyDown:(NSEvent *)event
{
  int32_t result = MotoliiDispatchKeymap(event, YES);
  if (result == 2) {
    if (_timelineRenderer) {
      (void)motolii_macos_timeline_renderer_keymap_delete(_timelineRenderer);
    }
    return;
  }
  if (result == 0) {
    [super keyDown:event];
  }
}

- (void)viewDidMoveToWindow
{
  [super viewDidMoveToWindow];
  if (self.window) {
    MotoliiInstallProductKeymapMonitor();
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
  (void)MotoliiEnsureProductHost();
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
  // host snapshot apply を live CAMetalLayer で走らせる。timer 初回を待たない。
  motolii_macos_renderer_render(_timelineRenderer);
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
  [_timelineView cancelActiveGesture];
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
- (void)emitHostTerminalIfPending;
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
    _metalView.stagePointerHandler =
        ^(uint32_t phase, uint32_t button, uint32_t modifiers, CGFloat x, CGFloat y) {
      MotoliiGpuComponentView *strongSelf = weakSelf;
      if (!strongSelf || !strongSelf->_renderer) return;
      CAMetalLayer *layer = (CAMetalLayer *)strongSelf->_metalView.layer;
      CGFloat scale = layer.contentsScale ?: 1.0;
      motolii_macos_stage_renderer_pointer(
          strongSelf->_renderer, phase, button, modifiers, x * scale, y * scale);
    };
    _metalView.stageScrollHandler =
        ^(CGFloat deltaX, CGFloat deltaY, CGFloat magnification, uint32_t modifiers, CGFloat x,
          CGFloat y) {
          MotoliiGpuComponentView *strongSelf = weakSelf;
          if (!strongSelf || !strongSelf->_renderer) {
            return;
          }
          CAMetalLayer *layer = (CAMetalLayer *)strongSelf->_metalView.layer;
          CGFloat scale = layer.contentsScale ?: 1.0;
          motolii_macos_stage_renderer_scroll(
              strongSelf->_renderer, deltaX * scale, deltaY * scale, magnification,
              modifiers, x * scale, y * scale);
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

- (void)emitHostTerminalIfPending
{
  if (!_renderer || !_eventEmitter) {
    return;
  }
  bool accepted = false;
  uint8_t message[4096] = {};
  int64_t length = motolii_macos_renderer_take_host_terminal(
      _renderer, &accepted, message, sizeof(message));
  if (length < 0) {
    return;
  }
  auto emitter = std::static_pointer_cast<const MotoliiGpuViewEventEmitter>(_eventEmitter);
  MotoliiGpuViewEventEmitter::OnHostTerminal event = {
      .accepted = accepted,
      .message = std::string((const char *)message, (size_t)length),
  };
  emitter->onHostTerminal(event);
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
    // 空 id は fixture rectangle@0.5|pucker-bloat にしない。host Document が正本。
    if (!_createdItemId.empty() && _createdItemId != _lastSentCreatedItemId) {
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

- (BOOL)acceptsFirstResponder
{
  return YES;
}

- (void)keyDown:(NSEvent *)event
{
  if (MotoliiDispatchKeymap(event, NO) == 0) {
    [super keyDown:event];
  }
}

- (void)viewDidMoveToWindow
{
  [super viewDidMoveToWindow];
  if (self.window) {
    MotoliiInstallProductKeymapMonitor();
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
  (void)MotoliiEnsureProductHost();
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
  // product mount では fixture createdItemId を new() 後に再注入しない。
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
  // apply_host + eval frame を live CAMetalLayer で即時実行。offscreen warmup / timer に頼らない。
  motolii_macos_renderer_render(_renderer);

  __weak MotoliiGpuComponentView *weakSelf = self;
  _frameTimer = [NSTimer scheduledTimerWithTimeInterval:(1.0 / 60.0)
                                                repeats:YES
                                                  block:^(__unused NSTimer *timer) {
    MotoliiGpuComponentView *strongSelf = weakSelf;
    if (strongSelf && strongSelf->_renderer) {
      motolii_macos_renderer_render(strongSelf->_renderer);
      [strongSelf emitHostTerminalIfPending];
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
  [_metalView cancelActiveGesture];
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

void MotoliiInstallProductKeymapMonitor(void)
{
  static id monitor;
  static dispatch_once_t once;
  dispatch_once(&once, ^{
    monitor = [NSEvent addLocalMonitorForEventsMatchingMask:NSEventMaskKeyDown
                                                    handler:^NSEvent *(NSEvent *event) {
      NSResponder *fr = event.window.firstResponder;
      if (event.keyCode == 53 && [fr isKindOfClass:[MotoliiMetalView class]] &&
          [(MotoliiMetalView *)fr cancelActiveGesture]) {
        return nil;
      }
      BOOL timelineFocused = [fr isKindOfClass:[MotoliiTimelineMetalView class]] ||
          [fr isKindOfClass:[MotoliiTimelineComponentView class]];
      int32_t result = MotoliiDispatchKeymap(event, timelineFocused);
      // 2 は timeline 既存 delete。view keyDown に渡す。ここでは消費しない。
      if (result == 2) {
        return event;
      }
      if (result != 0) {
        return nil;
      }
      return event;
    }];
  });
}
