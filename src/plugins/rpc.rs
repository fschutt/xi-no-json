// Copyright 2017 Google Inc. All rights reserved.
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

//! RPC types, corresponding to protocol requests, notifications & responses.

use std::path::PathBuf;

use rope::rope::RopeDelta;
use super::PluginPid;
use syntax::SyntaxDefinition;
use tabs::{BufferIdentifier, ViewIdentifier};
use config::Table;

//TODO: At the moment (May 08, 2017) this is all very much in flux.
// At some point, it will be stabalized and then perhaps will live in another crate,
// shared with the plugin lib.

// ====================================================================
// core -> plugin RPC method types + responses
// ====================================================================

/// Buffer information sent on plugin init.
#[derive(Debug, Clone)]
pub struct PluginBufferInfo {
    /// The buffer's unique identifier.
    pub buffer_id: BufferIdentifier,
    /// The buffer's current views.
    pub views: Vec<ViewIdentifier>,
    pub rev: u64,
    pub buf_size: usize,
    pub nb_lines: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    pub syntax: SyntaxDefinition,
    pub config: Table,
}

//TODO: very likely this should be merged with PluginDescription
//TODO: also this does not belong here.
/// Describes an available plugin to the client.
#[derive(Debug)]
pub struct ClientPluginInfo {
    pub name: String,
    pub running: bool,
}

/// A simple update, sent to a plugin.
#[derive(  Debug, Clone)]
pub struct PluginUpdate {
    pub view_id: ViewIdentifier,
    /// The delta representing changes to the document.
    ///
    /// Note: Is `Some` in the general case; only if the delta involves
    /// inserting more than some maximum number of bytes, will this be `None`,
    /// indicating the plugin should flush cache and fetch manually.
    pub delta: Option<RopeDelta>,
    /// The size of the document after applying this delta.
    pub new_len: usize,
    pub rev: u64,
    pub edit_type: String,
    pub author: String,
}

/// A response to an `update` RPC sent to a plugin.
#[derive(  Debug)]
#[serde(untagged)]
pub enum UpdateResponse {
    /// An edit to the buffer.
    Edit(PluginEdit),
    /// An acknowledgement with no action. A response cannot be Null,
    /// so we send a uint.
    Ack(u64),
}

#[derive(Debug, Clone)]
pub struct EmptyStruct {}

#[derive(Debug, Clone)]
#[serde(rename_all = "snake_case")]
#[serde(tag = "method", content = "params")]
/// RPC requests sent from the host
pub enum HostRequest {
    Update(PluginUpdate),
}

#[derive(Debug, Clone)]
#[serde(rename_all = "snake_case")]
#[serde(tag = "method", content = "params")]
/// RPC Notifications sent from the host
pub enum HostNotification {
    Ping(EmptyStruct),
    Initialize { plugin_id: PluginPid, buffer_info: Vec<PluginBufferInfo> },
    DidSave { view_id: ViewIdentifier, path: PathBuf },
    ConfigChanged { view_id: ViewIdentifier, changes: Table },
    NewBuffer { buffer_info: Vec<PluginBufferInfo> },
    DidClose { view_id: ViewIdentifier },
    Shutdown(EmptyStruct),
}


// ====================================================================
// plugin -> core RPC method types
// ====================================================================


/// A simple edit, received from a plugin.
#[derive(  Debug, Clone)]
pub struct PluginEdit {
    pub rev: u64,
    pub delta: RopeDelta,
    /// the edit priority determines the resolution strategy when merging
    /// concurrent edits. The highest priority edit will be applied last.
    pub priority: u64,
    /// whether the inserted text prefers to be to the right of the cursor.
    pub after_cursor: bool,
    /// the originator of this edit: some identifier (plugin name, 'core', etc)
    pub author: String,
}

#[derive(  Debug, Clone, Copy)]
pub struct ScopeSpan {
    pub start: usize,
    pub end: usize,
    pub scope_id: u32,
}

#[derive(  Debug, Clone)]
#[serde(rename_all = "snake_case")]
#[serde(tag = "method", content = "params")]
/// RPC requests sent from plugins.
pub enum PluginRequest {
    GetData { offset: usize, max_size: usize, rev: u64 },
    LineCount,
    GetSelections,
}

#[derive(  Debug, Clone)]
#[serde(rename_all = "snake_case")]
#[serde(tag = "method", content = "params")]
/// RPC commands sent from plugins.
pub enum PluginNotification {
    AddScopes { scopes: Vec<Vec<String>> },
    UpdateSpans { start: usize, len: usize, spans: Vec<ScopeSpan>, rev: u64 },
    Edit { edit: PluginEdit },
    Alert { msg: String },
}

/// Common wrapper for plugin-originating RPCs.
pub struct PluginCommand<T> {
    pub view_id: ViewIdentifier,
    pub plugin_id: PluginPid,
    pub cmd: T,
}

impl PluginBufferInfo {
    pub fn new(buffer_id: BufferIdentifier, views: &[ViewIdentifier],
               rev: u64, buf_size: usize, nb_lines: usize,
               path: Option<PathBuf>, syntax: SyntaxDefinition,
               config: Table) -> Self {
        //TODO: do make any current assertions about paths being valid utf-8? do we want to?
        let path = path.map(|p| p.to_str().unwrap().to_owned());
        let views = views.to_owned();
        PluginBufferInfo { buffer_id, views, rev, buf_size,
        nb_lines, path, syntax, config }
    }
}

impl PluginUpdate {
    pub fn new<D>(view_id: ViewIdentifier, rev: u64, delta: D, new_len: usize,
                  edit_type: String, author: String) -> Self
        where D: Into<Option<RopeDelta>>
    {
        let delta = delta.into();
        PluginUpdate { view_id, delta, new_len, rev, edit_type, author }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

    #[test]
    fn test_plugin_update() {
        let json = r#"{
            "view_id": "view-id-42",
            "delta": {"base_len": 6, "els": [{"copy": [0,5]}, {"insert":"rofls"}]},
            "new_len": 404,
            "rev": 5,
            "edit_type": "something",
            "author": "me"
    }"#;

    let val: PluginUpdate = match serde_json::from_str(json) {
        Ok(val) => val,
        Err(err) => panic!("{:?}", err),
    };
    assert!(val.delta.is_some());
    assert!(val.delta.unwrap().as_simple_insert().is_some());
    }

    #[test]
    fn test_deserde_init() {
        let json = r#"
            {"buffer_id": 42,
             "views": ["view-id-4"],
             "rev": 1,
             "buf_size": 20,
             "nb_lines": 5,
             "path": "some_path",
             "syntax": "toml",
             "config": {"some_key": 420}}"#;

        let val: PluginBufferInfo = match serde_json::from_str(json) {
            Ok(val) => val,
            Err(err) => panic!("{:?}", err),
        };
        assert_eq!(val.rev, 1);
        assert_eq!(val.path, Some("some_path".to_owned()));
        assert_eq!(val.syntax, SyntaxDefinition::Toml);
    }

    #[test]
    fn test_de_plugin_rpc() {
        let json = r#"{"method": "alert", "params": {"view_id": "view-id-1", "plugin_id": 42, "msg": "ahhh!"}}"#;
        let de: PluginCommand<PluginNotification> = serde_json::from_str(json).unwrap();
        assert_eq!(de.view_id, "view-id-1".into());
        assert_eq!(de.plugin_id, PluginPid(42));
        match de.cmd {
            PluginNotification::Alert { ref msg } if msg == "ahhh!" => (),
            _ => panic!("{:?}", de.cmd),
        }
    }
}
