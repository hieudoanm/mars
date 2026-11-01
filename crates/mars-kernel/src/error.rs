use mars_runtime::RuntimeError;

#[derive(Debug, thiserror::Error)]
pub enum KernelError {
    #[error("failed to initialize runtime")]
    Runtime(#[source] RuntimeError),
}
