import 'dart:convert';

enum DocumentOperation {
  status('status'),
  notes('notes'),
  stageView('stageView'),
  select('select'),
  setProperty('setProperty'),
  previewProperties('previewProperties'),
  commitPreview('commitPreview'),
  cancelPreview('cancelPreview'),
  setText('setText'),
  setAttrs('setAttrs'),
  create('create'),
  duplicate('duplicate'),
  copy('copy'),
  cut('cut'),
  paste('paste'),
  delete('delete'),
  group('group'),
  ungroup('ungroup'),
  reorder('reorder'),
  split('split'),
  setTiming('setTiming'),
  toggleKey('toggleKey'),
  moveKeys('moveKeys'),
  ease('ease'),
  setColor('setColor'),
  previewColor('previewColor'),
  focusColor('focusColor'),
  applyPalette('applyPalette'),
  applyEffect('applyEffect'),
  removeEffect('removeEffect'),
  clip('clip'),
  addMarker('addMarker'),
  setMarker('setMarker'),
  deleteMarker('deleteMarker'),
  composition('composition'),
  importFiles('import'),
  placeAsset('placeAsset'),
  removeAsset('removeAsset'),
  replaceAsset('replaceAsset'),
  save('save'),
  newDocument('new'),
  undo('undo'),
  redo('redo'),
  seek('seek'),
  anchor('anchor'),
  freeze('freeze'),
  setFillMode('setFillMode'),
  previewBlend('previewBlend'),
  setTimings('setTimings'),
  previewTimings('previewTimings'),
  stageGesture('stageGesture'),
  exportDocument('export'),
  exportStatus('exportStatus'),
  cancelExport('cancelExport'),
  play('play'),
  pause('pause'),
  tick('tick'),
  moveLayers('moveLayers');

  const DocumentOperation(this.wireName);
  final String wireName;

  static DocumentOperation parse(String name) => values.firstWhere(
    (operation) => operation.wireName == name,
    orElse: () =>
        throw ArgumentError.value(name, 'op', 'Unknown document operation'),
  );

  bool get requiresPause => !{
    status,
    notes,
    exportStatus,
    copy,
    save,
    exportDocument,
    cancelExport,
  }.contains(this);

  bool get requiresRender => !{
    status,
    notes,
    exportStatus,
    copy,
    save,
    focusColor,
    exportDocument,
    cancelExport,
  }.contains(this);

  static void validateArguments(Map<String, dynamic> arguments) {
    if (arguments.containsKey('op')) {
      throw ArgumentError('Command arguments cannot override op');
    }
  }

  String encode(Map<String, dynamic> arguments) {
    validateArguments(arguments);
    return jsonEncode({'op': wireName, ...arguments});
  }
}
