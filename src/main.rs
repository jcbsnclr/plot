use image::GenericImage;

/// The [BadEvent] error is produced when the program receives an error that is in an invalid format
#[derive(Debug, thiserror::Error)]
enum BadEvent<'a> {
    #[error("bad event: malformed channel {0:?}")]
    Channel(Option<&'a str>),
    #[error("bad event: malformed timestamp {0:?}")]
    Timestamp(Option<&'a str>),
    #[error("bad event: malformed note {0:?}")]
    Note(Option<&'a str>),

    #[error("bad event {0:?}")]
    Entire(&'a str)
}

/// The [Event] type wraps up each event received on stdin in machine readable format
#[derive(Debug, Clone, Copy)]
struct Event {
    channel: u8,
    timestamp: u64,
    note: u8
}

impl Event {
    /// Parses an [Event] from a string 
    fn from_line<'a>(line: &'a str) -> Result<Event, BadEvent<'a>> {
        match (line.chars().next(), line.chars().last()) {
            // ensure the event takes the form of a tuple
            (Some('('), Some(')')) => {
                // remove opening and closing parenthesis for processing
                let line = &line[1..line.len() - 1];

                let mut split = line.splitn(3, ", ");

                let channel = split.next()
                    .ok_or(BadEvent::Channel(None))     // missing channel field
                    .and_then(|v| v.parse().map_err(|_| BadEvent::Channel(Some(v))))?;      // malformed channel field

                let timestamp = split.next()
                    .ok_or(BadEvent::Timestamp(None))       // missing timestamp field
                    .and_then(|v| v.parse().map_err(|_| BadEvent::Timestamp(Some(v))))?;        // malformed timestamp field

                let note = split.next()
                    .ok_or(BadEvent::Note(None))        // missing note field
                    .and_then(|v| v.parse().map_err(|_| BadEvent::Note(Some(v))))?;     // malformed note field

                Ok(Event {
                    channel, timestamp, note
                })
            },


            // the event is not in the expected format
            _ => Err(BadEvent::Entire(line))
        }
    }
}

fn main() -> anyhow::Result<()> {
    let pal: [u32; 16] = [
        0xaaaaaaff,
        0x005500ff,
        0x00aa00ff,
        0x00ff00ff,
        0x0000ffff,
        0x0055ffff,
        0x00aaffff,
        0x00ffffff,
        0xff0000ff,
        0xff5500ff,
        0xffaa00ff,
        0xffff00ff,
        0xff00ffff,
        0xff55ffff,
        0xffaaffff,
        0xffffffff
    ];

    // transform stdin into a list of events; ignore bad events, log to stderr
    let events = std::io::stdin()
        .lines()
        .filter_map(|l| l.ok())
        .filter_map(|line| {
            match Event::from_line(&line) {
                Ok(e) => Some(e),
                Err(e) => {
                    eprintln!("error: {}", e);
                    None
                }
            }
        })
        .collect::<Vec<_>>();

    let mut image = image::DynamicImage::new_rgb8(512, 128);

    // extract first and last timestamp
    let time_min = events.iter()
        .map(|e| e.timestamp)
        .min();
    let time_max = events.iter()
        .map(|e| e.timestamp)
        .max();

    let (min, max) = match (time_min, time_max) {
        (Some(min), Some(max)) => (min as f64, max as f64),
        _ => {
            // no min/max timestamp, therefor no data provided
            eprintln!("no data provided; aborting");
            return Ok(());
        }
    };

    for event in events.into_iter() {
        // the x position should map timestamps across the whole of the output image
        let x = (
            ((event.timestamp as f64 - min) / (max - min)) * (image.width() - 1) as f64
        ) as u32;
        // use note for y position
        let y = event.note as u32;
        // extract colour from pallete based on channel
        let colour = pal[event.channel as usize]
            .to_le_bytes().into();

        image.put_pixel(x, y, colour);
    }
    
    // scale up the image for easier viewing and write to file
    image.resize(512, 2048, image::imageops::Nearest);
    image.save("output.png")?;

    Ok(())
}
