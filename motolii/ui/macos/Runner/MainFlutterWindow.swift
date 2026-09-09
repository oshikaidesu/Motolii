import Cocoa
import FlutterMacOS

class MainFlutterWindow: NSWindow, NSWindowDelegate, NSDraggingDestination {
  private var probeHost: ProbeHost?
  private var closeApproved = false
  private var closePending = false
  override func awakeFromNib() {
    let controller = FlutterViewController()
    let originalFrame = frame
    contentViewController = controller
    setFrame(originalFrame, display: true)
    RegisterGeneratedPlugins(registry: controller)
    probeHost = ProbeHost.install(controller: controller, window: self)
    setContentSize(NSSize(width: 1280, height: 768))
    minSize = NSSize(width: 1000, height: 620)
    delegate = self
    registerForDraggedTypes([.fileURL])
    super.awakeFromNib()
  }
  private func droppedPaths(_ sender: NSDraggingInfo) -> [String] {
    let urls = sender.draggingPasteboard.readObjects(forClasses: [NSURL.self], options: [.urlReadingFileURLsOnly: true]) as? [URL] ?? []
    return urls.filter { $0.isFileURL }.map { $0.path }
  }
  func draggingEntered(_ sender: NSDraggingInfo) -> NSDragOperation { droppedPaths(sender).isEmpty ? [] : .copy }
  func prepareForDragOperation(_ sender: NSDraggingInfo) -> Bool { !droppedPaths(sender).isEmpty }
  func performDragOperation(_ sender: NSDraggingInfo) -> Bool {
    let paths = droppedPaths(sender)
    guard !paths.isEmpty else { return false }
    let point = contentView?.convert(sender.draggingLocation, from: nil) ?? sender.draggingLocation
    let height = contentView?.bounds.height ?? frame.height
    probeHost?.filesDropped(paths, point: [point.x, contentView?.isFlipped == true ? point.y : height - point.y])
    return true
  }
  func windowShouldClose(_ sender: NSWindow) -> Bool {
    if closeApproved || ProbeSession.shared.terminationApproved { return true }
    if closePending { return false }
    closePending = true
    ProbeSession.shared.confirmTermination { [weak self] approved in
      guard let self else { return }
      if !approved { self.closePending = false; return }
      ProbeSession.shared.shutdown { [weak self] in
        guard let self else { return }
        self.closeApproved = true
        self.closePending = false
        self.performClose(nil)
      }
    }
    return false
  }
  func windowWillClose(_ notification: Notification) { probeHost?.detachWindow(); probeHost = nil }
}

import Cocoa
import CoreVideo
import Darwin
import FlutterMacOS
import IOSurface
import UniformTypeIdentifiers

private enum ProbeFailure: Error {
  case message(String)
}

private final class ProbeRuntime {
  typealias Open = @convention(c) (UnsafePointer<CChar>) -> UnsafeMutableRawPointer?
  typealias Request = @convention(c) (UnsafeMutableRawPointer, UnsafePointer<CChar>) -> UnsafePointer<CChar>?
  typealias Render = @convention(c) (UnsafeMutableRawPointer, UInt32) -> Int32
  typealias Close = @convention(c) (UnsafeMutableRawPointer) -> Void
  private var library: UnsafeMutableRawPointer?
  private var context: UnsafeMutableRawPointer?
  private var requestFunction: Request?
  private var renderFunction: Render?
  private var closeFunction: Close?
  private(set) var rendered = 0

  func open(path: String) throws -> [String: Any] {
    let previous = (library: library, context: context, request: requestFunction, render: renderFunction, close: closeFunction, count: rendered)
    let location = ProcessInfo.processInfo.environment["MOTOLII_NATIVE_LIBRARY"] ?? URL(fileURLWithPath: #filePath).deletingLastPathComponent().deletingLastPathComponent().deletingLastPathComponent().deletingLastPathComponent().appendingPathComponent("target/debug/libmotolii_ui.dylib").path
    guard let library = dlopen(location, RTLD_NOW | RTLD_LOCAL) else {
      throw ProbeFailure.message(dlerror().map { String(cString: $0) } ?? "dlopen failed")
    }
    self.library = library
    context = nil
    requestFunction = nil
    renderFunction = nil
    closeFunction = nil
    do {
      func symbol<T>(_ name: String, _: T.Type) throws -> T {
        guard let address = dlsym(library, name) else { throw ProbeFailure.message("Missing symbol: \(name)") }
        return unsafeBitCast(address, to: T.self)
      }
      let start = try symbol("motolii_probe_open", Open.self)
      requestFunction = try symbol("motolii_probe_request", Request.self)
      renderFunction = try symbol("motolii_probe_render", Render.self)
      closeFunction = try symbol("motolii_probe_close", Close.self)
      context = path.withCString { start($0) }
      guard context != nil else { throw ProbeFailure.message("Rust could not open the document") }
      rendered = 0
      let reply = try status()
      if let old = previous.context { previous.close?(old) }
      if let old = previous.library { dlclose(old) }
      return reply
    } catch {
      close()
      self.library = previous.library
      context = previous.context
      requestFunction = previous.request
      renderFunction = previous.render
      closeFunction = previous.close
      rendered = previous.count
      throw error
    }
  }

  func request(_ command: String) throws -> [String: Any] {
    guard let context, let requestFunction else { throw ProbeFailure.message("Open a document first") }
    guard let pointer = command.withCString({ requestFunction(context, $0) }) else {
      throw ProbeFailure.message("Rust request returned no reply")
    }
    // Copy the reply before another FFI call can invalidate its pointer.
    let data = Data(String(cString: pointer).utf8)
    guard let reply = try JSONSerialization.jsonObject(with: data) as? [String: Any] else {
      throw ProbeFailure.message("Rust reply is not a JSON object")
    }
    return reply
  }

  func status() throws -> [String: Any] { try request("{\"op\":\"status\",\"bootstrap\":true}") }

  func render(known: Any? = nil, references: Any? = nil) throws -> (CVPixelBuffer, [String: Any], Int) {
    let state = try request("{\"op\":\"renderInfo\"}")
    guard let width = (state["width"] as? NSNumber)?.intValue,
          let height = (state["height"] as? NSNumber)?.intValue,
          width > 0, height > 0, width <= 16384, height <= 16384 else {
      throw ProbeFailure.message("Document dimensions missing or outside probe allocation limit")
    }
    let properties: [String: Any] = [
      kIOSurfaceWidth as String: width,
      kIOSurfaceHeight as String: height,
      kIOSurfaceBytesPerElement as String: 4,
      kIOSurfacePixelFormat as String: kCVPixelFormatType_32BGRA,
    ]
    guard let surface = IOSurfaceCreate(properties as CFDictionary) else {
      throw ProbeFailure.message("IOSurface allocation failed")
    }
    let attributes = [kCVPixelBufferMetalCompatibilityKey as String: true] as CFDictionary
    var unmanaged: Unmanaged<CVPixelBuffer>?
    let outcome = CVPixelBufferCreateWithIOSurface(kCFAllocatorDefault, surface, attributes, &unmanaged)
    guard outcome == kCVReturnSuccess, let unmanaged else {
      throw ProbeFailure.message("CVPixelBuffer wrapping failed: \(outcome)")
    }
    let buffer = unmanaged.takeRetainedValue()
    guard let context, let renderFunction else { throw ProbeFailure.message("Document is closed") }
    let code = renderFunction(context, IOSurfaceGetID(surface))
    guard code == 0 else { throw ProbeFailure.message("Rust render failed: \(code)") }
    rendered += 1
    var query: [String: Any] = ["op": "status"]
    if let known { query["knownSnapshotId"] = known }
    if let references { query["knownReferenceId"] = references }
    let data = try JSONSerialization.data(withJSONObject: query)
    return (buffer, try request(String(decoding: data, as: UTF8.self)), rendered)
  }

  func close() {
    if let context { closeFunction?(context) }
    context = nil
    requestFunction = nil
    renderFunction = nil
    closeFunction = nil
    if let library { dlclose(library) }
    library = nil
  }
}

private final class ProbeTexture: NSObject, FlutterTexture {
  private let lock = NSLock()
  private var latest: CVPixelBuffer?
  private var published = 0
  private var rasterRequests = 0
  private var hostReleases = 0

  func publish(_ buffer: CVPixelBuffer) {
    lock.lock()
    defer { lock.unlock() }
    if latest != nil { hostReleases += 1 }
    latest = buffer
    published += 1
  }

  func clear() {
    lock.lock()
    defer { lock.unlock() }
    if latest != nil { hostReleases += 1 }
    latest = nil
  }

  func counters() -> [String: Int] {
    lock.lock()
    defer { lock.unlock() }
    return ["publishedFrames": published, "rasterRequests": rasterRequests,
            "hostBufferReleases": hostReleases, "hostRetainedBuffers": latest == nil ? 0 : 1]
  }

  func copyPixelBuffer() -> Unmanaged<CVPixelBuffer>? {
    lock.lock()
    defer { lock.unlock() }
    guard let latest else { return nil }
    rasterRequests += 1
    return Unmanaged.passRetained(latest)
  }
}


final class ProbeSession {
  static let shared = ProbeSession()
  fileprivate let worker = DispatchQueue(label: "motolii.port.shared-document")
  fileprivate let runtime = ProbeRuntime()
  fileprivate let hosts = NSHashTable<ProbeHost>.weakObjects()
  fileprivate var paneState: [String: Any] = [:]
  fileprivate var epoch: UInt64 = 0
  fileprivate var latest: CVPixelBuffer?
  fileprivate var state: [String: Any] = [:]
  fileprivate var windows: [String: PanelFlutterWindow] = [:]
  private var confirming = false
  var terminationApproved = false

  fileprivate func broadcast(_ state: [String: Any], buffer: CVPixelBuffer? = nil, origin: ProbeHost? = nil, frameOnly: Bool = false) {
    precondition(Thread.isMainThread)
    self.state.merge(state) { _, next in next }
    if let buffer { latest = buffer }
    for host in hosts.allObjects where !host.closed {
      do {
        try host.ensureTexture()
        if let buffer { host.publish(buffer) }
        if host !== origin {
          var event = host.envelope(state, frameReady: buffer != nil)
          event["frameOnly"] = frameOnly
          host.channel.invokeMethod("documentChanged", arguments: event)
        }
      } catch {
        host.channel.invokeMethod("nativeError", arguments: String(describing: error))
      }
    }
  }

  fileprivate func clearFrames() {
    latest = nil
    for host in hosts.allObjects { host.detachTexture() }
  }

  func confirmTermination(_ completion: @escaping (Bool) -> Void) {
    precondition(Thread.isMainThread)
    if terminationApproved { completion(true); return }
    guard !confirming else { completion(false); return }
    confirming = true
    let group = DispatchGroup()
    for host in hosts.allObjects where !host.closed { group.enter(); host.channel.invokeMethod("flushEditors", arguments: nil) { _ in group.leave() } }
    group.notify(queue: .main) {
    self.worker.async {
      let status = try? self.runtime.status()
      DispatchQueue.main.async {
        if let status { self.state = status }
        guard self.state["dirty"] as? Bool == true else {
          self.confirming = false; completion(true); return
        }
        guard let main = self.hosts.allObjects.first(where: { $0.isMain && !$0.closed }) else {
          self.confirming = false; completion(false); return
        }
        main.window?.deminiaturize(nil)
        main.window?.makeKeyAndOrderFront(nil)
        main.channel.invokeMethod("documentChanged", arguments: main.envelope(self.state))
        main.channel.invokeMethod("confirmClose", arguments: nil) { reply in
          self.confirming = false
          completion((reply as? Bool) == true)
        }
      }
    }
  }
  }

  func shutdown(_ completion: @escaping () -> Void) {
    precondition(Thread.isMainThread)
    terminationApproved = true
    epoch &+= 1
    clearFrames()
    state = [:]
    worker.async {
      self.runtime.close()
      DispatchQueue.main.async {
        for window in Array(self.windows.values) { window.close() }
        completion()
      }
    }
  }

  fileprivate func openPanelWindow(_ panels: [String]) -> [String: Any] {
    let id = UUID().uuidString
    let window = PanelFlutterWindow(identifier: id, panels: panels)
    windows[id] = window
    window.center()
    window.makeKeyAndOrderFront(nil)
    return ["id": id, "panels": panels, "main": false]
  }
}

final class ProbeHost: NSObject {
  private let session = ProbeSession.shared
  private let registry: FlutterTextureRegistry
  fileprivate let channel: FlutterMethodChannel
  private var texture: ProbeTexture?
  private var textureID: Int64?
  fileprivate weak var window: NSWindow?
  fileprivate var closed = false
  let id: String
  let panels: [String]
  let isMain: Bool
  private var rendering = false

  static func install(controller: FlutterViewController, window: NSWindow? = nil,
                      id: String = "main", panels: [String] = [], main: Bool = true) -> ProbeHost {
    ProbeHost(controller: controller, window: window, id: id, panels: panels, main: main)
  }

  private init(controller: FlutterViewController, window: NSWindow?, id: String, panels: [String], main: Bool) {
    registry = controller.engine
    channel = FlutterMethodChannel(name: "motolii/probe", binaryMessenger: controller.engine.binaryMessenger)
    self.window = window
    self.id = id
    self.panels = panels
    isMain = main
    super.init()
    session.hosts.add(self)
    channel.setMethodCallHandler { [weak self] call, result in
      guard let self, !self.closed else {
        result(FlutterError(code: "closed", message: "Window detached", details: nil)); return
      }
      self.handle(call, result)
    }
  }

  fileprivate func ensureTexture() throws {
    guard textureID == nil else { return }
    let next = ProbeTexture()
    let id = registry.register(next)
    guard id != 0 else { throw ProbeFailure.message("Flutter texture registration failed") }
    texture = next
    textureID = id
    if let latest = session.latest { publish(latest) }
  }

  fileprivate func publish(_ buffer: CVPixelBuffer) {
    guard let texture, let textureID else { return }
    texture.publish(buffer)
    registry.textureFrameAvailable(textureID)
  }

  fileprivate func envelope(_ status: [String: Any], frameReady: Bool = false) -> [String: Any] {
    var reply: [String: Any] = ["status": status, "windowId": id, "frameReady": frameReady]
    if let textureID { reply["textureId"] = textureID }
    if let width = status["width"] { reply["width"] = width }
    if let height = status["height"] { reply["height"] = height }
    if let texture { reply.merge(texture.counters()) { _, new in new } }
    return reply
  }

  fileprivate func detachTexture() {
    precondition(Thread.isMainThread)
    if let textureID { registry.unregisterTexture(textureID) }
    texture?.clear()
    texture = nil
    textureID = nil
  }

  func filesDropped(_ paths: [String], point: [CGFloat]) {
    guard !closed else { return }
    channel.invokeMethod("filesDropped", arguments: ["paths": paths, "point": point])
  }

  func detachWindow() {
    guard !closed else { return }
    closed = true
    channel.setMethodCallHandler(nil)
    detachTexture()
    session.hosts.remove(self)
    for host in session.hosts.allObjects where host.isMain && !host.closed {
      host.channel.invokeMethod("windowClosed", arguments: ["id": id, "panels": panels])
    }
  }

  private func openPicker(_ panel: NSOpenPanel, result: @escaping FlutterResult, multiple: Bool) {
    let completion: (NSApplication.ModalResponse) -> Void = { response in
      guard response == .OK else { result(nil); return }
      if multiple { result(panel.urls.map { $0.path }) } else { result(panel.url?.path) }
    }
    if let window { panel.beginSheetModal(for: window, completionHandler: completion) }
    else { panel.begin(completionHandler: completion) }
  }

  private func handle(_ call: FlutterMethodCall, _ result: @escaping FlutterResult) {
    precondition(Thread.isMainThread)
    let args = call.arguments as? [String: Any] ?? [:]
    switch call.method {
    case "readSettings", "writeSettings":
      session.worker.async {
        let outcome: Result<Any, Error> = Result {
          let support = try FileManager.default.url(for: .applicationSupportDirectory, in: .userDomainMask, appropriateFor: nil, create: true)
          let directory = support.appendingPathComponent("MotoliiStage5", isDirectory: true)
          let file = directory.appendingPathComponent("layout.json")
          if call.method == "readSettings" {
            guard FileManager.default.fileExists(atPath: file.path) else { return [String: Any]() }
            guard let settings = try JSONSerialization.jsonObject(with: Data(contentsOf: file)) as? [String: Any] else {
              throw ProbeFailure.message("Saved layout is not a JSON object")
            }
            return settings
          }
          let data = try JSONSerialization.data(withJSONObject: args, options: [.sortedKeys])
          try FileManager.default.createDirectory(at: directory, withIntermediateDirectories: true)
          try data.write(to: file, options: .atomic)
          return true
        }
        DispatchQueue.main.async {
          do { result(try outcome.get()) }
          catch { self.fail(result, String(describing: error)) }
        }
      }
    case "openWeb":
      guard let raw = args["url"] as? String, let url = URL(string: raw), ["http", "https"].contains(url.scheme?.lowercased() ?? "") else { fail(result, "Enter an http or https URL"); return }
      result(NSWorkspace.shared.open(url))
    case "flushEditors":
      let group = DispatchGroup()
      for host in session.hosts.allObjects where !host.closed { group.enter(); host.channel.invokeMethod("flushEditors", arguments: nil) { _ in group.leave() } }
      group.notify(queue: .main) { result(true) }
    case "placePanel":
      guard let main = session.hosts.allObjects.first(where: { $0.isMain && !$0.closed }) else { fail(result, "Main window unavailable"); return }
      main.channel.invokeMethod("placePanel", arguments: args) { value in result(value) }
    case "setPaneState":
      session.paneState = args
      for host in session.hosts.allObjects where !host.closed { host.channel.invokeMethod("paneState", arguments: args) }
      result(true)
    case "windowInfo":
      result(["id": id, "panels": panels, "main": isMain, "paneState": session.paneState])
    case "focusWindow", "closeWindow":
      guard let requestedID = args["id"] as? String,
            let host = session.hosts.allObjects.first(where: { $0.id == requestedID && !$0.closed }),
            let target = host.window else { fail(result, "Window not found"); return }
      if call.method == "focusWindow" {
        target.deminiaturize(nil)
        target.makeKeyAndOrderFront(nil)
        result(true)
      } else {
        host.channel.invokeMethod("flushEditors", arguments: nil) { _ in target.performClose(nil); result(["requested": true]) }
      }
    case "openPanelWindow":
      let requested = (args["panels"] as? [String] ?? []).filter { !$0.isEmpty }
      guard !requested.isEmpty else { fail(result, "No supported panels requested"); return }
      result(session.openPanelWindow(requested))
    case "attach":
      perform(result, work: { try self.session.runtime.status() }) { status in
        try self.ensureTexture()
        return self.envelope(status)
      }
    case "pickOpen", "pickImport":
      let panel = NSOpenPanel()
      let importing = call.method == "pickImport"
      panel.canChooseDirectories = importing
      panel.canChooseFiles = true
      panel.allowsMultipleSelection = importing
      if importing {
        let types = ((args["extensions"] as? [String]) ?? []).compactMap { UTType(filenameExtension: $0) }
        if !types.isEmpty { panel.allowedContentTypes = types + [.folder] }
      } else {
        panel.allowedContentTypes = [UTType(filenameExtension: "rrd") ?? .data]
      }
      openPicker(panel, result: result, multiple: importing)
    case "pickSave", "pickExport":
      let panel = NSSavePanel()
      panel.nameFieldStringValue = args["name"] as? String ?? "Untitled.rrd"
      panel.allowedContentTypes = [UTType(filenameExtension: call.method == "pickExport" ? "mp4" : "rrd") ?? .data]
      let completion: (NSApplication.ModalResponse) -> Void = { response in result(response == .OK ? panel.url?.path : nil) }
      if let window { panel.beginSheetModal(for: window, completionHandler: completion) }
      else { panel.begin(completionHandler: completion) }
    case "reveal":
      guard let path = args["path"] as? String else { fail(result, "Missing file path"); return }
      NSWorkspace.shared.activateFileViewerSelecting([URL(fileURLWithPath: path)])
      result(true)
    case "open":
      guard isMain else { fail(result, "Open documents from the main window"); return }
      guard let path = args["path"] as? String else { fail(result, "open requires path"); return }
      session.epoch &+= 1
      session.terminationApproved = false
      rendering = false
      perform(result, work: {
        let status = try self.session.runtime.open(path: path)
        return status
      }) { status in
        self.session.clearFrames()
        self.session.broadcast(status, origin: self)
        return self.envelope(status)
      }
    case "noteClipboard":
      if let image = NSImage(pasteboard: .general), let tiff = image.tiffRepresentation, let bitmap = NSBitmapImageRep(data: tiff), let png = bitmap.representation(using: .png, properties: [:]) {
        result(["png": png.base64EncodedString()])
      } else { result(["text": NSPasteboard.general.string(forType: .string) ?? ""]) }
    case "easeModel":
      var query = args
      query["op"] = "easeModel"
      guard let data = try? JSONSerialization.data(withJSONObject: query), let command = String(data: data, encoding: .utf8) else { fail(result, "Invalid easing model query"); return }
      perform(result, work: { try self.session.runtime.request(command) }) { $0 }
    case "request":
      guard var command = args["command"] as? String else { fail(result, "request requires JSON command string"); return }
      if args["deferSnapshot"] != nil || args["knownSnapshotId"] != nil || args["knownReferenceId"] != nil {
        guard let data = command.data(using: .utf8), var query = (try? JSONSerialization.jsonObject(with: data)) as? [String: Any] else { fail(result, "Invalid command JSON"); return }
        for key in ["deferSnapshot", "knownSnapshotId", "knownReferenceId"] { if let value = args[key] { query[key] = value } }
        guard let data = try? JSONSerialization.data(withJSONObject: query) else { fail(result, "Invalid snapshot context"); return }
        command = String(decoding: data, as: UTF8.self)
      }
      perform(result, work: {
        let status = try self.session.runtime.request(command)
        return status
      }) { status in
        if status["error"] == nil { self.session.broadcast(status, origin: self) }
        return status
      }
    case "render":
      guard !rendering else { fail(result, "This window already has a render pending"); return }
      rendering = true
      perform(result, isRender: true, work: {
        if args["playing"] as? Bool == true {
          _ = try self.session.runtime.request("{\"op\":\"tick\",\"quiet\":true}")
        } else if let frame = args["frame"] as? NSNumber {
          _ = try self.session.runtime.request("{\"op\":\"seek\",\"quiet\":true,\"frame\":\(frame.int64Value)}")
        }
        return try self.session.runtime.render(known: args["knownSnapshotId"], references: args["knownReferenceId"])
      }) { buffer, status, rendered in
        self.session.broadcast(status, buffer: buffer, origin: self, frameOnly: args["frame"] != nil || args["playing"] as? Bool == true)
        var reply = self.envelope(status, frameReady: true)
        reply["renderedFrames"] = rendered
        reply["frameOnly"] = args["frame"] != nil || args["playing"] as? Bool == true
        return reply
      }
    case "close":
      if !isMain {
        detachTexture()
        result(["detached": true, "windowId": id])
        return
      }
      session.epoch &+= 1
      rendering = false
      session.clearFrames()
      session.state = [:]
      perform(result, work: { self.session.runtime.close(); return ["closed": true] }) { state in
        for host in self.session.hosts.allObjects where !host.closed {
          host.channel.invokeMethod("documentClosed", arguments: nil)
        }
        return state
      }
    default:
      result(FlutterMethodNotImplemented)
    }
  }

  private func perform<T>(_ result: @escaping FlutterResult, isRender: Bool = false,
                          work: @escaping () throws -> T, finish: @escaping (T) throws -> Any) {
    let epoch = session.epoch
    session.worker.async {
      let outcome = Result { try work() }
      DispatchQueue.main.async {
        guard !self.closed, epoch == self.session.epoch else {
          result(FlutterError(code: "superseded", message: "Document or window changed", details: nil)); return
        }
        if isRender { self.rendering = false }
        do { result(try finish(outcome.get())) }
        catch { self.fail(result, String(describing: error)) }
      }
    }
  }

  private func fail(_ result: FlutterResult, _ message: String) {
    result(FlutterError(code: "probe", message: message, details: nil))
  }
}

final class PanelFlutterWindow: NSWindow, NSWindowDelegate, NSDraggingDestination {
  private var host: ProbeHost?
  private let panelID: String
  private var closeApproved = false
  private var closePending = false
  init(identifier: String, panels: [String]) {
    panelID = identifier
    super.init(contentRect: NSRect(x: 0, y: 0, width: 760, height: 540),
               styleMask: [.titled, .closable, .miniaturizable, .resizable], backing: .buffered, defer: false)
    isReleasedWhenClosed = false
    title = "Motolii · " + panels.joined(separator: " / ")
    minSize = NSSize(width: 320, height: 220)
    delegate = self
    let controller = FlutterViewController()
    contentViewController = controller
    RegisterGeneratedPlugins(registry: controller)
    host = ProbeHost.install(controller: controller, window: self, id: identifier, panels: panels, main: false)
    registerForDraggedTypes([.fileURL])
    setContentSize(NSSize(width: 760, height: 540))
  }
  private func paths(_ sender: NSDraggingInfo) -> [String] {
    let urls = sender.draggingPasteboard.readObjects(forClasses: [NSURL.self], options: [.urlReadingFileURLsOnly: true]) as? [URL] ?? []
    return urls.filter { $0.isFileURL }.map { $0.path }
  }
  func draggingEntered(_ sender: NSDraggingInfo) -> NSDragOperation { paths(sender).isEmpty ? [] : .copy }
  func performDragOperation(_ sender: NSDraggingInfo) -> Bool {
    let files = paths(sender); guard !files.isEmpty else { return false }
    let point = contentView?.convert(sender.draggingLocation, from: nil) ?? sender.draggingLocation
    host?.filesDropped(files, point: [point.x, contentView?.isFlipped == true ? point.y : (contentView?.bounds.height ?? frame.height) - point.y])
    return true
  }
  func windowShouldClose(_ sender: NSWindow) -> Bool {
    if closeApproved { return true }
    if closePending { return false }
    closePending = true
    host?.channel.invokeMethod("flushEditors", arguments: nil) { [weak self] _ in
      self?.closeApproved = true
      self?.performClose(nil)
    }
    return false
  }
  func windowWillClose(_ notification: Notification) {
    host?.detachWindow()
    host = nil
    ProbeSession.shared.windows.removeValue(forKey: panelID)
  }
}
