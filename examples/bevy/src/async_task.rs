use std::{
    marker::PhantomData,
    sync::{mpsc, Mutex},
};

use bevy::prelude::*;

/// Async Task.
#[derive(Component)]
pub struct AsyncTask<M, T> {
    result: Option<T>,
    rx: Mutex<mpsc::Receiver<T>>,
    marker: PhantomData<M>,
}

/// Async Error.
pub enum AsyncError {
    /// Receiver is disconnected.
    Disconnected,

    /// Mutex is poisoned.
    Poisoned,
}

impl<M, T> AsyncTask<M, T> {
    /// Creates a new [`AsyncTask`].
    pub fn new(rx: Mutex<mpsc::Receiver<T>>) -> Self {
        Self {
            result: None,
            rx,
            marker: PhantomData,
        }
    }

    /// Polls an [`AsyncTask`] once.
    pub fn poll_once(&mut self) -> Result<&Option<T>, AsyncError> {
        if self.result.is_some() {
            return Ok(&self.result);
        }

        let rx = self.rx.lock().map_err(|_| AsyncError::Poisoned)?;

        match rx.try_recv() {
            Ok(result) => {
                self.result = Some(result);
            }
            Err(mpsc::TryRecvError::Disconnected) => return Err(AsyncError::Disconnected),
            Err(mpsc::TryRecvError::Empty) => {}
        };

        Ok(&self.result)
    }
}
