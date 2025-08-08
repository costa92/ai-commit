pub mod feishu;
pub mod wechat;
pub mod dingtalk;
pub mod email;

pub use feishu::FeishuProvider;
pub use wechat::WeChatProvider;
pub use dingtalk::DingTalkProvider;
pub use email::{EmailProvider, EmailConfig, EmailAttachment, EmailPoolConfig, EmailTemplateConfig};