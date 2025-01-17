#![doc(html_root_url = "https://docs.rs/http-body/0.2.0-alpha.1")]
#![deny(missing_debug_implementations, missing_docs, unreachable_pub)]
#![cfg_attr(test, deny(warnings))]

//! Asynchronous HTTP request or response body.
//!
//! See [`Body`] for more details.
//!
//! [`Body`]: trait.Body.html

mod size_hint;

pub use self::size_hint::SizeHint;

use bytes::Buf;
use http::HeaderMap;
use std::ops;
use std::pin::Pin;
use std::task::{Context, Poll};

/// Trait representing a streaming body of a Request or Response.
///
/// Data is streamed via the `poll_data` function, which asynchronously yields `T: Buf` values. The
/// `size_hint` function provides insight into the total number of bytes that will be streamed.
///
/// The `poll_trailers` function returns an optional set of trailers used to finalize the request /
/// response exchange. This is mostly used when using the HTTP/2.0 protocol.
///
pub trait Body {
    /// Values yielded by the `Body`.
    type Data: Buf;

    /// The error type this `Body` might generate.
    type Error;

    /// Attempt to pull out the next data buffer of this stream.
    fn poll_data(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Self::Data, Self::Error>>>;

    /// Poll for an optional **single** `HeaderMap` of trailers.
    ///
    /// This function should only be called once `poll_data` returns `None`.
    fn poll_trailers(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Result<Option<HeaderMap>, Self::Error>>;

    /// Returns `true` when the end of stream has been reached.
    ///
    /// An end of stream means that both `poll_data` and `poll_trailers` will
    /// return `None`.
    ///
    /// A return value of `false` **does not** guarantee that a value will be
    /// returned from `poll_stream` or `poll_trailers`.
    fn is_end_stream(&self) -> bool {
        false
    }

    /// Returns the bounds on the remaining length of the stream.
    ///
    /// When the **exact** remaining length of the stream is known, the upper bound will be set and
    /// will equal the lower bound.
    fn size_hint(&self) -> SizeHint {
        SizeHint::default()
    }
}

impl<T: Body + Unpin + ?Sized> Body for &mut T {
    type Data = T::Data;
    type Error = T::Error;

    fn poll_data(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Self::Data, Self::Error>>> {
        Pin::new(&mut **self).poll_data(cx)
    }

    fn poll_trailers(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Result<Option<HeaderMap>, Self::Error>> {
        Pin::new(&mut **self).poll_trailers(cx)
    }

    fn is_end_stream(&self) -> bool {
        Pin::new(&**self).is_end_stream()
    }

    fn size_hint(&self) -> SizeHint {
        Pin::new(&**self).size_hint()
    }
}

impl<P> Body for Pin<P>
where
    P: Unpin + ops::DerefMut,
    P::Target: Body,
{
    type Data = <<P as ops::Deref>::Target as Body>::Data;
    type Error = <<P as ops::Deref>::Target as Body>::Error;

    fn poll_data(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Self::Data, Self::Error>>> {
        Pin::get_mut(self).as_mut().poll_data(cx)
    }

    fn poll_trailers(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Result<Option<HeaderMap>, Self::Error>> {
        Pin::get_mut(self).as_mut().poll_trailers(cx)
    }

    fn is_end_stream(&self) -> bool {
        self.as_ref().is_end_stream()
    }

    fn size_hint(&self) -> SizeHint {
        self.as_ref().size_hint()
    }
}

impl<T: Body + Unpin + ?Sized> Body for Box<T> {
    type Data = T::Data;
    type Error = T::Error;

    fn poll_data(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Self::Data, Self::Error>>> {
        Pin::new(&mut **self).poll_data(cx)
    }

    fn poll_trailers(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Result<Option<HeaderMap>, Self::Error>> {
        Pin::new(&mut **self).poll_trailers(cx)
    }

    fn is_end_stream(&self) -> bool {
        self.as_ref().is_end_stream()
    }

    fn size_hint(&self) -> SizeHint {
        self.as_ref().size_hint()
    }
}
