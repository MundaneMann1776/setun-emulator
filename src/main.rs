//! Setun Emulator - CLI Entry Point
//!
//! Commands:
//! - `setun-emu run <program>` - Run a TROM or ASM file
//! - `setun-emu debug <program>` - Interactive debugger (Phase 4)
//! - `setun-emu asm <source>` - Assemble to TROM
//! - `setun-emu disasm <trom>` - Disassemble TROM

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "setun-emu")]
#[command(author = "Yigit")]
#[command(version = "0.1.0")]
#[command(about = "A balanced ternary emulator of the Soviet Setun (1958) computer")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Run a program until it halts
    Run {
        /// Path to the TROM or ASM file to execute
        program: String,
        /// Maximum number of cycles to run (default: 10000)
        #[arg(short, long, default_value = "10000")]
        max_cycles: u64,
        /// Show trace output
        #[arg(short, long)]
        trace: bool,
    },
    /// Interactive debugger (coming in Phase 4)
    Debug {
        /// Path to the TROM file to debug
        program: String,
    },
    /// Assemble source to TROM
    Asm {
        /// Path to the source file
        source: String,
        /// Output TROM file
        #[arg(short, long)]
        output: Option<String>,
    },
    /// Disassemble TROM to readable text
    Disasm {
        /// Path to the TROM file
        trom: String,
    },
    /// Run the built-in self-test
    Test,
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Run { program, max_cycles, trace }) => {
            run_program(&program, max_cycles, trace);
        }
        Some(Commands::Debug { program }) => {
            debug_program(&program);
        }
        Some(Commands::Asm { source, output }) => {
            assemble_file(&source, output);
        }
        Some(Commands::Disasm { trom }) => {
            disassemble_file(&trom);
        }
        Some(Commands::Test) => {
            run_self_test();
        }
        None => {
            println!("Setun Emulator v0.1.0");
            println!("A balanced ternary computer emulator");
            println!();
            println!("Use --help for available commands");
            println!();
            demo_ternary_primitives();
        }
    }
}

fn run_program(path: &str, max_cycles: u64, trace: bool) {
    use setun::{Cpu, Tryte9, load_trom, assemble};
    use setun::asm::disasm::disassemble_instruction;
    
    println!("🔧 Running: {}", path);
    
    // Load program (either TROM or ASM)
    let instructions: Vec<Tryte9> = if path.ends_with(".asm") {
        // Assemble first
        let source = match std::fs::read_to_string(path) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("❌ Failed to read file: {}", e);
                std::process::exit(1);
            }
        };
        
        match assemble(&source) {
            Ok(instrs) => {
                println!("📝 Assembled {} instructions", instrs.len());
                instrs
            }
            Err(e) => {
                eprintln!("❌ Assembly error: {}", e);
                std::process::exit(1);
            }
        }
    } else {
        // Load TROM
        match load_trom(path) {
            Ok(trom) => {
                println!("📂 Loaded {} instructions", trom.len());
                trom.instructions
            }
            Err(e) => {
                eprintln!("❌ Failed to load TROM: {}", e);
                std::process::exit(1);
            }
        }
    };
    
    if instructions.is_empty() {
        eprintln!("❌ No instructions to execute");
        std::process::exit(1);
    }
    
    // Create CPU and load program
    let mut cpu = Cpu::new();
    if let Err(e) = cpu.load_program(&instructions) {
        eprintln!("❌ Failed to load program: {}", e);
        std::process::exit(1);
    }
    
    println!();
    println!("━━━ Execution ━━━");
    
    // Run with optional trace
    let mut cycles = 0u64;
    while cpu.is_running() && cycles < max_cycles {
        let pc = cpu.regs.c.to_i32();
        
        match cpu.step() {
            Ok(instr) => {
                if trace {
                    let disasm = disassemble_instruction(setun::cpu::decode::encode(&instr));
                    println!("{:03}: {}  S={} ω={:?}", 
                        pc, disasm, cpu.regs.s.to_i64(), cpu.regs.omega);
                }
                cycles += 1;
            }
            Err(e) => {
                eprintln!("❌ CPU error at PC={}: {}", pc, e);
                std::process::exit(1);
            }
        }
    }
    
    println!();
    println!("━━━ Result ━━━");
    println!("Cycles: {}", cycles);
    println!("State: {:?}", cpu.state);
    println!("S (accumulator): {} ({})", cpu.regs.s, cpu.regs.s.to_i64());
    println!("R (multiplier):  {} ({})", cpu.regs.r, cpu.regs.r.to_i64());
    println!("F (index):       {}", cpu.regs.f.to_i32());
    println!("ω (omega):       {:?}", cpu.regs.omega);
    
    if cycles >= max_cycles {
        println!();
        println!("⚠️  Reached max cycles limit ({}). Use --max-cycles to increase.", max_cycles);
    }
}

fn debug_program(path: &str) {
    use setun::{Tryte9, load_trom, assemble};
    use setun::tui::run_debugger;
    
    println!("🔍 Loading: {}", path);
    
    // Load program (either TROM or ASM)
    let instructions: Vec<Tryte9> = if path.ends_with(".asm") {
        let source = match std::fs::read_to_string(path) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("❌ Failed to read file: {}", e);
                std::process::exit(1);
            }
        };
        
        match assemble(&source) {
            Ok(instrs) => {
                println!("📝 Assembled {} instructions", instrs.len());
                instrs
            }
            Err(e) => {
                eprintln!("❌ Assembly error: {}", e);
                std::process::exit(1);
            }
        }
    } else {
        match load_trom(path) {
            Ok(trom) => {
                println!("📂 Loaded {} instructions", trom.len());
                trom.instructions
            }
            Err(e) => {
                eprintln!("❌ Failed to load TROM: {}", e);
                std::process::exit(1);
            }
        }
    };
    
    if instructions.is_empty() {
        eprintln!("❌ No instructions to execute");
        std::process::exit(1);
    }
    
    println!("🚀 Launching debugger...");
    println!();
    
    if let Err(e) = run_debugger(instructions) {
        eprintln!("❌ Debugger error: {}", e);
        std::process::exit(1);
    }
}

fn assemble_file(source_path: &str, output: Option<String>) {
    use setun::{assemble, save_trom, TromFile};
    
    let out_path = output.unwrap_or_else(|| {
        source_path.replace(".asm", ".trom")
    });
    
    println!("📝 Assembling: {} → {}", source_path, out_path);
    
    // Read source
    let source = match std::fs::read_to_string(source_path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("❌ Failed to read file: {}", e);
            std::process::exit(1);
        }
    };
    
    // Assemble
    let instructions = match assemble(&source) {
        Ok(instrs) => instrs,
        Err(e) => {
            eprintln!("❌ Assembly error: {}", e);
            std::process::exit(1);
        }
    };
    
    println!("✓ Assembled {} instructions", instructions.len());
    
    // Save TROM
    let trom = TromFile {
        instructions: instructions.clone(),
        source_lines: instructions.iter().map(|i| format!("{}", i)).collect(),
    };
    
    if let Err(e) = save_trom(&out_path, &trom) {
        eprintln!("❌ Failed to save TROM: {}", e);
        std::process::exit(1);
    }
    
    println!("✓ Saved to {}", out_path);
}

fn disassemble_file(trom_path: &str) {
    use setun::{load_trom};
    use setun::asm::disasm::disassemble;
    
    println!("📖 Disassembling: {}", trom_path);
    println!();
    
    // Load TROM
    let trom = match load_trom(trom_path) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("❌ Failed to load TROM: {}", e);
            std::process::exit(1);
        }
    };
    
    // Disassemble
    let output = disassemble(&trom.instructions);
    println!("{}", output);
}

fn demo_ternary_primitives() {
    use setun::{Trit, Tryte9, Word18};
    use setun::ternary::arith;
    
    println!("━━━ Balanced Ternary Demo ━━━");
    println!();
    
    println!("Trits (single balanced ternary digits):");
    println!("  N = {:?} = {}", Trit::N, Trit::N.to_i8());
    println!("  O = {:?} = {}", Trit::O, Trit::O.to_i8());
    println!("  P = {:?} = {}", Trit::P, Trit::P.to_i8());
    println!();
    
    println!("Tryte9 (9-trit words, range -9841 to +9841):");
    let a = Tryte9::from_i32(42);
    let b = Tryte9::from_i32(-17);
    println!("  42 in balanced ternary: {}", a);
    println!("  -17 in balanced ternary: {}", b);
    println!();
    
    println!("Word18 arithmetic:");
    let x = Word18::from_i64(12345);
    let y = Word18::from_i64(6789);
    let (sum, _) = arith::add(&x, &y);
    let (diff, _) = arith::subtract(&x, &y);
    let (prod_lo, _) = arith::multiply(&x, &y);
    
    println!("  {} + {} = {}", x.to_i64(), y.to_i64(), sum.to_i64());
    println!("  {} - {} = {}", x.to_i64(), y.to_i64(), diff.to_i64());
    println!("  {} × {} = {}", x.to_i64(), y.to_i64(), prod_lo.to_i64());
    println!();
    
    println!("✓ Core ternary primitives working!");
}

fn run_self_test() {
    use setun::{Trit, Tryte9, Word18, Cpu};
    use setun::ternary::arith;
    use setun::cpu::decode::{Instruction, AddrMode, encode};
    
    println!("━━━ Setun Emulator Self-Test ━━━");
    println!();
    
    let mut passed = 0;
    let mut failed = 0;
    
    // Test 1: Trit negation involution
    print!("Trit negation involution... ");
    let mut ok = true;
    for t in Trit::ALL {
        if t.neg().neg() != t {
            ok = false;
            break;
        }
    }
    if ok { println!("✓"); passed += 1; } 
    else { println!("✗"); failed += 1; }
    
    // Test 2: Conversion roundtrip
    print!("Tryte9 conversion roundtrip... ");
    ok = true;
    for val in [-9841, -100, -1, 0, 1, 100, 9841] {
        if Tryte9::from_i32(val).to_i32() != val {
            ok = false;
            break;
        }
    }
    if ok { println!("✓"); passed += 1; } 
    else { println!("✗"); failed += 1; }
    
    // Test 3: Additive inverse
    print!("Additive inverse (a + -a = 0)... ");
    ok = true;
    for val in [-1000i64, -1, 0, 1, 1000] {
        let a = Word18::from_i64(val);
        let neg_a = arith::negate(&a);
        let (result, _) = arith::add(&a, &neg_a);
        if !result.is_zero() {
            ok = false;
            break;
        }
    }
    if ok { println!("✓"); passed += 1; } 
    else { println!("✗"); failed += 1; }
    
    // Test 4: Multiplication correctness
    print!("Multiplication correctness... ");
    let (prod, _) = arith::multiply(&Word18::from_i64(123), &Word18::from_i64(456));
    if prod.to_i64() == 56088 {
        println!("✓");
        passed += 1;
    } else {
        println!("✗ (got {}, expected 56088)", prod.to_i64());
        failed += 1;
    }
    
    // Test 5: Shift operations
    print!("Shift left (×3) correctness... ");
    let shifted = arith::shift_left(&Word18::from_i64(1), 3);
    if shifted.to_i64() == 27 {
        println!("✓");
        passed += 1;
    } else {
        println!("✗ (got {}, expected 27)", shifted.to_i64());
        failed += 1;
    }
    
    // Test 6: CPU execution
    print!("CPU halt instruction... ");
    let mut cpu = Cpu::new();
    cpu.load_program(&[encode(&Instruction::Hlt)]).unwrap();
    let result = cpu.run();
    if result.is_ok() && cpu.is_halted() {
        println!("✓");
        passed += 1;
    } else {
        println!("✗");
        failed += 1;
    }
    
    // Test 7: CPU arithmetic
    print!("CPU load/add/store... ");
    let mut cpu = Cpu::new();
    cpu.mem.write(91, Tryte9::from_i32(10));
    cpu.mem.write(92, Tryte9::from_i32(5));
    let program = [
        encode(&Instruction::Lda { addr: Tryte9::from_i32(10), mode: AddrMode::Direct }),
        encode(&Instruction::Add { addr: Tryte9::from_i32(11), mode: AddrMode::Direct }),
        encode(&Instruction::Hlt),
    ];
    cpu.load_program(&program).unwrap();
    cpu.run().unwrap();
    if cpu.regs.s.to_i64() == 15 {
        println!("✓");
        passed += 1;
    } else {
        println!("✗ (got {}, expected 15)", cpu.regs.s.to_i64());
        failed += 1;
    }
    
    println!();
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("Results: {} passed, {} failed", passed, failed);
    
    if failed == 0 {
        println!("✓ All tests passed!");
    } else {
        std::process::exit(1);
    }
}
