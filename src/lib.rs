// Copyright 2016 Google Inc. All rights reserved.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! The main library for xi-core.

#[macro_use]
extern crate lazy_static;
extern crate bytecount;
extern crate memchr;
extern crate time;
extern crate syntect;
extern crate toml;
#[cfg(feature = "notify")]
extern crate notify;

pub mod editor;
pub mod rope;

/// Internal data structures and logic.
///
/// These internals are not part of the public API (for the purpose of binding to
/// a front-end), but are exposed here, largely so they appear in documentation.
pub mod tabs;
pub mod view;
pub mod linewrap;
// pub mod plugins;
pub mod styles;
pub mod word_boundaries;
pub mod index_set;
pub mod selection;
pub mod movement;
pub mod syntax;
pub mod layers;
pub mod config;
#[cfg(feature = "notify")]
pub mod watcher;
pub mod line_cache_shadow;
pub mod unicode;
pub mod rpc;

pub use syntax::SyntaxDefinition;
pub use config::{BufferItems as BufferConfig};

use tabs::{Documents, ViewIdentifier, BufferContainerRef};