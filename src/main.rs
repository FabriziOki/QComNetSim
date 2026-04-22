use clap::Parser;
use qcomnetsim::config::SimulationToml;
use qcomnetsim::physics::PhysicsProfile;
use qcomnetsim::simulation::Simulation;
use std::io::Write;
use std::time::Instant;

// ============================================================
// ANSI COLOR CODES
// ============================================================

const GREEN: &str = "\x1b[32m";
const CYAN: &str = "\x1b[36m";
const YELLOW: &str = "\x1b[33m";
const BOLD: &str = "\x1b[1m";
const RESET: &str = "\x1b[0m";

// ============================================================
// CLI DEFINITION
// ============================================================

#[derive(Parser)]
#[command(
    name = "qcomnetsim",
    about = "Quantum network simulator — TOML-driven experiments",
    long_about = None,
    version
)]
struct Cli {
    /// Run a simulation from a TOML configuration file
    #[arg(short, long, value_name = "FILE")]
    config: Option<String>,

    /// List built-in hardware profiles and their key parameters
    #[arg(short, long)]
    profiles: bool,

    /// Write simulation results to a CSV file after the run
    /// (used by the validation engine to collect machine-readable metrics)
    #[arg(short, long, value_name = "FILE")]
    output: Option<String>,
}

// ============================================================
// ENTRY POINT
// ============================================================

fn main() {
    let cli = Cli::parse();

    match (&cli.config, cli.profiles) {
        (Some(path), _) => cmd_run(path, cli.output.as_deref()),
        (None, true) => cmd_profiles(),
        (None, false) => {
            use clap::CommandFactory;
            Cli::command().print_help().unwrap();
            println!();
            std::process::exit(0);
        }
    }
}

// ============================================================
// --config
// ============================================================

fn cmd_run(config_path: &str, output_path: Option<&str>) {
    let toml_cfg = SimulationToml::from_file(config_path).unwrap_or_else(|e| {
        eprintln!("{}Error:{} {}", BOLD, RESET, e);
        std::process::exit(1);
    });

    let resolved = toml_cfg.resolve().unwrap_or_else(|e| {
        eprintln!("{}Error:{} {}", BOLD, RESET, e);
        std::process::exit(1);
    });

    // Header
    let topo = &toml_cfg.topology;
    println!("{BOLD}QComNetSim{RESET}  {CYAN}{config_path}{RESET}\n");
    println!("  Platform  {GREEN}{}{RESET}", resolved.profile.name);
    println!(
        "  Topology  {:?} · {} nodes · {:.1} km · {:.2} dB/km",
        topo.kind, topo.nodes, topo.distance_km, topo.attenuation_db_per_km
    );
    println!("  Path      {:?}", resolved.path);
    println!(
        "  Target    {} pairs\n",
        resolved.sim_config.target_pairs
    );

    // Build simulation and attach progress callback
    let mut sim = Simulation::new(
        resolved.topology,
        resolved.gen_protocol,
        resolved.swap_protocol,
        resolved.sim_config,
        resolved.path,
    );

    let target = toml_cfg.simulation.target_pairs;
    let start = Instant::now();

    sim.set_progress_callback(move |done, total| {
        draw_progress(done, total, start.elapsed().as_secs_f64());
    });

    // Run
    sim.run();

    // Print final bar at 100% then newline before stats
    draw_progress(
        sim.stats.pairs_delivered,
        target,
        start.elapsed().as_secs_f64(),
    );
    println!();

    sim.stats.print_summary();

    if let Some(path) = output_path {
        write_output_csv(&sim.stats, start.elapsed().as_secs_f64() * 1000.0, path);
    }
}

// ============================================================
// --output  (machine-readable results for validation engine)
// ============================================================

/// Write a one-row CSV consumed by the Python validation engine.
///
/// Columns match the `SimulatorRunner` base schema:
///   success_rate  — generation_successes / generation_attempts
///   avg_fidelity  — mean fidelity of delivered pairs
///   throughput    — pairs/second using simulated time (last delivery time)
///   memory_used   — delivered pair count (pairs, not individual qubits)
///   wall_clock_ms — real elapsed time for the full run
fn write_output_csv(
    stats: &qcomnetsim::simulation::SimulationStats,
    wall_clock_ms: f64,
    path: &str,
) {
    let success_rate = stats.generation_success_rate();
    let avg_fidelity = stats.mean_fidelity().unwrap_or(0.0);
    let last_t_ms = stats.delivery_times_ms.iter().cloned().fold(0.0_f64, f64::max);
    let throughput = if last_t_ms > 0.0 {
        stats.pairs_delivered as f64 / (last_t_ms / 1000.0)
    } else {
        0.0
    };
    let memory_used = stats.pairs_delivered;

    let mut f = std::fs::File::create(path).unwrap_or_else(|e| {
        eprintln!("{}Warning:{} cannot write output file {path}: {e}", BOLD, RESET);
        std::process::exit(1);
    });
    writeln!(f, "success_rate,avg_fidelity,throughput,memory_used,wall_clock_ms").unwrap();
    writeln!(
        f,
        "{success_rate:.6},{avg_fidelity:.6},{throughput:.6},{memory_used},{wall_clock_ms:.3}"
    )
    .unwrap();
}

// ============================================================
// PROGRESS BAR
// ============================================================

/// Draws a single in-place progress bar on the current line.
/// Call `println!()` after the final draw to advance past it.
fn draw_progress(done: usize, total: usize, elapsed_s: f64) {
    const BAR_WIDTH: usize = 32;

    let fraction = if total == 0 { 1.0 } else { done as f64 / total as f64 };
    let filled = ((fraction * BAR_WIDTH as f64) as usize).min(BAR_WIDTH);
    let pct = (fraction * 100.0) as usize;

    // Build the bar string
    let bar: String = if filled == BAR_WIDTH {
        "=".repeat(BAR_WIDTH)
    } else if filled == 0 {
        " ".repeat(BAR_WIDTH)
    } else {
        format!("{}{}", "=".repeat(filled - 1), ">")
    };
    let empty = BAR_WIDTH - filled;

    print!(
        "\r  {GREEN}[{bar}{blank}]{RESET}  {CYAN}{done}/{total}  {pct}%{RESET}  {YELLOW}{elapsed:.2}s{RESET}  ",
        blank = " ".repeat(empty),
        elapsed = elapsed_s,
    );
    std::io::stdout().flush().unwrap();
}

// ============================================================
// --profiles
// ============================================================

fn cmd_profiles() {
    let profiles = [
        PhysicsProfile::erbium(),
        PhysicsProfile::nv_center(),
        PhysicsProfile::ideal(),
    ];

    println!("{BOLD}Built-in hardware profiles{RESET}\n");
    println!(
        "{CYAN}{:<14}  {:>13}  {:>10}  {:>10}  {:>13}  {:>10}{RESET}",
        "Profile", "Coherence(ms)", "Mem. eff.", "Det. eff.", "Gate fidelity", "BSM (swap)"
    );
    println!("{}", "-".repeat(76));
    for p in &profiles {
        println!(
            "{:<14}  {:>13.1}  {:>10.2}  {:>10.2}  {:>13.3}  {:>10.2}",
            p.name,
            p.coherence_time_ms,
            p.memory_efficiency,
            p.detector_efficiency,
            p.gate_fidelity,
            p.swap_bsm_efficiency,
        );
    }

    println!();
    println!("Use in a config file:");
    println!("  {CYAN}[hardware]{RESET}");
    println!("  profile = {GREEN}\"erbium\"{RESET}");
    println!();
    println!("Override individual fields on top of a preset:");
    println!("  {CYAN}[hardware]{RESET}");
    println!("  profile = {GREEN}\"erbium\"{RESET}");
    println!("  coherence_time_ms = {YELLOW}500.0{RESET}");
}
