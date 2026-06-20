use std::{
    sync::{
        Arc, Mutex, OnceLock,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};

use i_slint_backend_winit::{
    WinitWindowAccessor,
    winit::raw_window_handle::{HasWindowHandle, RawWindowHandle},
};
use slint::ComponentHandle;
use windows::Win32::{
    Foundation::{HWND, RECT},
    Graphics::Gdi::{
        GetMonitorInfoW, MONITOR_DEFAULTTONEAREST, MONITORINFO, MonitorFromWindow, UpdateWindow,
    },
    UI::{
        HiDpi::GetDpiForWindow,
        Input::KeyboardAndMouse::{GetAsyncKeyState, VK_LBUTTON},
        WindowsAndMessaging::{
            GWL_EXSTYLE, GWL_STYLE, GetWindowLongPtrW, GetWindowRect, HWND_NOTOPMOST,
            HWND_TOPMOST, LWA_ALPHA, SW_HIDE, SW_SHOWNOACTIVATE, SWP_FRAMECHANGED, SWP_NOACTIVATE,
            SWP_NOMOVE, SWP_NOSIZE, SetLayeredWindowAttributes, SetWindowLongPtrW, SetWindowPos,
            ShowWindow, WS_EX_APPWINDOW, WS_EX_LAYERED, WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW,
            WS_POPUP,
        },
    },
};

use crate::{
    geometry::SHELL_WIDTH,
    platform::{
        clickthrough::set_clickthrough,
        cursor::{cursor_position, point_inside_pill},
        fullscreen::is_foreground_fullscreen,
    },
    state::{ContentState, IslandState},
};

static WINDOW_HWND: OnceLock<isize> = OnceLock::new();
const WINDOW_TOP_OFFSET: i32 = 40;

pub fn initialize_window<T>(
    component: &T,
    width: i32,
    height: i32,
    state: Arc<Mutex<IslandState>>,
    always_on_top: Arc<AtomicBool>,
    get_collapsed: impl Fn() -> bool + Send + 'static,
    handle_outside_click: impl FnMut() + Send + 'static,
) where
    T: ComponentHandle + 'static,
{
    let weak = component.as_weak();

    slint::Timer::single_shot(Duration::from_millis(200), move || {
        if let Some(component) = weak.upgrade() {
            with_hwnd(&component, |hwnd| unsafe {
                let topmost = always_on_top.load(Ordering::Relaxed);
                configure_window(hwnd, topmost);
                position_top_center(hwnd, width, height, topmost);

                WINDOW_HWND.set(hwnd.0 as isize).ok();
                set_clickthrough(hwnd, true);

                let _ = ShowWindow(hwnd, SW_SHOWNOACTIVATE);
                let _ = UpdateWindow(hwnd);

                start_clickthrough_loop(
                    hwnd,
                    width,
                    height,
                    state.clone(),
                    always_on_top.clone(),
                    get_collapsed,
                    handle_outside_click,
                );
            });
        }
    });
}

fn with_hwnd<T>(component: &T, f: impl FnOnce(HWND))
where
    T: ComponentHandle,
{
    component.window().with_winit_window(|w| {
        if let Ok(handle) = w.window_handle() {
            if let RawWindowHandle::Win32(h) = handle.as_raw() {
                let hwnd = HWND(h.hwnd.get() as *mut _);
                f(hwnd);
            }
        }
    });
}

unsafe fn configure_window(hwnd: HWND, always_on_top: bool) {
    let style = WS_POPUP.0 as isize;

    unsafe {
        SetWindowLongPtrW(hwnd, GWL_STYLE, style);

        let mut ex_style = GetWindowLongPtrW(hwnd, GWL_EXSTYLE);

        ex_style &= !(WS_EX_APPWINDOW.0 as isize);
        ex_style |= WS_EX_TOOLWINDOW.0 as isize;
        ex_style |= WS_EX_LAYERED.0 as isize;
        ex_style |= WS_EX_NOACTIVATE.0 as isize;

        SetWindowLongPtrW(hwnd, GWL_EXSTYLE, ex_style);

        let _ = SetLayeredWindowAttributes(
            hwnd,
            windows::Win32::Foundation::COLORREF(0),
            255,
            LWA_ALPHA,
        );

        SetWindowPos(
            hwnd,
            Some(window_level(always_on_top)),
            0,
            0,
            0,
            0,
            SWP_FRAMECHANGED | SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE,
        )
        .ok();
    }
}

unsafe fn position_top_center(hwnd: HWND, width: i32, height: i32, always_on_top: bool) {
    unsafe {
        let placement = top_center_placement(hwnd, width, height);

        SetWindowPos(
            hwnd,
            Some(window_level(always_on_top)),
            placement.x,
            placement.y,
            width,
            height,
            SWP_NOACTIVATE,
        )
        .ok();
    }
}

fn window_level(always_on_top: bool) -> HWND {
    if always_on_top { HWND_TOPMOST } else { HWND_NOTOPMOST }
}

struct WindowPlacement {
    x: i32,
    y: i32,
}

unsafe fn top_center_placement(hwnd: HWND, width: i32, _height: i32) -> WindowPlacement {
    unsafe {
        let monitor = MonitorFromWindow(hwnd, MONITOR_DEFAULTTONEAREST);
        let mut monitor_info = MONITORINFO {
            cbSize: std::mem::size_of::<MONITORINFO>() as u32,
            ..Default::default()
        };

        if GetMonitorInfoW(monitor, &mut monitor_info).as_bool() {
            let monitor_rect = monitor_info.rcMonitor;
            return WindowPlacement {
                x: monitor_rect.left + ((monitor_rect.right - monitor_rect.left) - width) / 2,
                y: monitor_rect.top + WINDOW_TOP_OFFSET,
            };
        }

        WindowPlacement { x: 0, y: WINDOW_TOP_OFFSET }
    }
}

unsafe fn enforce_top_center(
    hwnd: HWND,
    width: i32,
    height: i32,
    always_on_top: bool,
    force_level: bool,
) {
    unsafe {
        let placement = top_center_placement(hwnd, width, height);
        let mut rect = RECT::default();
        let moved = GetWindowRect(hwnd, &mut rect).is_ok()
            && (rect.left != placement.x
                || rect.top != placement.y
                || rect.right - rect.left != width
                || rect.bottom - rect.top != height);

        if moved || force_level {
            SetWindowPos(
                hwnd,
                Some(window_level(always_on_top)),
                placement.x,
                placement.y,
                width,
                height,
                SWP_NOACTIVATE,
            )
            .ok();
        }
    }
}

unsafe fn start_clickthrough_loop(
    hwnd: HWND,
    width: i32,
    height: i32,
    state: Arc<Mutex<IslandState>>,
    always_on_top: Arc<AtomicBool>,
    get_collapsed: impl Fn() -> bool + Send + 'static,
    mut handle_outside_click: impl FnMut() + Send + 'static,
) {
    let timer = Box::leak(Box::new(slint::Timer::default()));

    let mut clickthrough_enabled = true;
    let mut left_mouse_down = false;

    let mut hidden_for_fullscreen = false;
    let mut window_topmost = always_on_top.load(Ordering::Relaxed);

    timer.start(slint::TimerMode::Repeated, Duration::from_millis(16), move || {
        let fullscreen = is_foreground_fullscreen(hwnd);
        if fullscreen {
            if !hidden_for_fullscreen {
                let _ = unsafe { ShowWindow(hwnd, SW_HIDE) };
                hidden_for_fullscreen = true;
            }

            return;
        }

        if hidden_for_fullscreen {
            let _ = unsafe { ShowWindow(hwnd, SW_SHOWNOACTIVATE) };
            hidden_for_fullscreen = false;
        }

        let requested_topmost = always_on_top.load(Ordering::Relaxed);

        unsafe {
            enforce_top_center(
                hwnd,
                width,
                height,
                requested_topmost,
                requested_topmost || requested_topmost != window_topmost,
            );
        }

        let (mx, my) = cursor_position();

        let mut rect = RECT::default();

        unsafe {
            GetWindowRect(hwnd, &mut rect).ok();
        }

        let (logical, close_on_outside_click) = {
            let state = state.lock().unwrap();
            (
                state.clone().bounds(),
                state.expanded || matches!(state.content, ContentState::Notification(_)),
            )
        };
        let collapsed = get_collapsed();

        let dpi = unsafe { GetDpiForWindow(hwnd) };
        let scale_factor = dpi as f64 / 96.0;
        let bounds = logical.physical(scale_factor);

        let island_x = (SHELL_WIDTH - bounds.width) / 2;

        let island_left = rect.left + island_x;
        let island_top = rect.top
            + if collapsed {
                ((-(logical.height - 10)) as f64 * scale_factor).round() as i32
            } else {
                0
            };

        let px = mx - island_left;
        let py = my - island_top;

        let inside = point_inside_pill(px, py, bounds.width, bounds.height, bounds.radius);
        let mouse_down = unsafe { left_mouse_button_down() };

        if mouse_down && !left_mouse_down && !inside && close_on_outside_click {
            handle_outside_click();
        }
        left_mouse_down = mouse_down;

        unsafe {
            if inside && clickthrough_enabled {
                set_clickthrough(hwnd, false);
                clickthrough_enabled = false;
            }

            if !inside && !clickthrough_enabled {
                set_clickthrough(hwnd, true);
                clickthrough_enabled = true;
            }
        }

        window_topmost = requested_topmost;
    });
}

unsafe fn left_mouse_button_down() -> bool {
    unsafe { GetAsyncKeyState(VK_LBUTTON.0 as i32) < 0 }
}
