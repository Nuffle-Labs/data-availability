use anyhow::Context;
use da_rpc::near::config::{self, Network};
pub use da_rpc::near::{config::Config, Client};
use da_rpc::CryptoHash;
use da_rpc::DataAvailability;
pub use da_rpc::Namespace;
pub use da_rpc::{Blob, BlobRef};

use ffi_helpers::error_handling::update_last_error;
use ffi_helpers::null_pointer_check;
use ffi_helpers::Nullable;
use ffi_support::FfiStr;
use libc::size_t;
use once_cell::sync::Lazy;
use std::ptr::null;
use std::ptr::{null, null_mut};

use std::{
    ffi::{c_char, CStr, CString},
    mem, slice,
};
use tokio::runtime::{self, Runtime};

pub type BlockHeight = u64;

// Denote the version to make sure we don't break the API downstream
pub const VERSION: u8 = 4;

/// TODO: fix a lot of these panics since they arent handled well by ffi!

static RUNTIME: Lazy<Runtime> = Lazy::new(|| {
    runtime::Builder::new_multi_thread()
        .enable_io()
        .enable_time()
        .build()
        .expect("Failed to create runtime")
});

#[no_mangle]
pub extern "C" fn get_error() -> *mut c_char {
    let err = ffi_helpers::take_last_error();
    match err {
        None => std::ptr::null_mut(),
        Some(err) => {
            let msg = err.to_string();
            let mut buf = vec![0; msg.len() + 1];

            buf[..msg.len()].copy_from_slice(msg.as_bytes());
            // Make sure to add a trailing null in case people use this as a bare char*
            buf[msg.len()] = u8::NULL;

            let ptr = buf.as_mut_ptr();
            mem::forget(buf);
            ptr as *mut c_char
        }
    }
}

/// # Safety
/// We check if the pointers are null
/// This is only used in a test
#[no_mangle]
pub unsafe extern "C" fn set_error(err: *const c_char) {
    null_pointer_check!(err);
    let msg = FfiStr::from_raw(err).into_string();
    ffi_helpers::error_handling::update_last_error(anyhow::anyhow!(msg));
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
    null_pointer_check!(key_path);

    let key_path = FfiStr::from_raw(key_path).into_string();
    let key_type = || config::KeyType::File(key_path.into());
    init_client(contract, network, namespace_version, namespace, key_type)
}

unsafe fn init_client<F: FnOnce() -> config::KeyType>(
    contract: *const c_char,
    network: *const c_char,
    namespace_version: u8,
    namespace: u32,
    f: F,
) -> *const Client {
    null_pointer_check!(contract);
    null_pointer_check!(network);

    let contract = FfiStr::from_raw(contract).into_string();
    let network = FfiStr::from_raw(network).as_str();

    let namespace = if namespace > 0 {
        Some(Namespace::new(namespace_version, namespace))
    } else {
        None
    };

    let network = Network::try_from(network);

    match network {
        Err(e) => {
            update_last_error(anyhow::anyhow!(e));
            null()
        }
        Ok(network) => {
            let config = Config {
                key: f(),
                contract,
                network,
                namespace,
                mode: Default::default(), // TODO: for now we don't expose mode to the client
            };

            Box::into_raw(Box::new(Client::new(&config)))
        }
    }
}

/// # Safety
/// We check if the pointers are null
#[no_mangle]
pub unsafe extern "C" fn new_client(
    account_id: *const c_char,
    secret_key: *const c_char,
    contract: *const c_char,
    network: *const c_char,
    // TODO: make option
    namespace_version: u8,
    namespace: u32,
) -> *const Client {
    null_pointer_check!(account_id);
    null_pointer_check!(secret_key);

    let account_id = FfiStr::from_raw(account_id).into_string();
    let secret_key = FfiStr::from_raw(secret_key).into_string();

    let key_type = || config::KeyType::SecretKey(account_id, secret_key);
    init_client(contract, network, namespace_version, namespace, key_type)
}

/// # Safety
/// We check if the client is null
#[no_mangle]
pub unsafe extern "C" fn free_client(client: *mut Client) {
    null_pointer_check!(client);
    let _ = Box::from_raw(client);
}

/// # Safety
/// We check if the slices are null
#[no_mangle]
pub unsafe extern "C" fn submit(client: *const Client, blob: *const BlobSafe) -> *mut c_char {
    null_pointer_check!(client);
    null_pointer_check!(blob);

    let client = &*client;
    let blob = &*blob;
    let blob = slice::from_raw_parts(blob.data, blob.len);

    RUNTIME
        .block_on(client.submit(Blob::new(blob.to_vec())))
        .map_err(|e| anyhow::anyhow!(e))
        .and_then(|x| {
            let ptr = CString::new(x.0.transaction_id)
                .with_context(|| "failed to convert transaction id to C string")?
                .into_raw();
            Ok(ptr as *mut c_char)
        })
        .unwrap_or(null_mut())
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct BlobSafe {
    pub data: *const u8,
    pub len: size_t,
}

impl From<BlobSafe> for Blob {
    fn from(blob: BlobSafe) -> Self {
        let data = unsafe { slice::from_raw_parts(blob.data, blob.len) };

        Self {
            data: data.to_vec(),
        }
    }
}
impl From<Blob> for BlobSafe {
    fn from(blob: Blob) -> Self {
        let (data, len) = vec_to_safe_ptr(blob.data);
        Self { data, len }
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

impl Nullable for RustSafeArray {
    const NULL: Self = RustSafeArray {
        data: null(),
        len: 0,
    };

    fn is_null(&self) -> bool {
        unsafe { *self.data == *Self::NULL.data && self.len == 0 }
    }
}

/// # Safety
/// We check if the slices are null and they should always be 32 bytes
#[no_mangle]
pub unsafe extern "C" fn get(client: *const Client, transaction_id: *const u8) -> *const BlobSafe {
    null_pointer_check!(client);
    null_pointer_check!(transaction_id);

    let client = &*client;

    let transaction_id = slice::from_raw_parts(transaction_id, 32);
    let transaction_id: Result<[u8; 32], _> = transaction_id.try_into();
    match transaction_id {
        Ok(transaction_id) => scoop_err(
            RUNTIME
                .block_on(client.get(CryptoHash(transaction_id)))
                .map_err(|e| anyhow::anyhow!(e))
                .map(|x| x.0.into()),
        ),
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
    null_pointer_check!(blob);

    unsafe {
        let _ = Box::from_raw(blob);
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
) -> *const RustSafeArray {
    null_pointer_check!(client);
    null_pointer_check!(candidate_hex);
    null_pointer_check!(tx_data);

    let client = unsafe { &*client };
    let candidate_hex = unsafe { scoop_err(CStr::from_ptr(candidate_hex).to_str()) };
    null_pointer_check!(candidate_hex);
    let candidate_hex = *candidate_hex;
    let candidate_hex = candidate_hex.to_owned();
    let tx_data = { unsafe { slice::from_raw_parts(tx_data, tx_data_len) } };

    // TODO: this is too coupled to OP
    // If batcher inbox, submit the tx
    if candidate_hex == "0xfF00000000000000000000000000000000000000" {
        // Prepare the blob for submission
        // TODO: namespace versioning
        let blob = Blob::new(tx_data.to_vec());

        scoop_err(
            RUNTIME
                .block_on(client.submit(blob))
                .map(|result| result.0)
                .map(|r| RustSafeArray::new((*r).to_vec()))
                .map_err(|e| anyhow::anyhow!(e)),
        )
    } else {
        eprintln!("Not a batcher inbox");
        update_last_error(anyhow::anyhow!("Not a batcher inbox"));
        &RustSafeArray::NULL
    }
}

fn scoop_err<T, E: Into<anyhow::Error>>(result: Result<T, E>) -> *const T {
    match result {
        Err(e) => {
            let e = e.into();
            eprintln!("NEAR FFI: {:?}", e);
            update_last_error(e);
            std::ptr::null()
        }
        Ok(t) => Box::into_raw(Box::new(t)),
    }
}

#[cfg(test)]
pub mod test {
    use super::*;
    use da_rpc::near::config::Network;
    use ffi_helpers::take_last_error;
    use std::env;
    use std::ffi::CString;
    use std::str::FromStr;

    const PREVIOUSLY_SUBMITTED_TX: &str = "4YPsDMPsF35x6eWnBpFqrz1PC36tV3JdWwhTx6ZggEQo";

    #[test]
    fn test_error_handling() {
        update_last_error(anyhow::anyhow!("test"));
        let error = unsafe { &*get_error() };
        let err_str = unsafe { CStr::from_ptr(error).to_str().unwrap() };
        println!("{:?}", err_str);
        assert_eq!("test", err_str);
        assert!(take_last_error().is_none());
    }

    #[test]
    fn test_error_handling_manual_clear() {
        update_last_error(anyhow::anyhow!("test"));
        assert!(!get_error().is_null());
        clear_error();
        assert!(get_error().is_null());
    }

    fn test_get_client() -> (Client, Config) {
        pretty_env_logger::try_init().ok();
        let account = env::var("TEST_NEAR_ACCOUNT").unwrap();
        let secret = env::var("TEST_NEAR_SECRET").unwrap();
        let config = Config {
            key: config::KeyType::SecretKey(account.clone(), secret),
            contract: account.to_string(),
            network: Network::Testnet,
            namespace: None,
            mode: Default::default(),
        };
        let client = Client::new(&config);
        (client, config)
    }

    // #[ignore = "This should be an integration test"]
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
    fn c_e2e() {
        unsafe {
            let (client, _) = test_get_client();
            let original_blob = Blob::new(vec![0x01, 0x02, 0x03]);

            let res = submit(&client, &original_blob.clone().into());
            assert!(!res.is_null());

            let tx_hash = CString::from_raw(res);
            println!("{:?}", tx_hash);

            let fetched = Blob::from((*get(&client, tx_hash.as_ptr() as *const u8)).clone());

            assert_eq!(original_blob.data, fetched.data);
        }
    }

    #[ignore = "This should be an integration test"]
    #[test]
    fn c_submit() {
        let blob: BlobSafe = Blob::new(vec![0x01, 0x02, 0x03]).into();
        let (client, _) = test_get_client();
        let res = unsafe { submit(&client, &blob) };
        assert!(!res.is_null());
        let binding = unsafe { CString::from_raw(res) };
        let str = binding;
        println!("{:?}", str);
    }

    // #[ignore = "This should be an integration test"]
    #[test]
    fn c_submit_1point5mb() {
        let blob: BlobSafe = Blob::new(vec![99u8; 1536 * 1024]).into();
        let (client, _) = test_get_client();
        let res = unsafe { submit(&client, &blob) };

        if res.is_null() {
            let error = unsafe { &*get_error() };
            let err_str = unsafe { CStr::from_ptr(error).to_str().unwrap() };
            println!("{:?}", err_str);
            panic!("Should not be null");
        }
        let binding = unsafe { CString::from_raw(res) };
        let str = binding;
        println!("{:?}", str);
    }

    #[test]
    #[ignore = "Wait for integration tests"]
    fn c_get() {
        let (client, _) = test_get_client();

        let hash = CryptoHash::from_str(PREVIOUSLY_SUBMITTED_TX).unwrap();
        let ptr = hash.0.as_ptr();

        let res = unsafe { get(&client, ptr) };
        assert!(!res.is_null());
        let safe_blob: &BlobSafe = unsafe { &*res };
        let safe_blob = safe_blob.clone();
        println!("{:?}", safe_blob);
        assert_eq!(safe_blob.len, 706);
        let data = unsafe { slice::from_raw_parts(safe_blob.data, safe_blob.len as usize) };
        assert_eq!(data.len(), 706);
    }

    #[test]
    fn test_blob_to_blobsafe() {
        let blob = Blob::new(vec![0x01, 0x02, 0x03]);
        let blob_safe: BlobSafe = blob.into();
        assert_eq!(blob_safe.len, 3);
        let data = unsafe { slice::from_raw_parts(blob_safe.data, blob_safe.len) };
        assert_eq!(data, &vec![0x01, 0x02, 0x03]);
    }
}
