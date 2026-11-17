---
title: 'R: Statistical Computing and Graphics'
description: 'R was built by statisticians, for statisticians. It still owns a huge share of academic research and data analysis — and it has real strengths worth respecting.'
author: 'Project Mars Team'
date: '2026-11-07'
order: 5
tags: ['r', 'statistics', 'data-science', 'tidyverse', 'mars']
---

# Why R?

Python gets most of the attention in data science, but a large share of academic research, biostatistics, and social science still runs on R. That is not inertia — R was designed by statisticians for statistical work, and in that domain it remains genuinely excellent.

## Built for statistics, not general-purpose code

R's core data structures are statistical objects. A data frame is not a library add-on; it is a native concept. The language was designed around the workflow of applied statistics:

- **Vectorized operations by default.** `x <- c(1, 2, 3)` and `mean(x)` work on whole vectors without loops. Statistical thinking maps directly onto the language.
- **The formula interface.** `lm(y ~ x1 + x2, data = df)` reads like the statistical model it fits. The syntax encodes domain knowledge.
- **Everything is an object.** Models, plots, and tables carry their own print and summary methods, so results display the way a statistician expects.

For someone who thinks in terms of distributions, regressions, and hypothesis tests, R's abstractions line up with the work.

## The tidyverse changed the language's ergonomics

For years R's biggest weakness was ergonomics — base R syntax was idiosyncratic and inconsistent. The **tidyverse** fixed that. `dplyr`, `ggplot2`, `tidyr`, and `readr` gave R a coherent, pipe-friendly data workflow:

```r
library(dplyr)
library(ggplot2)

storms %>%
  filter(wind > 50) %>%
  group_by(year) %>%
  summarise(avg_pressure = mean(pressure)) %>%
  ggplot(aes(x = year, y = avg_pressure)) +
  geom_line()
```

The tidyverse made R feel modern without changing what it is good at: fast, expressive data manipulation and publication-quality graphics out of the box.

## Where R struggles

Honesty requires acknowledging R's structural weaknesses:

- **Performance.** R is an interpreted language with copy-on-modify semantics. Large datasets mean memory copies; heavy loops mean slow execution. The standard answer is to drop into C++ via `Rcpp`, which works but breaks the illusion that you are writing pure R.
- **Memory model.** Everything is held in memory, and copies are common. A 10 GB dataset on a 16 GB machine is a real problem, not a theoretical one.
- **Package fragmentation.** R has multiple package ecosystems (CRAN, Bioconductor, r-universe) with their own conventions, and dependency conflicts are a recurring headache.
- **Deployment.** Shipping an R analysis means shipping an R environment. There is no equivalent of a single compiled binary for the whole workflow.

These are not reasons to dismiss R — they are reasons R and Python both lean on native extensions for the heavy lifting.

## Try it in Project Mars

R cells share a single R session per notebook. Assignments persist across cells, `cat()`/`print()` stream to cell output, and plots render inline — the same workflow as RStudio, inside a `.mars` document.

```r
# Cells share one R session; bindings persist across cells.
sepal_lengths <- c(5.1, 4.9, 4.7, 4.6, 5.0)

# R is vectorized by default — summary stats are one-liners.
summary(sepal_lengths)
```

```r
# <<- assigns into .GlobalEnv explicitly when needed inside functions.
iris_means <- data.frame(
  feature = c("sepal_length", "sepal_width"),
  mean    = c(mean(sepal_lengths), mean(c(3.5, 3.0, 3.2, 3.1, 3.6)))
)
iris_means
```

## Why Project Mars is thinking about R

Project Mars's document model was designed to be language-agnostic from the start: a notebook is prose, code cells, and outputs, and the metadata declares which language the code is written in. Supporting R is a natural extension of that design, not a bolt-on.

The case for R in Project Mars is the same case as Python: there is a large community of people whose data lives in R, whose models live in R, and whose notebooks currently force them into Jupyter. Giving them a first-class R runtime — with the same persistent sessions, rich outputs, and `.mars` reproducibility — means Project Mars serves the whole data science community, not just the Rust part of it.

Rust remains Project Mars's primary language. But a notebook platform that only speaks one language is a tool with a ceiling. R is the next language on that list, and the architecture is already shaped to accept it.
