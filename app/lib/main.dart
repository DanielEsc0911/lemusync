import 'package:flutter/material.dart';

import 'app/app_controller.dart';
import 'app/app_scope.dart';
import 'app/app_shell.dart';
import 'app/daemon_link.dart';
import 'app/destination.dart';
import 'bridge/frb_generated.dart';
import 'design/components/empty_state.dart';
import 'design/licenses.dart';
import 'design/theme.dart';
import 'design/tokens/colors.g.dart';
import 'l10n/generated/app_localizations.dart';
import 'screens/home/home_screen.dart';

Future<void> main() async {
  WidgetsFlutterBinding.ensureInitialized();
  await RustLib.init();
  registerBrandLicenses();
  runApp(LemuSyncApp(link: FrbDaemonLink()));
}

/// Root widget: owns the [AppController] built over [link] and the current
/// destination.
class LemuSyncApp extends StatefulWidget {
  const LemuSyncApp({super.key, required this.link, this.locale});

  final DaemonLink link;

  /// Forces a locale (tests, user override). Null follows the OS.
  final Locale? locale;

  @override
  State<LemuSyncApp> createState() => _LemuSyncAppState();
}

class _LemuSyncAppState extends State<LemuSyncApp> {
  late final AppController _controller = AppController(widget.link)..start();
  final ValueNotifier<Destination> _destination = ValueNotifier(
    Destination.home,
  );

  @override
  void dispose() {
    _destination.dispose();
    _controller.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return AppScope(
      controller: _controller,
      child: MaterialApp(
        onGenerateTitle: (context) => AppLocalizations.of(context).appTitle,
        locale: widget.locale,
        theme: lsTheme(LsColors.light),
        darkTheme: lsTheme(LsColors.dark),
        highContrastTheme: lsTheme(LsColors.lightHighContrast),
        highContrastDarkTheme: lsTheme(LsColors.darkHighContrast),
        localizationsDelegates: AppLocalizations.localizationsDelegates,
        supportedLocales: AppLocalizations.supportedLocales,
        home: ValueListenableBuilder<Destination>(
          valueListenable: _destination,
          builder: (context, current, _) => AppShell(
            current: current,
            onSelect: (d) => _destination.value = d,
            body: switch (current) {
              Destination.home => const HomeScreen(),
              // Stub until the Games screen (Task 9).
              Destination.games => Center(
                child: EmptyState(
                  title: current.label(AppLocalizations.of(context)),
                ),
              ),
            },
          ),
        ),
      ),
    );
  }
}
