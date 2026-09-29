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

/// What the last look through the catalog did for the work's missing media, in one line (empty when there is nothing to
/// say: nothing was missing, or the catalog knew nothing more than the work did).
String relinkNotice(Map<String, dynamic> status) {
  final r = status['relink'];
  if (r is! Map) return '';
  int n(String key) => (r[key] as List? ?? const []).length;
  final found = n('relinked'), several = n('ambiguous'), away = n('unavailable');
  final parts = [
    if (found > 0) 'Found $found moved file${found == 1 ? '' : 's'} through the catalog',
    if (several > 0) '$several ${several == 1 ? 'file has' : 'files have'} several possible matches',
    if (away > 0) '$away ${away == 1 ? 'file is' : 'files are'} on a source that is not connected',
  ];
  return parts.join('; ');
}
