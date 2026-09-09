import Foundation

// VS Code local-history oracle: restore an immutable saved state; keep subsequent work recoverable.
@main struct HistoryStoreTest {
  static func main() throws {
    let root = FileManager.default.temporaryDirectory.appendingPathComponent(UUID().uuidString)
    defer { try? FileManager.default.removeItem(at: root) }
    let reports = root.appendingPathComponent("reports")
    try FileManager.default.createDirectory(at: reports, withIntermediateDirectories: true)
    try Data("OS crash detail".utf8).write(to: reports.appendingPathComponent("motolii_stage5-test.ips"))
    try Data("unrelated".utf8).write(to: reports.appendingPathComponent("Other-test.ips"))
    let directory = root.appendingPathComponent("history")
    var store = try HistoryStore(directory: directory, reportsDirectory: reports)
    var content = "original"
    var failSave = false
    let run: (String) throws -> [String: Any] = { raw in
      let command = try JSONSerialization.jsonObject(with: Data(raw.utf8)) as! [String: Any]
      switch command["op"] as? String {
      case "save":
        if failSave { return ["error": "Disk full"] }
        precondition(command["copy"] as? Bool == true)
        try Data(content.utf8).write(to: URL(fileURLWithPath: command["path"] as! String))
      case "restoreCheckpoint":
        content = try String(contentsOfFile: command["path"] as! String, encoding: .utf8)
      default: preconditionFailure("Unexpected command")
      }
      return ["path": "/project.rrd", "dirty": true]
    }
    let entry = try store.checkpoint("First", run: run)
    content = "later work"
    _ = try store.restore(entry["id"] as! String, run: run)
    precondition(content == "original")
    let rows = store.snapshot()["checkpoints"] as! [[String: Any]]
    precondition(rows.count == 2)
    let recoveryID = rows[0]["id"] as! String
    let recovery = try String(contentsOf: directory.appendingPathComponent(recoveryID + ".rrd"), encoding: .utf8)
    precondition(recovery == "later work")
    content = "unsaved after restore"
    failSave = true
    do { _ = try store.checkpoint("Failed", run: run); preconditionFailure("Must reject") } catch {}
    do { _ = try store.restore(entry["id"] as! String, run: run); preconditionFailure("Must not restore without backup") } catch {}
    precondition((store.snapshot()["checkpoints"] as! [Any]).count == 2)
    precondition(content == "unsaved after restore")
    precondition((store.snapshot()["reports"] as! [Any]).count == 1)
    let report = try store.report("motolii_stage5-test.ips")
    precondition(report == "OS crash detail")
    do { _ = try store.report("../Other-test.ips"); preconditionFailure("Must reject") } catch {}
    for i in 0..<505 { try store.record("save", "Saved \(i)") }
    precondition((store.snapshot()["records"] as! [Any]).count == 500)
    // An unclosed marker means interruption, not a confirmed crash.
    store = try HistoryStore(directory: directory, reportsDirectory: reports)
    let records = store.snapshot()["records"] as! [[String: Any]]
    precondition(records.contains { $0["kind"] as? String == "warning" })
    precondition((store.snapshot()["checkpoints"] as! [Any]).count == 2)
    try store.close()
    precondition(!FileManager.default.fileExists(atPath: directory.appendingPathComponent("active-session.json").path))
    print("History persistence, recovery, rejection, report isolation, bounds and restart: passed")
  }
}
