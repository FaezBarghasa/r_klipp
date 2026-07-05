use tokio::sync::mpsc::{Receiver, Sender};
use r_klipp_api::HostToMcu;
use std::time::{SystemTime, UNIX_EPOCH};

// FFI bindings for rknn_api
#[repr(C)]
pub struct RknnContext(u64);

#[link(name = "rknnrt")]
extern "C" {
    fn rknn_init(
        context: *mut RknnContext,
        model_path: *const std::os::raw::c_char,
        flag: u32,
        config: *mut std::ffi::c_void,
    ) -> i32;
    // Define other rknn functions here...
}

pub struct CvActor {
    raw_frame_rx: Receiver<Vec<u8>>,
    host_cmd_tx: Sender<HostToMcu>,
    rknn_context: RknnContext,
}

impl CvActor {
    pub fn new(
        raw_frame_rx: Receiver<Vec<u8>>,
        host_cmd_tx: Sender<HostToMcu>,
    ) -> Result<Self, anyhow::Error> {
        let mut rknn_context = RknnContext(0);
        let model_path = std::ffi::CString::new("/path/to/yolov8.rknn").unwrap();
        let ret = unsafe { rknn_init(&mut rknn_context, model_path.as_ptr(), 0, std::ptr::null_mut()) };
        if ret < 0 {
            return Err(anyhow::anyhow!("rknn_init failed with {}", ret));
        }

        Ok(Self {
            raw_frame_rx,
            host_cmd_tx,
            rknn_context,
        })
    }

    pub async fn run(&mut self) {
        while let Some(frame) = self.raw_frame_rx.recv().await {
            // Preprocess frame, run inference, postprocess
            let fault_detected = true; // Placeholder

            if fault_detected {
                let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_micros() as u64;
                // The actual E-Stop command would be more specific
                let _ = self.host_cmd_tx.send(HostToMcu::EmergencyStop).await;
            }
        }
    }
}
