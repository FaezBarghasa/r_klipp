
use gstreamer as gst;
use gstreamer_app as gst_app;

pub fn setup_camera_pipeline() -> Result<(), gst::glib::Error> {
    gst::init()?;

    let pipeline = gst::parse_launch(
        "v4l2src device=/dev/video0 ! videoconvert ! tee name=t \
         t. ! queue ! jpegenc ! multipartmux ! appsink name=mjpeg_sink \
         t. ! queue ! videoconvert ! video/x-raw,format=RGB ! appsink name=raw_sink"
    )?;

    let pipeline = pipeline.dynamic_cast::<gst::Pipeline>().unwrap();

    // Get the appsinks
    let mjpeg_sink = pipeline.by_name("mjpeg_sink").unwrap().dynamic_cast::<gst_app::AppSink>().unwrap();
    let raw_sink = pipeline.by_name("raw_sink").unwrap().dynamic_cast::<gst_app::AppSink>().unwrap();

    // Set callbacks or pull samples from the sinks here

    pipeline.set_state(gst::State::Playing)?;

    Ok(())
}
