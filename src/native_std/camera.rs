use crate::vm::Value;
use image::{ImageBuffer, Rgb};
use nokhwa::{
    Camera,
    pixel_format::RgbFormat,
    query,
    utils::{ApiBackend, CameraIndex, RequestedFormat, RequestedFormatType},
};
use std::collections::HashMap;

pub fn init() -> HashMap<String, Value> {
    let mut m = HashMap::new();

    // camera.devices()
    m.insert(
        "devices".into(),
        Value::NativeCallback(|_| {
            let cameras = query(ApiBackend::Auto).map_err(|e| e.to_string())?;

            let mut out = Vec::new();

            for cam in cameras {
                let mut map = HashMap::new();
                let index = cam.index().as_index().map_err(|e| e.to_string())?;

                map.insert("index".into(), Value::Int(index as i64));

                map.insert("name".into(), Value::String(cam.human_name().to_string()));

                map.insert(
                    "description".into(),
                    Value::String(cam.description().to_string()),
                );

                out.push(Value::Formula(map));
            }

            Ok(Value::Tuple(out))
        }),
    );

    // camera.capture(index, "photo.png")
    m.insert(
        "capture".into(),
        Value::NativeCallback(|args| {
            if args.len() != 2 {
                return Err("capture(index, path) expected".into());
            }

            let index = match &args[0] {
                Value::Int(v) => *v as u32,
                _ => return Err("index must be int".into()),
            };

            let path = match &args[1] {
                Value::String(s) => s.trim_matches('"').to_string(),
                v => v.to_string().trim_matches('"').to_string(),
            };

            let requested =
                RequestedFormat::new::<RgbFormat>(RequestedFormatType::AbsoluteHighestFrameRate);

            let mut camera =
                Camera::new(CameraIndex::Index(index), requested).map_err(|e| e.to_string())?;
            camera.open_stream().map_err(|e| e.to_string())?;

            let frame = camera.frame().map_err(|e| e.to_string())?;

            let rgb = frame
                .decode_image::<RgbFormat>()
                .map_err(|e| e.to_string())?;

            let img =
                ImageBuffer::<Rgb<u8>, _>::from_raw(rgb.width(), rgb.height(), rgb.into_raw())
                    .ok_or("Invalid image")?;

            img.save(path).map_err(|e| e.to_string())?;

            camera.stop_stream().ok();

            Ok(Value::Bool(true))
        }),
    );

    m
}
