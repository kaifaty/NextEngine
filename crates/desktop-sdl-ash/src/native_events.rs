use sdl3::event::{Event, WindowEvent};
use sdl3::keyboard::{Keycode, Mod, Scancode};
use sdl3::mouse::MouseState;

use crate::{DesktopAdapterError, sdl_error};

pub(super) fn inject_startup_lifecycle_probe(
    event_subsystem: &sdl3::EventSubsystem,
    window_id: u32,
    extent: [u32; 2],
) -> Result<(), DesktopAdapterError> {
    let width = i32::try_from(extent[0]).map_err(|_| DesktopAdapterError::InvalidExtent)?;
    let height = i32::try_from(extent[1]).map_err(|_| DesktopAdapterError::InvalidExtent)?;
    let probe = [
        Event::Window {
            timestamp: 9,
            window_id,
            win_event: WindowEvent::FocusGained,
        },
        Event::KeyUp {
            timestamp: 8,
            window_id,
            keycode: Some(Keycode::F11),
            scancode: Some(Scancode::F11),
            keymod: Mod::NOMOD,
            repeat: false,
            which: 1,
            raw: 0,
        },
        Event::KeyDown {
            timestamp: 7,
            window_id,
            keycode: Some(Keycode::F11),
            scancode: Some(Scancode::F11),
            keymod: Mod::NOMOD,
            repeat: false,
            which: 1,
            raw: 0,
        },
        Event::Window {
            timestamp: 6,
            window_id,
            win_event: WindowEvent::Restored,
        },
        Event::Window {
            timestamp: 5,
            window_id,
            win_event: WindowEvent::Minimized,
        },
        Event::Window {
            timestamp: 4,
            window_id,
            win_event: WindowEvent::Resized(width, height),
        },
        Event::MouseMotion {
            timestamp: 4,
            window_id,
            which: 1,
            mousestate: MouseState::from_sdl_state(0),
            x: 0.0,
            y: 0.0,
            xrel: 4.0,
            yrel: -2.0,
        },
        Event::KeyUp {
            timestamp: 3,
            window_id,
            keycode: Some(Keycode::W),
            scancode: Some(Scancode::W),
            keymod: Mod::NOMOD,
            repeat: false,
            which: 1,
            raw: 0,
        },
        Event::KeyDown {
            timestamp: 2,
            window_id,
            keycode: Some(Keycode::W),
            scancode: Some(Scancode::W),
            keymod: Mod::NOMOD,
            repeat: false,
            which: 1,
            raw: 0,
        },
        Event::Window {
            timestamp: 1,
            window_id,
            win_event: WindowEvent::FocusLost,
        },
    ];
    for event in probe {
        event_subsystem.push_event(event).map_err(sdl_error)?;
    }
    Ok(())
}

pub(super) fn sort_key(event: &Event) -> (u64, u8, i64, i64) {
    let timestamp = event.get_timestamp();
    match event {
        Event::AppWillEnterBackground { .. } | Event::AppDidEnterBackground { .. } => {
            (timestamp, 0, 0, 0)
        }
        Event::AppWillEnterForeground { .. } | Event::AppDidEnterForeground { .. } => {
            (timestamp, 1, 0, 0)
        }
        Event::Window { win_event, .. } => {
            let (rank, first, second) = window_sort_key(*win_event);
            (timestamp, rank, first, second)
        }
        Event::KeyDown {
            scancode, which, ..
        } => (
            timestamp,
            40,
            scancode.map_or(-1, |value| i64::from(value as i32)),
            i64::from(*which),
        ),
        Event::KeyUp {
            scancode, which, ..
        } => (
            timestamp,
            41,
            scancode.map_or(-1, |value| i64::from(value as i32)),
            i64::from(*which),
        ),
        Event::MouseMotion { which, .. } => (timestamp, 42, i64::from(*which), 0),
        Event::Quit { .. } | Event::AppTerminating { .. } => (timestamp, 50, 0, 0),
        _ => (timestamp, u8::MAX, 0, 0),
    }
}

const fn window_sort_key(event: WindowEvent) -> (u8, i64, i64) {
    match event {
        WindowEvent::Hidden | WindowEvent::Minimized | WindowEvent::Occluded => (10, 0, 0),
        WindowEvent::Exposed | WindowEvent::Restored | WindowEvent::Shown => (11, 0, 0),
        WindowEvent::FocusLost => (12, 0, 0),
        WindowEvent::FocusGained => (13, 0, 0),
        WindowEvent::Resized(width, height) | WindowEvent::PixelSizeChanged(width, height) => {
            (14, width as i64, height as i64)
        }
        WindowEvent::CloseRequested => (15, 0, 0),
        _ => (39, 0, 0),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn callback_batch_sorts_by_sample_before_translation() {
        let mut events = [
            Event::Window {
                timestamp: 3,
                window_id: 1,
                win_event: WindowEvent::FocusGained,
            },
            Event::Window {
                timestamp: 1,
                window_id: 1,
                win_event: WindowEvent::FocusLost,
            },
            Event::Window {
                timestamp: 2,
                window_id: 1,
                win_event: WindowEvent::Resized(960, 540),
            },
        ];
        events.sort_by_key(sort_key);
        assert_eq!(events.map(|event| event.get_timestamp()), [1_u64, 2, 3]);
    }
}
