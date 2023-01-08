use prust::prt;
use ctor::ctor;
use lazy_static::lazy_static;
use prust::prt::KeyOrUri;

// note: these tests only work single threaded ATM

lazy_static! {
    static ref PRT_CONTEXT: Box<prt::PrtContext> = {
        let init_result = prt::init(None, Some(prt::LogLevel::LOG_INFO));
        init_result.unwrap()
    };
}

#[ctor]
fn initialize() {
    let prt_context = &PRT_CONTEXT;
    println!("PRT has been initialized: {}", prt_context.as_ref());
}


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

    impl prt::LogHandler for CustomLogHandler {
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
fn test_generate() {
    let mut log_handler = Box::new(prt::DefaultLogHandler::default());
    prt::add_log_handler(&mut log_handler);

    let rule_file_uri = KeyOrUri::Uri(format!("rpk:file:{}/tests/extrude.rpk!/bin/extrude.cgb",
                                env!("CARGO_MANIFEST_DIR")));

    let initial_shape = Box::new(prt::InitialShapeBuilder::default()
        .vertex_coords(vec![0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 1.0, 1.0, 0.0, 1.0, 0.0, 0.0])
        .indices(vec![0, 1, 2, 3])
        .face_counts(vec![4])
        .rule_file(rule_file_uri)
        .start_rule("Default$Init".to_string())
        .random_seed(0)
        .name("rust_shape".to_string())
        .build().unwrap());

    let initial_shapes: Vec<Box<prt::InitialShape>> = vec![initial_shape];
    let encoders = vec!["com.esri.prt.codecs.OBJEncoder".to_string()];
    let encoder_options = vec![prt::EncoderOptions::default()];
    let mut callbacks = Box::new(prt::FileCallbacks::default());

    let generate_status = prt::generate(&initial_shapes, &encoders, &encoder_options,
                                        &mut callbacks);
    assert_eq!(generate_status, prt::Status::STATUS_OK);
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