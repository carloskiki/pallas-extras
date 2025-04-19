use std::error::Error;

use futures::executor::LocalPool;

fn main() -> Result<(), Box<dyn Error>> {
    let mut tp = LocalPool::new();
    
    Ok(())
}
