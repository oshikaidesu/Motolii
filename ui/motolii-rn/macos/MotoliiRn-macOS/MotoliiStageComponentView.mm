#import "MotoliiStageComponentView.h"

#import "MotoliiHostBridge.h"

#import <AppKit/AppKit.h>
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
    self.wantsLayer = YES;
    self.layer.backgroundColor = NSColor.windowBackgroundColor.CGColor;
  }
  return self;
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
  [self sendResizeIntent];
}

- (void)viewDidChangeBackingProperties
{
  [super viewDidChangeBackingProperties];
  [self sendResizeIntent];
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

- (void)drawRect:(NSRect)dirtyRect
{
  [super drawRect:dirtyRect];

  [[NSColor colorWithWhite:0.08 alpha:1.0] setFill];
  NSRectFill(self.bounds);

  NSDictionary *attributes = @{
    NSFontAttributeName : [NSFont systemFontOfSize:12 weight:NSFontWeightSemibold],
    NSForegroundColorAttributeName : [NSColor colorWithWhite:0.88 alpha:1.0],
  };
  NSString *title = @"Fabric Stage · placeholder";
  [title drawAtPoint:NSMakePoint(18, NSMaxY(self.bounds) - 30) withAttributes:attributes];

  NSDictionary *detailAttributes = @{
    NSFontAttributeName : [NSFont monospacedSystemFontOfSize:10 weight:NSFontWeightRegular],
    NSForegroundColorAttributeName : [NSColor colorWithWhite:0.65 alpha:1.0],
  };
  NSString *detail = [NSString stringWithFormat:@"host %@ · stage %@\n%@",
                                                HandleString(_hostHandle),
                                                HandleString(_stageHandle),
                                                _snapshotText ?: @"snapshot unavailable"];
  [detail drawAtPoint:NSMakePoint(18, NSMaxY(self.bounds) - 54)
       withAttributes:detailAttributes];
}

- (void)activateStageIfNeeded
{
  if (_hostHandle == 0 || _stageHandle != 0 || self.window == nil) {
    return;
  }

  _stageHandle = motolii_rn_stage_register(_hostHandle);
  if (_stageHandle == 0) {
    _snapshotText = @"stage registration rejected";
    [self setNeedsDisplay:YES];
    return;
  }

  _mounted = YES;
  [self sendIntent:@"stage_mount"];
  [self sendResizeIntent];
  [self sendFocusIntent];
  [self readSnapshot];
}

- (void)deactivateStage
{
  if (_stageHandle == 0) {
    return;
  }

  if (_mounted) {
    [self sendIntent:@"stage_unmount"];
  }
  motolii_rn_stage_destroy(_stageHandle);
  _mounted = NO;
  _focused = NO;
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

  NSView *content = self.window.contentView;
  if (content == nil) {
    return;
  }
  NSRect rect = [self convertRect:self.bounds toView:content];
  CGFloat top = content.isFlipped ? NSMinY(rect)
                                  : NSHeight(content.bounds) - NSMaxY(rect);
  motolii_rn_stage_register_layout(
      _hostHandle,
      _stageHandle,
      NSMinX(rect),
      top,
      NSWidth(rect),
      NSHeight(rect));
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
