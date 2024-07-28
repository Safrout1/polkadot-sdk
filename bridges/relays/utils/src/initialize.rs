// Copyright 2019-2021 Parity Technologies (UK) Ltd.
// This file is part of Parity Bridges Common.

// Parity Bridges Common is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity Bridges Common is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity Bridges Common.  If not, see <http://www.gnu.org/licenses/>.

//! Relayer initialization functions.

use console::style;
use parking_lot::Mutex;
use std::{cell::RefCell, fmt::Display, io::Write};
use sp_tracing::tracing_subscriber;
use sp_tracing::tracing::Level;
use sp_tracing::tracing_subscriber::fmt::time::OffsetTime;
use sp_tracing::tracing_subscriber::fmt::SubscriberBuilder;
use sp_tracing::tracing_subscriber::EnvFilter;

/// Relayer version that is provided as metric. Must be set by a binary
/// (get it with `option_env!("CARGO_PKG_VERSION")` from a binary package code).
pub static RELAYER_VERSION: Mutex<Option<String>> = Mutex::new(None);

async_std::task_local! {
	pub(crate) static LOOP_NAME: RefCell<String> = RefCell::new(String::default());
}

/// Initialize relay environment.
pub fn initialize_relay() {
	initialize_logger(true);
}

/// Initialize Relay logger instance.
pub fn initialize_logger(with_timestamp: bool) {
	let format = time::format_description::parse(
		"[year]-[month]-[day] \
		[hour repr:24]:[minute]:[second] [offset_hour sign:mandatory]",
	)
	.expect("static format string is valid");

	let local_time = OffsetTime::new(
		time::UtcOffset::current_local_offset().unwrap_or(time::UtcOffset::UTC),
		format,
	);

	let builder = SubscriberBuilder::default()
		.with_env_filter(EnvFilter::from_default_env())
		.with_filter(Level::WARN)
		.with_filter_module("bridge", Level::INFO);

	let builder = if with_timestamp {
		builder.with_timer(local_time)
	} else {
		builder.without_time()
	};

	builder.init();
}

/// Initialize relay loop. Must only be called once per every loop task.
pub(crate) fn initialize_loop(loop_name: String) {
	LOOP_NAME.with(|g_loop_name| *g_loop_name.borrow_mut() = loop_name);
}

/// Returns loop name prefix to use in logs. The prefix is initialized with the `initialize_loop`
/// call.
fn loop_name_prefix() -> String {
	// try_with to avoid panic outside of async-std task context
	LOOP_NAME
		.try_with(|loop_name| {
			// using borrow is ok here, because loop is only initialized once (=> borrow_mut will
			// only be called once)
			let loop_name = loop_name.borrow();
			if loop_name.is_empty() {
				String::new()
			} else {
				format!("[{loop_name}] ")
			}
		})
		.unwrap_or_else(|_| String::new())
}

enum Either<A, B> {
	Left(A),
	Right(B),
}
impl<A: Display, B: Display> Display for Either<A, B> {
	fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
		match self {
			Self::Left(a) => write!(fmt, "{a}"),
			Self::Right(b) => write!(fmt, "{b}"),
		}
	}
}

fn color_target(target: &str) -> impl Display + '_ {
	if cfg!(windows) {
		Either::Left(target)
	} else {
		Either::Right(style(target).black().bright().to_string())
	}
}

fn color_level(level: log::Level) -> impl Display {
	if cfg!(windows) {
		Either::Left(level)
	} else {
		let s = level.to_string();
		Either::Right(match level {
			log::Level::Error => style(s).red().bright().bold().to_string(),
			log::Level::Warn => style(s).yellow().bright().bold().to_string(),
			log::Level::Info => style(s).green().bright().to_string(),
			log::Level::Debug => style(s).cyan().bright().to_string(),
			log::Level::Trace => style(s).blue().bright().to_string(),
		})
	}
}
