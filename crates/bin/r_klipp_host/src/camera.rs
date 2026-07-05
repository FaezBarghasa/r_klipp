use gstreamer as gst;
use gstreamer::prelude::*;
use tokio::sync::mpsc::Sender;

pub struct Camera {
    pipeline: gst::Pipeline,
}

impl Camera {
    pub fn new(raw_frame_tx: Sender<Vec<u8>>) -> Result<Self, anyhow::Error> {
        gst::init()?;
        let pipeline_str = "v4l2src device=/dev/video0 ! videoconvert ! tee name=t \
                            t. ! queue ! jpegenc ! appsink name=mjpeg_sink \
                            t. ! queue ! video/x-raw,format=RGB ! appsink name=raw_sink";
        let pipeline = gst::parse_launch(pipeline_str)?
            .downcast::<gst::Pipeline>()
            .expect("Failed to downcast to gst::Pipeline");

        let mjpeg_sink = pipeline
            .by_name("mjpeg_sink")
            .expect("mjpeg_sink not found")
            .downcast::<gst::AppSink>()
            .expect("Failed to downcast to AppSink");

        let raw_sink = pipeline
            .by_name("raw_sink")
            .expect("raw_sink not found")
            .downcast::<gst::AppSink>()
            .expect("Failed to downcast to AppSink");

        raw_sink.set_callbacks(
            gst::appsink::AppSinkCallbacks::builder()
                .new_sample(move |sink| {
                    let sample = sink.pull_sample().map_err(|_| gst::FlowError::Eos)?;
                    let buffer = sample.buffer().ok_or(gst::FlowError::Error)?;
                    let map = buffer.map_readable().map_err(|_| gst::FlowError::Error)?;
                    let _ = raw_frame_tx.try_send(map.as_slice().to_vec());
                    Ok(gst::FlowSuccess::Ok)
                })
                .build(),
        );

        Ok(Self { pipeline })
    }

    pub fn start(&self) -> Result<(), anyhow::Error> {
        self.pipeline.set_state(gst::State::Playing)?;
        Ok(())
    }
}
