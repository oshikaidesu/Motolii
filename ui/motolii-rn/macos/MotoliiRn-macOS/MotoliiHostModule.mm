#import "MotoliiHostModule.h"

#include <cstdint>
#include <cstring>
#include <vector>

extern "C" bool motolii_rnapp_host_ensure(const uint8_t *path_utf8, size_t path_len);
extern "C" int64_t motolii_rnapp_host_dispatch_json(
    const uint8_t *in_utf8, size_t in_len, uint8_t *out, size_t out_cap);
extern "C" int64_t motolii_rnapp_host_snapshot_json(uint8_t *out, size_t out_cap);

@implementation MotoliiHostModule

RCT_EXPORT_MODULE(NativeMotoliiHost)

+ (BOOL)requiresMainQueueSetup
{
  return NO;
}

RCT_EXPORT_BLOCKING_SYNCHRONOUS_METHOD(dispatchIntent:(NSString *)intentJson)
{
  if (intentJson == nil) {
    return @"{\"accepted\":false}";
  }
  const char *utf8 = intentJson.UTF8String;
  if (utf8 == NULL) {
    return @"{\"accepted\":false}";
  }
  size_t len = strlen(utf8);
  std::vector<uint8_t> out(16384);
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

RCT_EXPORT_BLOCKING_SYNCHRONOUS_METHOD(readSnapshot)
{
  std::vector<uint8_t> out(16384);
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
