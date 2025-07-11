use std::{fmt::Debug, io};

use iced::{
	Element,
	futures::{Stream, channel::mpsc},
	stream::channel,
	widget::{Column, text},
};
use tracing::instrument;
use tracing_subscriber::{fmt::MakeWriter, prelude::*, util::SubscriberInitExt};

use crate::{TRACING_GUARD, add_directive, env_filter};

#[derive(Debug, Clone)]
pub struct Event(pub Vec<(String, Option<tracing::Level>)>);

#[instrument(level = "trace")]
pub fn view<'a, Message: 'a>(events: &[Event]) -> Element<'a, Message> {
	Column::with_children(
		events
			.iter()
			.map(|event| text(event.0.first().unwrap().0.clone()).into()),
	)
	.into()
}

#[instrument(level = "trace")]
pub fn stream() -> impl Stream<Item = Event> {
	channel(0, |tx: mpsc::Sender<Event>| async move {
		let file_appender =
			tracing_appender::rolling::hourly(emtk_core::log_dir().await.unwrap(), "gui.log");
		let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);
		TRACING_GUARD.set(guard).unwrap();

		let filter = env_filter();
		let filter = add_directive(filter, "emtk_launcher=debug");
		let filter = add_directive(filter, "emtk_core=debug");

		let appender_filter = env_filter();
		let appender_filter = add_directive(appender_filter, "emtk_launcher=debug");
		let appender_filter = add_directive(appender_filter, "emtk_core=debug");

		let subscriber = tracing_subscriber::registry()
			.with(tracing_subscriber::fmt::layer().with_filter(filter))
			.with(
				tracing_subscriber::fmt::layer()
					.with_ansi(false)
					.with_writer(non_blocking)
					.with_filter(appender_filter),
			);

		#[cfg(debug_assertions)]
		let subscriber = {
			let tracy_filter = env_filter();
			let tracy_filter = add_directive(tracy_filter, "emtk_launcher=trace");
			let tracy_filter = add_directive(tracy_filter, "emtk_core=trace");
			subscriber.with(tracing_tracy::TracyLayer::default().with_filter(tracy_filter))
		};

		let gui_filter = env_filter();
		let gui_filter = add_directive(gui_filter, "emtk_launcher=debug");
		let gui_filter = add_directive(gui_filter, "emtk_core=debug");
		let subscriber = subscriber.with(
			tracing_subscriber::fmt::layer()
				.with_writer(Writer::new(tx))
				.with_ansi(false)
				.with_filter(gui_filter),
		);

		subscriber.init()
	})
}

pub struct Writer {
	level: Option<tracing::Level>,
	tx: mpsc::Sender<Event>,
}

impl Writer {
	pub fn new(tx: mpsc::Sender<Event>) -> Self {
		Self { level: None, tx }
	}
}

impl io::Write for Writer {
	fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
		self.tx
			.try_send(Event(vec![(
				String::from_utf8_lossy(buf).to_string(),
				self.level,
			)]))
			.unwrap();
		Ok(buf.len())
	}

	fn flush(&mut self) -> io::Result<()> {
		Ok(())
	}
}

impl<'a> MakeWriter<'a> for Writer {
	type Writer = Self;

	fn make_writer(&'a self) -> Self::Writer {
		Self::new(self.tx.clone())
	}

	fn make_writer_for(&'a self, meta: &tracing::Metadata<'_>) -> Self::Writer {
		Self {
			level: Some(*meta.level()),
			tx: self.tx.clone(),
		}
	}
}

// impl fmt::Write for Writer {
// 	fn write_str(&mut self, s: &str) -> fmt::Result {
// 		println!("fmt write called");
// 		Ok(())
// 	}
// }

// 	fn make_writer_for(&'a self, meta: &tracing::Metadata<'_>) -> Self::Writer {
// 		Self {
// 			level: Some(*meta.level()),
// 			tx: self.tx.clone(),
// 		}
// 	}
// }

// pub struct Format {
// 	timer: SystemTime,
// 	tx: mpsc::Sender<Event>,
// }

// impl Format {
// 	pub fn new(tx: mpsc::Sender<Event>) -> Self {
// 		Self {
// 			timer: SystemTime,
// 			tx,
// 		}
// 	}
// }

// impl<S, N> FormatEvent<S, N> for Format
// where
// 	S: Subscriber + for<'a> LookupSpan<'a>,
// 	N: for<'a> FormatFields<'a> + 'static,
// {
// 	fn format_event(
// 		&self,
// 		ctx: &FmtContext<'_, S, N>,
// 		_writer: format::Writer<'_>,
// 		event: &tracing::Event<'_>,
// 	) -> fmt::Result {
// 		let meta = event.metadata();
// 		let mut timestamp = String::new();
// 		let mut level = String::new();
// 		let mut thread_name = String::new();
// 		let mut thread_id = String::new();
// 		let mut scope = String::new();
// 		let mut target = String::new();
// 		let mut filename = String::new();
// 		let mut fields = String::new();

// 		{
// 			let mut w = format::Writer::new(&mut timestamp);
// 			self.timer.format_time(&mut w)?;
// 			w.write_char(' ')?;

// 			let mut w = format::Writer::new(&mut level);
// 			let level = match *meta.level() {
// 				tracing::Level::TRACE => "TRACE",
// 				tracing::Level::DEBUG => "DEBUG",
// 				tracing::Level::INFO => "INFO",
// 				tracing::Level::WARN => "WARN",
// 				tracing::Level::ERROR => "ERROR",
// 			};
// 			write!(w, "{} ", level)?;

// 			let current_thread = std::thread::current();
// 			if let Some(name) = current_thread.name() {
// 				use std::sync::atomic::{
// 					AtomicUsize,
// 					Ordering::{AcqRel, Acquire, Relaxed},
// 				};

// 				// Track the longest thread name length we've seen so far in an atomic,
// 				// so that it can be updated by any thread.
// 				static MAX_LEN: AtomicUsize = AtomicUsize::new(0);
// 				let len = name.len();
// 				// Snapshot the current max thread name length.
// 				let mut max_len = MAX_LEN.load(Relaxed);

// 				while len > max_len {
// 					// Try to set a new max length, if it is still the value we took a
// 					// snapshot of.
// 					match MAX_LEN.compare_exchange(max_len, len, AcqRel, Acquire) {
// 						// We successfully set the new max value
// 						Ok(_) => break,
// 						// Another thread set a new max value since we last observed
// 						// it! It's possible that the new length is actually longer than
// 						// ours, so we'll loop again and check whether our length is
// 						// still the longest. If not, we'll just use the newer value.
// 						Err(actual) => max_len = actual,
// 					}
// 				}

// 				// pad thread name using `max_len`
// 				write!(thread_name, "{:>width$} ", name, width = max_len)?;
// 			}

// 			write!(thread_id, "{:0>2?} ", current_thread.id())?;

// 			if let Some(event_scope) = ctx.event_scope() {
// 				let mut seen = false;
// 				for span in event_scope.from_root() {
// 					write!(scope, "{}", span.metadata().name())?;
// 					seen = true;

// 					let ext = span.extensions();
// 					if let Some(fields) = &ext.get::<FormattedFields<N>>() {
// 						if !fields.is_empty() {
// 							write!(scope, "{{{}}}", fields)?;
// 						}
// 					}
// 					write!(scope, ":")?;
// 				}

// 				if seen {
// 					scope.push(' ');
// 				}
// 			}

// 			write!(target, "{}: ", meta.target())?;

// 			{
// 				let line_number = meta.line();
// 				if let Some(name) = meta.file() {
// 					write!(
// 						filename,
// 						"{}:{}",
// 						name,
// 						if line_number.is_some() { "" } else { " " }
// 					)?
// 				}

// 				if let Some(line_number) = line_number {
// 					write!(filename, "{}: ", line_number)?;
// 				}
// 			}

// 			let w = format::Writer::new(&mut fields);
// 			ctx.format_fields(w, event)?;
// 			fields.write_str("\n")?;
// 		}

// 		let e = Event(vec![
// 			timestamp,
// 			level,
// 			thread_name,
// 			thread_id,
// 			scope,
// 			target,
// 			filename,
// 			fields,
// 		]);

// 		self.tx.clone().try_send(e).unwrap();

// 		Ok(())
// 	}
// }
