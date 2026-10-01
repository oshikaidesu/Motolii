import Cocoa
import FlutterMacOS

@main
class AppDelegate: FlutterAppDelegate {
  private var terminationPending = false
  override func applicationShouldTerminateAfterLastWindowClosed(_ sender: NSApplication) -> Bool { true }
  override func applicationSupportsSecureRestorableState(_ app: NSApplication) -> Bool { true }
  override func applicationShouldTerminate(_ sender: NSApplication) -> NSApplication.TerminateReply {
    if ProbeSession.shared.terminationApproved { return .terminateNow }
    if terminationPending { return .terminateLater }
    terminationPending = true
    ProbeSession.shared.confirmTermination { approved in
      if !approved {
        self.terminationPending = false
        sender.reply(toApplicationShouldTerminate: false)
        return
      }
      ProbeSession.shared.shutdown {
        self.terminationPending = false
        sender.reply(toApplicationShouldTerminate: true)
      }
    }
    return .terminateLater
  }
}
