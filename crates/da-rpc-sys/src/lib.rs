use da_rpc::near::config;
pub use da_rpc::near::{config::Config, Client};
use da_rpc::CryptoHash;
use da_rpc::DataAvailability;
pub use da_rpc::Namespace;
pub use da_rpc::{Blob, FrameRef};
use ffi_helpers::error_handling::update_last_error;
use libc::size_t;
use once_cell::sync::Lazy;
use std::{
    ffi::{c_char, c_int, CStr, CString},
    mem, slice,
};
use tokio::runtime::{self, Runtime};

pub type BlockHeight = u64;
pub type Commitment = [u8; 32];
pub type ShareVersion = u32;

// Denote the version to make sure we don't break the ABI downstream
pub const VERSION: u8 = 2;

static RUNTIME: Lazy<Runtime> = Lazy::new(|| {
    runtime::Builder::new_multi_thread()
        .enable_io()
        .enable_time()
        .build()
        .unwrap()
});

#[no_mangle]
pub extern "C" fn get_error() -> *mut c_char {
    if ffi_helpers::error_handling::error_message().is_none() {
        return std::ptr::null_mut();
    }
    unsafe {
        let len = ffi_helpers::error_handling::last_error_length();
        let mut buf = vec![0; len as usize];
        ffi_helpers::error_handling::error_message_utf8(
            buf.as_mut_ptr() as *mut c_char,
            buf.len() as c_int,
        );
        let ptr = buf.as_mut_ptr();
        mem::forget(buf);
        ptr as *mut c_char
    }
}

#[no_mangle]
pub extern "C" fn clear_error() {
    ffi_helpers::error_handling::clear_last_error();
}

/// # Safety
/// We check if the pointers are null
#[no_mangle]
pub unsafe extern "C" fn new_client_file(
    key_path: *const c_char,
    contract: *const c_char,
    network: *const c_char,
    namespace_version: u8,
    namespace: u32,
) -> *const Client {
    let key_path = unsafe {
        assert!(!key_path.is_null());
        CStr::from_ptr(key_path)
    }
    .to_str()
    .unwrap()
    .to_string();

    let contract = unsafe {
        assert!(!contract.is_null());
        CStr::from_ptr(contract)
    }
    .to_str()
    .unwrap()
    .to_string();

    let network = unsafe {
        assert!(!network.is_null());
        CStr::from_ptr(network)
    }
    .to_str()
    .unwrap();

    let config = Config {
        key: config::KeyType::File(key_path.into()),
        contract,
        network: network.try_into().unwrap(),
        namespace: Namespace::new(namespace_version, namespace),
    };

    Box::into_raw(Box::new(Client::new(&config)))
}

/// # Safety
/// We check if the pointers are null
#[no_mangle]
pub unsafe extern "C" fn new_client(
    account_id: *const c_char,
    secret_key: *const c_char,
    contract: *const c_char,
    network: *const c_char,
    namespace_version: u8,
    namespace: u32,
) -> *const Client {
    let account_id = unsafe {
        assert!(!account_id.is_null());
        CStr::from_ptr(account_id)
    }
    .to_str()
    .unwrap()
    .to_string();

    let secret_key = unsafe {
        assert!(!secret_key.is_null());
        CStr::from_ptr(secret_key)
    }
    .to_str()
    .unwrap()
    .to_string();

    let contract = unsafe {
        assert!(!contract.is_null());
        CStr::from_ptr(contract)
    }
    .to_str()
    .unwrap()
    .to_string();

    let network = unsafe {
        assert!(!network.is_null());
        CStr::from_ptr(network)
    }
    .to_str()
    .unwrap();

    let config = Config {
        key: config::KeyType::SecretKey(account_id, secret_key),
        contract,
        network: network.try_into().unwrap(),
        namespace: Namespace::new(namespace_version, namespace),
    };

    Box::into_raw(Box::new(Client::new(&config)))
}

/// # Safety
/// We check if the client is null
#[no_mangle]
pub unsafe extern "C" fn free_client(client: *mut Client) {
    if !client.is_null() {
        unsafe {
            let _ = Box::from_raw(client);
        }
    }
}

/// # Safety
/// We check if the slices are null
#[no_mangle]
pub unsafe extern "C" fn submit(
    client: *const Client,
    blobs: *const BlobSafe,
    len: size_t,
) -> *mut c_char {
    let client = unsafe {
        assert!(!client.is_null());
        &*client
    };
    let blobs = unsafe {
        assert!(!blobs.is_null());
        slice::from_raw_parts(blobs, len)
    };
    let blobs = blobs
        .iter()
        .map(|blob| blob.clone().into())
        .collect::<Vec<Blob>>();
    match RUNTIME.block_on(client.submit(&blobs)) {
        Ok(x) => {
            let str = CString::new(x.0).unwrap();
            let ptr = str.into_raw();
            let char: *mut c_char = ptr as *mut c_char;

            char
        }
        Err(e) => {
            update_last_error(anyhow::anyhow!(e));
            std::ptr::null_mut()
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct BlobSafe {
    pub namespace_version: u8,
    pub namespace_id: u32,
    pub commitment: Commitment,
    pub share_version: ShareVersion,
    pub data: *const u8,
    pub len: size_t,
}

impl From<BlobSafe> for Blob {
    fn from(blob: BlobSafe) -> Self {
        let data = unsafe { slice::from_raw_parts(blob.data, blob.len) };

        Self {
            namespace: Namespace::new(blob.namespace_version, blob.namespace_id),
            commitment: blob.commitment,
            share_version: blob.share_version,
            data: data.to_vec(),
        }
    }
}
impl From<Blob> for BlobSafe {
    fn from(blob: Blob) -> Self {
        let (data, len) = vec_to_safe_ptr(blob.data);

        Self {
            namespace_id: blob.namespace.id,
            namespace_version: blob.namespace.version,
            commitment: blob.commitment,
            share_version: blob.share_version,
            data,
            len,
        }
    }
}

pub fn vec_to_safe_ptr<T>(vec: Vec<T>) -> (*const T, size_t) {
    let mut vec = vec;
    vec.shrink_to_fit();

    let ptr = vec.as_ptr();
    let len = vec.len();
    mem::forget(vec);

    (ptr, len as size_t)
}

#[repr(C)]
pub struct RustSafeArray {
    pub data: *const u8,
    pub len: size_t,
}

impl RustSafeArray {
    pub fn new(vec: Vec<u8>) -> Self {
        let (data, len) = vec_to_safe_ptr(vec);

        Self { data, len }
    }
}

/// # Safety
/// We check if the slices are null and they should always be 32 bytes
#[no_mangle]
pub unsafe extern "C" fn get(client: *const Client, transaction_id: *const u8) -> *const BlobSafe {
    let client = unsafe {
        assert!(!client.is_null());
        &*client
    };
    let transaction_id = unsafe {
        assert!(!transaction_id.is_null());
        slice::from_raw_parts(transaction_id, 32)
    };

    match RUNTIME.block_on(client.get(CryptoHash(transaction_id.try_into().unwrap()))) {
        Ok(x) => {
            let blob_safe: BlobSafe = x.0.into();
            println!("GET: {:?}", blob_safe);

            Box::into_raw(Box::new(blob_safe))
        }
        Err(e) => {
            update_last_error(anyhow::anyhow!(e));
            std::ptr::null()
        }
    }
}

/// # Safety
/// We check if the slices are null
#[no_mangle]
pub unsafe extern "C" fn free_blob(blob: *mut BlobSafe) {
    if !blob.is_null() {
        unsafe {
            let _ = Box::from_raw(blob);
        }
    }
}

/// # Safety
/// We check if the slices are null
#[no_mangle]
pub unsafe extern "C" fn submit_batch(
    client: *const Client,
    candidate_hex: *const c_char,
    tx_data: *const u8,
    tx_data_len: size_t,
) -> RustSafeArray {
    let client = unsafe {
        assert!(!client.is_null());

        &*client
    };
    let candidate_hex = unsafe {
        assert!(!candidate_hex.is_null());
        CStr::from_ptr(candidate_hex)
    }
    .to_str()
    .unwrap();

    let tx_data = {
        assert!(!tx_data.is_null());
        unsafe { slice::from_raw_parts(tx_data, tx_data_len) }
    };

    // If batcher inbox, submit the tx
    if candidate_hex == "0xfF00000000000000000000000000000000000000" {
        // Prepare the blob for submission
        // TODO: namespace versioning
        let blob = Blob::new_v0(client.config.namespace, tx_data.to_vec());
        let commitment = blob.commitment;
        match RUNTIME.block_on(client.submit(&[blob])) {
            Ok(result) => {
                let tx = result.0;
                let frame_ref = FrameRef::new(tx, commitment);
                RustSafeArray::new(frame_ref.to_celestia_format().to_vec())
            }
            Err(e) => {
                update_last_error(anyhow::anyhow!(e));
                RustSafeArray::new(vec![])
            }
        }
    } else {
        RustSafeArray::new(vec![])
    }
}

#[cfg(test)]
pub mod test {
    use super::*;
    use da_rpc::log::LevelFilter;
    use da_rpc::near::config::Network;
    use std::env;
    use std::ffi::CString;
    use std::str::FromStr;
    use ffi_helpers::Nullable;

    const PREVIOUSLY_SUBMITTED_TX: &str = "4YPsDMPsF35x6eWnBpFqrz1PC36tV3JdWwhTx6ZggEQo";

    #[test]
    fn test_error_handling() {
        update_last_error(anyhow::anyhow!("test"));
        let error = unsafe { &*get_error() };
        let err_str = unsafe { CStr::from_ptr(error).to_str().unwrap() };
        println!("{:?}", err_str);
        assert_eq!("test", err_str);

        let next_error = unsafe { &*get_error() };
        assert!(!next_error.is_null());
        clear_error();
        let cleared_error = unsafe { &*get_error() };
        assert!(cleared_error.is_null());
    }

    fn test_get_client() -> (Client, Config) {
        pretty_env_logger::formatted_builder()
            .filter_level(LevelFilter::Debug)
            .filter_module("near_jsonrpc_client", LevelFilter::Off)
            .filter_module("hyper", LevelFilter::Off)
            .filter_module("reqwest", LevelFilter::Off)
            .try_init()
            .ok();
        let account = env::var("TEST_NEAR_ACCOUNT").unwrap();
        let secret = env::var("TEST_NEAR_SECRET").unwrap();
        let config = Config {
            key: config::KeyType::SecretKey(account, secret),
            contract: "throwawaykey.testnet".to_string().into(),
            network: Network::Testnet,
            namespace: Namespace::default(),
        };
        let client = Client::new(&config);
        (client, config)
    }

    #[ignore = "This should be an integration test"]
    #[allow(temporary_cstring_as_ptr)] // JUSTIFICATION: it only lives in this scope, so it's fine
    #[test]
    fn test_init_client() {
        let (_, config) = test_get_client();
        assert!(unsafe {
            !new_client_file(
                CString::new("throwaway-key.json").unwrap().as_ptr(),
                CString::new(config.contract.to_string()).unwrap().as_ptr(),
                CString::new(config.network.to_string()).unwrap().as_ptr(),
                Namespace::default().version,
                Namespace::default().id,
            )
            .is_null()
        });
    }

    #[ignore = "This should be an integration test"]
    #[test]
    fn c_submit() {
        let blobs: Vec<BlobSafe> =
            vec![Blob::new_v0(Namespace::default(), vec![0x01, 0x02, 0x03]).into()];
        let (client, _) = test_get_client();
        let res = unsafe { submit(&client, blobs.as_ptr(), blobs.len().into()) };
        assert!(!res.is_null());
        let binding = unsafe { CString::from_raw(res) };
        let str = binding.to_str().unwrap();
        println!("{:?}", str);
    }

    #[ignore = "This should be an integration test"]
    #[test]
    fn c_submit_100kb() {
        let blobs: Vec<BlobSafe> =
            vec![Blob::new_v0(Namespace::default(), vec![99; 100000]).into()];
        let (client, _) = test_get_client();
        let res = unsafe { submit(&client, blobs.as_ptr(), blobs.len().into()) };
        assert!(!res.is_null());
        let binding = unsafe { CString::from_raw(res) };
        let str = binding.to_str().unwrap();
        println!("{:?}", str);
    }

    #[test]
    #[ignore = "Wait for integration tests"]
    fn c_get() {
        let (client, _) = test_get_client();

        let hash = CryptoHash::from_str(PREVIOUSLY_SUBMITTED_TX).unwrap();
        let ptr = hash.0.as_ptr();
        let ptr = ptr as *const u8;

        let res = unsafe { get(&client, ptr) };
        assert!(!res.is_null());
        let safe_blob: &BlobSafe = unsafe { &*res };
        let safe_blob = safe_blob.clone();
        println!("{:?}", safe_blob);
        assert_eq!(safe_blob.namespace_id, 55);
        assert_eq!(safe_blob.namespace_version, 0);
        assert_eq!(safe_blob.share_version, 0);
        assert_eq!(safe_blob.len, 706);
        assert_eq!(
            safe_blob.commitment,
            [
                140, 108, 21, 145, 178, 18, 82, 208, 152, 215, 28, 192, 41, 55, 132, 31, 182, 78,
                137, 220, 59, 101, 247, 22, 52, 226, 26, 194, 23, 139, 201, 228
            ]
        );

        let data = unsafe { slice::from_raw_parts(safe_blob.data, safe_blob.len as usize) };
        assert_eq!(data.len(), 706);
    }

    #[test]
    fn test_blob_to_blobsafe() {
        let blob = Blob::new_v0(Namespace::default(), vec![0x01, 0x02, 0x03]);
        let blob_safe: BlobSafe = blob.into();
        assert_eq!(blob_safe.namespace_id, 0);
        assert_eq!(blob_safe.namespace_version, 0);
        assert_eq!(
            blob_safe.commitment,
            [
                152, 207, 32, 36, 87, 1, 17, 6, 238, 3, 69, 178, 178, 181, 205, 35, 156, 227, 107,
                87, 153, 125, 67, 152, 97, 76, 3, 33, 17, 57, 223, 222
            ]
        );
        assert_eq!(blob_safe.share_version, 0);
        assert_eq!(blob_safe.len, 3);
        let data = unsafe { slice::from_raw_parts(blob_safe.data, blob_safe.len as usize) };
        assert_eq!(data, &vec![0x01, 0x02, 0x03]);
    }
}
