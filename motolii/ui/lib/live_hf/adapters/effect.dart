import 'package:flutter/widgets.dart';

import '../../hf/insp/panel.dart';
import '../../session/editor_session.dart';
import 'effect_store.dart';

/// One effect's parameters for the Inspector's effect card: the generic Toys over the effect's declared rows.
class NewEffectParams extends StatefulWidget {
  const NewEffectParams({super.key, required this.controller, required this.layerId, required this.effectId, this.advancedOpen = false});
  final EditorSession controller;
  final int layerId;
  final Object effectId;
  final bool advancedOpen;
  @override
  State<NewEffectParams> createState() => _NewEffectParamsState();
}

class _NewEffectParamsState extends State<NewEffectParams> {
  static const _watched = ['layers', 'selectedId', 'selectedIds', 'capabilities', 'documentRevision'];
  EditorSession get c => widget.controller;
  late SessionEffectStore store = SessionEffectStore(c, widget.layerId, widget.effectId);

  @override
  void initState() {
    super.initState();
    c.slice('newEffect', _watched).addListener(store.absorb);
    c.rendered.addListener(store.absorb);
  }

  @override
  void didUpdateWidget(NewEffectParams old) {
    super.didUpdateWidget(old);
    if (old.layerId != widget.layerId || old.effectId != widget.effectId) {
      store.dispose();
      store = SessionEffectStore(c, widget.layerId, widget.effectId);
    } else {
      store.absorb();
    }
  }

  @override
  void dispose() {
    c.slice('newEffect', _watched).removeListener(store.absorb);
    c.rendered.removeListener(store.absorb);
    store.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) => ParamSheet(store, thingId: '${widget.effectId}', advancedOpen: widget.advancedOpen);
}
