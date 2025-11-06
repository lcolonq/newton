use redis::Commands;
use teleia::*;
use std::{io::{Read, Write}, process};
use byteorder::WriteBytesExt;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

use crate::overlay;

const SEGMENT_LENGTH: f32 = 4.8;

fn ffmpeg_to_adts(sample_rate: u32, samples: &[f32]) -> Option<Vec<u8>> {
    let proc = process::Command::new("ffmpeg")
        .args([
            "-f", "f32le",
            "-ar", &format!("{sample_rate}"),
            "-ac", "2",
            "-i", "pipe:0",
            "-vn",
            "-c:a", "aac",
            "-f", "adts",
            "-ar", "48000",
            "-ac", "2",
            "pipe:1"
        ])
        .stdin(process::Stdio::piped())
        .stdout(process::Stdio::piped())
        .stderr(process::Stdio::null())
        .spawn().ok()?;
    {
        let mut inp = proc.stdin?;
        for s in samples {
            inp.write_f32::<byteorder::LE>(*s).ok()?;
        }
        inp.flush().ok()?;
    }
    let mut out = proc.stdout?;
    let mut ret = Vec::new();
    out.read_to_end(&mut ret).ok()?;
    Some(ret)
}

fn upload_sample(conn: &mut redis::Connection, sequence: u32, sample_rate: u32, sample: &[f32]) {
    let max: f32 = *sample.iter().max_by(|x, y| f32::total_cmp(x, y)).unwrap();
    let cells = (max / 0.1) as usize;
    let adts = ffmpeg_to_adts(sample_rate, sample).unwrap();
    let _: () = conn.lpush("hlssamples", adts).unwrap();
    let _: () = conn.ltrim("hlssamples", 0, 10).unwrap();
    let _: () = conn.set("hlssequence", sequence).unwrap();
}

pub struct Overlay {
    stream: cpal::Stream,
}

impl Overlay {
    pub fn new(ctx: &context::Context) -> Self {
        let redis = redis::Client::open("redis://shiro").unwrap();
        let mut redis_conn = redis.get_connection().unwrap();
        let host = cpal::default_host();
        let device = host.default_input_device().unwrap();
        let config = device.default_input_config().unwrap();
        let sample_rate = config.sample_rate().0;
        let mut buf: Vec<f32> = Vec::new();
        let mut sequence = 0;
        let _: () = redis_conn.del("hlssamples").unwrap();
        let _: () = redis_conn.set("hlssequence", 0).unwrap();
        let stream = device.build_input_stream(
            &config.into(),
            move |samples: &[f32], info| {
                buf.extend_from_slice(samples);
                let upload_size = (SEGMENT_LENGTH * 2.0 * sample_rate as f32) as usize;
                if buf.len() > upload_size {
                    upload_sample(&mut redis_conn, sequence, sample_rate, &buf[0..upload_size]);
                    buf.drain(0..upload_size);
                    sequence += 1;
                }
            },
            |err| {
                println!("error: {}", err);
            },
            None,
        ).unwrap();
        stream.play().unwrap();
        Self {
            stream,
        }
    }
}

impl overlay::Overlay for Overlay {}
