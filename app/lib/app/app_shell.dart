import 'package:flutter/material.dart';

import '../design/tokens/size.g.dart';
import '../l10n/generated/app_localizations.dart';
import 'app_scope.dart';
import 'destination.dart';
import 'shortcuts.dart';

/// The destinations as bottom bar, rail or sidebar by width (navigation.md,
/// Adaptive containers). Only the active body is built.
class AppShell extends StatelessWidget {
  const AppShell({
    super.key,
    required this.current,
    required this.onSelect,
    required this.body,
  });

  final Destination current;
  final ValueChanged<Destination> onSelect;
  final Widget body;

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context);
    final width = MediaQuery.sizeOf(context).width;
    void select(int i) => onSelect(Destination.values[i]);
    final content = FocusTraversalOrder(
      order: const NumericFocusOrder(1),
      child: FocusTraversalGroup(child: body),
    );

    final Widget layout;
    if (width < LsSize.breakpointMedium) {
      layout = Column(
        children: [
          Expanded(child: content),
          FocusTraversalOrder(
            order: const NumericFocusOrder(0),
            child: NavigationBar(
              selectedIndex: current.index,
              onDestinationSelected: select,
              destinations: [
                for (final d in Destination.values)
                  NavigationDestination(
                    icon: Icon(d.icon),
                    label: d.label(l10n),
                  ),
              ],
            ),
          ),
        ],
      );
    } else {
      final extended = width >= LsSize.breakpointExpanded;
      layout = Row(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          FocusTraversalOrder(
            order: const NumericFocusOrder(0),
            child: NavigationRail(
              extended: extended,
              labelType: extended
                  ? NavigationRailLabelType.none
                  : NavigationRailLabelType.all,
              selectedIndex: current.index,
              onDestinationSelected: select,
              destinations: [
                for (final d in Destination.values)
                  NavigationRailDestination(
                    icon: Icon(d.icon),
                    label: Text(d.label(l10n)),
                  ),
              ],
            ),
          ),
          Expanded(child: content),
        ],
      );
    }

    return Shortcuts(
      shortcuts: {...destinationShortcuts, ...syncShortcuts},
      child: Actions(
        actions: {
          SyncNowIntent: CallbackAction<SyncNowIntent>(
            onInvoke: (_) {
              // No-op unless the daemon is up; never an error dialog.
              final c = AppScope.maybeRead(context);
              if (c != null && c.state.online) c.syncNow();
              return null;
            },
          ),
          DestinationIntent: CallbackAction<DestinationIntent>(
            onInvoke: (intent) {
              onSelect(intent.destination);
              return null;
            },
          ),
        },
        child: FocusTraversalGroup(
          policy: OrderedTraversalPolicy(),
          child: Focus(
            autofocus: true,
            skipTraversal: true,
            child: Scaffold(body: layout),
          ),
        ),
      ),
    );
  }
}
