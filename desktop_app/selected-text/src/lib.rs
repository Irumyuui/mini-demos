use cfg_if::cfg_if;

cfg_if! {
    if #[cfg(target_os = "windows")] {
        mod windows;

        use crate::windows::get_selected_text as get_selected_text_impl;
        pub use crate::windows::Error as Error;
    }
}

/// Get the text selected by the cursor.
///
/// # Example
///
/// ```
/// use selected_text::get_selected_text;
///
/// let text = get_selected_text().expect("Failed to get selected text");
/// println!("Selected text: {}", text);
/// ```
pub fn get_selected_text() -> Result<String, Error> {
    get_selected_text_impl().map(|s| s.trim().to_owned())
}
