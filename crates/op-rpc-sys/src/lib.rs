use ffi_helpers::error_handling::update_last_error;
use libc::{c_uint, size_t};
use once_cell::sync::Lazy;
pub use op_rpc::near::{config::Config, Client};
use op_rpc::DataAvailability;
pub use op_rpc::Namespace;
pub use op_rpc::{Blob, FrameRef, SubmitResult};
use std::{
    ffi::{c_char, c_int, CStr, CString},
    mem, slice,
};
use tokio::runtime::{self, Runtime};

pub type BlockHeight = u64;
pub type Commitment = [u8; 32];
pub type ShareVersion = u32;

pub const VERSION: u8 = 1;

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
        println!(
            "STRING: {:?}",
            CStr::from_ptr(buf.as_ptr() as *const c_char)
        );

        let ptr = buf.as_mut_ptr();
        mem::forget(buf);
        ptr as *mut c_char
    }
}

// TODO: this is unoptimal, ideally we use the same runtime as the main thread
static RUNTIME: Lazy<Runtime> = Lazy::new(|| {
    runtime::Builder::new_multi_thread()
        .enable_io()
        .enable_time()
        .build()
        .unwrap()
});

#[no_mangle]
pub extern "C" fn new_client(
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
        key_path: key_path.into(),
        contract,
        network: match network {
            "mainnet" => op_rpc::near::config::Network::Mainnet,
            "testnet" => op_rpc::near::config::Network::Testnet,
            "localnet" => op_rpc::near::config::Network::Localnet,
            _ => panic!("invalid network"),
        },
        namespace: Namespace::new(namespace_version, namespace),
    };

    Box::into_raw(Box::new(Client::new(&config)))
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
    blobs: *const BlobSafe,
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
    let blobs = blobs
        .into_iter()
        .map(|blob| blob.clone().into())
        .collect::<Vec<Blob>>();
    match RUNTIME.block_on(client.submit(&blobs)) {
        Ok(x) => Box::into_raw(Box::new(x)),
        Err(e) => {
            update_last_error(anyhow::anyhow!(e));
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
    pub namespace_version: u8,
    pub namespace_id: u32,
    pub commitment: Commitment,
    pub share_version: ShareVersion,
    pub data: *const u8,
    pub len: size_t,
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

impl Into<Blob> for BlobSafe {
    fn into(self) -> Blob {
        let data = unsafe { slice::from_raw_parts(self.data, self.len as usize) };
        Blob {
            namespace: Namespace::new(self.namespace_version, self.namespace_id),
            commitment: self.commitment,
            share_version: self.share_version,
            data: data.to_vec(),
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

#[no_mangle]
pub extern "C" fn get(client: *const Client, height: BlockHeight) -> *const BlobSafe {
    let client = unsafe {
        assert!(!client.is_null());

        &*client
    };
    match RUNTIME.block_on(client.get(&client.config.namespace, height)) {
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

#[no_mangle]
pub extern "C" fn fast_get(client: *const Client, commitment: *const u8) -> *const BlobSafe {
    let client = unsafe {
        assert!(!client.is_null());

        &*client
    };
    let commitment = unsafe {
        assert!(!commitment.is_null());
        slice::from_raw_parts(commitment, 32 as usize)
    };
    match RUNTIME.block_on(client.fast_get(commitment.try_into().unwrap())) {
        Ok(x) => {
            println!("FAST_GET: {:?}", x);
            Box::into_raw(Box::new(x.0.into()))
        }
        Err(e) => {
            update_last_error(anyhow::anyhow!(e));
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
    pub blob_len: size_t,
    pub heights: *const BlockHeight,
    pub heights_len: size_t,
}

#[no_mangle]
pub extern "C" fn get_all(client: *const Client) -> *const GetAllResult {
    let client = unsafe {
        assert!(!client.is_null());

        &*client
    };
    match RUNTIME.block_on(client.get_all(&client.config.namespace)) {
        Ok(x) => {
            let blobs =
                x.0.iter()
                    .map(|(_, blob)| blob.clone().into())
                    .collect::<Vec<_>>();
            println!("GET_ALL: {:?}", blobs);
            let (blobs_ptr, blobs_len) = vec_to_safe_ptr(blobs);
            let (heights, heights_len) = vec_to_safe_ptr(
                x.0.into_iter()
                    .map(|(height, _)| height)
                    .collect::<Vec<_>>(),
            );
            Box::into_raw(Box::new(GetAllResult {
                blobs: blobs_ptr,
                blob_len: blobs_len,
                heights,
                heights_len,
            }))
        }
        Err(e) => {
            update_last_error(anyhow::anyhow!(e));
            std::ptr::null()
        }
    }
}

#[no_mangle]
pub extern "C" fn submit_batch(
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

    let tx_data = unsafe {
        assert!(!tx_data.is_null());
        slice::from_raw_parts(tx_data, tx_data_len as usize)
    };

    // If batcher inbox, submit the tx
    if candidate_hex == "0xfF00000000000000000000000000000000000000" {
        // Prepare the blob for submission
        // TODO: namespace versioning
        let blob = Blob::new_v0(client.config.namespace, tx_data.to_vec());
        let commitment = blob.commitment.clone();
        match RUNTIME.block_on(client.submit(&vec![blob])) {
            Ok(result) => {
                let height = result.0;
                let frame_ref = FrameRef::new(height, commitment);
                RustSafeArray::new(frame_ref.to_celestia_format().to_vec())
            }
            Err(e) => {
                update_last_error(anyhow::anyhow!(e));
                RustSafeArray::new(vec![])
            }
        }
    } else {
        // TODO: handle this in c side, since empty array is 1 byte
        RustSafeArray::new(vec![])
    }
}

#[cfg(test)]
pub mod test {
    use super::*;
    use ffi_helpers::take_last_error;
    use op_rpc::log::LevelFilter;
    use op_rpc::near::config::Network;
    use std::ffi::CString;

    const PREVIOUSLY_SUBMITTED_HEIGHT: u64 = 137391028;

    #[test]
    fn test_error_handling() {
        unsafe {
            update_last_error(anyhow::anyhow!("test"));
            let error = unsafe { &*get_error() };
            let err_str = CStr::from_ptr(error).to_str().unwrap();
            println!("{:?}", err_str);
            assert_eq!("test", err_str);
            //assert!(take_last_error().is_some());
        }
    }

    fn test_get_client() -> (Client, Config) {
        pretty_env_logger::formatted_builder()
            .filter_level(LevelFilter::Debug)
            .filter_module("near_jsonrpc_client", LevelFilter::Off)
            .filter_module("hyper", LevelFilter::Off)
            .filter_module("reqwest", LevelFilter::Off)
            .try_init()
            .ok();
        let config = Config {
            key_path: "throwaway-key.json".to_string().into(),
            contract: "throwawaykey.testnet".to_string().into(),
            network: Network::Testnet,
            namespace: Namespace::default(),
        };
        let client = Client::new(&config);
        (client, config)
    }

    #[test]
    fn test_init_client() {
        let (_, config) = test_get_client();
        assert!(!new_client(
            CString::new(config.key_path.to_str().unwrap())
                .unwrap()
                .as_ptr(),
            CString::new(config.contract.to_string()).unwrap().as_ptr(),
            CString::new(config.network.to_string()).unwrap().as_ptr(),
            Namespace::default().version,
            Namespace::default().id,
        )
        .is_null());
    }

    #[test]
    fn c_submit() {
        let blobs: Vec<BlobSafe> =
            vec![Blob::new_v0(Namespace::default(), vec![0x01, 0x02, 0x03]).into()];
        let (client, _) = test_get_client();
        let res = submit(&client, blobs.as_ptr(), blobs.len().into());
        assert!(!res.is_null());
        println!("{:?}", unsafe { &*res });
    }

    #[test]
    fn c_get() {
        let (client, _) = test_get_client();

        let res = get(&client, PREVIOUSLY_SUBMITTED_HEIGHT);
        assert!(!res.is_null());
        let blob: &BlobSafe = unsafe { &*res };
        let blob = blob.clone();
        println!("{:?}", blob);
        assert_eq!(blob.namespace_id, 1);
        assert_eq!(blob.namespace_version, 1);
        assert_eq!(blob.commitment, [0_u8; 32]);
        assert_eq!(blob.share_version, 0);
        assert_eq!(blob.len, 3);

        let data = unsafe { slice::from_raw_parts(blob.data, blob.len as usize) };
        assert_eq!(data, &vec![0x01, 0x02, 0x03]);
    }

    #[test]
    fn c_get_all() {
        let (client, _) = test_get_client();

        let res = get_all(&client);
        assert!(!res.is_null());
        let blobs: &GetAllResult = unsafe { &*res };
        let blobs = unsafe { slice::from_raw_parts(blobs.blobs, blobs.blob_len as usize) };
        println!("{:?}", blobs);
        let blob = blobs[0].clone();
        assert_eq!(blob.namespace_id, 1);
        assert_eq!(blob.namespace_version, 1);
        assert_eq!(blob.commitment, [0_u8; 32]);
        assert_eq!(blob.share_version, 0);
        assert_eq!(blob.len, 3);
        let data = unsafe { slice::from_raw_parts(blob.data, blob.len as usize) };

        assert_eq!(data, &vec![0x01, 0x02, 0x03]);
    }

    #[test]
    fn c_fast_get() {
        let (client, _) = test_get_client();

        let res = fast_get(&client, [0_u8; 32].as_ptr());
        assert!(!res.is_null());
        let blob: &BlobSafe = unsafe { &*res };
        let blob = blob.clone();
        println!("{:?}", blob);
        assert_eq!(blob.namespace_id, 1);
        assert_eq!(blob.namespace_version, 1);
        assert_eq!(blob.commitment, [0_u8; 32]);
        assert_eq!(blob.share_version, 0);
        assert_eq!(blob.len, 3);
        let data = unsafe { slice::from_raw_parts(blob.data, blob.len as usize) };
        assert_eq!(data, &vec![0x01, 0x02, 0x03]);
    }

    #[test]
    fn test_live_fast_get() {
        let (client, _) = test_get_client();
        let commitment = [
            175, 84, 112, 64, 175, 148, 227, 31, 105, 18, 204, 218, 248, 104, 163, 0, 111, 45, 150,
            193, 84, 239, 131, 216, 90, 188, 202, 0, 117, 47, 78, 30,
        ];
        let res = fast_get(&client, commitment.as_ptr());
        println!("{:?}", res);
    }

    #[tokio::test]
    async fn test_live_get() {
        let (mut client, _) = test_get_client();
        // client.config.namespace = [
        //     0, 0, 8, 229, 246, 121, 191, 113, 22, 203, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        //     85, 0, 0, 0, 0, 0, 0, 0,
        // ];
        let res = client.get_all(&Namespace::new(1, 1)).await.unwrap();
        // let res = get(&client, 137935763);
        println!("{:?}", res);
    }

    #[test]
    fn test_blob_to_blobsafe() {
        let blob = Blob::new_v0(Namespace::default(), vec![0x01, 0x02, 0x03]);
        let blob_safe: BlobSafe = blob.into();
        assert_eq!(blob_safe.namespace_id, 1);
        assert_eq!(blob_safe.namespace_version, 1);
        assert_eq!(blob_safe.commitment, [0_u8; 32]);
        assert_eq!(blob_safe.share_version, 0);
        assert_eq!(blob_safe.len, 3);
        let data = unsafe { slice::from_raw_parts(blob_safe.data, blob_safe.len as usize) };
        assert_eq!(data, &vec![0x01, 0x02, 0x03]);
    }
}
