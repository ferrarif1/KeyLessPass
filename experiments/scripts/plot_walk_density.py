#!/usr/bin/env python3
"""Plot full-corpus FF1 cycle-walk observations from the recorded JSON."""

import json
from pathlib import Path

import matplotlib.pyplot as plt
import numpy as np


ROOT = Path(__file__).resolve().parents[2]
INPUT = ROOT / "experiments/performance/walk_corpus.json"
PDF = ROOT / "artifact/generated/walk_density.pdf"
PNG = ROOT / "artifact/generated/walk_density.png"


def main():
    PDF.parent.mkdir(parents=True, exist_ok=True)
    payload = json.loads(INPUT.read_text(encoding="utf-8"))
    rows = [row for row in payload["records"] if row["status"] == "MEASURED"]
    density = np.array([row["domainDensity"] for row in rows])
    mean_walks = np.array([row["walks"]["mean"] for row in rows])

    grid = np.linspace(density.min(), density.max(), 300)
    figure, axis = plt.subplots(figsize=(6.4, 3.7))
    axis.scatter(
        density,
        mean_walks,
        s=24,
        alpha=0.72,
        color="#1f5a99",
        edgecolors="white",
        linewidths=0.35,
        label="Policy mean (32 generations)",
    )
    axis.plot(
        grid,
        1.0 / grid,
        color="#a33b2b",
        linewidth=1.5,
        linestyle="--",
        label="Ideal large-domain reference $1/(N/M)$",
    )
    axis.set_xlabel("Domain density $N/M$")
    axis.set_ylabel("Mean primitive FF1 calls")
    axis.set_xlim(max(0.49, density.min() - 0.02), 1.0)
    axis.set_ylim(0.95, max(mean_walks.max(), (1.0 / grid).max()) + 0.18)
    axis.grid(True, linewidth=0.4, alpha=0.35)
    axis.legend(frameon=False, fontsize=8, loc="upper right")
    figure.tight_layout()
    figure.savefig(PDF, bbox_inches="tight")
    figure.savefig(PNG, dpi=300, bbox_inches="tight")


if __name__ == "__main__":
    main()
