"""Generate a 1024x1024 clock icon for TimerApp (Tauri bundle source)."""

from __future__ import annotations

import math
from pathlib import Path

from PIL import Image, ImageDraw

SIZE = 1024
OUTPUT = Path(__file__).resolve().parents[1] / "timer" / "src-tauri" / "icons" / "app-icon.png"

BG = (11, 22, 34, 255)
FACE = (18, 36, 52, 255)
ACCENT = (0, 212, 255, 255)
HAND = (224, 244, 255, 255)
CENTER = (0, 212, 255, 255)


def hand_endpoint(cx: int, cy: int, length: float, hour: int, minute: int, *, minute_hand: bool) -> tuple[float, float]:
    if minute_hand:
        deg = minute * 6
    else:
        deg = (hour % 12) * 30 + minute * 0.5
    rad = math.radians(deg)
    return cx + length * math.sin(rad), cy - length * math.cos(rad)


def main() -> None:
    img = Image.new("RGBA", (SIZE, SIZE), (0, 0, 0, 0))
    draw = ImageDraw.Draw(img)

    cx = cy = SIZE // 2
    radius = 430

    draw.ellipse((cx - radius, cy - radius, cx + radius, cy + radius), fill=BG)
    draw.ellipse((cx - radius, cy - radius, cx + radius, cy + radius), outline=ACCENT, width=30)

    inner = radius - 56
    draw.ellipse((cx - inner, cy - inner, cx + inner, cy + inner), fill=FACE)

    for hour in range(12):
        rad = math.radians(hour * 30)
        outer = inner - 24
        inner_tick = inner - (72 if hour % 3 == 0 else 48)
        width = 18 if hour % 3 == 0 else 10
        color = ACCENT if hour % 3 == 0 else (0, 170, 210, 180)
        x1, y1 = cx + outer * math.sin(rad), cy - outer * math.cos(rad)
        x2, y2 = cx + inner_tick * math.sin(rad), cy - inner_tick * math.cos(rad)
        draw.line((x1, y1, x2, y2), fill=color, width=width)

    hour = 10
    minute = 10
    hx, hy = hand_endpoint(cx, cy, inner * 0.46, hour, minute, minute_hand=False)
    mx, my = hand_endpoint(cx, cy, inner * 0.68, hour, minute, minute_hand=True)
    draw.line((cx, cy, hx, hy), fill=HAND, width=34)
    draw.line((cx, cy, mx, my), fill=ACCENT, width=24)
    draw.ellipse((cx - 22, cy - 22, cx + 22, cy + 22), fill=CENTER)

    OUTPUT.parent.mkdir(parents=True, exist_ok=True)
    img.save(OUTPUT)
    print(f"Wrote {OUTPUT}")


if __name__ == "__main__":
    main()
