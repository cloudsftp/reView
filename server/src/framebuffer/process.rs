use std::fs::File;

use anyhow::{Context, Error, anyhow};
use procfs::process::{MMapPath, MemoryMap, Process, all_processes};

pub fn get_xochitl_memory_file() -> Result<(File, usize), Error> {
    let process = get_process().context("could not get xochitl process")?;

    let memory_file = process.mem().context("could not get xochitl memory file")?;
    let offset = get_framebuffer_offset_in_process_memory(&process)
        .context("could not get framebuffer offset")?;

    Ok((memory_file, offset))
}

fn get_process() -> Result<Process, Error> {
    let mut processes = all_processes()
        .context("could not get process iterator")?
        .filter_map(|p| {
            let p = p.ok()?;
            (p.stat().ok()?.comm == "xochitl").then_some(p)
        });

    let process = processes.next().context("no xochitl process found")?;

    if let Some(_) = processes.next() {
        return Err(anyhow!("found more than one xochitl process"));
    }

    Ok(process)
}

/// Get the offset of the framebuffer in the process memory
///
/// The framebuffer is the next mapped file after /dev/fb0
fn get_framebuffer_offset_in_process_memory(process: &Process) -> Result<usize, Error> {
    let framebuffer_path_name =
        MMapPath::from("/dev/fb0").context("could not build framebuffer path name")?;

    let matches_path_name = |m: &&MemoryMap| m.pathname == framebuffer_path_name;

    let maps = process.maps().context("could not get process maps")?;

    let mut maps = maps.iter().skip_while(|m| !matches_path_name(m)).skip(1);

    let framebuffer_map = maps.next().expect("found no framebuffer map");

    if maps.find(matches_path_name).is_some() {
        return Err(anyhow!("found more than one framebuffer map"));
    }

    let offset = framebuffer_map.address.0 as usize;
    Ok(offset)
}
