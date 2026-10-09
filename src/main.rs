pub(crate) mod graph;
pub(crate) mod heuristicas;
pub(crate) mod structs;

use std::time::Instant;

fn main() -> std::io::Result<()> {
    println!("--- Testando primeira execução (deve ler CSV e salvar rp.graph) ---");
    let inicio = Instant::now();
    let grafo = structs::Grafo::carregar()?;
    let tempo = inicio.elapsed();
    println!(
        "Grafo carregado com sucesso em {:?}! Nós: {}, Arestas: {}",
        tempo,
        grafo.total_nos(),
        grafo.total_arestas()
    );

    println!("\n--- Testando segunda execução (deve ler diretamente de rp.graph) ---");
    let inicio2 = Instant::now();
    let grafo2 = structs::Grafo::carregar_binario("rp.graph")?;
    let tempo2 = inicio2.elapsed();
    println!(
        "Grafo binário carregado em {:?}! Nós: {}, Arestas: {}",
        tempo2,
        grafo2.total_nos(),
        grafo2.total_arestas()
    );

    Ok(())
}
