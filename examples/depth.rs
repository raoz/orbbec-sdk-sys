use orbbec_sdk_sys::*;
use std::ffi::CStr;
use std::process;
use std::ptr;
use std::sync::mpsc;
use std::thread;

unsafe fn from_c_str(ptr: *const std::os::raw::c_char) -> String {
    if ptr.is_null() {
        return String::from("(null)");
    }
    unsafe { CStr::from_ptr(ptr) }
        .to_string_lossy()
        .into_owned()
}

// Helper function to check for errors and exit if there is one
unsafe fn check_ob_error(err: *mut *mut ob_error) {
    unsafe {
        if !(*err).is_null() {
            let msg_ptr = ob_error_get_message(*err);
            let error_message = from_c_str(msg_ptr);
            eprintln!("Error: {}", error_message);
            ob_delete_error(*err);
            process::exit(-1);
        }
        *err = ptr::null_mut();
    }
}

fn main() {
    unsafe {
        let mut error: *mut ob_error = ptr::null_mut();

        let pipeline = ob_create_pipeline(&mut error);
        check_ob_error(&mut error);

        let config = ob_create_config(&mut error);
        check_ob_error(&mut error);

        ob_config_enable_stream(config, OBStreamType_OB_STREAM_DEPTH, &mut error);
        check_ob_error(&mut error);

        ob_pipeline_start_with_config(pipeline, config, &mut error);
        check_ob_error(&mut error);

        println!("Depth Stream Started. Press 'Enter' key to exit the program.");

        // Setup non-blocking input listener (replaces ob_smpl_wait_for_key_press)
        let (tx, rx) = mpsc::channel();
        thread::spawn(move || {
            let mut input = String::new();
            let _ = std::io::stdin().read_line(&mut input);
            let _ = tx.send(());
        });

        // Wait frameset in a loop, exit when Enter is pressed.
        loop {
            // Check for exit signal
            if rx.try_recv().is_ok() {
                println!("Exiting...");
                break;
            }

            // Wait for a frameset, timeout after 1000 milliseconds.
            let frameset = ob_pipeline_wait_for_frameset(pipeline, 1000, &mut error);
            check_ob_error(&mut error);

            // If no frameset is available within the timeout, continue waiting.
            if frameset.is_null() {
                continue;
            }

            // Get the depth frame from frameset.
            let depth_frame = ob_frameset_get_depth_frame(frameset, &mut error);
            check_ob_error(&mut error);

            if !depth_frame.is_null() {
                // Get index from depth frame.
                let index = ob_frame_get_index(depth_frame, &mut error);
                check_ob_error(&mut error);

                // Print the distance of the center pixel every 30 frames to reduce output
                if index % 30 == 0 {
                    // Get the width of the depth frame.
                    let width = ob_video_frame_get_width(depth_frame, &mut error);
                    check_ob_error(&mut error);

                    // Get the height of the depth frame.
                    let height = ob_video_frame_get_height(depth_frame, &mut error);
                    check_ob_error(&mut error);

                    // Get the scale of the depth frame.
                    let scale = ob_depth_frame_get_value_scale(depth_frame, &mut error);
                    check_ob_error(&mut error);

                    // Get the data of the depth frame.
                    // The C code casts to uint16_t*. In Rust, we get a *mut c_void,
                    // so we cast it to *const u16.
                    let data_ptr = ob_frame_get_data(depth_frame, &mut error) as *const u16;
                    check_ob_error(&mut error);

                    let center_index = ((width * height) / 2 + (width / 2)) as isize;

                    let pixel_value = *data_ptr.offset(center_index);

                    // Pixel value multiplied by scale is the actual distance value in millimeters
                    let center_distance = (pixel_value as f32) * scale;

                    // Attention: if the distance is 0, it means that the depth camera
                    // cannot detect the object (may be out of detection range)
                    println!(
                        "Facing an object at a distance of {:.3} mm away.",
                        center_distance
                    );
                }

                // Delete the depth frame
                ob_delete_frame(depth_frame, &mut error);
                check_ob_error(&mut error);
            }

            // Delete the frameset
            ob_delete_frame(frameset, &mut error);
            check_ob_error(&mut error);
        }

        // Stop the pipeline
        ob_pipeline_stop(pipeline, &mut error);
        check_ob_error(&mut error);

        ob_delete_config(config, &mut error);
        check_ob_error(&mut error);

        ob_delete_pipeline(pipeline, &mut error);
        check_ob_error(&mut error);
    }
}
