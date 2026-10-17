# Run State Machine Specification

This document defines the states, transitions, and invariants of a simulation run in Hypersim.

## States

| State | Description |
|---|---|
| `Created` | Run ID assigned; output directory created; CSV header written; manifest.json created; engine not yet started. |
| `Running` | Engine stepping; observers firing; CSV appended each evaluation window. |
| `Paused` | Engine stopped between steps; CSV flushed; can resume. |
| `Interrupted` | Stopped by user Reset/Quit, or by unrecoverable error; CSV flushed and sealed; partial report written. |
| `Completed` | Reached `max_steps` or natural halt condition; CSV sealed; final report written. |

## Transitions

```
Created     → Running      : Play command
Running     → Paused       : Pause command
Paused      → Running      : Resume command
Paused      → Interrupted  : Reset (while paused)
Running     → Interrupted  : Reset (while running), or unrecoverable error
Running     → Completed    : step == max_steps, or halt condition met
Interrupted → Created      : Reset creates a NEW run_id, fresh CSV
Completed   → Created      : Reset creates a NEW run_id, fresh CSV
```

### Transition Details

**Created → Running (Play)**
- Start simulation thread
- Begin stepping engine
- Observers begin firing
- CSV rows appended per evaluation window

**Running → Paused (Pause)**
- Signal simulation thread to stop at step boundary
- Wait for thread to acknowledge pause
- Flush CSV (ensure current window row is written)
- Update shared state to `Paused`

**Paused → Running (Resume)**
- Signal simulation thread to continue
- Update shared state to `Running`

**Paused → Interrupted (Reset while paused)**
- Seal current run as `Interrupted`
- Write partial report
- Generate new run_id
- Create new output directory
- Write new CSV header
- Enter `Created` state

**Running → Interrupted (Reset while running)**
- Signal simulation thread to stop immediately
- Wait for thread to exit
- Flush CSV
- Seal current run as `Interrupted`
- Write partial report
- Generate new run_id
- Create new output directory
- Write new CSV header
- Enter `Created` state

**Running → Interrupted (Unrecoverable error)**
- Simulation thread catches panic or returns error
- Flush CSV
- Seal current run as `Interrupted`
- Write partial report with error message
- GUI displays error state

**Running → Completed**
- Step counter reaches `max_steps`
- Or halt condition met (no matches for N consecutive steps, graph empty, etc.)
- Flush final CSV row
- Seal CSV
- Write final report
- Update shared state to `Completed`

**Interrupted → Created (Reset)**
- Generate new run_id (timestamp + seed)
- Create new output directory
- Write new CSV header
- Create new manifest.json
- Enter `Created` state

**Completed → Created (Reset)**
- Same as above: new run_id, new directory, new CSV

## Invariants

1. **Run directory is never overwritten**: Each `run_id` is unique (format: `<experiment>_<seed>_<timestamp>`). A new run always creates a new directory.

2. **Interrupted runs are preserved**: An `Interrupted` run's directory and partial CSV are preserved forever. They can be loaded post-hoc in the GUI.

3. **Interrupted runs cannot be resumed**: `Reset` always starts fresh with a new `run_id`. There is no "continue from checkpoint" in this version.

4. **Completed and Interrupted runs can be loaded post-hoc**: The GUI can open any run directory and display its CSV, report, knowledge graph, and snapshots.

5. **CSV is sealed on terminal states**: Both `Completed` and `Interrupted` seal the CSV (no further writes). The manifest is updated with end state and timestamp.

6. **Single CSV per run**: Exactly one CSV file exists per run directory. Named `metrics.csv`.

## State in Shared Memory

The `RunState` enum is stored in shared `Arc<RwLock<SimProgress>>` and updated by the simulation thread:

```rust
pub enum RunState { Created, Running, Paused, Interrupted, Completed }

pub struct SimProgress {
    pub step:       u64,
    pub max_steps:  u64,
    pub run_state:  RunState,
    pub sim_hz:     f32,    // steps per second, rolling average
}
```

The GUI reads `SimProgress` at 30 Hz for the status bar. No channel needed for step counting.

## CLI Behavior

**Headless run (`hypersim-cli`)**:
- Starts in `Created`
- Immediately transitions to `Running`
- Runs to `Completed` or `Interrupted` (on error)
- Exits with code 0 on `Completed`, non-zero on `Interrupted` (error)

**Campaign run**:
- Each condition × seed combination is a separate run with its own state machine
- Runs execute in parallel (Rayon) up to `parallel` limit
- Each produces independent output directory

## GUI Behavior

- Toolbar buttons reflect current state (Play enabled in Created/Paused, Pause in Running, etc.)
- Status bar shows `RunState` with colored indicator
- Reset button: confirms if running, then transitions to Interrupted → Created
- Close window: if Running/Paused, seals as Interrupted first