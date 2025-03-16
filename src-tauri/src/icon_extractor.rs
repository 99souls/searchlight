use std::os::windows::ffi::OsStrExt;
use std::path::Path;
use std::io;
use std::collections::HashMap;
use std::sync::Mutex;
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use winapi::um::shellapi::{SHGetFileInfoW, SHFILEINFOW, SHGFI_ICON, SHGFI_LARGEICON};
use winapi::um::winuser::{DestroyIcon, DrawIconEx};
use winapi::um::wingdi::{CreateCompatibleDC, DeleteDC};
use winapi::shared::windef::{HWND, HDC};
use winapi::ctypes::c_void;
use image::{ImageOutputFormat, RgbaImage};

extern "system" {
    fn GetDC(hWnd: HWND) -> HDC;
    fn ReleaseDC(hWnd: HWND, hdc: HDC) -> i32;
}

pub struct IconCache {
    cache: Mutex<HashMap<String, String>>,
}

impl IconCache {
    pub fn new() -> Self {
        Self {
            cache: Mutex::new(HashMap::new()),
        }
    }
    
    pub fn get_icon_data(&self, app_path: &str) -> Option<String> {
        {
            let cache = self.cache.lock().unwrap();
            if let Some(data) = cache.get(app_path) {
                return Some(data.clone());
            }
        }
        
        match self.extract_icon_as_base64(app_path) {
            Ok(data) => {
                let mut cache = self.cache.lock().unwrap();
                cache.insert(app_path.to_string(), data.clone());
                Some(data)
            },
            Err(e) => {
                eprintln!("Failed to extract icon for {}: {}", app_path, e);
                None
            }
        }
    }

    fn extract_icon_as_base64(&self, app_path: &str) -> io::Result<String> {
        unsafe {
            let wide_path: Vec<u16> = Path::new(app_path)
                .as_os_str()
                .encode_wide()
                .chain(Some(0))
                .collect();
                
            let mut file_info: SHFILEINFOW = std::mem::zeroed();
            
            SHGetFileInfoW(
                wide_path.as_ptr(),
                0,
                &mut file_info,
                std::mem::size_of::<SHFILEINFOW>() as u32,
                SHGFI_ICON | SHGFI_LARGEICON,
            );
            
            if file_info.hIcon.is_null() {
                return Err(io::Error::new(io::ErrorKind::Other, "Failed to get icon handle"));
            }
            
            let screen_dc = GetDC(std::ptr::null_mut());
            let hdc = CreateCompatibleDC(screen_dc);
            
            let bitmap_info = winapi::um::wingdi::BITMAPINFO {
                bmiHeader: winapi::um::wingdi::BITMAPINFOHEADER {
                    biSize: std::mem::size_of::<winapi::um::wingdi::BITMAPINFOHEADER>() as u32,
                    biWidth: 32,
                    biHeight: 32,
                    biPlanes: 1,
                    biBitCount: 32,
                    biCompression: winapi::um::wingdi::BI_RGB,
                    biSizeImage: 0,
                    biXPelsPerMeter: 0,
                    biYPelsPerMeter: 0,
                    biClrUsed: 0,
                    biClrImportant: 0,
                },
                bmiColors: [winapi::um::wingdi::RGBQUAD {
                    rgbBlue: 0,
                    rgbGreen: 0,
                    rgbRed: 0,
                    rgbReserved: 0,
                }],
            };
            
            let mut bits: *mut c_void = std::ptr::null_mut();
            let hbmp = winapi::um::wingdi::CreateDIBSection(
                hdc,
                &bitmap_info,
                winapi::um::wingdi::DIB_RGB_COLORS,
                &mut bits,
                std::ptr::null_mut(),
                0,
            );
            
            let old_bmp = winapi::um::wingdi::SelectObject(hdc, hbmp as _);
            
            DrawIconEx(
                hdc,
                0, 0, 
                file_info.hIcon,
                32, 32,
                0,
                std::ptr::null_mut(),
                winapi::um::winuser::DSS_NORMAL,
            );
            
            let data = std::slice::from_raw_parts(bits as *const u8, 32 * 32 * 4);
            let img = RgbaImage::from_raw(32, 32, data.to_vec())
                .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "Failed to create image"))?;
            
            let mut buffer = std::io::Cursor::new(Vec::new());
            img.write_to(&mut buffer, ImageOutputFormat::Png)
                .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("Image write error: {}", e)))?;
            
            let encoded = BASE64.encode(buffer.into_inner());
            let data_url = format!("data:image/png;base64,{}", encoded);
            
            winapi::um::wingdi::SelectObject(hdc, old_bmp);
            winapi::um::wingdi::DeleteObject(hbmp as _);
            DeleteDC(hdc);
            ReleaseDC(std::ptr::null_mut(), screen_dc);
            DestroyIcon(file_info.hIcon);
            
            Ok(data_url)
        }
    }
}
