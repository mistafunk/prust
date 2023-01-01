use prust::prt;
use prust::prt::LogHandler;

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

#[test]
fn test_init() {
    let mut log_handler = Box::new(prt::DefaultLogHandler::default());
    prt::add_log_handler(&mut log_handler);

    let prt_context = prt::init(None, Some(prt::LogLevel::LOG_INFO));
    assert!(prt_context.is_ok());
}

#[test]
fn test_version() {
    let prt_version_res = prt::get_version();
    assert!(prt_version_res.is_ok());
    let prt_version = prt_version_res.ok().unwrap();
    assert_eq!(prt_version.version_major, 2);
    assert_eq!(prt_version.version_minor, 7);
    assert_eq!(prt_version.version_string, "2.7.8538");
}