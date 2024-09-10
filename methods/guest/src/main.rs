#![cfg_attr(target_arch = "riscv32", no_std, no_main)]

use nexus_rt::{println, read_private_input, write_output};
use zkdoom_common::{FrameMode, InputData, OutputData};
use puredoom_rs::{
    doom_get_framebuffer, doom_init, doom_set_exit, doom_set_file_io, doom_set_getenv,
    doom_set_gettime, doom_set_malloc, doom_set_print, doom_update,
};

#[derive(Debug)]
pub struct GameMonitor {
    pub frames: Vec<Vec<u8>>,
    pub gametics: u32,
    pub frame_mode: FrameMode,
}

impl GameMonitor {
    fn new(frame_mode: FrameMode) -> Self {
        Self {
            frames: vec![],
            gametics: 0,
            frame_mode,
        }
    }
    fn capture_frame(&mut self) {
        let channels: usize = 3;
        unsafe {
            let buff = doom_get_framebuffer(channels as c_int);
            self.frames.push(
                std::slice::from_raw_parts(
                    buff,
                    channels
                        * puredoom_rs::SCREENWIDTH as usize
                        * puredoom_rs::SCREENHEIGHT as usize,
                )
                .to_vec(),
            )
        }
    }

    fn finalize(&mut self) {
        unsafe {
            let output = OutputData {
                frames: std::mem::take(&mut self.frames),
                gametics: puredoom_rs::gametic as u32,
            };
            env::commit(&output);
        }
    }
}

static GAME: OnceLock<Mutex<GameMonitor>> = OnceLock::new();
#[nexus_rt::main]
fn main() {
    print!("Hello, World!\n");
    println!("Initializing DOOM in Nexus zkVM...");
    let input: InputData = read_private_input().expect("Failed to read input");
    let mut argv = vec!["doom", "-file", "/wads/doom1.wad", "-timedemo", "demo"];
    if input.frame_mode == FrameMode::None {
        // Disable rendering if we are not saving any frames
        argv.push("-nodraw");
        argv.push("-noblit");
    }

    let argc_raw = argv.len().try_into().expect("Failed to convert argv len");
    let mut converted: Vec<*mut c_char> = argv
        .into_iter()
        .map(|x| {
            CString::new(x)
                .expect("Invalid argv str pointer")
                .into_raw()
        })
        .collect();
    let converted = converted.as_mut_ptr();

    // Initialized the FS
    {
        let mut fs = fs().lock().expect("Failed to get FS lock");
        fs.add_file("/wads/doom1.wad", puredoom_rs::DOOM_WAD);
        fs.add_file("demo.lmp", &input.lmp_data);
    }

    let frame_mode = input.frame_mode;

    GAME.set(Mutex::new(GameMonitor::new(frame_mode)))
        .expect("Failed to set GAME global");

    unsafe {
        // init functions
        doom_set_print(Some(zkvm_doom_print));
        doom_set_malloc(Some(zkvm_doom_malloc), Some(zkvm_doom_free));
        doom_set_getenv(Some(zkvm_doom_getenv));
        doom_set_exit(Some(zkvm_doom_exit));
        doom_set_gettime(Some(zkvm_doom_gettime));
        doom_set_file_io(
            Some(zkvm_doom_open),
            Some(zkvm_doom_close),
            Some(zkvm_doom_read),
            Some(zkvm_doom_write),
            Some(zkvm_doom_seek),
            Some(zkvm_doom_tell),
            Some(zkvm_doom_eof),
        );

        // initialize doom
        doom_init(
            argc_raw,
            converted,
            puredoom_rs::DOOM_FLAG_MENU_DARKEN_BG as i32,
        );
}
