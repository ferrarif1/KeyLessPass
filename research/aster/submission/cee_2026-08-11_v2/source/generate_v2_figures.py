#!/usr/bin/env python3
"""Generate the ASTER v2 graphical abstract and manuscript figures from fixed evidence."""

from __future__ import annotations

import html
import json
import math
import subprocess
from pathlib import Path

from PIL import Image


ROOT = Path(__file__).resolve().parents[1]
OUT = ROOT / "figures"
DATA = ROOT / "data"

FONT = "Arial, Helvetica, Liberation Sans, sans-serif"
DARK = "#1F2937"
MUTED = "#667085"
NAVY = "#173A5E"
BLUE = "#2F6B9A"
TEAL = "#2A9D8F"
GREEN = "#4C956C"
AMBER = "#D89B2B"
ORANGE = "#D97745"
RED = "#C94C4C"
PURPLE = "#7657A5"
LIGHT = "#F6F8FB"
GRID = "#DDE4EC"
BORDER = "#B8C4D1"


def esc(value: object) -> str:
    return html.escape(str(value), quote=True)


class SVG:
    def __init__(self, width: int, height: int, title: str):
        self.width = width
        self.height = height
        self.items = [
            f'<svg xmlns="http://www.w3.org/2000/svg" width="{width}" height="{height}" viewBox="0 0 {width} {height}">',
            f"<title>{esc(title)}</title>",
            "<defs>",
            '<marker id="arrow" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="8" markerHeight="8" orient="auto-start-reverse"><path d="M 0 0 L 10 5 L 0 10 z" fill="#41566B"/></marker>',
            '<marker id="arrowGreen" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="8" markerHeight="8" orient="auto-start-reverse"><path d="M 0 0 L 10 5 L 0 10 z" fill="#4C956C"/></marker>',
            '<marker id="arrowRed" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="8" markerHeight="8" orient="auto-start-reverse"><path d="M 0 0 L 10 5 L 0 10 z" fill="#C94C4C"/></marker>',
            "</defs>",
            f'<rect width="{width}" height="{height}" fill="#FFFFFF"/>',
        ]

    def add(self, item: str) -> None:
        self.items.append(item)

    def rect(self, x, y, w, h, fill="#FFFFFF", stroke=BORDER, sw=2, rx=18, dash=None) -> None:
        extra = f' stroke-dasharray="{dash}"' if dash else ""
        self.add(f'<rect x="{x}" y="{y}" width="{w}" height="{h}" rx="{rx}" fill="{fill}" stroke="{stroke}" stroke-width="{sw}"{extra}/>' )

    def line(self, x1, y1, x2, y2, stroke="#41566B", sw=3, arrow=False, dash=None) -> None:
        extra = ' marker-end="url(#arrow)"' if arrow else ""
        if stroke == GREEN and arrow:
            extra = ' marker-end="url(#arrowGreen)"'
        elif stroke == RED and arrow:
            extra = ' marker-end="url(#arrowRed)"'
        if dash:
            extra += f' stroke-dasharray="{dash}"'
        self.add(f'<line x1="{x1}" y1="{y1}" x2="{x2}" y2="{y2}" stroke="{stroke}" stroke-width="{sw}" stroke-linecap="round"{extra}/>' )

    def path(self, d, stroke="#41566B", sw=3, fill="none", arrow=False, dash=None) -> None:
        extra = ' marker-end="url(#arrow)"' if arrow else ""
        if stroke == GREEN and arrow:
            extra = ' marker-end="url(#arrowGreen)"'
        elif stroke == RED and arrow:
            extra = ' marker-end="url(#arrowRed)"'
        if dash:
            extra += f' stroke-dasharray="{dash}"'
        self.add(f'<path d="{d}" stroke="{stroke}" stroke-width="{sw}" fill="{fill}" stroke-linejoin="round" stroke-linecap="round"{extra}/>' )

    def circle(self, cx, cy, r, fill="#FFFFFF", stroke=BORDER, sw=2) -> None:
        self.add(f'<circle cx="{cx}" cy="{cy}" r="{r}" fill="{fill}" stroke="{stroke}" stroke-width="{sw}"/>')

    def text(self, x, y, lines, size=30, fill=DARK, weight=400, anchor="middle", line=1.2, italic=False) -> None:
        if isinstance(lines, str):
            lines = [lines]
        style = "italic" if italic else "normal"
        self.add(f'<text x="{x}" y="{y}" text-anchor="{anchor}" font-family="{FONT}" font-size="{size}" font-weight="{weight}" font-style="{style}" fill="{fill}">')
        for i, value in enumerate(lines):
            dy = 0 if i == 0 else size * line
            self.add(f'<tspan x="{x}" dy="{dy}">{esc(value)}</tspan>')
        self.add("</text>")

    def badge(self, cx, cy, label, fill=NAVY) -> None:
        self.circle(cx, cy, 24, fill=fill, stroke=fill, sw=1)
        self.text(cx, cy + 9, label, size=24, fill="#FFFFFF", weight=700)

    def save(self, path: Path) -> None:
        path.write_text("\n".join(self.items + ["</svg>"]) + "\n", encoding="utf-8")


def panel_label(svg: SVG, x: int, y: int, label: str, title: str) -> None:
    svg.rect(x, y - 32, 44, 44, fill=NAVY, stroke=NAVY, sw=1, rx=8)
    svg.text(x + 22, y, label, size=25, fill="#FFFFFF", weight=700)
    svg.text(x + 62, y, title, size=30, weight=700, anchor="start")


def node(svg: SVG, x, y, w, h, title, subtitle=None, fill="#F7FAFD", stroke=BLUE, icon=None) -> None:
    svg.rect(x, y, w, h, fill=fill, stroke=stroke, sw=3, rx=22)
    if icon:
        icon_x, icon_y = x + 55, y + h / 2
        if icon == "policy":
            svg.rect(icon_x - 22, icon_y - 28, 44, 56, fill="#FFFFFF", stroke=stroke, sw=2, rx=5)
            svg.line(icon_x - 12, icon_y - 10, icon_x + 13, icon_y - 10, stroke=stroke, sw=2)
            svg.line(icon_x - 12, icon_y + 1, icon_x + 13, icon_y + 1, stroke=stroke, sw=2)
            svg.line(icon_x - 12, icon_y + 12, icon_x + 5, icon_y + 12, stroke=stroke, sw=2)
        elif icon == "ticket":
            svg.path(f"M {icon_x-28} {icon_y-20} H {icon_x+28} V {icon_y+20} H {icon_x-28} Z", fill="#FFFFFF", stroke=stroke, sw=2)
            svg.circle(icon_x - 28, icon_y, 7, fill=fill, stroke=stroke, sw=2)
            svg.circle(icon_x + 28, icon_y, 7, fill=fill, stroke=stroke, sw=2)
            svg.line(icon_x - 6, icon_y - 10, icon_x - 6, icon_y + 10, stroke=stroke, sw=2, dash="4 4")
        elif icon == "threshold":
            for dx, dy in [(-20, 12), (20, 12), (0, -22)]:
                svg.circle(icon_x + dx, icon_y + dy, 14, fill="#FFFFFF", stroke=stroke, sw=2)
            svg.line(icon_x, icon_y - 8, icon_x - 13, icon_y + 2, stroke=stroke, sw=2)
            svg.line(icon_x, icon_y - 8, icon_x + 13, icon_y + 2, stroke=stroke, sw=2)
        elif icon == "password":
            svg.rect(icon_x - 30, icon_y - 18, 60, 42, fill="#FFFFFF", stroke=stroke, sw=2, rx=6)
            for dx in (-17, 0, 17):
                svg.circle(icon_x + dx, icon_y + 3, 4, fill=stroke, stroke=stroke, sw=1)
        tx = x + 100
        anchor = "start"
    else:
        tx = x + w / 2
        anchor = "middle"
    if subtitle:
        svg.text(tx, y + h / 2 - 6, title, size=29, weight=700, anchor=anchor)
        svg.text(tx, y + h / 2 + 32, subtitle, size=23, fill=MUTED, anchor=anchor)
    else:
        svg.text(tx, y + h / 2 + 10, title, size=29, weight=700, anchor=anchor)


def graphical_abstract() -> SVG:
    s = SVG(2656, 1062, "ASTER graphical abstract")
    s.text(1328, 82, "ASTER: exact-scope credentials with Root-Epoch healing", size=48, fill=NAVY, weight=700)
    s.line(150, 120, 2506, 120, stroke=GRID, sw=2)

    lane_x, lane_w = 72, 178
    s.rect(lane_x, 188, lane_w, 286, fill="#EAF2F8", stroke=BLUE, sw=2, rx=22)
    s.text(lane_x + lane_w / 2, 300, ["NORMAL", "DERIVATION"], size=27, fill=NAVY, weight=700, line=1.25)
    s.text(lane_x + lane_w / 2, 395, "one output", size=22, fill=MUTED, weight=600)
    s.rect(lane_x, 594, lane_w, 286, fill="#FBECEC", stroke=RED, sw=2, rx=22)
    s.text(lane_x + lane_w / 2, 680, ["POST", "COMPROMISE", "HEALING"], size=22, fill="#8B2F2F", weight=700, line=1.15)
    s.text(lane_x + lane_w / 2, 820, "replace root", size=22, fill=MUTED, weight=600)

    xs = [300, 870, 1440, 2010]
    widths = [430, 430, 430, 500]
    top_titles = [
        ("Exact policy space", "Count + Rank/Unrank", "policy", "#EEF5FB", BLUE),
        ("Exact-scope capability", "context + generation", "ticket", "#FFF6DE", AMBER),
        ("Threshold evaluation", "Root-Epoch stays shared", "threshold", "#EAF6F2", TEAL),
        ("One compliant password", "no reusable endpoint key", "password", "#ECF6EE", GREEN),
    ]
    for i, (title, subtitle, icon, fill, stroke) in enumerate(top_titles):
        node(s, xs[i], 230, widths[i], 190, title, subtitle, fill, stroke, icon)
        if i < 3:
            s.line(xs[i] + widths[i] + 20, 325, xs[i + 1] - 20, 325, arrow=True)
    s.text(1328, 505, "ONE AUTHORIZED OUTPUT - NO DERIVATION AMPLIFICATION", size=26, fill=NAVY, weight=700)

    bottom = [
        ("Old root disclosed", "exposure remains", "#FBECEC", RED),
        ("Independent new Root-Epoch", "replace, not refresh", "#F2ECF8", PURPLE),
        ("Evidence-bounded migration", "retain both if ambiguous", "#FFF6DE", AMBER),
        ("Progressive healing", "old-root exposure falls to 0", "#ECF6EE", GREEN),
    ]
    for i, (title, subtitle, fill, stroke) in enumerate(bottom):
        node(s, xs[i], 635, widths[i], 190, title, subtitle, fill, stroke)
        if i < 3:
            s.line(xs[i] + widths[i] + 20, 730, xs[i + 1] - 20, 730, arrow=True)
    s.text(1328, 936, "INDEPENDENT ROOT REPLACEMENT + CONCLUSIVE REMOTE EVIDENCE", size=26, fill="#7A3F16", weight=700)
    return s


def figure1() -> SVG:
    s = SVG(2200, 1450, "ASTER architecture and Root-Epoch healing")
    s.text(1100, 70, "ASTER architecture: exact-scope derivation and Root-Epoch healing", size=43, fill=NAVY, weight=700)
    panel_label(s, 75, 150, "A", "Normal operation")
    s.rect(70, 175, 2060, 675, fill="#FBFCFE", stroke=GRID, sw=2, rx=24)

    node(s, 150, 420, 330, 150, "Client endpoint", "no reusable root key", "#EEF5FB", BLUE)
    node(s, 670, 230, 360, 150, "Approval Authority", "single-use capability", "#FFF6DE", AMBER)
    s.rect(1180, 245, 430, 300, fill="#EAF6F2", stroke=TEAL, sw=3, rx=24)
    s.text(1395, 300, "Evaluator domains", size=31, weight=700)
    for i, cx in enumerate([1270, 1395, 1520], 1):
        s.circle(cx, 385, 48, fill="#FFFFFF", stroke=TEAL, sw=3)
        s.text(cx, 395, f"E{i}", size=26, fill=TEAL, weight=700)
    s.text(1395, 490, "threshold exact-domain evaluation", size=22, fill=MUTED, weight=600)
    node(s, 1770, 420, 300, 150, "Legacy target", "submit + verify", "#F5F1FA", PURPLE)
    node(s, 720, 655, 330, 125, "Freshness service", "monotonic checkpoint", "#F4F6F8", "#6B7280")

    s.path("M 480 430 C 540 360, 590 315, 670 285", arrow=True)
    s.badge(548, 360, "1")
    s.path("M 690 350 C 620 410, 550 455, 480 475", arrow=True)
    s.badge(590, 420, "2")
    s.path("M 480 500 C 700 455, 900 420, 1160 390", arrow=True)
    s.badge(760, 445, "3")
    s.badge(1395, 610, "4", fill=TEAL)
    s.path("M 1180 470 C 980 585, 720 615, 480 535", arrow=True)
    s.badge(820, 610, "5", fill=GREEN)
    s.path("M 480 555 C 850 675, 1420 675, 1750 535", arrow=True)
    s.badge(1660, 610, "6")
    s.path("M 320 570 C 430 720, 580 720, 700 710", arrow=True, dash="10 8")

    panel_label(s, 75, 930, "B", "Post-compromise healing")
    s.rect(70, 955, 2060, 405, fill="#FBFCFE", stroke=GRID, sw=2, rx=24)
    node(s, 125, 1080, 270, 125, "Old root disclosed", "K(old) exposed", "#FBECEC", RED)
    node(s, 500, 1080, 325, 125, "Independent epoch", "replace, not refresh", "#F2ECF8", PURPLE)
    node(s, 930, 1080, 325, 125, "Per-credential migration", "retain descriptors", "#FFF6DE", AMBER)
    s.path("M 395 1142 H 480", arrow=True)
    s.path("M 825 1142 H 910", arrow=True)
    s.path("M 1255 1142 H 1350", arrow=True)
    s.path("M 1350 1142 L 1470 1040", stroke=GREEN, arrow=True)
    s.path("M 1350 1142 L 1470 1245", stroke=RED, arrow=True)
    s.circle(1350, 1142, 52, fill="#FFFFFF", stroke=AMBER, sw=3)
    s.text(1350, 1135, ["remote", "evidence"], size=20, weight=700, line=1.15)
    node(s, 1490, 980, 290, 120, "NewOnly", "commit new epoch", "#ECF6EE", GREEN)
    node(s, 1490, 1190, 290, 120, "Ambiguous", "preserve both paths", "#FBECEC", RED)
    node(s, 1850, 980, 230, 120, "Retire old", "only if unreferenced", "#F4F6F8", "#6B7280")
    s.line(1780, 1040, 1830, 1040, stroke=GREEN, arrow=True)
    s.text(1100, 1410, "Healing is credential-by-credential and evidence-bounded; share refresh alone does not heal a disclosed root.", size=24, fill=NAVY, weight=600)
    return s


def chart_axes(s: SVG, x0, y0, w, h, title, panel, y_title) -> None:
    panel_label(s, x0, y0 - 54, panel, title)
    for value in range(0, 33, 8):
        y = y0 + h - h * value / 32
        s.line(x0, y, x0 + w, y, stroke=GRID, sw=2)
        s.text(x0 - 22, y + 8, str(value), size=22, fill=MUTED, anchor="end")
    s.line(x0, y0, x0, y0 + h, stroke=DARK, sw=2)
    s.line(x0, y0 + h, x0 + w, y0 + h, stroke=DARK, sw=2)
    s.add(f'<text x="{x0-80}" y="{y0+h/2}" transform="rotate(-90 {x0-80} {y0+h/2})" text-anchor="middle" font-family="{FONT}" font-size="24" font-weight="600" fill="{DARK}">{esc(y_title)}</text>')


def marker(s: SVG, x, y, color, shape) -> None:
    if shape == "circle":
        s.circle(x, y, 8, fill="#FFFFFF", stroke=color, sw=4)
    elif shape == "square":
        s.rect(x - 8, y - 8, 16, 16, fill="#FFFFFF", stroke=color, sw=4, rx=2)
    else:
        s.path(f"M {x} {y-10} L {x+10} {y+8} L {x-10} {y+8} Z", fill="#FFFFFF", stroke=color, sw=4)


def plot_series(s: SVG, xs, ys, x0, y0, w, h, color, shape, dash=None) -> None:
    pts = []
    for i, value in enumerate(ys):
        x = x0 + w * i / (len(xs) - 1)
        y = y0 + h - h * value / 32
        pts.append((x, y))
    d = "M " + " L ".join(f"{x:.1f} {y:.1f}" for x, y in pts)
    s.path(d, stroke=color, sw=4, dash=dash)
    for x, y in pts:
        marker(s, x, y, color, shape)


def figure2() -> SVG:
    raw = json.loads((DATA / "rq2_summary.json").read_text())
    qs = raw["qValues"]
    rows = raw["rows"]
    by_mode = {}
    for mode in ("exact", "projected_service_account", "wildcard"):
        selected = sorted((r for r in rows if r["mode"] == mode), key=lambda r: r["q"])
        assert [r["q"] for r in selected] == qs
        by_mode[mode] = selected
    s = SVG(2100, 980, "Authorization amplification by capability scope")
    s.text(1050, 70, "Authorization amplification by capability scope", size=43, fill=NAVY, weight=700)
    left = (170, 230, 760, 560)
    right = (1190, 230, 760, 560)
    chart_axes(s, *left, "Accepted outputs versus capability budget q", "A", "Accepted outputs")
    chart_axes(s, *right, "Unauthorized spill versus capability budget q", "B", "Unauthorized spill")
    styles = [
        ("exact", "Exact", BLUE, "circle", None),
        ("projected_service_account", "Projected service/account", AMBER, "square", "12 8"),
        ("wildcard", "Wildcard", RED, "triangle", "4 7"),
    ]
    for mode, _, color, shape, dash in styles:
        plot_series(s, qs, [r["acceptedSet"] for r in by_mode[mode]], *left, color, shape, dash)
        plot_series(s, qs, [r["unauthorizedSpill"] for r in by_mode[mode]], *right, color, shape, dash)
    for x0, y0, w, h in (left, right):
        for i, q in enumerate(qs):
            x = x0 + w * i / (len(qs) - 1)
            s.text(x, y0 + h + 38, str(q), size=22, fill=MUTED)
        s.text(x0 + w / 2, y0 + h + 82, "Capability budget q", size=25, weight=600)
    lx = 620
    for i, (_, label, color, shape, dash) in enumerate(styles):
        x = lx + i * 390
        s.line(x, 885, x + 70, 885, stroke=color, sw=4, dash=dash)
        marker(s, x + 35, 885, color, shape)
        s.text(x + 88, 894, label, size=23, anchor="start", weight=600)
    return s


def figure3() -> SVG:
    raw = json.loads((DATA / "rq4_summary.json").read_text())
    curve = raw["exposureCurve"]
    xs = [r["conclusivelyMigrated"] for r in curve]
    old = [r["stillDerivableByOldRoot"] for r in curve]
    healed = [r["healedAgainstOldRootOnly"] for r in curve]
    assert raw["shareRefreshPreservedOutputs"] is True
    refresh = [raw["records"]] * len(xs)
    s = SVG(1800, 1080, "Progressive healing after independent Root-Epoch replacement")
    s.text(900, 70, "Progressive healing after independent Root-Epoch replacement", size=42, fill=NAVY, weight=700)
    x0, y0, w, h = 190, 190, 1390, 670
    for value in range(0, 101, 25):
        y = y0 + h - h * value / 100
        s.line(x0, y, x0 + w, y, stroke=GRID, sw=2)
        s.text(x0 - 25, y + 8, str(value), size=23, fill=MUTED, anchor="end")
    s.line(x0, y0, x0, y0 + h, stroke=DARK, sw=2)
    s.line(x0, y0 + h, x0 + w, y0 + h, stroke=DARK, sw=2)
    s.add(f'<text x="85" y="{y0+h/2}" transform="rotate(-90 85 {y0+h/2})" text-anchor="middle" font-family="{FONT}" font-size="26" font-weight="600" fill="{DARK}">Credentials</text>')
    s.text(x0 + w / 2, 940, "Conclusive migrations", size=27, weight=600)

    def draw(values, color, shape, dash=None):
        pts = []
        for xval, value in zip(xs, values):
            x = x0 + w * xval / 100
            y = y0 + h - h * value / 100
            pts.append((x, y))
        s.path("M " + " L ".join(f"{x:.1f} {y:.1f}" for x, y in pts), stroke=color, sw=5, dash=dash)
        for x, y in pts:
            marker(s, x, y, color, shape)

    draw(old, RED, "circle")
    draw(healed, GREEN, "square")
    draw(refresh, MUTED, "triangle", "14 9")
    for xval in xs:
        x = x0 + w * xval / 100
        s.text(x, y0 + h + 40, str(xval), size=23, fill=MUTED)
    legends = [
        ("Old-root exposed", RED, "circle", None),
        ("Healed", GREEN, "square", None),
        ("Share-refresh control", MUTED, "triangle", "14 9"),
    ]
    for i, (label, color, shape, dash) in enumerate(legends):
        x = 250 + i * 500
        s.line(x, 1010, x + 70, 1010, stroke=color, sw=5, dash=dash)
        marker(s, x + 35, 1010, color, shape)
        s.text(x + 90, 1019, label, size=22, anchor="start", weight=600)
    s.text(900, 145, "Share refresh preserves the disclosed root; independent replacement enables credential-by-credential healing.", size=24, fill=MUTED, weight=600)
    return s


def figure4() -> SVG:
    s = SVG(1900, 1230, "Failure-safe migration state machine")
    s.text(950, 72, "Failure-safe migration state machine", size=44, fill=NAVY, weight=700)
    node(s, 90, 235, 300, 135, "Committed(e,g)", "current descriptor", "#EEF5FB", BLUE)
    node(s, 500, 235, 300, 135, "Prepared", "candidate persisted", "#FFF6DE", AMBER)
    node(s, 910, 235, 360, 135, "Submitted / Verifying", "await remote evidence", "#F2ECF8", PURPLE)
    node(s, 1460, 125, 350, 135, "Committed(e+1,j)", "candidate authoritative", "#ECF6EE", GREEN)
    node(s, 1460, 350, 350, 135, "Committed(e,g)", "candidate aborted", "#EEF5FB", BLUE)
    node(s, 890, 665, 400, 155, "UnknownOutcome", "preserve both descriptors", "#FBECEC", RED)
    node(s, 380, 680, 300, 125, "Reconcile", "adapter-authorized", "#F4F6F8", "#6B7280")

    s.line(390, 302, 480, 302, arrow=True)
    s.text(435, 180, ["persist candidate", "descriptor"], size=21, fill=MUTED, weight=600, line=1.1)
    s.line(800, 302, 890, 302, arrow=True)
    s.text(845, 205, "submit candidate", size=21, fill=MUTED, weight=600)
    s.path("M 1270 270 C 1360 245, 1380 205, 1440 195", stroke=GREEN, arrow=True)
    s.text(1370, 170, "NewOnly", size=23, fill=GREEN, weight=700)
    s.path("M 1270 335 C 1360 360, 1380 415, 1440 418", arrow=True)
    s.text(1370, 390, "OldOnly", size=23, fill=BLUE, weight=700)
    s.path("M 1090 370 V 640", stroke=RED, arrow=True)
    s.text(1115, 510, ["Both / Neither /", "Contradictory / Unknown"], size=22, fill=RED, weight=700, anchor="start", line=1.15)
    s.line(890, 742, 700, 742, arrow=True)
    s.text(795, 710, "later reconciliation", size=21, fill=MUTED, weight=600)
    s.path("M 530 680 C 520 545, 700 480, 910 355", arrow=True, dash="12 8")
    s.text(600, 540, ["re-enter evidence", "classification"], size=22, fill=MUTED, weight=600, line=1.15)

    s.rect(160, 940, 1580, 190, fill="#FFF8F1", stroke=ORANGE, sw=3, rx=22)
    s.circle(245, 1035, 42, fill="#FFFFFF", stroke=ORANGE, sw=3)
    s.text(245, 1048, "!", size=38, fill=ORANGE, weight=700)
    s.text(325, 1005, "UnknownOutcome is a safety state, not an error state.", size=29, fill="#8A451F", weight=700, anchor="start")
    s.text(325, 1055, "It retains both reconstruction paths and may only leave through adapter-authorized reconciliation.", size=25, fill=DARK, anchor="start")
    s.text(325, 1095, "It is never cleared solely because a timeout expires or a transport retry succeeds.", size=25, fill=DARK, anchor="start")
    return s


def export(svg: SVG, stem: str, tiff: bool = False) -> None:
    OUT.mkdir(parents=True, exist_ok=True)
    svg_path = OUT / f"{stem}.svg"
    png_path = OUT / f"{stem}.png"
    pdf_path = OUT / f"{stem}.pdf"
    svg.save(svg_path)
    subprocess.run(["rsvg-convert", "--width", str(svg.width), "--height", str(svg.height), "--output", str(png_path), str(svg_path)], check=True)
    subprocess.run(["rsvg-convert", "--format", "pdf", "--output", str(pdf_path), str(svg_path)], check=True)
    image = Image.open(png_path).convert("RGB")
    image.save(png_path, dpi=(300, 300), optimize=True)
    if tiff:
        image.save(OUT / f"{stem}.tif", compression="tiff_lzw", dpi=(300, 300))


def main() -> int:
    export(graphical_abstract(), "Graphical_Abstract_v2", tiff=True)
    export(figure1(), "Figure_1_v2")
    export(figure2(), "Figure_2_v2")
    export(figure3(), "Figure_3_v2")
    export(figure4(), "Figure_4_v2")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
