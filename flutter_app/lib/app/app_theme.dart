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

@immutable
class KpPalette extends ThemeExtension<KpPalette> {
  const KpPalette({
    required this.primary,
    required this.primaryStrong,
    required this.primaryDisabled,
    required this.canvas,
    required this.sidebar,
    required this.surfaceSoft,
    required this.surfaceCard,
    required this.surfaceElevated,
    required this.hairline,
    required this.hairlineStrong,
    required this.ink,
    required this.body,
    required this.bodyStrong,
    required this.muted,
    required this.mutedSoft,
    required this.success,
    required this.warning,
    required this.error,
    required this.dangerSurface,
  });

  final Color primary;
  final Color primaryStrong;
  final Color primaryDisabled;
  final Color canvas;
  final Color sidebar;
  final Color surfaceSoft;
  final Color surfaceCard;
  final Color surfaceElevated;
  final Color hairline;
  final Color hairlineStrong;
  final Color ink;
  final Color body;
  final Color bodyStrong;
  final Color muted;
  final Color mutedSoft;
  final Color success;
  final Color warning;
  final Color error;
  final Color dangerSurface;

  static const dark = KpPalette(
    primary: KpColors.primary,
    primaryStrong: KpColors.primary,
    primaryDisabled: KpColors.primaryDisabled,
    canvas: KpColors.canvas,
    sidebar: KpColors.canvas,
    surfaceSoft: KpColors.surfaceSoft,
    surfaceCard: KpColors.surfaceCard,
    surfaceElevated: KpColors.surfaceElevated,
    hairline: KpColors.hairline,
    hairlineStrong: KpColors.hairlineStrong,
    ink: KpColors.ink,
    body: KpColors.body,
    bodyStrong: KpColors.bodyStrong,
    muted: KpColors.muted,
    mutedSoft: KpColors.mutedSoft,
    success: KpColors.success,
    warning: KpColors.warning,
    error: KpColors.error,
    dangerSurface: KpColors.surfaceCard,
  );

  static const light = KpPalette(
    primary: Color(0xffe4ed4d),
    primaryStrong: Color(0xff5f6700),
    primaryDisabled: Color(0xffe7e9d2),
    canvas: Color(0xfff6f7f0),
    sidebar: Color(0xfff0f2e8),
    surfaceSoft: Color(0xfff8f9f3),
    surfaceCard: Color(0xffffffff),
    surfaceElevated: Color(0xffeef1e5),
    hairline: Color(0xffdde2cf),
    hairlineStrong: Color(0xffc8cfb7),
    ink: Color(0xff171812),
    body: Color(0xff3f4238),
    bodyStrong: Color(0xff25271f),
    muted: Color(0xff6f7467),
    mutedSoft: Color(0xffa5ac99),
    success: Color(0xff15803d),
    warning: Color(0xffb45309),
    error: Color(0xffdc2626),
    dangerSurface: Color(0xfffff7f7),
  );

  Color resolveTone(Color tone) {
    if (tone == KpColors.primary || tone == KpColors.primaryActive) {
      return primaryStrong;
    }
    if (tone == KpColors.success) return success;
    if (tone == KpColors.warning) return warning;
    if (tone == KpColors.error) return error;
    return tone;
  }

  @override
  KpPalette copyWith({
    Color? primary,
    Color? primaryStrong,
    Color? primaryDisabled,
    Color? canvas,
    Color? sidebar,
    Color? surfaceSoft,
    Color? surfaceCard,
    Color? surfaceElevated,
    Color? hairline,
    Color? hairlineStrong,
    Color? ink,
    Color? body,
    Color? bodyStrong,
    Color? muted,
    Color? mutedSoft,
    Color? success,
    Color? warning,
    Color? error,
    Color? dangerSurface,
  }) {
    return KpPalette(
      primary: primary ?? this.primary,
      primaryStrong: primaryStrong ?? this.primaryStrong,
      primaryDisabled: primaryDisabled ?? this.primaryDisabled,
      canvas: canvas ?? this.canvas,
      sidebar: sidebar ?? this.sidebar,
      surfaceSoft: surfaceSoft ?? this.surfaceSoft,
      surfaceCard: surfaceCard ?? this.surfaceCard,
      surfaceElevated: surfaceElevated ?? this.surfaceElevated,
      hairline: hairline ?? this.hairline,
      hairlineStrong: hairlineStrong ?? this.hairlineStrong,
      ink: ink ?? this.ink,
      body: body ?? this.body,
      bodyStrong: bodyStrong ?? this.bodyStrong,
      muted: muted ?? this.muted,
      mutedSoft: mutedSoft ?? this.mutedSoft,
      success: success ?? this.success,
      warning: warning ?? this.warning,
      error: error ?? this.error,
      dangerSurface: dangerSurface ?? this.dangerSurface,
    );
  }

  @override
  KpPalette lerp(ThemeExtension<KpPalette>? other, double t) {
    if (other is! KpPalette) return this;
    return KpPalette(
      primary: Color.lerp(primary, other.primary, t)!,
      primaryStrong: Color.lerp(primaryStrong, other.primaryStrong, t)!,
      primaryDisabled: Color.lerp(primaryDisabled, other.primaryDisabled, t)!,
      canvas: Color.lerp(canvas, other.canvas, t)!,
      sidebar: Color.lerp(sidebar, other.sidebar, t)!,
      surfaceSoft: Color.lerp(surfaceSoft, other.surfaceSoft, t)!,
      surfaceCard: Color.lerp(surfaceCard, other.surfaceCard, t)!,
      surfaceElevated: Color.lerp(surfaceElevated, other.surfaceElevated, t)!,
      hairline: Color.lerp(hairline, other.hairline, t)!,
      hairlineStrong: Color.lerp(hairlineStrong, other.hairlineStrong, t)!,
      ink: Color.lerp(ink, other.ink, t)!,
      body: Color.lerp(body, other.body, t)!,
      bodyStrong: Color.lerp(bodyStrong, other.bodyStrong, t)!,
      muted: Color.lerp(muted, other.muted, t)!,
      mutedSoft: Color.lerp(mutedSoft, other.mutedSoft, t)!,
      success: Color.lerp(success, other.success, t)!,
      warning: Color.lerp(warning, other.warning, t)!,
      error: Color.lerp(error, other.error, t)!,
      dangerSurface: Color.lerp(dangerSurface, other.dangerSurface, t)!,
    );
  }
}

extension KpThemeTokens on BuildContext {
  KpPalette get kp => Theme.of(this).extension<KpPalette>() ?? KpPalette.dark;
}

ThemeData buildKeylessPassTheme([Brightness brightness = Brightness.dark]) {
  final isDark = brightness == Brightness.dark;
  final base = isDark
      ? ThemeData.dark(useMaterial3: true)
      : ThemeData.light(useMaterial3: true);
  final colors = isDark ? KpPalette.dark : KpPalette.light;
  final textTheme = TextTheme(
    headlineSmall: TextStyle(
        fontSize: 32,
        fontWeight: FontWeight.w700,
        height: 1.2,
        color: colors.ink),
    titleLarge: TextStyle(
        fontSize: 24,
        fontWeight: FontWeight.w700,
        height: 1.3,
        color: colors.ink),
    titleMedium: TextStyle(
        fontSize: 18,
        fontWeight: FontWeight.w600,
        height: 1.4,
        color: colors.bodyStrong),
    titleSmall: TextStyle(
        fontSize: 16,
        fontWeight: FontWeight.w600,
        height: 1.4,
        color: colors.bodyStrong),
    bodyLarge: TextStyle(
        fontSize: 16,
        fontWeight: FontWeight.w400,
        height: 1.55,
        color: colors.body),
    bodyMedium: TextStyle(
        fontSize: 14,
        fontWeight: FontWeight.w400,
        height: 1.55,
        color: colors.body),
    bodySmall: TextStyle(
        fontSize: 13,
        fontWeight: FontWeight.w400,
        height: 1.45,
        color: colors.muted),
    labelLarge: TextStyle(
        fontSize: 14,
        fontWeight: FontWeight.w600,
        height: 1,
        color: colors.ink),
    labelMedium: TextStyle(
        fontSize: 12,
        fontWeight: FontWeight.w600,
        height: 1.4,
        color: colors.muted),
    labelSmall: TextStyle(
        fontSize: 11,
        fontWeight: FontWeight.w600,
        height: 1.3,
        color: colors.muted),
  );

  final inputBorder = OutlineInputBorder(
    borderRadius: BorderRadius.circular(8),
    borderSide: BorderSide(color: colors.hairlineStrong),
  );

  return base.copyWith(
    extensions: <ThemeExtension<dynamic>>[colors],
    scaffoldBackgroundColor: colors.canvas,
    canvasColor: colors.canvas,
    dividerColor: colors.hairline,
    colorScheme: (isDark
            ? ColorScheme.dark(
                primary: colors.primary,
                onPrimary: colors.canvas,
                secondary: colors.primary,
                surface: colors.surfaceCard,
                onSurface: colors.ink,
                error: colors.error,
              )
            : ColorScheme.light(
                primary: colors.primaryStrong,
                onPrimary: colors.surfaceCard,
                secondary: colors.primaryStrong,
                surface: colors.surfaceCard,
                onSurface: colors.ink,
                error: colors.error,
              ))
        .copyWith(
            outline: colors.hairlineStrong,
            surfaceContainerHighest: colors.surfaceElevated),
    textTheme: textTheme,
    primaryTextTheme: textTheme,
    filledButtonTheme: FilledButtonThemeData(
      style: FilledButton.styleFrom(
        backgroundColor: colors.primary,
        foregroundColor: KpColors.canvas,
        disabledBackgroundColor: colors.primaryDisabled,
        disabledForegroundColor: colors.muted,
        minimumSize: const Size(0, 40),
        padding: const EdgeInsets.symmetric(horizontal: 18, vertical: 12),
        shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(8)),
        textStyle: const TextStyle(
            fontSize: 14, fontWeight: FontWeight.w700, height: 1),
      ),
    ),
    outlinedButtonTheme: OutlinedButtonThemeData(
      style: OutlinedButton.styleFrom(
        foregroundColor: colors.bodyStrong,
        side: BorderSide(color: colors.hairlineStrong),
        minimumSize: const Size(0, 40),
        padding: const EdgeInsets.symmetric(horizontal: 18, vertical: 12),
        shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(8)),
      ),
    ),
    textButtonTheme: TextButtonThemeData(
      style: TextButton.styleFrom(
        foregroundColor: colors.bodyStrong,
        shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(8)),
      ),
    ),
    iconButtonTheme: IconButtonThemeData(
      style: IconButton.styleFrom(
        foregroundColor: colors.bodyStrong,
        backgroundColor: colors.surfaceElevated,
        shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(999)),
      ),
    ),
    inputDecorationTheme: InputDecorationTheme(
      filled: true,
      fillColor: colors.surfaceCard,
      labelStyle: TextStyle(color: colors.muted),
      hintStyle: TextStyle(color: colors.mutedSoft),
      enabledBorder: inputBorder,
      focusedBorder: inputBorder.copyWith(
          borderSide: BorderSide(color: colors.primaryStrong)),
      errorBorder:
          inputBorder.copyWith(borderSide: BorderSide(color: colors.error)),
      focusedErrorBorder:
          inputBorder.copyWith(borderSide: BorderSide(color: colors.error)),
      contentPadding: const EdgeInsets.symmetric(horizontal: 14, vertical: 12),
    ),
    listTileTheme: ListTileThemeData(
      textColor: colors.body,
      iconColor: colors.body,
      selectedColor: colors.primaryStrong,
      shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(8)),
    ),
    navigationRailTheme: NavigationRailThemeData(
      backgroundColor: colors.sidebar,
      selectedIconTheme: IconThemeData(color: colors.primaryStrong),
      unselectedIconTheme: IconThemeData(color: colors.muted),
      selectedLabelTextStyle: TextStyle(
          color: colors.ink, fontSize: 14, fontWeight: FontWeight.w600),
      unselectedLabelTextStyle: TextStyle(
          color: colors.muted, fontSize: 14, fontWeight: FontWeight.w500),
      indicatorColor: colors.surfaceElevated,
    ),
    chipTheme: base.chipTheme.copyWith(
      backgroundColor: colors.surfaceCard,
      selectedColor: colors.primary,
      side: BorderSide(color: colors.hairlineStrong),
      labelStyle: TextStyle(color: colors.body),
      secondaryLabelStyle: const TextStyle(color: KpColors.canvas),
    ),
    dialogTheme: DialogThemeData(
      backgroundColor: colors.surfaceSoft,
      surfaceTintColor: Colors.transparent,
      shape: RoundedRectangleBorder(
        side: BorderSide(color: colors.hairlineStrong),
        borderRadius: BorderRadius.circular(12),
      ),
      titleTextStyle: TextStyle(
          fontSize: 22, fontWeight: FontWeight.w700, color: colors.ink),
      contentTextStyle:
          TextStyle(fontSize: 14, height: 1.5, color: colors.body),
    ),
    sliderTheme: base.sliderTheme.copyWith(
      activeTrackColor: colors.primaryStrong,
      inactiveTrackColor: colors.hairlineStrong,
      thumbColor: colors.primary,
      overlayColor: colors.primary.withAlpha(45),
    ),
    switchTheme: SwitchThemeData(
      thumbColor: WidgetStateProperty.resolveWith(
        (states) => states.contains(WidgetState.selected)
            ? colors.primary
            : colors.surfaceCard,
      ),
      trackColor: WidgetStateProperty.resolveWith(
        (states) => states.contains(WidgetState.selected)
            ? colors.primaryStrong.withAlpha(110)
            : colors.hairlineStrong,
      ),
    ),
    segmentedButtonTheme: SegmentedButtonThemeData(
      style: ButtonStyle(
        backgroundColor: WidgetStateProperty.resolveWith(
          (states) => states.contains(WidgetState.selected)
              ? colors.primary
              : colors.surfaceCard,
        ),
        foregroundColor: WidgetStateProperty.resolveWith(
          (states) => states.contains(WidgetState.selected)
              ? KpColors.canvas
              : colors.body,
        ),
        side: WidgetStateProperty.all(BorderSide(color: colors.hairlineStrong)),
        shape: WidgetStateProperty.all(
            RoundedRectangleBorder(borderRadius: BorderRadius.circular(8))),
      ),
    ),
  );
}
