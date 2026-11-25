import 'package:flutter/material.dart';

import '../design/components/ls_surface.dart';
import '../design/tokens/colors.g.dart';
import '../design/tokens/space.g.dart';
import '../design/tokens/type.g.dart';

/// A CLI command the user can copy and run (P5: everything in the UI is also
/// in the CLI). It is code, so it is never translated, and it is not a
/// button: selectable text on a sunken panel, no ripple, no tap action.
class CommandLine extends StatelessWidget {
  const CommandLine({super.key, required this.command});

  final String command;

  @override
  Widget build(BuildContext context) {
    final c = Theme.of(context).extension<LsColors>()!;
    return LsSurface(
      level: LsSurfaceLevel.sunken,
      padding: const EdgeInsets.symmetric(
        horizontal: LsSpace.md,
        vertical: LsSpace.sm,
      ),
      child: SizedBox(
        width: double.infinity,
        child: SelectableText(
          command,
          style: LsType.mono.copyWith(color: c.onSurface),
        ),
      ),
    );
  }
}
