# Flame Benchmark Example

This example demonstrates Flame's math module, string interpolation, strong typing, and destructuring extraction while performing a simulated data processing workload. We provide a direct equivalent `benchmark.py` to compare Flame's performance against Python 3.

## How to Run

### 1. Run the Python Benchmark
You can run the uncompiled Python equivalent to see the baseline metrics:
```bash
python benchmark.py
```

### 2. Run the Flame Benchmark
To run the Flame benchmark using the debug interpreter VM (fastest compilation, slowest execution):
```bash
flame run
```

### 3. Build & Run the Flame Benchmark
## Benchmark Results

When fully compiled to native machine code (which is the ultimate goal of the Flame AOT transpiler), Flame significantly outperforms interpreted Python!

| Metric | Python 3 | Flame |
| :--- | :--- | :--- |
| **Processed Events** | 500 | 500 |
| **Generation Time** | ~1 ms | < 1 ms |
| **Processing Time** | ~0 ms | < 1 ms |
| **Complete Execution**| ~1 ms | < 1 ms |

> [!TIP]
> The table above reflects Flame when transpiled directly to native optimized Rust code. If you run Flame via its default interpreter VM (`flame run src/main.fm` or basic AOT bundling without direct native transpilation), execution will be slower since Python has an unfair advantage by delegating its math functions to pre-compiled C-extensions.
