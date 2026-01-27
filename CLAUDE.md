# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

sqs-monitor is a terminal-based UI application for monitoring AWS SQS queues in real-time. Built with Rust, it provides a TUI (Terminal User Interface) using ratatui and crossterm.

## Development Commands

### Build & Run
- `cargo build` - Build the project
- `cargo run` - Run the application (requires AWS credentials configured)
- `cargo check` - Fast check for compile errors

### Testing & Linting
- `cargo test` - Run tests
- `cargo clippy` - Run linter
- `cargo fmt` - Format code

## Architecture

### Module Structure

The codebase is organized into five core modules:

1. **app** (`src/app.rs`) - Central application state and logic
   - Manages `App` struct with queue list, selection state, and filtering
   - Handles refresh operations and state mutations
   - Maintains both `all_queues` and filtered `queues` for toggle functionality
   - Auto-refresh interval set to 30 seconds

2. **aws** (`src/aws/`) - AWS SQS client abstraction
   - `SqsClient` wraps AWS SDK operations
   - `list_queues()` fetches all queues with attributes (messages, in-flight, delayed)
   - `get_queue_info()` retrieves basic queue attributes
   - `get_queue_details()` fetches comprehensive queue configuration
   - Uses `aws-config` for credential loading from environment

3. **types** (`src/types.rs`) - Data models
   - `QueueInfo` - Basic queue data with message counts
   - `QueueDetails` - Extended attributes (ARN, retention, visibility timeout, etc.)

4. **events** (`src/events.rs`) - Input handling
   - `AppEvent` enum defines user actions (Quit, Refresh, NextQueue, PreviousQueue, ToggleFilter)
   - `poll_event()` polls keyboard input with timeout
   - Key bindings: q/Esc=quit, r=refresh, f=filter, ↑/↓=navigate, j/k=vim-style navigation

5. **ui** (`src/ui.rs`) - Terminal rendering with ratatui
   - Three-section layout: header (3 lines), main content (flexible), status bar (3 lines)
   - Main content split 40/60: queue list (left) and details panel (right)
   - Color-coded message counts: green (0), yellow (1-100), red (>100)
   - DLQ queues highlighted in magenta
   - Selected queue rendered with cyan background

### Application Flow

1. **Initialization** (`src/main.rs`)
   - Creates async tokio runtime
   - Sets up crossterm terminal (raw mode, alternate screen)
   - Initializes `App` with AWS SQS client
   - Performs initial queue refresh

2. **Main Loop** (`run_app()`)
   - Continuously renders UI with `terminal.draw()`
   - Checks auto-refresh timer (30s interval)
   - Polls for keyboard events with 100ms timeout
   - Processes events and updates app state
   - Refreshes selected queue details on navigation

3. **State Management**
   - Queues sorted by message count (descending) in `app.rs:43`
   - Filter toggling maintains separate `all_queues` and `queues` lists
   - Selection index automatically adjusted when filter changes queue count

## Key Implementation Details

- **Async Runtime**: Uses tokio with "full" feature set for AWS SDK operations
- **Terminal Management**: Properly restores terminal state (raw mode, alternate screen) on exit
- **Error Handling**: Uses `anyhow::Result` throughout for consistent error propagation
- **Time Handling**: `chrono` for timestamps, displayed in local timezone
- **AWS Credentials**: Automatically loaded via `aws_config::load_from_env()` - uses standard AWS credential chain (env vars, ~/.aws/credentials, IAM roles)

## Working with AWS Integration

When modifying AWS functionality:
- All SQS operations are in `src/aws/sqs.rs`
- Queue attributes are fetched using `get_queue_attributes()` with specific `QueueAttributeName` enums
- Error handling should preserve user-facing status messages in `app.status_message`
- New attributes should be added to both `QueueDetails` struct and the parsing logic in `get_queue_details()`

## UI Customization

- Layout constraints defined in `ui::draw()` using ratatui's `Layout` and `Constraint`
- Color scheme uses ratatui's `Color` enum (Cyan, Yellow, Red, Green, Magenta)
- Modify `draw_queue_list()` for queue list styling
- Modify `draw_queue_details()` for details panel content
- Status bar format defined in `draw_status_bar()`
