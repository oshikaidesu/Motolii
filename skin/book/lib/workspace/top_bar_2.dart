// Part of top_bar.dart (cut there by size, as it was).
part of 'top_bar.dart';

class _WaitedState extends State<_Waited> {
  final _open = OverlayPortalController(), _at = LayerLink();
  late final Timer _look;
  int _count = 0;

  @override
  void initState() {
    super.initState();
    _look = Timer.periodic(const Duration(milliseconds: 500), (_) {   // (read without waiting: the inside's count)
      final n = WsScope.read(context).inside?.brokenCount ?? 0;
      if (n != _count && mounted) setState(() => _count = n);
    });
  }

  @override
  void dispose() {
    _look.cancel();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    if (_count == 0) return const SizedBox.shrink();
    final lines = _open.isShowing ? (WsScope.read(context).inside?.broken ?? '').split('\n').reversed.take(40).toList() : const <String>[];
    return CompositedTransformTarget(
      link: _at,
      child: OverlayPortal(
        controller: _open,
        overlayChildBuilder: (_) => CompositedTransformFollower(
          link: _at,
          targetAnchor: Alignment.bottomRight,
          followerAnchor: Alignment.topRight,
          offset: const Offset(0, 6),
          child: Align(
            alignment: Alignment.topRight,
            child: GestureDetector(
              onTap: () => setState(_open.hide),
              child: Container(
                width: Shape.z(420),
                padding: EdgeInsets.all(Shape.s(10)),
                decoration: BoxDecoration(color: WsT.card, borderRadius: BorderRadius.all(Shape.r(6)), border: Border.all(color: Grey.g26)),
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  mainAxisSize: MainAxisSize.min,
                  children: [for (final l in lines) Text(l, maxLines: 1, overflow: TextOverflow.ellipsis, style: T.micro(Grey.g76))],
                ),
              ),
            ),
          ),
        ),
        child: _Hover(
          onTap: () => setState(_open.toggle),
          builder: (h) => Padding(
            padding: EdgeInsets.only(right: Shape.s(10)),
            child: Row(
              mainAxisSize: MainAxisSize.min,
              children: [
                Container(width: Shape.z(6), height: Shape.z(6), decoration: BoxDecoration(color: ColorTheme.of('Alert', const Color(0xFFE5484D)), shape: BoxShape.circle)),
                SizedBox(width: Shape.s(5)),
                Text('$_count', style: T.label(h ? Grey.g91 : Grey.g63)),
              ],
            ),
          ),
        ),
      ),
    );
  }
}
