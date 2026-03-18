# m1nd — Melhores Capacidades e Cross-Tooling

**Data:** 2025-03-15  
**Autor:** JIMI (testes empíricos via HTTP API localhost:1337)

---

## 1. Visão Geral

m1nd é um motor neuro-simbólico de análise de código. O grafo contém nós (arquivos, funções, structs, conceitos de memória) e arestas (imports, calls, contains, semantic). As ferramentas exploram esse grafo de formas complementares.

**Regra m1nd-first:** Use m1nd antes de grep/glob para exploração. Ordem: `ingest` (se vazio) → `activate`/`seek`/`impact`/`why`/`missing` → depois grep para matches exatos.

---

## 2. Ferramentas por Capacidade

### 2.1 Exploração Semântica

| Ferramenta | Melhor para | Latência | Output chave |
|------------|-------------|----------|--------------|
| **activate** | "O que está relacionado a X?" — spreading activation 4D | ~100–200ms | `activated`, `seeds`, `ghost_edges`, `structural_holes`, `plasticity` |
| **seek** | Busca por intenção natural — mais focada que activate | ~130ms | `results` com `score_breakdown`: embedding_similarity, graph_activation, temporal_recency |
| **warmup** | Pré-aquecer o grafo para uma tarefa | ~50ms | Primes seeds para queries subsequentes |

**Diferencial seek vs activate:**
- **seek** usa embeddings + grafo + recência. Retorna docs (`memory::docs::entry::...`), conceitos, structs, funções. Score breakdown explícito.
- **activate** usa spreading activation puro. Retorna ghost edges (arestas que "deveriam existir") e structural holes (regiões incompletas).

**Exemplo seek:** `query: "ghost edge structural hole detection"` → StructuralHole, GhostEdge, detect_ghost_edges, detect_structural_holes em query.rs + docs em STORYBOARD.md, VERIFICATION_REPORT.md.

---

### 2.2 Análise Estrutural

| Ferramenta | Melhor para | Latência | Output chave |
|------------|-------------|----------|--------------|
| **impact** | Raio de explosão antes de modificar código | ~50ms | `blast_radius`, `affected_nodes`, `risk_level` |
| **why** | Caminho entre dois nós | ~30ms | `path`, `hops` |
| **missing** | "O que falta que deveria existir?" | ~45–70ms | `holes` com region, adjacent_nodes, description |
| **layers** | Detecção de camadas arquiteturais | ~180ms | `layers` (L0 tests, L1 core, etc.), `violations` |
| **fingerprint** | Duplicatas / gêmeos estruturais | 1–107ms | Pares com similarity score |

---

### 2.3 Debugging e Stacktrace

| Ferramenta | Melhor para | Latência | Output chave |
|------------|-------------|----------|--------------|
| **trace** | Mapear stacktrace → suspeitos de root cause | ~6ms (Rust) | `suspects`, `causal_chain`, `fix_scope` (blast_radius, risk_level), `frames_mapped` |

**Requisito:** `error_text` com stacktrace real (Rust, Python, etc.). Formato genérico não mapeia frames.

**Exemplo:** Stacktrace Rust em `query.rs:267` → `suspects: [detect_ghost_edges]`, `fix_scope: blast_radius 40, risk_level critical`, `causal_chain: ["detect_ghost_edges"]`.

---

### 2.4 Navegação Guiada (Perspective)

| Ferramenta | Melhor para | Latência | Output chave |
|------------|-------------|----------|--------------|
| **perspective.start** | Iniciar navegação a partir de query ou anchor | ~150ms | `perspective_id`, `routes` (com provenance, score), `suggested` |
| **perspective.follow** | Seguir uma rota (drill-down) | ~150ms | `new_focus`, novas `routes`, `route_set_version` |
| **perspective.peek** | Inspecionar conteúdo de uma rota | ~150ms | Requer `route_set_version` atual (muda após follow) |

**Modos:** `Local` (query-driven) ou `Anchored` (pinned a `anchor_node`). Anchored degrada para local após 8 hops.

**Exemplo:** `anchor_node: file::m1nd-core/src/activation.rs` → routes: HybridEngine, ActivatedNode, partial_cmp, eq, Temporal, HeapEntry. Follow R_84deae → focus em HybridEngine → novas routes: topology.rs, counterfactual.rs, query.rs, lib.rs.

---

### 2.5 Validação e Risco

| Ferramenta | Melhor para | Latência | Output chave |
|------------|-------------|----------|--------------|
| **validate_plan** | Pré-voo: blast radius, gaps, risco | ~7ms | `blast_radius_total`, `gaps` (critical/warning), `suggested_additions`, `test_coverage`, `risk_score` |
| **counterfactual** | Simular remoção de nós | ~3ms | Cascade de impacto |
| **predict** | Co-change após modificar módulo | ~50ms | Arquivos que tendem a mudar junto |

---

### 2.6 Análise Avançada

| Ferramenta | Melhor para | Latência | Output chave |
|------------|-------------|----------|--------------|
| **resonate** | Ondas estacionárias — hubs onde sinal reforça | ~50ms | `antinodes`, `sympathetic_pairs`, `resonant_frequency` |
| **hypothesize** | Testar claims Bayesianos | ~100ms | `confidence`, `verdict` (likely_true/likely_false), `evidence` |
| **flow_simulate** | Race conditions via partículas concorrentes | ~6s (200 partículas) | `turbulence_points`, `valve_points`, `paths` (traces de colisão) |
| **epidemic** | Propagação SIR de bugs | ~38ms | `predictions`, `R0`, `peak_infected` |
| **layers** | Camadas + violações | ~180ms | Output grande (L0–L5, node_count, violations) |

**flow_simulate:** 461 turbulence points no m1nd codebase (activation engine). temporal.rs é hotspot de colisão. `valve_points` = gargalos de lock.

---

## 3. Pipelines Cross-Tool Recomendados

### 3.1 Debugging: trace → activate → why

```
1. trace(error_text=<stacktrace>)     → suspects, causal_chain, fix_scope
2. activate(query=<suspect_label>)    → contexto ao redor do suspeito
3. why(from=<caller>, to=<suspect>)   → caminho exato
```

### 3.2 Pré-edição: impact → validate_plan → missing

```
1. impact(node_id=<file>)             → blast radius do arquivo
2. validate_plan(actions=[...])       → gaps críticos, suggested_additions
3. missing(query=<region>)           → structural holes na região
```

### 3.3 Exploração guiada: warmup → activate → perspective → resonate

```
1. warmup(task=<tema>)               → prime seeds
2. activate(query=<tema>)            → activated + structural_holes
3. perspective.start(query=<tema>, anchor_node=<top_activated>)
4. perspective.follow(route_id=...)  → drill-down
5. resonate(query=<tema>)            → antinodes, harmonic groups
```

### 3.4 Validação de hipóteses: activate → hypothesize → learn

```
1. activate(query=<claim>)           → structural_holes, ghost_edges
2. hypothesize(claim="<afirmação>", evidence_from="activate")  → confidence, verdict
3. learn(feedback="correct"|"wrong", node_ids=[...])  → Hebbian update
```

### 3.5 Race / propagação: flow_simulate → epidemic → validate_plan

```
1. flow_simulate(entry_nodes=[...], num_particles=4)  → turbulence_points
2. epidemic(infected_nodes=<turbulence hotspots>)      → R0, predictions
3. validate_plan(actions=[fix files])                 → risk antes de ship
```

### 3.6 Busca conceitual: seek → activate (structural_holes)

```
1. seek(query="<conceito>")          → docs + code (embedding + graph)
2. activate(query="<conceito>")      → ghost_edges, structural_holes (o que falta)
```

---

## 4. Casos de Uso Inventados (Cross-Tooling)

### 4.1 "Bug Hunt com Perspective"

Stacktrace → trace(suspects) → perspective.start(anchor=top_suspect) → follow rotas até caller → peek para ver código.

### 4.2 "Gap Analysis antes de Spec"

missing(region) → activate(region) → structural_holes + ghost_edges → lista de "o que implementar".

### 4.3 "Resonance + Flow para Hotspots"

resonate(query) → antinodes = hubs estruturais. flow_simulate(entry_nodes=antinodes) → turbulence nos hubs = race candidates.

### 4.4 "Validate Plan + Test Coverage"

validate_plan(actions) → suggested_additions (incl. test) → test_coverage.untested_files = prioridade de testes.

### 4.5 "Seek + Learn para Memória"

seek(query) → resultados usados → learn(feedback="correct", node_ids) → próxima seek mais relevante.

### 4.6 "Hypothesize + Activate para Claims"

"activation depende de query para ghost edges" → hypothesize(claim) → 3.7% confidence, verdict likely_false. Activate fornece evidence (path_found vs no_path).

---

## 5. Pontos de Atenção

- **perspective.peek:** Requer `route_set_version` do último `start` ou `follow`. Version muda após cada follow.
- **trace:** Stacktrace deve ser formato real (Rust, Python, etc.). Mensagens genéricas não mapeiam.
- **flow_simulate:** Com `query` auto-descobre entry points. Pode gerar output grande (3MB+ com paths).
- **layers:** Output muito grande; filtrar por violations ou L0/L1 se necessário.
- **seek vs activate:** seek = "encontrar isto"; activate = "o que está conectado + o que falta".

---

## 6. Referências

- API Reference: `mcp/m1nd/.github/wiki/API-Reference.md`
- Use Cases: `mcp/m1nd/.github/wiki/Use-Cases.md`
- Verification Report: `mcp/m1nd/.github/wiki/VERIFICATION_REPORT.md`
- Regra m1nd-first: `.cursor/rules/m1nd-first.mdc`
