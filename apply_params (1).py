#!/usr/bin/env python3
"""
apply_params.py — cur_stim parameter patcher
=============================================
Usage (run from the root of the cur_stim repo in Codespaces):
    python3 apply_params.py [params.json]

Reads params.json (produced by stimulator_gui.html), patches src/main.rs with
the configured values, then runs `cargo bin` to produce the binary.

Switch states and pulse sequences follow "Analog Circuit Details":
  - S1 connects DAC+,  S2 connects DAC-,  S0 connects the source capacitor,
    S3 bridges the DAC- branch to the common output node.
  - s4/s5 = DAC power (high), s6/s7 = chopper+filter, s8 = 1.5V DC. These are
    left at their existing initial values and are not changed by mode.

Per-mode behaviour (from the PDF):

  Biphasic square wave
    initial: S0 low, S1 low, S2 low, S3 high
    loop:    S2 high  -> delay1 (negative pulse)
             S2 low   -> delay2 (inter-phase gap)
             S1 high  -> delay3 (positive pulse)
             S1 low   -> delay4 (post-stimulation period)

  Capacitor-coupled biphasic
    initial: S0 high, S1 low, S2 low, S3 low
    loop:    S1+S2 high -> delay1 (negative pulse + capacitor charge)
             S1+S2 low  -> delay2 (inter-phase gap)
             S3 high    -> delay3 (positive pulse = capacitor discharge)
             S3 low     -> delay4 (post-stimulation period)

  Monophasic square wave (DOC-style single phase)
    initial: S0 low, S1 low, S2 low, S3 high
    loop:    S1 high -> delay1 (positive pulse)
             S1 low  -> delay2 (post-stimulation period)

Parameter -> delay mapping:
    t1   = positive pulse length
    t2   = negative pulse length
    gap  = inter-phase gap (delay2 in biphasic / capacitor modes)
    post = period after stimulation (delay4; defaults to 300 us if absent)
"""

import json
import re
import subprocess
import sys
import shutil
from pathlib import Path
from datetime import datetime, timezone

# ── locate files ──────────────────────────────────────────────────────────────
PARAMS_FILE = Path(sys.argv[1]) if len(sys.argv) > 1 else Path("params.json")
MAIN_RS     = Path("src/main.rs")
BACKUP_RS   = Path("src/main.rs.bak")


def die(msg):
    print(f"\n[ERROR] {msg}")
    sys.exit(1)


if not PARAMS_FILE.exists():
    die(f"Could not find {PARAMS_FILE}.\n"
        "  -> Download params.json from the GUI, place it in the repo root,\n"
        "     then re-run:  python3 apply_params.py")

if not MAIN_RS.exists():
    die(f"Could not find {MAIN_RS}. Run this script from the cur_stim repo root.")

# ── load & validate params ────────────────────────────────────────────────────
print(f"\n[1/4] Loading {PARAMS_FILE} ...")
with open(PARAMS_FILE) as f:
    p = json.load(f)

required = ["mode", "ipos", "ineg", "t1", "t2", "gap", "prescaler"]
for k in required:
    if k not in p:
        die(f"Missing key '{k}' in {PARAMS_FILE}. Re-export from the GUI.")

mode      = p["mode"]                 # "biphasic" | "capacitor" | "monophasic"
ipos      = float(p["ipos"])
ineg      = float(p["ineg"])
t1        = int(p["t1"])
t2        = int(p["t2"])
gap       = int(p["gap"])
prescaler = int(p["prescaler"])
post      = int(p.get("post", 300))   # delay4 / post-stimulation period (us)

if mode not in ("biphasic", "capacitor", "monophasic"):
    die(f"Unknown mode '{mode}'. Expected biphasic, capacitor, or monophasic.")

# clamp to hardware limits
ipos = max(0.1, min(1.6, ipos))
ineg = max(-1.6, min(0.0, ineg))
if mode == "monophasic":
    ineg = 0.0
t1        = max(50, min(10000, t1))
t2        = max(50, min(10000, t2))
gap       = max(0,  min(2000,  gap))
post      = max(0,  min(100000, post))
prescaler = max(1,  min(100,   prescaler))

print(f"    mode       = {mode}")
print(f"    Ipos       = {ipos:.1f} mA")
print(f"    Ineg       = {ineg:.1f} mA")
print(f"    t1 (pos)   = {t1} us")
print(f"    t2 (neg)   = {t2} us")
print(f"    gap        = {gap} us")
print(f"    post       = {post} us")
print(f"    prescaler  = {prescaler}")

# ── per-mode initial switch states (from Analog Circuit Details) ───────────────
# Only s0..s3 differ by mode. s4..s8 keep their existing initial values.
SWITCH_INITIAL = {
    "biphasic":   {"s0": "low",  "s1": "low", "s2": "low", "s3": "high"},
    "capacitor":  {"s0": "high", "s1": "low", "s2": "low", "s3": "low"},
    "monophasic": {"s0": "low",  "s1": "low", "s2": "low", "s3": "high"},
}


def loop_body(mode, t1, t2, gap, post):
    if mode == "biphasic":
        return (
            "        // Biphasic square wave -- negative phase first, then positive\n"
            "        s2.set_high();       // S2 high: DAC- -> negative phase ON\n"
            f"        delay_us({t2});      // delay 1: negative pulse length\n"
            "        s2.set_low();        // S2 low\n"
            f"        delay_us({gap});      // delay 2: inter-phase gap\n"
            "        s1.set_high();       // S1 high: DAC+ -> positive phase ON\n"
            f"        delay_us({t1});      // delay 3: positive pulse length\n"
            "        s1.set_low();        // S1 low\n"
            f"        delay_us({post});     // delay 4: period after stimulation"
        )
    elif mode == "capacitor":
        return (
            "        // Capacitor-coupled biphasic\n"
            "        s1.set_high();       // S1 & S2 high: charge cap / negative phase\n"
            "        s2.set_high();\n"
            f"        delay_us({t2});      // delay 1: negative pulse + capacitor charge\n"
            "        s1.set_low();        // S1 & S2 low\n"
            "        s2.set_low();\n"
            f"        delay_us({gap});      // delay 2: inter-phase gap\n"
            "        s3.set_high();       // S3 high: capacitor discharge -> positive phase\n"
            f"        delay_us({t1});      // delay 3: positive pulse length\n"
            "        s3.set_low();        // S3 low\n"
            f"        delay_us({post});     // delay 4: period after stimulation"
        )
    else:  # monophasic
        return (
            "        // Monophasic square wave -- single positive phase\n"
            "        s1.set_high();       // S1 high: DAC+ -> positive phase ON\n"
            f"        delay_us({t1});      // delay 1: positive pulse length\n"
            "        s1.set_low();        // S1 low\n"
            f"        delay_us({post});     // delay 2: period after stimulation"
        )


# ── brace matcher that ignores comments/strings ───────────────────────────────
def find_matching_brace(text, open_idx):
    """Given index of an opening '{', return index of its matching '}',
    skipping // line comments, /* */ block comments, and "..." strings."""
    depth = 0
    i = open_idx
    n = len(text)
    in_line = in_block = in_str = False
    while i < n:
        c = text[i]
        nxt = text[i + 1] if i + 1 < n else ""
        if in_line:
            if c == "\n":
                in_line = False
        elif in_block:
            if c == "*" and nxt == "/":
                in_block = False
                i += 1
        elif in_str:
            if c == "\\":
                i += 1
            elif c == '"':
                in_str = False
        else:
            if c == "/" and nxt == "/":
                in_line = True
                i += 1
            elif c == "/" and nxt == "*":
                in_block = True
                i += 1
            elif c == '"':
                in_str = True
            elif c == "{":
                depth += 1
            elif c == "}":
                depth -= 1
                if depth == 0:
                    return i
        i += 1
    return -1


# ── patch main.rs ─────────────────────────────────────────────────────────────
print("\n[2/4] Patching src/main.rs ...")

shutil.copy(MAIN_RS, BACKUP_RS)
print(f"    Backup saved -> {BACKUP_RS}")

src = MAIN_RS.read_text(encoding="utf-8")
changes = []


def replace_line(src, pattern, replacement, description, count=1):
    new_src, n = re.subn(pattern, replacement, src, count=count)
    if n == 0:
        print(f"    [WARN] Could not patch: {description} (pattern not found)")
    else:
        changes.append(description)
    return new_src


# 1. prescaler
src = replace_line(
    src,
    r'(tim1_config\.prescaler\s*=\s*)\d+(\s*-\s*1\s*;)',
    rf'\g<1>{prescaler}\2',
    f"prescaler = {prescaler} - 1",
)

# 2. Ipos  (matches `let Ipos = 0.7;`)
src = replace_line(
    src,
    r'(let\s+Ipos\s*=\s*)-?[\d.]+(\s*;)',
    rf'\g<1>{ipos:.1f}\2',
    f"Ipos = {ipos:.1f}",
)

# 3. Ineg  (matches `let Ineg = 0.0;` / `let Ineg = -1.0;`)
src = replace_line(
    src,
    r'(let\s+Ineg\s*=\s*)-?[\d.]+(\s*;)',
    rf'\g<1>{ineg:.1f}\2',
    f"Ineg = {ineg:.1f}",
)

# 4. t1  (matches `let mut t1 = 1000;` keeping any trailing comment)
src = replace_line(
    src,
    r'(let\s+mut\s+t1\s*=\s*)\d+(\s*;)',
    rf'\g<1>{t1}\2',
    f"t1 = {t1}",
)

# 5. t2  (matches `let mut t2 = 1200;`)
src = replace_line(
    src,
    r'(let\s+mut\s+t2\s*=\s*)\d+(\s*;)',
    rf'\g<1>{t2}\2',
    f"t2 = {t2}",
)

# 6. initial switch states for s0..s3 (lines ~123-133). s4..s8 untouched.
#    Each regex targets the FIRST occurrence, which is the initial setup block
#    (the loop body that follows is rewritten wholesale in step 7).
init = SWITCH_INITIAL[mode]
for sw in ("s0", "s1", "s2", "s3"):
    state = init[sw]
    src = replace_line(
        src,
        rf'({sw}\.)set_(?:high|low)(\(\))',
        rf'\g<1>set_{state}\2',
        f"{sw}.set_{state}() (initial, {mode})",
    )

# 7. rewrite the stimulation loop body
m = re.search(r'\bloop\s*\{', src)
if not m:
    shutil.copy(BACKUP_RS, MAIN_RS)
    die("Could not find the `loop {` block in main.rs. No changes applied.")

open_brace = src.index("{", m.start())
close_brace = find_matching_brace(src, open_brace)
if close_brace == -1:
    shutil.copy(BACKUP_RS, MAIN_RS)
    die("Could not find the matching `}` for the loop. No changes applied.")

new_loop = "loop {\n" + loop_body(mode, t1, t2, gap, post) + "\n    }"
src = src[: m.start()] + new_loop + src[close_brace + 1:]
changes.append(f"loop body rewritten ({mode} mode)")

# 8. stamp a generated header comment at the top
stamp = (
    f"// [PATCHED by apply_params.py -- {datetime.now(timezone.utc).strftime('%Y-%m-%d %H:%M')} UTC]\n"
    f"// mode={mode} Ipos={ipos} Ineg={ineg} t1={t1} t2={t2} gap={gap} post={post} prescaler={prescaler}\n"
)
src = re.sub(r'// \[PATCHED by apply_params\.py[^\n]*\n[^\n]*\n', '', src)
src = stamp + src

MAIN_RS.write_text(src, encoding="utf-8")
print(f"    Patched {len(changes)} item(s):")
for c in changes:
    print(f"      [ok] {c}")

# ── build ─────────────────────────────────────────────────────────────────────
print("\n[3/4] Running cargo bin ...")
print("      (this usually takes 30-90 seconds)\n")

result = subprocess.run(["cargo", "bin"])

if result.returncode != 0:
    print("\n[ERROR] cargo bin failed.")
    print(f"  Restoring original main.rs from {BACKUP_RS} ...")
    shutil.copy(BACKUP_RS, MAIN_RS)
    print("  Original restored. Fix the error above and try again.")
    sys.exit(1)

# ── locate binary ─────────────────────────────────────────────────────────────
print("\n[4/4] Locating binary ...")
bin_file = None
for c in sorted(Path(".").glob("**/*.bin")):
    if "u5_example" in c.name:
        bin_file = c
        break
if not bin_file:
    candidates = sorted(Path(".").glob("*.bin"))
    bin_file = candidates[0] if candidates else None

if bin_file:
    size_kb = bin_file.stat().st_size / 1024
    print(f"\n  [done] Binary ready: {bin_file}  ({size_kb:.1f} KB)")
    print(f"\n  -> Right-click '{bin_file.name}' in the Codespaces file explorer")
    print("     and select 'Download'. Then copy it to the Nucleo-144 drive to flash.\n")
else:
    print("\n  [WARN] Could not find a .bin file -- check the target/ directory manually.\n")

print("Done.\n")
