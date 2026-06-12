#[cfg(windows)]
use base64::Engine;
#[cfg(windows)]
use base64::engine::general_purpose::URL_SAFE_NO_PAD;

#[cfg(windows)]
const ENTROPY: &[u8] = b"NetStitch cloud local auth secret v1";

#[cfg(windows)]
pub(crate) fn protect_string(plaintext: &str) -> Result<String, String> {
    use windows_sys::Win32::Security::Cryptography::{
        CRYPT_INTEGER_BLOB, CRYPTPROTECT_UI_FORBIDDEN, CryptProtectData,
    };

    let mut input = plaintext.as_bytes().to_vec();
    let mut entropy = ENTROPY.to_vec();
    let mut input_blob = CRYPT_INTEGER_BLOB {
        cbData: input.len() as u32,
        pbData: input.as_mut_ptr(),
    };
    let entropy_blob = CRYPT_INTEGER_BLOB {
        cbData: entropy.len() as u32,
        pbData: entropy.as_mut_ptr(),
    };
    let mut output_blob = CRYPT_INTEGER_BLOB {
        cbData: 0,
        pbData: std::ptr::null_mut(),
    };

    let ok = unsafe {
        CryptProtectData(
            &mut input_blob,
            std::ptr::null(),
            &entropy_blob,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            CRYPTPROTECT_UI_FORBIDDEN,
            &mut output_blob,
        )
    };
    if ok == 0 {
        return Err("failed to protect cloud auth secret with Windows DPAPI".to_string());
    }

    let protected = unsafe {
        std::slice::from_raw_parts(output_blob.pbData, output_blob.cbData as usize).to_vec()
    };
    free_data_blob(output_blob);
    Ok(URL_SAFE_NO_PAD.encode(protected))
}

#[cfg(windows)]
pub(crate) fn unprotect_string(protected: &str) -> Result<String, String> {
    use windows_sys::Win32::Security::Cryptography::{
        CRYPT_INTEGER_BLOB, CRYPTPROTECT_UI_FORBIDDEN, CryptUnprotectData,
    };

    let mut input = URL_SAFE_NO_PAD
        .decode(protected.trim())
        .map_err(|_| "stored cloud auth secret is not valid base64url".to_string())?;
    let mut entropy = ENTROPY.to_vec();
    let mut input_blob = CRYPT_INTEGER_BLOB {
        cbData: input.len() as u32,
        pbData: input.as_mut_ptr(),
    };
    let entropy_blob = CRYPT_INTEGER_BLOB {
        cbData: entropy.len() as u32,
        pbData: entropy.as_mut_ptr(),
    };
    let mut output_blob = CRYPT_INTEGER_BLOB {
        cbData: 0,
        pbData: std::ptr::null_mut(),
    };

    let ok = unsafe {
        CryptUnprotectData(
            &mut input_blob,
            std::ptr::null_mut(),
            &entropy_blob,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            CRYPTPROTECT_UI_FORBIDDEN,
            &mut output_blob,
        )
    };
    if ok == 0 {
        return Err("failed to unprotect cloud auth secret with Windows DPAPI".to_string());
    }

    let plaintext = unsafe {
        std::slice::from_raw_parts(output_blob.pbData, output_blob.cbData as usize).to_vec()
    };
    free_data_blob(output_blob);
    String::from_utf8(plaintext)
        .map_err(|_| "stored cloud auth secret is not valid UTF-8".to_string())
}

#[cfg(windows)]
fn free_data_blob(blob: windows_sys::Win32::Security::Cryptography::CRYPT_INTEGER_BLOB) {
    if !blob.pbData.is_null() {
        unsafe {
            windows_sys::Win32::Foundation::LocalFree(blob.pbData.cast());
        }
    }
}

#[cfg(not(windows))]
pub(crate) fn protect_string(_plaintext: &str) -> Result<String, String> {
    Err("secure local cloud auth storage is available only on Windows".to_string())
}

#[cfg(not(windows))]
pub(crate) fn unprotect_string(_protected: &str) -> Result<String, String> {
    Err("secure local cloud auth storage is available only on Windows".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(windows)]
    #[test]
    fn dpapi_roundtrip_does_not_store_plaintext() {
        let protected = protect_string("CorrectHorseBatteryStaple")
            .expect("auth secret should be protected by DPAPI");

        assert_ne!(protected, "CorrectHorseBatteryStaple");
        assert!(!protected.contains("CorrectHorseBatteryStaple"));
        assert_eq!(
            unprotect_string(&protected).expect("auth secret should be unprotected by DPAPI"),
            "CorrectHorseBatteryStaple"
        );
    }
}
