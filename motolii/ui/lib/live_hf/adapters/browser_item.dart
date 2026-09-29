import 'package:flutter/foundation.dart';

/// One asset of a result set as the Browser's views see it: identity, what it is, what is known about it, and the faces
/// already made for it. Where it came from (the real catalog, a fixture) is not the views' business: every view reads
/// the same list of these.
@immutable
class BrowserItem {
  const BrowserItem({
    required this.id,
    required this.name,
    required this.path,
    required this.kind,
    required this.mime,
    this.source = '',
    this.rel = '',
    this.size,
    this.mtimeNs,
    this.width,
    this.height,
    this.seconds,
    this.sampleRate,
    this.channels,
    this.faceKey = '',
    this.fingerprint,
    this.missing = false,
    this.thumbnail,
    this.peaks,
  });

  final String id, name, path, mime, source, rel, faceKey;

  /// The catalog's cheap content fingerprint, once learned: two assets with the same one are the same bytes.
  final String? fingerprint;

  /// image | video | audio | model | environment (the catalog's own words for the media type).
  final String kind;
  final int? size, mtimeNs, width, height, sampleRate, channels;
  final double? seconds;
  final bool missing;

  /// A picture for a clip or an environment: a data URI (an image is drawn from its own file).
  final String? thumbnail;
  final Object? peaks;

  /// The library's own family word, which the shelf's faces switch on.
  String get family => const {'image': '2D', 'video': 'Video', 'audio': 'Audio', 'model': '3D', 'environment': 'HDR'}[kind] ?? '2D';

  /// The mark of a piece that is not a still (empty for a still).
  String get mark {
    String clock(double s) => '${s ~/ 60}:${(s.round() % 60).toString().padLeft(2, '0')}';
    return switch (kind) {
      'video' => seconds == null ? '▶' : '▶ ${clock(seconds!)}',
      'audio' => seconds == null ? '♪' : '♪ ${clock(seconds!)}',
      'model' => '3D ↻',
      'environment' => '360° ↔',
      _ => '',
    };
  }

  double get aspect => switch (kind) {
        'environment' => 2,
        'audio' => 1.7,
        'model' => 1,
        _ => (width != null && height != null && height! > 0) ? (width! / height!).clamp(.5, 2.4).toDouble() : (kind == 'video' ? 16 / 9 : 1),
      };

  /// The map the Media shelf's faces (materialFace, MaterialCard) draw from.
  Map<String, dynamic> get shelf => {
        'id': id,
        'name': name,
        'path': path,
        'mime': mime,
        'family': family,
        'mediaFamily': family,
        'facts': {if (width != null) 'width': width, if (height != null) 'height': height, if (seconds != null) 'seconds': seconds},
        if (seconds != null) 'seconds': seconds,
        if (thumbnail != null) 'thumbnail': thumbnail,
        if (peaks != null) 'peaks': peaks,
        'missing': missing,
        'used': false,
      };

  String get typeWord => switch (kind) { 'image' => 'Image', 'video' => 'Video', 'audio' => 'Audio', 'model' => '3D', 'environment' => 'HDR', _ => kind };
}

/// A Result Set: what the Browser draws, from wherever it comes (the catalog owner's answer to a query, or a fixture).
abstract class ResultSource extends Listenable {
  List<BrowserItem> get items;
}

/// "12.4 MB", "980 KB" — a file size as people read it.
String sizeText(int? bytes) {
  if (bytes == null) return '';
  if (bytes >= 1 << 30) return '${(bytes / (1 << 30)).toStringAsFixed(1)} GB';
  if (bytes >= 1 << 20) return '${(bytes / (1 << 20)).toStringAsFixed(1)} MB';
  if (bytes >= 1 << 10) return '${(bytes / (1 << 10)).round()} KB';
  return '$bytes B';
}

/// "2025/09/26" from a modified time in nanoseconds since the epoch.
String dateText(int? ns) {
  if (ns == null || ns <= 0) return '';
  final t = DateTime.fromMicrosecondsSinceEpoch(ns ~/ 1000);
  return '${t.year}/${t.month.toString().padLeft(2, '0')}/${t.day.toString().padLeft(2, '0')}';
}

String clockText(double? s) => s == null ? '' : '${s ~/ 60}:${(s.round() % 60).toString().padLeft(2, '0')}';
