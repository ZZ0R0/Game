#!/usr/bin/env python3
import sys
from pathlib import Path


def count_rs_lines(root: Path) -> int:
    total = 0
    for p in root.rglob("*.rs"):
        try:
            with p.open("r", encoding="utf-8", errors="ignore") as f:
                total += sum(1 for _ in f)
        except Exception as e:
            print(f"Impossible de lire {p}: {e}", file=sys.stderr)
    return total


def main():
    if len(sys.argv) < 2:
        print("Usage: python liner.py <repertoire>")
        sys.exit(1)
    root = Path(sys.argv[1]).resolve()
    if not root.is_dir():
        print(f"Non trouvé ou pas un dossier: {root}", file=sys.stderr)
        sys.exit(1)
    total = count_rs_lines(root)
    print(total)


if __name__ == "__main__":
    main()
