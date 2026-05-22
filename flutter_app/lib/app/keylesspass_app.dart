import 'dart:async';
import 'dart:io';

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';

import 'app_theme.dart';
import '../ffi/core_api.dart';
import '../ffi/rust_core.dart';
import '../models/core_models.dart';
import '../widgets/desktop_widgets.dart';

enum _View { services, add, enroll, recovery, security, settings }

UsbCandidate? _preferredUsbCandidate(List<UsbCandidate> candidates, {bool requireReadable = false}) {
  if (candidates.isEmpty) return null;
  if (requireReadable) {
    for (final candidate in candidates) {
      if (candidate.readable) return candidate;
    }
  }
  return candidates.first;
}

class KeylessPassDesktopApp extends StatelessWidget {
  const KeylessPassDesktopApp({super.key});

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      debugShowCheckedModeBanner: false,
      title: 'KeylessPass',
      theme: buildKeylessPassTheme(),
      home: const _HomeWindow(),
    );
  }
}

class _HomeWindow extends StatefulWidget {
  const _HomeWindow();

  @override
  State<_HomeWindow> createState() => _HomeWindowState();
}

class _HomeWindowState extends State<_HomeWindow> {
  CoreApi? _api;
  _View _view = _View.services;
  AppStatus? _status;
  List<CredentialRecord> _records = [];
  List<UsbCandidate> _usbCandidates = [];
  CredentialRecord? _selected;
  String? _error;
  bool _busy = false;

  @override
  void initState() {
    super.initState();
    try {
      _api = CoreApi(RustCore.instance);
      _refresh();
    } catch (error) {
      _error = '$error';
    }
  }

  Future<void> _refresh() async {
    final api = _api;
    if (api == null) return;
    setState(() {
      _busy = true;
      _error = null;
    });
    try {
      final status = await api.getAppStatus();
      final records = status.enrolled ? await api.listCredentials() : <CredentialRecord>[];
      final candidates = await api.listUsbCandidates();
      setState(() {
        _status = status;
        _records = records;
        _usbCandidates = candidates;
        _selected = records.isEmpty
            ? null
            : records.firstWhere(
                (item) => item.recordId == _selected?.recordId,
                orElse: () => records.first,
              );
        if (!status.enrolled) {
          _view = _View.enroll;
        }
      });
    } catch (error) {
      setState(() => _error = '$error');
    } finally {
      setState(() => _busy = false);
    }
  }

  @override
  Widget build(BuildContext context) {
    return CallbackShortcuts(
      bindings: {
        SingleActivator(LogicalKeyboardKey.keyN, control: !Platform.isMacOS, meta: Platform.isMacOS): () {
          setState(() => _view = _View.add);
        },
        SingleActivator(LogicalKeyboardKey.keyR, control: !Platform.isMacOS, meta: Platform.isMacOS): () {
          if (_selected != null) _showRotationDialog(_selected!);
        },
        SingleActivator(LogicalKeyboardKey.keyD, control: !Platform.isMacOS, meta: Platform.isMacOS): () {
          if (_selected != null) _showDeriveDialog(_selected!);
        },
      },
      child: Focus(
        autofocus: true,
        child: Scaffold(
          backgroundColor: KpColors.canvas,
          body: Row(
            children: [
              _buildNavigation(),
              const VerticalDivider(width: 1, color: KpColors.hairline),
              Expanded(child: _buildContent()),
            ],
          ),
        ),
      ),
    );
  }

  Widget _buildNavigation() {
    return Container(
      width: 284,
      color: KpColors.canvas,
      child: Column(
        children: [
          Container(
            height: 86,
            padding: const EdgeInsets.symmetric(horizontal: 18, vertical: 14),
            alignment: Alignment.centerLeft,
            child: Row(
              children: [
                Container(
                  width: 38,
                  height: 38,
                  decoration: BoxDecoration(
                    color: KpColors.primary,
                    borderRadius: BorderRadius.circular(8),
                  ),
                  child: const Icon(Icons.key_rounded, size: 22, color: KpColors.canvas),
                ),
                const SizedBox(width: 12),
                Expanded(
                  child: Column(
                    mainAxisAlignment: MainAxisAlignment.center,
                    mainAxisSize: MainAxisSize.min,
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      const Text(
                        'KeylessPass',
                        maxLines: 1,
                        overflow: TextOverflow.ellipsis,
                        style: TextStyle(fontSize: 18, fontWeight: FontWeight.w800, color: KpColors.ink),
                      ),
                      Text(
                        _status?.securityStatus.provider ?? 'Rust Core',
                        maxLines: 1,
                        overflow: TextOverflow.ellipsis,
                        style: const TextStyle(fontSize: 12, color: KpColors.muted),
                      ),
                    ],
                  ),
                ),
              ],
            ),
          ),
          Padding(
            padding: const EdgeInsets.fromLTRB(18, 0, 18, 14),
            child: Row(
              children: [
                StatusPill(
                  label: _status?.enrolled == true ? '已初始化' : '待初始化',
                  tone: _status?.enrolled == true ? KpColors.success : KpColors.warning,
                ),
                const SizedBox(width: 8),
                StatusPill(
                  label: _status?.securityStatus.degraded == true ? '降级保护' : '平台保护',
                  tone: _status?.securityStatus.degraded == true ? KpColors.warning : KpColors.primary,
                ),
              ],
            ),
          ),
          SizedBox(
            height: 330,
            child: NavigationRail(
              extended: true,
              selectedIndex: _view.index,
              onDestinationSelected: (index) => setState(() => _view = _View.values[index]),
              destinations: const [
                NavigationRailDestination(icon: Icon(Icons.view_list_rounded), label: Text('服务')),
                NavigationRailDestination(icon: Icon(Icons.add_rounded), label: Text('添加')),
                NavigationRailDestination(icon: Icon(Icons.verified_user_rounded), label: Text('初始化')),
                NavigationRailDestination(icon: Icon(Icons.settings_backup_restore_rounded), label: Text('恢复')),
                NavigationRailDestination(icon: Icon(Icons.security_rounded), label: Text('安全状态')),
                NavigationRailDestination(icon: Icon(Icons.tune_rounded), label: Text('设置')),
              ],
            ),
          ),
          const Divider(height: 1, color: KpColors.hairline),
          Expanded(child: _buildRecordList()),
          Padding(
            padding: const EdgeInsets.all(12),
            child: Row(
              children: [
                Expanded(
                  child: Text(
                    _busy ? '处理中...' : '离线本机模式',
                    overflow: TextOverflow.ellipsis,
                    style: const TextStyle(color: KpColors.muted),
                  ),
                ),
                IconButton(tooltip: '刷新', onPressed: _refresh, icon: const Icon(Icons.refresh_rounded)),
              ],
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildRecordList() {
    if (_records.isEmpty) {
      return const Center(
        child: Padding(
          padding: EdgeInsets.all(20),
          child: Text('暂无 CDR\n初始化后添加服务记录', textAlign: TextAlign.center, style: TextStyle(color: KpColors.muted)),
        ),
      );
    }
    return ListView.builder(
      padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 12),
      itemCount: _records.length,
      itemBuilder: (context, index) {
        final record = _records[index];
        final selected = record.recordId == _selected?.recordId;
        return Padding(
          padding: const EdgeInsets.only(bottom: 8),
          child: ListTile(
            selected: selected,
            tileColor: KpColors.surfaceSoft,
            selectedTileColor: KpColors.surfaceCard,
            shape: RoundedRectangleBorder(
              side: BorderSide(color: selected ? KpColors.primary : KpColors.hairline),
              borderRadius: BorderRadius.circular(8),
            ),
            title: Text(record.displayName, overflow: TextOverflow.ellipsis),
            subtitle: Text('seq ${record.recordSeq}  /  v${record.version}  /  ${record.state}'),
            trailing: selected ? const Icon(Icons.chevron_right_rounded, color: KpColors.primary) : null,
            onTap: () => setState(() {
              _selected = record;
              _view = _View.services;
            }),
          ),
        );
      },
    );
  }

  Widget _buildContent() {
    final api = _api;
    if (_error != null) {
      return _pageShell(
        title: '系统提示',
        child: SectionPanel(
          child: Text(_error!, style: const TextStyle(color: KpColors.error)),
        ),
      );
    }
    if (api == null) {
      return _pageShell(
        title: 'Rust Core 未加载',
        child: const SectionPanel(child: Text('请先构建 rust_core 动态库，或检查应用打包是否包含动态库。')),
      );
    }
    if (_status?.enrolled != true && _view != _View.enroll) {
      return _EnrollmentPage(api: api, usbCandidates: _usbCandidates, enrolled: false, onDone: _refresh);
    }
    return switch (_view) {
      _View.enroll => _EnrollmentPage(api: api, usbCandidates: _usbCandidates, enrolled: _status?.enrolled == true, onDone: _refresh),
      _View.add => _AddCredentialPage(api: api, onDone: _refresh),
      _View.recovery => _RecoveryPage(api: api, usbCandidates: _usbCandidates, onDone: _refresh),
      _View.security => _SecurityPage(status: _status, usbCandidates: _usbCandidates),
      _View.settings => _SettingsPage(status: _status),
      _View.services => _ServiceDetailPage(
          record: _selected,
          onDerive: _selected == null ? null : () => _showDeriveDialog(_selected!),
          onRotate: _selected == null ? null : () => _showRotationDialog(_selected!),
        ),
    };
  }

  Widget _pageShell({required String title, required Widget child}) {
    return Padding(
      padding: const EdgeInsets.all(32),
      child: Column(crossAxisAlignment: CrossAxisAlignment.start, children: [
        PageTitle(title: title, subtitle: 'KeylessPass 本机安全客户端 · Rust Core'),
        const SizedBox(height: 16),
        Expanded(child: child),
      ]),
    );
  }

  Future<void> _showDeriveDialog(CredentialRecord record) async {
    final api = _api;
    if (api == null) return;
    await showDialog<void>(
      context: context,
      builder: (context) => _DeriveDialog(api: api, record: record, usbCandidates: _usbCandidates),
    );
  }

  Future<void> _showRotationDialog(CredentialRecord record) async {
    final api = _api;
    if (api == null) return;
    await showDialog<void>(
      context: context,
      builder: (context) => _RotationDialog(api: api, record: record, onDone: _refresh),
    );
  }
}

class _EnrollmentPage extends StatefulWidget {
  const _EnrollmentPage({required this.api, required this.usbCandidates, required this.enrolled, required this.onDone});

  final CoreApi api;
  final List<UsbCandidate> usbCandidates;
  final bool enrolled;
  final Future<void> Function() onDone;

  @override
  State<_EnrollmentPage> createState() => _EnrollmentPageState();
}

class _EnrollmentPageState extends State<_EnrollmentPage> {
  final _mnemonic = TextEditingController();
  final _usbPath = TextEditingController();
  String? _message;
  bool _busy = false;

  @override
  void initState() {
    super.initState();
    _fillUsbPathIfEmpty();
  }

  @override
  void didUpdateWidget(covariant _EnrollmentPage oldWidget) {
    super.didUpdateWidget(oldWidget);
    _fillUsbPathIfEmpty();
  }

  void _fillUsbPathIfEmpty() {
    if (_usbPath.text.trim().isNotEmpty) return;
    final candidate = _preferredUsbCandidate(widget.usbCandidates);
    if (candidate != null) {
      _usbPath.text = candidate.rootPath;
    }
  }

  @override
  void dispose() {
    _mnemonic.dispose();
    _usbPath.dispose();
    super.dispose();
  }

  Future<void> _submit() async {
    if (widget.enrolled) {
      setState(() {
        _message = '当前设备已初始化。为避免覆盖 K_master、本机因子包和 USB 因子包，普通初始化已锁定；如需补 USB，请使用恢复向导。';
      });
      return;
    }
    setState(() {
      _busy = true;
      _message = null;
    });
    try {
      await widget.api.enroll(mnemonic: _mnemonic.text, usbPath: _usbPath.text);
      setState(() => _message = '初始化完成。Mnemonic 未保存，服务密码未保存。');
      await widget.onDone();
    } catch (error) {
      setState(() => _message = '$error');
    } finally {
      setState(() => _busy = false);
    }
  }

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.all(32),
      child: ListView(
        children: [
          const PageTitle(
            title: '初始化向导',
            subtitle: '生成本机因子、USB 因子和 CDR store；不保存 mnemonic，也不保存服务密码。',
          ),
          const SizedBox(height: 18),
          const WorkflowStepper(steps: ['Mnemonic', '本机因子', 'USB 因子', 'CDR Store'], current: 0),
          const SizedBox(height: 18),
          SectionPanel(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                InlineNotice(
                  text: '当前发现 ${widget.usbCandidates.length} 个可移动路径。初始化只会写入 KeylessPass 因子包，不会写入或保存任何服务密码。',
                  icon: Icons.usb_rounded,
                  tone: widget.usbCandidates.isEmpty ? KpColors.warning : KpColors.primary,
                ),
                if (widget.enrolled) ...[
                  const SizedBox(height: 12),
                  const InlineNotice(
                    text: '此设备已经初始化。重新初始化会生成新的 K_master，旧 CDR 将无法再派生出原服务密码，因此已禁止在普通初始化页面覆盖现有状态。USB 丢失请进入恢复向导重建 USB factor package。',
                    icon: Icons.lock_rounded,
                    tone: KpColors.warning,
                  ),
                ],
                const SizedBox(height: 16),
                TextField(
                  controller: _mnemonic,
                  obscureText: true,
                  enabled: !widget.enrolled,
                  decoration: const InputDecoration(labelText: 'Mnemonic phrase'),
                ),
                const SizedBox(height: 12),
                TextField(
                  controller: _usbPath,
                  enabled: !widget.enrolled,
                  decoration: InputDecoration(
                    labelText: 'USB 路径',
                    suffixIcon: PopupMenuButton<String>(
                      tooltip: '使用自动发现路径',
                      onSelected: (value) => _usbPath.text = value,
                      itemBuilder: (context) => [
                        for (final item in widget.usbCandidates)
                          PopupMenuItem(value: item.rootPath, child: Text(item.rootPath)),
                      ],
                      icon: const Icon(Icons.usb_rounded),
                    ),
                  ),
                ),
                const SizedBox(height: 16),
                const Wrap(
                  spacing: 12,
                  runSpacing: 12,
                  children: [
                    SizedBox(width: 250, child: SignalTile(label: '保存', value: '状态包')),
                    SizedBox(width: 250, child: SignalTile(label: '不保存', value: '口令库')),
                    SizedBox(width: 250, child: SignalTile(label: '派生路径', value: 'CDR 固定字段')),
                  ],
                ),
                const SizedBox(height: 16),
                const InfoRow(label: '保存', value: '受保护本机包、USB 因子包、CDR、恢复元数据'),
                const InfoRow(label: '不保存', value: '目标系统密码、加密口令库、mnemonic phrase'),
                const InfoRow(label: '会变化', value: 'recordSeq、recordId、version、salt、encodingDescriptor'),
                const InfoRow(label: '不会变化', value: 'displayName、serviceHint、accountHint'),
                if (_message != null) ...[
                  const SizedBox(height: 12),
                  InlineNotice(text: _message!, tone: KpColors.warning),
                ],
                const SizedBox(height: 16),
                Align(
                  alignment: Alignment.centerRight,
                  child: FilledButton.icon(
                    onPressed: _busy || widget.enrolled ? null : _submit,
                    icon: const Icon(Icons.verified_user_rounded),
                    label: Text(widget.enrolled ? '已初始化，禁止覆盖' : (_busy ? '初始化中...' : '创建本机因子和 USB 因子')),
                  ),
                ),
              ],
            ),
          ),
        ],
      ),
    );
  }
}

class _AddCredentialPage extends StatefulWidget {
  const _AddCredentialPage({required this.api, required this.onDone});

  final CoreApi api;
  final Future<void> Function() onDone;

  @override
  State<_AddCredentialPage> createState() => _AddCredentialPageState();
}

class _AddCredentialPageState extends State<_AddCredentialPage> {
  final _displayName = TextEditingController();
  final _serviceHint = TextEditingController();
  final _accountHint = TextEditingController();
  double _length = 18;
  String? _message;

  @override
  void dispose() {
    _displayName.dispose();
    _serviceHint.dispose();
    _accountHint.dispose();
    super.dispose();
  }

  Future<void> _submit() async {
    try {
      await widget.api.addCredential(
        displayName: _displayName.text,
        serviceHint: _serviceHint.text,
        accountHint: _accountHint.text,
        length: _length.round(),
      );
      _displayName.clear();
      _serviceHint.clear();
      _accountHint.clear();
      setState(() => _message = 'CDR 已创建。展示字段不参与派生。');
      await widget.onDone();
    } catch (error) {
      setState(() => _message = '$error');
    }
  }

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.all(32),
      child: ListView(
        children: [
          const PageTitle(
            title: '添加服务记录',
            subtitle: '创建新的 CDR。服务名、URL、账号标签只用于展示和搜索。',
          ),
          const SizedBox(height: 18),
          SectionPanel(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                const InlineNotice(
                  text: '派生路径只绑定 recordSeq、recordId、version 和 salt。后续修改显示名称、服务提示或账号提示，不会改变已派生密码。',
                  icon: Icons.route_rounded,
                ),
                const SizedBox(height: 16),
                TextField(controller: _displayName, decoration: const InputDecoration(labelText: '显示名称')),
                const SizedBox(height: 12),
                TextField(controller: _serviceHint, decoration: const InputDecoration(labelText: '服务提示')),
                const SizedBox(height: 12),
                TextField(controller: _accountHint, decoration: const InputDecoration(labelText: '账号提示')),
                const SizedBox(height: 12),
                Row(
                  children: [
                    SizedBox(width: 110, child: Text('密码长度 ${_length.round()}')),
                    Expanded(
                      child: Slider(
                        min: 8,
                        max: 64,
                        divisions: 56,
                        value: _length,
                        onChanged: (value) => setState(() => _length = value),
                      ),
                    ),
                  ],
                ),
                if (_message != null) ...[
                  const SizedBox(height: 12),
                  InlineNotice(text: _message!, tone: KpColors.warning),
                ],
                const SizedBox(height: 12),
                Align(
                  alignment: Alignment.centerRight,
                  child: FilledButton.icon(onPressed: _submit, icon: const Icon(Icons.add_rounded), label: const Text('保存 CDR')),
                ),
              ],
            ),
          ),
        ],
      ),
    );
  }
}

class _ServiceDetailPage extends StatelessWidget {
  const _ServiceDetailPage({required this.record, required this.onDerive, required this.onRotate});

  final CredentialRecord? record;
  final VoidCallback? onDerive;
  final VoidCallback? onRotate;

  @override
  Widget build(BuildContext context) {
    final record = this.record;
    if (record == null) {
      return const Center(child: Text('请选择或添加一个服务记录。', style: TextStyle(color: KpColors.muted)));
    }
    return Padding(
      padding: const EdgeInsets.all(32),
      child: ListView(
        children: [
          PageTitle(
            title: record.displayName,
            subtitle: record.serviceHint.isEmpty ? 'CDR seq ${record.recordSeq} · version ${record.version}' : record.serviceHint,
            trailing: Row(
              mainAxisSize: MainAxisSize.min,
              children: [
                FilledButton.icon(onPressed: onDerive, icon: const Icon(Icons.copy_rounded), label: const Text('派生')),
                const SizedBox(width: 8),
                OutlinedButton.icon(onPressed: onRotate, icon: const Icon(Icons.rotate_right_rounded), label: const Text('轮换')),
              ],
            ),
          ),
          const SizedBox(height: 18),
          Wrap(
            spacing: 12,
            runSpacing: 12,
            children: [
              SizedBox(width: 180, child: SignalTile(label: 'Seq', value: '${record.recordSeq}')),
              SizedBox(width: 180, child: SignalTile(label: 'Version', value: 'v${record.version}')),
              SizedBox(width: 220, child: SignalTile(label: 'State', value: record.state, tone: record.state == 'active' ? KpColors.success : KpColors.warning)),
            ],
          ),
          const SizedBox(height: 18),
          const InlineNotice(
            text: '这里的 serviceHint 和 accountHint 只用于识别记录。派生密码时，Rust Core 使用固定 CDR 字段和三个因子，不读取这些展示文本。',
            icon: Icons.lock_outline_rounded,
          ),
          const SizedBox(height: 18),
          SectionPanel(
            child: Column(
              children: [
                InfoRow(label: 'Record ID', value: record.recordId),
                InfoRow(label: 'Record Seq', value: '${record.recordSeq}'),
                InfoRow(label: 'Version', value: '${record.version}'),
                InfoRow(label: 'State', value: record.state),
                InfoRow(label: 'Service Hint', value: record.serviceHint),
                InfoRow(label: 'Account Hint', value: record.accountHint),
                InfoRow(label: 'Salt', value: record.salt),
              ],
            ),
          ),
          const SizedBox(height: 18),
          SectionPanel(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text('Encoding Descriptor', style: Theme.of(context).textTheme.titleMedium),
                const SizedBox(height: 12),
                SelectableText(
                  record.encodingDescriptor.toString(),
                  style: const TextStyle(fontFamily: 'monospace', color: KpColors.bodyStrong),
                ),
              ],
            ),
          ),
        ],
      ),
    );
  }
}

class _DeriveDialog extends StatefulWidget {
  const _DeriveDialog({required this.api, required this.record, required this.usbCandidates});

  final CoreApi api;
  final CredentialRecord record;
  final List<UsbCandidate> usbCandidates;

  @override
  State<_DeriveDialog> createState() => _DeriveDialogState();
}

class _DeriveDialogState extends State<_DeriveDialog> {
  final _mnemonic = TextEditingController();
  final _usbPath = TextEditingController();
  Timer? _clearTimer;
  String? _password;
  String? _message;

  @override
  void initState() {
    super.initState();
    _fillUsbPathIfEmpty();
  }

  @override
  void didUpdateWidget(covariant _DeriveDialog oldWidget) {
    super.didUpdateWidget(oldWidget);
    _fillUsbPathIfEmpty();
  }

  void _fillUsbPathIfEmpty() {
    if (_usbPath.text.trim().isNotEmpty) return;
    final candidate = _preferredUsbCandidate(widget.usbCandidates, requireReadable: true);
    if (candidate != null) {
      _usbPath.text = candidate.rootPath;
    }
  }

  @override
  void dispose() {
    _clearTimer?.cancel();
    _mnemonic.dispose();
    _usbPath.dispose();
    super.dispose();
  }

  Future<void> _derive() async {
    try {
      final response = await widget.api.derivePassword(
        recordId: widget.record.recordId,
        version: widget.record.version,
        mnemonic: _mnemonic.text,
        usbPath: _usbPath.text,
      );
      final password = response['password'] as String;
      await Clipboard.setData(ClipboardData(text: password));
      _clearTimer?.cancel();
      _clearTimer = Timer(const Duration(seconds: 30), () async {
        await Clipboard.setData(const ClipboardData(text: ''));
        if (mounted) setState(() => _password = null);
      });
      setState(() {
        _password = password;
        _message = '已复制到剪贴板，30 秒后自动清理。';
      });
    } catch (error) {
      setState(() => _message = '$error');
    }
  }

  @override
  Widget build(BuildContext context) {
    return AlertDialog(
      title: Text('派生密码 · ${widget.record.displayName}'),
      content: SizedBox(
        width: 560,
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          mainAxisSize: MainAxisSize.min,
          children: [
            InlineNotice(
              text: '密码只在本次会话中短暂显示并复制到剪贴板。30 秒后会自动清空剪贴板，不会写入日志或本地存储。',
              icon: Icons.timer_outlined,
              tone: _password == null ? KpColors.primary : KpColors.success,
            ),
            const SizedBox(height: 14),
            TextField(controller: _mnemonic, obscureText: true, decoration: const InputDecoration(labelText: 'Mnemonic phrase')),
            const SizedBox(height: 12),
            TextField(
              controller: _usbPath,
              decoration: InputDecoration(
                labelText: 'USB 路径',
                suffixIcon: PopupMenuButton<String>(
                  tooltip: '使用自动发现路径',
                  onSelected: (value) => _usbPath.text = value,
                  itemBuilder: (context) => [
                    for (final item in widget.usbCandidates)
                      PopupMenuItem(value: item.rootPath, child: Text(item.rootPath)),
                  ],
                  icon: const Icon(Icons.usb_rounded),
                ),
              ),
            ),
            if (_password != null) ...[
              const SizedBox(height: 16),
              Container(
                width: double.infinity,
                padding: const EdgeInsets.all(14),
                decoration: BoxDecoration(
                  color: KpColors.surfaceCard,
                  border: Border.all(color: KpColors.primary),
                  borderRadius: BorderRadius.circular(8),
                ),
                child: SelectableText(
                  _password!,
                  style: const TextStyle(fontFamily: 'monospace', fontSize: 16, color: KpColors.bodyStrong),
                ),
              ),
            ],
            if (_message != null) ...[
              const SizedBox(height: 12),
              InlineNotice(text: _message!, tone: _password == null ? KpColors.warning : KpColors.success),
            ],
          ],
        ),
      ),
      actions: [
        TextButton(onPressed: () => Navigator.pop(context), child: const Text('关闭')),
        FilledButton(onPressed: _derive, child: const Text('派生并复制')),
      ],
    );
  }
}

class _RotationDialog extends StatefulWidget {
  const _RotationDialog({required this.api, required this.record, required this.onDone});

  final CoreApi api;
  final CredentialRecord record;
  final Future<void> Function() onDone;

  @override
  State<_RotationDialog> createState() => _RotationDialogState();
}

class _RotationDialogState extends State<_RotationDialog> {
  double _length = 18;
  CredentialRecord? _pending;
  String? _message;

  @override
  void initState() {
    super.initState();
    _length = (widget.record.encodingDescriptor['length'] as num?)?.toDouble() ?? 18;
  }

  Future<void> _create() async {
    try {
      final pending = await widget.api.rotateCredential(record: widget.record, length: _length.round());
      setState(() {
        _pending = pending;
        _message = '已创建 pending_rotation v${pending.version}。请先在目标系统修改密码。';
      });
    } catch (error) {
      setState(() => _message = '$error');
    }
  }

  Future<void> _confirm() async {
    final pending = _pending;
    if (pending == null) return;
    await widget.api.confirmRotation(recordId: pending.recordId, version: pending.version);
    await widget.onDone();
    if (mounted) Navigator.pop(context);
  }

  @override
  Widget build(BuildContext context) {
    return AlertDialog(
      title: const Text('密码轮换向导'),
      content: SizedBox(
        width: 560,
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          mainAxisSize: MainAxisSize.min,
          children: [
            WorkflowStepper(steps: const ['新版本', '修改目标系统', '退休旧版本'], current: _pending == null ? 0 : 1),
            const SizedBox(height: 14),
            const InlineNotice(
              text: '轮换会创建新的 CDR version 和 salt。确认目标系统已修改成功后，旧版本才会标记 retired，期间不会保存旧密码或新密码。',
              icon: Icons.rotate_right_rounded,
            ),
            const SizedBox(height: 12),
            Row(
              children: [
                Text('长度 ${_length.round()}'),
                Expanded(child: Slider(min: 8, max: 64, divisions: 56, value: _length, onChanged: (value) => setState(() => _length = value))),
              ],
            ),
            if (_message != null) InlineNotice(text: _message!, tone: KpColors.warning),
          ],
        ),
      ),
      actions: [
        TextButton(onPressed: () => Navigator.pop(context), child: const Text('取消')),
        if (_pending == null) FilledButton(onPressed: _create, child: const Text('创建新版本')),
        if (_pending != null) FilledButton(onPressed: _confirm, child: const Text('确认修改成功')),
      ],
    );
  }
}

class _RecoveryPage extends StatefulWidget {
  const _RecoveryPage({required this.api, required this.usbCandidates, required this.onDone});

  final CoreApi api;
  final List<UsbCandidate> usbCandidates;
  final Future<void> Function() onDone;

  @override
  State<_RecoveryPage> createState() => _RecoveryPageState();
}

class _RecoveryPageState extends State<_RecoveryPage> {
  final _mnemonic = TextEditingController();
  final _usbPath = TextEditingController();
  bool _recoverUsb = true;
  String? _message;

  @override
  void initState() {
    super.initState();
    _fillUsbPathIfEmpty();
  }

  @override
  void didUpdateWidget(covariant _RecoveryPage oldWidget) {
    super.didUpdateWidget(oldWidget);
    _fillUsbPathIfEmpty();
  }

  void _fillUsbPathIfEmpty() {
    if (_usbPath.text.trim().isNotEmpty) return;
    final candidate = _preferredUsbCandidate(widget.usbCandidates, requireReadable: !_recoverUsb);
    if (candidate != null) {
      _usbPath.text = candidate.rootPath;
    }
  }

  @override
  void dispose() {
    _mnemonic.dispose();
    _usbPath.dispose();
    super.dispose();
  }

  Future<void> _submit() async {
    try {
      if (_recoverUsb) {
        await widget.api.recoverUsb(mnemonic: _mnemonic.text, usbPath: _usbPath.text);
      } else {
        await widget.api.recoverLocal(mnemonic: _mnemonic.text, usbPath: _usbPath.text);
      }
      await widget.onDone();
      setState(() => _message = '恢复完成，recovery metadata generation 已刷新。');
    } catch (error) {
      setState(() => _message = '$error');
    }
  }

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.all(32),
      child: ListView(
        children: [
          const PageTitle(
            title: '单因子恢复向导',
            subtitle: 'MVP 支持 USB 丢失和更换本机场景；单独任一因子都不足以恢复。',
          ),
          const SizedBox(height: 18),
          const WorkflowStepper(steps: ['选择模型', '验证材料', '刷新元数据'], current: 0),
          const SizedBox(height: 18),
          SectionPanel(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                SegmentedButton<bool>(
                  segments: const [
                    ButtonSegment(value: true, label: Text('USB 丢失')),
                    ButtonSegment(value: false, label: Text('更换本机')),
                  ],
                  selected: {_recoverUsb},
                  onSelectionChanged: (value) => setState(() {
                    _recoverUsb = value.first;
                    _usbPath.clear();
                    _fillUsbPathIfEmpty();
                  }),
                ),
                const SizedBox(height: 16),
                const InlineNotice(
                  text: '恢复只重建缺失因子包并刷新 recovery metadata generation；它不会改变 CDR 的 encodingDescriptor，也不会改变既有服务密码。',
                  icon: Icons.settings_backup_restore_rounded,
                ),
                const SizedBox(height: 16),
                InlineNotice(
                  text: widget.usbCandidates.isEmpty
                      ? '未发现可移动路径。也可以手动输入 Finder 中显示的挂载路径，例如 /Volumes/WD。'
                      : '已发现 ${widget.usbCandidates.length} 个可移动路径；当前会优先使用 ${_usbPath.text.isEmpty ? '手动输入路径' : _usbPath.text}。',
                  icon: Icons.usb_rounded,
                  tone: widget.usbCandidates.isEmpty ? KpColors.warning : KpColors.primary,
                ),
                const SizedBox(height: 16),
                TextField(controller: _mnemonic, obscureText: true, decoration: const InputDecoration(labelText: 'Mnemonic phrase')),
                const SizedBox(height: 12),
                TextField(
                  controller: _usbPath,
                  decoration: InputDecoration(
                    labelText: _recoverUsb ? '新 USB 路径' : '已有 USB 路径',
                    suffixIcon: PopupMenuButton<String>(
                      tooltip: '使用自动发现路径',
                      onSelected: (value) => _usbPath.text = value,
                      itemBuilder: (context) => [
                        for (final item in widget.usbCandidates)
                          PopupMenuItem(value: item.rootPath, child: Text(item.rootPath)),
                      ],
                      icon: const Icon(Icons.usb_rounded),
                    ),
                  ),
                ),
                const SizedBox(height: 12),
                const InfoRow(label: '恢复模型', value: '2-of-3-local 操作恢复'),
                const InfoRow(label: '单独因子', value: '单独 mnemonic、USB 或本机材料都不足以恢复'),
                if (_message != null) ...[
                  const SizedBox(height: 12),
                  InlineNotice(text: _message!, tone: KpColors.warning),
                ],
                const SizedBox(height: 12),
                Align(
                  alignment: Alignment.centerRight,
                  child: FilledButton.icon(
                    onPressed: _submit,
                    icon: const Icon(Icons.settings_backup_restore_rounded),
                    label: const Text('执行恢复'),
                  ),
                ),
              ],
            ),
          ),
        ],
      ),
    );
  }
}

class _SecurityPage extends StatelessWidget {
  const _SecurityPage({required this.status, required this.usbCandidates});

  final AppStatus? status;
  final List<UsbCandidate> usbCandidates;

  @override
  Widget build(BuildContext context) {
    final security = status?.securityStatus;
    return Padding(
      padding: const EdgeInsets.all(32),
      child: ListView(
        children: [
          const PageTitle(
            title: '安全状态检查',
            subtitle: '检查平台本机因子能力、恢复元数据和 USB 因子发现状态。',
          ),
          const SizedBox(height: 18),
          if (security?.degraded == true) ...[
            InlineNotice(
              text: security?.message.isNotEmpty == true
                  ? security!.message
                  : '当前处于降级保护模式。系统钥匙串不可用时仍可在离线企业环境使用，但本机状态保护能力低于平台钥匙串方案。',
              icon: Icons.warning_amber_rounded,
              tone: KpColors.warning,
            ),
            const SizedBox(height: 18),
          ],
          Wrap(
            spacing: 12,
            runSpacing: 12,
            children: [
              SizedBox(
                width: 260,
                child: SignalTile(
                  label: 'Provider',
                  value: security?.provider ?? '-',
                  tone: security?.degraded == true ? KpColors.warning : KpColors.primary,
                ),
              ),
              SizedBox(width: 220, child: SignalTile(label: 'USB', value: '${usbCandidates.length}')),
              SizedBox(width: 220, child: SignalTile(label: 'Recovery', value: '${status?.recovery?['generation'] ?? '-'}')),
            ],
          ),
          const SizedBox(height: 18),
          SectionPanel(
            child: Column(
              children: [
                InfoRow(label: 'Provider', value: security?.provider ?? '-'),
                InfoRow(label: 'Platform', value: security?.platform ?? '-'),
                InfoRow(label: 'System keystore', value: security?.systemKeystoreAvailable == true ? 'available' : 'reduced'),
                InfoRow(label: 'Message', value: security?.message ?? '-'),
                InfoRow(label: 'Recovery model', value: status?.recovery?['recoveryModel'] as String? ?? '-'),
                InfoRow(label: 'Generation', value: '${status?.recovery?['generation'] ?? '-'}'),
              ],
            ),
          ),
          const SizedBox(height: 18),
          SectionPanel(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text('USB 因子自动发现', style: Theme.of(context).textTheme.titleMedium),
                const SizedBox(height: 8),
                if (usbCandidates.isEmpty) const InlineNotice(text: '未发现 KeylessPass USB 因子包。请确认 U 盘已插入且因子包未损坏。', tone: KpColors.warning),
                for (final item in usbCandidates) InfoRow(label: item.readable ? '可读' : '异常', value: '${item.rootPath}\n${item.message}'),
              ],
            ),
          ),
        ],
      ),
    );
  }
}

class _SettingsPage extends StatelessWidget {
  const _SettingsPage({required this.status});

  final AppStatus? status;

  @override
  Widget build(BuildContext context) {
    final config = status?.config ?? const {};
    return Padding(
      padding: const EdgeInsets.all(32),
      child: ListView(
        children: [
          const PageTitle(
            title: '设置',
            subtitle: '本机客户端路径、用户标识和平台因子配置。',
          ),
          const SizedBox(height: 18),
          const InlineNotice(
            text: 'KeylessPass 第一版为严格本机部署：不联网、不云同步、不提供 Web 管理后台或浏览器自动填充入口。',
            icon: Icons.desktop_windows_rounded,
          ),
          const SizedBox(height: 18),
          SectionPanel(
            child: Column(
              children: [
                InfoRow(label: 'App version', value: '${config['appVersion'] ?? '-'}'),
                InfoRow(label: 'User ID', value: '${config['userId'] ?? '-'}'),
                InfoRow(label: 'Device ID', value: '${config['deviceId'] ?? '-'}'),
                InfoRow(label: 'CDR store', value: '${config['cdrStorePath'] ?? '-'}'),
                InfoRow(label: 'Local factor', value: '${config['localFactorPath'] ?? '-'}'),
              ],
            ),
          ),
          const SizedBox(height: 18),
          DangerPanel(
            title: '危险操作',
            body: '删除本机状态、USB 因子包或 mnemonic 遗失会导致无法派生密码。MVP 暂不提供一键清除。',
            actionLabel: '已了解',
            onPressed: () {},
          ),
        ],
      ),
    );
  }
}
