use std::io::{self, stdin};

use image::GenericImage;

#[derive(Debug, thiserror::Error)]
#[error("bad event")]
struct BadEvent;

#[derive(Debug)]
struct Event {
    channel: u8,
    timestamp: u64,
    note: u8
}

impl Event {
    fn from_line(line: impl AsRef<str>) -> Result<Event, BadEvent> {
        let len = line.as_ref().len();
        let line = &line.as_ref()[1..len - 1];

        println!("{}", line);

        let mut split = line.splitn(3, ", ");

        let channel = split.next()
            .ok_or(BadEvent)
            .and_then(|v| v.parse().map_err(|_| BadEvent))?;

        let timestamp = split.next()
            .ok_or(BadEvent)
            .and_then(|v| v.parse().map_err(|_| BadEvent))?;

        let note = split.next()
            .ok_or(BadEvent)
            .and_then(|v| v.parse().map_err(|_| BadEvent))?;

        Ok(Event {
            channel, timestamp, note
        })
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

    let events = stdin()
        .lines()
        .filter_map(|l| l.ok())
        .map(Event::from_line);

    let mut image = image::DynamicImage::new_rgb8(2048, 128);

    for event in events {
        let event = event.map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        image.put_pixel(event.timestamp as u32, event.note as u32, image::Rgba::from(pal[event.channel as usize].to_le_bytes()));
    }
    
    image.save("output.png")?;

    Ok(())
}
