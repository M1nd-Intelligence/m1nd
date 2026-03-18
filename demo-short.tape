# m1nd SHORT demo — ~42 seconds, real queries, cost comparison
# Timing:
#   0-2s:   command (hook)
#   2-10s:  queries execute live
#   10-18s: results + total (digest) — 8s hold
#   18-24s: grep comparison (contrast) — 6s hold
#   24-32s: cost table (the knockout) — 8s hold
#   32-36s: punchline + CTA (hold) — 4s hold

Output demo-short.gif

Set FontSize 14
Set Width 900
Set Height 700
Set Theme "Catppuccin Mocha"
Set Padding 20
Set TypingSpeed 25ms
Set PlaybackSpeed 1

Sleep 500ms
Type "bash demo-short.sh"
Sleep 500ms
Enter

# queries execute + total block visible
Sleep 18s

# grep comparison — let contrast sink in
Sleep 6s

# cost table — the main content, full 8s hold
Sleep 8s

# punchline + CTA — final hold
Sleep 4s
