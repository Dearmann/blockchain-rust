use isahc::{ReadResponseExt, Request};
use spec::types::Block;

pub trait NodeClient {
    fn get_template_block(&self) -> Block;
    fn send_block(&self, block: &Block);
}

pub struct NetworkClient {
    pub node_url: String,
}

impl NetworkClient {
    pub fn new(node_url: String) -> Self {
        NetworkClient { node_url }
    }
}

impl NodeClient for NetworkClient {
    fn get_template_block(&self) -> Block {
        let uri = format!("{}/block_template", self.node_url);
        let mut response = isahc::get(uri).unwrap();

        // check if response is sucessful
        assert_eq!(response.status().as_u16(), 200);

        // parse block template from response
        let raw_body = response.text().unwrap();
        serde_json::from_str(&raw_body).unwrap()
    }

    fn send_block(&self, block: &Block) {
        let uri = format!("{}/blocks", self.node_url);
        let body = serde_json::to_string(block).unwrap();

        let request = Request::post(uri)
            .header("Content-Type", "application/json")
            .body(body)
            .unwrap();

        isahc::send(request).unwrap();
    }
}
