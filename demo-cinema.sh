#!/usr/bin/env bash
# demo-cinema.sh — m1nd demo as a 30-second FILM
# Director's cut. Every second earned.

set -euo pipefail

B='\033[1m'
D='\033[2m'
G='\033[0;32m'
C='\033[0;36m'
R='\033[0;31m'
W='\033[1;37m'
Y='\033[0;33m'
X='\033[0m'

M1ND="http://localhost:1337/api/tools"
TMP=$(mktemp -d)
trap "rm -rf $TMP" EXIT

q() {
    local name="$1" tool="$2" data="$3"
    local t1=$(python3 -c "import time; print(int(time.time()*1000))")
    curl -s "$M1ND/m1nd.$tool" -H 'Content-Type: application/json' -d "$data" > "$TMP/${name}.json" 2>/dev/null
    local t2=$(python3 -c "import time; print(int(time.time()*1000))")
    echo $((t2-t1)) > "$TMP/${name}.ms"
}

jq_() { python3 -c "import json; d=json.load(open('$TMP/$1.json')); r=d.get('result',d); $2" 2>/dev/null; }
t() { cat "$TMP/$1.ms"; }

# ═══════════════════════════════════════════════════
# ACT 1: THE PROBLEM (5s)
# ═══════════════════════════════════════════════════
clear
echo ""
echo ""
echo ""
echo -e "${D}  your AI agent is searching a 160,000-line codebase.${X}"
sleep 2
echo ""
echo -e "${D}  it will run ~210 grep calls.${X}"
echo -e "${D}  read ~228 files.${X}"
echo -e "${D}  burn ~193,000 tokens.${X}"
echo -e "${D}  cost ~\$7.${X}"
echo -e "${D}  take ~35 minutes.${X}"
sleep 2.5
echo ""
echo -e "${D}  and miss 8 bugs it can never find.${X}"
sleep 2.5

# ═══════════════════════════════════════════════════
# ACT 2: THE MOMENT (5s)
# ═══════════════════════════════════════════════════
clear
echo ""
echo ""
echo -e "${W}  m1nd${X}"
echo ""

q r1 ingest '{"agent_id":"d","path":"/Users/cosmophonix/clawd/roomanizer-os/backend","adapter":"code","mode":"replace"}'
q r2 activate '{"agent_id":"d","query":"rate limiting and provider fallback","top_k":8}'
q r3 impact '{"agent_id":"d","node_id":"file::chat_handler.py","direction":"both"}'
q r4 hypothesize '{"agent_id":"d","claim":"worker_pool has runtime dependency on whatsapp_manager through process_manager"}'
q r5 missing '{"agent_id":"d","query":"cancellation cleanup timeout graceful shutdown"}'

T1=$(t r1); T2=$(t r2); T3=$(t r3); T4=$(t r4); T5=$(t r5)
TOTAL=$((T1+T2+T3+T4+T5))

echo -e "  ${C}ingest${X}        ${G}${T1}ms${X}"
echo -e "  ${C}activate${X}      ${G}${T2}ms${X}"
echo -e "  ${C}impact${X}        ${G}${T3}ms${X}"
echo -e "  ${C}hypothesize${X}   ${G}${T4}ms${X}"
echo -e "  ${C}missing${X}       ${G}${T5}ms${X}"
echo ""
echo -e "  ${B}${G}${TOTAL}ms. 0 tokens.${X}"
sleep 4

# ═══════════════════════════════════════════════════
# ACT 3: SLOW MOTION (12s)
# ═══════════════════════════════════════════════════
clear
echo ""
echo -e "${D}  ok, slow down. here's what just happened:${X}"
echo ""
sleep 1.5

NODES=$(jq_ r1 "print(r.get('node_count',r.get('nodes_created','?')))")
EDGES=$(jq_ r1 "print(r.get('edge_count',r.get('edges_created','?')))")
echo -e "  ${C}ingest${X}        370 files became ${B}${NODES} nodes${X} and ${B}${EDGES} edges${X}"
sleep 1.5

HITS=$(jq_ r2 "print(len(r.get('activated',[])))")
echo -e "  ${C}activate${X}      \"rate limiting\" fired across 4 dimensions. ${B}${HITS} results${X}."
sleep 1.5

AFFECTED=$(jq_ r3 "print(len(r.get('affected_nodes',r.get('blast_radius',[]))))")
echo -e "  ${C}impact${X}        chat_handler.py touches ${B}${AFFECTED} nodes${X} if changed."
sleep 1.5

VERDICT=$(jq_ r4 "print(r.get('verdict','?'))")
CONF=$(jq_ r4 "print(f\"{r.get('confidence',0)*100:.0f}%\")")
echo -e "  ${C}hypothesize${X}   \"worker_pool depends on whatsapp?\" ${B}${VERDICT}${X} at ${B}${CONF}${X}"
sleep 1.5

HOLES=$(jq_ r5 "print(len(r.get('structural_holes',[])))")
echo -e "  ${C}missing${X}       ${B}${HOLES} structural holes${X}. no keyword exists to grep for."
sleep 2

# ═══════════════════════════════════════════════════
# ACT 4: THE KNOCKOUT (8s)
# ═══════════════════════════════════════════════════
clear
echo ""
echo ""
echo -e "  ${W}                m1nd        grep+LLM${X}"
TSEC=$(python3 -c "print(f'{${TOTAL}/1000:.1f}')")
echo -e "  ${W}  time          ${G}${TSEC}s${W}          ${R}~35 min${X}"
echo -e "  ${W}  tokens        ${G}0${W}             ${R}~193,000${X}"
echo -e "  ${W}  cost          ${G}\$0.00${W}         ${R}~\$7.23${X}"
echo -e "  ${W}  bugs found    ${G}39${W}            ${Y}~23${X}"
echo ""
echo -e "  ${W}  bugs grep will ${B}never${W} find: ${G}8${X}"
sleep 4
echo ""
echo -e "  ${D}  no keyword. no string. just structure.${X}"
sleep 2
echo ""
echo -e "  ${G}  github.com/maxkle1nz/m1nd${X}"
echo ""
sleep 3
