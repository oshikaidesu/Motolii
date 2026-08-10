#import "AppDelegate.h"

#import "MotoliiHostBridge.h"
#import "MotoliiStageComponentView.h"
#import "MotoliiTimelineComponentView.h"

#import <React/RCTBundleURLProvider.h>
#import <React/RCTBridgeModule.h>
#import <ReactAppDependencyProvider/RCTAppDependencyProvider.h>

@interface AppDelegate ()
@property(nonatomic, assign) uint64_t motoliiHostHandle;
@property(nonatomic, assign) BOOL motoliiHostCreationAttempted;
@property(nonatomic, copy) NSString *motoliiProjectPath;
@property(nonatomic, copy) NSString *motoliiDiagnostic;
@property(nonatomic, copy) NSString *motoliiSnapshotJSON;
@end

namespace {

const size_t kHostBridgeBufferCapacity = 16 * 1024;

NSString *ProjectPathFromProcess(void)
{
  NSArray<NSString *> *arguments = [NSProcessInfo processInfo].arguments;
  for (NSUInteger index = 1; index + 1 < arguments.count; index += 1) {
    if ([arguments[index] isEqualToString:@"--motolii-project"]) {
      return [arguments[index + 1] stringByExpandingTildeInPath];
    }
  }

  NSString *environmentPath = [NSProcessInfo processInfo].environment[@"MOTOLII_PROJECT_PATH"];
  return [environmentPath stringByExpandingTildeInPath];
}

NSString *TypedReasonFromResponse(NSData *data)
{
  if (data == nil) {
    return nil;
  }
  NSDictionary *response = [NSJSONSerialization JSONObjectWithData:data options:0 error:nil];
  if (![response isKindOfClass:[NSDictionary class]]) {
    return nil;
  }
  NSArray *diagnostics = response[@"diagnostics"];
  NSDictionary *diagnostic =
      [diagnostics isKindOfClass:[NSArray class]] ? diagnostics.firstObject : nil;
  NSString *reason =
      [diagnostic isKindOfClass:[NSDictionary class]] ? diagnostic[@"reason"] : nil;
  return [reason isKindOfClass:[NSString class]] && reason.length > 0 ? reason : nil;
}

NSString *SnapshotJSONFromCreateResponse(NSData *data)
{
  if (data == nil) {
    return nil;
  }
  NSDictionary *response = [NSJSONSerialization JSONObjectWithData:data options:0 error:nil];
  if (![response isKindOfClass:[NSDictionary class]]) {
    return nil;
  }
  if (![response[@"accepted"] boolValue]) {
    return nil;
  }
  NSDictionary *snapshot = response[@"snapshot"];
  if (![snapshot isKindOfClass:[NSDictionary class]]) {
    return nil;
  }
  NSData *snapshotData = [NSJSONSerialization dataWithJSONObject:snapshot options:0 error:nil];
  if (snapshotData == nil) {
    return nil;
  }
  return [[NSString alloc] initWithData:snapshotData encoding:NSUTF8StringEncoding];
}

} // namespace

@interface MotoliiHostBridge : NSObject <RCTBridgeModule>
@end

@implementation MotoliiHostBridge

RCT_EXPORT_MODULE()

RCT_REMAP_METHOD(dispatchIntent,
                 dispatchIntentWithHostHandle:(NSString *)hostHandle
                 intentJSON:(NSString *)intentJSON
                 resolver:(RCTPromiseResolveBlock)resolve
                 rejecter:(RCTPromiseRejectBlock)reject)
{
  NSScanner *scanner = [NSScanner scannerWithString:hostHandle ?: @""];
  unsigned long long parsedHostHandle = 0;
  if (![scanner scanUnsignedLongLong:&parsedHostHandle] || !scanner.isAtEnd ||
      parsedHostHandle == 0) {
    reject(@"invalid_host_handle", @"host handle must be a positive integer", nil);
    return;
  }

  NSData *intentData = [intentJSON dataUsingEncoding:NSUTF8StringEncoding];
  if (intentData == nil) {
    reject(@"invalid_intent", @"intent must be valid UTF-8", nil);
    return;
  }

  uint8_t output[kHostBridgeBufferCapacity];
  int64_t result = motolii_rn_host_dispatch_intent_json(
      (uint64_t)parsedHostHandle,
      (const uint8_t *)intentData.bytes,
      intentData.length,
      output,
      sizeof(output));
  if (result <= 0 || (uint64_t)result > sizeof(output)) {
    reject(@"host_dispatch_failed", @"host intent dispatch failed", nil);
    return;
  }

  NSString *responseJSON =
      [[NSString alloc] initWithBytes:output
                              length:(NSUInteger)result
                            encoding:NSUTF8StringEncoding];
  if (responseJSON == nil) {
    reject(@"invalid_host_response", @"host response was not valid UTF-8", nil);
    return;
  }
  resolve(responseJSON);
}

@end

@implementation AppDelegate

- (NSDictionary<NSString *, Class<RCTComponentViewProtocol>> *)thirdPartyFabricComponents
{
  NSMutableDictionary *components = [[super thirdPartyFabricComponents] mutableCopy];
  components[@"MotoliiStageView"] = MotoliiStageComponentView.class;
  components[@"MotoliiTimelineView"] = MotoliiTimelineComponentView.class;
  return components;
}

- (void)createMotoliiHostOnce
{
  if (self.motoliiHostCreationAttempted) {
    return;
  }
  self.motoliiHostCreationAttempted = YES;

  NSString *projectPath = ProjectPathFromProcess();
  if (projectPath.length == 0) {
    self.motoliiDiagnostic = @"project path missing; pass --motolii-project <path>";
    return;
  }

  BOOL isDirectory = NO;
  BOOL exists =
      [[NSFileManager defaultManager] fileExistsAtPath:projectPath isDirectory:&isDirectory];
  self.motoliiProjectPath = projectPath;
  if (!exists) {
    self.motoliiDiagnostic = @"project path does not exist";
    return;
  }
  if (isDirectory) {
    self.motoliiDiagnostic =
        @"project path is a directory; expected an existing project file path";
    return;
  }

  NSData *pathData = [projectPath dataUsingEncoding:NSUTF8StringEncoding];
  uint64_t hostHandle = 0;
  uint8_t output[kHostBridgeBufferCapacity];
  int64_t result = motolii_rn_host_create(
      (const uint8_t *)pathData.bytes,
      pathData.length,
      &hostHandle,
      output,
      sizeof(output));
  self.motoliiProjectPath = projectPath;
  if (result <= 0 || (uint64_t)result > sizeof(output)) {
    self.motoliiDiagnostic = @"host creation bridge failure";
    return;
  }

  NSData *responseData = [NSData dataWithBytes:output length:(NSUInteger)result];
  NSDictionary *response =
      [NSJSONSerialization JSONObjectWithData:responseData options:0 error:nil];
  if (![response isKindOfClass:[NSDictionary class]] || ![response[@"accepted"] boolValue] ||
      hostHandle == 0) {
    NSString *reason = TypedReasonFromResponse(responseData);
    self.motoliiDiagnostic = reason.length > 0
                                 ? [NSString stringWithFormat:@"host creation rejected: %@", reason]
                                 : @"host creation rejected";
    return;
  }

  NSString *snapshotJSON = SnapshotJSONFromCreateResponse(responseData);
  if (snapshotJSON.length == 0) {
    self.motoliiDiagnostic = @"host creation accepted without snapshot";
    uint8_t destroyOutput[kHostBridgeBufferCapacity];
    motolii_rn_host_destroy(hostHandle, destroyOutput, sizeof(destroyOutput));
    return;
  }

  self.motoliiHostHandle = hostHandle;
  self.motoliiSnapshotJSON = snapshotJSON;
}

- (void)applicationDidFinishLaunching:(NSNotification *)notification
{
  [self createMotoliiHostOnce];
  self.moduleName = @"MotoliiRn";
  self.initialProps = @{
    @"hostHandle" : [NSString stringWithFormat:@"%llu",
                                                (unsigned long long)self.motoliiHostHandle],
    @"projectPath" : self.motoliiProjectPath ?: @"",
    @"snapshotJSON" : self.motoliiSnapshotJSON ?: @"",
    @"diagnostic" : self.motoliiDiagnostic ?: @"",
  };
  self.dependencyProvider = [RCTAppDependencyProvider new];

  [super applicationDidFinishLaunching:notification];
}

- (void)applicationWillTerminate:(NSNotification *)notification
{
  if (self.motoliiHostHandle != 0) {
    uint8_t output[kHostBridgeBufferCapacity];
    motolii_rn_host_destroy(self.motoliiHostHandle, output, sizeof(output));
    self.motoliiHostHandle = 0;
  }
  [super applicationWillTerminate:notification];
}

- (NSURL *)sourceURLForBridge:(RCTBridge *)bridge
{
  return [self bundleURL];
}

- (NSURL *)bundleURL
{
#if DEBUG
  return [[RCTBundleURLProvider sharedSettings] jsBundleURLForBundleRoot:@"index"];
#else
  return [[NSBundle mainBundle] URLForResource:@"main" withExtension:@"jsbundle"];
#endif
}

- (BOOL)concurrentRootEnabled
{
#ifdef RN_FABRIC_ENABLED
  return YES;
#else
  return NO;
#endif
}

@end
