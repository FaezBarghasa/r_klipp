//! Async and sync helpers for framed I/O.
//!
//! This module provides a convenient `KlipperFramed` wrapper that combines an
//! async I/O transport (like a TCP stream or serial port) with a `KlipperCodec`
//! to create a `Stream` and `Sink` of `Message` objects.
//!
//! This module is only available with the `std` feature.

#![cfg(feature = "std")]

use crate::codec::KlipperCodec;
use crate::commands::Message;
use crate::Error;
use futures::{Sink, Stream};
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::io::{AsyncRead, AsyncWrite};
use tokio_util::codec::Framed;

/// A framed transport for Klipper messages.
///
/// This wraps an underlying `AsyncRead + AsyncWrite` stream and handles the
/// encoding and decoding of Klipper message frames.
pub struct KlipperFramed<T> {
    inner: Framed<T, KlipperCodec>,
}

impl<T> KlipperFramed<T>
where
    T: AsyncRead + AsyncWrite + Unpin,
{
    /// Creates a new `KlipperFramed` transport.
    ///
    /// # Arguments
    ///
    /// * `io` - The underlying I/O stream.
    pub fn new(io: T) -> Self {
        Self {
            inner: Framed::new(io, KlipperCodec::new()),
        }
    }
}

impl<T> Stream for KlipperFramed<T>
where
    T: AsyncRead + Unpin,
{
    type Item = Result<Message, Error>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        Pin::new(&mut self.inner).poll_next(cx)
    }
}

impl<T> Sink<Message> for KlipperFramed<T>
where
    T: AsyncWrite + Unpin,
{
    type Error = Error;

    fn poll_ready(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Pin::new(&mut self.inner).poll_ready(cx)
    }

    fn start_send(mut self: Pin<&mut Self>, item: Message) -> Result<(), Self::Error> {
        Pin::new(&mut self.inner).start_send(item)
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Pin::new(&mut self.inner).poll_flush(cx)
    }

    fn poll_close(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Pin::new(&mut self.inner).poll_close(cx)
    }
}

