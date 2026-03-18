# m1nd Stress Test — soundmap-frontend Bug Hunt (Duplo-Cego)

**Data:** 2025-03-15  
**Codebase:** `/Users/cosmophonix/clawd/RESON-mapster/soundmap-frontend/src`  
**Ingest:** 2533 nodes, 1916 edges, 623 files, 236ms

---

## Metodologia

**Missão:** Encontrar bugs reais no soundmap-frontend (terreno fértil — audit prévio documentou 10+ componentes score 1–3, dead code, mock data, hardcoded strings).

**Agent A (m1nd):** Usa activate, seek, missing, impact, flow_simulate.  
**Agent B (blind):** Usa apenas grep, glob. Sem m1nd, sem semantic search.

**Timebox:** ~3 min por agente (simulado em sequência).

---

## Resultados por Agente

### Agent A (m1nd) — 248ms total

| Query | Tool | Resultado |
|-------|------|-----------|
| console.error hardcoded | activate | errorReporter.ts, PlayerProvider handleError, onError — **ranked por relevância** |
| dead code 0 imports | activate | **socialServiceOptimized.ts**, **personaServiceWithErrors.ts** — ambos 0 imports (confirmado no audit) |
| mock data TODO | seek | **generateMockSpotifyData**, **generateMockInstagramData**, mockData.ts, externalPlatformService — **preciso** |
| AdminPanel blast radius | impact | **72 nodes** afetados — monólito 500+ linhas |
| Mapbox token | seek | artist-canvas types (falso positivo — "token" semântico diferente) |

**Bugs/riscos identificados:**
1. socialServiceOptimized, personaServiceWithErrors — dead code (0 imports)
2. externalPlatformService — mock data em produção
3. AdminPanel — blast radius 72, monólito
4. errorReporter/PlayerProvider — error handling centralizado mas 368 console.error no codebase

**flow_simulate:** 0 turbulence (2 particles, threshold 0.5) — grafo TS/React menos propenso a races que Python async.

---

### Agent B (blind) — ~220ms total

| Query | Ferramenta | Resultado |
|-------|------------|-----------|
| console.error | grep | **368 linhas** — qual importa? **Sem ranking.** |
| mock TODO placeholder | grep | 8 arquivos — precisa abrir cada um para verificar |
| "Arquivos com 0 imports" | — | **Impossível.** Grep não tem grafo. |
| Mapbox token | grep | App.tsx, main.tsx, types — não encontrou config/mapbox.ts diretamente |
| Blast radius AdminPanel | — | **Impossível.** Grep não calcula dependências. |
| Duplicate services | grep | socialService, personaService — qual é redundante? **Análise manual.** |

**Bugs/riscos identificados:**
1. 368 console.error — overwhelming, sem priorização
2. Arquivos com mock/TODO — lista bruta, sem contexto
3. **Não conseguiu:** dead code (0 imports), blast radius, redundância de serviços

---

## Comparativo

| Métrica | Agent A (m1nd) | Agent B (blind) |
|---------|----------------|-----------------|
| **Tempo** | 248ms | ~220ms |
| **Bugs reais encontrados** | 4 (dead code, mock, monólito, error handling) | 2 (console.error, mock files) |
| **Precisão** | Ranked, contextualizado | Bruto, sem prioridade |
| **Queries impossíveis** | 0 | 3 (0 imports, blast radius, redundância) |
| **Falsos positivos** | 1 (Mapbox seek → artist-canvas) | 0 (grep é literal) |

---

## Bugs Confirmados pelo Audit Prévio

Do `war-rooms/reson-audit/`:

| Bug | Agent A | Agent B |
|-----|---------|---------|
| socialServiceOptimized 0 imports | ✅ activate | ❌ |
| personaServiceWithErrors 0 imports | ✅ activate | ❌ |
| provinceChatService.legacy 0 imports | ⚠️ não testado | ❌ |
| externalPlatformService mock data | ✅ seek | ⚠️ grep encontrou arquivos, não função |
| AdminPanel 500+ linhas monólito | ✅ impact 72 nodes | ❌ |
| SoundmapCard 50 console.log comentados | ⚠️ não priorizado | ❌ |
| Dashboard massive mock data | ⚠️ seek mock | ⚠️ grep mock |

---

## Conclusões

1. **m1nd encontra bugs estruturais que grep não pode:** 0 imports, blast radius, redundância.
2. **m1nd ranqueia:** 368 console.error → m1nd devolve errorReporter, PlayerProvider como top. Grep devolve tudo.
3. **Tempo comparável:** m1nd ~248ms vs grep ~220ms. m1nd entrega mais valor no mesmo tempo.
4. **seek pode ter falsos positivos:** "Mapbox token" retornou artist-canvas (token = design token). activate com query mais específica ajudaria.
5. **flow_simulate** em TS/React: 0 turbulence com 2 particles. Código síncrono; races são raras. Em backend Python async, turbulence seria maior.

---

## Recomendações para Stress Test Futuro

1. **Ingest merge** com roomanizer-os para testar activate em codebase híbrida.
2. **flow_simulate** com mais particles (4–8) e entry_nodes explícitos.
3. **trace** com stacktrace real de soundmap (se houver).
4. **validate_plan** com ações de refactor (ex.: deletar socialServiceOptimized).
5. **Agent B+:** adicionar semantic search para comparação tripla (m1nd vs grep vs semantic). *Nota: semantic search no Cursor pode estar scoped ao workspace roomanizer-os; soundmap está em RESON-mapster.*

6. **Re-ingest roomanizer-os** após o teste — o ingest replace limpou o grafo anterior.
