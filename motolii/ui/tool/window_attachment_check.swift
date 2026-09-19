@main
enum WindowAttachmentCheck {
  static func main() {
    var window = WindowAttachment()
    let first = window.attach(reusing: nil)!
    precondition(window.attach(reusing: first) == first)
    let next = window.attach(reusing: nil)!
    precondition(next != first)
    precondition(!window.detach(first))
    precondition(window.id == next)
    precondition(window.attach(reusing: first) == nil)
    precondition(window.detach(next))
    precondition(window.id == nil)
    precondition(!window.detach(next))
    let reopened = window.attach(reusing: nil)!
    precondition(reopened != next)
    var other = WindowAttachment()
    let peer = other.attach(reusing: nil)!
    window.invalidate()
    precondition(window.id == nil)
    precondition(other.id == peer)
    print("Window attachment lifecycle: PASS")
  }
}
