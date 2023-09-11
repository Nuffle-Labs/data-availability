use std::{ffi::CStr, mem, slice};

use crate::{
    near::{config::Config, Client},
    Blob, DataAvailability, IndexRead, Read, SubmitResult,
};
use libc::size_t;
use near_da_primitives::{Commitment, Namespace, ShareVersion};
use near_primitives::types::BlockHeight;
use once_cell::sync::Lazy;
use tokio::runtime::{self, Runtime};

// TODO: this is unoptimal, ideally we use the same runtime as the main thread
static RUNTIME: Lazy<Runtime> = Lazy::new(|| {
    runtime::Builder::new_multi_thread()
        .enable_io()
        .enable_time()
        .build()
        .unwrap()
});

#[no_mangle]
pub extern "C" fn new_client(config: *const Config) -> *const Client {
    let config = unsafe {
        assert!(!config.is_null());

        &*config
    };

    Box::into_raw(Box::new(Client::new(config)))
}

#[no_mangle]
pub extern "C" fn free_client(client: *mut Client) {
    if !client.is_null() {
        unsafe {
            let _ = Box::from_raw(client);
        }
    }
}

#[no_mangle]
pub extern "C" fn submit(
    client: *const Client,
    blobs: *const Blob,
    len: size_t,
) -> *const SubmitResult {
    let client = unsafe {
        assert!(!client.is_null());

        &*client
    };
    let blobs = unsafe {
        assert!(!blobs.is_null());

        slice::from_raw_parts(blobs, len as usize)
    };
    match RUNTIME.block_on(client.submit(blobs)) {
        Ok(x) => Box::into_raw(Box::new(x)),
        Err(e) => {
            log::error!("submit failed: {}", e);
            std::ptr::null()
        }
    }
}

#[no_mangle]
pub extern "C" fn free_submit_result(result: *mut SubmitResult) {
    if !result.is_null() {
        unsafe {
            let _ = Box::from_raw(result);
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct BlobSafe {
    pub namespace: Namespace,
    pub commitment: Commitment,
    pub share_version: ShareVersion,
    pub data: *const u8,
    pub len: size_t,
}

impl From<Blob> for BlobSafe {
    fn from(blob: Blob) -> Self {
        let mut data = blob.data.clone();
        data.shrink_to_fit();

        let ptr = data.as_ptr();
        let len = data.len();
        mem::forget(data);

        let data = unsafe { slice::from_raw_parts(ptr, len) };
        println!("GET: RAW {:?}", data);

        Self {
            namespace: blob.namespace,
            commitment: blob.commitment,
            share_version: blob.share_version,
            data: ptr,
            len: len as size_t,
        }
    }
}

#[no_mangle]
pub extern "C" fn get(
    client: *const Client,
    namespace: *const u8,
    height: BlockHeight,
) -> *const BlobSafe {
    let client = unsafe {
        assert!(!client.is_null());

        &*client
    };
    let namespace = unsafe {
        assert!(!namespace.is_null());

        slice::from_raw_parts(namespace, 32)
    };
    match RUNTIME.block_on(client.get(namespace.try_into().unwrap(), height)) {
        Ok(x) => {
            let blob_safe: BlobSafe = x.0.into();
            println!("GET: {:?}", blob_safe);

            Box::into_raw(Box::new(blob_safe))
        }
        Err(e) => {
            log::error!("get failed: {}", e);
            std::ptr::null()
        }
    }
}

#[no_mangle]
pub extern "C" fn fast_get(
    client: *const Client,
    commitment: *const u8,
    len: size_t,
) -> *const BlobSafe {
    let client = unsafe {
        assert!(!client.is_null());

        &*client
    };
    let commitment = unsafe {
        assert!(!commitment.is_null());
        slice::from_raw_parts(commitment, len as usize)
    };
    match RUNTIME.block_on(client.fast_get(commitment.try_into().unwrap())) {
        Ok(x) => {
            println!("FAST_GET: {:?}", x);
            Box::into_raw(Box::new(x.0.into()))
        }
        Err(e) => {
            log::error!("fast_get failed: {}", e);
            std::ptr::null()
        }
    }
}

#[no_mangle]
pub extern "C" fn free_blob(blob: *mut BlobSafe) {
    if !blob.is_null() {
        unsafe {
            let _ = Box::from_raw(blob);
        }
    }
}

#[repr(C)]
pub struct GetAllResult {
    pub blobs: *const BlobSafe,
    pub len: size_t,
}

#[no_mangle]
pub extern "C" fn get_all(client: *const Client, namespace: *const u8) -> *const GetAllResult {
    let client = unsafe {
        assert!(!client.is_null());

        &*client
    };
    let namespace = unsafe {
        assert!(!namespace.is_null());

        slice::from_raw_parts(namespace, 32)
    };
    match RUNTIME.block_on(client.get_all(namespace.try_into().unwrap())) {
        Ok(x) => {
            let blobs = x.0.into_iter().map(|x| x.into()).collect::<Vec<_>>();
            println!("GET_ALL: {:?}", blobs);
            Box::into_raw(Box::new(GetAllResult {
                blobs: blobs.as_ptr(),
                len: blobs.len() as size_t,
            }))
        }
        Err(e) => {
            log::error!("get_all failed: {}", e);
            std::ptr::null()
        }
    }
}

#[cfg(test)]
pub mod test {
    use log::LevelFilter;

    use crate::near::config::Network;

    use super::*;

    const PREVIOUSLY_SUBMITTED_HEIGHT: u64 = 137391028;

    fn test_get_client() -> (Client, Config) {
        pretty_env_logger::formatted_builder()
            .filter_level(log::LevelFilter::Debug)
            .filter_module("near_jsonrpc_client", log::LevelFilter::Off)
            .filter_module("hyper", LevelFilter::Off)
            .filter_module("reqwest", LevelFilter::Off)
            .try_init()
            .ok();
        let config = Config {
            key_path: "throwaway-key.json".to_string().into(),
            contract: "throwawaykey.testnet".to_string().into(),
            network: Network::Testnet,
            namespace: "abc123".to_string(),
        };
        let client = Client::new(&config);
        (client, config)
    }

    #[test]
    fn test_init_client() {
        let (_, config) = test_get_client();
        assert!(!new_client(&config as *const Config).is_null());
    }

    #[test]
    fn c_submit() {
        let blobs = vec![Blob::new_v0([1_u8; 32], vec![0x01, 0x02, 0x03])];
        let (client, _) = test_get_client();
        let res = submit(&client, blobs.as_ptr(), blobs.len().into());
        assert!(!res.is_null());
        println!("{:?}", unsafe { &*res });
    }

    #[test]
    fn c_get() {
        let (client, _) = test_get_client();

        let res = get(&client, [1_u8; 32].as_ptr(), PREVIOUSLY_SUBMITTED_HEIGHT);
        assert!(!res.is_null());
        let blob: &BlobSafe = unsafe { &*res };
        let blob = blob.clone();
        println!("{:?}", blob);
        assert_eq!(blob.namespace, [1_u8; 32]);
        assert_eq!(blob.commitment, [0_u8; 32]);
        assert_eq!(blob.share_version, 0);
        assert_eq!(blob.len, 3);

        let data = unsafe { slice::from_raw_parts(blob.data, blob.len as usize) };
        assert_eq!(data, &vec![0x01, 0x02, 0x03]);
    }

    #[test]
    fn c_get_all() {
        let (client, _) = test_get_client();

        let res = get_all(&client, [1_u8; 32].as_ptr());
        assert!(!res.is_null());
        let blobs: &GetAllResult = unsafe { &*res };
        let blobs = unsafe { slice::from_raw_parts(blobs.blobs, blobs.len as usize) };
        println!("{:?}", blobs);
        assert_eq!(blobs.len(), 1);
        let blob = blobs[0].clone();
        assert_eq!(blob.namespace, [1_u8; 32]);
        assert_eq!(blob.commitment, [0_u8; 32]);
        assert_eq!(blob.share_version, 0);
        assert_eq!(blob.len, 3);
        let data = unsafe { slice::from_raw_parts(blob.data, blob.len as usize) };

        assert_eq!(data, &vec![0x01, 0x02, 0x03]);
    }

    #[test]
    fn c_fast_get() {
        let (client, _) = test_get_client();

        let res = fast_get(&client, [0_u8; 32].as_ptr(), 32);
        assert!(!res.is_null());
        let blob: &BlobSafe = unsafe { &*res };
        let blob = blob.clone();
        println!("{:?}", blob);
        assert_eq!(blob.namespace, [0_u8; 32]);
        assert_eq!(blob.commitment, [0_u8; 32]);
        assert_eq!(blob.share_version, 0);
        assert_eq!(blob.len, 3);
        let data = unsafe { slice::from_raw_parts(blob.data, blob.len as usize) };
        assert_eq!(data, &vec![0x01, 0x02, 0x03]);
    }

    #[test]
    fn test_blob_to_blobsafe() {
        let blob = Blob::new_v0([1_u8; 32], vec![0x01, 0x02, 0x03]);
        let blob_safe: BlobSafe = blob.into();
        assert_eq!(blob_safe.namespace, [1_u8; 32]);
        assert_eq!(blob_safe.commitment, [0_u8; 32]);
        assert_eq!(blob_safe.share_version, 0);
        assert_eq!(blob_safe.len, 3);
        let data = unsafe { slice::from_raw_parts(blob_safe.data, blob_safe.len as usize) };
        assert_eq!(data, &vec![0x01, 0x02, 0x03]);
    }
}
