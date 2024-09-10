#[warn(unused_imports)]
use clap::Parser;
use std::path::Path;

use nexus_sdk::{
    compile::CompileOpts,
    nova::seq::{Generate, Nova, PP},
    Local, Prover, Verifiable,
};

use zkdoom_common::{FrameMode, InputData, OutputData};

const PACKAGE: &str = "doom";

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// ZKVM Guest elf file
    #[arg(short, long)]
    pub elf_file: String,

    /// Optional directory to write out frame jpg's to
    #[arg(short, long)]
    pub frame_dir: Option<String>,

    /// Frame commitment mode
    /// 0 = last frame
    /// N = every N frames
    /// -1 = No frames are committed
    #[arg(short = 'p', long, default_value_t = 1)]
    pub frame_mode: i32,

    /// Number of calls to make to the doom_update() function
    #[arg(short, long, default_value_t = 5)]
    pub update_calls: u32,

    /// Demo file input
    #[arg(short, long)]
    pub demo_file: String,
}
fn main() {
    let args = Args::parse();
    let elf_file = Path::new(&args.elf_file);

    if !elf_file.exists() {
        tracing::error!("Failed to find elf_file input: {}", args.elf_file);
        return;
    }

    let elf = std::fs::read(elf_file).expect("Failed to read elf file");

    let demo_file = Path::new(&args.demo_file);

    if !demo_file.exists() {
        tracing::error!("Failed to find demo_file input: {}", args.elf_file);
        return;
    }

    let demo = std::fs::read(demo_file).expect("Failed to demo file");

    let frame_mode = match args.frame_mode {
        -1 => FrameMode::None,
        0 => FrameMode::Last,
        y => FrameMode::Many(y as u32),
    };
    
    if args.update_calls < 2 {
        tracing::error!("update_calls (-u) must be >= 2");
        return;
    }

    let input = InputData {
        lmp_data: demo,
        update_calls: args.update_calls,
        frame_mode,
    };
    let mut opts = CompileOpts::new(PACKAGE);


    println!("Setting up Nova public parameters...");
    let pp: PP = PP::generate().expect("failed to generate parameters");
    println!("Compiling guest program...");
    let prover: Nova<Local> = Nova::compile(&opts).expect("failed to compile guest program");

    opts.set_memlimit(8); // use an 8mb memory
    println!("Proving execution of guest program...");
    let proof = prover
        .prove_with_input::<InputData>(&pp, &input)
        .expect("failed to prove program");

    let output: OutputData = proof
        .output()
        .expect("failed to deserialize output");

    println!("Completed {} updates in {} gametics.", args.update_calls, output.gametics);

    println!(">>>>> Logging\n{}<<<<<", proof.logs().join(""));

    print!("Verifying execution...");
    proof.verify(&pp).expect("failed to verify proof");
    println!("  Succeeded!");
    if let Some(frame_dir) = args.frame_dir {
        save_frames(&output.frames, &frame_dir);
    }
    println!("frames saved !");
}
fn save_frames(frames: &[Vec<u8>], frame_dir: &str) {
    let out_dir = Path::new(frame_dir);
    let channels = 3;

    for (idx, frame) in frames.iter().enumerate() {
        // let mut img = RgbImage::new(puredoom_rs::SCREENWIDTH, puredoom_rs::SCREENHEIGHT);
        // let src_pitch = puredoom_rs::SCREENWIDTH * channels;

        // for y in 0..puredoom_rs::SCREENHEIGHT {
        //     for x in 0..puredoom_rs::SCREENWIDTH {
        //         let srck = (y * src_pitch + x * channels) as usize;
        //         let r = frame[srck];
        //         let g = frame[srck + 1];
        //         let b = frame[srck + 2];

        //         img.put_pixel(x, y, Rgb([r, g, b]));
        //     }
        // }
        // let path = out_dir.join(format!("frame{idx}.jpg"));
        // img.save(path).expect("Failed to save frame image");
    }
    println!("Saved {} frames to {}", frames.len(), frame_dir);
}