use windows::core::{IInspectable, Interface, Result, PCWSTR};
use windows::Foundation::TypedEventHandler;
use windows::Graphics::Capture::{Direct3D11CaptureFramePool, GraphicsCaptureItem};
use windows::Graphics::DirectX::DirectXPixelFormat;
use windows::Win32::Foundation::{HWND, RECT};
use windows::Win32::Graphics::Direct3D11::{
    ID3D11Resource, ID3D11Texture2D, D3D11_BIND_FLAG, D3D11_CPU_ACCESS_READ, D3D11_MAP_READ,
    D3D11_RESOURCE_MISC_FLAG, D3D11_TEXTURE2D_DESC, D3D11_USAGE_STAGING,
};
use windows::Win32::System::WinRT::{
    Graphics::Capture::IGraphicsCaptureItemInterop, RoInitialize, RO_INIT_SINGLETHREADED,
};
use windows::Win32::UI::WindowsAndMessaging::{GetWindowThreadProcessId, GetClassNameW, GetWindowTextW, FindWindowW, GetWindowRect};
use windows::core::Abi;
use windows::Graphics::DirectX::Direct3D11::IDirect3DDevice;
use windows::Win32::Graphics::{
    Direct3D::{D3D_DRIVER_TYPE, D3D_DRIVER_TYPE_HARDWARE, D3D_DRIVER_TYPE_WARP},
    Direct3D11::{
        D3D11CreateDevice, ID3D11Device, D3D11_CREATE_DEVICE_BGRA_SUPPORT,
        D3D11_CREATE_DEVICE_FLAG, D3D11_SDK_VERSION,
    },
    Dxgi::{IDXGIDevice, DXGI_ERROR_UNSUPPORTED},
};
use windows::Win32::System::WinRT::Direct3D11::{
    CreateDirect3D11DeviceFromDXGIDevice, IDirect3DDxgiInterfaceAccess,
};
use windows::Win32::Foundation::{BOOL, LPARAM};
use windows::Win32::Graphics::Dwm::{DwmGetWindowAttribute, DWMWA_CLOAKED, DWM_CLOAKED_SHELL};
use windows::Win32::System::Console::GetConsoleWindow;
use windows::Win32::UI::WindowsAndMessaging::{
    EnumWindows, GetAncestor, GetShellWindow, GetWindowLongW, IsWindowVisible, GA_ROOT,
    GWL_EXSTYLE, GWL_STYLE, WS_DISABLED, WS_EX_TOOLWINDOW,
};

use std::ffi::OsString;
use std::io::Write;
use std::sync::mpsc::channel;
use image::{DynamicImage, ImageBuffer, Bgra};

#[derive(Clone)]
pub struct WindowInfo {
    pub handle: HWND,
    pub title: String,
    pub class_name: String,
}

struct WindowEnumerationState {
    windows: Vec<WindowInfo>,
    console_window: Option<HWND>,
}

fn create_capture_item_for_window(window_handle: HWND) -> Result<GraphicsCaptureItem> {
    let interop = windows::core::factory::<GraphicsCaptureItem, IGraphicsCaptureItemInterop>()?;
    unsafe { interop.CreateForWindow(window_handle) }
}


pub fn capture(file_path: &str) -> bool {
    let target: &str = "umamusume";
    let id: HWND = unsafe { FindWindowW(PCWSTR::default(), OsString::from(target)) };
    let mut rect =  RECT { left: 0, top: 0, right: 0, bottom: 0 };
    let ret = unsafe { GetWindowRect(id, &mut rect) };

    if ret.as_bool() {
        unsafe { RoInitialize(RO_INIT_SINGLETHREADED).unwrap(); }

        let window = get_window_from_query(target).unwrap();
        let item = create_capture_item_for_window(window.handle).unwrap();
        take_screenshot(&item, &file_path).unwrap();

        return true;
    } else {
        return false;
    }
}

fn take_screenshot(item: &GraphicsCaptureItem, file_path: &str) -> Result<()> {
    let item_size = item.Size()?;

    let d3d_device = create_d3d_device()?;
    let d3d_context = unsafe {
        let mut d3d_context = None;
        d3d_device.GetImmediateContext(&mut d3d_context);
        d3d_context.unwrap()
    };
    let device = create_direct3d_device(&d3d_device)?;
    let frame_pool = Direct3D11CaptureFramePool::CreateFreeThreaded(
        &device,
        DirectXPixelFormat::B8G8R8A8UIntNormalized,
        1,
        &item_size,
    )?;
    let session = frame_pool.CreateCaptureSession(item)?;

    let (sender, receiver) = channel();
    frame_pool.FrameArrived(
        TypedEventHandler::<Direct3D11CaptureFramePool, IInspectable>::new({
            move |frame_pool, _| {
                let frame_pool = frame_pool.as_ref().unwrap();
                let frame = frame_pool.TryGetNextFrame()?;
                sender.send(frame).unwrap();
                Ok(())
            }
        }),
    )?;
    session.StartCapture()?;

    let texture = unsafe {
        let frame = receiver.recv().unwrap();

        let source_texture: ID3D11Texture2D =
            get_d3d_interface_from_object(&frame.Surface()?)?;
        let mut desc = D3D11_TEXTURE2D_DESC::default();
        source_texture.GetDesc(&mut desc);
        desc.BindFlags = D3D11_BIND_FLAG(0);
        desc.MiscFlags = D3D11_RESOURCE_MISC_FLAG(0);
        desc.Usage = D3D11_USAGE_STAGING;
        desc.CPUAccessFlags = D3D11_CPU_ACCESS_READ;
        let copy_texture = { d3d_device.CreateTexture2D(&desc, std::ptr::null())? };

        d3d_context.CopyResource(Some(copy_texture.cast()?), Some(source_texture.cast()?));

        session.Close()?;
        frame_pool.Close()?;

        copy_texture
    };

    let bits = unsafe {
        let mut desc = D3D11_TEXTURE2D_DESC::default();
        texture.GetDesc(&mut desc as *mut _);

        let resource: ID3D11Resource = texture.cast()?;
        let mapped = d3d_context.Map(Some(resource.clone()), 0, D3D11_MAP_READ, 0)?;

        // Get a slice of bytes
        let slice: &[u8] = {
            std::slice::from_raw_parts(
                mapped.pData as *const _,
                (desc.Height * mapped.RowPitch) as usize,
            )
        };

        let bytes_per_pixel = 4;
        let mut bits = vec![0u8; (desc.Width * desc.Height * bytes_per_pixel) as usize];
        for row in 0..desc.Height {
            let data_begin = (row * (desc.Width * bytes_per_pixel)) as usize;
            let data_end = ((row + 1) * (desc.Width * bytes_per_pixel)) as usize;
            let slice_begin = (row * mapped.RowPitch) as usize;
            let slice_end = slice_begin + (desc.Width * bytes_per_pixel) as usize;
            bits[data_begin..data_end].copy_from_slice(&slice[slice_begin..slice_end]);
        }

        d3d_context.Unmap(Some(resource), 0);

        bits
    };

    let image: ImageBuffer<Bgra<u8>, _> = ImageBuffer::from_raw(item_size.Width as u32, item_size.Height as u32, bits).unwrap();
    let dynamic_image = DynamicImage::ImageBgra8(image);
    let dynamic_image = dynamic_image.to_rgba8();
    dynamic_image.save(file_path).unwrap();

    Ok(())
}

fn get_window_from_query(query: &str) -> Result<WindowInfo> {
    let windows = find_window(query);
    let window = if windows.len() == 0 {
        println!("No window matching '{}' found!", query);
        std::process::exit(1);
    } else if windows.len() == 1 {
        &windows[0]
    } else {
        println!(
            "{} windows found matching '{}', please select one:",
            windows.len(),
            query
        );
        println!("    Num       PID    Window Title");
        for (i, window) in windows.iter().enumerate() {
            let mut pid = 0;
            unsafe { GetWindowThreadProcessId(window.handle, &mut pid) };
            println!("    {:>3}    {:>6}    {}", i, pid, window.title);
        }
        let index: usize;
        loop {
            print!("Please make a selection (q to quit): ");
            std::io::stdout().flush().unwrap();
            let mut input = String::new();
            std::io::stdin().read_line(&mut input).unwrap();
            if input.to_lowercase().contains("q") {
                std::process::exit(0);
            }
            let input = input.trim();
            let selection: Option<usize> = match input.parse::<usize>() {
                Ok(selection) => {
                    if selection < windows.len() {
                        Some(selection)
                    } else {
                        None
                    }
                }
                _ => None,
            };
            if let Some(selection) = selection {
                index = selection;
                break;
            } else {
                println!("Invalid input, '{}'!", input);
                continue;
            };
        }
        &windows[index]
    };

    Ok(window.clone())
}

fn find_window(window_name: &str) -> Vec<WindowInfo> {
    let window_list = enumerate_capturable_windows();
    let mut windows: Vec<WindowInfo> = Vec::new();
    for window_info in window_list.into_iter() {
        let title = window_info.title.to_lowercase();
        if title.contains(&window_name.to_string().to_lowercase()) {
            windows.push(window_info.clone());
        }
    }
    windows
}

impl WindowInfo {
    pub fn new(window_handle: HWND) -> Self {
        unsafe {
            let mut title = [0u16; 512];
            GetWindowTextW(window_handle, &mut title);
            let mut title = String::from_utf16_lossy(&title);
            truncate_to_first_null_char(&mut title);

            let mut class_name = [0u16; 512];
            GetClassNameW(window_handle, &mut class_name);
            let mut class_name = String::from_utf16_lossy(&class_name);
            truncate_to_first_null_char(&mut class_name);

            Self {
                handle: window_handle,
                title,
                class_name,
            }
        }
    }

    pub fn matches_title_and_class_name(&self, title: &str, class_name: &str) -> bool {
        self.title == title && self.class_name == class_name
    }
}

fn truncate_to_first_null_char(input: &mut String) {
    if let Some(index) = input.find('\0') {
        input.truncate(index);
    }
}


fn create_d3d_device_with_type(
    driver_type: D3D_DRIVER_TYPE,
    flags: D3D11_CREATE_DEVICE_FLAG,
    device: *mut Option<ID3D11Device>,
) -> Result<()> {
    unsafe {
        D3D11CreateDevice(
            None,
            driver_type,
            None,
            flags,
            &[],
            D3D11_SDK_VERSION as u32,
            device,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
        )
    }
}

fn create_d3d_device() -> Result<ID3D11Device> {
    let mut device = None;
    let mut result = create_d3d_device_with_type(
        D3D_DRIVER_TYPE_HARDWARE,
        D3D11_CREATE_DEVICE_BGRA_SUPPORT,
        &mut device,
    );
    if let Err(error) = &result {
        if error.code() == DXGI_ERROR_UNSUPPORTED {
            result = create_d3d_device_with_type(
                D3D_DRIVER_TYPE_WARP,
                D3D11_CREATE_DEVICE_BGRA_SUPPORT,
                &mut device,
            );
        }
    }
    result?;
    Ok(device.unwrap())
}

fn create_direct3d_device(d3d_device: &ID3D11Device) -> Result<IDirect3DDevice> {
    let dxgi_device: IDXGIDevice = d3d_device.cast()?;
    let inspectable = unsafe { CreateDirect3D11DeviceFromDXGIDevice(Some(dxgi_device))? };
    inspectable.cast()
}

fn get_d3d_interface_from_object<S: Interface, R: Interface + Abi>(object: &S) -> Result<R> {
    let access: IDirect3DDxgiInterfaceAccess = object.cast()?;
    let object = unsafe { access.GetInterface::<R>()? };
    Ok(object)
}


fn enumerate_capturable_windows() -> Vec<WindowInfo> {
    unsafe {
        // TODO: This works for Command Prompt but not Terminal
        let console_window = {
            let window_handle = GetConsoleWindow();
            if window_handle.0 == 0 {
                None
            } else {
                Some(window_handle)
            }
        };
        let state = Box::into_raw(Box::new(WindowEnumerationState {
            windows: Vec::new(),
            console_window,
        }));
        EnumWindows(Some(enum_window), LPARAM(state as isize));
        let state = Box::from_raw(state);
        state.windows
    }
}

extern "system" fn enum_window(window: HWND, state: LPARAM) -> BOOL {
    unsafe {
        let state = Box::leak(Box::from_raw(state.0 as *mut WindowEnumerationState));

        if let Some(console_window) = &state.console_window {
            if window == *console_window {
                return true.into();
            }
        }

        let window_info = WindowInfo::new(window);
        if window_info.is_capturable_window() {
            state.windows.push(window_info);
        }
    }
    true.into()
}

trait CaptureWindowCandidate {
    fn is_capturable_window(&self) -> bool;
}

impl CaptureWindowCandidate for WindowInfo {
    fn is_capturable_window(&self) -> bool {
        unsafe {
            if self.title.is_empty()
                || self.handle == GetShellWindow()
                || IsWindowVisible(self.handle).as_bool() == false
                || GetAncestor(self.handle, GA_ROOT) != self.handle
            {
                return false;
            }

            let style = GetWindowLongW(self.handle, GWL_STYLE);
            if style & (WS_DISABLED.0 as i32) == 1 {
                return false;
            }

            // No tooltips
            let ex_style = GetWindowLongW(self.handle, GWL_EXSTYLE);
            if ex_style & (WS_EX_TOOLWINDOW.0 as i32) == 1 {
                return false;
            }

            // Check to see if the self is cloaked if it's a UWP
            if self.class_name == "Windows.UI.Core.CoreWindow"
                || self.class_name == "ApplicationFrameWindow"
            {
                let mut cloaked: u32 = 0;
                if DwmGetWindowAttribute(
                    self.handle,
                    DWMWA_CLOAKED,
                    &mut cloaked as *mut _ as *mut _,
                    std::mem::size_of::<u32>() as u32,
                )
                .is_ok()
                    && cloaked == DWM_CLOAKED_SHELL
                {
                    return false;
                }
            }

            // Unfortunate work-around. Not sure how to avoid this.
            if is_known_blocked_window(self) {
                return false;
            }
        }
        true
    }
}

fn is_known_blocked_window(window_info: &WindowInfo) -> bool {
    // Task View
    window_info.matches_title_and_class_name("Task View", "Windows.UI.Core.CoreWindow") ||
    // XAML Islands
    window_info.matches_title_and_class_name("DesktopWindowXamlSource", "Windows.UI.Core.CoreWindow") ||
    // XAML Popups
    window_info.matches_title_and_class_name("PopupHost", "Xaml_WindowedPopupClass")
}
