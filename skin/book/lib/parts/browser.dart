// The browser (left of the window): tabs, then sections of tiles. Primitives are quiet; relations carry colour.
import 'package:flutter/widgets.dart';

import '../tokens.dart';
import 'controls.dart';
import 'glyphs.dart';
import 'relations.dart';

class BrowserPanel extends StatelessWidget {
  const BrowserPanel({super.key, this.look = Look.concept, this.tab = 0});
  final Look look;
  final int tab;

  static const tabs = ['Objects', 'Relations', 'Effects', 'Media'];
  static const primitives = ['Text', 'Shape', 'Image', 'Camera'];
  static const generators = ['Repeater', 'Grid', 'Circle', 'Spiral'];

  @override
  Widget build(BuildContext context) => Container(
        width: 330,
        color: N.g10,
        child: SingleChildScrollView(padding: const EdgeInsets.fromLTRB(12, 10, 12, 12), child: Flex(direction: Axis.vertical, crossAxisAlignment: CrossAxisAlignment.start, mainAxisSize: MainAxisSize.min, children: [
          Segmented(items: tabs, index: tab, expand: true),
          const SizedBox(height: 14),
          _Head('Primitives'),
          _Grid([for (final p in primitives) _PlainTile(p)]),
          const SizedBox(height: 14),
          _Head('Generators'),
          _Grid([for (final p in generators) _PlainTile(p)]),
          const SizedBox(height: 14),
          _Head('Relations'),
          _Grid([for (final f in Fam.all) RelationTile(f, look: look, width: 96)]),
        ])),
      );
}

class _Head extends StatelessWidget {
  const _Head(this.text);
  final String text;
  @override
  Widget build(BuildContext context) => Padding(padding: const EdgeInsets.only(bottom: 8), child: Text(text.toUpperCase(), style: T.micro(N.g56).copyWith(letterSpacing: .8)));
}

class _Grid extends StatelessWidget {
  const _Grid(this.children);
  final List<Widget> children;
  @override
  Widget build(BuildContext context) => Wrap(spacing: 8, runSpacing: 8, children: children);
}

class _PlainTile extends StatelessWidget {
  const _PlainTile(this.label);
  final String label;
  static const _kinds = {'Text': G.text, 'Shape': G.shape, 'Image': G.image, 'Camera': G.camera, 'Repeater': G.repeater, 'Grid': G.grid, 'Circle': G.circle, 'Spiral': G.spiral};
  @override
  Widget build(BuildContext context) => Container(
        width: 70, // four to a row in the 306 px browser (4 x 70 + 3 x 8 = 304), so no section ends in an orphan
        height: 64,
        decoration: BoxDecoration(color: N.g13, borderRadius: BorderRadius.circular(8), border: Border.all(color: N.g20)),
        child: Flex(direction: Axis.vertical, mainAxisAlignment: MainAxisAlignment.center, children: [
          Glyph(_kinds[label] ?? G.shape, size: 22, color: N.g76),
          const SizedBox(height: 7),
          Text(label, style: T.label(N.g76)),
        ]),
      );
}
