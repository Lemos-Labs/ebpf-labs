use anyhow::Result;
use libbpf_rs::PerfBufferBuilder;
use libbpf_rs::skel::{OpenSkel, Skel, SkelBuilder};
use plain::Plain;
use std::mem::MaybeUninit;
use std::time::Duration;

mod hello {
    include!(concat!(env!("OUT_DIR"), "/hello.skel.rs"));
}
use hello::*;

#[repr(C)]
#[derive(Default, Copy, Clone)]
struct DataT {
    pid: i32,
    uid: i32,
    command: [u8; 16],
    message: [u8; 12],
}

unsafe impl Plain for DataT {}

fn handle_event(_cpu: i32, bytes: &[u8]) {
    let mut event = DataT::default();
    plain::copy_from_bytes(&mut event, bytes).expect("data buffer too short");

    let command = std::str::from_utf8(&event.command)
        .unwrap_or("?")
        .trim_end_matches('\0');
    let message = std::str::from_utf8(&event.message)
        .unwrap_or("?")
        .trim_end_matches('\0');

    println!("{} {} {} {}", event.pid, event.uid, command, message);
}

fn handle_lost(cpu: i32, count: u64) {
    eprintln!("lost {count} events on cpu {cpu}");
}

fn main() -> Result<()> {
    let skel_builder = HelloSkelBuilder::default();

    // Storage for the OpenObject — lives on the stack for the rest of main().
    let mut open_object = MaybeUninit::uninit();

    let open_skel = skel_builder.open(&mut open_object)?;
    let mut skel = open_skel.load()?;
    skel.attach()?;

    let perf = PerfBufferBuilder::new(&skel.maps.output)
        .sample_cb(handle_event)
        .lost_cb(handle_lost)
        .build()?;

    println!("Tracing execve... Ctrl-C to stop.");
    loop {
        perf.poll(Duration::from_millis(100))?;
    }
}
