---
title: 'Sentiment Analysis: NLP Classification Project'
description: 'Classify movie review sentiment as positive or negative using TF-IDF features and logistic regression in Rust.'
author: 'Project Mars Team'
date: '2026-11-14'
order: 42
tags: ['machine-learning', 'rust', 'nlp', 'sentiment-analysis', 'text-classification']
---

# Machine Learning Project: Sentiment Analysis

**Sentiment analysis** classifies text as expressing a positive, negative,
or neutral opinion. The IMDb movie review dataset (50,000 reviews labeled
positive/negative) is the standard benchmark — it's also the dataset that
made deep learning for NLP famous.

In this project you will:

1. Reuse the TF-IDF pipeline from the spam project — the same machinery
   works for any text classification.
2. Train **logistic regression** on bag-of-words features.
3. Compare classical ML vs. the transformer approach (BERT).
4. Understand when deep learning actually helps for text.

> Rust ML note: same text pipeline as spam classification. For the
> transformer version, `rust-bert` or `candle` provides BERT inference.

## Why sentiment analysis is different from spam

Spam detection looks for _known bad patterns_ — specific words, phrases,
urgency markers. Sentiment is subtler:

- **Negation**: "not bad" is positive, "not good" is negative
- **Sarcasm**: "Great, another delay" is negative
- **Intensifiers**: "absolutely loved it" vs. "liked it"
- **Domain words**: "unputdownable" is positive for books, meaningless
  for products

Bag-of-words captures _which_ words appear but not _how_ they combine —
that's the limitation transformers solve.

```rust
// :dep regex = "1"
// :dep ndarray = "0.16"
// :dep linfa = "0.7"
// :dep linfa-logistic = "0.7"

use regex::Regex;

fn tokenize(text: &str) -> Vec<String> {
    // same pipeline as spam: lowercase, strip punct, split
    // plus: simple negation handling — prepend NOT_ to words after "not"
    let re = Regex::new(r"[^a-z0-9\s]").unwrap();
    let clean = re.replace_all(&text.to_lowercase(), " ");
    let words: Vec<String> = clean.split_whitespace()
        .map(|s| s.to_string()).collect();

    // bigram-ish negation: "not good" → "not_good" as single token
    let mut out = Vec::new();
    let mut prev_neg = false;
    for w in words {
        if prev_neg {
            out.push(format!("NOT_{}", w));
            prev_neg = false;
        } else if w == "not" || w == "never" || w == "no" {
            prev_neg = true;
        } else {
            out.push(w);
        }
    }
    out
}

let reviews = vec![
    ("An absolute masterpiece, I loved every minute", 1),
    ("Terrible film, a complete waste of time", 0),
    ("Not bad at all, surprisingly enjoyable", 1),
    ("I did not enjoy this movie", 0),
];

for (text, label) in &reviews {
    println!("label={} {:?}", label, tokenize(text));
}
```

## TF-IDF + logistic regression

Same vectorization as spam, but on longer documents (full reviews).
Logistic regression on TF-IDF features is the **classical baseline** —
it's fast, interpretable, and hard to beat on medium-sized data.

```rust
// Reuse Tfidf vectorizer from the spam project
let docs: Vec<Vec<String>> = train_texts.iter()
    .map(|t| tokenize(t)).collect();
let vectorizer = Tfidf::fit(&docs);

// Build feature matrix [n_samples, vocab_size]
let x = Array2::from_shape_fn((docs.len(), vectorizer.vocab.len()),
    |(i, _)| 0.0);   // fill via transform
for (i, doc) in docs.iter().enumerate() {
    let v = vectorizer.transform(doc);
    for (j, val) in v.iter().enumerate() {
        x[[i, j]] = *val;
    }
}

// sklearn: LogisticRegression(C=1.0, max_iter=1000)
let model = LogisticRegression::default()
    .fit(&Dataset::from(x).map_targets(|t| *t))
    .expect("fit failed");

// predict_proba → P(positive)
```

## Evaluation

| Model                        | Accuracy | Notes                                |
| ---------------------------- | -------- | ------------------------------------ |
| Majority class               | ~50%     | useless baseline                     |
| Logistic regression (TF-IDF) | ~88%     | fast, interpretable                  |
| Linear SVM (TF-IDF)          | ~89%     | slightly better, less interpretable  |
| BERT-base fine-tuned         | ~93%     | state-of-the-art for this dataset    |
| DistilBERT                   | ~92%     | 40% smaller, ~97% of BERT's accuracy |

The 5-point gap between classical and transformer models is real but
modest — for many production use-cases, logistic regression on TF-IDF
is the right cost/accuracy trade-off.

```rust
// Inspect top positive/negative words — model interpretability
let mut coef_sorted: Vec<(usize, f64)> = model
    .params().feature_weights().iter().enumerate()
    .map(|(i, &c)| (i, c))
    .collect();
coef_sorted.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

println!("top positive words:");
for (i, c) in coef_sorted.iter().take(10) {
    println!("  {} ({:+.2})", vocab_name(*i), c);
}
println!("top negative words:");
for (i, c) in coef_sorted.iter().rev().take(10) {
    println!("  {} ({:+.2})", vocab_name(*i), c);
}
```

## When to use transformers

**Use classical ML (TF-IDF + logistic) when:**

- Dataset is < 100k examples
- Latency/cost matters (no GPU, edge deployment)
- You need interpretability (which words drove the decision)
- Text is short and formulaic (reviews, support tickets)

**Use transformers (BERT, DistilBERT) when:**

- Dataset is large (> 500k examples)
- Text is long and complex (documents, conversations)
- Nuance matters (sarcasm, context, domain-specific language)
- You have GPU budget for fine-tuning

## The transformer alternative

For comparison, here's what the BERT approach looks like in Rust
using the `rust-bert` crate:

```rust
// rust-bert provides pretrained BERT models
// pip install equivalent: from transformers import pipeline
// classifier = pipeline("sentiment-analysis")
// result = classifier("I loved this movie!")
// → {'label': 'POSITIVE', 'score': 0.999}
```

The pipeline is simpler (pretrained model does everything), but you
give up interpretability and pay inference costs per request.

## Key takeaways

1. **TF-IDF + logistic regression is the 80/20 of sentiment analysis** —
   88% accuracy with zero deep learning complexity.
2. **Negation handling** (simple "not" → "NOT_" prefixing) is a cheap
   win that bag-of-words otherwise misses.
3. **Transformers buy ~5% accuracy** at the cost of GPU inference —
   worth it for high-stakes or high-volume applications.
4. **Interpretability matters in production** — knowing _why_ a review
   was flagged helps debug and build trust.

**Next project:** credit card fraud detection — extreme class imbalance
where the minority class is < 1% of data.
