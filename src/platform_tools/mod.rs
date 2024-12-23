use cocoa::appkit::NSCompositingOperation;
use cocoa::base::{id, nil};
use cocoa::foundation::{NSPoint, NSRect, NSSize, NSString};
use core_foundation::{
    base::TCFType,
    string::{CFString, CFStringRef},
};
use iced::Point;
use objc::runtime::{Class, Object};
use objc::{class, msg_send, sel, sel_impl};
use std::fs;

use tokio::task;

use crate::config::{WINDOW_HEIGHT, WINDOW_WIDTH};
use crate::storage::BrowserInfo;

pub async fn get_url_handlers() -> Vec<BrowserInfo> {
    let result = task::spawn_blocking(move || {
        let mut result = Vec::new();
        let app_dirs = ["/Applications"];

        for app_dir in app_dirs {
            if let Ok(entries) = fs::read_dir(app_dir) {
                for entry in entries {
                    if let Ok(entry) = entry {
                        let path = entry.path();
                        let converted_path = String::from(path.to_str().unwrap());
                        if path.extension().and_then(|s| s.to_str()) == Some("app") {
                            unsafe {
                                let workspace: id = msg_send![class!(NSWorkspace), sharedWorkspace];
                                if workspace == nil {
                                    continue;
                                }

                                let path_str = path.to_string_lossy();
                                let ns_path = NSString::alloc(nil).init_str(&path_str);
                                let bundle: id =
                                    msg_send![class!(NSBundle), bundleWithPath: ns_path];
                                if bundle == nil {
                                    continue;
                                }

                                let schemes_key = NSString::alloc(nil).init_str("CFBundleURLTypes");
                                let schemes: id =
                                    msg_send![bundle, objectForInfoDictionaryKey: schemes_key];

                                if schemes != nil {
                                    let count: usize = msg_send![schemes, count];
                                    let mut supports_http = false;

                                    for i in 0..count {
                                        let url_type: id = msg_send![schemes, objectAtIndex: i];
                                        let schemes_key =
                                            NSString::alloc(nil).init_str("CFBundleURLSchemes");
                                        let url_schemes: id =
                                            msg_send![url_type, objectForKey: schemes_key];

                                        if url_schemes != nil {
                                            let schemes_count: usize =
                                                msg_send![url_schemes, count];
                                            for j in 0..schemes_count {
                                                let scheme: id =
                                                    msg_send![url_schemes, objectAtIndex: j];
                                                let scheme_str: id = msg_send![scheme, UTF8String];
                                                if !scheme_str.is_null() {
                                                    let scheme_rust = std::ffi::CStr::from_ptr(
                                                        scheme_str as *const _,
                                                    )
                                                    .to_str()
                                                    .unwrap_or("");
                                                    if scheme_rust == "http"
                                                        || scheme_rust == "https"
                                                    {
                                                        supports_http = true;
                                                        break;
                                                    }
                                                }
                                            }
                                        }
                                        if supports_http {
                                            break;
                                        }
                                    }

                                    if supports_http {
                                        let file_manager: id =
                                            msg_send![class!(NSFileManager), defaultManager];
                                        if let Some(name) = get_app_name(file_manager, ns_path) {
                                            let icon_data = get_app_icon(&path_str);
                                            if !result
                                                .iter()
                                                .any(|app: &BrowserInfo| app.path == converted_path)
                                                && icon_data.is_some()
                                            {
                                                result.push(BrowserInfo {
                                                    name,
                                                    path: converted_path,
                                                    icon_data: icon_data.unwrap(),
                                                });
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        result
    })
    .await
    .unwrap_or_default();

    result
}

unsafe fn get_app_name(file_manager: id, path: id) -> Option<String> {
    if file_manager == nil {
        return None;
    }

    let display_name: id = msg_send![file_manager, displayNameAtPath: path];
    if display_name == nil {
        return None;
    }

    let name_str: id = msg_send![display_name, UTF8String];
    if name_str.is_null() {
        return None;
    }

    std::ffi::CStr::from_ptr(name_str as *const _)
        .to_str()
        .ok()
        .map(|s| s.to_owned())
}

fn get_app_icon(app_path: &str) -> Option<Vec<u8>> {
    unsafe {
        let workspace: id = msg_send![class!(NSWorkspace), sharedWorkspace];
        if workspace == nil {
            return None;
        }

        let path_str = NSString::alloc(nil).init_str(app_path);
        let icon: id = msg_send![workspace, iconForFile: path_str];

        if icon != nil {
            let small_image: id = msg_send![class!(NSImage), alloc];
            let small_image: id = msg_send![small_image, 
                initWithSize:NSSize::new(16.0, 16.0)];

            let _: () = msg_send![small_image, setResizingMode: 1];
            let _: () = msg_send![small_image, lockFocus];
            let icon_size: NSSize = msg_send![icon, size];
            let from_rect = NSRect::new(
                NSPoint::new(0.0, 0.0),
                NSSize::new(icon_size.width, icon_size.height),
            );
            let to_rect = NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(16.0, 16.0));

            let _: () = msg_send![icon,
                drawInRect:to_rect
                fromRect:from_rect
                operation:NSCompositingOperation::NSCompositeSourceOver
                fraction:1.0];

            let _: () = msg_send![small_image, unlockFocus];
            let tiff_data: id = msg_send![small_image, TIFFRepresentation];
            let bitmap_rep: id = msg_send![class!(NSBitmapImageRep), imageRepWithData:tiff_data];

            if bitmap_rep != nil {
                let png_data: id = msg_send![bitmap_rep, representationUsingType:4 properties:nil];
                if png_data != nil {
                    let length: usize = msg_send![png_data, length];
                    let bytes: *const u8 = msg_send![png_data, bytes];
                    if !bytes.is_null() {
                        return Some(std::slice::from_raw_parts(bytes, length).to_vec());
                    }
                }
            }
        }
        None
    }
}

pub fn set_as_default_browser() -> bool {
    unsafe {
        let bundle: id = msg_send![class!(NSBundle), mainBundle];
        let bundle_id: id = msg_send![bundle, bundleIdentifier];
        if bundle_id == nil {
            return false;
        }

        let bundle_id_str: id = msg_send![bundle_id, UTF8String];
        let cf_bundle_id = CFString::new(
            std::ffi::CStr::from_ptr(bundle_id_str as *const _)
                .to_str()
                .unwrap_or(""),
        );
        for scheme in ["http", "https"].iter() {
            let cf_scheme = CFString::new(scheme);
            let result: bool = LSSetDefaultHandlerForURLScheme(
                cf_scheme.as_concrete_TypeRef(),
                cf_bundle_id.as_concrete_TypeRef(),
            ) == 0;
            if !result {
                return false;
            }
        }
        true
    }
}

pub fn ensure_default_browser() -> bool {
    unsafe {
        let bundle: id = msg_send![class!(NSBundle), mainBundle];
        let bundle_id: id = msg_send![bundle, bundleIdentifier];
        if bundle_id == nil {
            return false;
        }

        let bundle_id_str: id = msg_send![bundle_id, UTF8String];
        let cf_bundle_id = CFString::new(
            std::ffi::CStr::from_ptr(bundle_id_str as *const _)
                .to_str()
                .unwrap_or(""),
        );

        for scheme in ["http", "https"].iter() {
            let cf_scheme = CFString::new(scheme);
            let handler = LSCopyDefaultHandlerForURLScheme(cf_scheme.as_concrete_TypeRef());
            if handler.is_null() {
                return false;
            }

            let current_handler = unsafe { CFString::wrap_under_create_rule(handler) };
            if current_handler.to_string() != cf_bundle_id.to_string() {
                return false;
            }
        }
        true
    }
}

pub fn get_mouse_position() -> Point {
    unsafe {
        let point: NSPoint = msg_send![class!(NSEvent), mouseLocation];
        let screen: id = msg_send![class!(NSScreen), mainScreen];
        let frame: NSRect = msg_send![screen, frame];

        let x = (point.x as f32).clamp(
            WINDOW_WIDTH / 2.0,
            frame.size.width as f32 - WINDOW_WIDTH / 2.0,
        ) - WINDOW_WIDTH / 2.0;

        let y = (frame.size.height - point.y) as f32;
        let y = y.clamp(
            WINDOW_HEIGHT / 2.0,
            frame.size.height as f32 - WINDOW_HEIGHT / 2.0,
        ) - WINDOW_HEIGHT / 2.0;

        Point::new(x, y)
    }
}

pub fn open_url(url: String, browser_path: String, profile: Option<String>) {
    if let Some(profile) = profile {
        std::process::Command::new("open")
            .arg("-na")
            .arg(browser_path.clone())
            .arg("--args")
            .arg(format!("--profile-directory={}", profile))
            .arg(url.clone())
            .spawn()
            .unwrap();
    } else {
        std::process::Command::new("open")
            .arg("-na")
            .arg(browser_path.clone())
            .arg(url.clone())
            .spawn()
            .unwrap();
    }
}

#[link(name = "CoreServices", kind = "framework")]
extern "C" {
    fn LSSetDefaultHandlerForURLScheme(scheme: CFStringRef, bundle_id: CFStringRef) -> i32;
    fn LSCopyDefaultHandlerForURLScheme(scheme: CFStringRef) -> CFStringRef;
}
