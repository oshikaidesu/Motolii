import Foundation

final class HistoryStore {
  let directory: URL
  private let reportsDirectory: URL
  private let sessionID = UUID().uuidString
  private var records: [[String: Any]] = []
  private var checkpoints: [[String: Any]] = []
  private var persistenceFailure: String?
  private var parent: String?
  private let stamp = ISO8601DateFormatter()

  init(directory: URL, reportsDirectory: URL) throws {
    self.directory = directory
    self.reportsDirectory = reportsDirectory
    try FileManager.default.createDirectory(at: directory, withIntermediateDirectories: true)
    records = try read("records.json")
    checkpoints = try read("checkpoints.json")
    let marker = directory.appendingPathComponent("active-session.json")
    if FileManager.default.fileExists(atPath: marker.path) {
      try record("warning", "Previous session did not close normally", "This can follow a crash, force quit, or development restart. Check OS crash reports for evidence.")
    }
    try Data(sessionID.utf8).write(to: marker, options: .atomic)
    try record("session", "Session started")
  }

  private func read(_ name: String) throws -> [[String: Any]] {
    let url = directory.appendingPathComponent(name)
    guard FileManager.default.fileExists(atPath: url.path) else { return [] }
    guard let rows = try JSONSerialization.jsonObject(with: Data(contentsOf: url)) as? [[String: Any]] else {
      throw NSError(domain: "History", code: 1, userInfo: [NSLocalizedDescriptionKey: "Invalid history file: \(name)"])
    }
    return rows
  }

  private func write(_ rows: [[String: Any]], _ name: String) throws {
    try JSONSerialization.data(withJSONObject: rows, options: [.sortedKeys]).write(
      to: directory.appendingPathComponent(name), options: .atomic)
  }

  func record(_ kind: String, _ title: String, _ detail: String = "") throws {
    let clipped = String(detail.prefix(32768))
    if let last = records.last, kind == "error", last["title"] as? String == title,
       last["detail"] as? String == clipped { return }
    let row: [String: Any] = ["id": UUID().uuidString, "time": stamp.string(from: Date()),
      "kind": kind, "title": String(title.prefix(256)), "detail": clipped, "session": sessionID]
    let next = Array((records + [row]).suffix(500))
    try write(next, "records.json")
    records = next
  }

  func close() throws {
    try record("session", "Session closed")
    let marker = directory.appendingPathComponent("active-session.json")
    if (try? String(contentsOf: marker, encoding: .utf8)) == sessionID {
      try FileManager.default.removeItem(at: marker)
    }
  }

  func documentOpened() { parent = nil }

  private func checked(_ command: [String: Any], run: (String) throws -> [String: Any]) throws -> [String: Any] {
    let data = try JSONSerialization.data(withJSONObject: command)
    let result = try run(String(decoding: data, as: UTF8.self))
    if let message = result["error"] as? String, !message.isEmpty {
      throw NSError(domain: "History", code: 2, userInfo: [NSLocalizedDescriptionKey: message])
    }
    return result
  }

  @discardableResult
  func checkpoint(_ name: String, run: (String) throws -> [String: Any]) throws -> [String: Any] {
    let name = name.trimmingCharacters(in: .whitespacesAndNewlines)
    guard !name.isEmpty, name.count <= 120 else {
      throw NSError(domain: "History", code: 3, userInfo: [NSLocalizedDescriptionKey: "Use a checkpoint name of 1–120 characters"])
    }
    let id = UUID().uuidString
    let path = directory.appendingPathComponent(id + ".rrd").path
    let state = try checked(["op": "save", "path": path, "copy": true], run: run)
    var entry: [String: Any] = ["id": id, "name": name, "time": stamp.string(from: Date()),
      "source": state["path"] as? String ?? "", "parent": parent ?? ""]
    entry["detail"] = "Local snapshot; external media stays at its original location."
    let next = checkpoints + [entry]
    try write(next, "checkpoints.json")
    checkpoints = next
    parent = id
    do { try record("checkpoint", "Checkpoint saved: \(name)", state["path"] as? String ?? "Untitled") }
    catch { persistenceFailure = String(describing: error) }
    return entry
  }

  func restore(_ id: String, run: (String) throws -> [String: Any]) throws -> [String: Any] {
    guard let entry = checkpoints.first(where: { $0["id"] as? String == id }) else {
      throw NSError(domain: "History", code: 4, userInfo: [NSLocalizedDescriptionKey: "Checkpoint not found"])
    }
    let path = directory.appendingPathComponent(id + ".rrd").path
    guard FileManager.default.isReadableFile(atPath: path) else {
      throw NSError(domain: "History", code: 5, userInfo: [NSLocalizedDescriptionKey: "Checkpoint file is missing"])
    }
    try checkpoint("Before restoring \(String((entry["name"] as? String ?? "checkpoint").prefix(80)))", run: run)
    let state = try checked(["op": "restoreCheckpoint", "path": path,
      "source": entry["source"] as? String ?? ""], run: run)
    parent = id
    do { try record("checkpoint", "Checkpoint restored: \(entry["name"] as? String ?? id)") }
    catch { persistenceFailure = String(describing: error) }
    return state
  }

  private func reports() -> [URL] {
    let urls = (try? FileManager.default.contentsOfDirectory(at: reportsDirectory,
      includingPropertiesForKeys: [.contentModificationDateKey])) ?? []
    return Array(urls.filter {
      $0.lastPathComponent.hasPrefix("motolii_stage5") && ["ips", "crash"].contains($0.pathExtension)
    }.sorted { $0.lastPathComponent > $1.lastPathComponent }.prefix(30))
  }

  func report(_ name: String) throws -> String {
    guard let url = reports().first(where: { $0.lastPathComponent == name }) else {
      throw NSError(domain: "History", code: 6, userInfo: [NSLocalizedDescriptionKey: "Crash report not found"])
    }
    return try String(contentsOf: url, encoding: .utf8)
  }

  func snapshot() -> [String: Any] {
    var result: [String: Any] = ["records": Array(records.reversed()), "checkpoints": Array(checkpoints.reversed()),
      "directory": directory.path, "reports": reports().map { ["name": $0.lastPathComponent] }]
    if let persistenceFailure { result["failure"] = persistenceFailure }
    return result
  }
}
