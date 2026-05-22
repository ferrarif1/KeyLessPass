import 'package:flutter/material.dart';

import '../app/app_theme.dart';

class SectionPanel extends StatelessWidget {
  const SectionPanel({required this.child, this.accent = false, super.key});

  final Widget child;
  final bool accent;

  @override
  Widget build(BuildContext context) {
    return Container(
      padding: const EdgeInsets.all(24),
      decoration: BoxDecoration(
        color: accent ? KpColors.primary : KpColors.surfaceCard,
        border: Border.all(color: accent ? KpColors.primary : KpColors.hairlineStrong),
        borderRadius: BorderRadius.circular(8),
      ),
      child: child,
    );
  }
}

class WorkflowStepper extends StatelessWidget {
  const WorkflowStepper({required this.steps, required this.current, super.key});

  final List<String> steps;
  final int current;

  @override
  Widget build(BuildContext context) {
    return Wrap(
      spacing: 8,
      runSpacing: 8,
      children: [
        for (var index = 0; index < steps.length; index++)
          Chip(
            avatar: CircleAvatar(
              backgroundColor: index <= current ? KpColors.canvas : KpColors.surfaceElevated,
              child: Text(
                '${index + 1}',
                style: TextStyle(color: index <= current ? KpColors.primary : KpColors.muted, fontSize: 12),
              ),
            ),
            label: Text(steps[index]),
            labelStyle: TextStyle(color: index <= current ? KpColors.canvas : KpColors.body),
            backgroundColor: index <= current ? KpColors.primary : KpColors.surfaceCard,
            side: BorderSide(color: index <= current ? KpColors.primary : KpColors.hairlineStrong),
          ),
      ],
    );
  }
}

class InfoRow extends StatelessWidget {
  const InfoRow({required this.label, required this.value, super.key});

  final String label;
  final String value;

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 6),
      child: Row(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          SizedBox(
            width: 150,
            child: Text(label.toUpperCase(), style: Theme.of(context).textTheme.labelMedium),
          ),
          Expanded(
            child: SelectableText(
              value.isEmpty ? '-' : value,
              style: const TextStyle(fontFamily: 'monospace', color: KpColors.bodyStrong, fontSize: 13),
            ),
          ),
        ],
      ),
    );
  }
}

class DangerPanel extends StatelessWidget {
  const DangerPanel({
    required this.title,
    required this.body,
    required this.actionLabel,
    required this.onPressed,
    super.key,
  });

  final String title;
  final String body;
  final String actionLabel;
  final VoidCallback onPressed;

  @override
  Widget build(BuildContext context) {
    return Container(
      padding: const EdgeInsets.all(14),
      decoration: BoxDecoration(
        color: KpColors.surfaceCard,
        border: Border.all(color: KpColors.error),
        borderRadius: BorderRadius.circular(8),
      ),
      child: Row(
        children: [
          const Icon(Icons.warning_amber_rounded, color: KpColors.error),
          const SizedBox(width: 12),
          Expanded(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text(title, style: Theme.of(context).textTheme.titleSmall),
                const SizedBox(height: 4),
                Text(body),
              ],
            ),
          ),
          OutlinedButton.icon(
            onPressed: onPressed,
            icon: const Icon(Icons.check_rounded),
            label: Text(actionLabel),
          ),
        ],
      ),
    );
  }
}

class PageTitle extends StatelessWidget {
  const PageTitle({
    required this.title,
    required this.subtitle,
    this.trailing,
    super.key,
  });

  final String title;
  final String subtitle;
  final Widget? trailing;

  @override
  Widget build(BuildContext context) {
    return Row(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Expanded(
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Text(title, style: Theme.of(context).textTheme.headlineSmall),
              const SizedBox(height: 8),
              Text(subtitle, style: Theme.of(context).textTheme.bodyMedium),
            ],
          ),
        ),
        if (trailing != null) ...[
          const SizedBox(width: 16),
          trailing!,
        ],
      ],
    );
  }
}

class SignalTile extends StatelessWidget {
  const SignalTile({required this.label, required this.value, this.tone = KpColors.primary, super.key});

  final String label;
  final String value;
  final Color tone;

  @override
  Widget build(BuildContext context) {
    return Container(
      constraints: const BoxConstraints(minHeight: 96),
      padding: const EdgeInsets.all(16),
      decoration: BoxDecoration(
        color: KpColors.surfaceSoft,
        border: Border.all(color: KpColors.hairline),
        borderRadius: BorderRadius.circular(8),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Text(label.toUpperCase(), style: Theme.of(context).textTheme.labelMedium),
          const SizedBox(height: 8),
          FittedBox(
            fit: BoxFit.scaleDown,
            alignment: Alignment.centerLeft,
            child: Text(
              value,
              maxLines: 1,
              style: TextStyle(color: tone, fontSize: 26, fontWeight: FontWeight.w700, height: 1),
            ),
          ),
        ],
      ),
    );
  }
}

class StatusPill extends StatelessWidget {
  const StatusPill({
    required this.label,
    this.tone = KpColors.primary,
    super.key,
  });

  final String label;
  final Color tone;

  @override
  Widget build(BuildContext context) {
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 6),
      decoration: BoxDecoration(
        color: KpColors.surfaceCard,
        border: Border.all(color: KpColors.hairlineStrong),
        borderRadius: BorderRadius.circular(999),
      ),
      child: Row(
        mainAxisSize: MainAxisSize.min,
        children: [
          Container(
            width: 7,
            height: 7,
            decoration: BoxDecoration(color: tone, shape: BoxShape.circle),
          ),
          const SizedBox(width: 8),
          Text(label, style: Theme.of(context).textTheme.labelMedium?.copyWith(color: KpColors.bodyStrong)),
        ],
      ),
    );
  }
}

class InlineNotice extends StatelessWidget {
  const InlineNotice({
    required this.text,
    this.icon = Icons.info_outline_rounded,
    this.tone = KpColors.primary,
    super.key,
  });

  final String text;
  final IconData icon;
  final Color tone;

  @override
  Widget build(BuildContext context) {
    return Container(
      width: double.infinity,
      padding: const EdgeInsets.all(14),
      decoration: BoxDecoration(
        color: KpColors.surfaceSoft,
        border: Border.all(color: KpColors.hairline),
        borderRadius: BorderRadius.circular(8),
      ),
      child: Row(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Icon(icon, color: tone, size: 18),
          const SizedBox(width: 10),
          Expanded(child: Text(text)),
        ],
      ),
    );
  }
}
