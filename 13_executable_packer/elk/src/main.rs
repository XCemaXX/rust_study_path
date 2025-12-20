use std::error::Error;

use crate::procfs::Mapping;
use clap::{Parser, Subcommand};

mod name;
mod process;
mod procfs;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    nested: SubCommand,
}

#[derive(Subcommand)]
enum SubCommand {
    Autosym(AutosymArgs),
    Run(RunArgs),
    Dig(DigArgs),
}

#[derive(clap::Args)]
/// Given a PID, spit out GDB commands to load all .so files
/// mapped in memory.
struct AutosymArgs {
    /// the PID of the process to examine
    pid: u32,
}

#[derive(clap::Args)]
/// Load and run an ELF executable
struct RunArgs {
    /// the absolute path of an executable file to load and run
    exec_path: String,
    /// arguments for the executable file
    args: Vec<String>,
}

#[derive(clap::Args)]
/// Shows information about an address in a memory's address space
struct DigArgs {
    #[arg(long)]
    /// the PID of the process whose memory space to examine
    pid: u32,
    #[arg(long)]
    /// the address to look for
    addr: u64,
}

type AnyError = Box<dyn Error>;

fn main() {
    if let Err(e) = do_main() {
        eprintln!("Fatal error: {e}");
    }
}

fn do_main() -> Result<(), AnyError> {
    let args = Args::parse();
    match args.nested {
        SubCommand::Autosym(args) => cmd_autosym(args),
        SubCommand::Run(args) => cmd_run(args),
        SubCommand::Dig(args) => cmd_dig(args),
    }
}

#[derive(thiserror::Error, Debug)]
enum WithMappingsError {
    #[error("parsgin failed: {0}")]
    Parse(String),
}

fn with_mappings<F, T>(pid: u32, f: F) -> Result<T, AnyError>
where
    F: Fn(&Vec<procfs::Mapping<'_>>) -> Result<T, AnyError>,
{
    let maps = std::fs::read_to_string(format!("/proc/{}/maps", pid))?;
    match procfs::mappings(&maps) {
        Ok((_, mappings)) => f(&mappings),
        Err(e) => Err(Box::new(WithMappingsError::Parse(format!("{e:?}")))),
    }
}

fn cmd_autosym(args: AutosymArgs) -> Result<(), AnyError> {
    with_mappings(args.pid, |mappings| {
        mappings
            .iter()
            .filter(|m| m.perms.x && m.source.is_file())
            .try_for_each(analyze)
    })
}

fn cmd_dig(args: DigArgs) -> Result<(), AnyError> {
    let addr = delf::Addr(args.addr);

    with_mappings(args.pid, |mappings| {
        if let Some(mapping) = mappings
            .iter()
            .find(|mapping| mapping.addr_range.contains(&addr))
        {
            dig(mapping, addr)?;
        }
        Ok(())
    })
}

fn dig(mapping: &Mapping<'_>, addr: delf::Addr) -> Result<(), AnyError> {
    println!("Mapped {:?} from {:?}", mapping.perms, mapping.source);
    println!(
        "(Map range: {:?}, {:?} total)",
        mapping.addr_range,
        Size(mapping.addr_range.end - mapping.addr_range.start)
    );
    let procfs::Source::File(path) = mapping.source else {
        return Ok(());
    };

    let contents = std::fs::read(path)?;
    let Some(file) = delf::File::parse_or_print_error(&contents) else {
        return Ok(());
    };

    let offset = addr + mapping.offset - mapping.addr_range.start;
    let Some(segment) = file
        .program_headers
        .iter()
        .find(|ph| ph.file_range().contains(&offset))
    else {
        return Ok(());
    };

    let vaddr = offset + segment.vaddr - segment.offset;
    println!("Object virtual addres: {vaddr:?}");

    let Some(section) = file
        .section_headers
        .iter()
        .find(|sh| sh.mem_range().contains(&vaddr))
    else {
        return Ok(());
    };

    let name = String::from_utf8_lossy(file.shstrtab_entry(section.name));
    let section_offset = vaddr - section.addr;
    println!(
        "At section {:?} + {} (0x{:x})",
        name, section_offset.0, section_offset.0
    );

    match file.read_symtab_entries() {
        Err(e) => println!("Could not read syms: {e:?}"),
        Ok(syms) => {
            for sym in &syms {
                let sym_range = sym.value..(sym.value + delf::Addr(sym.size));
                if !(sym.value == vaddr || sym_range.contains(&vaddr)) {
                    continue;
                }
                let sym_offset = vaddr - sym.value;
                let sym_name = String::from_utf8_lossy(file.strtab_entry(sym.name));
                println!(
                    "At symbol {:?} + {} (0x{:x})",
                    sym_name, sym_offset.0, sym_offset.0
                );
            }
        }
    }
    Ok(())
}

fn cmd_run(args: RunArgs) -> Result<(), AnyError> {
    let mut proc = process::Process::new();
    let exec_index = proc.load_object_and_dependencies(&args.exec_path)?;
    let proc = proc.allocate_tls();
    let proc = proc.apply_relocations()?;
    let proc = proc.initialize_tls();
    let proc = proc.adjust_protections()?;

    use std::ffi::CString;

    let args = std::iter::once(CString::new(args.exec_path.as_bytes()).unwrap())
        .chain(
            args.args
                .iter()
                .map(|s| CString::new(s.as_bytes()).unwrap()),
        )
        .collect();
    let env = std::env::vars()
        .map(|(k, v)| CString::new(format!("{k}={v}").as_bytes()).unwrap())
        .collect();

    let opts = process::StartOptions {
        exec_index,
        args,
        env,
        auxv: process::Auxv::get_known(),
    };
    proc.start(&opts)
}

fn analyze(mapping: &procfs::Mapping) -> Result<(), AnyError> {
    if mapping.deleted {
        return Ok(());
    }
    let procfs::Source::File(path) = mapping.source else {
        return Ok(());
    };
    let contents = std::fs::read(path)?;
    let Some(file) = delf::File::parse_or_print_error(&contents) else {
        return Ok(());
    };
    let Some(section) = file
        .section_headers
        .iter()
        .find(|sh| file.shstrtab_entry(sh.name) == b".text")
    else {
        return Ok(());
    };
    let text_addr = mapping.addr_range.start - mapping.offset + section.offset;
    println!("add-symbol-file {path:?} 0x{text_addr:?}");

    Ok(())
}

struct Size(pub delf::Addr);

impl std::fmt::Debug for Size {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        const KIB: u64 = 1024;
        const MIB: u64 = 1024 * KIB;

        let x = (self.0).0;
        match x {
            0..KIB => write!(f, "{x} B"),
            KIB..=MIB => write!(f, "{} KiB", x / KIB),
            _ => write!(f, "{} MiB", x / MIB),
        }
    }
}
