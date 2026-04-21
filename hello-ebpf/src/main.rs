use std::fs::File;
use std::io::{BufRead, BufReader};
use std::mem::MaybeUninit;

use libbpf_rs::skel::{OpenSkel, Skel, SkelBuilder};

mod hello {
    include!(concat!(env!("OUT_DIR"), "/hello.skel.rs"));
}

use hello::HelloSkelBuilder;

fn bump_memlock_rlimit() -> anyhow::Result<()> {
    // Older kernels are charged against RLIMIT_MEMLOCK with BPF allocations.
    let rlimit = libc::rlimit {
        rlim_cur: libc::RLIM_INFINITY,
        rlim_max: libc::RLIM_INFINITY,
    };
    if unsafe { libc::setrlimit(libc::RLIMIT_MEMLOCK, &rlimit) } != 0 {
        anyhow::bail!("failed to raise RLIMIT_MEMLOCK");
    }
    Ok(())
}

fn main() -> anyhow::Result<()> {
    bump_memlock_rlimit();

    // Build the skeleton and open the BPF project
    let mut open_object = MaybeUninit::uninit();
    let skel_builder = HelloSkelBuilder::default();
    let open_skel = skel_builder.open(&mut open_object)?;

    // Create maps, verifies and loads programs in kernel
    let mut skel = open_skel.load()?;

    // Attach all programs according to its SEC() annotation
    let _links = skel.attach()?;

    println!("Tracing execve... Hit Ctrl-C to stop.");

    // Stream the global trace pipe
    let pipe = File::open("sys/kernel/tracing/trace_pipe")
        .or_else(|_| File::open("sys/kernel/debug/tracing/trace_pipe"))?;
    let reader = BufReader::new(pipe);
    for line in reader.lines() {
        println!("{}", line?);
    }

    Ok(())
}
