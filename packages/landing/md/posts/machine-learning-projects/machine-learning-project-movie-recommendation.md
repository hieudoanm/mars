---
title: "Movie Recommendation System: Collaborative Filtering"
description: "Build a recommender system with collaborative filtering and matrix factorization on the MovieLens dataset in Rust."
author: "Project Mars Team"
date: "2026-11-14"
order: 39
tags: ["machine-learning", "rust", "recommender-systems", "collaborative-filtering", "als"]
---

# Machine Learning Project: Movie Recommendation System

Recommender systems power Netflix, Spotify, and Amazon. The classic
approach — **collaborative filtering** — needs no knowledge of what movies
*are*; it only needs the pattern of who watched what and how they rated it.

We'll use **MovieLens-100K**: 100,000 ratings from ~1,000 users on ~1,700
movies, each rated 1–5 stars.

In this project you will:

1. Frame recommendation as a **matrix completion** problem.
2. Implement **user-based** and **item-based** collaborative filtering.
3. Train **matrix factorization** (SVD-style latent factors) with ALS.
4. Evaluate with RMSE and ranking metrics.

> Rust ML note: `ndarray` for the sparse rating matrix,
> `linfa` or hand-rolled ALS for factorization.

## The rating matrix

Ratings form a sparse matrix `R[user, movie] → rating`:

```
        m1   m2   m3   m4
u1      5    ?    3    ?
u2      ?    4    ?    2
u3      4    ?    ?    5
```

`?` means "not rated". The goal: fill in the missing entries well enough
to rank unseen movies for each user. This is **collaborative filtering** —
users who agreed in the past tend to agree in the future.

```rust
// :dep csv = "1"
// :dep serde = { version = "1", features = ["derive"] }
// :dep ndarray = "0.16"
// :dep sprs = "0.11"   // sparse matrices

use sprs::CsMat;
use std::collections::HashMap;

// MovieLens u.data format: user_id::movie_id::rating::timestamp
#[derive(serde::Deserialize)]
struct Rating {
    user_id:  u32,
    movie_id: u32,
    rating:   f32,
}

fn load_ratings(path: &str) -> (CsMat<f32>, HashMap<u32,u32>, HashMap<u32,u32>) {
    let mut users = HashMap::new();
    let mut movies = HashMap::new();
    let mut entries = Vec::new();

    let rdr = csv::ReaderBuilder::new()
        .delimiter(b':')
        .has_headers(false)
        .from_path(path).unwrap();

    for rec in rdr.deserialize::<Rating>() {
        let r = rec.unwrap();
        let u = *users.entry(r.user_id).or_insert(users.len() as u32);
        let m = *movies.entry(r.movie_id).or_insert(movies.len() as u32);
        entries.push(((u as usize, m as usize), r.rating));
    }
    let shape = (users.len(), movies.len());
    let mat = CsMat::new(shape, entries);
    (mat, users, movies)
}

let (ratings, user_map, movie_map) = load_ratings("u.data");
println!("ratings matrix: {} users × {} movies, {} nonzeros",
         ratings.rows(), ratings.cols(), ratings.nnz());
```

## Baseline — popularity

Before any real model, compute the dumbest baseline: recommend the
globally most-rated movies to everyone. It scores surprisingly well on
RMSE but has zero personalization — every user gets the same list.
This is the baseline your real model must beat.

```rust
// Item popularity = mean rating × log(count) — a simple hybrid
let mut item_sum = vec![0.0f32; ratings.cols()];
let mut item_cnt = vec![0usize; ratings.cols()];

for ((_, m), v) in ratings.iter() {
    item_sum[m] += v;
    item_cnt[m] += 1;
}

let popularity: Vec<f32> = (0..ratings.cols())
    .map(|m| {
        let mean = item_sum[m] / item_cnt[m].max(1) as f32;
        mean * (item_cnt[m] as f32).ln().max(1.0)   // dampen one-hit wonders
    })
    .collect();

// Top-10 global recommendations
let mut ranked: Vec<usize> = (0..ratings.cols()).collect();
ranked.sort_by(|&a, &b| popularity[b].partial_cmp(&popularity[a]).unwrap());
println!("top-10 popular movies: {:?}", &ranked[..10]);
```

## User-based collaborative filtering

Find users *similar* to you (correlation or cosine similarity of rating
vectors), then predict your rating for a movie as the weighted average of
similar users' ratings for that movie.

This works but doesn't scale — comparing every pair of users is O(n²).
Real systems switched to item-based CF, then to matrix factorization.

```rust
fn cosine_sim(a: &[(usize,f32)], b: &[(usize,f32)]) -> f64 {
    // sparse dot product of two rating vectors
    let mut dot = 0.0; let mut na = 0.0; let mut nb = 0.0;
    for &(_, v) in a { na += (v as f64).powi(2); }
    for &(_, v) in b { nb += (v as f64).powi(2); }
    // intersection via merge
    let (mut i, mut j) = (0, 0);
    while i < a.len() && j < b.len() {
        match a[i].0.cmp(&b[j].0) {
            std::cmp::Ordering::Equal => {
                dot += a[i].1 as f64 * b[j].1 as f64; i += 1; j += 1;
            }
            std::cmp::Ordering::Less => i += 1,
            std::cmp::Ordering::Greater => j += 1,
        }
    }
    dot / (na.sqrt() * nb.sqrt()).max(1e-9)
}

// predict(user u, movie m):
//   neighbors = top-k users by cosine_sim(u, ·)
//   pred = Σ sim(u,v)·R[v,m] / Σ |sim(u,v)|
// Only users who rated m contribute — this is why CF needs overlap.
```

## Matrix factorization — the Netflix prize approach

Instead of comparing users directly, learn **latent factors**: each user
and each movie gets a k-dimensional vector such that

```
R[u,m] ≈ U[u] · V[m]ᵀ  +  bias_u  +  bias_m  +  global_mean
```

The dot product captures taste: if a user's vector aligns with a movie's
vector, they'll like it. Train by minimizing squared error with
**alternating least squares (ALS)** — fix U, solve for V; fix V, solve for U.
Each step is a small ridge-regression problem with a closed-form solution.

```rust
fn als_step(
    fixed: &ndarray::Array2<f64>,   // user or item factors [n, k]
    ratings: &CsMat<f32>,
    lam: f64,
) -> ndarray::Array2<f64> {
    let k = fixed.ncols();
    let n = fixed.nrows();
    let mut out = ndarray::Array2::zeros((n, k));

    for i in 0..n {
        // gather this entity's observed ratings
        let obs: Vec<(usize, f32)> = ratings
            .outer_iterator()
            .nth(i).map(|row| {
                row.iter().map(|(j, v)| (j, v)).collect()
            }).unwrap_or_default();

        // A = Σ V_j V_jᵀ + λI ;  b = Σ r_ij V_j
        let mut a = ndarray::Array2::zeros((k, k));
        let mut b = ndarray::Array1::zeros(k);
        for (j, r) in &obs {
            let vj = fixed.row(*j);
            for p in 0..k {
                b[p] += *r as f64 * vj[p];
                for q in 0..k {
                    a[[p, q]] += vj[p] * vj[q];
                }
            }
        }
        for p in 0..k { a[[p, p]] += lam; }
        out.row_mut(i).assign(&ndarray_linalg::solve::Inverse::inv(&a)
            .unwrap().dot(&b));
    }
    out
}

// Iterate: U = als_step(V); V = als_step(U);  ~10-20 iterations
// k=50 latent factors, λ=0.1 regularization are good starting points.
```

## Evaluation

For rating prediction, **RMSE** on held-out ratings:

| Model | RMSE |
| ----- | ---- |
| Global mean | ~1.0 |
| Popularity baseline | ~0.95 |
| Matrix factorization (k=50) | ~0.90 |
| SVD++ / biased MF | ~0.88 |

For the *real* task — ranking — RMSE is a proxy. Production systems measure
**precision@k** and **recall@k**: did the top-10 list contain movies the
user actually rated highly?

## Key takeaways

1. **Recommendation is matrix completion**, not classification.
2. **Popularity is the baseline** — always compute it first.
3. **User-based CF doesn't scale**; matrix factorization is the scalable
   version of the same idea.
4. **RMSE ≠ ranking quality** — a model can predict ratings well but rank
   badly; evaluate both.

**Next steps:**

- Add **time-decay** weights (recent ratings matter more) and **bias**
  terms (some users rate everything 5★).
- For content-rich domains, combine CF with content features — see the
  sentiment-analysis project for text embeddings.
- Real systems (Netflix, YouTube) use two-tower neural retrieval +
  ranking models; ALS is the classical foundation they replaced.
