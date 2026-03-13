# Structural Holes

Most developer tools find things that exist. m1nd finds things that *should* exist but do not. A missing error handler. A test file that was never written. A validation function that every sibling module has except one. These are structural holes -- gaps in the code graph that reveal omissions, risks, and design inconsistencies.

## The network theory origin

The concept of structural holes comes from sociologist Ronald Burt's 1992 book *Structural Holes: The Social Structure of Competition*. Burt observed that in social networks, the most valuable positions are not the most connected ones -- they are the ones that *bridge* otherwise disconnected groups. The gap between two groups is a structural hole. The person who spans it controls information flow.

Burt was studying people and organizations, but the insight transfers directly to code. In a codebase graph, a structural hole is a place where a connection *should* exist based on the surrounding structure but does not. The module that imports five out of six modules in a cluster but skips the sixth. The service that handles create, read, update, but not delete. The directory where every file has a test except one.

These gaps are invisible to text search. You cannot grep for something that is absent. You cannot find a missing test by scanning the test directory -- you have to know what *should* be there and check whether it *is* there. This requires structural reasoning over the graph, not pattern matching over text.

## How m1nd detects structural holes

m1nd detects structural holes through a combination of activation context and neighborhood analysis. The detection is embedded in the query pipeline and can also be invoked directly via the `m1nd.missing` tool.

### The algorithm

The core insight is: **a node surrounded by activated neighbors but not activated itself is probably missing something.**

After a spreading activation query produces its ranked results, the structural hole detector scans every node in the graph:

```rust
pub fn detect_structural_holes(
    &self,
    graph: &Graph,
    activation: &ActivationResult,
    min_sibling_activation: FiniteF32,
) -> M1ndResult<Vec<StructuralHole>> {
    // Build activation lookup
    let mut act_map = vec![0.0f32; n];
    for a in &activation.activated {
        act_map[a.node.as_usize()] = a.activation.get();
    }

    for each node i:
        // Skip if already activated (it is in the results)
        if act_map[i] > 0.01: continue

        // Count activated neighbors
        for each neighbor of i:
            if act_map[neighbor] > min_sibling_activation:
                activated_neighbors += 1
                neighbor_act_sum += act_map[neighbor]

        // If 2+ neighbors are activated but this node is not: structural hole
        if activated_neighbors >= 2:
            holes.push(StructuralHole {
                node: i,
                sibling_avg_activation: neighbor_act_sum / activated_neighbors,
                reason: format!("{} activated neighbors (avg={:.2}) but node inactive",
                    activated_neighbors, avg),
            })
```

The threshold of 2 activated neighbors prevents false positives from nodes that happen to touch one activated node. The `min_sibling_activation` parameter (default 0.3) filters out weakly activated neighbors. Results are sorted by the average activation of the surrounding neighbors -- the higher the average, the more conspicuous the absence.

### Why this works

Consider a query about "authentication." The activation pattern covers `auth.py`, `session.py`, `middleware.py`, `user_model.py`, and their neighbors. Now suppose `auth_test.py` does not exist. The test files for every other module in the auth cluster exist and are activated (because they have edges to the modules they test). `auth_test.py` is absent from the graph entirely. But if it *did* exist, it would be surrounded by activated nodes. Its absence is detectable by the gap it leaves.

In the more subtle case, `auth_test.py` exists but lacks tests for the password reset flow. The test file node exists and is activated, but `password_reset_handler.py` is activated while `auth_test.py`'s test coverage edges do not include it. The neighborhood analysis detects this: `password_reset_handler.py`'s siblings all have test edges, but this one does not.

## The m1nd.missing tool

The `m1nd.missing` tool wraps structural hole detection in a purpose-built interface. Instead of requiring the caller to run an activation query first, `m1nd.missing` accepts a natural-language query, runs internal activation, and returns only the holes.

The tool exposes the detection through the MCP protocol. Behind the scenes, it:

1. Finds seed nodes matching the query text.
2. Runs full four-dimension spreading activation (structural, semantic, temporal, causal).
3. Analyzes the activation pattern for structural holes using the neighborhood algorithm.
4. Returns the top 10 holes sorted by surrounding activation strength.

Each hole includes:
- **node_id**: the node at the center of the gap.
- **label**: human-readable identifier.
- **node_type**: file, function, class, etc.
- **reason**: a description of why this is a hole (e.g., "4 activated neighbors (avg=0.72) but node inactive").
- **sibling_avg_activation**: quantifies how strongly the surrounding nodes were activated.

## Real examples

### Missing error handler

Query: "error handling in the payment flow"

Activation lights up: `payment_handler.py`, `billing.py`, `refund.py`, `payment_errors.py`, `retry_logic.py`.

Structural hole detected: `webhook_handler.py` has 3 activated neighbors (`payment_handler.py`, `billing.py`, `payment_errors.py`) but is itself not activated. Investigation reveals: the webhook handler processes Stripe callbacks but has no error handling -- raw exceptions propagate to the caller. Every other module in the payment cluster catches and wraps errors via `payment_errors.py`. The webhook handler does not.

```
StructuralHole {
    node: "file::webhook_handler.py",
    sibling_avg_activation: 0.71,
    reason: "3 activated neighbors (avg=0.71) but node inactive"
}
```

### Missing test file

Query: "authentication testing"

Activation lights up: `test_auth.py`, `test_session.py`, `test_middleware.py`, `auth.py`, `session.py`.

Structural hole detected: `password_reset.py` has 4 activated neighbors but zero activation itself. The module handles password reset logic. It imports from `auth.py` and `session.py` (both activated). It is imported by `routes.py` (activated). But there is no `test_password_reset.py` in the graph, and `password_reset.py` has no test-relationship edges. Every sibling module in the auth cluster has a corresponding test file. This one does not.

```
StructuralHole {
    node: "file::password_reset.py",
    sibling_avg_activation: 0.68,
    reason: "4 activated neighbors (avg=0.68) but node inactive"
}
```

### Missing validation

Query: "input validation for API endpoints"

Activation lights up: `validate_user_input.py`, `validate_payment_input.py`, `validate_search_params.py`, `schema_validator.py`.

Structural hole detected: `admin_routes.py` has 3 activated neighbors (the validation modules it shares with other route handlers) but is not activated itself. Investigation reveals: user routes, payment routes, and search routes all import and call validation functions. Admin routes do not. The admin API accepts raw input without validation -- a security risk hidden in a structural gap.

```
StructuralHole {
    node: "file::admin_routes.py",
    sibling_avg_activation: 0.65,
    reason: "3 activated neighbors (avg=0.65) but node inactive"
}
```

## Why no other tool can do this

### Text search (grep, ripgrep)

Text search finds strings. It cannot find the *absence* of a string. You cannot grep for "the test file that should exist but does not." You would need to know in advance what you are looking for, which defeats the purpose.

### Static analysis (AST, linters)

Linters can enforce specific rules ("every function must have a docstring") but cannot detect structural patterns across the graph ("every module in this cluster has a test file except this one"). They operate on individual files, not on the relationships between files.

### RAG / embedding search

Embedding search finds documents similar to a query. It cannot detect the gap between documents. If `test_password_reset.py` does not exist, there is no document to embed. The absence is invisible.

### Code coverage tools

Coverage tools tell you which lines of code are executed during tests. They can identify untested code within a file. They cannot identify *missing files* -- a test file that was never created has zero coverage, indistinguishable from a file that exists but is not run.

### m1nd's advantage

m1nd operates on the graph structure, not on file contents. It reasons about the *topology* of relationships: which nodes connect to which other nodes, and where those connections are unexpectedly absent. The spreading activation query establishes a context ("what is related to authentication?"), and the structural hole detector finds gaps within that context.

This is a fundamentally different capability. It is not better text search, or better static analysis, or better coverage. It is structural reasoning about the shape of the code graph -- something that requires a graph in the first place, and an activation pattern to establish context.

## Connection to other concepts

Structural hole detection works best when combined with other m1nd capabilities:

- **Spreading activation** (see [Spreading Activation](spreading-activation.md)) provides the activation context that defines "which neighbors count." Without activation, every node would be checked against every other node -- too noisy to be useful.

- **Hebbian plasticity** (see [Hebbian Plasticity](hebbian-plasticity.md)) sharpens the detection over time. As edges on well-tested paths strengthen and edges on untested paths weaken, the contrast between activated and inactive nodes increases. Holes become more conspicuous.

- **XLR noise cancellation** (see [XLR Noise Cancellation](xlr-noise-cancellation.md)) removes hub-node noise from the activation pattern before hole detection runs. Without XLR, `config.py` being activated might cause false-positive holes in unrelated modules that happen to import config. With XLR, the activation pattern is cleaner, and the holes that surface are more meaningful.

## Interpreting results

A structural hole is not necessarily a bug. It is a *signal* that something deviates from the surrounding pattern. Possible interpretations:

| Hole type | What it means | Action |
|-----------|--------------|--------|
| Missing test file | A module has no tests while its siblings do | Write tests or mark as intentionally untested |
| Missing error handling | A module does not use the error patterns its siblings use | Add error handling or document why it is unnecessary |
| Missing validation | An endpoint lacks input validation that peer endpoints have | Add validation -- likely a security gap |
| Missing import | A module does not import a shared utility that all siblings import | Check if the module implements the functionality differently |
| Missing documentation | A module has no doc-edges while siblings do | Write documentation or accept the gap |
| Intentional isolation | A module is deliberately decoupled from a cluster | No action -- but the hole confirms the isolation is real |

The `sibling_avg_activation` score helps prioritize. A hole where surrounding nodes have an average activation of 0.85 is more suspicious than one at 0.35. The former means the node is deeply embedded in a strongly activated cluster and conspicuously absent. The latter means the surrounding nodes were only weakly related to the query -- the "hole" may just be a node in a different domain.
