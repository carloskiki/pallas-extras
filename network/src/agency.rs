/// Trait representing which part has agency in a protocol.
pub trait Agency {
    const SERVER: bool;
    type Inverse: Agency;
}

pub enum Client {}
impl Agency for Client {
    const SERVER: bool = false;
    type Inverse = Server;
}

pub enum Server {}
impl Agency for Server {
    const SERVER: bool = true;
    type Inverse = Client;
}
