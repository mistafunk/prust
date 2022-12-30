use std::ffi::c_char;
use prust_manual::prt;
use prust_manual::prt::LogHandler;

// note: these tests only work single threaded ATM

#[test]
fn test_default_log_handler() {
    let mut log_handler = Box::new(prt::DefaultLogHandler::default());
    prt::add_log_handler(&mut log_handler);
    prt::log("hello log", prt::LogLevel::LOG_INFO);
    prt::remove_log_handler(&mut log_handler);
}

#[test]
fn test_custom_log_handler() {
    #[derive(Default)]
    struct CustomLogHandler {
        captured_message: String,
    }

    impl LogHandler for CustomLogHandler {
        fn handle_log_event(&mut self, msg: &str) {
            self.captured_message += msg;
        }
    }

    let mut custom_log_handler = Box::new(CustomLogHandler::default());
    prt::add_log_handler(&mut custom_log_handler);
    prt::log("capture me", prt::LogLevel::LOG_INFO);
    prt::remove_log_handler(&mut custom_log_handler);
    assert_eq!("capture me", custom_log_handler.captured_message);
}