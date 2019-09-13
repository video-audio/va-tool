use std::sync::Arc;
use std::sync::Mutex;
use std::thread;
use std::time::Duration;

use log::error;

use crate::error::{Error, Result};
use crate::input::Input;

pub struct Source<I> {
    input: Arc<Mutex<I>>,

    thread: Option<thread::JoinHandle<()>>,
}

impl<I: 'static> Source<I>
where
    I: Input + std::marker::Send,
{
    pub fn new(input: I) -> Source<I> {
        Source {
            input: Arc::new(Mutex::new(input)),

            thread: None,
        }
    }

    pub fn start(&mut self) -> Result<()> {
        let input = self.input.clone();

        fn fn_lock_map_err<I>(err: std::sync::PoisonError<std::sync::MutexGuard<'_, I>>) -> Error {
            Error::source_input_lock(err.to_string())
        }

        let fn_do = move || -> Result<()> {
            {
                input.lock().map_err(fn_lock_map_err)?.open()?;
            }

            loop {
                input.lock().map_err(fn_lock_map_err)?.read()?;

                thread::sleep(Duration::from_secs(1));
            }
        };

        self.thread = Some(
            thread::Builder::new()
                .name("source".to_string())
                .spawn(move || loop {
                    if let Err(err) = fn_do() {
                        error!("source perform error (:reason {})", err);

                        // will retry after timeout;
                        thread::sleep(Duration::from_secs(3));
                    } else {
                        return;
                    }
                })
                .map_err(Error::source_spawn)?,
        );

        Ok(())
    }

    #[allow(dead_code)]
    pub fn stop(&mut self) -> Result<()> {
        let result: Result<()> = Ok(());
        result.map_err(Error::source_stop)
    }

    #[allow(dead_code)]
    pub fn done(&mut self) -> Result<()> {
        match self.thread.take() {
            Some(t) => t.join().map_err(|err| {
                if let Some(err) = err.downcast_ref::<&'static str>() {
                    Error::source_join(err)
                } else {
                    Error::source_join(format!("{:?}", err))
                }
            }),
            None => Ok(()),
        }
    }
}
