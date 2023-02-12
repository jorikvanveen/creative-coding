use std::process::{Command, Stdio, Child};
use std::io::Read;
use std::sync::{Arc, atomic::{AtomicBool, Ordering}, mpsc::{sync_channel, Receiver, RecvError}};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AudioCaptureError {
    #[error("Failed to start ffmpeg: {0}")]
    FfmpegError(#[from] std::io::Error),

    #[error("Stdout was already taken")]
    NoStdout
}

pub struct AudioCapturer {
    ffmpeg: Child,
    do_read: Arc<AtomicBool>,
    receiver: Receiver<Vec<[u8; 2]>>
}

impl AudioCapturer {
    pub fn new(buffer_size: usize) -> Result<Self, AudioCaptureError> {
        let do_read = Arc::new(AtomicBool::new(true));

        let mut ffmpeg = Command::new("ffmpeg")
            .arg("-f").arg("alsa") // Input format
            .arg("-i").arg("default") // Recording device (desktop audio)
            .arg("-f").arg("s16le") // Output format (unsigned 32-bit little-endian ints)
            .arg("-v").arg("quiet")
            .arg("pipe:1") // Pipe PCM audio to stdout
            .stdout(Stdio::piped())
            .stderr(Stdio::null()) // Ignore stderr, so it doesn't inherit the stream from the
                                   // program.
            .spawn()?;

        // Set a bound on the channel to prevent memory leaks
        let (sender, receiver) = sync_channel::<Vec<[u8; 2]>>(1);

        // This reader thread is responsible for reading ffmpeg's output (PCM data from ALSA),
        // parsing it and calling the listeners.
        {
            let mut stdout = ffmpeg.stdout.take().ok_or(AudioCaptureError::NoStdout)?;
            let do_read = Arc::clone(&do_read);

            std::thread::spawn(move || {
                loop {
                    let mut raw_buf: Vec<u8> = vec![0; buffer_size*2];
                    if let Err(..) = stdout.read_exact(&mut raw_buf) {
                        // Exit if reading the stream failed
                        return;
                    }

                    // Every 4 bytes should be converted to a u32. The length of this
                    // buffer should be buffer_size.
                    let parsed_buf: Vec<[u8; 2]> = raw_buf
                        .iter()
                        .step_by(2)
                        .enumerate()
                        .map(|(i, _)| {
                            // The unwrap is safe because read_exact guarantees that raw_buf will
                            // be completely filled and the length of raw_buf is always divisible
                            // by 4.
                            let bytes: [u8; 2] = raw_buf[(i*2)..(i*2)+2].try_into().unwrap();
                            bytes
                        })
                        .collect();
                    
                    // If there is no one to recieve this buffer, disregard it. Better to skip
                    // some frames than to build up a backlog
                    let _ = sender.try_send(parsed_buf);

                    if !do_read.load(Ordering::SeqCst) {
                        // Stop reading
                        break;
                    }
                }
            });
        }

        Ok(AudioCapturer {
            ffmpeg,
            do_read,
            receiver
        })
    }

    pub fn read_frame(&self) -> Result<Vec<[u8; 2]>, RecvError> {
        self.receiver.recv()
    }

    pub fn stop(&mut self) {
        let _ = self.ffmpeg.kill();
        self.do_read.store(false, Ordering::Relaxed);
    }
}

impl Drop for AudioCapturer {
    fn drop(&mut self) {
        self.do_read.store(false, Ordering::Relaxed);
        self.stop(); 
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Instant;

    #[test]
    fn listen_to_audio_5_sec() {
        let mut capturer = AudioCapturer::new(2048).unwrap();

        let start = Instant::now();

        while Instant::now().duration_since(start).as_millis() < 5000 {
            let frame: Vec<i16> = capturer.read_frame().unwrap().into_iter()
                .map(|bytes| i16::from_le_bytes(bytes))
                .collect();
            //let f32_frame: Vec<f32> = frame.iter()
            //    .map(|amplitude| (*amplitude as f32) / (u32::max_value() as f32))
            //    .collect();
            println!("{:?}", frame);
        }

        capturer.stop();
    }
}
