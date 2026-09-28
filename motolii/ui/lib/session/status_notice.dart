// What the status says about a background job, as one line for the status bar: shared by every shell.

/// Freeze の裏仕事の進み。走っていなければ null。失敗はその理由。
String? freezeNotice(Map<String, dynamic> status) {
  final job = status['freeze'];
  if (job is! Map) return null;
  final phase = '${job['phase']}';
  if (phase != 'running' && phase != 'cancelling' && phase != 'failed')
    return null;
  final layers = (status['layers'] as List? ?? const []).whereType<Map>();
  final name =
      layers
          .where((l) => l['id'] == job['layer'])
          .map((l) => '${l['name']}')
          .firstOrNull ??
      'layer';
  if (phase == 'failed') return 'Freeze failed: ${job['error']}';
  return 'Freezing $name ${job['done']}/${job['total']}';
}
