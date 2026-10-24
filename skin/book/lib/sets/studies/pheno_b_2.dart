// Part of pheno_b.dart (cut there by size, as it was).
part of 'pheno_b.dart';

class _PhenoCaseState extends State<_PhenoCase> {
  late final (PhenoValues, PhenoSim Function(PhenoValues)) _m = widget.make();

  Widget _well(double? w, double h) => Container(
        width: w,
        height: h,
        decoration: BoxDecoration(color: N.g07, borderRadius: BorderRadius.circular(4), border: Border.all(color: N.g20)),
        child: ClipRRect(borderRadius: BorderRadius.circular(3), child: PhenoCanvas(values: _m.$1, make: _m.$2)),
      );

  Widget _num((String, String) r) => Padding(
        padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
        child: Row(children: [
          SizedBox(width: 62, child: Text(r.$1, maxLines: 1, style: T.label(N.g63))),
          const SizedBox(width: 6),
          Expanded(
            child: Container(
              height: 22,
              alignment: Alignment.centerRight,
              padding: const EdgeInsets.symmetric(horizontal: 8),
              decoration: BoxDecoration(color: N.g07, borderRadius: BorderRadius.circular(4), border: Border.all(color: N.g20)),
              child: Text(r.$2, style: T.value(N.g91)),
            ),
          ),
        ]),
      );

  @override
  Widget build(BuildContext context) => Column(crossAxisAlignment: CrossAxisAlignment.start, mainAxisSize: MainAxisSize.min, children: [
        Text(widget.caption, style: T.label(N.g63)),
        const SizedBox(height: 12),
        Wrap(spacing: 24, runSpacing: 16, crossAxisAlignment: WrapCrossAlignment.start, children: [
          _well(320, 200),
          Container(
            width: 282,
            padding: const EdgeInsets.symmetric(vertical: 4),
            decoration: BoxDecoration(color: N.g13, borderRadius: BorderRadius.circular(4), border: Border.all(color: N.g20)),
            child: Column(mainAxisSize: MainAxisSize.min, children: [
              _num(widget.above),
              Padding(
                padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
                child: Row(crossAxisAlignment: CrossAxisAlignment.start, children: [
                  SizedBox(width: 62, child: Padding(padding: const EdgeInsets.only(top: 5), child: Text(widget.label, maxLines: 1, style: T.label(N.g63)))),
                  const SizedBox(width: 6),
                  Expanded(child: _well(null, widget.rowH)),
                ]),
              ),
              _num(widget.below),
            ]),
          ),
        ]),
      ]);
}
