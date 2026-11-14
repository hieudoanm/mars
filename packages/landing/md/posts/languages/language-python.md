---
title: "Python: The Data Science Standard"
description: "Python became the language of data science almost by accident. Here is why it won, what it costs, and why it still matters."
author: "Mars Team"
date: "2026-11-07"
order: 4
tags: ["python", "data-science", "ecosystem", "history", "mars"]
---

# Why Python?

Python was not designed for data science. It was designed in the late 1980s as a general-purpose scripting language — readable, batteries-included, friendly to beginners. It became the default language for scientific computing anyway, and the reasons say a lot about what people actually need from a tool.

## Readability as a feature

Python's most famous design choice is that it uses whitespace for structure. There are no curly braces, no `end` keywords, no semicolons. A function looks like what it does:

```python
def moving_average(values, window):
    totals = []
    for i in range(len(values)):
        chunk = values[max(0, i - window + 1) : i + 1]
        totals.append(sum(chunk) / len(chunk))
    return totals
```

For a language people read as much as they write — in notebooks, in tutorials, in code review — that matters more than it looks. Python lowered the barrier to entry so far that "I can read this" became "I can write this."

## The ecosystem did the rest

A language is only as useful as its libraries, and Python's ecosystem compounded over three decades:

- **NumPy** made array math fast by pushing the hot loops into C.
- **pandas** made tabular data feel like a first-class object.
- **scikit-learn, PyTorch, TensorFlow** made machine learning accessible to people who are not compiler engineers.
- **Jupyter** gave all of it an interactive home.

Each layer made the next one easier to adopt. By the time a serious alternative appeared, Python had a decade of tutorials, Stack Overflow answers, and institutional knowledge behind it.

## What Python costs

The same properties that made Python approachable create real structural costs:

- **The interpreter.** Python executes bytecode through a virtual machine. A tight loop in pure Python is slow — so slow that numerical code quietly delegates to C extensions. The "Python" in a data science stack is often a thin orchestration layer over native code.
- **The GIL.** The Global Interpreter Lock means only one thread executes Python bytecode at a time. Parallelism requires multiprocessing (with its own memory and serialization costs) or a different runtime entirely.
- **Dynamic typing.** Types are checked at runtime, so a wrong assumption surfaces as an exception mid-analysis — often after expensive computation has already run.
- **Dependency weight.** A "simple" Python environment is rarely simple. Versions conflict, binaries need compiling, and reproducing someone else's analysis means reproducing their entire environment.

None of this made Python a bad choice — it made it a *specific* choice, optimized for approachability and ecosystem breadth rather than raw performance or deployment simplicity.

## Where Mars fits

Mars does not ask you to leave Python behind. It imports and exports `.ipynb` notebooks, so existing Python work keeps working. What Mars offers is a notebook where the *code itself* is compiled Rust — for the cells where you want native speed, memory safety, and a single reproducible binary.

Python earned its place as the glue of data science. Mars is for the parts that deserve a sharper tool.
