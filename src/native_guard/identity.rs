pub fn session_id() -> Result<String, String> {
    let mut bytes = [0u8; 16];
    random_bytes(&mut bytes)?;
    bytes[6] = (bytes[6] & 15) | 64;
    bytes[8] = (bytes[8] & 63) | 128;
    let hex: String = bytes.iter().map(|b| format!("{b:02x}")).collect();
    Ok(format!(
        "{}-{}-{}-{}-{}",
        &hex[..8],
        &hex[8..12],
        &hex[12..16],
        &hex[16..20],
        &hex[20..]
    ))
}

/// Operating-system CSPRNG bytes (DD-37).
pub(super) fn random_bytes(bytes: &mut [u8]) -> Result<(), String> {
    #[cfg(unix)]
    {
        use std::io::Read;
        std::fs::File::open("/dev/urandom")
            .and_then(|mut file| file.read_exact(bytes))
            .map_err(|e| e.to_string())
    }
    #[cfg(windows)]
    {
        use windows_sys::Win32::Security::Cryptography::{
            BCRYPT_USE_SYSTEM_PREFERRED_RNG, BCryptGenRandom,
        };
        let length = u32::try_from(bytes.len()).map_err(|_| "random request too large")?;
        let status = unsafe {
            BCryptGenRandom(
                std::ptr::null_mut(),
                bytes.as_mut_ptr(),
                length,
                BCRYPT_USE_SYSTEM_PREFERRED_RNG,
            )
        };
        if status < 0 {
            return Err(format!("system random generator failed: {status:#x}"));
        }
        Ok(())
    }
}
