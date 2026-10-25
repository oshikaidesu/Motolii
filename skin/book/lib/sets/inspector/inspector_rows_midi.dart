// A property keyed from a MIDI file, as om_midi's panel keys After Effects (github.com/otomad/om_midi, ui/Portal.ts):
// the file (the system's own open dialog: Flutter's file_selector), its channel, its BPM (the file's at first, 120 if
// it says none), what of its notes is keyed (om_midi's null object sliders), then Apply.
import 'package:file_selector/file_selector.dart';
import 'package:flutter/widgets.dart';
import 'package:lab_book/sets/inspector/inspector_parts.dart';
import 'package:lab_book/tokens.dart';

/// om_midi's kinds of keys, by the inside's names (midi.h: midi_keys), in its panel's words.
const midiKinds = [('velocity', 'Velocity'), ('pitch', 'Pitch'), ('count', 'Count'), ('on', 'Note on'), ('bool', 'Bool')];

class MidiKeys extends StatefulWidget {
  const MidiKeys({super.key, required this.prop, required this.onDone});
  final String prop;
  final VoidCallback onDone;
  @override
  State<MidiKeys> createState() => _MidiKeysState();
}

class _MidiKeysState extends State<MidiKeys> {
  String? _file, _trouble;
  String _kind = 'count';
  int _channel = -1;
  final _bpm = TextEditingController(text: '120');
  final _f = FocusNode();

  @override
  void dispose() {
    _bpm.dispose();
    _f.dispose();
    super.dispose();
  }

  Future<void> _choose(Doc doc) async {
    final got = await openFile(acceptedTypeGroups: const [XTypeGroup(label: 'MIDI', extensions: ['mid', 'midi'], uniformTypeIdentifiers: ['public.midi-audio'])]);
    if (got == null || !mounted) return;
    setState(() {
      _file = got.path;
      _trouble = null;
      final bpm = doc.midiBpm?.call(got.path) ?? 120;
      _bpm.text = bpm == bpm.roundToDouble() ? '${bpm.round()}' : bpm.toStringAsFixed(2);
    });
  }

  void _apply(Doc doc) {
    final bpm = double.tryParse(_bpm.text.trim());
    if (_file == null || bpm == null || bpm <= 0) return setState(() => _trouble = _file == null ? 'Choose a MIDI file' : 'BPM is a number above 0');
    if (doc.onMidi?.call(widget.prop, _file!, _kind, _channel, bpm) ?? false) return widget.onDone();
    setState(() => _trouble = 'No notes of that channel');
  }

  @override
  Widget build(BuildContext context) {
    final doc = Ctx.of(context).doc;
    Widget line(String label, Widget child) => Padding(
      padding: EdgeInsets.symmetric(vertical: Shape.s(2)),
      child: Row(
        children: [
          SizedBox(width: Shape.z(56), child: Text(label, style: T.label(Grey.g63))),
          Expanded(child: child),
        ],
      ),
    );
    final name = _file?.substring(_file!.lastIndexOf('/') + 1);
    return Padding(
      padding: EdgeInsets.all(Shape.s(8)),
      child: Column(
        mainAxisSize: MainAxisSize.min,
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          line(
            'MIDI',
            Row(
              children: [
                Chip('Choose…', on: false, onTap: () => _choose(doc)),
                SizedBox(width: Shape.s(6)),
                Expanded(child: Text(name ?? 'None', maxLines: 1, overflow: TextOverflow.ellipsis, style: T.name(name == null ? Grey.g56 : Grey.g91))),
              ],
            ),
          ),
          line(
            'Channel',
            Wrap(
              spacing: Shape.s(2),
              runSpacing: Shape.s(2),
              children: [for (var c = -1; c < 16; c++) Chip(c < 0 ? 'Any' : '${c + 1}', on: _channel == c, onTap: () => setState(() => _channel = c))],
            ),
          ),
          line(
            'BPM',
            Container(
              height: Shape.z(20),
              padding: EdgeInsets.symmetric(horizontal: Shape.s(6)),
              alignment: Alignment.centerLeft,
              decoration: BoxDecoration(color: Grey.g15, borderRadius: BorderRadius.all(Shape.r(4))),
              child: EditableText(controller: _bpm, focusNode: _f, style: T.name(Grey.g95), cursorColor: Grey.g95, backgroundCursorColor: Grey.g20, maxLines: 1),
            ),
          ),
          line(
            'Keys',
            Wrap(
              spacing: Shape.s(2),
              runSpacing: Shape.s(2),
              children: [for (final (k, w) in midiKinds) Chip(w, on: _kind == k, onTap: () => setState(() => _kind = k))],
            ),
          ),
          SizedBox(height: Shape.s(4)),
          Row(
            children: [
              Chip('Apply', on: true, onTap: () => _apply(doc)),
              SizedBox(width: Shape.s(8)),
              if (_trouble != null) Expanded(child: Text(_trouble!, style: T.label(Grey.g56))),
            ],
          ),
        ],
      ),
    );
  }
}
