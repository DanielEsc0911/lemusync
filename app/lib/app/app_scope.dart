import 'package:flutter/widgets.dart';

import 'app_controller.dart';

/// Gives the widget tree the [AppController] and refreshes it when the app
/// comes back to the foreground (the only lifecycle state it reacts to).
class AppScope extends StatefulWidget {
  const AppScope({required this.controller, required this.child, super.key});

  final AppController controller;
  final Widget child;

  /// The controller, rebuilding the caller when the state changes.
  static AppController of(BuildContext context) {
    final scope = context
        .dependOnInheritedWidgetOfExactType<_AppScopeInherited>();
    assert(scope != null, 'No AppScope above this context');
    return scope!.notifier!;
  }

  /// The controller if there is an [AppScope] above, else null.
  static AppController? maybeOf(BuildContext context) => context
      .dependOnInheritedWidgetOfExactType<_AppScopeInherited>()
      ?.notifier;

  @override
  State<AppScope> createState() => _AppScopeState();
}

class _AppScopeState extends State<AppScope> with WidgetsBindingObserver {
  @override
  void initState() {
    super.initState();
    WidgetsBinding.instance.addObserver(this);
  }

  @override
  void dispose() {
    WidgetsBinding.instance.removeObserver(this);
    super.dispose();
  }

  @override
  void didChangeAppLifecycleState(AppLifecycleState state) {
    if (state == AppLifecycleState.resumed) {
      widget.controller.refresh();
    }
  }

  @override
  Widget build(BuildContext context) =>
      _AppScopeInherited(notifier: widget.controller, child: widget.child);
}

class _AppScopeInherited extends InheritedNotifier<AppController> {
  const _AppScopeInherited({required super.notifier, required super.child});
}
