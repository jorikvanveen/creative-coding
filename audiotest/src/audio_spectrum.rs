use ringbuffer::{AllocRingBuffer, RingBuffer, RingBufferExt};
use simple_pulse_desktop_capture::DesktopAudioRecorder;
use spectrum_analyzer::FrequencySpectrum;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::Arc;
use std::thread;

// Record samples
// Throw last 4096 samples into FFT
// Send fft result through channel to be received by different thread
pub struct Capturer {
    do_capture: AtomicBool,
    capture_thread: thread::JoinHandle<()>,
    spectrum_receiver: Receiver<FrequencySpectrum>,
    ready_sender: Sender<()>,
}

impl Capturer {
    pub fn new(application_name: String, bufsize: usize) -> Capturer {
        let do_capture = Arc::new(AtomicBool::new(true));
        let (spectrum_sender, spectrum_receiver) = std::sync::mpsc::channel();
        let (ready_sender, ready_receiver) = std::sync::mpsc::channel::<()>();

        let capture_thread;
        {
            let do_capture = Arc::clone(&do_capture);
            capture_thread = thread::spawn(move || {
                let mut buffer = AllocRingBuffer::with_capacity(bufsize);
                let mut recorder = match DesktopAudioRecorder::new(&application_name) {
                    Ok(recorder) => recorder,
                    Err(e) => {
                        eprintln!("Recorder thread stopped: {}", e);
                        return;
                    }
                };

                let mut is_consumer_ready = false;

                // Quits when do_capture is false.
                while do_capture.load(Ordering::SeqCst) {
                    let frame = match recorder.read_frame() {
                        Ok(f) => f,
                        Err(e) => {
                            eprintln!("Failed to receive frame: {}", e);
                            continue;
                        }
                    };

                    buffer.extend(frame);
                    let ready_recv_result = ready_receiver.try_recv();

                    match ready_recv_result {
                        Ok(..) => is_consumer_ready = true,
                        Err(..) => {}
                    }

                    if buffer.is_full() && is_consumer_ready {
                        // Do FFT
                        // TODO: Error handling
                        let spectrum = fft(&buffer.to_vec()).unwrap();
                        is_consumer_ready = false;
                        let _ = spectrum_sender.send(spectrum);
                    }
                }
            });
        }

        let _ = ready_sender.send(());

        Capturer {
            do_capture: AtomicBool::new(false),
            capture_thread,
            spectrum_receiver,
            ready_sender,
        }
    }

    pub fn get_spectrum(&self) -> FrequencySpectrum {
        let spectrum = self.spectrum_receiver.recv().unwrap();
        let _ = self.ready_sender.send(());

        spectrum
    }
}

impl Drop for Capturer {
    fn drop(&mut self) {
        self.do_capture.store(false, Ordering::SeqCst);
    }
}

fn fft(
    samples: &Vec<i32>,
) -> Result<FrequencySpectrum, spectrum_analyzer::error::SpectrumAnalyzerError> {
    use spectrum_analyzer::scaling::divide_by_N;
    use spectrum_analyzer::windows::hann_window;
    use spectrum_analyzer::{samples_fft_to_spectrum, FrequencyLimit};

    // Convert samples into f32
    let f32_samples: Vec<f32> = samples
        .iter()
        .map(|sample| (*sample as f32) / (u32::max_value() as f32))
        .collect();

    let hann_window = hann_window(&f32_samples);

    Ok(samples_fft_to_spectrum(
        &hann_window,
        44100,
        FrequencyLimit::All,
        Some(&divide_by_N),
    )?)
}
