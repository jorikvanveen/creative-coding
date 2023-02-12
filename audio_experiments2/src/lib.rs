use std::cell::RefCell;
use std::rc::Rc;

use libpulse_binding as pulse;
use pulse::callbacks::ListResult;
use pulse::context::{Context, FlagSet as ContextFlagSet, State as ContextState};
use pulse::def::BufferAttr;
use pulse::mainloop::standard::{IterateResult, Mainloop};
use pulse::operation::State as OperationState;
use pulse::sample::Spec;
use pulse::stream::{FlagSet as StreamFlagSet, PeekResult, Stream};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum GetMonitorError {
    #[error("Mainloop terminated unexpectedly")]
    MainLoopExited,

    #[error("No default output found")]
    NoDefaultSink,

    #[error("No monitor found for default output")]
    NoDefaultMonitor,
}

fn get_default_sink_monitor(
    context: &mut Context,
    mainloop: &mut Mainloop,
) -> Result<String, GetMonitorError> {
    use GetMonitorError::*;
    let default_sink_name: Rc<RefCell<Option<String>>> = Rc::new(RefCell::new(None));

    {
        let default_sink_name = Rc::clone(&default_sink_name);
        context.introspect().get_server_info(move |server_info| {
            let name = &*match server_info.default_sink_name.clone() {
                Some(name) => name,
                None => return,
            };
            default_sink_name.replace(Some(name.into()));
        });
    }

    // Wait for default_sink_name to be set.
    loop {
        match mainloop.iterate(true) {
            IterateResult::Success(..) => {}
            _ => return Err(MainLoopExited),
        }

        if default_sink_name.borrow().is_some() {
            break;
        }
    }

    let default_sink_name = match default_sink_name.borrow().clone() {
        Some(name) => name,
        None => return Err(NoDefaultSink),
    };

    let default_sink_monitor_name: Rc<RefCell<Option<String>>> = Rc::new(RefCell::new(None));
    let default_sink_monitor_name_op;
    {
        let default_sink_monitor_name = Rc::clone(&default_sink_monitor_name);
        default_sink_monitor_name_op =
            context
                .introspect()
                .get_sink_info_by_name(&default_sink_name, move |sink_info| {
                    match sink_info {
                        ListResult::Item(sink_info) => {
                            if default_sink_monitor_name.borrow().is_some() {
                                // Ignore subsequent results.
                                return;
                            }

                            let name = match sink_info.monitor_source_name.clone() {
                                Some(name) => name.to_string(),
                                None => return,
                            };

                            default_sink_monitor_name.replace(Some(name));
                        }
                        _ => return,
                    }
                });
    }

    loop {
        match mainloop.iterate(true) {
            IterateResult::Success(..) => {}
            _ => return Err(MainLoopExited),
        }

        if default_sink_monitor_name.borrow().is_some() {
            break;
        }

        if default_sink_monitor_name_op.get_state() == OperationState::Done {
            // Callback errored
            return Err(NoDefaultMonitor);
        }
    }

    // Unwrap here is okay because we asserted the existance in the loop above.
    let default_sink_monitor_name = default_sink_monitor_name.borrow().clone().unwrap();
    Ok(default_sink_monitor_name)
}

pub struct DesktopAudioRecorder {
    mainloop: Mainloop,
    context: Context,
    stream: Stream
}

#[derive(Error, Debug)]
pub enum CreateError {
    #[error("Failed to create main loop")]
    MainLoopCreationFail,

    #[error("Failed to create context")]
    ContextCreateFail,

    #[error("Failed to initiate context connection")]
    ConnectionInitFail(#[from] pulse::error::PAErr),

    #[error("Failed to connect context")]
    ConnectionFail,

    #[error("Failed to get monitor for default output")]
    MonitorFail(#[from] GetMonitorError),

    #[error("Main loop exited unexpectedly")]
    MainLoopExited,
}

#[derive(Error, Debug)]
pub enum ReadError {
    #[error("Main loop exited unexpectedly")]
    MainLoopExited,

    #[error("Error reading stream")]
    StreamReadError(#[from] pulse::error::PAErr)
}

impl DesktopAudioRecorder {
    pub fn new(application_name: &str) -> Result<Self, CreateError> {
        use CreateError::*;

        let mut mainloop = Mainloop::new().ok_or(MainLoopCreationFail)?;
        let mut context = Context::new(&mainloop, application_name).ok_or(ContextCreateFail)?;
        context.connect(None, ContextFlagSet::NOFLAGS, None)?;

        loop {
            match mainloop.iterate(true) {
                IterateResult::Err(_) | IterateResult::Quit(_) => {
                    eprintln!("Loop exited");
                    return Err(MainLoopExited);
                }
                IterateResult::Success(_) => {}
            }

            match context.get_state() {
                ContextState::Ready => {
                    println!("Ready");
                    break;
                }
                ContextState::Failed | ContextState::Terminated => {
                    eprintln!("Failed to connect");
                    return Err(ConnectionFail);
                }
                _ => {}
            }
        }

        let monitor_source_name = get_default_sink_monitor(&mut context, &mut mainloop)?;
        let sample_spec = Spec {
            channels: 1,
            format: pulse::sample::Format::S32le,
            rate: 44100
        };

        assert!(sample_spec.is_valid());

        let mut stream = Stream::new(
            &mut context,
            "Epic experiment stream",
            &sample_spec,
            None
        ).unwrap();
        
        stream.connect_record(
            Some(&monitor_source_name),
            Some(&BufferAttr {
                maxlength: u32::max_value(),
                tlength: u32::max_value(),
                prebuf: u32::max_value(),
                minreq: u32::max_value(),
                fragsize: 2^16
            }),
            StreamFlagSet::NOFLAGS
        ).unwrap();


        Ok(DesktopAudioRecorder { mainloop, context, stream })
    }

    pub fn read_frame(&mut self) -> Result<Vec<i32>, ReadError> {
        use ReadError::*;

        loop {
            match self.mainloop.iterate(true) {
                IterateResult::Success(..) => {},
                _ => return Err(MainLoopExited)
            };

            match self.stream.get_state() {
                pulse::stream::State::Ready => {},
                _ => {
                    continue;
                }
            }
            let peek_result = self.stream.peek()?;
            match peek_result {
                PeekResult::Data(data) => {
                    // There is probably a nicer way to do this.
                    let parsed_data: Vec<i32> = data.into_iter()
                        .step_by(4)
                        .enumerate()
                        .map(|(i, _)| {
                            i32::from_le_bytes(data[i*4..(i+1)*4].try_into().unwrap())
                        })
                        .collect();

                    self.stream.discard().unwrap();
                    return Ok(parsed_data);
                },
                PeekResult::Empty => {},
                PeekResult::Hole(..) => { self.stream.discard().unwrap(); }
            }
        };
    }

    /// Do cleanup so that the program doesn't segfault. Automatically called when self goes out of
    /// scope
    pub fn quit(&mut self) {
        self.mainloop.quit(pulse::def::Retval(0));
        self.context.disconnect();
        let _ = self.stream.disconnect();
    }
}

impl Drop for DesktopAudioRecorder {
    fn drop(&mut self) {
        self.quit();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn record_5_seconds() {
        use std::time::Instant;
        let mut recorder = DesktopAudioRecorder::new("Experiment").unwrap();

        let start = Instant::now();

        loop {
            match recorder.read_frame() {
                Ok(data) => println!("{:?}", data),
                Err(e) => eprintln!("{}", e)
            };

            if Instant::now().duration_since(start).as_millis() > 5000 {
                break;
            }
        }
    }
}
