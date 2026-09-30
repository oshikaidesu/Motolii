import Foundation

final class Runtime {
  // ruleid: motolii-swift-status-as-dictionary
  func status() throws -> [String: Any] { try request("{}") }

  // ruleid: motolii-swift-status-as-dictionary
  fileprivate func renderInto(_ targets: [Int]) throws -> ([String: Any], Int) { return ([:], 0) }

  // ruleid: motolii-swift-status-as-dictionary
  func catalog(_ command: String, completion: @escaping (Result<[String: Any], Error>) -> Void) {}

  // ruleid: motolii-swift-status-as-dictionary
  func render(known: Any? = nil) throws -> ([String: CVPixelBuffer], [String: Any], Int) { fatalError() }

  // ok: motolii-swift-status-as-dictionary
  func request(_ command: String) throws -> [String: Any] { [:] }

  // ok: motolii-swift-status-as-dictionary
  func status() throws -> NativeStatus { try requestStatus("{}") }

  // ok: motolii-swift-status-as-dictionary
  func catalog(_ command: String, completion: @escaping (Result<Data, Error>) -> Void) {}

  // ok: motolii-swift-status-as-dictionary
  private func easeModel(_ query: [String: Any]) throws -> [String: Any] { [:] }
}
