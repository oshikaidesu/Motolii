#import "MotoliiGpuComponentView.h"

#import <QuartzCore/CAMetalLayer.h>
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
extern "C" bool motolii_macos_renderer_get_stats(void *handle, MotoliiRenderStats *stats);

@interface MotoliiMetalView : NSView
@end

@interface MotoliiTimelineMetalView : MotoliiMetalView
@property(nonatomic, copy) void (^timelineClickHandler)(CGFloat x, CGFloat y);
@end

@interface MotoliiStageMetalView : MotoliiMetalView
@property(nonatomic, copy) void (^stagePointerHandler)(uint32_t phase, CGFloat x, CGFloat y);
@end

@implementation MotoliiStageMetalView
- (BOOL)acceptsFirstResponder { return YES; }
- (void)emitPhase:(uint32_t)phase event:(NSEvent *)event
{
  NSPoint point = [self convertPoint:event.locationInWindow fromView:nil];
  if (self.stagePointerHandler) self.stagePointerHandler(phase, point.x, NSHeight(self.bounds) - point.y);
}
- (void)mouseDown:(NSEvent *)event
{
  [self.window makeFirstResponder:self];
  [self emitPhase:0 event:event];
}
- (void)mouseDragged:(NSEvent *)event { [self emitPhase:1 event:event]; }
- (void)mouseUp:(NSEvent *)event { [self emitPhase:2 event:event]; }
@end

@implementation MotoliiTimelineMetalView

- (BOOL)acceptsFirstResponder
{
  return YES;
}

- (void)mouseDown:(NSEvent *)event
{
  NSPoint point = [self convertPoint:event.locationInWindow fromView:nil];
  if (self.timelineClickHandler) {
    self.timelineClickHandler(point.x, NSHeight(self.bounds) - point.y);
  }
}

- (void)mouseDragged:(NSEvent *)event
{
  [self mouseDown:event];
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
    _timelineView.timelineClickHandler = ^(CGFloat x, CGFloat y) {
      MotoliiTimelineComponentView *strongSelf = weakSelf;
      if (!strongSelf || !strongSelf->_timelineRenderer) {
        return;
      }
      CAMetalLayer *layer = (CAMetalLayer *)strongSelf->_timelineView.layer;
      CGFloat scale = layer.contentsScale ?: 1.0;
      MotoliiTimelineFeedback feedback = {};
      if (!motolii_macos_timeline_renderer_hit_test(
              strongSelf->_timelineRenderer, x * scale, y * scale, &feedback)) {
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
      CAMetalLayer *layer = (CAMetalLayer *)strongSelf->_metalView.layer;
      CGFloat scale = layer.contentsScale ?: 1.0;
      motolii_macos_stage_renderer_pointer(strongSelf->_renderer, phase, x * scale, y * scale);
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
  if (NSPointInRect(point, _metalView.bounds) && NSWidth(_metalView.bounds) > 0 && NSHeight(_metalView.bounds) > 0) {
    x = point.x / NSWidth(_metalView.bounds);
    y = 1.0 - point.y / NSHeight(_metalView.bounds);
  }
  auto emitter = std::static_pointer_cast<const MotoliiGpuViewEventEmitter>(_eventEmitter);
  MotoliiGpuViewEventEmitter::OnStageDrop drop = {.x = x, .y = y};
  emitter->onStageDrop(drop);
}

- (void)updateProps:(const Props::Shared &)props oldProps:(const Props::Shared &)oldProps
{
  const auto &newProps = static_cast<const MotoliiGpuViewProps &>(*props);
  _createdItemId = newProps.createdItemId;
  _draggedItemId = newProps.draggedItemId;
  if (_renderer) {
    motolii_macos_stage_renderer_set_created_item(_renderer, _createdItemId.c_str());
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

  __weak MotoliiGpuComponentView *weakSelf = self;
  _frameTimer = [NSTimer scheduledTimerWithTimeInterval:(1.0 / 60.0)
                                                repeats:YES
                                                  block:^(__unused NSTimer *timer) {
    MotoliiGpuComponentView *strongSelf = weakSelf;
    if (strongSelf && strongSelf->_renderer) {
      motolii_macos_renderer_render(strongSelf->_renderer);
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
    motolii_macos_renderer_destroy(_renderer);
    _renderer = nullptr;
    NSLog(@"[MotoliiRerunStage] renderer unmounted");
  }
}

@end
