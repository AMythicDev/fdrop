use crate::protocol::TransferType;

#[derive(Clone)]
pub enum DisplayContent {
    Text(String),
    DisplayFileTransfer(DisplayFileTransfer),
}

#[derive(Clone, serde::Serialize)]
pub struct DisplayFileTransfer {
    pub file_path: String,
    pub assoc_text: Option<String>,
}

#[derive(Clone, serde::Serialize)]
pub struct Transfer {
    pub ttype: TransferType,
    pub display_content: DisplayContent,
}

impl serde::Serialize for DisplayContent {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Self::Text(inner) => inner.serialize(serializer),
            Self::DisplayFileTransfer(inner) => inner.serialize(serializer),
        }
    }
}
