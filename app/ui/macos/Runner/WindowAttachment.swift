/// Identifies a UI attachment, not the document or the OS window.
struct WindowAttachment {
  private var generation = 0
  private(set) var id: Int?

  mutating func attach(reusing expected: Int?) -> Int? {
    if let expected { return expected == id ? id : nil }
    generation = generation &+ 1
    if generation <= 0 { generation = 1 }
    id = generation
    return id
  }

  mutating func detach(_ expected: Int) -> Bool {
    guard id == expected else { return false }
    id = nil
    return true
  }

  mutating func invalidate() { id = nil }
}
