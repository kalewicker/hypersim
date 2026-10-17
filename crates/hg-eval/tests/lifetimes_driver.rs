//! Offline driver for the handle-lifetime instrument (v5 §5, R4):
//! writes `lifetimes.csv` into each run dir named in LIFETIMES_RUN_DIRS
//! (colon-separated; each must contain `snapshots/`). Run with
//! --ignored. Companion to `chains::census_run_dir` — same offline
//! discipline, no live observer.

#[test]
#[ignore]
fn write_lifetimes_for_run_dirs() {
    let dirs = std::env::var("LIFETIMES_RUN_DIRS")
        .expect("LIFETIMES_RUN_DIRS=dir1:dir2:… (run dirs containing snapshots/)");
    for dir in dirs.split(':').filter(|s| !s.is_empty()) {
        let rows = hg_eval::lifetimes::lifetimes_run_dir(std::path::Path::new(dir))
            .unwrap_or_else(|e| panic!("{dir}: {e}"));
        let open = rows.iter().filter(|r| r.death_step.is_none()).count();
        let max_len = rows.iter().map(|r| r.max_length).max().unwrap_or(0);
        println!(
            "{dir} | handles {} | open-at-end {open} | max_length {max_len} | bound {}",
            rows.len(),
            rows.first().map(|r| r.resolution_bound_steps).unwrap_or(0),
        );
    }
}
