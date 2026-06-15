# HFT-type Rust Coding Cheatsheet

## Error Handling
| ✅ | Guideline                                                                                                        | Rationale                                                                  |
| - | ---------------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------- |
| ☐ | **Return `Result<T, E>` for every fallible boundary** (I/O, parsing, timeouts, arithmetic overflow).             | Makes the contract explicit; callers must decide a policy.                 |
| ☐ | **Reserve `panic!` for violated invariants only** (impossible states, `unsafe` contract breaches).               | Panics stall engines; external data should never trigger them.             |
| ☐ | **Define one typed error enum per subsystem** and derive `thiserror::Error`; wrap foreign errors with `#[from]`. | Exhaustive pattern-matching, zero-cost `?` conversions.                    |
| ☐ | **Attach context once** on the way out (`.context("pair={pair}, seq={seq}")`).                                   | A single, lightweight breadcrumb → quick root-cause without megabyte logs. |
| ☐ | **Library crates stay strongly typed; top-level binary collapses to `anyhow::Error`.**                           | Downstream code gets rich variants; CLI/service prints one tidy line.      |
| ☐ | **Log *or* propagate—never both**; designate the outer event loop as the sole log/metric sink.                   | Prevents duplicate noise and skewed alert counts.                          |
| ☐ | **Bounded retries with back-off** for network errors; **fail fast** on logic errors.                             | Avoids livelocks while keeping the engine alive under packet loss.         |
| ☐ | **Mark cold error paths** `#[cold] #[inline(never)]`; keep happy path in the I-cache.                            | Minimises branch-prediction and cache pollution.                           |
| ☐ | **Avoid heap in hot loops**—stack enums; delay `format!` until you actually log.                                 | Maintains micro-second P99 latency.                                        |
| ☐ | **Instrument every propagated error with `tracing` spans/fields** and increment Prometheus counters.             | Real-time visibility → quicker post-mortems and tail-latency tuning.       |

