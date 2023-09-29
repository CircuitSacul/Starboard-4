pub mod attachment_handle;
pub mod builder;
mod gifv;
mod handle;
pub mod image_only_embed;
mod imgur;
mod parser;
mod youtube;

pub use attachment_handle::AttachmentHandle;
pub use handle::Embedder;
