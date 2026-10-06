import AVFoundation
import CoreVideo
import Foundation

let path = CommandLine.arguments[1]
let asset = AVURLAsset(url: URL(fileURLWithPath: path))
let item = AVPlayerItem(asset: asset)
let attrs: [String: Any] = [
    kCVPixelBufferPixelFormatTypeKey as String: kCVPixelFormatType_32BGRA,
    kCVPixelBufferMetalCompatibilityKey as String: true,
    kCVPixelBufferIOSurfacePropertiesKey as String: [String: Any]()
]
let out = AVPlayerItemVideoOutput(pixelBufferAttributes: attrs)
item.add(out)
let player = AVPlayer(playerItem: item)
player.rate = 0
func pump(_ ms: Double) { RunLoop.current.run(until: Date(timeIntervalSinceNow: ms / 1000)) }
var waited = 0.0
while item.status != .readyToPlay && waited < 5000 { pump(5); waited += 5 }
print("item status:", item.status.rawValue, "duration:", CMTimeGetSeconds(item.duration))
var times: [Double] = []
var iosurface = 0
srand48(7)
for _ in 0..<30 {
    let t = drand48() * 200.0
    let start = DispatchTime.now().uptimeNanoseconds
    var done = false
    player.seek(to: CMTime(seconds: t, preferredTimescale: 600), toleranceBefore: .zero, toleranceAfter: .zero) { _ in done = true }
    while !done { pump(0.5) }
    var got = false
    var spins = 0
    while !got && spins < 4000 {
        let ct = item.currentTime()
        if let pb = out.copyPixelBuffer(forItemTime: ct, itemTimeForDisplay: nil) {
            got = true
            if CVPixelBufferGetIOSurface(pb) != nil { iosurface += 1 }
        } else { pump(0.5); spins += 1 }
    }
    let ms = Double(DispatchTime.now().uptimeNanoseconds - start) / 1e6
    times.append(got ? ms : -1)
}
let ok = times.filter { $0 >= 0 }.sorted()
print("exact seeks ok:", ok.count, "of 30; IOSurface-backed:", iosurface)
if !ok.isEmpty {
    print(String(format: "seek->CVPixelBuffer ms: min %.1f  median %.1f  p90 %.1f  max %.1f", ok.first!, ok[ok.count/2], ok[Int(Double(ok.count)*0.9)], ok.last!))
}
