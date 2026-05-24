import 'package:flutter/material.dart';

class KpColors {
  static const primary = Color(0xfffaff69);
  static const primaryActive = Color(0xffe6eb52);
  static const primaryDisabled = Color(0xff3a3a1f);
  static const canvas = Color(0xff0a0a0a);
  static const surfaceSoft = Color(0xff121212);
  static const surfaceCard = Color(0xff1a1a1a);
  static const surfaceElevated = Color(0xff242424);
  static const hairline = Color(0xff2a2a2a);
  static const hairlineStrong = Color(0xff3a3a3a);
  static const ink = Color(0xffffffff);
  static const body = Color(0xffcccccc);
  static const bodyStrong = Color(0xffe6e6e6);
  static const muted = Color(0xff888888);
  static const mutedSoft = Color(0xff5a5a5a);
  static const success = Color(0xff22c55e);
  static const warning = Color(0xfff59e0b);
  static const error = Color(0xffef4444);
}

ThemeData buildKeylessPassTheme([Brightness brightness = Brightness.dark]) {
  final isDark = brightness == Brightness.dark;
  final base = isDark ? ThemeData.dark(useMaterial3: true) : ThemeData.light(useMaterial3: true);
  const textTheme = TextTheme(
    headlineSmall: TextStyle(fontSize: 32, fontWeight: FontWeight.w700, height: 1.2, color: KpColors.ink),
    titleLarge: TextStyle(fontSize: 24, fontWeight: FontWeight.w700, height: 1.3, color: KpColors.ink),
    titleMedium: TextStyle(fontSize: 18, fontWeight: FontWeight.w600, height: 1.4, color: KpColors.bodyStrong),
    titleSmall: TextStyle(fontSize: 16, fontWeight: FontWeight.w600, height: 1.4, color: KpColors.bodyStrong),
    bodyLarge: TextStyle(fontSize: 16, fontWeight: FontWeight.w400, height: 1.55, color: KpColors.body),
    bodyMedium: TextStyle(fontSize: 14, fontWeight: FontWeight.w400, height: 1.55, color: KpColors.body),
    labelLarge: TextStyle(fontSize: 14, fontWeight: FontWeight.w600, height: 1, color: KpColors.canvas),
    labelMedium: TextStyle(fontSize: 12, fontWeight: FontWeight.w600, height: 1.4, color: KpColors.muted),
  );

  final inputBorder = OutlineInputBorder(
    borderRadius: BorderRadius.circular(8),
    borderSide: const BorderSide(color: KpColors.hairlineStrong),
  );

  return base.copyWith(
    scaffoldBackgroundColor: isDark ? KpColors.canvas : const Color(0xfff7f8f2),
    canvasColor: isDark ? KpColors.canvas : const Color(0xfff7f8f2),
    dividerColor: KpColors.hairline,
    colorScheme: (isDark
            ? const ColorScheme.dark(
                primary: KpColors.primary,
                onPrimary: KpColors.canvas,
                secondary: KpColors.primary,
                surface: KpColors.surfaceCard,
                onSurface: KpColors.ink,
                error: KpColors.error,
              )
            : const ColorScheme.light(
                primary: KpColors.primaryActive,
                onPrimary: KpColors.canvas,
                secondary: KpColors.primaryActive,
                surface: Color(0xffffffff),
                onSurface: Color(0xff111111),
                error: KpColors.error,
              ))
        .copyWith(outline: KpColors.hairlineStrong),
    textTheme: textTheme,
    primaryTextTheme: textTheme,
    filledButtonTheme: FilledButtonThemeData(
      style: FilledButton.styleFrom(
        backgroundColor: KpColors.primary,
        foregroundColor: KpColors.canvas,
        disabledBackgroundColor: KpColors.primaryDisabled,
        disabledForegroundColor: KpColors.muted,
        minimumSize: const Size(0, 40),
        padding: const EdgeInsets.symmetric(horizontal: 18, vertical: 12),
        shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(8)),
        textStyle: const TextStyle(fontSize: 14, fontWeight: FontWeight.w700, height: 1),
      ),
    ),
    outlinedButtonTheme: OutlinedButtonThemeData(
      style: OutlinedButton.styleFrom(
        foregroundColor: KpColors.ink,
        side: const BorderSide(color: KpColors.hairlineStrong),
        minimumSize: const Size(0, 40),
        padding: const EdgeInsets.symmetric(horizontal: 18, vertical: 12),
        shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(8)),
      ),
    ),
    textButtonTheme: TextButtonThemeData(
      style: TextButton.styleFrom(
        foregroundColor: KpColors.ink,
        shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(8)),
      ),
    ),
    iconButtonTheme: IconButtonThemeData(
      style: IconButton.styleFrom(
        foregroundColor: KpColors.ink,
        backgroundColor: KpColors.surfaceCard,
        shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(999)),
      ),
    ),
    inputDecorationTheme: InputDecorationTheme(
      filled: true,
      fillColor: KpColors.surfaceCard,
      labelStyle: const TextStyle(color: KpColors.muted),
      hintStyle: const TextStyle(color: KpColors.mutedSoft),
      enabledBorder: inputBorder,
      focusedBorder: inputBorder.copyWith(borderSide: const BorderSide(color: KpColors.primary)),
      errorBorder: inputBorder.copyWith(borderSide: const BorderSide(color: KpColors.error)),
      focusedErrorBorder: inputBorder.copyWith(borderSide: const BorderSide(color: KpColors.error)),
      contentPadding: const EdgeInsets.symmetric(horizontal: 14, vertical: 12),
    ),
    listTileTheme: ListTileThemeData(
      textColor: KpColors.body,
      iconColor: KpColors.body,
      selectedColor: KpColors.primary,
      shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(8)),
    ),
    navigationRailTheme: const NavigationRailThemeData(
      backgroundColor: KpColors.canvas,
      selectedIconTheme: IconThemeData(color: KpColors.primary),
      unselectedIconTheme: IconThemeData(color: KpColors.muted),
      selectedLabelTextStyle: TextStyle(color: KpColors.ink, fontSize: 14, fontWeight: FontWeight.w600),
      unselectedLabelTextStyle: TextStyle(color: KpColors.muted, fontSize: 14, fontWeight: FontWeight.w500),
      indicatorColor: KpColors.surfaceCard,
    ),
    chipTheme: base.chipTheme.copyWith(
      backgroundColor: KpColors.surfaceCard,
      selectedColor: KpColors.primary,
      side: const BorderSide(color: KpColors.hairlineStrong),
      labelStyle: const TextStyle(color: KpColors.body),
      secondaryLabelStyle: const TextStyle(color: KpColors.canvas),
    ),
    dialogTheme: DialogThemeData(
      backgroundColor: KpColors.surfaceSoft,
      surfaceTintColor: Colors.transparent,
      shape: RoundedRectangleBorder(
        side: const BorderSide(color: KpColors.hairlineStrong),
        borderRadius: BorderRadius.circular(12),
      ),
      titleTextStyle: const TextStyle(fontSize: 22, fontWeight: FontWeight.w700, color: KpColors.ink),
      contentTextStyle: const TextStyle(fontSize: 14, height: 1.5, color: KpColors.body),
    ),
    sliderTheme: base.sliderTheme.copyWith(
      activeTrackColor: KpColors.primary,
      inactiveTrackColor: KpColors.hairlineStrong,
      thumbColor: KpColors.primary,
      overlayColor: KpColors.primary.withAlpha(30),
    ),
    segmentedButtonTheme: SegmentedButtonThemeData(
      style: ButtonStyle(
        backgroundColor: WidgetStateProperty.resolveWith(
          (states) => states.contains(WidgetState.selected) ? KpColors.primary : KpColors.surfaceCard,
        ),
        foregroundColor: WidgetStateProperty.resolveWith(
          (states) => states.contains(WidgetState.selected) ? KpColors.canvas : KpColors.body,
        ),
        side: WidgetStateProperty.all(const BorderSide(color: KpColors.hairlineStrong)),
        shape: WidgetStateProperty.all(RoundedRectangleBorder(borderRadius: BorderRadius.circular(8))),
      ),
    ),
  );
}
