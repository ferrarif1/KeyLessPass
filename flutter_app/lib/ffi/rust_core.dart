import 'dart:convert';
import 'dart:ffi';
import 'dart:io';

import 'package:ffi/ffi.dart';

typedef _CoreCallNative = Pointer<Utf8> Function(Pointer<Utf8>);
typedef _CoreCallDart = Pointer<Utf8> Function(Pointer<Utf8>);
typedef _CoreFreeNative = Void Function(Pointer<Utf8>);
typedef _CoreFreeDart = void Function(Pointer<Utf8>);

class KeylessPassException implements Exception {
  KeylessPassException(this.message);

  final String message;

  @override
  String toString() => message;
}

class RustCore {
  RustCore._(this._lib)
      : _call = _lib.lookupFunction<_CoreCallNative, _CoreCallDart>(
            'keylesspass_ffi_json'),
        _free = _lib.lookupFunction<_CoreFreeNative, _CoreFreeDart>(
            'keylesspass_ffi_free');

  // Keep the dynamic library handle alive for the lifetime of the function pointers.
  // ignore: unused_field
  final DynamicLibrary _lib;
  final _CoreCallDart _call;
  final _CoreFreeDart _free;

  static RustCore? _instance;

  static RustCore get instance {
    _instance ??= RustCore._(_openLibrary());
    return _instance!;
  }

  Future<T> invoke<T>(String op, Map<String, Object?> payload,
      T Function(Object? value) decode) async {
    final request = jsonEncode({'op': op, 'payload': payload});
    final input = request.toNativeUtf8();
    Pointer<Utf8> output = nullptr;
    try {
      output = _call(input);
      if (output == nullptr) {
        throw KeylessPassException('Rust Core 返回空响应。');
      }
      final decoded = jsonDecode(output.toDartString()) as Map<String, Object?>;
      if (decoded['ok'] != true) {
        throw KeylessPassException((decoded['error'] as String?) ?? '操作失败。');
      }
      return decode(decoded['data']);
    } finally {
      calloc.free(input);
      if (output != nullptr) {
        _free(output);
      }
    }
  }

  static DynamicLibrary _openLibrary() {
    final executableDir = File(Platform.resolvedExecutable).parent;
    final candidates = <String>[
      if (Platform.isMacOS) ...[
        '${executableDir.parent.path}/Frameworks/libkeylesspass_core.dylib',
        '${executableDir.path}/libkeylesspass_core.dylib',
        'libkeylesspass_core.dylib',
        '../rust_core/target/debug/libkeylesspass_core.dylib',
        '../../rust_core/target/debug/libkeylesspass_core.dylib',
        'rust_core/target/debug/libkeylesspass_core.dylib',
      ],
      if (Platform.isLinux) ...[
        '${executableDir.path}/lib/libkeylesspass_core.so',
        '${executableDir.path}/libkeylesspass_core.so',
        'libkeylesspass_core.so',
        '../rust_core/target/debug/libkeylesspass_core.so',
        '../../rust_core/target/debug/libkeylesspass_core.so',
        'rust_core/target/debug/libkeylesspass_core.so',
      ],
      if (Platform.isWindows) ...[
        '${executableDir.path}\\keylesspass_core.dll',
        'keylesspass_core.dll',
        r'..\rust_core\target\debug\keylesspass_core.dll',
        r'..\..\rust_core\target\debug\keylesspass_core.dll',
        r'rust_core\target\debug\keylesspass_core.dll',
      ],
    ];

    Object? lastError;
    for (final path in candidates) {
      try {
        return DynamicLibrary.open(path);
      } catch (error) {
        lastError = error;
      }
    }
    throw KeylessPassException('无法加载 Rust Core 动态库，请先构建 rust_core。$lastError');
  }
}
