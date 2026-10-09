// Charlie Tanfoll Pereira Lobo - 16827968
// Marina Cintra Queiroz - 17074404

pub(crate) mod graph;
pub(crate) mod heuristicas;
pub(crate) mod structs;

use std::io::Error;

use graph::*;

fn main() -> Result<(), Error> {

    let ribeirao_graph = Grafo::new()?;
    
    Ok(())
}
