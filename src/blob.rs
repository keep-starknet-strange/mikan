use malachitebft_proto::Protobuf;
use prost::Name;
use serde::{Deserialize, Serialize};

use crate::block::blockproto;

#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize)]
pub struct Blob {
    pub app_id: Vec<u8>,
    pub data: Vec<u8>,
}

#[allow(dead_code)]
impl Blob {
    pub fn new(data: Vec<u8>, app_id: Vec<u8>) -> Self {
        Self { data, app_id }
    }
}

impl Name for blockproto::Blob {
    const NAME: &'static str = "Blob";
    const PACKAGE: &'static str = "mikan";

    fn full_name() -> String {
        "mikan.Blob".into()
    }

    fn type_url() -> String {
        "/mikan.Blob".into()
    }
}

impl Protobuf for Blob {
    type Proto = blockproto::Blob;

    fn from_proto(proto: Self::Proto) -> Result<Self, malachitebft_proto::Error> {
        let blob = Blob {
            app_id: proto.app_id,
            data: proto.data,
        };

        Ok(blob)
    }

    fn to_proto(&self) -> Result<Self::Proto, malachitebft_proto::Error> {
        let proto = blockproto::Blob {
            app_id: self.app_id.clone(),
            data: self.data.clone(),
        };

        Ok(proto)
    }
}
