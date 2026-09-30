part of '../inspector.dart';

/// What the panel remembers about what is open: the cards it has been told
/// to close, and the effects showing their advanced rows.
mixin _InspectorFolds on State<InspectorPanel> {
  final _closed = <String>{};

  String _effectSection(Map<String, dynamic> layer, Object? effect) =>
      'effect:${layer['id']}:$effect';

  Widget _card({
    Key? key,
    required String title,
    required List<Widget> children,
    String? section,
    Widget? leading,
    Widget? trailing,
    bool dim = false,
  }) {
    final id = section ?? title;
    return EditorCard(
      key: key ?? ValueKey('section:$id'),
      title: title,
      expanded: !_closed.contains(id),
      onToggle: () => setState(() {
        if (!_closed.remove(id)) _closed.add(id);
      }),
      leading: leading,
      trailing: trailing,
      dim: dim,
      children: children,
    );
  }

  /// Effects whose advanced fold is open, by effect id.
  /// Which effects show their advanced rows: a fold's own signal, so opening
  /// one does not rebuild the panel.
  final _advancedOpen = ValueNotifier<Set<String>>(const {});

  bool _effectsClosed(Map<String, dynamic> layer) => panelRows(layer['effects'])
      .every((effect) => _closed.contains(_effectSection(layer, effect['id'])));
}
