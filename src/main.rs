use std::borrow::BorrowMut;
use std::env;
use std::ffi::{c_ulong, CString};
use std::mem::size_of;
use std::ptr::null_mut;
use windows::{core, s, w, Win32::System::Registry::*};
use windows::Win32::Foundation::{NO_ERROR, WIN32_ERROR};


fn main() -> Result<(), WIN32_ERROR> {
    let args: Vec<_> = env::args().collect();
    args.clone().into_iter().for_each(|y| println!("{y}"));
    let mut new_value = String::from("Computer\\HKEY_LOCAL_MACHINE\\SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion\\Winlogon");
    if args.len() > 1 {
        new_value = args[1].clone();
    }

    //Testen ob das ganze ein valider pfad ist, sonst kaputt???

    let hivekey: HKEY = HKEY_CURRENT_USER;
    unsafe {
        let mut phkresult = HKEY::default();
        let status = RegOpenKeyExW(hivekey, w!("Software\\Microsoft\\Windows\\CurrentVersion\\Applets\\Regedit"), 0, KEY_ALL_ACCESS, &mut phkresult);
        if status != NO_ERROR {
            println!("Error opening RegKey {status:?}");
            return Err(status);
        }

        //Get Buffer Size
        let mut size: u32 = 0;
        let status = RegQueryValueExW(phkresult, w!("LastKey"), None, None, None, Some(&mut size));
        if status != NO_ERROR {
            println!("Error reading RegKey {status:?}");
            return Err(status);
        }
        println!("size: {size}");

        //Creating Vector in right size
        let mut buffer: Vec<u8> = Vec::with_capacity(size as usize);
        buffer.set_len(size as usize);

        let debug = buffer.capacity();
        println!("Bufferlenght = {debug:?}");

        //Query Value

        let status = RegQueryValueExW(phkresult, w!("LastKey"), None, None, Some(buffer.as_mut_ptr()), Some(&mut size));
        if status != NO_ERROR {
            println!("Error reading RegKey {status:?}");
            return Err(status);
        }
        println!("LPDATA -> {buffer:?}");
        //https://stackoverflow.com/a/57172592/14502777
        let debug: String = String::from_utf16_lossy(
            buffer.chunks_exact(2)
                .into_iter().map(|element| u16::from_ne_bytes([element[0], element[1]]))
                .collect::<Vec<u16>>()
                .as_slice());
        println!("Debug -> {debug}");

        //TESTDATA Computer\HKEY_LOCAL_MACHINE\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon
        //Computer\HKEY_CURRENT_USER\Software\Microsoft\Windows\CurrentVersion\Applets\Regedit



        // UTF-16 String erstellen -> Dann das Ganze vom auslesen wieder rückwärts.
        let mut u16string: Vec<u16> = new_value.encode_utf16().collect();
        u16string.push(0);
        let lpdata = u16string.into_iter().map( |e| e.to_ne_bytes()).flatten().collect::<Vec<u8>>();


        let status = RegSetValueExW(phkresult, w!("LastKey"), 0, REG_SZ, Some(lpdata.as_slice()));
        if status != NO_ERROR {
            println!("Error writing RegKey {status:?}");
            return Err(status);
        }

        let status = RegCloseKey(hivekey);
        if status != NO_ERROR {
            println!("Error closing RegKey {status:?}");
            return Err(status);
        } else {
            println!("Closed Key sucessfully");
        }
    }
    let regedit = std::process::Command::new("regedit.exe").spawn().expect("Unable to run regedit.exe");
    Ok(())
}