# Documentation Fix Report: Tool Count & Language Count

## Summary

Fixed outdated "6 extractors (5 regex + 1 generic fallback)" text in the "When NOT to use m1nd"
section across all 6 i18n README translations. The main English README was already correct.
Also updated CONTRIBUTING.md project structure to mention 28 languages.

## Verified Correct (no changes needed)

| File | Status | Notes |
|------|--------|-------|
| `README.md` | OK | Already says 52 tools, 28 languages throughout |
| `CHANGELOG.md` | OK | v0.1.0 entry correctly says "43 MCP tools" (historical). v0.2.0 correctly says "52 tools (up from 43)" |
| `COMMIT-MESSAGE.md` | OK | "6 language translations" correctly refers to 6 translation files (PT-BR, ES, DE, FR, JA, ZH) |
| `EXAMPLES.md` | OK | No tool count or language count references |
| `USE-CASES.md` | OK | No tool count references (uses individual tool names) |
| `.github/wiki/Home.md` | OK | Already says 52 tools, 28 languages |
| `.github/wiki/Architecture.md` | OK | Already says 52 tools, 28 languages |
| `.github/wiki/Getting-Started.md` | OK | Already says 52 tools, 28 languages |
| `.github/wiki/API-Reference.md` | OK | "43,152 blast radius" is a data point, not a tool count |

## Changes Made

### 1. i18n/README.pt-BR.md (line 537)
- **Before:** "O build padrao vem com 6 extratores (5 regex + 1 fallback generico)"
- **After:** "O m1nd vem com extratores para 28 linguagens em dois tiers tree-sitter (entregues, nao planejados). O build padrao inclui Tier 2 (8 linguagens). Adicione `--features tier1` para habilitar todas as 28."

### 2. i18n/README.es.md (line 537)
- **Before:** "El build por defecto trae 6 extractores (5 regex + 1 fallback generico)"
- **After:** "m1nd viene con extractores para 28 lenguajes en dos tiers tree-sitter (entregados, no planificados). El build por defecto incluye Tier 2 (8 lenguajes). Agrega `--features tier1` para habilitar los 28."

### 3. i18n/README.de.md (line 538)
- **Before:** "Der Standard-Build kommt mit 6 Extraktoren (5 Regex + 1 generischer Fallback)"
- **After:** "m1nd wird mit Extraktoren fur 28 Sprachen in zwei tree-sitter-Stufen ausgeliefert (geliefert, nicht geplant). Der Standard-Build enthalt Stufe 2 (8 Sprachen). Fugen Sie `--features tier1` hinzu, um alle 28 zu aktivieren."

### 4. i18n/README.fr.md (line 538)
- **Before:** "Le build par defaut est livre avec 6 extracteurs (5 regex + 1 fallback generique)"
- **After:** "m1nd est livre avec des extracteurs pour 28 langages repartis en deux niveaux tree-sitter (livres, pas planifies). Le build par defaut inclut le Niveau 2 (8 langages). Ajoutez `--features tier1` pour activer les 28."

### 5. i18n/README.ja.md (line 535)
- **Before:** "6 extractors (5 regex + 1 generic fallback)" in Japanese
- **After:** "28 languages across two tree-sitter tiers (shipped, not planned)" in Japanese

### 6. i18n/README.zh.md (line 535)
- **Before:** "6 extractors (5 regex + 1 generic fallback)" in Chinese
- **After:** "28 languages across two tree-sitter tiers (shipped, not planned)" in Chinese

### 7. CONTRIBUTING.md (line 18)
- **Before:** `m1nd-ingest/   Language extractors (Python, Rust, TS/JS, Go, Java, generic)`
- **After:** `m1nd-ingest/   Language extractors (28 languages), memory adapter, JSON adapter`

## Tool Count Verification

| Category | Count | Verified |
|----------|-------|----------|
| Foundation | 13 | ingest, activate, impact, why, learn, drift, health, seek, scan, timeline, diverge, warmup, federate |
| Perspective Navigation | 12 | start, routes, follow, back, peek, inspect, suggest, affinity, branch, compare, list, close |
| Lock System | 5 | create, watch, diff, rebase, release |
| Superpowers | 13 | hypothesize, counterfactual, missing, resonate, fingerprint, trace, validate_plan, predict, trail.save, trail.resume, trail.merge, trail.list, differential |
| Superpowers Extended | 9 | antibody_scan, antibody_list, antibody_create, flow_simulate, epidemic, tremor, trust, layers, layer_inspect |
| **Total** | **52** | |

## Post-Fix Verification

After fixes, grep for "43 tool|43 MCP" across all md files returns only the historical CHANGELOG v0.1.0 entry (correct).
Grep for "6 extractor" variants returns zero matches (all fixed).
