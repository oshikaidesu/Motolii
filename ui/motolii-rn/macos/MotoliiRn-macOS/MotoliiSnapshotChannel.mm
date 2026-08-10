#import "MotoliiSnapshotChannel.h"

#import <React/RCTInvalidating.h>

#import <memory>

namespace {

const NSUInteger kMaximumSnapshotJSONLength = 16 * 1024;
__weak MotoliiSnapshotChannel *sActiveChannel = nil;
uint64_t sBoundHostHandle = 0;

} // namespace

@interface MotoliiSnapshotChannel () <NativeMotoliiSnapshotChannelSpec,
                                        RCTInvalidating>
// 任意スレッドからの即時 fail-closed 用。sActiveChannel の所有は main のみ。
@property(atomic, assign) BOOL invalidated;
@end

@implementation MotoliiSnapshotChannel
RCT_EXPORT_MODULE()

+ (BOOL)requiresMainQueueSetup
{
  return YES;
}

+ (void)bindHostHandle:(uint64_t)hostHandle
{
  dispatch_assert_queue(dispatch_get_main_queue());
  sBoundHostHandle = hostHandle;
}

+ (void)unbindHostHandle:(uint64_t)hostHandle
{
  dispatch_assert_queue(dispatch_get_main_queue());
  if (sBoundHostHandle == hostHandle) {
    sBoundHostHandle = 0;
    sActiveChannel = nil;
  }
}

+ (void)publishSnapshotJSON:(NSString *)snapshotJSON
              forHostHandle:(uint64_t)hostHandle
{
  NSString *snapshotCopy = [snapshotJSON copy];
  dispatch_async(dispatch_get_main_queue(), ^{
    MotoliiSnapshotChannel *channel = sActiveChannel;
    if (channel == nil || channel.invalidated || hostHandle == 0 ||
        hostHandle != sBoundHostHandle || snapshotCopy.length == 0 ||
        snapshotCopy.length > kMaximumSnapshotJSONLength) {
      return;
    }
    [channel emitOnSnapshotChanged:snapshotCopy];
  });
}

- (instancetype)init
{
  self = [super init];
  if (self != nil) {
    _invalidated = NO;
    // JS スレッド構築でも assert/sync せず、main 上で唯一の active 登録を行う。
    __weak MotoliiSnapshotChannel *weakSelf = self;
    dispatch_async(dispatch_get_main_queue(), ^{
      MotoliiSnapshotChannel *channel = weakSelf;
      if (channel == nil || channel.invalidated) {
        return;
      }
      sActiveChannel = channel;
    });
  }
  return self;
}

- (void)invalidate
{
  // 呼び出し元キューを問わず即 fail-closed。global クリアだけ main へ async。
  self.invalidated = YES;
  MotoliiSnapshotChannel *channel = self;
  dispatch_async(dispatch_get_main_queue(), ^{
    if (sActiveChannel == channel) {
      sActiveChannel = nil;
    }
  });
}

- (std::shared_ptr<facebook::react::TurboModule>)getTurboModule:
    (const facebook::react::ObjCTurboModule::InitParams &)params
{
  return std::make_shared<facebook::react::NativeMotoliiSnapshotChannelSpecJSI>(
      params);
}

@end
