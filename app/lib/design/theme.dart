import 'package:flutter/material.dart';

import 'tokens/colors.g.dart';
import 'tokens/opacity.g.dart';
import 'tokens/shape.g.dart';
import 'tokens/size.g.dart';
import 'tokens/space.g.dart';
import 'tokens/type.g.dart';

/// Builds the Material theme of one color set. Every value is a token.
ThemeData lsTheme(LsColors c) {
  final scheme = ColorScheme(
    brightness: c.brightness,
    primary: c.accent,
    onPrimary: c.onAccent,
    secondary: c.accent,
    onSecondary: c.onAccent,
    secondaryContainer: c.accent,
    onSecondaryContainer: c.onAccent,
    error: c.danger,
    onError: c.onDanger,
    surface: c.surface,
    onSurface: c.onSurface,
    onSurfaceVariant: c.onSurfaceMuted,
    outline: c.outline,
    outlineVariant: c.bevelDark,
    surfaceContainerLowest: c.surfaceSunken,
    surfaceContainerLow: c.surfaceSunken,
    surfaceContainer: c.surface,
    surfaceContainerHigh: c.surfaceRaised,
    surfaceContainerHighest: c.surfaceRaised,
  );

  TextStyle s(TextStyle base) => base.copyWith(color: c.onSurface);
  final text = TextTheme(
    displayLarge: s(LsType.display),
    displayMedium: s(LsType.display),
    displaySmall: s(LsType.display),
    headlineLarge: s(LsType.headline),
    headlineMedium: s(LsType.headline),
    headlineSmall: s(LsType.headline),
    titleLarge: s(LsType.title),
    titleMedium: s(LsType.title),
    titleSmall: s(LsType.title),
    bodyLarge: s(LsType.body),
    bodyMedium: s(LsType.body),
    bodySmall: s(LsType.caption),
    labelLarge: s(LsType.label),
    labelMedium: s(LsType.label),
    labelSmall: s(LsType.label),
  );

  const padding = EdgeInsets.symmetric(
    horizontal: LsSpace.lg,
    vertical: LsSpace.sm,
  );
  const minSize = Size.square(LsSize.target);
  final focusSide = BorderSide(
    color: c.focus,
    width: LsSize.focusRing,
    strokeAlign: BorderSide.strokeAlignOutside,
  );
  final shape = RoundedRectangleBorder(
    borderRadius: BorderRadius.circular(LsShape.radiusMd),
  );

  WidgetStateProperty<Color?> overlay(Color base) =>
      WidgetStateProperty.resolveWith((states) {
        if (states.contains(WidgetState.focused)) {
          return base.withValues(alpha: LsOpacity.focus);
        }
        if (states.contains(WidgetState.pressed)) {
          return base.withValues(alpha: LsOpacity.pressed);
        }
        if (states.contains(WidgetState.hovered)) {
          return base.withValues(alpha: LsOpacity.hover);
        }
        return null;
      });

  WidgetStateProperty<BorderSide?> sideWithFocus(BorderSide idle) =>
      WidgetStateProperty.resolveWith(
        (states) => states.contains(WidgetState.focused) ? focusSide : idle,
      );

  final disabledFg = c.onSurface.withValues(alpha: LsOpacity.disabled);
  WidgetStateProperty<Color?> foreground(Color enabled) =>
      WidgetStateProperty.resolveWith(
        (states) =>
            states.contains(WidgetState.disabled) ? disabledFg : enabled,
      );

  ButtonStyle button(Color overlayBase, BorderSide idle) => ButtonStyle(
    minimumSize: const WidgetStatePropertyAll(minSize),
    padding: const WidgetStatePropertyAll(padding),
    elevation: const WidgetStatePropertyAll(0),
    shadowColor: const WidgetStatePropertyAll(Colors.transparent),
    overlayColor: overlay(overlayBase),
    shape: WidgetStatePropertyAll(shape),
    side: sideWithFocus(idle),
  );

  OutlineInputBorder inputBorder(Color color, double width) =>
      OutlineInputBorder(
        borderRadius: BorderRadius.circular(LsShape.radiusMd),
        borderSide: BorderSide(color: color, width: width),
      );

  // Selection never relies on color alone: the selected label is bolder and
  // the selected icon sits on the accent indicator.
  final navLabel = LsType.label.copyWith(color: c.onSurfaceMuted);
  final navLabelSelected = LsType.label.copyWith(
    color: c.onSurface,
    fontWeight: LsType.title.fontWeight,
  );
  final navIcon = IconThemeData(color: c.onSurfaceMuted, size: LsSize.iconMd);
  final navIconSelected = IconThemeData(color: c.onAccent, size: LsSize.iconMd);

  return ThemeData(
    useMaterial3: true,
    brightness: c.brightness,
    colorScheme: scheme,
    scaffoldBackgroundColor: c.surface,
    canvasColor: c.surface,
    focusColor: c.focus.withValues(alpha: LsOpacity.focus),
    hoverColor: c.onSurface.withValues(alpha: LsOpacity.hover),
    visualDensity: VisualDensity.standard,
    splashFactory: NoSplash.splashFactory,
    materialTapTargetSize: MaterialTapTargetSize.padded,
    textTheme: text,
    primaryTextTheme: text.apply(
      bodyColor: c.onAccent,
      displayColor: c.onAccent,
    ),
    extensions: [c],
    pageTransitionsTheme: PageTransitionsTheme(
      builders: {
        for (final p in TargetPlatform.values)
          p: const FadeForwardsPageTransitionsBuilder(),
      },
    ),
    filledButtonTheme: FilledButtonThemeData(
      style: button(c.onAccent, BorderSide.none).copyWith(
        backgroundColor: WidgetStateProperty.resolveWith(
          (states) => states.contains(WidgetState.disabled)
              ? c.onSurface.withValues(alpha: LsOpacity.pressed)
              : c.accent,
        ),
        foregroundColor: foreground(c.onAccent),
      ),
    ),
    outlinedButtonTheme: OutlinedButtonThemeData(
      style: button(
        c.onSurface,
        BorderSide(color: c.outline, width: LsSize.bevel),
      ).copyWith(foregroundColor: foreground(c.onSurface)),
    ),
    textButtonTheme: TextButtonThemeData(
      style: button(
        c.accent,
        BorderSide.none,
      ).copyWith(foregroundColor: foreground(c.accent)),
    ),
    iconButtonTheme: IconButtonThemeData(
      style: button(c.onSurface, BorderSide.none).copyWith(
        padding: const WidgetStatePropertyAll(EdgeInsets.all(LsSpace.md)),
      ),
    ),
    cardTheme: CardThemeData(
      color: c.surfaceRaised,
      elevation: 0,
      shadowColor: Colors.transparent,
      surfaceTintColor: Colors.transparent,
      shape: RoundedRectangleBorder(
        borderRadius: BorderRadius.circular(LsShape.radiusMd),
        side: BorderSide(color: c.outline, width: LsSize.bevel),
      ),
    ),
    inputDecorationTheme: InputDecorationThemeData(
      filled: true,
      fillColor: c.surfaceSunken,
      contentPadding: padding,
      border: inputBorder(c.outline, LsSize.bevel),
      enabledBorder: inputBorder(c.outline, LsSize.bevel),
      focusedBorder: inputBorder(c.focus, LsSize.focusRing),
      errorBorder: inputBorder(c.danger, LsSize.bevel),
      focusedErrorBorder: inputBorder(c.danger, LsSize.focusRing),
      disabledBorder: inputBorder(
        c.outline.withValues(alpha: LsOpacity.disabled),
        LsSize.bevel,
      ),
    ),
    tooltipTheme: TooltipThemeData(
      decoration: BoxDecoration(
        color: c.onSurface,
        borderRadius: BorderRadius.circular(LsShape.radiusSm),
      ),
      textStyle: LsType.caption.copyWith(color: c.surface),
      padding: const EdgeInsets.symmetric(
        horizontal: LsSpace.md,
        vertical: LsSpace.xs,
      ),
    ),
    navigationBarTheme: NavigationBarThemeData(
      height: LsSize.target + LsSpace.lg,
      backgroundColor: c.surfaceSunken,
      surfaceTintColor: Colors.transparent,
      shadowColor: Colors.transparent,
      elevation: 0,
      indicatorColor: c.accent,
      indicatorShape: shape,
      overlayColor: overlay(c.onSurface),
      iconTheme: WidgetStateProperty.resolveWith(
        (states) =>
            states.contains(WidgetState.selected) ? navIconSelected : navIcon,
      ),
      labelTextStyle: WidgetStateProperty.resolveWith(
        (states) =>
            states.contains(WidgetState.selected) ? navLabelSelected : navLabel,
      ),
    ),
    navigationRailTheme: NavigationRailThemeData(
      backgroundColor: c.surfaceSunken,
      elevation: 0,
      indicatorColor: c.accent,
      indicatorShape: shape,
      minWidth: LsSize.railWidth,
      minExtendedWidth: LsSize.sidebarWidth,
      useIndicator: true,
      selectedIconTheme: navIconSelected,
      unselectedIconTheme: navIcon,
      selectedLabelTextStyle: navLabelSelected,
      unselectedLabelTextStyle: navLabel,
    ),
    dividerTheme: DividerThemeData(
      color: c.outline,
      thickness: LsSize.bevel,
      space: LsSize.bevel,
    ),
  );
}
