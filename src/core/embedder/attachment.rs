use twilight_model::{
    channel::{embed::Embed, Attachment as ReceivedAttachment},
    http::attachment::Attachment,
};
use twilight_util::builder::embed::{EmbedBuilder, ImageSource};

pub struct AttachmentHandle {
    pub filename: String,
    pub content_type: Option<String>,
    pub url: String,
    pub proxy_url: String,
}

impl AttachmentHandle {
    pub fn into_attachment(self, id: u64) -> Attachment {
        Attachment::from_bytes(self.filename, b"hello".to_vec(), id)
    }

    pub fn from_attachment(attachment: &ReceivedAttachment) -> Self {
        Self {
            filename: attachment.filename.clone(),
            content_type: attachment.content_type.clone(),
            url: attachment.url.clone(),
            proxy_url: attachment.proxy_url.clone(),
        }
    }

    pub fn as_embed(&self) -> Option<Embed> {
        self.embedable_image()
            .map(|image| EmbedBuilder::new().image(image).validate().unwrap().build())
    }

    pub fn url_list_item(&self) -> String {
        self.proxy_url.clone()
    }

    pub fn embedable_image(&self) -> Option<ImageSource> {
        Some(ImageSource::url(&self.url).unwrap())
    }
}

pub trait VecAttachments {
    fn into_attachments(self) -> Vec<Attachment>;
}

impl VecAttachments for Vec<AttachmentHandle> {
    fn into_attachments(self) -> Vec<Attachment> {
        let mut attachments = Vec::new();
        for (current_id, attachment) in self.into_iter().enumerate() {
            attachments.push(attachment.into_attachment(current_id as u64));
        }
        attachments
    }
}
