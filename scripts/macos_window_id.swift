import CoreGraphics
import Foundation

guard CommandLine.arguments.count == 2, let pidValue = Int32(CommandLine.arguments[1]) else {
    fputs("usage: macos_window_id <pid>\n", stderr)
    exit(2)
}

let pid = pid_t(pidValue)
let options: CGWindowListOption = [.optionOnScreenOnly, .excludeDesktopElements]
let windows = (CGWindowListCopyWindowInfo(options, kCGNullWindowID) as? [[String: Any]]) ?? []

var best: (id: Int, area: Double)?
for window in windows {
    guard
        let owner = window[kCGWindowOwnerPID as String] as? NSNumber,
        owner.int32Value == pidValue,
        let layer = window[kCGWindowLayer as String] as? NSNumber,
        layer.intValue == 0,
        let number = window[kCGWindowNumber as String] as? NSNumber
    else {
        continue
    }

    var area = 1.0
    if let dictionary = window[kCGWindowBounds as String] as? NSDictionary {
        var rect = CGRect.zero
        if CGRectMakeWithDictionaryRepresentation(dictionary, &rect) {
            area = max(0.0, Double(rect.width * rect.height))
        }
    }

    let candidate = (id: number.intValue, area: area)
    if best == nil || candidate.area > best!.area {
        best = candidate
    }
}

if let best {
    print(best.id)
    exit(0)
}

exit(1)
