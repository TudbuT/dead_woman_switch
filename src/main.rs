use std::{env, fs, io, path::Path, process, thread, time::Duration};

extern "C" {
    fn kill(pid: i32, sig: core::ffi::c_int) -> core::ffi::c_int;
    fn sync();
}

fn main() {
    let argv = env::args().collect::<Vec<_>>();

    if argv.len() <= 1 {
        help();
        unsafe {
            kill(process::id() as i32, 15);
        }
        return;
    }

    let dev = argv[1].to_owned();

    let path = Path::new(dev.as_str());

    if !path.exists() {
        eprintln!("Device is not currently present. Exiting.");
        return;
    }

    #[cfg(debug_assertions)]
    println!("Waiting for the file to die...");

    while path.exists() {
        thread::sleep(Duration::from_millis(5));
    }

    #[cfg(debug_assertions)]
    println!("File died, killing system.");

    if kill_all().is_err() {
        #[cfg(debug_assertions)]
        println!("E-Stop due to error.");
        thread::sleep(Duration::from_millis(500));
        unsafe {
            sync();
        }
        fs::write("/proc/sysrq-trigger", "o").unwrap();
    }
}

fn help() {
    println!("dead_woman_switch DEVFILE");
    println!();
    println!(" Kills the OS when a device is unplugged. ");
}

fn kill_all() -> io::Result<()> {
    let files = fs::read_dir("/proc/")?
        .map(Result::unwrap)
        .filter(|x| x.file_type().unwrap().is_dir())
        .filter(|x| {
            x.file_name()
                .to_str()
                .unwrap()
                .chars()
                .all(|x| x.is_numeric())
        })
        .map(|x| x.file_name().into_string().unwrap().parse::<i32>().unwrap())
        .collect::<Vec<_>>();

    for process in &files {
        let process = *process;
        unsafe {
            if process as u32 == process::id() {
                continue;
            }
            kill(process, 15);
        }
    }

    thread::sleep(Duration::from_millis(200));

    for process in &files {
        let process = *process;
        unsafe {
            if process as u32 == process::id() {
                continue;
            }
            kill(process, 9);
        }
    }

    unsafe {
        sync();
    }
    thread::sleep(Duration::from_millis(50));
    fs::write("/proc/sysrq-trigger", "u")?;
    thread::sleep(Duration::from_millis(50));
    fs::write("/proc/sysrq-trigger", "o")?;

    Ok(())
}
