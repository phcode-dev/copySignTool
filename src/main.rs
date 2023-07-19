use bitness::{self, Bitness};
use std::{env, path::{Path, PathBuf}};

use winreg::{
    enums::{HKEY_LOCAL_MACHINE, KEY_READ, KEY_WOW64_32KEY},
    RegKey,
};
use fs_extra::file::copy;
use fs_extra::file::CopyOptions;

mod utils;

fn main() {
    let path = locate_signtool().unwrap();

    println!("signtool path ={:?}", path.as_path());
    let options = CopyOptions {
        overwrite: true,
        buffer_size: 64000,
        skip_exist: false,
    };
    let exe_file_path =  env::current_exe().expect("unable to get path");
    let folder_location = exe_file_path.parent().expect("No parent folder");
    let source_file = folder_location.to_str().unwrap().to_owned() + "\\signtool.exe";


    println!("{:?}", source_file);
    let result = copy(source_file, path.as_path(), &options);
    println!("result = {:?}", result);
}

// sign code forked from https://github.com/forbjok/rust-codesign
fn locate_signtool() -> utils::Result<PathBuf> {
    const INSTALLED_ROOTS_REGKEY_PATH: &str = r"SOFTWARE\Microsoft\Windows Kits\Installed Roots";
    const KITS_ROOT_REGVALUE_NAME: &str = r"KitsRoot10";

    let installed_roots_key_path = Path::new(INSTALLED_ROOTS_REGKEY_PATH);

    // Open 32-bit HKLM "Installed Roots" key
    let installed_roots_key = RegKey::predef(HKEY_LOCAL_MACHINE)
        .open_subkey_with_flags(installed_roots_key_path, KEY_READ | KEY_WOW64_32KEY)
        .map_err(|_| utils::Error::OpenRegistry(INSTALLED_ROOTS_REGKEY_PATH.to_string()))?;

    // Get the Windows SDK root path
    let kits_root_10_path: String = installed_roots_key
        .get_value(KITS_ROOT_REGVALUE_NAME)
        .map_err(|_| utils::Error::GetRegistryValue(KITS_ROOT_REGVALUE_NAME.to_string()))?;

    // Construct Windows SDK bin path
    let kits_root_10_bin_path = Path::new(&kits_root_10_path).join("bin");

    let mut installed_kits: Vec<String> = installed_roots_key
        .enum_keys()
        /* Report and ignore errors, pass on values. */
        .filter_map(|res| match res {
            Ok(v) => Some(v),
            Err(_) => None,
        })
        .collect();

    // Sort installed kits
    installed_kits.sort();

    /* Iterate through installed kit version keys in reverse (from newest to oldest),
    adding their bin paths to the list.
    Windows SDK 10 v10.0.15063.468 and later will have their signtools located there. */
    let mut kit_bin_paths: Vec<PathBuf> = installed_kits
        .iter()
        .rev()
        .map(|kit| kits_root_10_bin_path.join(kit))
        .collect();

    /* Add kits root bin path.
    For Windows SDK 10 versions earlier than v10.0.15063.468, signtool will be located there. */
    kit_bin_paths.push(kits_root_10_bin_path);

    // Choose which version of SignTool to use based on OS bitness
    let arch_dir = match bitness::os_bitness().expect("failed to get os bitness") {
        Bitness::X86_32 => "x86",
        Bitness::X86_64 => "x64",
        _ => return Err(utils::Error::UnsupportedBitness),
    };

    /* Iterate through all bin paths, checking for existence of a SignTool executable. */
    for kit_bin_path in &kit_bin_paths {
        /* Construct SignTool path. */
        let signtool_path = kit_bin_path.join(arch_dir).join("signtool.exe");

        /* Check if SignTool exists at this location. */
        if signtool_path.exists() {
            // SignTool found. Return it.
            return Ok(signtool_path);
        }
    }

    Err(utils::Error::SignToolNotFound)
}