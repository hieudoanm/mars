---
title: 'NetworkX vs petgraph: Graph Libraries in Python vs Rust'
description: "Compare Python's NetworkX with Rust's petgraph for graph data structures and network algorithms in notebook environments."
author: 'Project Mars Team'
date: '2026-11-14'
order: 21
tags: ['data-science', 'networkx', 'petgraph', 'rust', 'python', 'graphs', 'mars']
---

# NetworkX vs petgraph: Graph Data Structures Compared

NetworkX is the go-to Python library for graphs and networks. Rust's **`petgraph`** provides graph data structures and algorithms—from traversals to shortest paths—with strong performance and type safety for network analysis tasks.

## What NetworkX does

NetworkX provides a comprehensive graph toolkit:

- **Graph types**: directed, undirected, multigraphs
- **Algorithms**: shortest paths, centrality, clustering, community detection
- **I/O**: support for GML, GraphML, JSON, and other formats
- **Drawing** via Matplotlib integration
- **Analysis tools**: degree distribution, connectivity, assortativity
- **Large-scale** computation with optimized algorithms

It became the default library for network analysis in Python, used across social network analysis, bioinformatics, and infrastructure modeling.

## What petgraph brings to Rust

**`petgraph`** is a Rust library for graph data structures and algorithms:

- **Graph types**: directed, undirected, multigraphs with generic node/edge types
- **Algorithms**: Dijkstra, A*, Bellman-Ford, Floyd-Warshall, centrality measures
- **Traversals**: BFS, DFS, topological sort
- **Connectivity**: strongly connected components, minimum spanning trees
- **Performance**: O(1) neighbor access, memory-efficient representations
- **Type safety**: compile-time guarantees for graph operations

The key difference: petgraph provides graph primitives with Rust's type system and compile-time safety, making it suitable for performance-critical network analysis tasks.

## Comparison

| Feature        | NetworkX (Python)                     | petgraph (Rust)                |
| -------------- | ------------------------------------- | ------------------------------ |
| Graph creation | `G = nx.Graph()`                      | `Graph::new()`                 |
| Add edge       | `G.add_edge(0, 1)`                    | `g.add_edge(0, 1)`             |
| Shortest path  | `nx.shortest_path(G, source, target)` | `dijkstra(&g, source, target)` |
| Type safety    | Runtime                               | Compile-time                   |
| Performance    | Python interpreter                    | Native compilation             |
| Memory model   | Python objects                        | Struct-based, efficient        |

## Example: NetworkX

```python
import networkx as nx

G = nx.Graph()
G.add_edge(0, 1)
G.add_edge(1, 2)
path = nx.shortest_path(G, 0, 2)
```

## Example: petgraph

```rust
use petgraph::Graph;
use petgraph::algo::dijkstra;

let mut g = Graph::new();
g.add_edge(0, 1, ());
g.add_edge(1, 2, ());
let path = dijkstra(&g, 0, Some(2));
```

## When to use each

- **NetworkX** in Python when you need a comprehensive graph toolkit, visualization integration, or rapid prototyping of network analysis.
- **petgraph** in Rust when you need type-safe graph operations, performance for large-scale network analysis, or want to avoid the Python interpreter overhead.

## How Project Mars supports both

Project Mars supports both NetworkX and petgraph in their respective language runtimes:

- **Python cells** can use NetworkX with standard library imports
- **Rust cells** can use petgraph via Cargo dependencies
- Both can coexist in the same `.mars` notebook
- Graphs persist across cells within each language runtime

Start with the [getting started guide](/docs/getting-started) to explore NetworkX and petgraph in Project Mars notebooks.
