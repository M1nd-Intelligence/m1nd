// === crates/m1nd-mcp/src/tools.rs ===

use m1nd_core::error::M1ndResult;
use m1nd_core::temporal::ImpactDirection;
use m1nd_core::types::*;
use m1nd_core::query::QueryConfig;
use crate::brand;
use crate::session::SessionState;
use crate::protocol::*;
use std::time::Instant;

// ---------------------------------------------------------------------------
// Tool handlers — one per MCP tool (03-MCP Section 2)
// Each handler: parse input -> call engine -> format output.
// All handlers take &mut SessionState for graph + engine access.
// ---------------------------------------------------------------------------

fn normalized_ingest_mode(mode: &str) -> &str {
    if mode.eq_ignore_ascii_case("merge") {
        "merge"
    } else {
        "replace"
    }
}

fn finalize_ingest(
    state: &mut SessionState,
    input: &IngestInput,
    adapter: &str,
    new_graph: m1nd_core::graph::Graph,
    stats: m1nd_ingest::IngestStats,
) -> M1ndResult<serde_json::Value> {
    let mode = normalized_ingest_mode(&input.mode).to_string();
    let namespace = input.namespace.clone().or_else(|| {
        if adapter == "memory" {
            Some("memory".to_string())
        } else {
            None
        }
    });

    let combined_graph = if mode == "merge" {
        let current = state.graph.read();
        if current.num_nodes() > 0 {
            m1nd_ingest::merge::merge_graphs(&current, &new_graph)?
        } else {
            new_graph
        }
    } else {
        new_graph
    };

    {
        let mut graph = state.graph.write();
        *graph = combined_graph;
        if !graph.finalized {
            graph.finalize()?;
        }
    }

    state.rebuild_engines()?;

    // Track ingest roots for L3 git discovery
    if !state.ingest_roots.contains(&input.path) {
        state.ingest_roots.push(input.path.clone());
    }

    if let Err(e) = state.persist() {
        eprintln!("{}", brand::log(&format!("auto-persist after ingest failed: {}", e)));
    }

    let (node_count, edge_count) = {
        let graph = state.graph.read();
        (graph.num_nodes(), graph.num_edges())
    };

    Ok(serde_json::json!({
        "mode": mode,
        "adapter": adapter,
        "namespace": namespace,
        "files_scanned": stats.files_scanned,
        "files_parsed": stats.files_parsed,
        "nodes_created": stats.nodes_created,
        "edges_created": stats.edges_created,
        "elapsed_ms": stats.elapsed_ms,
        "node_count": node_count,
        "edge_count": edge_count,
    }))
}

/// Handle m1nd.activate (03-MCP Section 2.1).
/// Replaces: ConnectomeEngine.query() + AdaptiveXLREngine.query() + PlasticityEngine.query()
pub fn handle_activate(
    state: &mut SessionState,
    input: ActivateInput,
) -> M1ndResult<ActivateOutput> {
    let start = Instant::now();

    let dimensions: Vec<Dimension> = input
        .dimensions
        .iter()
        .filter_map(|d| match d.as_str() {
            "structural" => Some(Dimension::Structural),
            "semantic" => Some(Dimension::Semantic),
            "temporal" => Some(Dimension::Temporal),
            "causal" => Some(Dimension::Causal),
            _ => None,
        })
        .collect();

    let config = QueryConfig {
        query: input.query.clone(),
        agent_id: input.agent_id.clone(),
        top_k: input.top_k,
        dimensions: if dimensions.is_empty() {
            vec![
                Dimension::Structural,
                Dimension::Semantic,
                Dimension::Temporal,
                Dimension::Causal,
            ]
        } else {
            dimensions
        },
        xlr_enabled: input.xlr,
        include_ghost_edges: input.include_ghost_edges,
        include_structural_holes: input.include_structural_holes,
        propagation: PropagationConfig::default(),
    };

    let result = {
        let mut graph = state.graph.write();
        state.orchestrator.query(&mut graph, &config)?
    };

    state.queries_processed += 1;
    if state.should_persist() {
        let _ = state.persist();
    }

    let graph = state.graph.read();

    // Map seeds
    let seeds: Vec<SeedOutput> = result
        .activation
        .seeds
        .iter()
        .map(|&(node, relevance)| {
            let idx = node.as_usize();
            let label = if idx < graph.num_nodes() as usize {
                graph.strings.resolve(graph.nodes.label[idx]).to_string()
            } else {
                format!("node_{}", idx)
            };
            SeedOutput {
                node_id: label.clone(),
                label,
                relevance: relevance.get(),
            }
        })
        .collect();

    // Build reverse lookup: NodeId -> external ID string
    let mut node_to_ext: Vec<String> = vec![String::new(); graph.num_nodes() as usize];
    for (interned, &nid) in &graph.id_to_node {
        let idx = nid.as_usize();
        if idx < node_to_ext.len() {
            node_to_ext[idx] = graph.strings.resolve(*interned).to_string();
        }
    }

    // Map activated nodes
    let activated: Vec<ActivatedNodeOutput> = result
        .activation
        .activated
        .iter()
        .map(|a| {
            let idx = a.node.as_usize();
            let (ext_id, label, node_type, tags, provenance) = if idx < graph.num_nodes() as usize {
                let eid = &node_to_ext[idx];
                let l = graph.strings.resolve(graph.nodes.label[idx]).to_string();
                let t = format!("{:?}", graph.nodes.node_type[idx]);
                let tg: Vec<String> = graph.nodes.tags[idx]
                    .iter()
                    .map(|&ti| graph.strings.resolve(ti).to_string())
                    .collect();
                let provenance = graph.resolve_node_provenance(a.node);
                let provenance = if provenance.is_empty() {
                    None
                } else {
                    Some(ProvenanceOutput {
                        source_path: provenance.source_path,
                        line_start: provenance.line_start,
                        line_end: provenance.line_end,
                        excerpt: provenance.excerpt,
                        namespace: provenance.namespace,
                        canonical: provenance.canonical,
                    })
                };
                (eid.clone(), l, t, tg, provenance)
            } else {
                (
                    format!("node_{}", idx),
                    format!("node_{}", idx),
                    "Unknown".into(),
                    vec![],
                    None,
                )
            };
            ActivatedNodeOutput {
                node_id: ext_id,
                label,
                node_type,
                activation: a.activation.get(),
                dimensions: DimensionsOutput {
                    structural: a.dimensions[0].get(),
                    semantic: a.dimensions[1].get(),
                    temporal: a.dimensions[2].get(),
                    causal: a.dimensions[3].get(),
                },
                pagerank: if idx < graph.nodes.pagerank.len() {
                    graph.nodes.pagerank[idx].get()
                } else {
                    0.0
                },
                tags,
                provenance,
            }
        })
        .collect();

    // Map ghost edges
    let ghost_edges: Vec<GhostEdgeOutput> = result
        .ghost_edges
        .iter()
        .map(|ge| {
            let src_idx = ge.source.as_usize();
            let tgt_idx = ge.target.as_usize();
            let src = if src_idx < graph.num_nodes() as usize {
                graph.strings.resolve(graph.nodes.label[src_idx]).to_string()
            } else {
                format!("node_{}", src_idx)
            };
            let tgt = if tgt_idx < graph.num_nodes() as usize {
                graph.strings.resolve(graph.nodes.label[tgt_idx]).to_string()
            } else {
                format!("node_{}", tgt_idx)
            };
            GhostEdgeOutput {
                source: src,
                target: tgt,
                shared_dimensions: ge
                    .shared_dimensions
                    .iter()
                    .map(|d| format!("{:?}", d).to_lowercase())
                    .collect(),
                strength: ge.strength.get(),
            }
        })
        .collect();

    // Map structural holes
    let structural_holes: Vec<StructuralHoleOutput> = result
        .structural_holes
        .iter()
        .map(|sh| {
            let idx = sh.node.as_usize();
            let (label, node_type) = if idx < graph.num_nodes() as usize {
                (
                    graph.strings.resolve(graph.nodes.label[idx]).to_string(),
                    format!("{:?}", graph.nodes.node_type[idx]),
                )
            } else {
                (format!("node_{}", idx), "Unknown".into())
            };
            StructuralHoleOutput {
                node_id: label.clone(),
                label,
                node_type,
                reason: sh.reason.clone(),
            }
        })
        .collect();

    let plasticity = PlasticityOutput {
        edges_strengthened: result.plasticity.edges_strengthened,
        edges_decayed: result.plasticity.edges_decayed,
        ltp_events: result.plasticity.ltp_events,
        priming_nodes: result.plasticity.priming_nodes,
    };

    let elapsed_ms = start.elapsed().as_secs_f64() * 1000.0;

    Ok(ActivateOutput {
        query: input.query,
        seeds,
        activated,
        ghost_edges,
        structural_holes,
        plasticity,
        elapsed_ms,
    })
}

/// Handle m1nd.impact (03-MCP Section 2.2).
/// Replaces: ImpactRadiusCalculator.compute() + CausalChainDetector.detect()
pub fn handle_impact(
    state: &mut SessionState,
    input: ImpactInput,
) -> M1ndResult<ImpactOutput> {
    let graph = state.graph.read();

    let node_id = graph.resolve_id(&input.node_id);
    let node = match node_id {
        Some(n) => n,
        None => {
            return Ok(ImpactOutput {
                source: input.node_id.clone(),
                source_label: input.node_id,
                direction: input.direction.clone(),
                blast_radius: vec![],
                total_energy: 0.0,
                max_hops_reached: 0,
                causal_chains: vec![],
            });
        }
    };

    let direction = match input.direction.as_str() {
        "reverse" => ImpactDirection::Reverse,
        "both" => ImpactDirection::Both,
        _ => ImpactDirection::Forward,
    };

    let impact = state.temporal.impact_calculator.compute(&graph, node, direction)?;

    // Causal chains
    let chains = if input.include_causal_chains {
        state.temporal.chain_detector.detect(&graph, node)?
    } else {
        vec![]
    };

    let source_label = {
        let idx = node.as_usize();
        if idx < graph.num_nodes() as usize {
            graph.strings.resolve(graph.nodes.label[idx]).to_string()
        } else {
            input.node_id.clone()
        }
    };

    let blast_radius: Vec<BlastRadiusEntry> = impact
        .blast_radius
        .iter()
        .map(|e| {
            let idx = e.node.as_usize();
            let (label, node_type) = if idx < graph.num_nodes() as usize {
                (
                    graph.strings.resolve(graph.nodes.label[idx]).to_string(),
                    format!("{:?}", graph.nodes.node_type[idx]),
                )
            } else {
                (format!("node_{}", idx), "Unknown".into())
            };
            BlastRadiusEntry {
                node_id: label.clone(),
                label,
                node_type,
                signal_strength: e.signal_strength.get(),
                hop_distance: e.hop_distance,
            }
        })
        .collect();

    let causal_chains: Vec<CausalChainOutput> = chains
        .iter()
        .map(|c| {
            let path: Vec<String> = c
                .path
                .iter()
                .map(|&n| {
                    let idx = n.as_usize();
                    if idx < graph.num_nodes() as usize {
                        graph.strings.resolve(graph.nodes.label[idx]).to_string()
                    } else {
                        format!("node_{}", idx)
                    }
                })
                .collect();
            let relations: Vec<String> = c
                .relations
                .iter()
                .map(|&r| graph.strings.resolve(r).to_string())
                .collect();
            CausalChainOutput {
                path,
                relations,
                cumulative_strength: c.cumulative_strength.get(),
            }
        })
        .collect();

    Ok(ImpactOutput {
        source: input.node_id,
        source_label,
        direction: input.direction,
        blast_radius,
        total_energy: impact.total_energy.get(),
        max_hops_reached: impact.max_hops_reached,
        causal_chains,
    })
}

/// Handle m1nd.missing (03-MCP Section 2.3).
/// Replaces: ConnectomeEngine.query() + StructuralHoleDetector.detect()
pub fn handle_missing(
    state: &mut SessionState,
    input: MissingInput,
) -> M1ndResult<serde_json::Value> {
    let config = QueryConfig {
        query: input.query.clone(),
        agent_id: input.agent_id.clone(),
        top_k: 20,
        xlr_enabled: true,
        include_ghost_edges: false,
        include_structural_holes: true,
        ..QueryConfig::default()
    };

    let result = {
        let mut graph = state.graph.write();
        state.orchestrator.query(&mut graph, &config)?
    };

    let graph = state.graph.read();

    let holes: Vec<serde_json::Value> = result
        .structural_holes
        .iter()
        .map(|sh| {
            let idx = sh.node.as_usize();
            let label = if idx < graph.num_nodes() as usize {
                graph.strings.resolve(graph.nodes.label[idx]).to_string()
            } else {
                format!("node_{}", idx)
            };
            serde_json::json!({
                "node_id": label,
                "sibling_avg_activation": sh.sibling_avg_activation.get(),
                "reason": sh.reason,
            })
        })
        .collect();

    Ok(serde_json::json!({
        "query": input.query,
        "structural_holes": holes,
        "ghost_edges": result.ghost_edges.len(),
    }))
}

/// Handle m1nd.why (03-MCP Section 2.4).
/// Replaces: bidirectional BFS + DimensionResult.paths + CommunityDetector
pub fn handle_why(
    state: &mut SessionState,
    input: WhyInput,
) -> M1ndResult<serde_json::Value> {
    let graph = state.graph.read();

    let source = graph.resolve_id(&input.source);
    let target = graph.resolve_id(&input.target);

    let (source_node, target_node) = match (source, target) {
        (Some(s), Some(t)) => (s, t),
        _ => {
            return Ok(serde_json::json!({
                "source": input.source,
                "target": input.target,
                "paths": [],
                "reason": "One or both nodes not found",
            }));
        }
    };

    // BFS from source to target (max_hops)
    let n = graph.num_nodes() as usize;
    let max_hops = input.max_hops as usize;
    let mut parent: Vec<Option<(usize, usize)>> = vec![None; n]; // (prev_node, edge_idx)
    let mut visited = vec![false; n];
    let mut queue = std::collections::VecDeque::new();

    visited[source_node.as_usize()] = true;
    queue.push_back((source_node, 0usize));

    let mut found = false;
    while let Some((node, depth)) = queue.pop_front() {
        if node == target_node {
            found = true;
            break;
        }
        if depth >= max_hops {
            continue;
        }
        // Forward edges
        let range = graph.csr.out_range(node);
        for j in range {
            let tgt = graph.csr.targets[j];
            let tgt_idx = tgt.as_usize();
            if tgt_idx < n && !visited[tgt_idx] {
                visited[tgt_idx] = true;
                parent[tgt_idx] = Some((node.as_usize(), j));
                queue.push_back((tgt, depth + 1));
            }
        }
        // Reverse edges (traverse incoming edges for full bidirectional BFS)
        let rev_range = graph.csr.in_range(node);
        for j in rev_range {
            let src = graph.csr.rev_sources[j];
            let src_idx = src.as_usize();
            let fwd_edge = graph.csr.rev_edge_idx[j].as_usize();
            if src_idx < n && !visited[src_idx] {
                visited[src_idx] = true;
                parent[src_idx] = Some((node.as_usize(), fwd_edge));
                queue.push_back((src, depth + 1));
            }
        }
    }

    let mut paths = Vec::new();
    if found {
        // Reconstruct path
        let mut path_nodes = vec![target_node.as_usize()];
        let mut path_relations = Vec::new();
        let mut current = target_node.as_usize();
        while let Some((prev, edge_j)) = parent[current] {
            path_nodes.push(prev);
            let rel = graph.strings.resolve(graph.csr.relations[edge_j]).to_string();
            path_relations.push(rel);
            current = prev;
            if current == source_node.as_usize() {
                break;
            }
        }
        path_nodes.reverse();
        path_relations.reverse();

        let path_labels: Vec<String> = path_nodes
            .iter()
            .map(|&i| {
                if i < graph.num_nodes() as usize {
                    graph.strings.resolve(graph.nodes.label[i]).to_string()
                } else {
                    format!("node_{}", i)
                }
            })
            .collect();

        paths.push(serde_json::json!({
            "nodes": path_labels,
            "relations": path_relations,
            "hops": path_labels.len() - 1,
        }));
    }

    // Check community membership
    let same_community = {
        let communities = state.topology.community_detector.detect(&graph);
        match communities {
            Ok(c) => {
                let s = source_node.as_usize();
                let t = target_node.as_usize();
                if s < c.assignments.len() && t < c.assignments.len() {
                    c.assignments[s] == c.assignments[t]
                } else {
                    false
                }
            }
            Err(_) => false,
        }
    };

    Ok(serde_json::json!({
        "source": input.source,
        "target": input.target,
        "paths": paths,
        "same_community": same_community,
        "found": found,
    }))
}

/// Handle m1nd.warmup (03-MCP Section 2.5).
/// Replaces: SeedFinder.find_seeds() + QueryMemory.get_priming_signal()
pub fn handle_warmup(
    state: &mut SessionState,
    input: WarmupInput,
) -> M1ndResult<serde_json::Value> {
    let graph = state.graph.read();

    // Find seeds related to the task description
    let seeds = m1nd_core::seed::SeedFinder::find_seeds(
        &graph,
        &input.task_description,
        50,
    )?;

    let seed_nodes: Vec<NodeId> = seeds.iter().map(|s| s.0).collect();

    // Get priming signal from plasticity memory
    let priming = state.plasticity.get_priming(
        &seed_nodes,
        FiniteF32::new(input.boost_strength),
    );

    let seed_output: Vec<serde_json::Value> = seeds
        .iter()
        .take(20)
        .map(|&(node, relevance)| {
            let idx = node.as_usize();
            let label = if idx < graph.num_nodes() as usize {
                graph.strings.resolve(graph.nodes.label[idx]).to_string()
            } else {
                format!("node_{}", idx)
            };
            serde_json::json!({
                "node_id": label,
                "relevance": relevance.get(),
            })
        })
        .collect();

    let priming_output: Vec<serde_json::Value> = priming
        .iter()
        .take(20)
        .map(|&(node, strength)| {
            let idx = node.as_usize();
            let label = if idx < graph.num_nodes() as usize {
                graph.strings.resolve(graph.nodes.label[idx]).to_string()
            } else {
                format!("node_{}", idx)
            };
            serde_json::json!({
                "node_id": label,
                "priming_strength": strength.get(),
            })
        })
        .collect();

    Ok(serde_json::json!({
        "task_description": input.task_description,
        "seeds": seed_output,
        "priming_nodes": priming_output,
        "total_seeds": seeds.len(),
        "total_priming": priming.len(),
    }))
}

/// Handle m1nd.counterfactual (03-MCP Section 2.6).
/// Replaces: NodeRemovalSimulator + CascadeAnalyzer + WhatIfSimulator
pub fn handle_counterfactual(
    state: &mut SessionState,
    input: CounterfactualInput,
) -> M1ndResult<serde_json::Value> {
    let graph = state.graph.read();

    let remove_nodes: Vec<NodeId> = input
        .node_ids
        .iter()
        .filter_map(|id| graph.resolve_id(id))
        .collect();

    if remove_nodes.is_empty() {
        return Ok(serde_json::json!({
            "error": "No valid node IDs found",
            "node_ids": input.node_ids,
        }));
    }

    let config = PropagationConfig::default();

    // Combined removal (all nodes at once)
    let result = state.counterfactual.simulate_removal(
        &graph,
        &state.orchestrator.engine,
        &config,
        &remove_nodes,
    )?;

    // Cascade analysis for first node
    let cascade = if input.include_cascade && !remove_nodes.is_empty() {
        let c = state.counterfactual.cascade_analysis(
            &graph,
            &state.orchestrator.engine,
            &config,
            remove_nodes[0],
        )?;
        Some(serde_json::json!({
            "cascade_depth": c.cascade_depth,
            "total_affected": c.total_affected,
            "affected_by_depth": c.affected_by_depth.iter().map(|d| d.len()).collect::<Vec<_>>(),
        }))
    } else {
        None
    };

    // Synergy analysis: only when multiple nodes are removed.
    // Compares combined impact vs sum of individual impacts.
    //   synergy_factor > 1.0 → synergistic (together worse than sum of parts)
    //   synergy_factor < 1.0 → redundant (together less bad than sum of parts)
    //   synergy_factor ≈ 1.0 → independent
    let synergy = if remove_nodes.len() > 1 {
        let mut individual_impacts: Vec<serde_json::Value> = Vec::new();
        let mut sum_individual: f32 = 0.0;

        for &node in &remove_nodes {
            let individual = state.counterfactual.simulate_removal(
                &graph,
                &state.orchestrator.engine,
                &config,
                &[node],
            )?;
            let pct_lost = individual.pct_activation_lost.get();
            sum_individual += pct_lost;

            let idx = node.as_usize();
            let label = if idx < graph.num_nodes() as usize {
                graph.strings.resolve(graph.nodes.label[idx]).to_string()
            } else {
                format!("node_{}", idx)
            };
            individual_impacts.push(serde_json::json!({
                "node_id": label,
                "pct_activation_lost": pct_lost,
            }));
        }

        let combined_impact = result.pct_activation_lost.get();
        let synergy_factor = if sum_individual > 0.0 {
            combined_impact / sum_individual
        } else {
            1.0
        };

        Some(serde_json::json!({
            "individual_impacts": individual_impacts,
            "combined_impact": combined_impact,
            "synergy_factor": synergy_factor,
        }))
    } else {
        None
    };

    Ok(serde_json::json!({
        "removed_nodes": input.node_ids,
        "total_impact": result.total_impact.get(),
        "pct_activation_lost": result.pct_activation_lost.get(),
        "orphaned_count": result.orphaned_nodes.len(),
        "weakened_count": result.weakened_nodes.len(),
        "reachability_before": result.reachability_before,
        "reachability_after": result.reachability_after,
        "cascade": cascade,
        "synergy": synergy,
    }))
}

/// Handle m1nd.predict (03-MCP Section 2.7).
/// Replaces: CoChangeMatrix.predict() + VelocityScorer.score()
pub fn handle_predict(
    state: &mut SessionState,
    input: PredictInput,
) -> M1ndResult<serde_json::Value> {
    let graph = state.graph.read();

    let node = match graph.resolve_id(&input.changed_node) {
        Some(n) => n,
        None => {
            return Ok(serde_json::json!({
                "error": "Node not found",
                "changed_node": input.changed_node,
            }));
        }
    };

    let co_change_predictions = state.temporal.co_change.predict(node, input.top_k);

    // --- Structural fallback (Issue 3) ---
    // If co-change returns fewer than top_k results, supplement with
    // structural predictions: nodes connected via imports/calls/references
    // edges, scored by edge weight.  Co-change results rank higher.
    let mut seen: std::collections::HashSet<NodeId> = co_change_predictions
        .iter()
        .map(|p| p.target)
        .collect();

    let mut structural_predictions: Vec<m1nd_core::temporal::CoChangeEntry> = Vec::new();

    if co_change_predictions.len() < input.top_k {
        let structural_relations: Vec<&str> = vec!["imports", "calls", "references"];
        let structural_interned: Vec<InternedStr> = structural_relations
            .iter()
            .filter_map(|r| {
                // Only match if the string is already interned (don't create it)
                graph.strings.lookup(r)
            })
            .collect();

        let range = graph.csr.out_range(node);
        for k in range {
            let target = graph.csr.targets[k];
            if target == node || seen.contains(&target) {
                continue;
            }
            let rel = graph.csr.relations[k];
            if structural_interned.contains(&rel) {
                let weight = graph.csr.read_weight(EdgeIdx::new(k as u32));
                structural_predictions.push(m1nd_core::temporal::CoChangeEntry {
                    target,
                    strength: weight,
                });
                seen.insert(target);
            }
        }

        // Also check incoming edges (reverse CSR) — if X imports this node,
        // X is likely impacted by changes here.
        let rev_range = graph.csr.in_range(node);
        for k in rev_range {
            let source = graph.csr.rev_sources[k];
            if source == node || seen.contains(&source) {
                continue;
            }
            let fwd_idx = graph.csr.rev_edge_idx[k];
            let rel = graph.csr.relations[fwd_idx.as_usize()];
            if structural_interned.contains(&rel) {
                let weight = graph.csr.read_weight(fwd_idx);
                structural_predictions.push(m1nd_core::temporal::CoChangeEntry {
                    target: source,
                    strength: weight,
                });
                seen.insert(source);
            }
        }

        // Sort structural by weight descending
        structural_predictions.sort_by(|a, b| b.strength.cmp(&a.strength));
    }

    // Merge: co-change first (higher priority), then structural fill
    let remaining = input.top_k.saturating_sub(co_change_predictions.len());
    structural_predictions.truncate(remaining);

    let all_predictions: Vec<&m1nd_core::temporal::CoChangeEntry> = co_change_predictions
        .iter()
        .chain(structural_predictions.iter())
        .collect();

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs_f64())
        .unwrap_or(0.0);

    let velocity = if input.include_velocity {
        let v = m1nd_core::temporal::VelocityScorer::score_one(&graph, node, now)?;
        Some(serde_json::json!({
            "velocity": v.velocity.get(),
            "trend": format!("{:?}", v.trend),
        }))
    } else {
        None
    };

    let prediction_output: Vec<serde_json::Value> = all_predictions
        .iter()
        .map(|p| {
            let idx = p.target.as_usize();
            let label = if idx < graph.num_nodes() as usize {
                graph.strings.resolve(graph.nodes.label[idx]).to_string()
            } else {
                format!("node_{}", idx)
            };
            serde_json::json!({
                "node_id": label,
                "coupling_strength": p.strength.get(),
            })
        })
        .collect();

    Ok(serde_json::json!({
        "changed_node": input.changed_node,
        "predictions": prediction_output,
        "co_change_count": co_change_predictions.len(),
        "structural_fallback_count": structural_predictions.len(),
        "velocity": velocity,
    }))
}

/// Handle m1nd.fingerprint (03-MCP Section 2.8).
/// Replaces: ActivationFingerprinter.compute_fingerprints() + find_equivalents()
pub fn handle_fingerprint(
    state: &mut SessionState,
    input: FingerprintInput,
) -> M1ndResult<serde_json::Value> {
    let graph = state.graph.read();

    // Generate probe queries from probe_queries or use defaults
    let probe_seeds: Vec<Vec<(NodeId, FiniteF32)>> = match &input.probe_queries {
        Some(queries) => queries
            .iter()
            .filter_map(|q| {
                let seeds = m1nd_core::seed::SeedFinder::find_seeds(&graph, q, 5).ok()?;
                if seeds.is_empty() {
                    None
                } else {
                    Some(seeds)
                }
            })
            .collect(),
        None => {
            // Default: use a few deterministic probes
            let n = graph.num_nodes();
            (0..5.min(n))
                .map(|i| vec![(NodeId::new(i), FiniteF32::ONE)])
                .collect()
        }
    };

    if probe_seeds.is_empty() {
        return Ok(serde_json::json!({
            "error": "No valid probe queries could be resolved",
        }));
    }

    let fingerprints = state.topology.fingerprinter.compute_fingerprints(
        &graph,
        &state.orchestrator.engine,
        &probe_seeds,
    )?;

    let result = if let Some(ref target_id) = input.target_node {
        // Find equivalents of a specific node
        match graph.resolve_id(target_id) {
            Some(target) => {
                let pairs = state.topology.fingerprinter.find_equivalents_of(
                    target,
                    &fingerprints,
                    &graph,
                )?;
                let equivalents: Vec<serde_json::Value> = pairs
                    .iter()
                    .map(|p| {
                        let idx_b = p.node_b.as_usize();
                        let label = if idx_b < graph.num_nodes() as usize {
                            graph.strings.resolve(graph.nodes.label[idx_b]).to_string()
                        } else {
                            format!("node_{}", idx_b)
                        };
                        serde_json::json!({
                            "node_id": label,
                            "cosine_similarity": p.cosine_similarity.get(),
                            "directly_connected": p.directly_connected,
                        })
                    })
                    .collect();
                serde_json::json!({
                    "target_node": target_id,
                    "equivalents": equivalents,
                })
            }
            None => serde_json::json!({
                "error": "Target node not found",
                "target_node": target_id,
            }),
        }
    } else {
        // Find all equivalent pairs
        let pairs = state
            .topology
            .fingerprinter
            .find_equivalents(&fingerprints, &graph)?;
        let output: Vec<serde_json::Value> = pairs
            .iter()
            .take(20)
            .map(|p| {
                let idx_a = p.node_a.as_usize();
                let idx_b = p.node_b.as_usize();
                let label_a = if idx_a < graph.num_nodes() as usize {
                    graph.strings.resolve(graph.nodes.label[idx_a]).to_string()
                } else {
                    format!("node_{}", idx_a)
                };
                let label_b = if idx_b < graph.num_nodes() as usize {
                    graph.strings.resolve(graph.nodes.label[idx_b]).to_string()
                } else {
                    format!("node_{}", idx_b)
                };
                serde_json::json!({
                    "node_a": label_a,
                    "node_b": label_b,
                    "cosine_similarity": p.cosine_similarity.get(),
                    "directly_connected": p.directly_connected,
                })
            })
            .collect();
        serde_json::json!({
            "equivalent_pairs": output,
            "total_pairs": pairs.len(),
        })
    };

    Ok(result)
}

/// Handle m1nd.drift (03-MCP Section 2.9).
/// Replaces: PlasticityEngine state diff + CommunityDetector + VelocityScorer
pub fn handle_drift(
    state: &mut SessionState,
    input: DriftInput,
) -> M1ndResult<serde_json::Value> {
    let graph = state.graph.read();

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs_f64())
        .unwrap_or(0.0);

    // Weight drift: find edges whose current weight differs most from baseline.
    // If since == "last_session" and a plasticity state file exists, use it as baseline.
    // Otherwise, fall back to original_weight comparison.
    let weight_drift = if input.include_weight_drift {
        // Try to load saved plasticity state as baseline
        let baseline_map: Option<std::collections::HashMap<(String, String, String), f32>> =
            if input.since == "last_session" {
                let state_path = std::path::Path::new("plasticity_state.json");
                match m1nd_core::snapshot::load_plasticity_state(state_path) {
                    Ok(states) => {
                        let mut map = std::collections::HashMap::new();
                        for s in &states {
                            map.insert(
                                (s.source_label.clone(), s.target_label.clone(), s.relation.clone()),
                                s.current_weight,
                            );
                        }
                        Some(map)
                    }
                    Err(_) => None, // file missing or corrupt — fall back
                }
            } else {
                None
            };

        let num_edges = graph.edge_plasticity.original_weight.len();
        let num_nodes = graph.num_nodes() as usize;

        // Build edge_source map: edge_idx → source node index (from CSR offsets)
        let num_csr = graph.csr.num_edges();
        let mut edge_source = vec![0usize; num_csr];
        for i in 0..num_nodes {
            let lo = graph.csr.offsets[i] as usize;
            let hi = graph.csr.offsets[i + 1] as usize;
            for j in lo..hi {
                edge_source[j] = i;
            }
        }

        // Build node_ext_id: NodeId → external id string
        let mut node_ext_id = vec![String::new(); num_nodes];
        for (&interned, &node_id) in &graph.id_to_node {
            if node_id.as_usize() < num_nodes {
                node_ext_id[node_id.as_usize()] = graph.strings.resolve(interned).to_string();
            }
        }

        let cap = num_edges.min(num_csr);
        let mut drifts: Vec<(usize, f32, f32, f32)> = (0..cap)
            .filter_map(|j| {
                let curr = graph.edge_plasticity.current_weight[j].get();

                let baseline_weight = if let Some(ref bmap) = baseline_map {
                    let src_idx = edge_source[j];
                    let tgt_idx = graph.csr.targets[j].as_usize();
                    let src_label = if src_idx < num_nodes { &node_ext_id[src_idx] } else { return None; };
                    let tgt_label = if tgt_idx < num_nodes { &node_ext_id[tgt_idx] } else { return None; };
                    let rel = graph.strings.try_resolve(graph.csr.relations[j])
                        .unwrap_or("edge")
                        .to_string();
                    let key = (src_label.clone(), tgt_label.clone(), rel);
                    *bmap.get(&key).unwrap_or(&graph.edge_plasticity.original_weight[j].get())
                } else {
                    graph.edge_plasticity.original_weight[j].get()
                };

                let delta = (curr - baseline_weight).abs();
                if delta > 0.001 {
                    Some((j, delta, baseline_weight, curr))
                } else {
                    None
                }
            })
            .collect();
        drifts.sort_by(|a, b| b.1.total_cmp(&a.1));
        drifts.truncate(20);

        let drift_output: Vec<serde_json::Value> = drifts
            .iter()
            .map(|&(j, delta, baseline, curr)| {
                serde_json::json!({
                    "edge_idx": j,
                    "baseline_weight": baseline,
                    "current_weight": curr,
                    "delta": delta,
                })
            })
            .collect();
        Some(drift_output)
    } else {
        None
    };

    // Velocity analysis
    let velocities = m1nd_core::temporal::VelocityScorer::score_all(&graph, now)?;
    let top_velocities: Vec<serde_json::Value> = velocities
        .iter()
        .take(10)
        .map(|v| {
            let idx = v.node.as_usize();
            let label = if idx < graph.num_nodes() as usize {
                graph.strings.resolve(graph.nodes.label[idx]).to_string()
            } else {
                format!("node_{}", idx)
            };
            serde_json::json!({
                "node_id": label,
                "velocity": v.velocity.get(),
                "trend": format!("{:?}", v.trend),
            })
        })
        .collect();

    Ok(serde_json::json!({
        "since": input.since,
        "queries_processed": state.queries_processed,
        "weight_drift": weight_drift,
        "top_velocities": top_velocities,
        "uptime_seconds": state.uptime_seconds(),
    }))
}

/// Handle m1nd.learn (03-MCP Section 2.10).
/// Replaces: targeted edge strengthen/weaken bypass of Hebbian cycle
pub fn handle_learn(
    state: &mut SessionState,
    input: LearnInput,
) -> M1ndResult<serde_json::Value> {
    let mut graph = state.graph.write();

    let nodes: Vec<NodeId> = input
        .node_ids
        .iter()
        .filter_map(|id| graph.resolve_id(id))
        .collect();

    if nodes.is_empty() {
        return Ok(serde_json::json!({
            "error": "No valid node IDs found",
            "node_ids": input.node_ids,
        }));
    }

    // Expand the node set to include direct children (outgoing "contains"
    // edges).  This ensures that learn(["file::a.rs", "file::b.rs"]) also
    // strengthens/weakens edges between functions/structs contained in those
    // files, which is where the actual cross-file relationships live.
    let mut expanded: Vec<NodeId> = nodes.clone();
    if let Some(contains_str) = graph.strings.lookup("contains") {
        for &node in &nodes {
            let range = graph.csr.out_range(node);
            for k in range {
                if graph.csr.relations[k] == contains_str {
                    let child = graph.csr.targets[k];
                    if !expanded.contains(&child) {
                        expanded.push(child);
                    }
                }
            }
        }
    }

    let strength = input.strength;
    let mut edges_modified = 0u32;

    // Determine which node pairs to strengthen/weaken based on feedback type.
    // "correct"  → strengthen edges between all given nodes (Hebbian: fire together, wire together)
    // "wrong"    → weaken edges between all given nodes
    // "partial"  → strengthen edges among first half, weaken edges between first half and rest
    //
    // Uses the expanded set (specified nodes + their children) so that
    // cross-file function/struct edges are included.
    let (strengthen_set, weaken_set): (Vec<(NodeId, NodeId)>, Vec<(NodeId, NodeId)>) =
        match input.feedback.as_str() {
            "correct" => {
                // Strengthen all pairs
                let mut pairs = Vec::new();
                for i in 0..expanded.len() {
                    for j in (i + 1)..expanded.len() {
                        pairs.push((expanded[i], expanded[j]));
                    }
                }
                (pairs, Vec::new())
            }
            "wrong" => {
                // Weaken all pairs
                let mut pairs = Vec::new();
                for i in 0..expanded.len() {
                    for j in (i + 1)..expanded.len() {
                        pairs.push((expanded[i], expanded[j]));
                    }
                }
                (Vec::new(), pairs)
            }
            "partial" => {
                let mid = (expanded.len() + 1) / 2; // first half (rounded up)
                let first_half = &expanded[..mid];
                let rest = &expanded[mid..];
                // Strengthen edges among first half
                let mut s_pairs = Vec::new();
                for i in 0..first_half.len() {
                    for j in (i + 1)..first_half.len() {
                        s_pairs.push((first_half[i], first_half[j]));
                    }
                }
                // Weaken edges between first half and rest
                let mut w_pairs = Vec::new();
                for &a in first_half {
                    for &b in rest {
                        w_pairs.push((a, b));
                    }
                }
                (s_pairs, w_pairs)
            }
            _ => {
                // Unrecognized feedback — fall back to treating as "correct"
                let mut pairs = Vec::new();
                for i in 0..expanded.len() {
                    for j in (i + 1)..expanded.len() {
                        pairs.push((expanded[i], expanded[j]));
                    }
                }
                (pairs, Vec::new())
            }
        };

    // Helper closure: modify edge weight between src→tgt (if edge exists)
    let apply_delta = |graph: &mut m1nd_core::graph::Graph, src: NodeId, tgt: NodeId, delta: f32| -> u32 {
        let mut count = 0u32;
        let range = graph.csr.out_range(src);
        for k in range {
            if graph.csr.targets[k] == tgt {
                let edge_idx = EdgeIdx::new(k as u32);
                let current = graph.csr.read_weight(edge_idx).get();
                let new_weight = (current + delta).clamp(0.05, 3.0);
                let _ = graph.csr.atomic_write_weight(
                    edge_idx,
                    FiniteF32::new(new_weight),
                    64,
                );
                if k < graph.edge_plasticity.current_weight.len() {
                    graph.edge_plasticity.current_weight[k] = FiniteF32::new(new_weight);
                }
                count += 1;
            }
        }
        count
    };

    // Strengthen pairs
    for &(a, b) in &strengthen_set {
        edges_modified += apply_delta(&mut graph, a, b, strength);
        edges_modified += apply_delta(&mut graph, b, a, strength);
    }

    // Weaken pairs
    for &(a, b) in &weaken_set {
        edges_modified += apply_delta(&mut graph, a, b, -strength);
        edges_modified += apply_delta(&mut graph, b, a, -strength);
    }

    // Drop graph write-lock before accessing temporal (co_change needs &mut self)
    drop(graph);

    // Record co-change for all pairs of input nodes (feeds the predict tool).
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs_f64())
        .unwrap_or(0.0);
    for i in 0..nodes.len() {
        for j in (i + 1)..nodes.len() {
            let _ = state.temporal.co_change.record_co_change(nodes[i], nodes[j], now);
            let _ = state.temporal.co_change.record_co_change(nodes[j], nodes[i], now);
        }
    }

    Ok(serde_json::json!({
        "query": input.query,
        "feedback": input.feedback,
        "nodes_found": nodes.len(),
        "nodes_expanded": expanded.len(),
        "edges_modified": edges_modified,
        "strength": strength,
    }))
}

/// Handle m1nd.ingest (03-MCP Section 2.11).
/// Replaces: CodebaseIngestor.ingest() / ingest_incremental()
pub fn handle_ingest(
    state: &mut SessionState,
    input: IngestInput,
) -> M1ndResult<serde_json::Value> {
    use m1nd_ingest::IngestAdapter;

    let path = std::path::PathBuf::from(&input.path);
    if input.incremental && input.adapter != "code" {
        return Ok(serde_json::json!({
            "error": "incremental ingest is only supported for adapter 'code'",
        }));
    }

    match input.adapter.as_str() {
        "code" => {
            // Existing code ingestion path (default)
            let config = m1nd_ingest::IngestConfig {
                root: path.clone(),
                ..m1nd_ingest::IngestConfig::default()
            };

            let ingestor = m1nd_ingest::Ingestor::new(config);

            if input.incremental {
                let graph = state.graph.read();
                // For incremental, we'd need changed files list -- simplified here
                let changed: Vec<std::path::PathBuf> = vec![path];
                let (diff, stats) = ingestor.ingest_incremental(&graph, &changed)?;
                Ok(serde_json::json!({
                    "mode": "incremental",
                    "adapter": "code",
                    "files_scanned": stats.files_scanned,
                    "files_parsed": stats.files_parsed,
                    "nodes_created": stats.nodes_created,
                    "edges_created": stats.edges_created,
                    "elapsed_ms": stats.elapsed_ms,
                }))
            } else {
                let (new_graph, stats) = ingestor.ingest()?;
                finalize_ingest(state, &input, "code", new_graph, stats)
            }
        }
        "json" => {
            // JSON descriptor adapter -- domain-agnostic ingestion
            let adapter = m1nd_ingest::json_adapter::JsonIngestAdapter;
            let (new_graph, stats) = adapter.ingest(&path)?;
            finalize_ingest(state, &input, "json", new_graph, stats)
        }
        "memory" => {
            let adapter = m1nd_ingest::memory_adapter::MemoryIngestAdapter::new(input.namespace.clone());
            let (new_graph, stats) = adapter.ingest(&path)?;
            finalize_ingest(state, &input, "memory", new_graph, stats)
        }
        other => {
            Ok(serde_json::json!({
                "error": format!("Unknown adapter: '{}'. Supported: 'code', 'json', 'memory'", other),
            }))
        }
    }
}

/// Handle m1nd.resonate — resonance analysis via ResonanceEngine.
/// Exposes harmonics, sympathetic pairs, and resonant frequencies.
pub fn handle_resonate(
    state: &mut SessionState,
    input: ResonateInput,
) -> M1ndResult<serde_json::Value> {
    let graph = state.graph.read();

    // Resolve seeds: either from query or from a specific node_id
    let seeds: Vec<(NodeId, FiniteF32)> = if let Some(ref query) = input.query {
        let found = m1nd_core::seed::SeedFinder::find_seeds(&graph, query, 50)?;
        found
    } else if let Some(ref nid) = input.node_id {
        match graph.resolve_id(nid) {
            Some(node) => vec![(node, FiniteF32::ONE)],
            None => {
                return Ok(serde_json::json!({
                    "error": "Node not found",
                    "node_id": nid,
                }));
            }
        }
    } else {
        return Ok(serde_json::json!({
            "error": "Either 'query' or 'node_id' must be provided",
        }));
    };

    if seeds.is_empty() {
        return Ok(serde_json::json!({
            "error": "No seed nodes found for the given input",
        }));
    }

    let report = state.resonance.analyze(&graph, &seeds)?;

    let top_k = input.top_k;

    // Map harmonic results
    let harmonics: Vec<serde_json::Value> = report
        .harmonics
        .harmonics
        .iter()
        .map(|hr| {
            let antinodes: Vec<serde_json::Value> = hr
                .antinodes
                .iter()
                .take(top_k)
                .map(|&(node, amp)| {
                    let idx = node.as_usize();
                    let label = if idx < graph.num_nodes() as usize {
                        graph.strings.resolve(graph.nodes.label[idx]).to_string()
                    } else {
                        format!("node_{}", idx)
                    };
                    serde_json::json!({
                        "node_id": label,
                        "amplitude": amp.get(),
                    })
                })
                .collect();
            serde_json::json!({
                "harmonic": hr.harmonic,
                "frequency": hr.frequency.get(),
                "total_energy": hr.total_energy.get(),
                "antinodes": antinodes,
            })
        })
        .collect();

    // Map sympathetic resonance pairs
    let sympathetic_pairs: Vec<serde_json::Value> = report
        .sympathetic
        .sympathetic_nodes
        .iter()
        .take(top_k)
        .map(|&(node, amp)| {
            let idx = node.as_usize();
            let label = if idx < graph.num_nodes() as usize {
                graph.strings.resolve(graph.nodes.label[idx]).to_string()
            } else {
                format!("node_{}", idx)
            };
            serde_json::json!({
                "node_id": label,
                "resonance_amplitude": amp.get(),
            })
        })
        .collect();

    // Map resonant frequencies
    let resonant_frequencies: Vec<serde_json::Value> = report
        .resonant_frequencies
        .iter()
        .map(|rf| {
            serde_json::json!({
                "frequency": rf.frequency.get(),
                "total_energy": rf.total_energy.get(),
            })
        })
        .collect();

    // Standing wave summary
    let wave_pattern = serde_json::json!({
        "total_energy": report.standing_wave.total_energy.get(),
        "pulses_processed": report.standing_wave.pulses_processed,
        "antinode_count": report.standing_wave.antinodes.len(),
        "wave_node_count": report.standing_wave.wave_nodes.len(),
    });

    Ok(serde_json::json!({
        "harmonics": harmonics,
        "sympathetic_pairs": sympathetic_pairs,
        "resonant_frequencies": resonant_frequencies,
        "wave_pattern": wave_pattern,
        "harmonic_groups": report.harmonics.harmonic_groups.len(),
    }))
}

/// Handle m1nd.health (03-MCP Section 2.12).
pub fn handle_health(
    state: &mut SessionState,
    _input: HealthInput,
) -> M1ndResult<HealthOutput> {
    let graph = state.graph.read();

    let last_persist = state
        .last_persist_time
        .map(|t| format!("{:.0}s ago", t.elapsed().as_secs_f64()));

    Ok(HealthOutput {
        status: "ok".into(),
        node_count: graph.num_nodes(),
        edge_count: graph.num_edges() as u64,
        queries_processed: state.queries_processed,
        uptime_seconds: state.uptime_seconds(),
        memory_usage_bytes: 0, // simplified -- would need jemalloc stats
        plasticity_state: format!(
            "{} edges tracked",
            graph.edge_plasticity.original_weight.len()
        ),
        last_persist_time: last_persist,
        active_sessions: state.session_summary(),
    })
}
