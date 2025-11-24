use orbbec_sdk_sys::*;
use std::ffi::CStr;
use std::process;
use std::ptr;
use std::sync::mpsc;
use std::thread;
use std::time::Instant;

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

// Struct to replace the 'static' variables used in the C code.
// This holds the state for counting frames and measuring time.
struct StreamTracker {
    count: u64,
    last_timestamp: Instant,
}

impl StreamTracker {
    fn new() -> Self {
        Self {
            count: 0,
            last_timestamp: Instant::now(),
        }
    }

    fn update_and_print(&mut self, frame: *mut ob_frame, name: &str, error: &mut *mut ob_error) {
        unsafe {
            if frame.is_null() {
                return;
            }

            self.count += 1;
            let now = Instant::now();
            let duration = now.duration_since(self.last_timestamp);

            // Calculate frame rate every second (approx)
            if duration.as_secs_f64() >= 1.0 {
                let index = ob_frame_get_index(frame, error);
                check_ob_error(error);

                let width = ob_video_frame_get_width(frame, error);
                check_ob_error(error);

                let height = ob_video_frame_get_height(frame, error);
                check_ob_error(error);

                let frame_rate = self.count as f64 / duration.as_secs_f64();

                println!(
                    "{} frame index: {}, width: {}, height: {}, frame rate: {:.2}",
                    name, index, width, height, frame_rate
                );

                // Reset counter and timestamp
                self.count = 0;
                self.last_timestamp = now;
            }

            // Important: The frame obtained from get_frame must be released
            ob_delete_frame(frame, error);
            check_ob_error(error);
        }
    }
}

unsafe fn calculate_and_print_frame_rate(
    frameset: *mut ob_frame,
    color_tracker: &mut StreamTracker,
    depth_tracker: &mut StreamTracker,
) {
    unsafe {
        let mut error: *mut ob_error = ptr::null_mut();

        // 1. Process Color Frame
        let color_frame = ob_frameset_get_frame(frameset, OBFrameType_OB_FRAME_COLOR, &mut error);
        check_ob_error(&mut error);

        if !color_frame.is_null() {
            color_tracker.update_and_print(color_frame, "Color", &mut error);
        }

        // 2. Process Depth Frame
        let depth_frame = ob_frameset_get_frame(frameset, OBFrameType_OB_FRAME_DEPTH, &mut error);
        check_ob_error(&mut error);

        if !depth_frame.is_null() {
            depth_tracker.update_and_print(depth_frame, "Depth", &mut error);
        }
    }
}

fn main() {
    unsafe {
        let mut error: *mut ob_error = ptr::null_mut();

        // Create a pipeline to manage the streams
        let pipe = ob_create_pipeline(&mut error);
        check_ob_error(&mut error);

        // Start Pipeline with default configuration
        // (By default, it starts color and depth streams)
        ob_pipeline_start(pipe, &mut error);
        check_ob_error(&mut error);

        println!("Streams have been started.");
        println!("Press 'Enter' to stop the pipeline and exit the program.");

        // Setup native Rust channel for non-blocking input handling
        // This replaces the C 'kbhit' implementation.
        let (tx, rx) = mpsc::channel();
        thread::spawn(move || {
            let mut input = String::new();
            // This blocks the spawned thread, not the main thread
            let _ = std::io::stdin().read_line(&mut input);
            let _ = tx.send(());
        });

        // Initialize state trackers
        let mut color_tracker = StreamTracker::new();
        let mut depth_tracker = StreamTracker::new();

        loop {
            // Check if user pressed Enter (non-blocking)
            if rx.try_recv().is_ok() {
                println!("Stop requested...");
                break;
            }

            // Wait for frameset from pipeline, with a timeout of 100ms
            // (Reduced from 1000ms to allow the loop to check for key presses more often)
            let frameset = ob_pipeline_wait_for_frameset(pipe, 100, &mut error);
            check_ob_error(&mut error);

            if frameset.is_null() {
                continue;
            }

            calculate_and_print_frame_rate(frameset, &mut color_tracker, &mut depth_tracker);

            // Destroy the frameset (it wraps the individual frames)
            ob_delete_frame(frameset, &mut error);
            check_ob_error(&mut error);
        }

        // Stop Pipeline
        ob_delete_pipeline(pipe, &mut error);
        check_ob_error(&mut error);

        println!("Program exited successfully.");
    }
}
