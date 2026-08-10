#import <Foundation/Foundation.h>
#import <MotoliiRnSpec/MotoliiRnSpec.h>

NS_ASSUME_NONNULL_BEGIN

@interface MotoliiSnapshotChannel : NativeMotoliiSnapshotChannelSpecBase

+ (void)bindHostHandle:(uint64_t)hostHandle;
+ (void)unbindHostHandle:(uint64_t)hostHandle;
+ (void)publishSnapshotJSON:(NSString *)snapshotJSON
              forHostHandle:(uint64_t)hostHandle;

@end

NS_ASSUME_NONNULL_END
