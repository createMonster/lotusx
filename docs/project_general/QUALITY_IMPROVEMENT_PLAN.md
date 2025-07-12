# LotusX Quality Improvement Plan

**Objective**: To elevate the LotusX framework from an architecturally excellent project to a production-grade, market-leading solution that is robust, easy to use, and fully-featured.

This plan outlines actionable steps to enhance developer experience, production readiness, and core functionality.

---

## 1. Developer Experience & Onboarding

The project's internal quality is high, but its adoption depends on how easily new developers can use it. The goal is to reduce the time-to-first-bot from hours to minutes.

*   **Action 1.1: Create a "Quick Start" Guide.**
    *   **Problem**: The learning curve for a new user is steep. They must read multiple documents and examples to understand how to build a simple application.
    *   **Solution**: Add a top-level `QUICK_START.md`. This guide should provide a complete, copy-pasteable example of a simple trading bot (e.g., one that fetches the price of BTCUSDT from Binance and prints it every 5 seconds). It should cover project setup, configuration, and execution in less than 50 lines of code.

*   **Action 1.2: Implement a Unified Configuration System.**
    *   **Problem**: Relying solely on environment variables is cumbersome for strategies involving multiple exchanges, each requiring several variables.
    *   **Solution**: Introduce a file-based configuration system (e.g., `config.toml`). This would allow a user to define all their exchange connections, API keys, and strategy parameters in a single, version-controllable file, dramatically simplifying deployment.

*   **Action 1.3: Develop a Small CLI Tool.**
    *   **Problem**: Simple tasks like validating API keys or checking exchange status currently require writing a Rust program.
    *   **Solution**: Create a companion CLI tool (`lotusx-cli`) that provides utility functions. This would serve as both a useful tool and a practical demonstration of the framework's capabilities.
        *   `lotusx-cli check-keys --exchange binance`
        *   `lotusx-cli status --exchange hyperliquid`
        *   `lotusx-cli get-ticker --exchange bybit --symbol BTCUSDT`

---

## 2. Production Readiness & Observability

For a trading system, what you can't see can hurt you. World-class observability is a requirement for production deployment.

*   **Action 2.1: Integrate Metrics and Tracing into the Kernel.**
    *   **Problem**: The system currently lacks deep, structured observability.
    *   **Solution**: Instrument the kernel's `RestClient` and `WsSession` traits using the `tracing` and `metrics` crates. This will provide automatic, system-wide visibility into every API call and WebSocket event with minimal overhead.
    *   **Metrics to Capture**: `http_request_latency_seconds`, `http_requests_total{status_code, exchange}`, `websocket_messages_total{exchange, direction}`, `websocket_connection_status`.

*   **Action 2.2: Implement Granular, Structured Error Handling.**
    *   **Problem**: The current `ExchangeError` enum is good but can be made more granular. A generic `ApiError` hides the specific reason for failure.
    *   **Solution**: Use `thiserror` to create detailed, exchange-specific error enums that map directly to official exchange error codes (e.g., Binance's `-2010` for insufficient funds). This allows the calling application to build resilient logic that can programmatically react to different failure modes.

*   **Action 2.3: Add a `health_check` Method.**
    *   **Problem**: There is no standardized way to confirm that a configured connector is operational.
    *   **Solution**: Add an `async fn health_check(&self) -> Result<(), ExchangeError>` method to the `ExchangeConnector` trait. This method should perform a simple, authenticated API call (like fetching permissions) to validate connectivity and credentials.

---

## 3. Core Feature Enhancement

To support more sophisticated strategies, the core trading and data features must be expanded.

*   **Action 3.1: Implement Advanced Order Execution.**
    *   **Problem**: The `OrderPlacer` trait is basic and lacks features required for HFT.
    *   **Solution**: Extend the `OrderRequest` struct and `OrderPlacer` trait to support:
        *   **Time-in-Force Flags**: `IOC` (Immediate-Or-Cancel) and `FOK` (Fill-Or-Kill).
        *   **Execution Flags**: `Post-Only` to ensure the order is a maker order.
        *   **Batch Operations**: `place_batch_orders` and `cancel_batch_orders` methods to reduce round-trip latency.

*   **Action 3.2: Provide a Live Order Book Utility.**
    *   **Problem**: The WebSocket streams provide order book *diffs*, but the user is responsible for reconstructing and maintaining the live order book.
    *   **Solution**: Create a utility struct, `LiveOrderBook`, that subscribes to a WebSocket stream and maintains a consistent, real-time view of the order book. This is a high-value utility that saves every user from implementing the same complex and error-prone logic.

*   **Action 3.3: Complete the Kernel Architecture Migration.**
    *   **Problem**: Not all exchanges (e.g., Bybit) have been refactored to the new kernel architecture, leading to inconsistency.
    *   **Solution**: Prioritize the refactoring of all remaining legacy connectors. This is a prerequisite for achieving the full quality and maintenance benefits of the kernel design.

---

## 4. Testing & Validation Strategy

A robust testing strategy is the foundation of trust for a financial framework.

*   **Action 4.1: Implement End-to-End (E2E) Scenario Tests.**
    *   **Problem**: Integration tests primarily validate individual API calls.
    *   **Solution**: Create a new E2E test suite that simulates a complete trading lifecycle: fetch markets, place a small limit order, poll until the order is open, cancel the order, and verify the final state. This provides a much higher level of confidence than simple unit tests.

*   **Action 4.2: Develop a Mock Exchange Server for CI.**
    *   **Problem**: The current test suite relies on live exchange testnets, which can be unreliable and cannot easily simulate all error conditions.
    *   **Solution**: Use a tool like `wiremock-rs` to create a mock exchange server. This will allow for deterministic testing of various scenarios in CI, including:
        *   Rate-limiting responses (HTTP 429).
        *   Server errors (HTTP 503).
        *   Malformed JSON payloads.
        *   Authentication failures.

*   **Action 4.3: Introduce Property-Based Testing.**
    *   **Problem**: The `conversions.rs` modules are critical for correctness but are only tested with a few hand-picked examples.
    *   **Solution**: Use a crate like `proptest` to perform property-based testing on all data conversion functions. This will test them against a massive range of automatically generated inputs, uncovering edge cases that manual testing would miss.
