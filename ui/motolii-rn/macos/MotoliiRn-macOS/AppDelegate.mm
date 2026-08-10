#import "AppDelegate.h"

#import "MotoliiHostBridge.h"
#import "MotoliiStageComponentView.h"

#import <React/RCTBundleURLProvider.h>
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

NSString *SnapshotJSONForHost(uint64_t hostHandle)
{
  uint8_t output[kHostBridgeBufferCapacity];
  int64_t result =
      motolii_rn_host_read_snapshot_json(hostHandle, output, sizeof(output));
  if (result <= 0 || (uint64_t)result > sizeof(output)) {
    return nil;
  }
  NSData *data = [NSData dataWithBytes:output length:(NSUInteger)result];
  return [[NSString alloc] initWithData:data encoding:NSUTF8StringEncoding];
}

} // namespace

@implementation AppDelegate

- (NSDictionary<NSString *, Class<RCTComponentViewProtocol>> *)thirdPartyFabricComponents
{
  NSMutableDictionary *components = [[super thirdPartyFabricComponents] mutableCopy];
  components[@"MotoliiStageView"] = MotoliiStageComponentView.class;
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
  uint64_t hostHandle =
      motolii_rn_host_create((const uint8_t *)pathData.bytes, pathData.length);
  self.motoliiProjectPath = projectPath;
  if (hostHandle == 0) {
    self.motoliiDiagnostic =
        @"host creation rejected (invalid project or another host is active)";
    return;
  }

  self.motoliiHostHandle = hostHandle;
  self.motoliiSnapshotJSON = SnapshotJSONForHost(hostHandle);
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
    motolii_rn_host_destroy(self.motoliiHostHandle);
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
