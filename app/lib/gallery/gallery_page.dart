import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';

import '../design/design.dart';
import '../design/tokens/colors.g.dart';
import '../design/tokens/space.g.dart';
import '../design/tokens/type.g.dart';
import '../l10n/generated/app_localizations.dart';
import 'component_sections.dart';
import 'token_sections.dart';

/// The four themes the gallery can show, in switcher order.
const galleryThemes = <String, LsColors>{
  'Light': LsColors.light,
  'Dark': LsColors.dark,
  'Light high contrast': LsColors.lightHighContrast,
  'Dark high contrast': LsColors.darkHighContrast,
};

/// The gallery app: owns the theme, locale and overlay switches.
class GalleryApp extends StatefulWidget {
  const GalleryApp({super.key, this.initialTheme = LsColors.light});

  final LsColors initialTheme;

  @override
  State<GalleryApp> createState() => _GalleryAppState();
}

class _GalleryAppState extends State<GalleryApp> {
  late LsColors _colors = widget.initialTheme;
  Locale _locale = const Locale('en');
  bool _overlay = false;

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: 'LemuSync design gallery',
      debugShowCheckedModeBanner: false,
      showPerformanceOverlay: _overlay,
      theme: lsTheme(_colors),
      locale: _locale,
      localizationsDelegates: AppLocalizations.localizationsDelegates,
      supportedLocales: AppLocalizations.supportedLocales,
      home: GalleryPage(
        colors: _colors,
        onColors: (v) => setState(() => _colors = v),
        locale: _locale,
        onLocale: (v) => setState(() => _locale = v),
        overlay: _overlay,
        onOverlay: (v) => setState(() => _overlay = v),
      ),
    );
  }
}

/// One scrolling page with every token and component.
/// The headings are developer-facing English and are not localised; the
/// components keep their own localised strings.
class GalleryPage extends StatelessWidget {
  const GalleryPage({
    super.key,
    required this.colors,
    required this.onColors,
    required this.locale,
    required this.onLocale,
    required this.overlay,
    required this.onOverlay,
  });

  final LsColors colors;
  final ValueChanged<LsColors> onColors;
  final Locale locale;
  final ValueChanged<Locale> onLocale;
  final bool overlay;
  final ValueChanged<bool> onOverlay;

  @override
  Widget build(BuildContext context) {
    final c = Theme.of(context).extension<LsColors>()!;
    final sections = <(String, WidgetBuilder)>[
      ('Color roles', (_) => const ColorRolesSection()),
      ('Type scale', (_) => const TypeScaleSection()),
      ('Space', (_) => const SpaceSection()),
      ('Size', (_) => const SizeSection()),
      ('Shape', (_) => const ShapeSection()),
      ('Opacity', (_) => const OpacitySection()),
      ('Motion', (_) => const MotionSection()),
      ('Icons', (_) => const IconsSection()),
      ('Mark', (_) => const MarkSection()),
      ('Surfaces', (_) => const SurfacesSection()),
      ('Status badge', (_) => const BadgesSection()),
      ('Progress ring', (_) => const RingsSection()),
      ('Status hero', (_) => const HeroesSection()),
      ('Banner', (_) => const BannersSection()),
      ('Game card', (_) => const CardsSection()),
      ('Empty and error states', (_) => const StatesSection()),
    ];
    return Scaffold(
      body: CustomScrollView(
        slivers: [
          SliverAppBar(
            pinned: true,
            title: const Text('Design gallery'),
            actions: [
              PopupMenuButton<LsColors>(
                tooltip: 'Theme',
                icon: const Icon(Icons.palette_outlined),
                onSelected: onColors,
                itemBuilder: (_) => [
                  for (final e in galleryThemes.entries)
                    CheckedPopupMenuItem(
                      value: e.value,
                      checked: e.value == colors,
                      child: Text(e.key),
                    ),
                ],
              ),
              PopupMenuButton<Locale>(
                tooltip: 'Language',
                icon: const Icon(Icons.translate),
                onSelected: onLocale,
                itemBuilder: (_) => [
                  for (final l in AppLocalizations.supportedLocales)
                    CheckedPopupMenuItem(
                      value: l,
                      checked: l == locale,
                      child: Text(l.languageCode),
                    ),
                ],
              ),
              if (!kReleaseMode)
                IconButton(
                  tooltip: 'Performance overlay',
                  isSelected: overlay,
                  icon: const Icon(Icons.speed),
                  onPressed: () => onOverlay(!overlay),
                ),
            ],
          ),
          SliverPadding(
            padding: const EdgeInsets.all(LsSpace.lg),
            sliver: SliverList.separated(
              itemCount: sections.length,
              separatorBuilder: (_, _) => const SizedBox(height: LsSpace.xl),
              itemBuilder: (context, i) {
                final (title, build) = sections[i];
                return Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  spacing: LsSpace.md,
                  children: [
                    Text(
                      title,
                      style: LsType.headline.copyWith(color: c.onSurface),
                    ),
                    build(context),
                  ],
                );
              },
            ),
          ),
        ],
      ),
    );
  }
}
