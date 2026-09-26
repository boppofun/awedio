use awedio::{Sound, Stop};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let Some(file_path) = args() else {
        eprintln!("usage: FILE_PATH");
        std::process::exit(2);
    };

    let mut sound = awedio::sounds::open_file(file_path)?;

    let mut num_samples = 0;
    let mut buf = vec![0; 1024 * sound.channel_count() as usize];

    loop {
        let filled = sound.next_samples(&mut buf);
        num_samples += filled.written;
        match filled.stop {
            None => (),
            Some(Stop::Paused) => {
                println!("Encountered a Pause. Stopping.");
                break;
            }
            Some(Stop::Finished) => {
                break;
            }
            Some(Stop::MetadataChanged) => {
                println!(
                    "Encountered MetadataChanged. New sample rate: {}, New channel count: {}",
                    sound.sample_rate(),
                    sound.channel_count()
                );
                buf = vec![0; 1024 * sound.channel_count() as usize];
            }
            Some(Stop::Error(e)) => {
                println!("Encountered error: {:?}", e);
                break;
            }
        }
    }
    println!("Read {} samples.", num_samples);

    Ok(())
}

fn args() -> Option<String> {
    let mut args = std::env::args();
    args.next()?;
    args.next()
}
