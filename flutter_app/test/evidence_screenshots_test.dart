import 'dart:io';

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:keylesspass_desktop/app/app_theme.dart';
import 'package:keylesspass_desktop/widgets/desktop_widgets.dart';

void main() {
  setUpAll(_loadEvidenceFonts);

  Future<void> renderEvidenceScreen(
    WidgetTester tester,
    Widget body,
    String goldenName, {
    int selectedIndex = 0,
    bool enrolled = true,
  }) async {
    await tester.binding.setSurfaceSize(const Size(1280, 800));
    await tester.pumpWidget(
      MaterialApp(
        debugShowCheckedModeBanner: false,
        theme: _evidenceTheme(),
        home: Scaffold(
          body: _EvidenceShell(
            selectedIndex: selectedIndex,
            enrolled: enrolled,
            body: body,
          ),
        ),
      ),
    );
    await tester.pumpAndSettle();
    await expectLater(
        find.byType(Scaffold), matchesGoldenFile('goldens/$goldenName'));
  }

  testWidgets('enrollment screenshot', (tester) async {
    await renderEvidenceScreen(
      tester,
      const _EnrollmentEvidence(),
      'enrollment.png',
      selectedIndex: 1,
      enrolled: false,
    );
  });

  testWidgets('cdr list screenshot', (tester) async {
    await renderEvidenceScreen(
      tester,
      const _CdrListEvidence(),
      'cdr_list.png',
      selectedIndex: 2,
    );
  });

  testWidgets('derive password screenshot', (tester) async {
    await renderEvidenceScreen(
      tester,
      const _DeriveEvidence(),
      'derive_password.png',
      selectedIndex: 2,
    );
  });

  testWidgets('rotation screenshot', (tester) async {
    await renderEvidenceScreen(
      tester,
      const _RotationEvidence(),
      'rotation.png',
      selectedIndex: 2,
    );
  });

  testWidgets('usb recovery screenshot', (tester) async {
    await renderEvidenceScreen(
      tester,
      const _UsbRecoveryEvidence(),
      'usb_recovery.png',
      selectedIndex: 3,
    );
  });
}

ThemeData _evidenceTheme() {
  final theme = buildKeylessPassTheme();
  return theme.copyWith(
    textTheme: theme.textTheme.apply(fontFamily: 'Roboto'),
    primaryTextTheme: theme.primaryTextTheme.apply(fontFamily: 'Roboto'),
    chipTheme: theme.chipTheme.copyWith(
      labelStyle: theme.chipTheme.labelStyle?.copyWith(fontFamily: 'Roboto'),
      secondaryLabelStyle:
          theme.chipTheme.secondaryLabelStyle?.copyWith(fontFamily: 'Roboto'),
    ),
    navigationRailTheme: theme.navigationRailTheme.copyWith(
      selectedLabelTextStyle: theme.navigationRailTheme.selectedLabelTextStyle
          ?.copyWith(fontFamily: 'Roboto'),
      unselectedLabelTextStyle: theme
          .navigationRailTheme.unselectedLabelTextStyle
          ?.copyWith(fontFamily: 'Roboto'),
    ),
  );
}

Future<void> _loadEvidenceFonts() async {
  final flutterRoot = Platform.environment['FLUTTER_ROOT'] ??
      '/Users/zhangyuanyi/development/flutter';
  final materialFonts = '$flutterRoot/bin/cache/artifacts/material_fonts';
  await _loadFont('Roboto', [
    '$materialFonts/Roboto-Regular.ttf',
    '$materialFonts/Roboto-Medium.ttf',
    '$materialFonts/Roboto-Bold.ttf',
  ]);
  await _loadFont('MaterialIcons', [
    '$materialFonts/MaterialIcons-Regular.otf',
  ]);
  await _loadFont('monospace', [
    '$materialFonts/Roboto-Regular.ttf',
  ]);
}

Future<void> _loadFont(String family, List<String> paths) async {
  final loader = FontLoader(family);
  for (final path in paths) {
    final file = File(path);
    if (file.existsSync()) {
      final bytes = await file.readAsBytes();
      loader.addFont(
          Future.value(ByteData.sublistView(Uint8List.fromList(bytes))));
    }
  }
  await loader.load();
}

class _EvidenceShell extends StatelessWidget {
  const _EvidenceShell({
    required this.selectedIndex,
    required this.enrolled,
    required this.body,
  });

  final int selectedIndex;
  final bool enrolled;
  final Widget body;

  @override
  Widget build(BuildContext context) {
    return Container(
      color: KpColors.canvas,
      child: Row(
        children: [
          Container(
            width: 324,
            decoration: const BoxDecoration(
              border: Border(right: BorderSide(color: KpColors.hairline)),
            ),
            child: Column(
              children: [
                Padding(
                  padding: const EdgeInsets.fromLTRB(18, 24, 18, 16),
                  child: Row(
                    children: [
                      Container(
                        width: 38,
                        height: 38,
                        decoration: BoxDecoration(
                          color: KpColors.primary,
                          borderRadius: BorderRadius.circular(8),
                        ),
                        child: const Icon(Icons.key_rounded,
                            color: KpColors.canvas, size: 24),
                      ),
                      const SizedBox(width: 12),
                      const Expanded(
                        child: Column(
                          crossAxisAlignment: CrossAxisAlignment.start,
                          children: [
                            Text(
                              'KeyLessPass',
                              style: TextStyle(
                                fontSize: 18,
                                fontWeight: FontWeight.w800,
                                color: KpColors.ink,
                              ),
                            ),
                            SizedBox(height: 2),
                            Text(
                              'Local desktop client',
                              style: TextStyle(
                                  fontSize: 12, color: KpColors.muted),
                            ),
                          ],
                        ),
                      ),
                    ],
                  ),
                ),
                Padding(
                  padding: const EdgeInsets.fromLTRB(18, 0, 18, 14),
                  child: Wrap(
                    spacing: 8,
                    runSpacing: 8,
                    children: [
                      StatusPill(
                        label: enrolled ? 'Ready' : 'Setup required',
                        tone: enrolled ? KpColors.success : KpColors.warning,
                      ),
                      const StatusPill(
                          label: 'Offline', tone: KpColors.primary),
                    ],
                  ),
                ),
                Expanded(
                  child: ListView(
                    padding: const EdgeInsets.symmetric(horizontal: 12),
                    children: [
                      _EvidenceNavItem(
                          icon: Icons.home_rounded,
                          label: 'Dashboard',
                          selected: selectedIndex == 0),
                      _EvidenceNavItem(
                          icon: Icons.verified_user_rounded,
                          label: 'Setup',
                          selected: selectedIndex == 1),
                      _EvidenceNavItem(
                          icon: Icons.view_list_rounded,
                          label: 'Records',
                          selected: selectedIndex == 2),
                      _EvidenceNavItem(
                          icon: Icons.usb_rounded,
                          label: 'USB Device',
                          selected: selectedIndex == 3),
                      _EvidenceNavItem(
                          icon: Icons.security_rounded,
                          label: 'Security',
                          selected: selectedIndex == 4),
                      _EvidenceNavItem(
                          icon: Icons.tune_rounded,
                          label: 'Settings',
                          selected: selectedIndex == 5),
                      _EvidenceNavItem(
                          icon: Icons.info_outline_rounded,
                          label: 'About',
                          selected: selectedIndex == 6),
                    ],
                  ),
                ),
                const Padding(
                  padding: EdgeInsets.all(16),
                  child: Row(
                    children: [
                      Icon(Icons.lock_outline_rounded,
                          color: KpColors.muted, size: 18),
                      SizedBox(width: 8),
                      Expanded(
                        child: Text(
                          'No service passwords stored',
                          overflow: TextOverflow.ellipsis,
                          style: TextStyle(color: KpColors.muted),
                        ),
                      ),
                    ],
                  ),
                ),
              ],
            ),
          ),
          Expanded(
              child: Padding(padding: const EdgeInsets.all(28), child: body)),
        ],
      ),
    );
  }
}

class _EvidenceNavItem extends StatelessWidget {
  const _EvidenceNavItem({
    required this.icon,
    required this.label,
    this.selected = false,
  });

  final IconData icon;
  final String label;
  final bool selected;

  @override
  Widget build(BuildContext context) {
    return Container(
      margin: const EdgeInsets.only(bottom: 6),
      padding: const EdgeInsets.symmetric(horizontal: 14, vertical: 12),
      decoration: BoxDecoration(
        color: selected ? KpColors.surfaceCard : Colors.transparent,
        borderRadius: BorderRadius.circular(8),
      ),
      child: Row(
        children: [
          Icon(icon,
              size: 20, color: selected ? KpColors.primary : KpColors.muted),
          const SizedBox(width: 12),
          Expanded(
            child: Text(
              label,
              overflow: TextOverflow.ellipsis,
              style: TextStyle(
                color: selected ? KpColors.ink : KpColors.muted,
                fontWeight: selected ? FontWeight.w700 : FontWeight.w500,
              ),
            ),
          ),
        ],
      ),
    );
  }
}

class _EnrollmentEvidence extends StatelessWidget {
  const _EnrollmentEvidence();

  @override
  Widget build(BuildContext context) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        const PageTitle(
          title: 'Enrollment',
          subtitle:
              'Create the local factor package and write the USB factor package. The mnemonic phrase is never stored.',
        ),
        const SizedBox(height: 22),
        const WorkflowStepper(steps: [
          'Mnemonic',
          'Platform factor',
          'USB factor',
          'Recovery metadata'
        ], current: 1),
        const SizedBox(height: 22),
        const Row(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Expanded(
              child: SectionPanel(
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Text('Mnemonic phrase',
                        style: TextStyle(
                            fontSize: 18, fontWeight: FontWeight.w700)),
                    SizedBox(height: 14),
                    _Field(
                        label: 'Mnemonic',
                        value: '******** ******** ******** ********'),
                    SizedBox(height: 12),
                    _Field(label: 'USB path', value: '/Volumes/WD'),
                    SizedBox(height: 14),
                    InlineNotice(
                      text:
                          'KeylessPass stores protected local state, CDR metadata, USB factor material, and recovery metadata only.',
                    ),
                  ],
                ),
              ),
            ),
            SizedBox(width: 20),
            SizedBox(
              width: 300,
              child: SectionPanel(
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Text('Security boundary',
                        style: TextStyle(
                            fontSize: 18, fontWeight: FontWeight.w700)),
                    SizedBox(height: 12),
                    InfoRow(label: 'Network', value: 'Disabled'),
                    InfoRow(label: 'Passwords', value: 'Not stored'),
                    InfoRow(label: 'Mnemonic', value: 'Not stored'),
                    InfoRow(label: 'USB', value: 'Ordinary removable storage'),
                  ],
                ),
              ),
            ),
          ],
        ),
        const Spacer(),
        Row(
          children: [
            FilledButton.icon(
                onPressed: () {},
                icon: const Icon(Icons.usb_rounded),
                label: const Text('Create factors', style: _buttonTextStyle)),
            const SizedBox(width: 12),
            OutlinedButton.icon(
                onPressed: () {},
                icon: const Icon(Icons.refresh_rounded),
                label: const Text('Rescan USB', style: _buttonTextStyle)),
          ],
        ),
      ],
    );
  }
}

class _CdrListEvidence extends StatelessWidget {
  const _CdrListEvidence();

  @override
  Widget build(BuildContext context) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        PageTitle(
          title: 'Credential Description Records',
          subtitle:
              'Display metadata is searchable, but password derivation uses stable recordSeq, recordID, version, and salt.',
          trailing: FilledButton.icon(
              onPressed: () {},
              icon: const Icon(Icons.add_rounded),
              label: const Text('Add record', style: _buttonTextStyle)),
        ),
        const SizedBox(height: 22),
        const Row(
          children: [
            Expanded(child: SignalTile(label: 'Active CDR', value: '3')),
            SizedBox(width: 14),
            Expanded(child: SignalTile(label: 'USB packages', value: '1')),
            SizedBox(width: 14),
            Expanded(child: SignalTile(label: 'Rotations', value: '0')),
          ],
        ),
        const SizedBox(height: 22),
        Expanded(
          child: Row(
            children: [
              Expanded(
                child: SectionPanel(
                  child: ListView(
                    children: const [
                      _RecordRow(
                          name: 'Operations Console',
                          service: 'ops.example.local',
                          account: 'operator'),
                      _RecordRow(
                          name: 'Vendor Portal',
                          service: 'vendor.example.local',
                          account: 'reviewer'),
                      _RecordRow(
                          name: 'Database Gateway',
                          service: 'db.example.local',
                          account: 'admin'),
                    ],
                  ),
                ),
              ),
              const SizedBox(width: 20),
              const SizedBox(
                width: 360,
                child: SectionPanel(
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      Text('Selected record',
                          style: TextStyle(
                              fontSize: 18, fontWeight: FontWeight.w700)),
                      SizedBox(height: 14),
                      InfoRow(label: 'recordSeq', value: '1'),
                      InfoRow(label: 'recordID', value: 'anonymous-uuid'),
                      InfoRow(label: 'version', value: '1'),
                      InfoRow(label: 'state', value: 'active'),
                      SizedBox(height: 14),
                      InlineNotice(
                          text:
                              'Editing displayName, serviceHint, or accountHint does not change the derived password.'),
                    ],
                  ),
                ),
              ),
            ],
          ),
        ),
      ],
    );
  }
}

class _RecordRow extends StatelessWidget {
  const _RecordRow(
      {required this.name, required this.service, required this.account});

  final String name;
  final String service;
  final String account;

  @override
  Widget build(BuildContext context) {
    return Container(
      margin: const EdgeInsets.only(bottom: 12),
      padding: const EdgeInsets.all(16),
      decoration: BoxDecoration(
        color: KpColors.surfaceSoft,
        border: Border.all(color: KpColors.hairline),
        borderRadius: BorderRadius.circular(8),
      ),
      child: Row(
        children: [
          const Icon(Icons.key_rounded, color: KpColors.primary),
          const SizedBox(width: 14),
          Expanded(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text(name,
                    style: const TextStyle(
                        color: KpColors.bodyStrong,
                        fontWeight: FontWeight.w700)),
                const SizedBox(height: 4),
                Text('$service  /  $account',
                    style: const TextStyle(color: KpColors.muted)),
              ],
            ),
          ),
          const StatusPill(label: 'active', tone: KpColors.success),
        ],
      ),
    );
  }
}

class _DeriveEvidence extends StatelessWidget {
  const _DeriveEvidence();

  @override
  Widget build(BuildContext context) {
    return Stack(
      children: [
        const _CdrListEvidence(),
        Center(
          child: SizedBox(
            width: 560,
            child: SectionPanel(
              child: Column(
                mainAxisSize: MainAxisSize.min,
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  const Text('Derive password',
                      style:
                          TextStyle(fontSize: 22, fontWeight: FontWeight.w800)),
                  const SizedBox(height: 10),
                  const Text(
                    'The derived service password is shown temporarily and is not written to storage or logs.',
                    style: TextStyle(color: KpColors.body),
                  ),
                  const SizedBox(height: 16),
                  const _Field(
                      label: 'Mnemonic', value: '******** ******** ********'),
                  const SizedBox(height: 12),
                  const _Field(label: 'USB path', value: '/Volumes/WD'),
                  const SizedBox(height: 14),
                  Container(
                    width: double.infinity,
                    padding: const EdgeInsets.all(14),
                    decoration: BoxDecoration(
                      color: KpColors.canvas,
                      border: Border.all(color: KpColors.hairlineStrong),
                      borderRadius: BorderRadius.circular(8),
                    ),
                    child: const Text(
                      'Derived password hidden in screenshot',
                      style: TextStyle(
                          color: KpColors.muted, fontFamily: 'monospace'),
                    ),
                  ),
                  const SizedBox(height: 18),
                  Row(
                    mainAxisAlignment: MainAxisAlignment.end,
                    children: [
                      OutlinedButton.icon(
                          onPressed: () {},
                          icon: const Icon(Icons.close_rounded),
                          label: const Text('Cancel', style: _buttonTextStyle)),
                      const SizedBox(width: 12),
                      FilledButton.icon(
                          onPressed: () {},
                          icon: const Icon(Icons.content_copy_rounded),
                          label: const Text('Derive and copy',
                              style: _buttonTextStyle)),
                    ],
                  ),
                ],
              ),
            ),
          ),
        ),
      ],
    );
  }
}

class _RotationEvidence extends StatelessWidget {
  const _RotationEvidence();

  @override
  Widget build(BuildContext context) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        const PageTitle(
          title: 'Current and previous versions',
          subtitle:
              'Creating a new current version keeps the previous version available for password derivation.',
        ),
        const SizedBox(height: 22),
        const WorkflowStepper(
            steps: ['Create new version', 'Derive current', 'Derive previous'],
            current: 2),
        const SizedBox(height: 22),
        Row(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            const Expanded(
              child: SectionPanel(
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Text('Version state',
                        style: TextStyle(
                            fontSize: 18, fontWeight: FontWeight.w700)),
                    SizedBox(height: 14),
                    InfoRow(label: 'current', value: 'v2 active'),
                    InfoRow(label: 'previous', value: 'v1 retained'),
                    InfoRow(label: 'salt', value: 'new random 128-bit salt'),
                    InfoRow(
                        label: 'descriptor',
                        value: 'immutable within each version'),
                    SizedBox(height: 14),
                    InlineNotice(
                        text:
                            'The previous version remains available for rollback checks.'),
                  ],
                ),
              ),
            ),
            const SizedBox(width: 20),
            Expanded(
              child: SectionPanel(
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    const Text('Version passwords',
                        style: TextStyle(
                            fontSize: 18, fontWeight: FontWeight.w700)),
                    const SizedBox(height: 14),
                    const _Field(
                        label: 'Mnemonic', value: '******** ******** ********'),
                    const SizedBox(height: 12),
                    const _Field(label: 'USB path', value: '/Volumes/WD'),
                    const SizedBox(height: 18),
                    Row(
                      children: [
                        Expanded(
                            child: OutlinedButton.icon(
                                onPressed: () {},
                                icon: const Icon(Icons.history_rounded),
                                label: const Text('Derive previous',
                                    style: _buttonTextStyle))),
                        const SizedBox(width: 12),
                        Expanded(
                            child: FilledButton.icon(
                                onPressed: () {},
                                icon: const Icon(Icons.password_rounded),
                                label: const Text('Derive current',
                                    style: _buttonTextStyle))),
                      ],
                    ),
                  ],
                ),
              ),
            ),
          ],
        ),
      ],
    );
  }
}

class _UsbRecoveryEvidence extends StatelessWidget {
  const _UsbRecoveryEvidence();

  @override
  Widget build(BuildContext context) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        const PageTitle(
          title: 'USB Device',
          subtitle:
              'Manage the ordinary removable USB factor package and recovery workflows from one place.',
        ),
        const SizedBox(height: 22),
        const Row(
          children: [
            Expanded(
                child: SignalTile(label: 'Detected USB volumes', value: '1')),
            SizedBox(width: 14),
            Expanded(
                child: SignalTile(label: 'Package status', value: 'Readable')),
            SizedBox(width: 14),
            Expanded(child: SignalTile(label: 'Local mode', value: 'Enabled')),
          ],
        ),
        const SizedBox(height: 22),
        Expanded(
          child: Row(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Expanded(
                child: SectionPanel(
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      const Text('USB actions',
                          style: TextStyle(
                              fontSize: 18, fontWeight: FontWeight.w700)),
                      const SizedBox(height: 14),
                      const _Field(label: 'USB path', value: '/Volumes/WD'),
                      const SizedBox(height: 12),
                      const _Field(
                          label: 'Mnemonic',
                          value: '******** ******** ********'),
                      const SizedBox(height: 16),
                      Row(
                        children: [
                          Expanded(
                              child: FilledButton.icon(
                                  onPressed: () {},
                                  icon: const Icon(Icons.verified_rounded),
                                  label: const Text('Verify package',
                                      style: _buttonTextStyle))),
                          const SizedBox(width: 12),
                          Expanded(
                              child: OutlinedButton.icon(
                                  onPressed: () {},
                                  icon: const Icon(Icons.usb_rounded),
                                  label: const Text('Rebuild USB',
                                      style: _buttonTextStyle))),
                        ],
                      ),
                    ],
                  ),
                ),
              ),
              const SizedBox(width: 20),
              const Expanded(
                child: SectionPanel(
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      Text('Recovery tools',
                          style: TextStyle(
                              fontSize: 18, fontWeight: FontWeight.w700)),
                      SizedBox(height: 14),
                      InlineNotice(
                          text:
                              'A single factor is not enough to recover. Choose a recovery mode when two factors are available.'),
                      SizedBox(height: 14),
                      InfoRow(
                          label: 'USB lost',
                          value: 'mnemonic + local material'),
                      InfoRow(
                          label: 'New device',
                          value: 'mnemonic + USB material'),
                      InfoRow(
                          label: 'Mnemonic forgotten',
                          value: 'enterprise recovery process'),
                    ],
                  ),
                ),
              ),
            ],
          ),
        ),
      ],
    );
  }
}

const _buttonTextStyle = TextStyle(fontFamily: 'Roboto');

class _Field extends StatelessWidget {
  const _Field({required this.label, required this.value});

  final String label;
  final String value;

  @override
  Widget build(BuildContext context) {
    return Container(
      width: double.infinity,
      padding: const EdgeInsets.symmetric(horizontal: 14, vertical: 12),
      decoration: BoxDecoration(
        color: KpColors.surfaceSoft,
        border: Border.all(color: KpColors.hairlineStrong),
        borderRadius: BorderRadius.circular(8),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Text(label,
              style: const TextStyle(
                  color: KpColors.muted,
                  fontSize: 12,
                  fontWeight: FontWeight.w700)),
          const SizedBox(height: 6),
          Text(value,
              style: const TextStyle(color: KpColors.bodyStrong, fontSize: 14)),
        ],
      ),
    );
  }
}
