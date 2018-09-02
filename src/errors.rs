error_chain! {
    foreign_links {
        JvmError(::j4rs::errors::J4RsError);
        IoError(::std::io::Error);
    }
}
