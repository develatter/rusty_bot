
#[derive(PartialEq, Debug, Clone)]
pub enum ChatRole {
    Assistant,
    User,
}

#[derive(PartialEq, Debug, Clone)]
pub struct ChatMessage {
    pub(crate) role: ChatRole,
    pub(crate) content: String,
}
