#!/usr/bin/env bash
# tools/screen2-interact.sh — drive Screen 2's behaviors with REAL input and
# check the resulting pixels (client coords, SKETCH_DPR=1):
#   File opens the backstage; canvas press / Escape / File again / another
#   tab close it; a press inside it keeps it open; hover states; Exit quits.
set -u
cd "$(dirname "$0")/.."
OUT=target/s2i
mkdir -p "$OUT"
ps1() { powershell -NoProfile -ExecutionPolicy Bypass -File "$@"; }
click() { ps1 tools/click-real.ps1 -X "$1" -Y "$2" >/dev/null; }
shot() { ps1 tools/shot.ps1 -Out "$OUT/$1.png" >/dev/null; }
px() { python -c "from PIL import Image; im=Image.open('$OUT/$1.png').convert('RGB'); print(' '.join('%02x%02x%02x' % im.getpixel(tuple(map(int, p.split(',')))) for p in '$2'.split()))"; }
# Probes: menu body (100,300) = 252526 when open / ffffff (canvas) when closed;
# File tab (10,40); Home tab (60,40); View tab (130,40).
probe() { echo "$1: menu(100,300) file(10,40) home(60,40) view(130,40) = $(px "$1" '100,300 10,40 60,40 130,40')"; }

taskkill /IM sketch-app.exe /F >/dev/null 2>&1; sleep 1
(SKETCH_DPR=1 ./target/debug/sketch-app.exe > target/app.log 2>&1 &)
sleep 6
shot start; probe start

click 25 46; shot open1; probe open1                    # File -> open
click 100 700; shot inside; probe inside                # press inside menu -> stays open
click 600 400; shot canvas; probe canvas                # canvas press -> closed, File stays active
click 25 46; shot open2; probe open2
ps1 tools/click-real.ps1 -Escape >/dev/null; shot esc; probe esc   # Escape -> closed
click 25 46; click 25 46; shot toggle; probe toggle     # File twice -> closed
click 25 46; click 144 46; shot totab; probe totab      # open, then View tab -> closed, View active

# Hover: ribbon button (Mini-Map 12..72 x 62..124) and File tab while open.
(ps1 tools/click-real.ps1 -X 42 -Y 80 -HoverOnly -WaitMs 3500 >/dev/null &)
sleep 1.2; shot hover-btn
echo "hover-btn: Mini-Map bg(14,64) border(12,64) | idle Mini-Map neighbor Next bg(223,67) = $(px hover-btn '14,64 12,64 223,67')"
sleep 3
click 25 46
(ps1 tools/click-real.ps1 -X 25 -Y 50 -HoverOnly -WaitMs 3500 >/dev/null &)
sleep 1.2; shot hover-file
echo "hover-file: File tab bg(5,50) (006bb3 expected) = $(px hover-file '5,50')"
sleep 3

# Exit closes the app (backstage is open from the click above).
click 128 840; sleep 1.5
if tasklist | grep -qi sketch-app.exe; then echo "exit: STILL RUNNING"; else echo "exit: process ended"; fi
