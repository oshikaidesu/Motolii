// text_hand.dart: what a hand does in a field: a click puts the caret, a double click picks a word, a drag picks a
// range, a shift-click stretches it. Flutter's own TextSelectionGestureDetectorBuilder, wrapped round the field as its
// TextField wraps its EditableText (material/text_field.dart: buildGestureDetector, translucent, the EditableText keyed
// by the delegate and rendererIgnoresPointer). The window's fields are bare EditableTexts, which put the caret and no
// more (golf_noise.md ④: a word not picked by a double click).
import 'package:flutter/widgets.dart';

class TextHand extends StatefulWidget {
  const TextHand({super.key, required this.field});

  /// The field, carrying [key] and rendererIgnoresPointer: true (the hand is this one's, not the field's own).
  final EditableText Function(GlobalKey<EditableTextState> key) field;

  @override
  State<TextHand> createState() => _TextHandState();
}

class _TextHandState extends State<TextHand> implements TextSelectionGestureDetectorBuilderDelegate {
  @override
  final editableTextKey = GlobalKey<EditableTextState>();
  @override
  bool get forcePressEnabled => false;
  @override
  bool get selectionEnabled => true;
  late final _hand = TextSelectionGestureDetectorBuilder(delegate: this);

  @override
  Widget build(BuildContext context) => _hand.buildGestureDetector(behavior: HitTestBehavior.translucent, child: widget.field(editableTextKey));
}

/// The letters picked in a thing's words, kept after the field lets go of the hand (Figma's and Canva's text keep their
/// selection while a colour is chosen for it): which words, and from where to where in them; null, none picked.
final wordsPicked = ValueNotifier<({String text, int start, int end})?>(null);
