use std::collections::hash_map::DefaultHasher;
use std::collections::BTreeMap;
use std::hash::{Hash, Hasher};
use std::io;
use std::os::unix::process::ExitStatusExt;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread;
use std::time::{Duration, Instant};

fn fuzz(target: Arc<str>, options: Arc<[String]>, tmpfile: &str, inp: &[u8]) -> io::Result<i32> {
    std::fs::write(tmpfile, &inp)?;

    let mut child = Command::new(target.as_ref())
        .args(options.as_ref())
        .arg(tmpfile)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()?;

    // Run program for a max of 0.5 seconds
    let start = std::time::Instant::now();
    while let Ok(status) = child.try_wait() {
        if let Some(status) = status {

            if let Some(signal) = status.signal() {
                return Ok(signal);
            }

            break;
        } else if status.is_none() {
            if start.elapsed().as_secs_f64() >= 0.5 {
                child.kill().expect("Failed to kill process");
            }
        }
    }


    Ok(0)
}

fn worker(target: Arc<str>, options: Arc<[String]>, corpus: Arc<BTreeMap<PathBuf, Vec<u8>>>, thr_id: usize, cases: Arc<AtomicUsize>, crashes: Arc<AtomicUsize>)
    -> io::Result<()> {
    let tmpfile = format!("tmp_inputs/tmp{}", thr_id);

    loop {
        // Select a random file from the corpus
        let sel_inp = rand::random::<usize>() % corpus.len();
        
        let inp_pair = corpus.iter().nth(sel_inp).unwrap();
        let inp_file = inp_pair.0;
        let mut inp = inp_pair.1.clone();

        // Mutate the input randomly
        for _ in 0..8 {
            let len = inp.len();

            let idx = rand::random::<usize>() % len;
            let byte = rand::random::<u8>();

            inp[idx] = byte;
        }

        // Run fuzz case and check for a crash (signal 11)
        let signal = fuzz(Arc::clone(&target), Arc::clone(&options), &tmpfile, &inp);

        match signal {
            Ok(signal) => {
                if signal == 11 {
                    crashes.fetch_add(1, Ordering::SeqCst);
                    let mut hasher = DefaultHasher::new();
                    inp.hash(&mut hasher);
                    let hash = hasher.finish();

                    std::fs::write(
                        format!("crashes/crash_{:x}",
                                hash
                        ),
                        inp,
                    )?;

                    println!("[ {} ] Found crash - input `{}` - hash {:x}",
                             target,
                             inp_file
                             .file_name().unwrap()
                             .to_str().unwrap(),
                             hash
                    );
                }
                ()
            },
            Err(e) => eprintln!("Failed to run fuzz case: {}", e),
        };

        cases.fetch_add(1, Ordering::SeqCst);
    }
}

fn main() -> io::Result<()> {
    // Parse arguments for target file and cmdline args
    let args: Vec<String> = std::env::args().collect();
    
    if args.len() < 2 {
        eprintln!("Expected at least one argument");
        std::process::exit(1);
    }

    let target = Arc::from(args.get(1).unwrap().as_str());
    let options = Arc::from(&args[2..]);
    
    // Check for files in corpus
    let dir: Vec<_> = std::fs::read_dir("corpus")?.collect();

    if dir.len() < 1 {
        eprintln!("Corpus is empty");
        std::process::exit(1);
    }

    // Read all input file contents into memory
    let mut corpus = BTreeMap::new();

    for file in dir {
        let filename = file?.path();
        corpus.insert(filename.clone(), std::fs::read(&filename)?);
    }
    
    let corpus: Arc<BTreeMap<PathBuf, Vec<u8>>> = Arc::from(corpus);

    // Create atomic counters for statistics
    let cases = Arc::new(AtomicUsize::new(0));
    let crashes = Arc::new(AtomicUsize::new(0));

    std::fs::create_dir_all("crashes/")?;
    std::fs::create_dir_all("tmp_inputs/")?;

    let mut threads = Vec::new();

    // Run threads
    for thr_id in 0..16 {
        // Create new references for each thread
        let target = Arc::clone(&target);
        let options = Arc::clone(&options);
        let corpus = Arc::clone(&corpus);
        let cases = Arc::clone(&cases);
        let crashes = Arc::clone(&crashes);

        // Run the worker
        let thread = thread::spawn(move || {
            worker(target, options, corpus, thr_id, cases, crashes).expect("Failed to run worker");
        });

        threads.push(thread);
    }

    // Store start time to calculate elapsed time
    let start = Instant::now();

    // Spawn thread to print stats
    thread::spawn(move || {
        loop {
            // Print stats every second
            thread::sleep(Duration::from_secs(1));

            let cases = cases.load(Ordering::SeqCst);
            let crashes = crashes.load(Ordering::SeqCst);
            let elapsed = start.elapsed().as_secs_f64();
            
            println!("| {} | {:>7} cases | {:>5} secs | {:>6} cps | {:>4} crashes |",
                     target,
                     cases,
                     elapsed as usize,
                     cases / elapsed as usize,
                     crashes
            );
        }
    });

    // Wait for threads to finish
    for thread in threads {
        thread.join().unwrap();
    }

    Ok(())
}

