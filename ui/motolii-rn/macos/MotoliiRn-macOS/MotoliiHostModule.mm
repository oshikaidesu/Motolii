#import "MotoliiHostModule.h"

#include <cstdint>
#include <cstring>
#include <vector>

extern "C" bool motolii_rnapp_host_ensure(const uint8_t *path_utf8, size_t path_len);
extern "C" int64_t motolii_rnapp_host_dispatch_json(
    const uint8_t *in_utf8, size_t in_len, uint8_t *out, size_t out_cap);
extern "C" int64_t motolii_rnapp_host_snapshot_json(uint8_t *out, size_t out_cap);
extern "C" bool motolii_rnapp_is_timeline_interacting(void);
extern "C" int32_t motolii_rnapp_host_key_event(
    uint16_t keyCode, uint32_t modifierBits, const uint8_t *charsUtf8, size_t charsLen,
    bool isRepeat, bool timelineFocused);
extern "C" void *motolii_macos_active_stage_renderer(void);
extern "C" bool motolii_macos_stage_renderer_fit_view(void *handle);
extern "C" bool motolii_macos_stage_renderer_one_to_one(void *handle);
extern "C" int64_t motolii_macos_active_stage_preview_transform(
    const uint8_t *target_utf8, size_t target_len, const uint8_t *revision_utf8,
    size_t revision_len, int32_t kind, double a, double b, uint8_t *out, size_t out_cap);
extern "C" int64_t motolii_macos_active_stage_commit_transform(
    const uint8_t *target_utf8, size_t target_len, const uint8_t *revision_utf8,
    size_t revision_len, int32_t kind, double a, double b, uint8_t *out, size_t out_cap);
extern "C" int64_t motolii_macos_active_stage_cancel_transform(uint8_t *out, size_t out_cap);
extern "C" bool motolii_rnapp_commit_stage_transform(
    const uint8_t *target_utf8, size_t target_len, const uint8_t *revision_utf8,
    size_t revision_len, int32_t kind, double a, double b);

static NSString *MotoliiStageTransformResponse(int64_t written, const uint8_t *out)
{
  if (written == 0) {
    return @"{\"accepted\":true}";
  }
  NSString *message = written > 0
      ? [[NSString alloc] initWithBytes:out
                                length:(NSUInteger)written
                              encoding:NSUTF8StringEncoding]
      : nil;
  NSDictionary *payload = @{
    @"accepted": @NO,
    @"message": message ?: @"Stage transform rejected",
  };
  NSData *data = [NSJSONSerialization dataWithJSONObject:payload options:0 error:nil];
  if (data == nil) {
    return @"{\"accepted\":false,\"message\":\"Stage transform rejected\"}";
  }
  return [[NSString alloc] initWithData:data encoding:NSUTF8StringEncoding]
      ?: @"{\"accepted\":false,\"message\":\"Stage transform rejected\"}";
}

/// AppDelegate と同じ live-project。slot 既存なら true。失敗しても slot は消さない。
extern "C" bool MotoliiEnsureProductHost(void)
{
  NSArray<NSURL *> *supports = [[NSFileManager defaultManager]
      URLsForDirectory:NSApplicationSupportDirectory
             inDomains:NSUserDomainMask];
  NSURL *supportRoot = supports.firstObject;
  if (supportRoot == nil) {
    return false;
  }
  NSURL *motoliiDir = [supportRoot URLByAppendingPathComponent:@"MotoliiRn" isDirectory:YES];
  [[NSFileManager defaultManager] createDirectoryAtURL:motoliiDir
                           withIntermediateDirectories:YES
                                            attributes:nil
                                                 error:nil];
  NSURL *projectFile = [motoliiDir URLByAppendingPathComponent:@"live-project"];
  NSString *path = projectFile.path;
  const char *utf8 = path.UTF8String;
  if (utf8 == NULL) {
    return false;
  }
  return motolii_rnapp_host_ensure(reinterpret_cast<const uint8_t *>(utf8), strlen(utf8));
}

@implementation MotoliiHostModule

RCT_EXPORT_MODULE(NativeMotoliiHost)

+ (BOOL)requiresMainQueueSetup
{
  return NO;
}

RCT_EXPORT_BLOCKING_SYNCHRONOUS_METHOD(dispatchIntent:(NSString *)intentJson)
{
  (void)MotoliiEnsureProductHost();
  if (intentJson == nil) {
    return @"{\"accepted\":false}";
  }
  const char *utf8 = intentJson.UTF8String;
  if (utf8 == NULL) {
    return @"{\"accepted\":false}";
  }
  size_t len = strlen(utf8);
  std::vector<uint8_t> out(131072);
  int64_t written = motolii_rnapp_host_dispatch_json(
      reinterpret_cast<const uint8_t *>(utf8), len, out.data(), out.size());
  NSLog(@"[MotoliiHost] dispatchIntent in=%zu out=%lld", len, (long long)written);
  if (written <= 0) {
    return @"{\"accepted\":false}";
  }
  return [[NSString alloc] initWithBytes:out.data()
                                  length:(NSUInteger)written
                                encoding:NSUTF8StringEncoding]
      ?: @"{\"accepted\":false}";
}

RCT_EXPORT_BLOCKING_SYNCHRONOUS_METHOD(isTimelineInteracting)
{
  return @(motolii_rnapp_is_timeline_interacting());
}

RCT_EXPORT_BLOCKING_SYNCHRONOUS_METHOD(fitStageView)
{
  void *handle = motolii_macos_active_stage_renderer();
  if (handle == nullptr) {
    return @NO;
  }
  return @(motolii_macos_stage_renderer_fit_view(handle));
}

RCT_EXPORT_BLOCKING_SYNCHRONOUS_METHOD(stageViewOneToOne)
{
  void *handle = motolii_macos_active_stage_renderer();
  if (handle == nullptr) {
    return @NO;
  }
  return @(motolii_macos_stage_renderer_one_to_one(handle));
}

RCT_EXPORT_BLOCKING_SYNCHRONOUS_METHOD(previewStageTransform:(NSString *)target
                                                    revision:(NSString *)revision
                                                        kind:(nonnull NSNumber *)kind
                                                           a:(nonnull NSNumber *)a
                                                           b:(nonnull NSNumber *)b)
{
  (void)MotoliiEnsureProductHost();
  const char *targetUtf8 = target.UTF8String;
  const char *revisionUtf8 = revision.UTF8String;
  if (targetUtf8 == NULL || revisionUtf8 == NULL) {
    return @"{\"accepted\":false,\"message\":\"The Stage transform request is invalid\"}";
  }
  uint8_t out[1024] = {};
  uint8_t *outPtr = out;
  __block int64_t written = -1;
  dispatch_block_t invoke = ^{
    written = motolii_macos_active_stage_preview_transform(
        reinterpret_cast<const uint8_t *>(targetUtf8), strlen(targetUtf8),
        reinterpret_cast<const uint8_t *>(revisionUtf8), strlen(revisionUtf8),
        kind.intValue, a.doubleValue, b.doubleValue, outPtr, sizeof(out));
  };
  if ([NSThread isMainThread]) {
    invoke();
  } else {
    dispatch_sync(dispatch_get_main_queue(), invoke);
  }
  return MotoliiStageTransformResponse(written, out);
}

RCT_EXPORT_BLOCKING_SYNCHRONOUS_METHOD(commitStageTransformGesture:(NSString *)target
                                                           revision:(NSString *)revision
                                                               kind:(nonnull NSNumber *)kind
                                                                  a:(nonnull NSNumber *)a
                                                                  b:(nonnull NSNumber *)b)
{
  (void)MotoliiEnsureProductHost();
  const char *targetUtf8 = target.UTF8String;
  const char *revisionUtf8 = revision.UTF8String;
  if (targetUtf8 == NULL || revisionUtf8 == NULL) {
    return @"{\"accepted\":false,\"message\":\"The Stage transform request is invalid\"}";
  }
  uint8_t out[1024] = {};
  uint8_t *outPtr = out;
  __block int64_t written = -1;
  dispatch_block_t invoke = ^{
    written = motolii_macos_active_stage_commit_transform(
        reinterpret_cast<const uint8_t *>(targetUtf8), strlen(targetUtf8),
        reinterpret_cast<const uint8_t *>(revisionUtf8), strlen(revisionUtf8),
        kind.intValue, a.doubleValue, b.doubleValue, outPtr, sizeof(out));
  };
  if ([NSThread isMainThread]) {
    invoke();
  } else {
    dispatch_sync(dispatch_get_main_queue(), invoke);
  }
  return MotoliiStageTransformResponse(written, out);
}

RCT_EXPORT_BLOCKING_SYNCHRONOUS_METHOD(cancelStageTransform)
{
  uint8_t out[1024] = {};
  uint8_t *outPtr = out;
  __block int64_t written = -1;
  dispatch_block_t invoke = ^{
    written = motolii_macos_active_stage_cancel_transform(outPtr, sizeof(out));
  };
  if ([NSThread isMainThread]) {
    invoke();
  } else {
    dispatch_sync(dispatch_get_main_queue(), invoke);
  }
  return MotoliiStageTransformResponse(written, out);
}

RCT_EXPORT_BLOCKING_SYNCHRONOUS_METHOD(commitStageTransform:(NSString *)target
                                                  revision:(NSString *)revision
                                                      kind:(nonnull NSNumber *)kind
                                                         a:(nonnull NSNumber *)a
                                                         b:(nonnull NSNumber *)b)
{
  (void)MotoliiEnsureProductHost();
  if (target == nil || revision == nil) {
    return @NO;
  }
  const char *targetUtf8 = target.UTF8String;
  const char *revisionUtf8 = revision.UTF8String;
  if (targetUtf8 == NULL || revisionUtf8 == NULL) {
    return @NO;
  }
  return @(motolii_rnapp_commit_stage_transform(
      reinterpret_cast<const uint8_t *>(targetUtf8), strlen(targetUtf8),
      reinterpret_cast<const uint8_t *>(revisionUtf8), strlen(revisionUtf8),
      kind.intValue, a.doubleValue, b.doubleValue));
}

RCT_EXPORT_BLOCKING_SYNCHRONOUS_METHOD(hostKeyEvent:(nonnull NSNumber *)keyCode
                                      modifierBits:(nonnull NSNumber *)modifierBits
                                             chars:(NSString *)chars
                                          isRepeat:(BOOL)isRepeat
                                  timelineFocused:(BOOL)timelineFocused)
{
  (void)MotoliiEnsureProductHost();
  NSString *safeChars = chars ?: @"";
  const char *utf8 = safeChars.UTF8String ?: "";
  int32_t result = motolii_rnapp_host_key_event(
      keyCode.unsignedShortValue, modifierBits.unsignedIntValue,
      reinterpret_cast<const uint8_t *>(utf8), strlen(utf8), isRepeat, timelineFocused);
  return @(result);
}

RCT_EXPORT_BLOCKING_SYNCHRONOUS_METHOD(readSnapshot)
{
  (void)MotoliiEnsureProductHost();
  std::vector<uint8_t> out(131072);
  int64_t written = motolii_rnapp_host_snapshot_json(out.data(), out.size());
  static dispatch_once_t once;
  dispatch_once(&once, ^{
    NSLog(@"[MotoliiHost] first readSnapshot out=%lld", (long long)written);
  });
  if (written <= 0) {
    return @"";
  }
  return [[NSString alloc] initWithBytes:out.data()
                                  length:(NSUInteger)written
                                encoding:NSUTF8StringEncoding]
      ?: @"";
}

@end
