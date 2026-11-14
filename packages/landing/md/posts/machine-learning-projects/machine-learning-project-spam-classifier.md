---
title: "Spam Message Classifier: NLP Project"
description: "Classify SMS messages as spam or ham using TF-IDF vectorization and multinomial Naive Bayes in Rust."
author: "Mars Team"
date: "2026-11-14"
order: 38
tags: ["machine-learning", "rust", "nlp", "text-classification", "naive-bayes"]
---

# Machine Learning Project: Spam Message Classifier

Spam filtering is one of the most commercially important ML applications in
history — and one of the simplest to implement. The **SMS Spam Collection**
dataset has ~5,500 labeled text messages; your job is to predict `spam` vs
`ham` (not spam) from the message text alone.

In this project you will:

1. Turn raw text into numbers — the core NLP preprocessing pipeline.
2. Build a **TF-IDF** representation and a **Naive Bayes** classifier.
3. Understand why Naive Bayes still beats fancy models on small text data.
4. Evaluate with precision/recall — where spam filtering actually lives.

> Rust ML note: `tokenizers`/`regex` for text processing, `ndarray` for
> vectors, `linfa-bayes` or a hand-rolled multinomial NB.

## From text to numbers

Computers don't read text — you must convert each message into a vector.
The classic pipeline:

```
"FREE!!! Claim your prize NOW" 
  → lowercase, strip punctuation
  → tokenize: ["free", "claim", "your", "prize", "now"]
  → count words → TF-IDF vector
  → Naive Bayes predicts P(spam | words)
```

**TF-IDF** (term frequency × inverse document frequency) downweights words
that appear everywhere ("the", "you") and upweights distinctive ones
("prize", "viagra", "winner").

```rust
// :dep regex = "1"
// :dep ndarray = "0.16"

use regex::Regex;
use std::collections::HashMap;

fn tokenize(text: &str) -> Vec<String> {
    // lowercase, replace non-alphanumeric with space, split
    let re = Regex::new(r"[^a-z0-9\s]").unwrap();
    re.replace_all(&text.to_lowercase(), " ")
        .split_whitespace()
        .map(|s| s.to_string())
        .collect()
}

let messages = vec![
    "FREE entry in 2 a wkly comp to win FA Cup final tkts",
    "Hey are we still on for lunch tomorrow?",
    "URGENT! Your account has been suspended. Call now",
];

for msg in &messages {
    println!("{:?} -> {:?}", &msg[..20], tokenize(msg));
}
```

## TF-IDF vectorization

Build a vocabulary of all words, then represent each document as a vector.
Naive Bayes actually only needs word **counts** — but TF-IDF vectors work
with any linear model, so we compute them anyway.

```rust
use ndarray::{s, Array2, Axis};
use std::collections::HashSet;

struct Tfidf {
    vocab: HashMap<String, usize>,
    idf:   Vec<f64>,
}

impl Tfidf {
    fn fit(documents: &[Vec<String>]) -> Self {
        // vocabulary: every unique word, sorted for determinism
        let vocab_set: HashSet<&String> =
            documents.iter().flatten().collect();
        let mut vocab: Vec<String> =
            vocab_set.into_iter().cloned().collect();
        vocab.sort();
        let vocab_map: HashMap<String, usize> = vocab
            .iter().enumerate()
            .map(|(i, w)| (w.clone(), i))
            .collect();

        // idf[w] = ln(N / df[w]) + 1, df = # docs containing w
        let n = documents.len() as f64;
        let mut df = vec![0.0f64; vocab.len()];
        for doc in documents {
            let mut seen = HashSet::new();
            for w in doc {
                if seen.insert(w) {
                    if let Some(&i) = vocab_map.get(w) { df[i] += 1.0; }
                }
            }
        }
        let idf: Vec<f64> = df.iter().map(|&d| (n / d).ln() + 1.0).collect();

        Self { vocab: vocab_map, idf }
    }

    fn transform(&self, doc: &[String]) -> Vec<f64> {
        let mut tf = vec![0.0f64; self.vocab.len()];
        let mut counts = HashMap::new();
        for w in doc { *counts.entry(w).or_insert(0) += 1; }
        let total = doc.len() as f64;
        for (w, c) in counts {
            if let Some(&i) = self.vocab.get(w) {
                tf[i] = (c as f64 / total) * self.idf[i];
            }
        }
        tf
    }
}
```

## Multinomial Naive Bayes

Naive Bayes makes one strong (naive) assumption: **words are independent
given the class**. It's wrong — "free" and "win" are correlated — but it
works shockingly well in practice, especially on small datasets.

For each class, it learns `P(word | class)` from training counts, then
scores a new message with Bayes' rule and picks the class with the highest
posterior probability.

```rust
struct MultinomialNB {
    class_log_prior: HashMap<String, f64>,
    feature_log_prob: HashMap<String, HashMap<String, f64>>,
    class_totals: HashMap<String, f64>,
    vocab_size: usize,
    alpha: f64,   // Laplace smoothing
}

impl MultinomialNB {
    fn fit(texts: &[Vec<String>], labels: &[String]) -> Self {
        let vocab_size = texts.iter().flatten()
            .collect::<HashSet<_>>().len();
        let alpha = 1.0;

        // count words per class
        let mut class_counts: HashMap<String, usize> = HashMap::new();
        let mut word_counts: HashMap<String, HashMap<String, f64>> = HashMap::new();
        let mut totals: HashMap<String, f64> = HashMap::new();

        for (doc, label) in texts.iter().zip(labels.iter()) {
            *class_counts.entry(label.clone()).or_insert(0) += 1;
            for w in doc {
                *word_counts.entry(label.clone())
                    .or_default().entry(w.clone()).or_insert(0.0) += 1.0;
                *totals.entry(label.clone()).or_insert(0.0) += 1.0;
            }
        }

        let n = texts.len() as f64;
        let class_log_prior: HashMap<String, f64> = class_counts
            .iter().map(|(c, &n_c)| {
                (c.clone(), (n_c as f64 / n).ln())
            }).collect();

        let feature_log_prob = word_counts.iter().map(|(c, wc)| {
            let total = totals[c] + alpha * vocab_size as f64;
            let probs = wc.iter().map(|(w, &count)| {
                // log P(w|c) with Laplace smoothing
                (w.clone(), ((count + alpha) / total).ln())
            }).collect();
            (c.clone(), probs)
        }).collect();

        Self { class_log_prior, feature_log_prob,
               class_totals: totals, vocab_size, alpha }
    }

    fn predict_log_proba(&self, doc: &[String]) -> HashMap<String, f64> {
        self.class_log_prior.iter().map(|(c, &prior)| {
            let mut score = prior;
            if let Some(probs) = self.feature_log_prob.get(c) {
                let total = self.class_totals.get(c).copied().unwrap_or(0.0)
                    + self.alpha * self.vocab_size as f64;
                for w in doc {
                    // unseen word: floor probability
                    score += probs.get(w).copied().unwrap_or_else(|| {
                        (self.alpha / total).ln()
                    });
                }
            }
            (c.clone(), score)
        }).collect()
    }
}
```

## Why Naive Bayes beats fancy models here

On the SMS Spam dataset (~5.5k short messages), a multinomial NB trained
on raw word counts typically hits **~97% accuracy** — competitive with
SVMs and neural nets, at a fraction of the compute.

Reasons:

- **Small data**: deep models need millions of examples; NB is robust
  with thousands.
- **Short texts**: word independence is less wrong for 1–2 sentence
  messages than for paragraphs.
- **Calibrated-ish probabilities**: the spam score can double as a
  "how confident" dial for filtering thresholds.

## Evaluation — precision and recall

Accuracy alone is misleading when classes are imbalanced (~13% spam).
The metrics that matter:

| Metric | Question it answers |
| ------ | ------------------- |
| **Precision** | Of messages flagged spam, how many really are? (false-positive cost) |
| **Recall** | Of real spam, how much did we catch? (false-negative cost) |
| **F1** | Harmonic mean — balances both |

For a spam filter, **precision > recall** is usually right: marking a
real email as spam is worse than letting one spam through.

```rust
fn precision_recall(preds: &[bool], labels: &[bool])
    -> (f64, f64, f64)
{
    let tp = preds.iter().zip(labels).filter(|(&p, &l)| p && l).count();
    let fp = preds.iter().zip(labels).filter(|(&p, &l)| p && !l).count();
    let fn_ = preds.iter().zip(labels).filter(|(&p, &l)| !p && l).count();

    let precision = tp as f64 / (tp + fp).max(1) as f64;
    let recall    = tp as f64 / (tp + fn_).max(1) as f64;
    let f1 = 2.0 * precision * recall / (precision + recall).max(1e-9);
    (precision, recall, f1)
}

// Threshold tuning: NB gives a spam probability; you choose the cutoff.
// cutoff=0.5 → balanced; cutoff=0.7 → higher precision, lower recall.
// In production you'd pick the cutoff on a validation set, not test.
```

## Results and next steps

Typical SMS-spam results:

| Model | Accuracy | Precision | Recall | F1 |
| ----- | -------- | --------- | ------ | -- |
| Multinomial NB | ~97% | ~0.93 | ~0.92 | ~0.92 |
| Linear SVM (TF-IDF) | ~98% | ~0.95 | ~0.94 | ~0.94 |
| Logistic regression | ~97% | ~0.94 | ~0.93 | ~0.93 |

**Next steps:**

- Add **bigrams** ("free entry", "call now") — captures phrases unigrams miss.
- Try `linfa-svm` or logistic regression on the same TF-IDF matrix.
- For real email, add HTML stripping, URL/domain features, and sender
  reputation — text alone isn't enough.

**Key takeaway:** TF-IDF + Naive Bayes is the 80/20 of text classification.
Master this baseline before reaching for transformers.
