# m1nd Ingest — Cálculo Matemático e Otimização

**Data:** 2025-03-15

---

## 1. Benchmarks de referência (API Reference + demo)

| Fonte | Arquivos | Tempo | ms/arquivo |
|-------|----------|-------|------------|
| API Reference (code) | 335 | 910 ms | **2.72** |
| API Reference (memory) | 82 | 138 ms | **1.68** |
| demo-real (backend) | 370 | 1 391 ms | **3.76** |
| stress test (soundmap) | 623 | 236 ms | **0.38** |

**Variação:** 0.38–3.76 ms/arquivo (depende de linguagem, tamanho, complexidade).

---

## 2. Fórmula estimada

```
T(segundos) ≈ N_arquivos × k

k = 2.0 ms/arquivo (média conservadora para code)
k = 1.7 ms/arquivo (memory adapter)
```

---

## 3. Cálculo por pasta

| Pasta | Arquivos | Adapter | k | T estimado |
|-------|----------|---------|---|------------|
| **clawd/memory** | 106 | code (md→generic) | ~1.5 | **0.16 s** |
| **clawd/memory** | 106 | memory | 1.68 | **0.18 s** |
| **clawd (inteiro)** | 39 794 | code | 2.0 | **79.6 s** ≈ **1 min 20 s** |
| **roomanizer-os/backend** | 370 | code | 3.76 | **1.4 s** |
| **soundmap-frontend/src** | 623 | code | 0.38 | **0.24 s** |

---

## 4. Por que memory falhou com 30s?

- **memory (106 files):** ~0.2 s — deveria passar.
- **clawd inteiro (40k files):** ~80 s — estoura 30 s.

O timeout de 30 s corta ingests grandes. Com 120 s, clawd inteiro (~80 s) passa.

---

## 5. O ingest pode ser mais rápido?

### Já existe

- **Rayon par_iter** — extração em paralelo (default 8 threads)
- **Skip dirs** — node_modules, .git, target, dist, etc.
- **Timeout interno** — 300 s no IngestConfig

### Possíveis otimizações

| Otimização | Impacto | Esforço |
|------------|---------|---------|
| **Aumentar parallelism** | 8 → 16 ou num_cpus | Médio — I/O pode limitar |
| **Incremental ingest** | Só arquivos modificados (git diff) | Alto — precisa diff + merge |
| **Cache de extração** | Reusar resultado se mtime inalterado | Alto |
| **Extractor mais leve para .md** | Generic é regex; memory adapter é mais rápido | Médio — usar memory para .md |
| **Graph building paralelo** | Hoje é sequencial após extração | Alto — Graph::add_node pode ter contenção |
| **Skip arquivos > N KB** | Arquivos gigantes (ex.: 2026-03-13.md 54KB) | Baixo |
| **Lazy PageRank** | Calcular PageRank só quando necessário | Médio |

---

## 6. Estimativa de ganho

| Cenário | Tempo atual | Com parallelism 16 | Com incremental (10% changed) |
|---------|-------------|-------------------|-------------------------------|
| clawd 40k | ~80 s | ~50–60 s | ~8 s |
| memory 106 | ~0.2 s | ~0.15 s | irrelevante |

**Maior ganho:** incremental ingest — em codebases grandes, poucos arquivos mudam entre sessões.

---

## 7. Resumo

| Pergunta | Resposta |
|----------|----------|
| **Quanto tempo leva?** | ~2 ms/arquivo (code), ~1.7 ms (memory). clawd 40k ≈ 80 s. |
| **Pode melhorar?** | Sim. Paralelismo, incremental e cache são os principais. |
| **Timeout ideal?** | 120 s cobre clawd inteiro. Para repos > 60k arquivos, 180–300 s. |
