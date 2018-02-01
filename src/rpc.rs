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

//! The main RPC protocol, for communication between `xi-core` and the client.
//!
//! We rely on [Serde] for serialization and deserialization between
//! the JSON-RPC protocol and the types here.
//!
//! [Serde]: https://serde.rs


use std::path::PathBuf;

use tabs::ViewIdentifier;
use config::{ConfigDomain};

// =============================================================================
//  Command types
// =============================================================================

#[derive(Debug, PartialEq)]
#[doc(hidden)]
pub struct EmptyStruct {}

/// The notifications which make up the base of the protocol.
///
/// # Note
///
/// For serialization, all identifiers are converted to "snake_case".
///
/// # Examples
///
/// The `close_view` command:
///
/// ```
/// # extern crate xi_core_lib as xi_core;
/// extern crate serde_json;
/// # fn main() {
/// use xi_core::rpc::CoreNotification;
///
/// let json = r#"{
///     "method": "close_view",
///     "params": { "view_id": "view-id-1" }
///     }"#;
///
/// let cmd: CoreNotification = serde_json::from_str(&json).unwrap();
/// match cmd {
///     CoreNotification::CloseView { .. } => (), // expected
///     other => panic!("Unexpected variant"),
/// }
/// # }
/// ```
///
/// The `client_started` command:
///
/// ```
/// # extern crate xi_core_lib as xi_core;
/// extern crate serde_json;
/// # fn main() {
/// use xi_core::rpc::CoreNotification;
///
/// let json = r#"{
///     "method": "client_started",
///     "params": {}
///     }"#;
///
/// let cmd: CoreNotification = serde_json::from_str(&json).unwrap();
/// match cmd {
///     CoreNotification::ClientStarted { .. }  => (), // expected
///     other => panic!("Unexpected variant"),
/// }
/// # }
/// ```
#[derive(Debug, PartialEq)]
pub enum CoreNotification {
    /// The 'edit' namespace, for view-specific editor actions.
    ///
    /// The params object has internal `method` and `params` members,
    /// which are parsed into the appropriate `EditNotification`.
    ///
    /// # Note:
    ///
    /// All edit commands (notifications and requests) include in their
    /// inner params object a `view_id` field. On the xi-core side, we
    /// pull out this value during parsing, and use it for routing.
    ///
    /// For more on the edit commands, see [`EditNotification`] and
    /// [`EditRequest`].
    ///
    /// [`EditNotification`]: enum.EditNotification.html
    /// [`EditRequest`]: enum.EditRequest.html
    ///
    /// # Examples
    ///
    /// ```
    /// # extern crate xi_core_lib as xi_core;
    /// #[macro_use]
    /// extern crate serde_json;
    /// use xi_core::rpc::*;
    /// # fn main() {
    /// let edit = EditCommand {
    ///     view_id: "view-id-1".into(),
    ///     cmd: EditNotification::Insert { chars: "hello!".into() },
    /// };
    /// let rpc = CoreNotification::Edit(edit);
    /// let expected = json!({
    ///     "method": "edit",
    ///     "params": {
    ///         "method": "insert",
    ///         "params": {
    ///             "view_id": "view-id-1",
    ///             "chars": "hello!",
    ///         }
    ///     }
    /// });
    /// assert_eq!(serde_json::to_value(&rpc).unwrap(), expected);
    /// # }
    /// ```
    Edit(EditCommand<EditNotification>),
    /// The 'plugin' namespace, for interacting with plugins.
    ///
    /// As with edit commands, the params object has is a nested RPC,
    /// with the name of the command included as the `command` field.
    ///
    /// (this should be changed to more accurately reflect the behaviour
    /// of the edit commands).
    ///
    ///For the available commands, see [`PluginNotification`].
    ///
    /// [`PluginNotification`]: enum.PluginNotification.html
    ///
    /// # Examples
    ///
    /// ```
    /// # extern crate xi_core_lib as xi_core;
    /// #[macro_use]
    /// extern crate serde_json;
    /// use xi_core::rpc::*;
    /// # fn main() {
    /// let rpc = CoreNotification::Plugin(
    ///     PluginNotification::Start {
    ///         view_id: "view-id-1".into(),
    ///         plugin_name: "syntect".into(),
    ///     });
    ///
    /// let expected = json!({
    ///     "method": "plugin",
    ///     "params": {
    ///         "command": "start",
    ///         "view_id": "view-id-1",
    ///         "plugin_name": "syntect",
    ///     }
    /// });
    /// assert_eq!(serde_json::to_value(&rpc).unwrap(), expected);
    /// # }
    /// ```
    Plugin(PluginNotification),
    /// Tells `xi-core` to close the specified view.
    CloseView { view_id: ViewIdentifier },
    /// Tells `xi-core` to save the contents of the specified view's
    /// buffer to the specified path.
    Save { view_id: ViewIdentifier, file_path: String },
    /// Tells `xi-core` to set the theme.
    SetTheme { theme_name: String },
    /// Notifies `xi-core` that the client has started.
    ClientStarted {
        config_dir: Option<PathBuf>,
        /// Path to additional plugins, included by the client.
        client_extras_dir: Option<PathBuf>,
    },
    // Updates the user's config for the given domain. Where keys in
    // `changes` are `null`, those keys are cleared in the user config
    // for that domain; otherwise the config is updated with the new
    // value.
    //
    // Note: If the client is using file-based config, the only valid
    // domain argument is `ConfigDomain::UserOverride(_)`, which
    // represents non-persistent view-specific settings, such as when
    // a user manually changes whitespace settings for a given view.
    // ModifyUserConfig { domain: ConfigDomain, changes: Table },
}

/// The requests which make up the base of the protocol.
///
/// All requests expect a response.
///
/// # Examples
///
/// The `new_view` command:
/// ```
/// # extern crate xi_core_lib as xi_core;
/// extern crate serde_json;
/// # fn main() {
/// use xi_core::rpc::CoreRequest;
///
/// let json = r#"{
///     "method": "new_view",
///     "params": { "file_path": "~/my_very_fun_file.rs" }
///     }"#;
///
/// let cmd: CoreRequest = serde_json::from_str(&json).unwrap();
/// match cmd {
///     CoreRequest::NewView { .. } => (), // expected
///     other => panic!("Unexpected variant {:?}", other),
/// }
/// # }
/// ```
#[derive(Debug, PartialEq)]
pub enum CoreRequest {
    /// The 'edit' namespace, for view-specific requests.
    Edit(EditCommand<EditRequest>),
    /// Tells `xi-core` to create a new view. If the `file_path`
    /// argument is present, `xi-core` should attempt to open the file
    /// at that location.
    ///
    /// Returns the view identifier that should be used to interact
    /// with the newly created view.
    NewView { file_path: Option<String> },
    /// Returns the current collated config object for the given view.
    GetConfig { view_id: ViewIdentifier },
}

/// A helper type, which extracts the `view_id` field from edit
/// requests and notifications.
///
/// Edit requests and notifications have 'method', 'params', and
/// 'view_id' param members. We use this wrapper, which has custom
/// `Deserialize` and `Serialize` implementations, to pull out the
/// `view_id` field.
///
/// # Examples
///
/// ```
/// # extern crate xi_core_lib as xi_core;
/// extern crate serde_json;
/// # fn main() {
/// use xi_core::rpc::*;
///
/// let json = r#"{
///     "view_id": "view-id-1",
///     "method": "scroll",
///     "params": [0, 6]
///     }"#;
///
/// let cmd: EditCommand<EditNotification> = serde_json::from_str(&json).unwrap();
/// match cmd.cmd {
///     EditNotification::Scroll( .. ) => (), // expected
///     other => panic!("Unexpected variant {:?}", other),
/// }
/// # }
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct EditCommand<T> {
    pub view_id: ViewIdentifier,
    pub cmd: T,
}

/// An enum representing touch and mouse gestures applied to the text.
#[derive(PartialEq, Eq, Debug)]
pub enum GestureType {
    ToggleSel,
}

/// An inclusive range.
///
/// # Note:
///
/// Several core protocol commands use a params array to pass arguments
/// which are named, internally. this type use custom Serialize /
/// Deserialize impls to accomodate this.
#[derive(PartialEq, Eq, Debug)]
pub struct LineRange {
    pub first: i64,
    pub last: i64,
}

/// A mouse event. See the note for [`LineRange`].
///
/// [`LineRange`]: enum.LineRange.html
#[derive(PartialEq, Eq, Debug)]
pub struct MouseAction {
    pub line: u64,
    pub column: u64,
    pub flags: u64,
    pub click_count: Option<u64>,
}

/// The edit-related notifications.
///
/// Alongside the [`EditRequest`] members, these commands constitute
/// the API for interacting with a particular window and document.
#[derive(Debug, PartialEq)]
pub enum EditNotification {
    Insert { chars: String },
    DeleteForward,
    DeleteBackward,
    DeleteWordForward,
    DeleteWordBackward,
    DeleteToEndOfParagraph,
    DeleteToBeginningOfLine,
    InsertNewline,
    InsertTab,
    MoveUp,
    MoveUpAndModifySelection,
    MoveDown,
    MoveDownAndModifySelection,
    MoveLeft,
    // synoynm for `MoveLeft`
    MoveBackward,
    MoveLeftAndModifySelection,
    MoveRight,
    // synoynm for `MoveRight`
    MoveForward,
    MoveRightAndModifySelection,
    MoveWordLeft,
    MoveWordLeftAndModifySelection,
    MoveWordRight,
    MoveWordRightAndModifySelection,
    MoveToBeginningOfParagraph,
    MoveToEndOfParagraph,
    MoveToLeftEndOfLine,
    MoveToLeftEndOfLineAndModifySelection,
    MoveToRightEndOfLine,
    MoveToRightEndOfLineAndModifySelection,
    MoveToBeginningOfDocument,
    MoveToBeginningOfDocumentAndModifySelection,
    MoveToEndOfDocument,
    MoveToEndOfDocumentAndModifySelection,
    ScrollPageUp,
    PageUpAndModifySelection,
    ScrollPageDown,
    PageDownAndModifySelection,
    SelectAll,
    AddSelectionAbove,
    AddSelectionBelow,
    Scroll(LineRange),
    GotoLine { line: u64 },
    RequestLines(LineRange),
    Yank,
    Transpose,
    Click(MouseAction),
    Drag(MouseAction),
    Gesture { line: u64, col: u64, ty: GestureType},
    Undo,
    Redo,
    FindNext { wrap_around: Option<bool>, allow_same: Option<bool> },
    FindPrevious { wrap_around: Option<bool> },
    DebugRewrap,
    /// Prints the style spans present in the active selection.
    DebugPrintSpans,
    CancelOperation,
}

/// The edit related requests.
#[derive(Debug, PartialEq)]
pub enum EditRequest {
    /// Cuts the active selection, returning their contents,
    /// or `Null` if the selection was empty.
    Cut,
    /// Copies the active selection, returning their contents or
    /// or `Null` if the selection was empty.
    Copy,
    /// Searches the document for `chars`, if present, falling back on
    /// the last selection region if `chars` is `None`.
    ///
    /// If `chars` is `None` and there is an active selection, returns
    /// the string value used for the search, else returns `Null`.
    Find { chars: Option<String>, case_sensitive: bool },
}


/// The plugin related notifications.
#[derive(Debug, PartialEq)]
pub enum PluginNotification {
    Start { view_id: ViewIdentifier, plugin_name: String },
    Stop { view_id: ViewIdentifier, plugin_name: String },
    /*PluginRpc { view_id: ViewIdentifier, receiver: String, rpc: PlaceholderRpc },*/
}
