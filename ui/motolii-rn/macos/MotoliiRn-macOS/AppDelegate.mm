#import "AppDelegate.h"

#import <React/RCTBundleURLProvider.h>
#import <ReactAppDependencyProvider/RCTAppDependencyProvider.h>
#import "MotoliiGpuComponentView.h"

#include <cstdint>
#include <cstring>

extern "C" bool motolii_rnapp_host_ensure(const uint8_t *path_utf8, size_t path_len);

@implementation AppDelegate

- (NSDictionary<NSString *, Class<RCTComponentViewProtocol>> *)thirdPartyFabricComponents
{
  NSMutableDictionary *components = [[super thirdPartyFabricComponents] mutableCopy];
  components[@"MotoliiGpuView"] = MotoliiGpuComponentView.class;
  components[@"MotoliiTimelineView"] = MotoliiTimelineComponentView.class;
  return components;
}

- (void)applicationDidFinishLaunching:(NSNotification *)notification
{
  MotoliiInstallProductKeymapMonitor();
  self.moduleName = @"MotoliiRn";
  // You can add your custom initial props in the dictionary below.
  // They will be passed down to the ViewController used by React Native.
  self.initialProps = @{};
  self.dependencyProvider = [RCTAppDependencyProvider new];

  NSArray<NSURL *> *supports = [[NSFileManager defaultManager]
      URLsForDirectory:NSApplicationSupportDirectory
             inDomains:NSUserDomainMask];
  NSURL *supportRoot = supports.firstObject;
  if (supportRoot != nil) {
    NSURL *motoliiDir = [supportRoot URLByAppendingPathComponent:@"MotoliiRn" isDirectory:YES];
    [[NSFileManager defaultManager] createDirectoryAtURL:motoliiDir
                             withIntermediateDirectories:YES
                                              attributes:nil
                                                   error:nil];
    // ProjectSession::open は document file identity を取る(dirではない)。
    NSURL *projectFile = [motoliiDir URLByAppendingPathComponent:@"live-project"];
    NSString *path = projectFile.path;
    const char *utf8 = path.UTF8String;
    if (utf8 != NULL) {
      (void)motolii_rnapp_host_ensure(reinterpret_cast<const uint8_t *>(utf8), strlen(utf8));
    }
  }

  return [super applicationDidFinishLaunching:notification];
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

/// This method controls whether the `concurrentRoot`feature of React18 is turned on or off.
///
/// @see: https://reactjs.org/blog/2022/03/29/react-v18.html
/// @note: This requires to be rendering on Fabric (i.e. on the New Architecture).
/// @return: `true` if the `concurrentRoot` feature is enabled. Otherwise, it returns `false`.
- (BOOL)concurrentRootEnabled
{
#ifdef RN_FABRIC_ENABLED
  return true;
#else
  return false;
#endif
}

@end
