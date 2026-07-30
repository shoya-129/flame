# Concurrency and Async in Flame

Flame provides a powerful model for concurrent programming using `async`, `await`, and `spawn`. Under the hood, these concepts map directly to Rust's lightweight futures and tasks.

## The `async` Keyword

You can make a function asynchronous by prefixing it with the `async` keyword. 
An `async` function returns a `Future` instead of blocking the current thread.

```flame
async fn fetch_data(url: String) -> String {
    // perform network request...
    return "data"
}
```

## The `await` Keyword

To wait for an asynchronous operation to complete, use the `await` keyword. You can only use `await` inside an `async` function.
While waiting, `await` allows the executor to run other asynchronous tasks, keeping your application responsive.

```flame
async fn main_flow() {
    let result1 = fetch_data("https://api.example.com/1").await
    let result2 = fetch_data("https://api.example.com/2").await
    print($"Fetched: {result1} and {result2}")
}
```

## Concurrency with `spawn`

To run multiple asynchronous tasks concurrently (simultaneously in the background), you can use `spawn`. `spawn` takes a block of code (or a future) and schedules it to run on the executor immediately, returning a `JoinHandle`.

```flame
async fn process_in_background() {
    let handle1 = spawn {
        fetch_data("https://api.example.com/1").await
    }
    
    let handle2 = spawn {
        fetch_data("https://api.example.com/2").await
    }
    
    // Do some other work while fetch_data runs in the background
    print("Working on something else...")
    
    // Wait for the spawned tasks to finish
    let data1 = handle1.await
    let data2 = handle2.await
    
    print($"All data fetched!")
}
```

## Best Practices

1. **Don't block the async runtime**: If you need to perform heavy CPU-bound computations, do not do them directly in an `async` function. Instead, offload them to a dedicated thread pool to avoid starving other tasks.
2. **Resource safety**: Since tasks can be cancelled or dropped at `.await` points, use `defer` to ensure any acquired resources are always cleaned up properly.
