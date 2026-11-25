import 'package:flutter/material.dart';

import '../app/app_scope.dart';
import '../bridge/link/model.dart' as dto;
import '../design/components/empty_state.dart';
import '../design/components/error_state.dart';
import '../design/components/ls_banner.dart';
import '../design/status.dart';
import '../design/tokens/colors.g.dart';
import '../design/tokens/size.g.dart';
import '../design/tokens/space.g.dart';
import '../design/tokens/type.g.dart';
import '../l10n/generated/app_localizations.dart';
import 'command_line.dart';

/// The one switch from the connection to the full-area states. While the
/// daemon cannot be reached it shows what is wrong (and the CLI line that
/// fixes it); with a cached state it keeps the child under a stale banner;
/// online it is the child.
class ConnectionGate extends StatelessWidget {
  const ConnectionGate({super.key, required this.child});

  final Widget child;

  @override
  Widget build(BuildContext context) {
    final controller = AppScope.of(context);
    final state = controller.state;
    final l = AppLocalizations.of(context);
    final cached = state.status != null || state.games.isNotEmpty;
    return switch (state.connection) {
      dto.ConnectionState_Online() => child,
      dto.ConnectionState_Connecting() ||
      dto.ConnectionState_NotRunning() when cached => Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          Padding(
            padding: const EdgeInsets.all(LsSpace.lg),
            child: LsBanner(tone: LsStatus.offline, message: l.homeStale),
          ),
          Expanded(child: child),
        ],
      ),
      dto.ConnectionState_Connecting() => _Centered(
        children: [Text(l.homeConnecting, style: _body(context))],
      ),
      dto.ConnectionState_NotRunning() => _Centered(
        children: [
          ErrorState(
            title: l.homeOfflineDaemonTitle,
            body: l.homeOfflineDaemonBody,
          ),
          const CommandLine(command: 'lemusync service start'),
          _Retry(l.actionRetry, controller.refresh),
        ],
      ),
      dto.ConnectionState_Unsupported() => _Centered(
        children: [
          EmptyState(
            title: l.homeUnsupportedTitle,
            body: l.homeUnsupportedBody,
          ),
        ],
      ),
      dto.ConnectionState_Refused(:final message) => _Centered(
        children: [
          ErrorState(title: l.homeRefusedTitle, body: l.homeRefusedBody),
          _Detail(message),
        ],
      ),
      dto.ConnectionState_Failed(:final code, :final message) => _Centered(
        children: [
          ErrorState(title: l.homeFailedTitle, body: l.homeFailedBody(code)),
          const CommandLine(command: 'lemusync logs'),
          _Detail(message),
          _Retry(l.actionRetry, controller.refresh),
        ],
      ),
      dto.ConnectionState_Unsafe() => _Centered(
        children: [
          ErrorState(title: l.homeUnsafeTitle, body: l.homeUnsafeBody),
        ],
      ),
    };
  }

  static TextStyle _body(BuildContext context) => LsType.body.copyWith(
    color: Theme.of(context).extension<LsColors>()!.onSurfaceMuted,
  );
}

/// Retry sits after the command line it follows: read the fix, then retry.
class _Retry extends StatelessWidget {
  const _Retry(this.label, this.onPressed);

  final String label;
  final VoidCallback onPressed;

  @override
  Widget build(BuildContext context) => Center(
    child: FilledButton(onPressed: onPressed, child: Text(label)),
  );
}

/// The daemon's own words: secondary detail, never the lead.
class _Detail extends StatelessWidget {
  const _Detail(this.message);

  final String message;

  @override
  Widget build(BuildContext context) {
    final c = Theme.of(context).extension<LsColors>()!;
    return SelectableText(
      message,
      textAlign: TextAlign.center,
      style: LsType.caption.copyWith(color: c.onSurfaceMuted),
    );
  }
}

class _Centered extends StatelessWidget {
  const _Centered({required this.children});

  final List<Widget> children;

  @override
  Widget build(BuildContext context) {
    return Center(
      child: SingleChildScrollView(
        padding: const EdgeInsets.all(LsSpace.lg),
        child: ConstrainedBox(
          constraints: const BoxConstraints(maxWidth: LsSize.breakpointMedium),
          child: Column(
            mainAxisSize: MainAxisSize.min,
            crossAxisAlignment: CrossAxisAlignment.stretch,
            spacing: LsSpace.md,
            children: children,
          ),
        ),
      ),
    );
  }
}
