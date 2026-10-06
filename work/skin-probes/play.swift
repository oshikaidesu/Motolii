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
func pump(_ ms: Double) { RunLoop.current.run(until: Date(timeIntervalSinceNow: ms / 1000)) }
var w = 0.0
while item.status != .readyToPlay && w < 5000 { pump(5); w += 5 }

func run(rate: Float, from t: Double, seconds: Double) {
    var done = false
    player.seek(to: CMTime(seconds: t, preferredTimescale: 600), toleranceBefore: .zero, toleranceAfter: .zero) { _ in done = true }
    while !done { pump(1) }
    player.rate = 0
    var frames = 0
    var lastTime = -1.0
    var gaps: [Double] = []
    var lastWall = 0.0
    let start = DispatchTime.now().uptimeNanoseconds
    player.playImmediately(atRate: rate)
    var firstWall: Double? = nil
    while Double(DispatchTime.now().uptimeNanoseconds - start) / 1e9 < seconds {
        let now = item.currentTime()
        if out.hasNewPixelBuffer(forItemTime: now) {
            var disp = CMTime.zero
            if let pb = out.copyPixelBuffer(forItemTime: now, itemTimeForDisplay: &disp) {
                _ = pb
                let ts = CMTimeGetSeconds(disp)
                if ts != lastTime {
                    frames += 1
                    let wall = Double(DispatchTime.now().uptimeNanoseconds - start) / 1e6
                    if firstWall == nil { firstWall = wall } else { gaps.append(wall - lastWall) }
                    lastWall = wall
                    lastTime = ts
                }
            }
        }
        pump(0.5)
    }
    player.rate = 0
    let g = gaps.sorted()
    let secs = seconds - (firstWall ?? 0) / 1000
    print(String(format: "rate %+.1f: %d distinct frames in %.1fs wall = %.1f frames/s; inter-frame gap ms median %.1f p95 %.1f max %.1f",
                 rate, frames, secs, Double(frames) / secs, g.isEmpty ? 0 : g[g.count/2], g.isEmpty ? 0 : g[Int(Double(g.count)*0.95)], g.last ?? 0))
}
print("canPlayReverse:", item.canPlayReverse, " canPlayFastForward:", item.canPlayFastForward)
run(rate: 1.0, from: 30, seconds: 5)
run(rate: 2.0, from: 30, seconds: 5)
run(rate: 4.0, from: 30, seconds: 4)
run(rate: -1.0, from: 60, seconds: 5)
run(rate: -2.0, from: 90, seconds: 4)
