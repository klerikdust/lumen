use std::{collections::HashMap, sync::{LazyLock, Mutex}};

use anyhow::Result;
use windows::{ApplicationModel::AppInfo, Win32::{Foundation::PROPERTYKEY, System::Com::CoTaskMemFree, UI::Shell::{IShellItem2, SHCreateItemFromParsingName}}};
use windows_core::{HSTRING, PCWSTR};

static NAME_CACHE: LazyLock<Mutex<HashMap<String, String>>> = LazyLock::new(|| Mutex::new(HashMap::new()));

const PKEY_SOFTWARE_PRODUCTNAME: PROPERTYKEY = PROPERTYKEY {
    fmtid: windows::core::GUID::from_u128(0x0CEF7D53_FA64_11D1_A203_0000F81FEDEE),
    pid: 7,
};

pub fn resolve_name_from_aumid(aumid: &str) -> String {
    if let Some(name) = NAME_CACHE.lock().unwrap().get(aumid) {
        return name.clone();
    }

    if let Ok(name) = get_display_name(aumid) { return name; }
    if let Ok(name) = get_win32_app_name(aumid) { return name; }

    aumid.to_string()
}

fn get_display_name(aumid: &str) -> Result<String> {
    let aumid_hstring = HSTRING::from(aumid);

    let app_info = AppInfo::GetFromAppUserModelId(&aumid_hstring)?;
    let display_info = app_info.DisplayInfo()?;

    let name = display_info.DisplayName()?.to_string();
    NAME_CACHE.lock().unwrap().insert(aumid.to_string(), name.clone());

    Ok(name)
}

fn get_win32_app_name(aumid: &str) -> Result<String> {
    let path = format!("shell:AppsFolder\\{aumid}");
    let path_hstring = HSTRING::from(&path);

    unsafe {
        let shell_item: IShellItem2 = SHCreateItemFromParsingName(
            PCWSTR(path_hstring.as_ptr()), 
            None
        )?;

        if let Ok(name) = shell_item.GetString(&PKEY_SOFTWARE_PRODUCTNAME) {
            let s = name.to_string()?;
            CoTaskMemFree(Some(name.as_ptr() as *const _));
            if !s.is_empty() {
                return Ok(s);
            }
        }

        let display_pwstr = shell_item
            .GetString(&windows::Win32::Storage::EnhancedStorage::PKEY_ItemNameDisplay)?;
        let display_name = display_pwstr.to_string().unwrap_or_default();
        CoTaskMemFree(Some(display_pwstr.as_ptr() as *const _));

        if display_name.is_empty() {
            anyhow::bail!("Empty display name");
        }

        NAME_CACHE.lock().unwrap().insert(aumid.to_string(), display_name.clone());

        Ok(display_name)
    }

}