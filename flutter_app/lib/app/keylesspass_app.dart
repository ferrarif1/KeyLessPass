import 'dart:async';
import 'dart:convert';
import 'dart:io';

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_localizations/flutter_localizations.dart';

import '../ffi/core_api.dart';
import '../ffi/rust_core.dart';
import '../l10n/generated/app_localizations.dart';
import '../models/core_models.dart';
import '../platform/native_platform_api.dart';
import '../widgets/desktop_widgets.dart';
import 'app_theme.dart';

enum _Section {
  dashboard,
  setup,
  records,
  add,
  derive,
  rotation,
  usb,
  security,
  settings,
  about
}

enum _RecordFilter { all, active, pending, retired, conflict, error }

enum _SetupMode { create, recover }

enum _RecoveryMode { usbLost, newDevice, resetMnemonic }

class _RecordVersionGroup {
  const _RecordVersionGroup({required this.current, this.previous});

  final CredentialRecord current;
  final CredentialRecord? previous;
}

class _DetailItem {
  const _DetailItem(this.label, this.value);

  final String label;
  final String value;
}

List<_RecordVersionGroup> _groupRecordVersions(List<CredentialRecord> records) {
  final byId = <String, List<CredentialRecord>>{};
  for (final record in records) {
    byId.putIfAbsent(record.recordId, () => []).add(record);
  }
  final groups = <_RecordVersionGroup>[];
  for (final versions in byId.values) {
    versions.sort((a, b) => b.version.compareTo(a.version));
    final current = versions.firstWhere(
      (record) => record.state != 'retired',
      orElse: () => versions.first,
    );
    CredentialRecord? previous;
    for (final record in versions) {
      if (record.version < current.version) {
        previous = record;
        break;
      }
    }
    groups.add(_RecordVersionGroup(current: current, previous: previous));
  }
  groups.sort((a, b) => a.current.recordSeq.compareTo(b.current.recordSeq));
  return groups;
}

class _ResponsiveGrid extends StatelessWidget {
  const _ResponsiveGrid({
    required this.children,
    this.minItemWidth = 220,
    this.maxColumns = 3,
    this.spacing = 12,
  });

  final List<Widget> children;
  final double minItemWidth;
  final int maxColumns;
  final double spacing;

  @override
  Widget build(BuildContext context) {
    return LayoutBuilder(
      builder: (context, constraints) {
        final width = constraints.maxWidth;
        if (!width.isFinite || width <= 0) {
          return Column(
            crossAxisAlignment: CrossAxisAlignment.stretch,
            children: [
              for (final child in children) ...[
                child,
                if (child != children.last) SizedBox(height: spacing),
              ],
            ],
          );
        }

        final columns = ((width + spacing) / (minItemWidth + spacing))
            .floor()
            .clamp(1, maxColumns)
            .toInt();
        final itemWidth = (width - spacing * (columns - 1)) / columns;
        return Wrap(
          spacing: spacing,
          runSpacing: spacing,
          children: [
            for (final child in children)
              SizedBox(width: itemWidth, child: child),
          ],
        );
      },
    );
  }
}

class _ActionButton extends StatelessWidget {
  const _ActionButton({required this.child});

  final Widget child;

  @override
  Widget build(BuildContext context) {
    return SizedBox(width: double.infinity, child: child);
  }
}

extension _Text on BuildContext {
  AppLocalizations get t => AppLocalizations.of(this);
}

class KeylessPassDesktopApp extends StatefulWidget {
  const KeylessPassDesktopApp({super.key});

  @override
  State<KeylessPassDesktopApp> createState() => _KeylessPassDesktopAppState();
}

class _KeylessPassDesktopAppState extends State<KeylessPassDesktopApp> {
  Locale? _locale = _initialLocaleFromEnvironment();
  ThemeMode _themeMode = ThemeMode.dark;

  void _setLocale(Locale? locale) => setState(() => _locale = locale);
  void _setThemeMode(ThemeMode mode) => setState(() => _themeMode = mode);

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      debugShowCheckedModeBanner: false,
      title: 'KeyLessPass',
      locale: _locale,
      supportedLocales: AppLocalizations.supportedLocales,
      localizationsDelegates: const [
        AppLocalizations.delegate,
        GlobalMaterialLocalizations.delegate,
        GlobalCupertinoLocalizations.delegate,
        GlobalWidgetsLocalizations.delegate,
      ],
      themeMode: _themeMode,
      darkTheme: buildKeylessPassTheme(Brightness.dark),
      theme: buildKeylessPassTheme(Brightness.light),
      home: _HomeWindow(
        locale: _locale,
        themeMode: _themeMode,
        onLocaleChanged: _setLocale,
        onThemeModeChanged: _setThemeMode,
      ),
    );
  }
}

Locale? _initialLocaleFromEnvironment() {
  switch (Platform.environment['KEYLESSPASS_LOCALE']?.trim().toLowerCase()) {
    case 'en':
    case 'en_us':
    case 'english':
      return const Locale('en');
    case 'zh':
    case 'zh_cn':
    case 'simplifiedchinese':
      return const Locale('zh');
    default:
      return null;
  }
}

class _HomeWindow extends StatefulWidget {
  const _HomeWindow({
    required this.locale,
    required this.themeMode,
    required this.onLocaleChanged,
    required this.onThemeModeChanged,
  });

  final Locale? locale;
  final ThemeMode themeMode;
  final ValueChanged<Locale?> onLocaleChanged;
  final ValueChanged<ThemeMode> onThemeModeChanged;

  @override
  State<_HomeWindow> createState() => _HomeWindowState();
}

class _HomeWindowState extends State<_HomeWindow> {
  CoreApi? _api;
  _Section _section = _initialSectionFromEnvironment();
  _RecordFilter _filter = _RecordFilter.all;
  AppStatus? _status;
  List<CredentialRecord> _records = [];
  List<UsbCandidate> _usbCandidates = [];
  UsbCdrStatus? _usbCdrStatus;
  CredentialRecord? _selected;
  String _search = '';
  String? _message;
  bool _busy = false;
  Timer? _usbPoller;
  int _clipboardTimeout = 30;
  int _defaultLength = 18;
  bool _advancedMode = false;
  String _passwordDerivationAlgorithm = 'hkdf-sha256';

  @override
  void initState() {
    super.initState();
    try {
      _api = CoreApi(RustCore.instance);
      _refresh();
      _usbPoller = Timer.periodic(const Duration(seconds: 8), (_) {
        if (mounted && !_busy) {
          _refresh(silent: true);
        }
      });
    } catch (_) {
      _message = 'core-unavailable';
    }
  }

  @override
  void dispose() {
    _usbPoller?.cancel();
    super.dispose();
  }

  Future<void> _refresh({bool silent = false}) async {
    final api = _api;
    if (api == null) return;
    if (!silent) {
      setState(() {
        _busy = true;
        _message = null;
      });
    }
    try {
      final status = await api.getAppStatus();
      final records =
          status.enrolled ? await api.listCredentials() : <CredentialRecord>[];
      final candidates = await api.listUsbCandidates();
      UsbCdrStatus? usbCdrStatus;
      if (status.enrolled) {
        final readable = candidates
            .where((item) => item.readable && item.rootPath.isNotEmpty);
        if (readable.isNotEmpty) {
          try {
            usbCdrStatus =
                await api.getUsbCdrStatus(usbPath: readable.first.rootPath);
          } catch (_) {
            usbCdrStatus = null;
          }
        }
      }
      setState(() {
        _status = status;
        if (status.enrolled) {
          _passwordDerivationAlgorithm = status.passwordDerivationAlgorithm;
        }
        _records = records;
        _usbCandidates = candidates;
        _usbCdrStatus = usbCdrStatus;
        _selected = _pickSelected(records);
      });
    } catch (_) {
      setState(() => _message = context.t.operationFailed);
    } finally {
      if (mounted && !silent) setState(() => _busy = false);
    }
  }

  CredentialRecord? _pickSelected(List<CredentialRecord> records) {
    final groups = _groupRecordVersions(records);
    if (groups.isEmpty) return null;
    if (_selected == null) return groups.first.current;
    return records.firstWhere(
      (record) =>
          record.recordId == _selected!.recordId &&
          record.version == _selected!.version,
      orElse: () => groups.first.current,
    );
  }

  List<_RecordVersionGroup> get _visibleRecordGroups {
    final query = _search.trim().toLowerCase();
    return _groupRecordVersions(_records).where((group) {
      final record = group.current;
      final previous = group.previous;
      final stateOk = _filter == _RecordFilter.all ||
          _stateMatches(record.state, _filter) ||
          (previous != null && _stateMatches(previous.state, _filter));
      final searchOk = query.isEmpty ||
          record.displayName.toLowerCase().contains(query) ||
          record.serviceHint.toLowerCase().contains(query) ||
          record.accountHint.toLowerCase().contains(query) ||
          record.notes.toLowerCase().contains(query) ||
          (previous?.notes.toLowerCase().contains(query) ?? false);
      return stateOk && searchOk;
    }).toList();
  }

  int get _recordGroupCount => _groupRecordVersions(_records).length;

  bool _stateMatches(String state, _RecordFilter filter) {
    return switch (filter) {
      _RecordFilter.all => true,
      _RecordFilter.active => state == 'active',
      _RecordFilter.pending => state == 'pending_rotation',
      _RecordFilter.retired => state == 'retired',
      _RecordFilter.conflict => state == 'conflict',
      _RecordFilter.error => state == 'error',
    };
  }

  Future<void> _completeSetup() async {
    await _refresh();
    await _syncCdrToFirstUsb(silent: true);
    if (mounted) setState(() => _section = _Section.dashboard);
  }

  String? _defaultUsbPath() {
    final readable = _usbCandidates
        .where((item) => item.readable && item.rootPath.isNotEmpty);
    if (readable.isNotEmpty) return readable.first.rootPath;
    final any = _usbCandidates.where((item) => item.rootPath.isNotEmpty);
    return any.isEmpty ? null : any.first.rootPath;
  }

  Future<void> _syncCdrToFirstUsb({bool silent = false}) async {
    final path = _defaultUsbPath();
    if (path == null) return;
    await _syncCdrToUsb(path, silent: silent);
  }

  Future<void> _syncCdrToUsb(String usbPath, {bool silent = false}) async {
    final api = _api;
    if (api == null || usbPath.trim().isEmpty) return;
    try {
      await api.syncCdrToUsb(usbPath: usbPath.trim());
      await _refresh(silent: silent);
      if (!silent && mounted) {
        setState(() => _message = context.t.cdrSyncedToUsb);
      }
    } catch (_) {
      if (!silent && mounted) {
        setState(() => _message = context.t.operationFailed);
      }
    }
  }

  Future<void> _restoreCdrFromUsb(String usbPath) async {
    final api = _api;
    if (api == null || usbPath.trim().isEmpty) return;
    final t = context.t;
    final confirmed = await showDialog<bool>(
      context: context,
      builder: (dialogContext) => AlertDialog(
        title: Text(t.confirmRestoreCdrTitle),
        content: Text(t.confirmRestoreCdrBody),
        actions: [
          TextButton(
              onPressed: () => Navigator.pop(dialogContext, false),
              child: Text(t.cancel)),
          FilledButton(
              onPressed: () => Navigator.pop(dialogContext, true),
              child: Text(t.restoreLocalFromUsb)),
        ],
      ),
    );
    if (confirmed != true) return;
    try {
      await api.restoreCdrFromUsb(usbPath: usbPath.trim());
      await _refresh();
      if (mounted) setState(() => _message = t.cdrRestoredFromUsb);
    } catch (_) {
      if (mounted) setState(() => _message = t.operationFailed);
    }
  }

  @override
  Widget build(BuildContext context) {
    return CallbackShortcuts(
      bindings: {
        SingleActivator(LogicalKeyboardKey.keyN,
            control: !Platform.isMacOS, meta: Platform.isMacOS): () {
          setState(() => _section = _Section.add);
        },
        SingleActivator(LogicalKeyboardKey.keyD,
            control: !Platform.isMacOS, meta: Platform.isMacOS): () {
          setState(() => _section = _Section.derive);
        },
        SingleActivator(LogicalKeyboardKey.keyR,
            control: !Platform.isMacOS, meta: Platform.isMacOS): () {
          setState(() => _section = _Section.rotation);
        },
      },
      child: Focus(
        autofocus: true,
        child: Scaffold(
          body: Row(
            children: [
              _navigation(),
              const VerticalDivider(width: 1, color: KpColors.hairline),
              Expanded(child: _content()),
            ],
          ),
        ),
      ),
    );
  }

  Widget _navigation() {
    final t = context.t;
    return SizedBox(
      width: 292,
      child: Column(
        children: [
          Padding(
            padding: const EdgeInsets.fromLTRB(18, 18, 18, 12),
            child: Row(
              children: [
                SizedBox(
                  width: 40,
                  height: 40,
                  child: ClipRRect(
                    borderRadius: BorderRadius.circular(8),
                    child: Image.asset(
                      'assets/logo.png',
                      width: 40,
                      height: 40,
                      fit: BoxFit.cover,
                    ),
                  ),
                ),
                const SizedBox(width: 12),
                Expanded(
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      Text(t.appName,
                          style: Theme.of(context).textTheme.titleMedium),
                      Text(t.appSubtitle,
                          overflow: TextOverflow.ellipsis,
                          style: Theme.of(context).textTheme.labelMedium),
                    ],
                  ),
                ),
              ],
            ),
          ),
          Padding(
            padding: const EdgeInsets.fromLTRB(18, 0, 18, 12),
            child: Wrap(
              spacing: 8,
              runSpacing: 8,
              children: [
                StatusPill(
                    label: _status?.enrolled == true
                        ? t.initialized
                        : t.notInitialized,
                    tone: _status?.enrolled == true
                        ? KpColors.success
                        : KpColors.warning),
                StatusPill(
                    label: _status?.securityStatus.degraded == true
                        ? t.reducedProtection
                        : t.platformProtected),
              ],
            ),
          ),
          Expanded(
            child: ListView(
              key: const ValueKey('main-navigation'),
              padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 4),
              children: [
                _navTile(_Section.dashboard, Icons.home_rounded, t.dashboard),
                _navTile(_Section.setup, Icons.verified_user_rounded, t.setup),
                _navTile(_Section.records, Icons.view_list_rounded, t.records),
                _navTile(_Section.usb, Icons.usb_rounded, t.usbDevice),
                _navTile(_Section.security, Icons.security_rounded, t.security),
                _navTile(_Section.settings, Icons.tune_rounded, t.settings),
                _navTile(_Section.about, Icons.info_outline_rounded, t.about),
              ],
            ),
          ),
          Padding(
            padding: const EdgeInsets.all(12),
            child: Row(
              children: [
                Expanded(
                    child: Text(_busy ? '...' : t.localOnly,
                        overflow: TextOverflow.ellipsis,
                        style: Theme.of(context).textTheme.labelMedium)),
                IconButton(
                    tooltip: t.refresh,
                    onPressed: _refresh,
                    icon: const Icon(Icons.refresh_rounded)),
              ],
            ),
          ),
        ],
      ),
    );
  }

  Widget _navTile(_Section section, IconData icon, String label) {
    final selected = _navSelected(section);
    return Padding(
      padding: const EdgeInsets.only(bottom: 4),
      child: Material(
        color: selected ? KpColors.surfaceCard : Colors.transparent,
        borderRadius: BorderRadius.circular(8),
        child: ListTile(
          selected: selected,
          leading: Icon(icon),
          title: Text(label, overflow: TextOverflow.ellipsis),
          onTap: () => setState(() => _section = section),
        ),
      ),
    );
  }

  bool _navSelected(_Section section) {
    if (section == _Section.records) {
      return {
        _Section.records,
        _Section.add,
        _Section.derive,
        _Section.rotation,
      }.contains(_section);
    }
    return _section == section;
  }

  Widget _content() {
    return switch (_section) {
      _Section.dashboard => _dashboard(),
      _Section.setup => _SetupPage(
          status: _status,
          api: _api,
          usbCandidates: _usbCandidates,
          passwordDerivationAlgorithm: _passwordDerivationAlgorithm,
          onDone: _completeSetup),
      _Section.records => _recordsPage(),
      _Section.add => _status?.enrolled == true
          ? _AddRecordPage(
              defaultLength: _defaultLength, onCreate: _createRecord)
          : _SetupPage(
              status: _status,
              api: _api,
              usbCandidates: _usbCandidates,
              passwordDerivationAlgorithm: _passwordDerivationAlgorithm,
              onDone: _completeSetup),
      _Section.derive => _DerivePage(
          records: _records,
          selected: _selected,
          timeoutSeconds: _clipboardTimeout,
          onSelect: (record) => setState(() => _selected = record),
          api: _api),
      _Section.rotation => _RotationPage(
          records: _records,
          selected: _selected,
          defaultLength: _defaultLength,
          api: _api,
          onSelect: (record) => setState(() => _selected = record),
          onDerivePending: (record) => setState(() {
                _selected = record;
                _section = _Section.derive;
              }),
          onDone: () async {
            await _refresh();
            await _syncCdrToFirstUsb(silent: true);
          }),
      _Section.usb => _UsbDevicePage(
          api: _api,
          enrolled: _status?.enrolled == true,
          usbCandidates: _usbCandidates,
          cdrStatus: _usbCdrStatus,
          onRefresh: _refresh,
          onSyncCdrToUsb: _syncCdrToUsb,
          onRestoreCdrFromUsb: _restoreCdrFromUsb,
          onSetup: () => setState(() => _section = _Section.setup),
        ),
      _Section.security => _securityPage(),
      _Section.settings => _settingsPage(),
      _Section.about => _aboutPage(),
    };
  }

  Widget _dashboard() {
    final t = context.t;
    return _page(
      title: t.dashboard,
      subtitle: t.dashboardSubtitle,
      children: [
        _messageBar(),
        if (_usbCdrStatus?.needsAction == true)
          SectionPanel(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text(t.cdrBackupStatus,
                    style: Theme.of(context).textTheme.titleMedium),
                const SizedBox(height: 10),
                InlineNotice(
                    text: t.cdrBackupNeedsAction,
                    icon: Icons.usb_rounded,
                    tone: KpColors.warning),
                const SizedBox(height: 12),
                Wrap(
                  spacing: 12,
                  runSpacing: 12,
                  children: [
                    FilledButton.icon(
                      onPressed: _defaultUsbPath() == null
                          ? null
                          : () => _syncCdrToUsb(_defaultUsbPath()!),
                      icon: const Icon(Icons.upload_file_rounded),
                      label: Text(t.syncLocalToUsb),
                    ),
                    OutlinedButton.icon(
                      onPressed: _defaultUsbPath() == null ||
                              _usbCdrStatus?.status == 'missing' ||
                              _usbCdrStatus?.status == 'invalid'
                          ? null
                          : () => _restoreCdrFromUsb(_defaultUsbPath()!),
                      icon: const Icon(Icons.download_for_offline_rounded),
                      label: Text(t.restoreLocalFromUsb),
                    ),
                  ],
                ),
              ],
            ),
          ),
        _ResponsiveGrid(
          minItemWidth: 210,
          maxColumns: 3,
          spacing: 12,
          children: [
            SignalTile(
                label: t.activeRecords,
                value: '$_recordGroupCount',
                tone: KpColors.primary),
            SignalTile(
                label: t.usbStatus,
                value: _usbCandidates.isEmpty ? t.notFound : t.available,
                tone: _usbCandidates.isEmpty
                    ? KpColors.warning
                    : KpColors.success),
            SignalTile(label: t.integrity, value: t.ok, tone: KpColors.success),
          ],
        ),
        SectionPanel(
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Text(t.quickActions,
                  style: Theme.of(context).textTheme.titleMedium),
              const SizedBox(height: 14),
              _ResponsiveGrid(
                minItemWidth: 190,
                maxColumns: 4,
                spacing: 12,
                children: [
                  _ActionButton(
                    child: FilledButton.icon(
                        onPressed: _status?.enrolled == true
                            ? () => setState(() => _section = _Section.add)
                            : null,
                        icon: const Icon(Icons.add_rounded),
                        label: Text(t.actionAddRecord)),
                  ),
                  _ActionButton(
                    child: OutlinedButton.icon(
                        onPressed: _records.isEmpty
                            ? null
                            : () => setState(() => _section = _Section.derive),
                        icon: const Icon(Icons.password_rounded),
                        label: Text(t.actionDerive)),
                  ),
                  _ActionButton(
                    child: OutlinedButton.icon(
                        onPressed: _records.isEmpty
                            ? null
                            : () =>
                                setState(() => _section = _Section.rotation),
                        icon: const Icon(Icons.rotate_right_rounded),
                        label: Text(t.actionRotate)),
                  ),
                  _ActionButton(
                    child: OutlinedButton.icon(
                        onPressed: () =>
                            setState(() => _section = _Section.usb),
                        icon: const Icon(Icons.settings_backup_restore_rounded),
                        label: Text(t.actionRecovery)),
                  ),
                ],
              ),
            ],
          ),
        ),
        InlineNotice(text: t.safetyReminder, icon: Icons.privacy_tip_outlined),
      ],
    );
  }

  Widget _recordsPage() {
    final t = context.t;
    final groups = _visibleRecordGroups;
    return _page(
      title: t.records,
      subtitle: t.recordsCount(groups.length),
      trailing: FilledButton.icon(
          onPressed: () => setState(() => _section = _Section.add),
          icon: const Icon(Icons.add_rounded),
          label: Text(t.addRecord)),
      children: [
        Row(
          children: [
            Expanded(
              child: TextField(
                decoration: InputDecoration(
                    labelText: t.search,
                    prefixIcon: const Icon(Icons.search_rounded)),
                onChanged: (value) => setState(() => _search = value),
              ),
            ),
            const SizedBox(width: 12),
            SizedBox(
              width: 220,
              child: DropdownButtonFormField<_RecordFilter>(
                initialValue: _filter,
                decoration: InputDecoration(labelText: t.filter),
                items: _RecordFilter.values
                    .map((value) => DropdownMenuItem(
                        value: value, child: Text(_filterLabel(value))))
                    .toList(),
                onChanged: (value) =>
                    setState(() => _filter = value ?? _RecordFilter.all),
              ),
            ),
          ],
        ),
        SectionPanel(
          child: groups.isEmpty
              ? Text(t.noRecords)
              : Column(children: groups.map(_recordRow).toList()),
        ),
      ],
    );
  }

  Widget _recordRow(_RecordVersionGroup group) {
    final t = context.t;
    final record = group.current;
    final previous = group.previous;
    final selected = record.recordId == _selected?.recordId;
    return Padding(
      padding: const EdgeInsets.only(bottom: 12),
      child: Container(
        padding: const EdgeInsets.all(16),
        decoration: BoxDecoration(
          color: selected ? KpColors.surfaceElevated : KpColors.surfaceSoft,
          border: Border.all(
              color: selected ? KpColors.primary : KpColors.hairlineStrong),
          borderRadius: BorderRadius.circular(8),
        ),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Row(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                const Padding(
                  padding: EdgeInsets.only(top: 4),
                  child: Icon(Icons.key_rounded, color: KpColors.primary),
                ),
                const SizedBox(width: 12),
                Expanded(
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      Text(record.displayName,
                          style: Theme.of(context).textTheme.titleMedium),
                      const SizedBox(height: 4),
                      Text(
                        '${record.accountHint.isEmpty ? '-' : record.accountHint} · ${t.version} ${record.version} · ${_stateLabel(record.state)}',
                        style: Theme.of(context).textTheme.bodyMedium,
                      ),
                    ],
                  ),
                ),
                Wrap(
                  spacing: 8,
                  runSpacing: 8,
                  children: [
                    FilledButton.icon(
                        onPressed: () => _deriveRecordVersion(record),
                        icon: const Icon(Icons.password_rounded),
                        label: Text(t.deriveCurrentVersion)),
                    if (previous != null)
                      OutlinedButton.icon(
                          onPressed: () => _deriveRecordVersion(previous),
                          icon: const Icon(Icons.history_rounded),
                          label: Text(t.derivePreviousVersion)),
                    OutlinedButton.icon(
                        onPressed: () => _rotateRecord(record),
                        icon: const Icon(Icons.rotate_right_rounded),
                        label: Text(t.createNewVersion)),
                    IconButton(
                        tooltip: t.editMetadata,
                        onPressed: () => _showEditMetadata(record),
                        icon: const Icon(Icons.edit_rounded)),
                    IconButton(
                        tooltip: t.viewIntegrity,
                        onPressed: () =>
                            setState(() => _section = _Section.security),
                        icon: const Icon(Icons.verified_rounded)),
                  ],
                ),
              ],
            ),
            const SizedBox(height: 14),
            _recordVersionDetails(t.currentVersion, record),
            if (previous != null) ...[
              const SizedBox(height: 10),
              _recordVersionDetails(t.previousVersion, previous),
            ],
            if (_advancedMode) ...[
              const SizedBox(height: 10),
              Text(t.advancedDetails,
                  style: Theme.of(context).textTheme.titleSmall),
              InfoRow(label: t.recordSequence, value: '${record.recordSeq}'),
              InfoRow(label: t.recordId, value: record.recordId),
              InfoRow(label: t.salt, value: record.salt),
              InfoRow(
                  label: t.encodingRule,
                  value:
                      '${record.encodingDescriptor['alphabetProfile'] ?? '-'}'),
            ],
          ],
        ),
      ),
    );
  }

  Widget _recordVersionDetails(String label, CredentialRecord record) {
    final t = context.t;
    return Container(
      padding: const EdgeInsets.all(12),
      decoration: BoxDecoration(
        color: KpColors.surfaceCard,
        border: Border.all(color: KpColors.hairline),
        borderRadius: BorderRadius.circular(8),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Text(label, style: Theme.of(context).textTheme.titleSmall),
          const SizedBox(height: 8),
          _detailWrap([
            _DetailItem(t.displayName, record.displayName),
            _DetailItem(t.serviceHint, record.serviceHint),
            _DetailItem(t.accountHint, record.accountHint),
            _DetailItem(t.state, _stateLabel(record.state)),
            _DetailItem(t.version, '${record.version}'),
            _DetailItem(t.lastUpdated, _shortDate(record.updatedAt)),
            if (record.notes.isNotEmpty) _DetailItem(t.notes, record.notes),
          ]),
        ],
      ),
    );
  }

  Widget _detailWrap(List<_DetailItem> items) {
    return Wrap(
      spacing: 16,
      runSpacing: 8,
      children: [
        for (final item in items)
          SizedBox(
            width: 230,
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text(item.label.toUpperCase(),
                    style: Theme.of(context).textTheme.labelMedium),
                const SizedBox(height: 3),
                SelectableText(
                  item.value.isEmpty ? '-' : item.value,
                  style: const TextStyle(
                      fontFamily: 'monospace',
                      color: KpColors.bodyStrong,
                      fontSize: 13),
                ),
              ],
            ),
          ),
      ],
    );
  }

  void _deriveRecordVersion(CredentialRecord record) {
    setState(() {
      _selected = record;
      _section = _Section.derive;
    });
  }

  Future<void> _rotateRecord(CredentialRecord record) async {
    final api = _api;
    if (api == null) return;
    try {
      final length = (record.encodingDescriptor['length'] as num?)?.toInt() ??
          _defaultLength;
      final next = await api.rotateCredential(record: record, length: length);
      await _refresh();
      await _syncCdrToFirstUsb(silent: true);
      setState(() {
        _selected = next;
        _message = context.t.newVersionCreated;
      });
    } catch (_) {
      setState(() => _message = context.t.operationFailed);
    }
  }

  Future<void> _createRecord(_RecordDraft draft) async {
    final api = _api;
    if (api == null) return;
    try {
      final record = await api.addCredential(
        displayName: draft.displayName,
        serviceHint: draft.serviceHint,
        accountHint: draft.accountHint,
        notes: draft.notes,
        length: draft.length,
        requireUpper: draft.requireUpper,
        requireLower: draft.requireLower,
        requireDigit: draft.requireDigit,
        requireSymbol: draft.requireSymbol,
        forbiddenChars: draft.forbiddenChars,
      );
      await _refresh();
      await _syncCdrToFirstUsb(silent: true);
      setState(() {
        _selected = record;
        _section = _Section.records;
        _message = context.t.recordCreated;
      });
    } catch (_) {
      setState(() => _message = context.t.operationFailed);
    }
  }

  Future<void> _showEditMetadata(CredentialRecord record) async {
    final t = context.t;
    final name = TextEditingController(text: record.displayName);
    final service = TextEditingController(text: record.serviceHint);
    final account = TextEditingController(text: record.accountHint);
    final notes = TextEditingController(text: record.notes);
    final saved = await showDialog<bool>(
      context: context,
      builder: (dialogContext) => AlertDialog(
        title: Text(t.editMetadata),
        content: SizedBox(
          width: 520,
          child: Column(
            mainAxisSize: MainAxisSize.min,
            children: [
              TextField(
                  controller: name,
                  decoration: InputDecoration(labelText: t.displayName)),
              const SizedBox(height: 10),
              TextField(
                  controller: service,
                  decoration: InputDecoration(labelText: t.serviceHint)),
              const SizedBox(height: 10),
              TextField(
                  controller: account,
                  decoration: InputDecoration(labelText: t.accountHint)),
              const SizedBox(height: 10),
              TextField(
                  controller: notes,
                  decoration: InputDecoration(labelText: t.notes),
                  maxLines: 3),
              const SizedBox(height: 12),
              InlineNotice(
                  text: t.metadataDoesNotChangePassword,
                  icon: Icons.info_outline_rounded),
            ],
          ),
        ),
        actions: [
          TextButton(
              onPressed: () => Navigator.pop(dialogContext, false),
              child: Text(t.cancel)),
          FilledButton(
              onPressed: () => Navigator.pop(dialogContext, true),
              child: Text(t.save)),
        ],
      ),
    );
    if (saved == true) {
      try {
        await _api?.updateCredentialDisplay(record.copyWith(
          displayName: name.text.trim(),
          serviceHint: service.text.trim(),
          accountHint: account.text.trim(),
          notes: notes.text.trim(),
        ));
        await _refresh();
        await _syncCdrToFirstUsb(silent: true);
        setState(() => _message = t.metadataSaved);
      } catch (_) {
        setState(() => _message = t.operationFailed);
      }
    }
    name.dispose();
    service.dispose();
    account.dispose();
    notes.dispose();
  }

  Widget _securityPage() {
    final t = context.t;
    final security = _status?.securityStatus;
    final localAvailable = _status?.enrolled == true;
    final usbAvailable = _usbCandidates.any((u) => u.readable);
    return _page(
      title: t.security,
      subtitle: t.integrityCheck,
      children: [
        Wrap(
          spacing: 12,
          runSpacing: 12,
          children: [
            SizedBox(
                width: 220,
                child: SignalTile(
                    label: t.cdrMac, value: t.ok, tone: KpColors.success)),
            SizedBox(
                width: 220,
                child: SignalTile(
                    label: t.usbAuthentication,
                    value: _usbCandidates.any((u) => u.readable)
                        ? t.ok
                        : t.notFound,
                    tone: _usbCandidates.any((u) => u.readable)
                        ? KpColors.success
                        : KpColors.warning)),
            SizedBox(
                width: 220,
                child: SignalTile(
                    label: t.analytics,
                    value: t.disabled,
                    tone: KpColors.success)),
          ],
        ),
        SectionPanel(
          child: Column(
            children: [
              InfoRow(label: t.status, value: security?.provider ?? '-'),
              InfoRow(label: t.localOnly, value: t.enabled),
              InfoRow(
                  label: t.clipboardClearing,
                  value: '$_clipboardTimeout ${t.seconds}'),
              InfoRow(label: t.logSafety, value: t.ok),
            ],
          ),
        ),
        SectionPanel(
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Text(t.recovery, style: Theme.of(context).textTheme.titleMedium),
              const SizedBox(height: 10),
              InfoRow(
                  label: t.recoveryPathMnemonicComputer,
                  value: localAvailable ? t.available : t.notFound),
              InfoRow(
                  label: t.recoveryPathMnemonicUsb,
                  value: usbAvailable ? t.available : t.notFound),
              InfoRow(
                  label: t.recoveryPathComputerUsb,
                  value: localAvailable && usbAvailable
                      ? t.available
                      : t.notFound),
              const SizedBox(height: 10),
              InlineNotice(
                  text: t.singleFactorNotEnough,
                  icon: Icons.lock_outline_rounded),
            ],
          ),
        ),
      ],
    );
  }

  Widget _settingsPage() {
    final t = context.t;
    return _page(
      title: t.settings,
      subtitle: t.privacySummary,
      children: [
        SectionPanel(
          child: Column(
            children: [
              _languageSelector(),
              const SizedBox(height: 12),
              DropdownButtonFormField<ThemeMode>(
                key: ValueKey(widget.themeMode),
                initialValue: widget.themeMode,
                decoration: InputDecoration(labelText: t.theme),
                items: [
                  DropdownMenuItem(
                      value: ThemeMode.system, child: Text(t.systemDefault)),
                  DropdownMenuItem(value: ThemeMode.dark, child: Text(t.dark)),
                  DropdownMenuItem(
                      value: ThemeMode.light, child: Text(t.light)),
                ],
                onChanged: (value) =>
                    widget.onThemeModeChanged(value ?? ThemeMode.dark),
              ),
              const SizedBox(height: 18),
              _slider(t.clipboardTimeout, _clipboardTimeout.toDouble(), 10, 120,
                  (value) => setState(() => _clipboardTimeout = value.round())),
              _slider(t.defaultPasswordLength, _defaultLength.toDouble(), 8, 64,
                  (value) => setState(() => _defaultLength = value.round())),
              const SizedBox(height: 8),
              _derivationAlgorithmSelector(),
              const SizedBox(height: 12),
              Row(
                children: [
                  Expanded(child: Text(t.advancedMode)),
                  Switch(
                    value: _advancedMode,
                    onChanged: (value) => setState(() => _advancedMode = value),
                  ),
                ],
              ),
            ],
          ),
        ),
        SectionPanel(
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Text(t.exportDiagnostics,
                  style: Theme.of(context).textTheme.titleMedium),
              const SizedBox(height: 10),
              OutlinedButton.icon(
                onPressed: _showDiagnostics,
                icon: const Icon(Icons.description_outlined),
                label: Text(t.exportDiagnostics),
              ),
              const SizedBox(height: 18),
              DangerPanel(
                title: t.resetApplicationData,
                body: t.resetWarning,
                actionLabel: t.resetApplicationData,
                onPressed: _confirmResetApplicationData,
              ),
            ],
          ),
        ),
      ],
    );
  }

  Widget _languageSelector() {
    final t = context.t;
    final value = widget.locale?.languageCode ?? 'system';
    return DropdownButtonFormField<String>(
      key: ValueKey(value),
      initialValue: value,
      decoration: InputDecoration(labelText: t.language),
      items: [
        DropdownMenuItem(value: 'system', child: Text(t.systemDefault)),
        DropdownMenuItem(value: 'en', child: Text(t.english)),
        DropdownMenuItem(value: 'zh', child: Text(t.simplifiedChinese)),
      ],
      onChanged: (value) {
        if (value == 'en') widget.onLocaleChanged(const Locale('en'));
        if (value == 'zh') widget.onLocaleChanged(const Locale('zh'));
        if (value == 'system') widget.onLocaleChanged(null);
      },
    );
  }

  Future<void> _showDiagnostics() async {
    final t = context.t;
    final report = const JsonEncoder.withIndent('  ').convert({
      'app': 'KeyLessPass',
      'localOnly': true,
      'enrolled': _status?.enrolled == true,
      'platform': _status?.securityStatus.platform ?? '-',
      'provider': _status?.securityStatus.provider ?? '-',
      'degradedProtection': _status?.securityStatus.degraded ?? true,
      'recordCount': _recordGroupCount,
      'usbVolumeCount': _usbCandidates.length,
      'readableUsbPackageCount':
          _usbCandidates.where((item) => item.readable).length,
      'clipboardTimeoutSeconds': _clipboardTimeout,
      'passwordDerivationAlgorithm': _effectivePasswordDerivationAlgorithm(),
      'analytics': false,
    });
    final copied = await showDialog<bool>(
      context: context,
      builder: (dialogContext) => AlertDialog(
        title: Text(t.diagnosticsTitle),
        content: SizedBox(
          width: 560,
          child: SelectableText(report),
        ),
        actions: [
          TextButton(
              onPressed: () => Navigator.pop(dialogContext, false),
              child: Text(t.close)),
          FilledButton.icon(
            onPressed: () => Navigator.pop(dialogContext, true),
            icon: const Icon(Icons.content_copy_rounded),
            label: Text(t.copyDiagnostics),
          ),
        ],
      ),
    );
    if (copied == true) {
      await Clipboard.setData(ClipboardData(text: report));
      setState(() => _message = t.diagnosticsCopied);
    }
  }

  Future<void> _confirmResetApplicationData() async {
    final t = context.t;
    final controller = TextEditingController();
    final confirmed = await showDialog<bool>(
      context: context,
      builder: (dialogContext) => AlertDialog(
        title: Text(t.resetConfirmTitle),
        content: SizedBox(
          width: 480,
          child: Column(
            mainAxisSize: MainAxisSize.min,
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              InlineNotice(
                  text: t.resetWarning,
                  tone: KpColors.error,
                  icon: Icons.warning_amber_rounded),
              const SizedBox(height: 12),
              TextField(
                controller: controller,
                decoration:
                    InputDecoration(labelText: t.resetConfirmationPrompt),
              ),
            ],
          ),
        ),
        actions: [
          TextButton(
              onPressed: () => Navigator.pop(dialogContext, false),
              child: Text(t.cancel)),
          FilledButton(
              onPressed: () => Navigator.pop(dialogContext, true),
              child: Text(t.resetApplicationData)),
        ],
      ),
    );

    if (confirmed != true) {
      controller.dispose();
      return;
    }
    if (controller.text.trim() != 'RESET') {
      controller.dispose();
      setState(() => _message = t.resetConfirmationMismatch);
      return;
    }
    controller.dispose();

    try {
      await _api?.resetApplicationData(confirmation: 'RESET');
      await _refresh();
      if (mounted) {
        setState(() {
          _records = [];
          _selected = null;
          _section = _Section.setup;
          _message = t.resetComplete;
        });
      }
    } catch (_) {
      if (mounted) setState(() => _message = t.operationFailed);
    }
  }

  Widget _slider(String label, double value, double min, double max,
      ValueChanged<double> onChanged) {
    return Row(
      children: [
        SizedBox(width: 220, child: Text('$label ${value.round()}')),
        Expanded(
            child: Slider(
                value: value,
                min: min,
                max: max,
                divisions: (max - min).round(),
                onChanged: onChanged)),
      ],
    );
  }

  Widget _derivationAlgorithmSelector() {
    final t = context.t;
    final enrolled = _status?.enrolled == true;
    final value = _effectivePasswordDerivationAlgorithm();
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        DropdownButtonFormField<String>(
          key: ValueKey('derivation-$value-$enrolled'),
          initialValue: value,
          decoration: InputDecoration(labelText: t.derivationAlgorithm),
          items: const [
            'hkdf-sha256',
            'argon2id',
            'scrypt',
            'pbkdf2-hmac-sha256',
          ]
              .map((algorithm) => DropdownMenuItem(
                    value: algorithm,
                    child: Text(_algorithmLabelStatic(algorithm)),
                  ))
              .toList(),
          onChanged: enrolled
              ? null
              : (value) => setState(
                  () => _passwordDerivationAlgorithm = value ?? 'hkdf-sha256'),
        ),
        const SizedBox(height: 8),
        InlineNotice(
          text: enrolled
              ? (_status?.hasStoredPasswordDerivationAlgorithm == true
                  ? t.algorithmLockedUntilReset
                  : t.legacyHkdfDetected)
              : t.algorithmAppliesOnNextSetup,
          icon: Icons.info_outline_rounded,
        ),
      ],
    );
  }

  String _effectivePasswordDerivationAlgorithm() {
    final status = _status;
    if (status?.enrolled == true) {
      return status!.passwordDerivationAlgorithm;
    }
    return _passwordDerivationAlgorithm;
  }

  Widget _aboutPage() {
    final t = context.t;
    return _page(
      title: t.about,
      subtitle: t.supportEmail,
      children: [
        SectionPanel(
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Text(t.appName, style: Theme.of(context).textTheme.titleLarge),
              const SizedBox(height: 12),
              Text(t.aboutBody),
              const SizedBox(height: 12),
              InlineNotice(
                  text: t.privacySummary, icon: Icons.privacy_tip_outlined),
            ],
          ),
        ),
      ],
    );
  }

  Widget _messageBar() {
    if (_message == null) return const SizedBox.shrink();
    final text =
        _message == 'core-unavailable' ? context.t.coreUnavailable : _message!;
    return InlineNotice(text: text, tone: KpColors.warning);
  }

  Widget _page(
      {required String title,
      required String subtitle,
      Widget? trailing,
      required List<Widget> children}) {
    return Padding(
      padding: const EdgeInsets.all(28),
      child: ListView(
        children: [
          PageTitle(title: title, subtitle: subtitle, trailing: trailing),
          const SizedBox(height: 18),
          ...children.expand((child) => [child, const SizedBox(height: 16)]),
        ],
      ),
    );
  }

  String _stateLabel(String state) {
    final t = context.t;
    return switch (state) {
      'active' => t.active,
      'pending_rotation' => t.pending,
      'retired' => t.retired,
      'conflict' => t.conflict,
      'error' => t.error,
      _ => state,
    };
  }

  String _filterLabel(_RecordFilter filter) {
    final t = context.t;
    return switch (filter) {
      _RecordFilter.all => t.all,
      _RecordFilter.active => t.active,
      _RecordFilter.pending => t.pending,
      _RecordFilter.retired => t.retired,
      _RecordFilter.conflict => t.conflict,
      _RecordFilter.error => t.error,
    };
  }

  String _shortDate(String value) {
    if (value.length >= 16) {
      return value.substring(0, 16).replaceFirst('T', ' ');
    }
    return value.isEmpty ? '-' : value;
  }
}

_Section _initialSectionFromEnvironment() {
  switch (
      Platform.environment['KEYLESSPASS_START_SECTION']?.trim().toLowerCase()) {
    case 'setup':
      return _Section.setup;
    case 'records':
      return _Section.records;
    case 'add':
      return _Section.add;
    case 'derive':
      return _Section.derive;
    case 'rotation':
      return _Section.rotation;
    case 'usb':
      return _Section.usb;
    case 'security':
      return _Section.security;
    case 'settings':
      return _Section.settings;
    case 'about':
      return _Section.about;
    default:
      return _Section.dashboard;
  }
}

Future<void> _chooseUsbPath(TextEditingController controller) async {
  final path = await NativePlatformApi.chooseUsbDirectory();
  if (path != null) {
    controller.text = path;
  }
}

String _algorithmLabelStatic(String value) {
  return switch (value) {
    'argon2id' => 'Argon2id',
    'scrypt' => 'scrypt',
    'pbkdf2-hmac-sha256' => 'PBKDF2-HMAC-SHA256',
    _ => 'HKDF-SHA256',
  };
}

class _RecordDraft {
  const _RecordDraft({
    required this.displayName,
    required this.serviceHint,
    required this.accountHint,
    required this.notes,
    required this.length,
    required this.requireUpper,
    required this.requireLower,
    required this.requireDigit,
    required this.requireSymbol,
    required this.forbiddenChars,
  });

  final String displayName;
  final String serviceHint;
  final String accountHint;
  final String notes;
  final int length;
  final bool requireUpper;
  final bool requireLower;
  final bool requireDigit;
  final bool requireSymbol;
  final String forbiddenChars;
}

class _SetupPage extends StatefulWidget {
  const _SetupPage({
    required this.status,
    required this.api,
    required this.usbCandidates,
    required this.passwordDerivationAlgorithm,
    required this.onDone,
  });

  final AppStatus? status;
  final CoreApi? api;
  final List<UsbCandidate> usbCandidates;
  final String passwordDerivationAlgorithm;
  final Future<void> Function() onDone;

  @override
  State<_SetupPage> createState() => _SetupPageState();
}

class _SetupPageState extends State<_SetupPage> {
  final _mnemonic = TextEditingController();
  final _usbPath = TextEditingController();
  String _mnemonicLanguage = 'english';
  int _mnemonicWordCount = 20;
  _SetupMode _setupMode = _SetupMode.create;
  bool _showMnemonic = false;
  bool _busy = false;
  bool _generatingMnemonic = false;
  String? _message;

  @override
  void initState() {
    super.initState();
    _fillUsb();
  }

  @override
  void didUpdateWidget(covariant _SetupPage oldWidget) {
    super.didUpdateWidget(oldWidget);
    _fillUsb();
  }

  void _fillUsb() {
    if (_usbPath.text.isNotEmpty || widget.usbCandidates.isEmpty) return;
    final writable =
        widget.usbCandidates.where((item) => item.rootPath.isNotEmpty);
    _usbPath.text =
        (writable.isEmpty ? widget.usbCandidates.first : writable.first)
            .rootPath;
  }

  @override
  void dispose() {
    _mnemonic.dispose();
    _usbPath.dispose();
    super.dispose();
  }

  Future<void> _submit() async {
    final api = widget.api;
    if (api == null ||
        _mnemonic.text.trim().isEmpty ||
        _usbPath.text.trim().isEmpty) {
      return;
    }
    setState(() {
      _busy = true;
      _message = null;
    });
    try {
      await api.enroll(
        mnemonic: _mnemonic.text,
        usbPath: _usbPath.text.trim(),
        passwordDerivationAlgorithm: widget.passwordDerivationAlgorithm,
      );
      _mnemonic.clear();
      await widget.onDone();
      if (mounted) setState(() => _message = context.t.setupComplete);
    } catch (_) {
      _mnemonic.clear();
      if (mounted) setState(() => _message = context.t.operationFailed);
    } finally {
      if (mounted) setState(() => _busy = false);
    }
  }

  Future<void> _generateMnemonic() async {
    final api = widget.api;
    if (api == null) return;
    final t = context.t;
    setState(() {
      _generatingMnemonic = true;
      _message = null;
    });
    try {
      final result = await api.generateMnemonic(
        language: _mnemonicLanguage,
        wordCount: _mnemonicWordCount,
      );
      _mnemonic.text = (result['mnemonic'] as String?) ?? '';
      if (mounted) setState(() => _message = t.generatedMnemonicReady);
    } catch (_) {
      if (mounted) setState(() => _message = t.operationFailed);
    } finally {
      if (mounted) setState(() => _generatingMnemonic = false);
    }
  }

  @override
  Widget build(BuildContext context) {
    final t = context.t;
    final enrolled = widget.status?.enrolled == true;
    return Padding(
      padding: const EdgeInsets.all(28),
      child: ListView(
        children: [
          PageTitle(
              title: t.setup,
              subtitle: enrolled ? t.setupLocked : t.setupStartSubtitle),
          const SizedBox(height: 18),
          SectionPanel(
            child: enrolled
                ? Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      InlineNotice(
                          text: t.setupLockedMessage,
                          icon: Icons.lock_outline_rounded),
                      const SizedBox(height: 12),
                      InfoRow(label: t.status, value: t.initialized),
                      InfoRow(label: t.localOnly, value: t.enabled),
                      InfoRow(
                          label: t.derivationAlgorithm,
                          value: _algorithmLabelStatic(
                              widget.status?.passwordDerivationAlgorithm ??
                                  'hkdf-sha256')),
                    ],
                  )
                : Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      Text(t.setupStartTitle,
                          style: Theme.of(context).textTheme.titleMedium),
                      const SizedBox(height: 12),
                      SegmentedButton<_SetupMode>(
                        segments: [
                          ButtonSegment(
                              value: _SetupMode.create,
                              label: Text(t.createNewProfile),
                              icon: const Icon(Icons.add_rounded)),
                          ButtonSegment(
                              value: _SetupMode.recover,
                              label: Text(t.recoverExistingProfile),
                              icon: const Icon(
                                  Icons.settings_backup_restore_rounded)),
                        ],
                        selected: {_setupMode},
                        onSelectionChanged: _busy || _generatingMnemonic
                            ? null
                            : (value) =>
                                setState(() => _setupMode = value.first),
                      ),
                      const SizedBox(height: 16),
                      if (_setupMode == _SetupMode.create) ...[
                        WorkflowStepper(steps: [
                          t.mnemonicPhrase,
                          t.createFactors,
                          t.usbDevice,
                          t.recovery
                        ], current: 0),
                        const SizedBox(height: 16),
                        Wrap(
                          spacing: 12,
                          runSpacing: 12,
                          crossAxisAlignment: WrapCrossAlignment.center,
                          children: [
                            SizedBox(
                              width: 330,
                              child: SegmentedButton<String>(
                                segments: [
                                  ButtonSegment(
                                      value: 'english',
                                      label: Text(t.englishMnemonic)),
                                  ButtonSegment(
                                      value: 'simplifiedChinese',
                                      label: Text(t.chineseMnemonic)),
                                ],
                                selected: {_mnemonicLanguage},
                                onSelectionChanged: _busy || _generatingMnemonic
                                    ? null
                                    : (value) => setState(
                                        () => _mnemonicLanguage = value.first),
                              ),
                            ),
                            SizedBox(
                              width: 150,
                              child: DropdownButtonFormField<int>(
                                initialValue: _mnemonicWordCount,
                                decoration:
                                    InputDecoration(labelText: t.wordCount),
                                items: const [
                                  DropdownMenuItem(
                                      value: 20, child: Text('20')),
                                  DropdownMenuItem(
                                      value: 24, child: Text('24')),
                                  DropdownMenuItem(
                                      value: 28, child: Text('28')),
                                ],
                                onChanged: _busy || _generatingMnemonic
                                    ? null
                                    : (value) => setState(
                                        () => _mnemonicWordCount = value ?? 20),
                              ),
                            ),
                            OutlinedButton.icon(
                              onPressed: _busy || _generatingMnemonic
                                  ? null
                                  : _generateMnemonic,
                              icon: const Icon(Icons.auto_awesome_rounded),
                              label: Text(t.generateMnemonic),
                            ),
                          ],
                        ),
                        const SizedBox(height: 12),
                        InlineNotice(
                            text: t.mnemonicGeneratedLocally,
                            icon: Icons.lock_outline_rounded),
                        const SizedBox(height: 12),
                        InfoRow(
                            label: t.derivationAlgorithm,
                            value: _algorithmLabelStatic(
                                widget.passwordDerivationAlgorithm)),
                        const SizedBox(height: 12),
                        TextField(
                          controller: _mnemonic,
                          obscureText: !_showMnemonic,
                          decoration: InputDecoration(
                            labelText: t.mnemonicPhrase,
                            suffixIcon: IconButton(
                              tooltip: _showMnemonic
                                  ? t.hideMnemonic
                                  : t.showMnemonic,
                              onPressed: () => setState(
                                  () => _showMnemonic = !_showMnemonic),
                              icon: Icon(_showMnemonic
                                  ? Icons.visibility_off_rounded
                                  : Icons.visibility_rounded),
                            ),
                          ),
                        ),
                        const SizedBox(height: 12),
                        TextField(
                          controller: _usbPath,
                          decoration: InputDecoration(
                            labelText: t.usbPath,
                            suffixIcon: IconButton(
                              tooltip: t.chooseUsb,
                              onPressed: () => _chooseUsbPath(_usbPath),
                              icon: const Icon(Icons.folder_open_rounded),
                            ),
                          ),
                        ),
                        const SizedBox(height: 12),
                        InlineNotice(
                            text: t.manualUsbHint, icon: Icons.usb_rounded),
                        if (_message != null) ...[
                          const SizedBox(height: 12),
                          InlineNotice(text: _message!)
                        ],
                        const SizedBox(height: 16),
                        Align(
                          alignment: Alignment.centerRight,
                          child: FilledButton.icon(
                              onPressed: _busy ? null : _submit,
                              icon: const Icon(Icons.verified_user_rounded),
                              label: Text(t.createFactors)),
                        ),
                      ] else ...[
                        InlineNotice(
                            text: t.recoverLocalHelp,
                            icon: Icons.computer_rounded,
                            tone: KpColors.primary),
                        const SizedBox(height: 16),
                        _RecoveryPanel(
                          api: widget.api,
                          usbCandidates: widget.usbCandidates,
                          initialMode: _RecoveryMode.newDevice,
                          allowedModes: const {_RecoveryMode.newDevice},
                          onDone: widget.onDone,
                        ),
                      ],
                    ],
                  ),
          ),
        ],
      ),
    );
  }
}

class _UsbDevicePage extends StatefulWidget {
  const _UsbDevicePage({
    required this.api,
    required this.enrolled,
    required this.usbCandidates,
    required this.cdrStatus,
    required this.onRefresh,
    required this.onSyncCdrToUsb,
    required this.onRestoreCdrFromUsb,
    required this.onSetup,
  });

  final CoreApi? api;
  final bool enrolled;
  final List<UsbCandidate> usbCandidates;
  final UsbCdrStatus? cdrStatus;
  final Future<void> Function() onRefresh;
  final Future<void> Function(String usbPath, {bool silent}) onSyncCdrToUsb;
  final Future<void> Function(String usbPath) onRestoreCdrFromUsb;
  final VoidCallback onSetup;

  @override
  State<_UsbDevicePage> createState() => _UsbDevicePageState();
}

class _UsbDevicePageState extends State<_UsbDevicePage> {
  final _mnemonic = TextEditingController();
  final _usbPath = TextEditingController();
  String? _message;
  bool _busy = false;

  @override
  void initState() {
    super.initState();
    _fillUsb();
  }

  @override
  void didUpdateWidget(covariant _UsbDevicePage oldWidget) {
    super.didUpdateWidget(oldWidget);
    _fillUsb();
  }

  void _fillUsb() {
    if (_usbPath.text.isNotEmpty || widget.usbCandidates.isEmpty) return;
    final readable = widget.usbCandidates.where((item) => item.readable);
    _usbPath.text =
        (readable.isEmpty ? widget.usbCandidates.first : readable.first)
            .rootPath;
  }

  @override
  void dispose() {
    _mnemonic.dispose();
    _usbPath.dispose();
    super.dispose();
  }

  Future<void> _verify() async {
    final api = widget.api;
    if (api == null || _usbPath.text.trim().isEmpty) return;
    setState(() {
      _busy = true;
      _message = null;
    });
    try {
      await api.verifyUsbPackage(usbPath: _usbPath.text.trim());
      await widget.onRefresh();
      if (mounted) setState(() => _message = context.t.usbVerified);
    } catch (_) {
      if (mounted) setState(() => _message = context.t.operationFailed);
    } finally {
      if (mounted) setState(() => _busy = false);
    }
  }

  Future<void> _rebuild() async {
    final api = widget.api;
    if (api == null) return;
    setState(() {
      _busy = true;
      _message = null;
    });
    try {
      await api.recoverUsb(
          mnemonic: _mnemonic.text, usbPath: _usbPath.text.trim());
      _mnemonic.clear();
      await widget.onRefresh();
      if (mounted) setState(() => _message = context.t.usbRebuilt);
    } catch (_) {
      _mnemonic.clear();
      if (mounted) setState(() => _message = context.t.operationFailed);
    } finally {
      if (mounted) setState(() => _busy = false);
    }
  }

  @override
  Widget build(BuildContext context) {
    final t = context.t;
    final readable = widget.usbCandidates.where((item) => item.readable).length;
    return Padding(
      padding: const EdgeInsets.all(28),
      child: ListView(
        children: [
          PageTitle(
            title: t.usbDevice,
            subtitle: t.usbCount(widget.usbCandidates.length),
            trailing: OutlinedButton.icon(
                onPressed: widget.onRefresh,
                icon: const Icon(Icons.refresh_rounded),
                label: Text(t.rescanUsb)),
          ),
          const SizedBox(height: 18),
          Wrap(
            spacing: 12,
            runSpacing: 12,
            children: [
              SizedBox(
                  width: 220,
                  child: SignalTile(
                      label: t.detectedUsb,
                      value: '${widget.usbCandidates.length}',
                      tone: widget.usbCandidates.isEmpty
                          ? KpColors.warning
                          : KpColors.success)),
              SizedBox(
                  width: 220,
                  child: SignalTile(
                      label: t.packageStatus,
                      value:
                          readable > 0 ? t.packageReadable : t.packageMissing,
                      tone:
                          readable > 0 ? KpColors.success : KpColors.warning)),
            ],
          ),
          const SizedBox(height: 16),
          if (widget.enrolled) ...[
            SectionPanel(
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Text(t.cdrBackup,
                      style: Theme.of(context).textTheme.titleMedium),
                  const SizedBox(height: 12),
                  InlineNotice(
                    text: widget.cdrStatus?.status == 'consistent'
                        ? t.cdrBackupConsistent
                        : t.cdrBackupNeedsAction,
                    icon: Icons.backup_table_rounded,
                    tone: widget.cdrStatus?.status == 'consistent'
                        ? KpColors.success
                        : KpColors.warning,
                  ),
                  const SizedBox(height: 12),
                  InfoRow(
                      label: t.cdrBackupStatus,
                      value: widget.cdrStatus?.status ?? t.notFound),
                  InfoRow(
                      label: t.localRecordCount,
                      value: '${widget.cdrStatus?.localRecordCount ?? 0}'),
                  InfoRow(
                      label: t.usbRecordCount,
                      value: '${widget.cdrStatus?.usbRecordCount ?? 0}'),
                  const SizedBox(height: 12),
                  Wrap(
                    spacing: 12,
                    runSpacing: 12,
                    children: [
                      FilledButton.icon(
                        onPressed: _busy || _usbPath.text.trim().isEmpty
                            ? null
                            : () => widget.onSyncCdrToUsb(_usbPath.text.trim()),
                        icon: const Icon(Icons.upload_file_rounded),
                        label: Text(t.syncLocalToUsb),
                      ),
                      OutlinedButton.icon(
                        onPressed: _busy ||
                                _usbPath.text.trim().isEmpty ||
                                widget.cdrStatus == null ||
                                widget.cdrStatus!.status == 'missing' ||
                                widget.cdrStatus!.status == 'invalid'
                            ? null
                            : () => widget
                                .onRestoreCdrFromUsb(_usbPath.text.trim()),
                        icon: const Icon(Icons.download_for_offline_rounded),
                        label: Text(t.restoreLocalFromUsb),
                      ),
                    ],
                  ),
                ],
              ),
            ),
            const SizedBox(height: 16),
          ],
          const SizedBox(height: 16),
          SectionPanel(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text(t.usbActions,
                    style: Theme.of(context).textTheme.titleMedium),
                const SizedBox(height: 12),
                TextField(
                  controller: _usbPath,
                  decoration: InputDecoration(
                    labelText: t.usbPath,
                    suffixIcon: IconButton(
                      tooltip: t.chooseUsb,
                      onPressed: () => _chooseUsbPath(_usbPath),
                      icon: const Icon(Icons.folder_open_rounded),
                    ),
                  ),
                ),
                const SizedBox(height: 12),
                TextField(
                    controller: _mnemonic,
                    obscureText: true,
                    decoration: InputDecoration(labelText: t.mnemonicPhrase)),
                const SizedBox(height: 12),
                InlineNotice(
                    text: t.usbHelpHint, icon: Icons.help_outline_rounded),
                const SizedBox(height: 12),
                InlineNotice(
                    text: t.usbFactorContainerHelp, icon: Icons.usb_rounded),
                if (_message != null) ...[
                  const SizedBox(height: 12),
                  InlineNotice(text: _message!)
                ],
                const SizedBox(height: 16),
                Wrap(
                  spacing: 12,
                  runSpacing: 12,
                  children: [
                    FilledButton.icon(
                        onPressed: _busy ? null : _verify,
                        icon: const Icon(Icons.verified_rounded),
                        label: Text(t.verifyUsbPackage)),
                    OutlinedButton.icon(
                        onPressed: _busy || !widget.enrolled ? null : _rebuild,
                        icon: const Icon(Icons.usb_rounded),
                        label: Text(t.rebuildUsbPackage)),
                    if (!widget.enrolled)
                      OutlinedButton.icon(
                          onPressed: widget.onSetup,
                          icon: const Icon(Icons.verified_user_rounded),
                          label: Text(t.setup)),
                  ],
                ),
              ],
            ),
          ),
          const SizedBox(height: 16),
          SectionPanel(
            child: widget.usbCandidates.isEmpty
                ? Text(t.notFound)
                : Column(
                    children: widget.usbCandidates.map((candidate) {
                      return InfoRow(
                        label: candidate.readable
                            ? t.packageReadable
                            : t.packageMissing,
                        value: '${candidate.rootPath}\n${candidate.message}',
                      );
                    }).toList(),
                  ),
          ),
          const SizedBox(height: 16),
          SectionPanel(
            child: _RecoveryPanel(
              api: widget.api,
              usbCandidates: widget.usbCandidates,
              onDone: widget.onRefresh,
            ),
          ),
        ],
      ),
    );
  }
}

class _AddRecordPage extends StatefulWidget {
  const _AddRecordPage({required this.defaultLength, required this.onCreate});

  final int defaultLength;
  final Future<void> Function(_RecordDraft draft) onCreate;

  @override
  State<_AddRecordPage> createState() => _AddRecordPageState();
}

class _AddRecordPageState extends State<_AddRecordPage> {
  final _formKey = GlobalKey<FormState>();
  final _displayName = TextEditingController();
  final _serviceHint = TextEditingController();
  final _accountHint = TextEditingController();
  final _notes = TextEditingController();
  final _forbiddenChars = TextEditingController(text: '"\'`\\/:;?&<>{}[]()|, ');
  late double _length = widget.defaultLength.toDouble();
  bool _requireUpper = true;
  bool _requireLower = true;
  bool _requireDigit = true;
  bool _requireSymbol = true;
  bool _busy = false;

  @override
  void dispose() {
    _displayName.dispose();
    _serviceHint.dispose();
    _accountHint.dispose();
    _notes.dispose();
    _forbiddenChars.dispose();
    super.dispose();
  }

  Future<void> _submit() async {
    if (!_formKey.currentState!.validate()) return;
    setState(() => _busy = true);
    await widget.onCreate(_RecordDraft(
      displayName: _displayName.text.trim(),
      serviceHint: _serviceHint.text.trim(),
      accountHint: _accountHint.text.trim(),
      notes: _notes.text.trim(),
      length: _length.round(),
      requireUpper: _requireUpper,
      requireLower: _requireLower,
      requireDigit: _requireDigit,
      requireSymbol: _requireSymbol,
      forbiddenChars: _forbiddenChars.text,
    ));
    if (mounted) setState(() => _busy = false);
  }

  @override
  Widget build(BuildContext context) {
    final t = context.t;
    return Padding(
      padding: const EdgeInsets.all(28),
      child: ListView(
        children: [
          PageTitle(title: t.addRecord, subtitle: t.ruleChangeRequiresRotation),
          const SizedBox(height: 18),
          SectionPanel(
            child: Form(
              key: _formKey,
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  TextFormField(
                      controller: _displayName,
                      decoration: InputDecoration(labelText: t.displayName),
                      validator: (value) =>
                          value == null || value.trim().isEmpty
                              ? t.requiredField
                              : null),
                  const SizedBox(height: 12),
                  TextFormField(
                      controller: _serviceHint,
                      decoration: InputDecoration(labelText: t.serviceHint)),
                  const SizedBox(height: 12),
                  TextFormField(
                      controller: _accountHint,
                      decoration: InputDecoration(labelText: t.accountHint)),
                  const SizedBox(height: 12),
                  TextFormField(
                      controller: _notes,
                      decoration: InputDecoration(labelText: t.notes),
                      maxLines: 3),
                  const SizedBox(height: 18),
                  Row(
                    children: [
                      SizedBox(
                          width: 170,
                          child: Text('${t.length} ${_length.round()}')),
                      Expanded(
                          child: Slider(
                              min: 8,
                              max: 64,
                              divisions: 56,
                              value: _length,
                              onChanged: (value) =>
                                  setState(() => _length = value))),
                    ],
                  ),
                  const SizedBox(height: 12),
                  Text(t.requiredClasses,
                      style: Theme.of(context).textTheme.titleSmall),
                  const SizedBox(height: 8),
                  Wrap(
                    spacing: 12,
                    runSpacing: 8,
                    children: [
                      _classToggle(t.requireUppercase, _requireUpper,
                          (value) => setState(() => _requireUpper = value)),
                      _classToggle(t.requireLowercase, _requireLower,
                          (value) => setState(() => _requireLower = value)),
                      _classToggle(t.requireDigits, _requireDigit,
                          (value) => setState(() => _requireDigit = value)),
                      _classToggle(t.requireSymbols, _requireSymbol,
                          (value) => setState(() => _requireSymbol = value)),
                    ],
                  ),
                  const SizedBox(height: 12),
                  TextFormField(
                      controller: _forbiddenChars,
                      decoration:
                          InputDecoration(labelText: t.forbiddenCharacters)),
                  const SizedBox(height: 12),
                  InlineNotice(text: t.metadataDoesNotChangePassword),
                  const SizedBox(height: 18),
                  Align(
                    alignment: Alignment.centerRight,
                    child: FilledButton.icon(
                        onPressed: _busy ? null : _submit,
                        icon: const Icon(Icons.add_rounded),
                        label: Text(t.save)),
                  ),
                ],
              ),
            ),
          ),
        ],
      ),
    );
  }

  Widget _classToggle(String label, bool value, ValueChanged<bool> onChanged) {
    return FilterChip(
      selected: value,
      label: Text(label),
      onSelected: onChanged,
      showCheckmark: true,
    );
  }
}

class _RecordPicker extends StatelessWidget {
  const _RecordPicker(
      {required this.records, required this.selected, required this.onChanged});

  final List<CredentialRecord> records;
  final CredentialRecord? selected;
  final ValueChanged<CredentialRecord> onChanged;

  @override
  Widget build(BuildContext context) {
    final t = context.t;
    final selectedKey =
        selected == null ? null : '${selected!.recordId}:${selected!.version}';
    final values =
        records.map((record) => '${record.recordId}:${record.version}').toSet();
    return DropdownButtonFormField<String>(
      key: ValueKey(selectedKey ?? 'none'),
      initialValue: values.contains(selectedKey) ? selectedKey : null,
      decoration: InputDecoration(labelText: t.selectRecord),
      items: records
          .map((record) => DropdownMenuItem(
              value: '${record.recordId}:${record.version}',
              child: Text(
                  '${record.displayName} · ${t.version} ${record.version} · ${record.accountHint.isEmpty ? '-' : record.accountHint}')))
          .toList(),
      onChanged: (value) {
        final record = records
            .firstWhere((item) => '${item.recordId}:${item.version}' == value);
        onChanged(record);
      },
    );
  }
}

class _DerivePage extends StatefulWidget {
  const _DerivePage({
    required this.records,
    required this.selected,
    required this.timeoutSeconds,
    required this.onSelect,
    required this.api,
  });

  final List<CredentialRecord> records;
  final CredentialRecord? selected;
  final int timeoutSeconds;
  final ValueChanged<CredentialRecord> onSelect;
  final CoreApi? api;

  @override
  State<_DerivePage> createState() => _DerivePageState();
}

class _DerivePageState extends State<_DerivePage> {
  final _mnemonic = TextEditingController();
  Timer? _timer;
  String? _password;
  String? _message;
  bool _show = false;

  @override
  void dispose() {
    _timer?.cancel();
    _mnemonic.dispose();
    super.dispose();
  }

  Future<void> _derive() async {
    final api = widget.api;
    final record = widget.selected;
    if (api == null || record == null) return;
    final clipboardClearFailed = context.t.clipboardClearFailed;
    try {
      final response = await api.derivePassword(
        recordId: record.recordId,
        version: record.version,
        mnemonic: _mnemonic.text,
      );
      final password = response['password'] as String;
      await Clipboard.setData(ClipboardData(text: password));
      _timer?.cancel();
      _timer = Timer(Duration(seconds: widget.timeoutSeconds), () async {
        try {
          await Clipboard.setData(const ClipboardData(text: ''));
        } catch (_) {
          if (mounted) setState(() => _message = clipboardClearFailed);
        } finally {
          if (mounted) setState(() => _password = null);
        }
      });
      setState(() {
        _password = password;
        _show = false;
        _message = context.t.passwordCopied;
      });
    } catch (_) {
      setState(() => _message = context.t.operationFailed);
    } finally {
      _mnemonic.clear();
    }
  }

  @override
  Widget build(BuildContext context) {
    final t = context.t;
    return Padding(
      padding: const EdgeInsets.all(28),
      child: ListView(
        children: [
          PageTitle(title: t.derivePassword, subtitle: t.clearOnLeave),
          const SizedBox(height: 18),
          SectionPanel(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                _RecordPicker(
                    records: widget.records,
                    selected: widget.selected,
                    onChanged: widget.onSelect),
                const SizedBox(height: 12),
                InlineNotice(
                  text: t.normalDerivationHelp,
                  icon: Icons.devices_rounded,
                ),
                const SizedBox(height: 12),
                TextField(
                    controller: _mnemonic,
                    obscureText: true,
                    decoration: InputDecoration(labelText: t.mnemonicPhrase)),
                const SizedBox(height: 16),
                if (_password != null)
                  Container(
                    width: double.infinity,
                    padding: const EdgeInsets.all(14),
                    decoration: BoxDecoration(
                        color: KpColors.surfaceSoft,
                        border: Border.all(color: KpColors.primary),
                        borderRadius: BorderRadius.circular(8)),
                    child: SelectableText(
                        _show ? _password! : '••••••••••••••••',
                        style: Theme.of(context).textTheme.titleMedium),
                  ),
                if (_message != null) ...[
                  const SizedBox(height: 12),
                  InlineNotice(text: _message!, tone: KpColors.success)
                ],
                const SizedBox(height: 16),
                Wrap(
                  spacing: 12,
                  children: [
                    FilledButton.icon(
                        onPressed: widget.selected == null ? null : _derive,
                        icon: const Icon(Icons.content_copy_rounded),
                        label: Text(t.deriveAndCopy)),
                    OutlinedButton.icon(
                        onPressed: _password == null
                            ? null
                            : () => setState(() => _show = !_show),
                        icon: Icon(_show
                            ? Icons.visibility_off_rounded
                            : Icons.visibility_rounded),
                        label: Text(_show ? t.hidePassword : t.showPassword)),
                  ],
                ),
              ],
            ),
          ),
        ],
      ),
    );
  }
}

class _RotationPage extends StatefulWidget {
  const _RotationPage({
    required this.records,
    required this.selected,
    required this.defaultLength,
    required this.api,
    required this.onSelect,
    required this.onDerivePending,
    required this.onDone,
  });

  final List<CredentialRecord> records;
  final CredentialRecord? selected;
  final int defaultLength;
  final CoreApi? api;
  final ValueChanged<CredentialRecord> onSelect;
  final ValueChanged<CredentialRecord> onDerivePending;
  final Future<void> Function() onDone;

  @override
  State<_RotationPage> createState() => _RotationPageState();
}

class _RotationPageState extends State<_RotationPage> {
  String? _message;

  List<_RecordVersionGroup> get _groups => _groupRecordVersions(widget.records);

  _RecordVersionGroup? get _selectedGroup {
    final groups = _groups;
    if (groups.isEmpty) return null;
    final selected = widget.selected;
    if (selected == null) return groups.first;
    return groups.firstWhere(
      (group) => group.current.recordId == selected.recordId,
      orElse: () => groups.first,
    );
  }

  Future<void> _create() async {
    final api = widget.api;
    final record = _selectedGroup?.current;
    if (api == null || record == null) return;
    try {
      final length = (record.encodingDescriptor['length'] as num?)?.toInt() ??
          widget.defaultLength;
      final pending =
          await api.rotateCredential(record: record, length: length);
      widget.onSelect(pending);
      await widget.onDone();
      setState(() => _message = context.t.newVersionCreated);
    } catch (_) {
      setState(() => _message = context.t.operationFailed);
    }
  }

  @override
  Widget build(BuildContext context) {
    final t = context.t;
    final group = _selectedGroup;
    final record = group?.current;
    final previous = group?.previous;
    final currentRecords = _groups.map((group) => group.current).toList();
    return Padding(
      padding: const EdgeInsets.all(28),
      child: ListView(
        children: [
          PageTitle(title: t.rotation, subtitle: t.oldVersionRemainsActive),
          const SizedBox(height: 18),
          SectionPanel(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                if (currentRecords.isEmpty)
                  Text(t.noRecords)
                else
                  _RecordPicker(
                      records: currentRecords,
                      selected: record,
                      onChanged: widget.onSelect),
                const SizedBox(height: 16),
                InlineNotice(
                  text: record == null
                      ? t.rotationNoRecord
                      : t.rotationCreateHelp,
                  icon: Icons.rotate_right_rounded,
                  tone: KpColors.primary,
                ),
                if (record != null) ...[
                  const SizedBox(height: 16),
                  InfoRow(
                      label: t.currentVersion,
                      value:
                          '${record.displayName} · ${t.version} ${record.version} · ${_stateLabelForRotation(t, record.state)}'),
                  if (previous != null)
                    InfoRow(
                        label: t.previousVersion,
                        value:
                            '${previous.displayName} · ${t.version} ${previous.version} · ${_stateLabelForRotation(t, previous.state)}'),
                  const SizedBox(height: 12),
                  Wrap(
                    spacing: 12,
                    runSpacing: 12,
                    children: [
                      FilledButton.icon(
                          onPressed: () => widget.onDerivePending(record),
                          icon: const Icon(Icons.password_rounded),
                          label: Text(t.deriveCurrentVersion)),
                      if (previous != null)
                        OutlinedButton.icon(
                            onPressed: () => widget.onDerivePending(previous),
                            icon: const Icon(Icons.history_rounded),
                            label: Text(t.derivePreviousVersion)),
                      OutlinedButton.icon(
                          onPressed: _create,
                          icon: const Icon(Icons.rotate_right_rounded),
                          label: Text(t.createNewVersion)),
                    ],
                  ),
                ],
                if (_message != null) ...[
                  const SizedBox(height: 12),
                  InlineNotice(text: _message!)
                ],
                const SizedBox(height: 16),
              ],
            ),
          ),
        ],
      ),
    );
  }

  String _stateLabelForRotation(AppLocalizations t, String state) {
    return switch (state) {
      'active' => t.active,
      'pending_rotation' => t.pending,
      'retired' => t.retired,
      'conflict' => t.conflict,
      'error' => t.error,
      _ => state,
    };
  }
}

class _RecoveryPanel extends StatefulWidget {
  const _RecoveryPanel({
    required this.api,
    required this.usbCandidates,
    required this.onDone,
    this.initialMode = _RecoveryMode.usbLost,
    this.allowedModes = const {
      _RecoveryMode.usbLost,
      _RecoveryMode.newDevice,
      _RecoveryMode.resetMnemonic,
    },
  });

  final CoreApi? api;
  final List<UsbCandidate> usbCandidates;
  final Future<void> Function() onDone;
  final _RecoveryMode initialMode;
  final Set<_RecoveryMode> allowedModes;

  @override
  State<_RecoveryPanel> createState() => _RecoveryPanelState();
}

class _RecoveryPanelState extends State<_RecoveryPanel> {
  final _mnemonic = TextEditingController();
  final _confirmMnemonic = TextEditingController();
  final _usbPath = TextEditingController();
  late _RecoveryMode _mode;
  String? _message;
  bool _busy = false;

  @override
  void initState() {
    super.initState();
    _mode = _resolvedInitialMode();
    if (widget.usbCandidates.isNotEmpty) {
      _usbPath.text = widget.usbCandidates.first.rootPath;
    }
  }

  @override
  void didUpdateWidget(covariant _RecoveryPanel oldWidget) {
    super.didUpdateWidget(oldWidget);
    if (!widget.allowedModes.contains(_mode)) {
      _mode = _resolvedInitialMode();
    }
    if (_usbPath.text.isEmpty && widget.usbCandidates.isNotEmpty) {
      _usbPath.text = widget.usbCandidates.first.rootPath;
    }
  }

  _RecoveryMode _resolvedInitialMode() {
    if (widget.allowedModes.contains(widget.initialMode)) {
      return widget.initialMode;
    }
    return widget.allowedModes.first;
  }

  @override
  void dispose() {
    _mnemonic.dispose();
    _confirmMnemonic.dispose();
    _usbPath.dispose();
    super.dispose();
  }

  Future<void> _submit() async {
    final api = widget.api;
    final mnemonic = _mnemonic.text.trim();
    final usbPath = _usbPath.text.trim();
    if (api == null || _busy || mnemonic.isEmpty || usbPath.isEmpty) {
      return;
    }
    if (_mode == _RecoveryMode.resetMnemonic &&
        mnemonic != _confirmMnemonic.text.trim()) {
      setState(() => _message = context.t.newMnemonicMismatch);
      return;
    }
    setState(() {
      _busy = true;
      _message = null;
    });
    try {
      if (_mode == _RecoveryMode.usbLost) {
        await api.recoverUsb(mnemonic: mnemonic, usbPath: usbPath);
      } else if (_mode == _RecoveryMode.newDevice) {
        await api.recoverLocal(mnemonic: mnemonic, usbPath: usbPath);
      } else {
        await api.resetMnemonic(newMnemonic: mnemonic, usbPath: usbPath);
      }
      await widget.onDone();
      if (mounted) {
        setState(() => _message = _mode == _RecoveryMode.resetMnemonic
            ? context.t.mnemonicResetComplete
            : context.t.recoveryComplete);
      }
    } catch (_) {
      if (mounted) setState(() => _message = context.t.operationFailed);
    } finally {
      _mnemonic.clear();
      _confirmMnemonic.clear();
      if (mounted) setState(() => _busy = false);
    }
  }

  IconData _modeIcon(_RecoveryMode mode) {
    return switch (mode) {
      _RecoveryMode.usbLost => Icons.usb_rounded,
      _RecoveryMode.newDevice => Icons.computer_rounded,
      _RecoveryMode.resetMnemonic => Icons.key_rounded,
    };
  }

  String _modeTitle(AppLocalizations t, _RecoveryMode mode) {
    return switch (mode) {
      _RecoveryMode.usbLost => t.rebuildUsbPackage,
      _RecoveryMode.newDevice => t.recoverLocal,
      _RecoveryMode.resetMnemonic => t.resetMnemonic,
    };
  }

  String _modeFactors(AppLocalizations t, _RecoveryMode mode) {
    return switch (mode) {
      _RecoveryMode.usbLost => t.recoveryPathMnemonicComputer,
      _RecoveryMode.newDevice => t.recoveryPathMnemonicUsb,
      _RecoveryMode.resetMnemonic => t.recoveryPathComputerUsb,
    };
  }

  String _modeExplanation(AppLocalizations t, _RecoveryMode mode) {
    return switch (mode) {
      _RecoveryMode.usbLost => t.rebuildUsbExplanation,
      _RecoveryMode.newDevice => t.recoverComputerExplanation,
      _RecoveryMode.resetMnemonic => t.resetMnemonicExplanation,
    };
  }

  String _selectedHelp(AppLocalizations t, _RecoveryMode mode) {
    return switch (mode) {
      _RecoveryMode.usbLost => t.usbLostHelp,
      _RecoveryMode.newDevice => t.recoverLocalHelp,
      _RecoveryMode.resetMnemonic => t.resetMnemonicFactorHelp,
    };
  }

  Widget _pathCard(AppLocalizations t, _RecoveryMode mode) {
    final selected = _mode == mode;
    final borderColor = selected ? KpColors.primary : KpColors.hairlineStrong;
    return InkWell(
      borderRadius: BorderRadius.circular(8),
      onTap: _busy ? null : () => setState(() => _mode = mode),
      child: AnimatedContainer(
        duration: const Duration(milliseconds: 140),
        padding: const EdgeInsets.all(14),
        decoration: BoxDecoration(
          borderRadius: BorderRadius.circular(8),
          border: Border.all(color: borderColor),
          color: selected ? KpColors.surfaceSoft : Colors.transparent,
        ),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Icon(_modeIcon(mode), color: borderColor),
            const SizedBox(height: 10),
            Text(_modeTitle(t, mode),
                style: Theme.of(context).textTheme.titleSmall),
            const SizedBox(height: 6),
            Text(_modeFactors(t, mode),
                style: Theme.of(context).textTheme.labelMedium),
            const SizedBox(height: 8),
            Text(_modeExplanation(t, mode),
                style: Theme.of(context).textTheme.bodySmall),
          ],
        ),
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    final t = context.t;
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Text(t.recovery, style: Theme.of(context).textTheme.titleMedium),
        const SizedBox(height: 6),
        Text(t.singleFactorNotEnough,
            style: Theme.of(context).textTheme.bodyMedium),
        const SizedBox(height: 16),
        if (widget.allowedModes.length > 1) ...[
          _ResponsiveGrid(
            minItemWidth: 240,
            children: [
              for (final mode in const [
                _RecoveryMode.usbLost,
                _RecoveryMode.newDevice,
                _RecoveryMode.resetMnemonic,
              ])
                if (widget.allowedModes.contains(mode)) _pathCard(t, mode),
            ],
          ),
          const SizedBox(height: 16),
        ] else ...[
          _pathCard(t, _mode),
          const SizedBox(height: 16),
        ],
        InlineNotice(text: _selectedHelp(t, _mode), icon: _modeIcon(_mode)),
        const SizedBox(height: 12),
        TextField(
          controller: _mnemonic,
          obscureText: true,
          decoration: InputDecoration(
              labelText: _mode == _RecoveryMode.resetMnemonic
                  ? t.newMnemonicPhrase
                  : t.mnemonicPhrase),
        ),
        if (_mode == _RecoveryMode.resetMnemonic) ...[
          const SizedBox(height: 12),
          TextField(
            controller: _confirmMnemonic,
            obscureText: true,
            decoration: InputDecoration(labelText: t.confirmNewMnemonicPhrase),
          ),
        ],
        const SizedBox(height: 12),
        TextField(
          controller: _usbPath,
          decoration: InputDecoration(
            labelText: t.usbPath,
            suffixIcon: IconButton(
              tooltip: t.chooseUsb,
              onPressed: () => _chooseUsbPath(_usbPath),
              icon: const Icon(Icons.folder_open_rounded),
            ),
          ),
        ),
        if (_message != null) ...[
          const SizedBox(height: 12),
          InlineNotice(text: _message!)
        ],
        const SizedBox(height: 16),
        FilledButton.icon(
            onPressed: _busy ? null : _submit,
            icon: const Icon(Icons.settings_backup_restore_rounded),
            label: Text(_busy ? '...' : t.runRecovery)),
      ],
    );
  }
}
