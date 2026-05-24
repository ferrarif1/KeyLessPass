import 'dart:io';

import 'package:flutter/services.dart';

class NativePlatformApi {
  const NativePlatformApi._();

  static const _channel = MethodChannel('keylesspass/native');

  static Future<String?> chooseUsbDirectory() async {
    if (!Platform.isMacOS) return null;
    final path = await _channel.invokeMethod<String>('chooseUsbDirectory');
    if (path == null || path.trim().isEmpty) return null;
    return path;
  }
}
