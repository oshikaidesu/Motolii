import 'dart:convert';

import 'package:flutter/foundation.dart';

import '../../../foundation/panel_catalog.dart';
import '../../../../session/editor_session.dart';

/// What the Desk follows: the selection, its keys, and the kind of layer — Classic's own [deskIdentity]
/// (`lib/panels/desk.dart`), read the same way so the two hosts never disagree about when to re-decide.
String deskIdentity(EditorSession c) => jsonEncode([c.selectedIds, c.state['selectedKeys'], c.activeLayer?['kind']]);

/// Which drawer the Desk shows, and why: the same rule as Classic's `_DeskPanelState` — a manual pick from
/// `c.deskDrawer`, else what the selection asks for (`Ease` for a Sequence or keys picked, `Depth` for a Camera),
/// else the starred default, else the empty "Tools" catalog. This is presentation-neutral state (what name is
/// shown), not a second copy of what any drawer does — Ease/Depth/Blend/History/Notes/Web still own themselves.
class DeskHostController extends ChangeNotifier {
  DeskHostController(this.c) {
    _selection = deskIdentity(c);
    _automatic = _selectionPanel();
    c.deskDrawer.addListener(_settle);
    c.deskDefault.addListener(_settle);
    c.panePlaces.addListener(_settle);
    c.editingFocus.addListener(_focus);
    _slice = c.slice('newDeskHost', const ['selectedKeys', 'capabilities'])..addListener(_read);
  }

  final EditorSession c;
  late final DocumentSlice _slice;
  String _selection = '';
  String? _automatic;
  bool inside = false; // a click that started inside the Desk should not lose the manual pick on blur

  bool _inDrawer(String name) => c.panePlaces.value[name] == null || c.panePlaces.value[name] == 'drawer';
  String _name(String value) => value == 'Text' || value == 'Reference' ? 'Notes' : value;

  String get shown {
    final manual = c.deskDrawer.value;
    if (manual == 'Tools') return 'Tools';
    for (final candidate in [manual, _automatic, c.deskDefault.value]) {
      if (candidate == null) continue;
      final name = _name(candidate);
      if (panelSpec(name)?.drawer == true && _inDrawer(name)) return name;
    }
    return 'Tools';
  }

  String? _selectionPanel() {
    if (EditorSession.maps(c.state['selectedKeys']).isNotEmpty) return 'Ease';
    if (c.selectedIds.length > 1) return 'Ease';
    return switch (c.activeLayer?['kind']) {
      'Camera' => 'Depth',
      _ => null,
    };
  }

  void _read() {
    final now = deskIdentity(c);
    if (now == _selection) return;
    _selection = now;
    if (!inside) _follow(_selectionPanel());
  }

  void _follow(String? name) {
    _automatic = name;
    c.deskDrawer.value = null;
    _settle();
  }

  void _focus() {
    final focus = c.editingFocus.value;
    if (focus['selection'] == true) {
      if (!inside) _follow(_selectionPanel());
      return;
    }
    final layer = c.layers.where((l) => l['id'] == focus['layer']).firstOrNull;
    if (layer == null || !c.selectedIds.contains(layer['id'])) return;
    final property = focus['property'];
    _follow(property == 'keyframes' ? 'Ease' : layer['kind'] == 'Camera' ? 'Depth' : property == 'blendMode' ? 'Blend' : null);
  }

  void _settle() => notifyListeners();

  /// Open a drawer: inline if it lives in the Desk, its own tab otherwise (Classic's `_open`).
  void open(String name) {
    if (_inDrawer(name)) {
      c.deskDrawer.value = name;
    } else {
      c.placePanel(name, 'show');
    }
  }

  void toolsCatalog() => c.deskDrawer.value = 'Tools';

  /// Every drawer the Desk can hold, in the order the catalog declares them.
  List<PanelSpec> get catalog => [for (final spec in panelCatalog) if (spec.drawer && _inDrawer(spec.name)) spec];

  String get starred => _name(c.deskDefault.value);
  void star(String name) => c.deskDefault.value = starred == name ? 'Tools' : name;

  @override
  void dispose() {
    c.deskDrawer.removeListener(_settle);
    c.deskDefault.removeListener(_settle);
    c.panePlaces.removeListener(_settle);
    c.editingFocus.removeListener(_focus);
    _slice.removeListener(_read);
    super.dispose();
  }
}
