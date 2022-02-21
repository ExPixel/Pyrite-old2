use arm::Isa;
use once_cell::sync::OnceCell;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::process::Command;

fn devkit_arm() -> &'static Path {
    static DEVKIT_ARM: OnceCell<PathBuf> = OnceCell::new();

    DEVKIT_ARM.get_or_init(|| {
        let mut path = if let Ok(arm) = std::env::var("DEVKITARM") {
            PathBuf::from(arm)
        } else if let Ok(pro) = std::env::var("DEVKITPRO") {
            PathBuf::from(pro).join("devkitARM")
        } else {
            panic!("DEVKITARM or DEVKITPRO environment variables must be defined")
        };

        // If the path doesn't exists on Windows then MSYS2 may have been used to install
        // devkitPRO in which case use cygpath to find the Windows path.
        if !path.exists() && cfg!(target_os = "windows") {
            if let Ok(result) = Command::new("cygpath").arg("-w").arg(&path).output() {
                let win_path = String::from_utf8_lossy(&result.stdout);
                let win_path = win_path.trim();
                if !win_path.trim().is_empty() {
                    println!("using windows path from cygpath: {}", win_path);
                    path = PathBuf::from(win_path);
                }
            }
        }

        if !path.exists() {
            panic!("DEVKITARM path {} does not exist", path.display());
        }

        path
    })
}

fn simple_linker_script() -> &'static Path {
    const SIMPLE_LINKER_SCRIPT: &str = "
    ENTRY(_start);
    SECTIONS
    {
        . = 0x0;

        /* Place special section .text.prologue before everything else */
        .text : {
            . = ALIGN(4);
            *(.text.prologue);
            *(.text*);
            . = ALIGN(4);
        }

        /* Output the data sections */
        .data : {
            . = ALIGN(4);
            *(.data*);
        }

        .rodata : {
            . = ALIGN(4);
            *(.rodata*);
        }

        /* The BSS section for uniitialized data */
        .bss : {
            . = ALIGN(4);
            __bss_start = .;
            *(COMMON);
            *(.bss);
            . = ALIGN(4);
            __bss_end = .;
        }

        /* Size of the BSS section in case it is needed */
        __bss_size = ((__bss_end)-(__bss_start));

        /* Remove the note that may be placed before the code by LD */
        /DISCARD/ : {
            *(.note.gnu.build-id);
            *(.ARM.attributes);
        }
    }
    ";
    static SIMPLE_LINKER_SCRIPT_PATH: OnceCell<PathBuf> = OnceCell::new();

    SIMPLE_LINKER_SCRIPT_PATH.get_or_init(|| {
        let tmp_dir = Path::new(env!("CARGO_TARGET_TMPDIR"));
        let path = tmp_dir.join("simple-linker-script.ld");
        std::fs::write(&path, SIMPLE_LINKER_SCRIPT).expect("failed to write simple linker script");
        path
    })
}

fn run_program<P: AsRef<OsStr>>(
    program: P,
    args: &[&str],
) -> std::io::Result<std::process::ExitStatus> {
    println!("executing: {}", program.as_ref().to_str().unwrap());

    let output = Command::new(program.as_ref())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .args(args)
        .output()?;

    let mut had_output = false;

    let in_obj_dump = program
        .as_ref()
        .to_str()
        .map(|s| s.contains("objdump"))
        .unwrap_or(false);
    let mut in_obj_dump_preamble = true;

    let stdout = String::from_utf8_lossy(&output.stdout);
    for line in stdout.lines() {
        if !in_obj_dump {
            println!("  out: {}", line.trim_end());
            had_output = true;
            continue;
        }

        // For objdump we do some special formatting for the output:
        if !in_obj_dump_preamble {
            println!("    {}", line.trim());
            had_output = true;
        }

        // After we encounter one of these lines:
        //    00000000 <.data>:
        //    00000000 <.text>:
        // we are no longer in the preamble.
        if line.contains(">:") {
            println!("  {}", line.trim());
            in_obj_dump_preamble = false;
            had_output = true;
        }
    }

    let stderr = String::from_utf8_lossy(&output.stderr);
    for (idx, line) in stderr.lines().enumerate() {
        if idx == 0 && had_output {
            println!()
        }
        println!("  err: {}", line.trim_end());
    }

    Ok(output.status)
}

fn run_arm_program(program: &str, args: &[&str]) -> std::io::Result<std::process::ExitStatus> {
    let mut p = devkit_arm().join("arm-none-eabi/bin");
    p.push(program);
    run_program(p, args)
}

trait PathOrString {
    fn convert_to_str(&self) -> &str;
}

impl PathOrString for &'_ Path {
    fn convert_to_str(&self) -> &str {
        self.to_str().unwrap_or("")
    }
}

impl PathOrString for PathBuf {
    fn convert_to_str(&self) -> &str {
        self.to_str().unwrap_or("")
    }
}

impl PathOrString for &'_ str {
    fn convert_to_str(&self) -> &str {
        *self
    }
}

impl PathOrString for &String {
    fn convert_to_str(&self) -> &str {
        *self
    }
}

macro_rules! arm_run {
    ($program:expr, $($arg:expr),* $(,)?) => {
        run_arm_program($program, &[
            $($arg.convert_to_str()),*
        ])
    };
}

pub fn assemble(isa: Isa, name: &str, source: &str) -> std::io::Result<Vec<u8>> {
    let tmp_dir = Path::new(env!("CARGO_TARGET_TMPDIR"));

    let source_file_path = tmp_dir.join(format!("{}.s", name));
    let object_file_path = tmp_dir.join(format!("{}.o", name));
    let elf_file_path = tmp_dir.join(format!("{}.elf", name));
    let bin_file_path = tmp_dir.join(format!("{}.bin", name));

    let files_to_destroy = [
        &source_file_path as &Path,
        &object_file_path,
        &elf_file_path,
        &bin_file_path,
    ];
    let _file_destructor = FileDestructor::new(&files_to_destroy);

    let preamble = "\
    .global _start
    _start:\n";
    let mut new_source = String::with_capacity(preamble.len() + source.len() + 1);
    new_source.push_str(preamble);
    new_source.push_str(source);
    new_source.push('\n');
    let source = new_source;

    // `as` outputs a warning if the file does not end with a newline or if there
    // is no `_start:` symbol.

    std::fs::write(&source_file_path, source)?;

    let as_output = if isa == Isa::Arm {
        arm_run!(
            "as",
            "-mcpu=arm7tdmi",
            "-march=armv4t",
            "-mthumb-interwork",
            "-o",
            object_file_path,
            source_file_path
        )?
    } else {
        arm_run!(
            "as",
            "-mthumb",
            "-mcpu=arm7tdmi",
            "-march=armv4t",
            "-mthumb-interwork",
            "-o",
            object_file_path,
            source_file_path
        )?
    };
    if !as_output.success() {
        panic!("failed to assemble {}", source_file_path.display());
    }

    let ld_script = format!("-T{}", simple_linker_script().display());
    if !arm_run!("ld", &ld_script, "-o", elf_file_path, object_file_path)?.success() {
        panic!("failed to link {}", object_file_path.display());
    }

    if !arm_run!("objcopy", "-O", "binary", elf_file_path, bin_file_path)?.success() {
        panic!("failed to extract binary from {}", elf_file_path.display());
    }

    let objdump_output = if isa == Isa::Arm {
        arm_run!(
            "objdump",
            "-b",
            "binary",
            "-m",
            "armv4t",
            "--adjust-vma=0x0",
            "-D",
            bin_file_path
        )?
    } else {
        arm_run!(
            "objdump",
            "-b",
            "binary",
            "-m",
            "armv4t",
            "-Mforce-thumb",
            "--adjust-vma=0x0",
            "-dj",
            ".text",
            bin_file_path
        )?
    };
    if !objdump_output.success() {
        panic!("failed to disassemble binary {}", bin_file_path.display())
    }

    std::fs::read(&bin_file_path)
}

struct FileDestructor<'p> {
    paths: &'p [&'p Path],
}

impl<'p> FileDestructor<'p> {
    pub fn new(paths: &'p [&'p Path]) -> Self {
        FileDestructor { paths }
    }
}

impl<'p> Drop for FileDestructor<'p> {
    fn drop(&mut self) {
        for &path in self.paths {
            if !path.exists() {
                continue;
            }

            if let Err(err) = std::fs::remove_file(path) {
                eprintln!(
                    "error occurred while deleting path `{}`: {}",
                    path.display(),
                    err
                );
            }
        }
    }
}
