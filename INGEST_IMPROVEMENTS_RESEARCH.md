# m1nd Ingest — Melhorias Baseadas em Pesquisa (2025-03-15)

**Fontes:** CocoIndex, Codeix, Sourcegraph SCIP, Cursor, Glean, CodeIntensor, tree-sitter, mmap.

---

## 1. Incremental Ingest (maior ganho)

### O que fazem

| Ferramenta | Abordagem |
|------------|-----------|
| **Glean** | "Units" (arquivos) — exclui mudados do base, empilha novos |
| **CodeIntensor** | `git diff` → só reindexa arquivos alterados |
| **Cursor** | Merkle tree — compara hashes, só processa ramos diferentes |
| **CocoIndex** | Data lineage em Postgres, reprocessa só mudados |
| **kit** | mtime + file size + SHA-256 — cache por hash |

### Ganho típico

- **25–36×** mais rápido em warm cache (kit)
- Se 1 de 1000 arquivos muda → analisa 1, usa cache para 999

### Implementação sugerida

```
1. Persistir hash (ou mtime+size) por arquivo no último ingest
2. Na próxima sessão: walk → para cada arquivo, comparar hash
3. Só extrair arquivos com hash diferente
4. Merge: remover nós de arquivos deletados, adicionar/atualizar mudados
```

**Complexidade:** Alta. Precisa de `ingest_incremental` real (hoje é stub).

---

## 2. mmap para leitura de arquivos

### O que fazem

- **memmap2** (Rust): mapeia arquivo em memória, acesso zero-copy
- **memchr**: busca SIMD sobre o slice mapeado
- **Benefício:** I/O mais rápido, menos alocações

### Implementação

```rust
// Hoje
let content = std::fs::read(&file.path).ok()?;  // aloca Vec<u8>

// Com mmap (para arquivos > N KB)
use memmap2::Mmap;
let file = std::fs::File::open(&file.path)?;
let mmap = unsafe { Mmap::map(&file)? };
let content: &[u8] = &mmap;  // zero-copy, sem alocação
```

**Cuidado:** tree-sitter e extractors esperam `&[u8]` ou `&str`. mmap retorna `&[u8]` — compatível. Para `&str` precisa de `from_utf8` (validação).

**Complexidade:** Média. Adicionar memmap2, usar para arquivos > 64KB.

---

## 3. Minimizar alocações (estilo SCIP)

### O que Sourcegraph fez

- **Symbol validation:** writer que descarta writes → ~99.96% menos alocações
- **Symbol parse:** 59% menos alocações
- **Resultado:** 413ns/op vs 928ns/op

### Aplicável ao m1nd

- Evitar `String::from` / `to_string` em hot paths
- Usar `Cow<str>` ou `&str` quando possível
- Reusar buffers em vez de alocar por arquivo
- `format!` é caro — preferir `write!` em buffer reutilizável

**Complexidade:** Média. Requer profiling para achar hotspots.

---

## 4. Git para change detection (simples)

### Abordagem

```bash
git diff --name-only HEAD~1  # ou desde último ingest
# ou
git ls-files -m  # modified
git ls-files -d  # deleted
```

- Só extrair arquivos em `git diff --name-only`
- Para merge: remover nós de arquivos deletados, re-extrair modificados
- Base: grafo do ingest anterior (persistido)

**Complexidade:** Média. Precisa de `git2` ou `std::process::Command` para git.

---

## 5. Cache de extração por hash

### Abordagem (kit)

1. Por arquivo: hash(content) ou (mtime, size)
2. Cache: `HashMap<path, (hash, ExtractionResult)>` persistido
3. Na ingest: walk → se hash igual, usar cache; senão extrair
4. Persistir cache em disco (JSON, SQLite, ou formato binário)

**Complexidade:** Média. Formato de cache e invalidação.

---

## 6. Streaming / lazy graph build

### O que Sourcegraph fez

- Parser streaming para SCIP — reduz pico de memória
- Processa em chunks em vez de carregar tudo

### Aplicável

- Em vez de `Vec::extend` de todos os nós e depois `graph.add_node` em loop
- Processar em batches (ex.: 1000 arquivos por batch, finalize parcial)
- Ou: pipeline streaming — cada arquivo extraído vai direto para o graph (com cuidado com concorrência)

**Complexidade:** Alta. O graph build hoje é single-threaded por design.

---

## 7. Skip arquivos muito grandes

### Abordagem

- Arquivos > 1MB raramente precisam de parse completo
- Opção: `skip_files` por tamanho, ou truncar para primeiros N KB
- Reduz I/O e CPU em monorepos com assets/bundles

**Complexidade:** Baixa. Um `if metadata.len() > 1_000_000 { skip }`.

---

## 8. Resumo priorizado

| Melhoria | Ganho estimado | Esforço | Prioridade |
|----------|----------------|---------|------------|
| **Incremental (git diff)** | 10–30× em re-ingest | Alto | 1 |
| **Cache por hash** | 10–25× em warm | Médio | 2 |
| **mmap para arquivos grandes** | 10–30% I/O | Médio | 3 |
| **Skip files > 1MB** | Variável | Baixo | 4 |
| **Minimizar alocações** | 5–15% | Médio | 5 |
| **Streaming graph build** | Memória | Alto | 6 |

---

## 9. Quick Win: Skip arquivos > 1MB

Adicionar `max_file_size_bytes` ao config e ao walker. Arquivos gigantes (bundles, generated) raramente precisam de parse completo.

```rust
// walker: if metadata.len() > max_file_size { continue; }
// default: 1_048_576 (1MB)
```

**Complexidade:** Baixa. ~5 linhas.

---

## 10. Referências

- [CocoIndex](https://cocoindex.io/) — Rust, tree-sitter, incremental
- [Codeix](https://codeix.dev/) — .codeindex, SQLite FTS5
- [SCIP PR #258](https://github.com/sourcegraph/scip/pull/258) — allocation minimization
- [Glean incremental](https://glean.software/blog/incremental/)
- [Cursor indexing](https://cursor.com/blog/secure-codebase-indexing) — Merkle trees
- [memmap2](https://docs.rs/memmap2) — mmap em Rust
