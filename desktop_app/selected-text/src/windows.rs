use std::{thread, time::Duration};

use arboard::Clipboard;
use enigo::{Direction, Enigo, Key, Keyboard, Settings};
use windows::Win32::{
    System::{
        Com::{CLSCTX_ALL, CoCreateInstance, CoInitialize},
        DataExchange::GetClipboardSequenceNumber,
    },
    UI::Accessibility::{
        CUIAutomation, IUIAutomation, IUIAutomationTextPattern, UIA_TextPatternId,
    },
};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Com(#[from] windows::core::Error),

    #[error(transparent)]
    Clipboard(#[from] arboard::Error),

    #[error(transparent)]
    Copied(#[from] self::CopyError),

    #[error("{0}")]
    Custom(String),
}

pub fn get_selected_text() -> Result<String, Error> {
    let err1 = match unsafe { get_text_by_automation() } {
        Ok(text) if !text.is_empty() => return Ok(text),
        Ok(_) => {
            tracing::debug!("first step found empty text");
            None
        }
        Err(e) => {
            tracing::error!("first step error: {e}");
            Some(e)
        }
    };

    match get_selected_text_by_copied() {
        Ok(text) if !text.is_empty() => return Ok(text),
        Ok(_) => {}
        Err(e) => return Err(e), // will ignore first error...
    }

    match err1 {
        Some(e) => Err(e),
        None => Ok(String::new()),
    }
}

unsafe fn get_text_by_automation() -> Result<String, Error> {
    unsafe {
        let _ = CoInitialize(None);
        let auto: IUIAutomation = CoCreateInstance(&CUIAutomation, None, CLSCTX_ALL)?;
        let patten: IUIAutomationTextPattern = auto
            .GetFocusedElement()?
            .GetCurrentPatternAs(UIA_TextPatternId)?;
        let text_array = patten.GetSelection()?;
        let len = text_array.Length()?;
        let mut result = String::new();

        for i in 0..len {
            let text = text_array.GetElement(i)?;
            let str = text.GetText(-1)?.to_string();
            result.push_str(&str);
        }

        Ok(result.trim().to_string())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum CopyError {
    #[error(transparent)]
    NewEnigo(#[from] enigo::NewConError),

    #[error(transparent)]
    Input(#[from] enigo::InputError),

    #[error("Clipboard not copied")]
    NotCopied,
}

fn do_copy() -> Result<(), CopyError> {
    let num_before = unsafe { GetClipboardSequenceNumber() };

    let mut enigo = Enigo::new(&Settings::default())?;
    enigo.key(Key::Control, Direction::Release)?;
    enigo.key(Key::C, Direction::Release)?;
    enigo.key(Key::Control, Direction::Press)?;
    enigo.key(Key::C, Direction::Click)?;
    enigo.key(Key::Control, Direction::Release)?;
    thread::sleep(Duration::from_millis(100));

    let num_after = unsafe { GetClipboardSequenceNumber() };

    match num_before != num_after {
        true => Ok(()),
        false => Err(CopyError::NotCopied),
    }
}

fn get_selected_text_by_copied() -> Result<String, Error> {
    let text = Clipboard::new()?.get_text();
    let image = Clipboard::new()?.get_image();

    do_copy()?;

    let new_text = Clipboard::new()?.get_text();
    let mut cb = Clipboard::new()?;

    let handle = || {
        if let Ok(new) = new_text {
            Ok(new.trim().to_string())
        } else {
            Err(Error::Custom("Not text".into()))
        }
    };

    match (text, image) {
        (Ok(text), ..) => cb.set_text(text)?,
        (_, Ok(image)) => cb.set_image(image)?,
        _ => cb.clear()?,
    }
    handle()
}
