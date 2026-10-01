import Cocoa
import FlutterMacOS

class MainFlutterWindow: NSWindow {
  override func awakeFromNib() {
    let flutterViewController = FlutterViewController()
    self.contentViewController = flutterViewController
    // the lab's window opens large: a window is judged at its real size
    let screen = NSScreen.main?.visibleFrame ?? NSRect(x: 0, y: 0, width: 1512, height: 900)
    self.setFrame(NSRect(x: screen.minX + 20, y: screen.minY + 20, width: min(1480, screen.width - 40), height: min(940, screen.height - 40)), display: true)

    RegisterGeneratedPlugins(registry: flutterViewController)

    super.awakeFromNib()
    // after the xib has restored its own frame
    self.setFrame(NSRect(x: screen.minX + 20, y: screen.minY + 20, width: min(1480, screen.width - 40), height: min(940, screen.height - 40)), display: true)
  }
}
