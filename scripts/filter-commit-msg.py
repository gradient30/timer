"""Remove Cursor co-author trailers from commit messages."""

from __future__ import annotations

import sys

SKIP_PREFIXES = (
    "Co-authored-by: Cursor",
    "Co-Authored-By: Cursor",
)


def main() -> None:
    lines = sys.stdin.read().splitlines()
    kept = [line for line in lines if not any(line.startswith(p) for p in SKIP_PREFIXES)]
    text = "\n".join(kept).strip("\ufeff")
    sys.stdout.write(text + ("\n" if text else ""))


if __name__ == "__main__":
    main()
