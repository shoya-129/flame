use crate::lexer::Lexer;
use crate::parser::{Parser, Stmt};
use crate::runner::Runner;

pub struct TestStats {
    pub passed: usize,
    pub failed: usize,
    pub ignored: usize,
    pub measured: usize,
    pub filtered: usize,
}

pub fn execute_test_suite(runner: &mut Runner, stmts: &[Stmt], filename: &str) -> TestStats {
    let mut stats = TestStats {
        passed: 0,
        failed: 0,
        ignored: 0,
        measured: 0,
        filtered: 0,
    };

    println!("\nrunning tests in \x1b[1m{}\x1b[0m:", filename);

    let mut before_all = Vec::new();
    let mut after_all = Vec::new();
    let mut setup = Vec::new();
    let mut cleanup = Vec::new();
    let mut test_cases = Vec::new();
    let mut has_only_test = false;

    for stmt in stmts {
        if let Stmt::FuncDecl {
            name, annotations, ..
        } = stmt
        {
            for anno in annotations {
                match anno.name.as_str() {
                    "BeforeAll" => before_all.push(name.clone()),
                    "AfterAll" => after_all.push(name.clone()),
                    "Setup" => setup.push(name.clone()),
                    "Cleanup" => cleanup.push(name.clone()),
                    "Test" | "test" | "Benchmark" | "benchmark" | "Parameterized" | "parameterized" | "ExpectPanic" | "expect_panic" | "Ignore" | "ignore"
                    | "Only" | "only" => {
                        if !test_cases.contains(&name.clone()) {
                            test_cases.push(name.clone());
                        }
                        if anno.name == "Only"
                            || anno
                                .args
                                .iter()
                                .any(|arg| arg.contains("only: true") || arg == "only: true")
                        {
                            has_only_test = true;
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    for func_name in &before_all {
        let func_opt = runner.env.lock().unwrap().get(func_name);
        if let Some(func_val) = func_opt {
            if let Err(e) = runner.invoke_callback_value(&func_val, vec![]) {
                println!("  \x1b[1;31m[FAIL]\x1b[0m \x1b[1;36m@BeforeAll\x1b[0m {}", func_name);
                let span = runner.current_span.clone().unwrap_or(crate::lexer::Span { start: 0, end: 0, line: 1, col: 1 });
                crate::diagnostics::Diagnostic::new_error(e, runner.filepath.display().to_string(), span, None, None).print(&std::fs::read_to_string(&runner.filepath).unwrap_or_default());
                stats.failed += 1;
                return stats;
            }
        }
    }

    for func_name in &test_cases {
        let mut is_ignore = false;
        let mut is_only = false;
        let mut is_benchmark = false;
        let mut is_expect_panic = false;
        let mut parameterized_args = None;

        for stmt in stmts {
            if let Stmt::FuncDecl {
                name, annotations, ..
            } = stmt
            {
                if name == func_name {
                    for anno in annotations {
                        match anno.name.as_str() {
                            "Ignore" | "ignore" => is_ignore = true,
                            "Only" | "only" => is_only = true,
                            "Benchmark" | "benchmark" => is_benchmark = true,
                            "ExpectPanic" | "expect_panic" => is_expect_panic = true,
                            "Parameterized" | "parameterized" => {
                                if !anno.args.is_empty() {
                                    parameterized_args = Some(anno.args[0].clone());
                                }
                            }
                            "Test" | "test" => {
                                if anno.args.iter().any(|arg| arg.contains("skip: true")) {
                                    is_ignore = true;
                                }
                                if anno.args.iter().any(|arg| arg.contains("only: true")) {
                                    is_only = true;
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
        }

        if has_only_test && !is_only {
            stats.filtered += 1;
            continue;
        }

        if is_ignore {
            println!(
                "  \x1b[33m[SKIP]\x1b[0m \x1b[1;36m@Ignore\x1b[0m {}",
                func_name
            );
            stats.ignored += 1;
            continue;
        }

        for setup_name in &setup {
            let setup_opt = runner.env.lock().unwrap().get(setup_name);
            if let Some(s_val) = setup_opt {
                let _ = runner.invoke_callback_value(&s_val, vec![]);
            }
        }

        let test_func_opt = runner.env.lock().unwrap().get(func_name);
        if let Some(f_val) = test_func_opt {
            if is_benchmark {
                let mut durations = Vec::new();
                let mut benchmark_failed = false;
                for _ in 0..25 {
                    let start = std::time::Instant::now();
                    if let Err(e) = runner.invoke_callback_value(&f_val, vec![]) {
                        println!("  \x1b[1;31m[FAIL]\x1b[0m \x1b[1;36m@Benchmark\x1b[0m {}", func_name);
                        let span = runner.current_span.clone().unwrap_or(crate::lexer::Span { start: 0, end: 0, line: 1, col: 1 });
                        crate::diagnostics::Diagnostic::new_error(e, runner.filepath.display().to_string(), span, None, None).print(&std::fs::read_to_string(&runner.filepath).unwrap_or_default());
                        stats.failed += 1;
                        benchmark_failed = true;
                        break;
                    }
                    durations.push(start.elapsed().as_secs_f64() * 1000.0);
                }
                if !benchmark_failed && !durations.is_empty() {
                    let avg = durations.iter().sum::<f64>() / durations.len() as f64;
                    let min = durations.iter().fold(f64::INFINITY, |a, &b| a.min(b));
                    let max = durations.iter().fold(0.0_f64, |a, &b| a.max(b));
                    println!(
                        "  \x1b[1;32m[PASS]\x1b[0m \x1b[1;36m@Benchmark\x1b[0m {}",
                        func_name
                    );
                    println!("    Benchmark: {}", func_name);
                    println!("    -----------");
                    println!("    average: {:.2} ms", avg);
                    println!("    min: {:.2} ms", min);
                    println!("    max: {:.2} ms", max);
                    stats.measured += 1;
                }
            } else if let Some(arg_str) = parameterized_args {
                let mut l = Lexer::new(&arg_str);
                let mut tok_vec = Vec::new();
                loop {
                    let tok = l.next_token();
                    let e = tok.kind == crate::lexer::TokenKind::EOF;
                    tok_vec.push(tok);
                    if e {
                        break;
                    }
                }
                let mut p = Parser::new(tok_vec, "param_arg".to_string());
                if let Ok(expr) = p.parse_expr() {
                    let env_clone = runner.env.clone();
                    if let Ok(evaled) = runner.eval_expr(&expr, env_clone) {
                        let list = match evaled {
                            crate::vm::Value::Tuple(vec_val) => vec_val.clone(),
                            other => vec![other],
                        };
                        let mut all_ok = true;
                        let start = std::time::Instant::now();
                        for case in &list {
                            let call_args = match case {
                                crate::vm::Value::Tuple(tup) => tup.clone(),
                                single => vec![single.clone()],
                            };
                            if let Err(e) = runner.invoke_callback_value(&f_val, call_args) {
                                println!("  \x1b[1;31m[FAIL]\x1b[0m \x1b[1;36m@Parameterized\x1b[0m {} on argument {:?}", func_name, case);
                                let span = runner.current_span.clone().unwrap_or(crate::lexer::Span { start: 0, end: 0, line: 1, col: 1 });
                                crate::diagnostics::Diagnostic::new_error(e, runner.filepath.display().to_string(), span, None, None).print(&std::fs::read_to_string(&runner.filepath).unwrap_or_default());
                                all_ok = false;
                                break;
                            }
                        }
                        if all_ok {
                            println!(
                                "  \x1b[1;32m[PASS]\x1b[0m \x1b[1;36m@Parameterized\x1b[0m {} ({} parameter cases in {:.2}ms)",
                                func_name,
                                list.len(),
                                start.elapsed().as_secs_f64() * 1000.0
                            );
                            stats.passed += 1;
                        } else {
                            stats.failed += 1;
                        }
                    } else {
                        println!(
                            "  \x1b[1;31m[FAIL]\x1b[0m \x1b[1;36m@Parameterized\x1b[0m {}: failed to evaluate parameter argument expression",
                            func_name
                        );
                        stats.failed += 1;
                    }
                }
            } else {
                let start = std::time::Instant::now();
                let res = runner.invoke_callback_value(&f_val, vec![]);
                let elapsed = start.elapsed().as_secs_f64() * 1000.0;
                if is_expect_panic {
                    match res {
                        Err(e) => {
                            println!("  \x1b[1;32m[PASS]\x1b[0m \x1b[1;36m@ExpectPanic\x1b[0m {} (expected panic occurred in {:.2}ms: {})", func_name, elapsed, e);
                            stats.passed += 1;
                        }
                        Ok(_) => {
                            println!(
                                "  \x1b[1;31m[FAIL]\x1b[0m \x1b[1;36m@ExpectPanic\x1b[0m {}: function completed without expected error/panic!",
                                func_name
                            );
                            stats.failed += 1;
                        }
                    }
                } else {
                    match res {
                        Ok(_) => {
                            println!(
                                "  \x1b[1;32m[PASS]\x1b[0m \x1b[1;36m@Test\x1b[0m {} ({:.2}ms)",
                                func_name, elapsed
                            );
                            stats.passed += 1;
                        }
                        Err(e) => {
                            println!("  \x1b[1;31m[FAIL]\x1b[0m \x1b[1;36m@Test\x1b[0m {}", func_name);
                            let span = runner.current_span.clone().unwrap_or(crate::lexer::Span { start: 0, end: 0, line: 1, col: 1 });
                            crate::diagnostics::Diagnostic::new_error(e, runner.filepath.display().to_string(), span, None, None).print(&std::fs::read_to_string(&runner.filepath).unwrap_or_default());
                            stats.failed += 1;
                        }
                    }
                }
            }
        }

        for cleanup_name in &cleanup {
            let cleanup_opt = runner.env.lock().unwrap().get(cleanup_name);
            if let Some(c_val) = cleanup_opt {
                let _ = runner.invoke_callback_value(&c_val, vec![]);
            }
        }
    }

    for func_name in &after_all {
        let after_opt = runner.env.lock().unwrap().get(func_name);
        if let Some(func_val) = after_opt {
            let _ = runner.invoke_callback_value(&func_val, vec![]);
        }
    }

    stats
}
